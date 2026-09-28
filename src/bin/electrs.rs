extern crate error_chain;
#[macro_use]
extern crate log;

extern crate dedoo_electrs;

use error_chain::ChainedError;
use std::process;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use dedoo_electrs::{
    config::Config,
    daemon::Daemon,
    electrum::RPC as ElectrumRPC,
    errors::*,
    grpc,
    metrics::Metrics,
    new_index::{precache, supply, ChainQuery, FetchFrom, Indexer, Mempool, Query, Store},
    rest,
    signal::Waiter,
};

#[cfg(feature = "liquid")]
use dedoo_electrs::elements::AssetRegistry;
use dedoo_electrs::metrics::MetricOpts;

fn fetch_from(config: &Config, store: &Store) -> FetchFrom {
    let mut jsonrpc_import = config.jsonrpc_import;
    if !jsonrpc_import {
        // switch over to jsonrpc after the initial sync is done
        jsonrpc_import = store.done_initial_sync();
    }

    if jsonrpc_import {
        // slower, uses JSONRPC (good for incremental updates)
        FetchFrom::Bitcoind
    } else {
        // faster, uses blk*.dat files (good for initial indexing)
        FetchFrom::BlkFiles
    }
}

fn run_server(config: Arc<Config>) -> Result<()> {
    let signal = Waiter::start();
    let metrics = Metrics::new(config.monitoring_addr);
    metrics.start();

    let daemon = Arc::new(Daemon::new(
        &config.daemon_dir,
        &config.blocks_dir,
        config.daemon_rpc_addr,
        config.cookie_getter(),
        config.network_type,
        signal.clone(),
        &metrics,
    )?);
    let store = Arc::new(Store::open(&config.db_path.join("newindex"), &config));
    supply::init(store.history_db());
    supply::spawn_bootstrap(Arc::clone(&store), Arc::clone(&daemon));
    let mut indexer = Indexer::open(
        Arc::clone(&store),
        fetch_from(&config, &store),
        &config,
        &metrics,
    );
    let mut tip = indexer.update(&daemon)?;

    let chain = Arc::new(ChainQuery::new(
        Arc::clone(&store),
        Arc::clone(&daemon),
        &config,
        &metrics,
    ));

    if let Some(ref precache_file) = config.precache_scripts {
        let precache_scripthashes = precache::scripthashes_from_file(precache_file.to_string())
            .expect("cannot load scripts to precache");
        precache::precache(&chain, precache_scripthashes);
    }

    let mempool = Arc::new(RwLock::new(Mempool::new(
        Arc::clone(&chain),
        &metrics,
        Arc::clone(&config),
    )));
    loop {
        match Mempool::update(&mempool, &daemon) {
            Ok(_) => break,
            Err(e) => {
                warn!("Error performing initial mempool update, trying again in 5 seconds: {}", e.display_chain());
                signal.wait(Duration::from_secs(5), false)?;
            },
        }
    }

    #[cfg(feature = "liquid")]
    let asset_db = config.asset_db_path.as_ref().map(|db_dir| {
        let asset_db = Arc::new(RwLock::new(AssetRegistry::new(db_dir.clone())));
        AssetRegistry::spawn_sync(asset_db.clone());
        asset_db
    });

    let query = Arc::new(Query::new(
        Arc::clone(&chain),
        Arc::clone(&mempool),
        Arc::clone(&daemon),
        Arc::clone(&config),
        #[cfg(feature = "liquid")]
        asset_db,
    ));

    // TODO: configuration for which servers to start
    let rest_server = rest::start(Arc::clone(&config), Arc::clone(&query));
    let electrum_server = ElectrumRPC::start(Arc::clone(&config), Arc::clone(&query), &metrics);

    // Start gRPC server if configured
    let grpc_server = config.grpc_addr.map(|addr| {
        info!("Starting gRPC server on {}", addr);
        
        // Create shared ranking manager
        let ranking = std::sync::Arc::new(grpc::RankingManager::new());
        
        // Start the gRPC server with the shared ranking manager
        let grpc_handle = grpc::start_grpc_server(
            Arc::clone(&config),
            Arc::clone(&query),
            addr,
            Arc::clone(&ranking),
        );
        
        // Start the background ranking indexer
        info!("Starting background wallet ranking indexer...");
        let indexer = grpc::RankingIndexer::new(
            Arc::clone(&store),
            Arc::clone(&chain),
            Arc::clone(&ranking),
            config.network_type,
        );
        let _indexer_handle = indexer.spawn();
        
        grpc_handle
    });

    let main_loop_count = metrics.gauge(MetricOpts::new(
        "dedoo_electrs_main_loop_count",
        "count of iterations of dedoo-electrs main loop each 5 seconds or after interrupts",
    ));

    loop {

        main_loop_count.inc();

        if let Err(err) = signal.wait(Duration::from_secs(5), true) {
            info!("stopping server: {}", err);
            rest_server.stop();
            if let Some(grpc) = grpc_server {
                grpc.stop();
            }
            // the electrum server is stopped when dropped
            break;
        }

        // Index new blocks
        let current_tip = daemon.getbestblockhash()?;
        if current_tip != tip {
            indexer.update(&daemon)?;
            tip = current_tip;
        };

        // Update mempool
        if let Err(e) = Mempool::update(&mempool, &daemon) {
            // Log the error if the result is an Err
            warn!("Error updating mempool, skipping mempool update: {}", e.display_chain());
        }

        // Update subscribed clients
        electrum_server.notify();
    }
    info!("server stopped");
    Ok(())
}

fn main() {
    let config = Arc::new(Config::from_args());
    if let Err(e) = run_server(config) {
        error!("server failed: {}", e.display_chain());
        process::exit(1);
    }
}
