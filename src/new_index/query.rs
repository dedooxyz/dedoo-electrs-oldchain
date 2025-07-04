use rayon::prelude::*;

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::{Duration, Instant};

use crate::chain::{Network, OutPoint, Transaction, TxOut, Txid};
use crate::config::Config;
use crate::daemon::Daemon;
use crate::errors::*;
use crate::new_index::{ChainQuery, Mempool, ScriptStats, SpendingInput, Utxo};
use crate::util::{is_spendable, BlockId, Bytes, TransactionStatus};



#[cfg(feature = "liquid")]
use crate::{
    chain::AssetId,
    elements::{lookup_asset, AssetRegistry, AssetSorting, LiquidAsset},
};

const FEE_ESTIMATES_TTL: u64 = 60; // seconds

const CONF_TARGETS: [u16; 28] = [
    1u16, 2u16, 3u16, 4u16, 5u16, 6u16, 7u16, 8u16, 9u16, 10u16, 11u16, 12u16, 13u16, 14u16, 15u16,
    16u16, 17u16, 18u16, 19u16, 20u16, 21u16, 22u16, 23u16, 24u16, 25u16, 144u16, 504u16, 1008u16,
];

pub struct Query {
    chain: Arc<ChainQuery>, // TODO: should be used as read-only
    mempool: Arc<RwLock<Mempool>>,
    daemon: Arc<Daemon>,
    config: Arc<Config>,
    cached_estimates: RwLock<(HashMap<u16, f64>, Option<Instant>)>,
    cached_relayfee: RwLock<Option<f64>>,
    #[cfg(feature = "liquid")]
    asset_db: Option<Arc<RwLock<AssetRegistry>>>,
}

impl Query {
    #[cfg(not(feature = "liquid"))]
    pub fn new(
        chain: Arc<ChainQuery>,
        mempool: Arc<RwLock<Mempool>>,
        daemon: Arc<Daemon>,
        config: Arc<Config>,
    ) -> Self {
        Query {
            chain,
            mempool,
            daemon,
            config,
            cached_estimates: RwLock::new((HashMap::new(), None)),
            cached_relayfee: RwLock::new(None),
        }
    }

    pub fn chain(&self) -> &ChainQuery {
        &self.chain
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn network(&self) -> Network {
        self.config.network_type
    }

    pub fn mempool(&self) -> RwLockReadGuard<Mempool> {
        self.mempool.read().unwrap()
    }

    pub fn broadcast_raw(&self, txhex: &str) -> Result<Txid> {
        let txid = self.daemon.broadcast_raw(txhex)?;
        self.mempool
            .write()
            .unwrap()
            .add_by_txid(&self.daemon, &txid);
        Ok(txid)
    }

    pub fn utxo(&self, scripthash: &[u8]) -> Result<Vec<Utxo>> {
        let mut utxos = self.chain.utxo(scripthash, self.config.utxos_limit)?;
        let mempool = self.mempool();
        utxos.retain(|utxo| !mempool.has_spend(&OutPoint::from(utxo)));
        utxos.extend(mempool.utxo(scripthash));
        Ok(utxos)
    }
    
    pub fn utxo_paginated(&self, scripthash: &[u8], start_index: usize, limit: usize) -> Result<(Vec<Utxo>, usize)> {
        // Get paginated UTXOs from the chain with the total count
        let (mut chain_utxos, total_chain_count) = self.chain.utxo_paginated(scripthash, start_index, limit)?;
        
        // Get mempool UTXOs
        let mempool = self.mempool();
        
        // Remove chain UTXOs that are spent in the mempool
        chain_utxos.retain(|utxo| !mempool.has_spend(&OutPoint::from(utxo)));
        
        // Get all mempool UTXOs for this scripthash
        let mempool_utxos = mempool.utxo(scripthash);
        
        // Calculate the total count (chain + mempool)
        let total_count = total_chain_count + mempool_utxos.len();
        
        // If we have fewer chain UTXOs than the limit after filtering, add some mempool UTXOs
        if chain_utxos.len() < limit {
            let remaining = limit - chain_utxos.len();
            
            // Add mempool UTXOs up to the remaining limit
            chain_utxos.extend(mempool_utxos.into_iter().take(remaining));
        }
        
        Ok((chain_utxos, total_count))
    }
    
    pub fn utxo_with_cursor(&self, scripthash: &[u8], cursor: Option<(Txid, u32)>, limit: usize) -> Result<(Vec<Utxo>, usize, Option<(Txid, u32)>)> {
        // Get UTXOs with cursor from the chain
        let (mut chain_utxos, total_chain_count, chain_next_cursor) = self.chain.utxo_with_cursor(scripthash, cursor, limit)?;
        
        // Handle mempool UTXOs
        let mempool = self.mempool();
        
        // Filter out spent outputs
        chain_utxos.retain(|utxo| !mempool.has_spend(&OutPoint::from(utxo)));
        
        // Get mempool UTXOs
        let mempool_utxos = mempool.utxo(scripthash);
        
        // Calculate total count including mempool UTXOs
        // Note: This is an approximation since we're not considering spent outputs in the total count
        let total_count = total_chain_count + mempool_utxos.len();
        
        // If we have space left after chain UTXOs and there's no more chain UTXOs, add mempool UTXOs
        let mut next_cursor = chain_next_cursor;
        
        if chain_utxos.len() < limit && next_cursor.is_none() {
            // We've exhausted chain UTXOs, so we need to add mempool UTXOs
            // Sort mempool UTXOs by txid and vout for consistent pagination
            let mut sorted_mempool: Vec<_> = mempool_utxos.into_iter().collect();
            sorted_mempool.sort_by(|a, b| {
                a.txid.cmp(&b.txid).then(a.vout.cmp(&b.vout))
            });
            
            // Find position after cursor in mempool UTXOs
            let mempool_start = if let Some((cursor_txid, cursor_vout)) = cursor {
                // If we have a cursor, find the position after it in mempool UTXOs
                sorted_mempool.iter().position(|utxo| {
                    utxo.txid > cursor_txid || (utxo.txid == cursor_txid && utxo.vout > cursor_vout)
                }).unwrap_or(0)
            } else {
                // No cursor, but we still have chain UTXOs, so skip mempool UTXOs for now
                if !chain_utxos.is_empty() {
                    sorted_mempool.len()
                } else {
                    0 // No chain UTXOs, start from beginning of mempool
                }
            };
            
            // Calculate how many mempool UTXOs we can add
            let remaining_slots = limit - chain_utxos.len();
            
            if mempool_start < sorted_mempool.len() {
                // Take one extra to determine if there are more results
                let take_count = std::cmp::min(remaining_slots + 1, sorted_mempool.len() - mempool_start);
                let mempool_page: Vec<_> = sorted_mempool.into_iter().skip(mempool_start).take(take_count).collect();
                
                // Determine if we need a next cursor from mempool UTXOs
                if mempool_page.len() > remaining_slots {
                    // We have more results, so provide a cursor for the next page
                    let last_utxo = &mempool_page[remaining_slots - 1];
                    next_cursor = Some((last_utxo.txid, last_utxo.vout));
                    
                    // Add only up to the limit
                    chain_utxos.extend(mempool_page.into_iter().take(remaining_slots));
                } else {
                    // Add all mempool UTXOs we found
                    chain_utxos.extend(mempool_page);
                }
            }
        }
        
        Ok((chain_utxos, total_count, next_cursor))
    }

    pub fn history_txids(&self, scripthash: &[u8], limit: usize) -> Vec<(Txid, Option<BlockId>)> {
        let confirmed_txids = self.chain.history_txids(scripthash, limit);
        let confirmed_len = confirmed_txids.len();
        let confirmed_txids = confirmed_txids.into_iter().map(|(tx, b)| (tx, Some(b)));

        let mempool_txids = self
            .mempool()
            .history_txids(scripthash, None, limit - confirmed_len)
            .into_iter()
            .map(|tx| (tx, None));

        confirmed_txids.chain(mempool_txids).collect()
    }

    pub fn stats(&self, scripthash: &[u8]) -> (ScriptStats, ScriptStats) {
        (
            self.chain.stats(scripthash),
            self.mempool().stats(scripthash),
        )
    }

    pub fn lookup_txn(&self, txid: &Txid) -> Option<Transaction> {
        self.chain
            .lookup_txn(txid, None)
            .or_else(|| self.mempool().lookup_txn(txid))
    }
    pub fn lookup_raw_txn(&self, txid: &Txid) -> Option<Bytes> {
        self.chain
            .lookup_raw_txn(txid, None)
            .or_else(|| self.mempool().lookup_raw_txn(txid))
    }

    pub fn lookup_txos(&self, outpoints: &BTreeSet<OutPoint>) -> HashMap<OutPoint, TxOut> {
        // the mempool lookup_txos() internally looks up confirmed txos as well
        self.mempool()
            .lookup_txos(outpoints)
            .expect("failed loading txos")
    }

    pub fn lookup_spend(&self, outpoint: &OutPoint) -> Option<SpendingInput> {
        self.chain
            .lookup_spend(outpoint)
            .or_else(|| self.mempool().lookup_spend(outpoint))
    }

    pub fn lookup_tx_spends(&self, tx: Transaction) -> Vec<Option<SpendingInput>> {
        let txid = tx.txid();

        tx.output
            .par_iter()
            .enumerate()
            .map(|(vout, txout)| {
                if is_spendable(txout) {
                    self.lookup_spend(&OutPoint {
                        txid,
                        vout: vout as u32,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_tx_status(&self, txid: &Txid) -> TransactionStatus {
        TransactionStatus::from(self.chain.tx_confirming_block(txid))
    }

    pub fn get_mempool_tx_fee(&self, txid: &Txid) -> Option<u64> {
        self.mempool().get_tx_fee(txid)
    }

    pub fn has_unconfirmed_parents(&self, txid: &Txid) -> bool {
        self.mempool().has_unconfirmed_parents(txid)
    }

    pub fn estimate_fee(&self, conf_target: u16) -> Option<f64> {
        if self.config.network_type.is_regtest() {
            return self.get_relayfee().ok();
        }
        if let (ref cache, Some(cache_time)) = *self.cached_estimates.read().unwrap() {
            if cache_time.elapsed() < Duration::from_secs(FEE_ESTIMATES_TTL) {
                return cache.get(&conf_target).copied();
            }
        }

        self.update_fee_estimates();
        self.cached_estimates
            .read()
            .unwrap()
            .0
            .get(&conf_target)
            .copied()
    }

    pub fn estimate_fee_map(&self) -> HashMap<u16, f64> {
        if let (ref cache, Some(cache_time)) = *self.cached_estimates.read().unwrap() {
            if cache_time.elapsed() < Duration::from_secs(FEE_ESTIMATES_TTL) {
                return cache.clone();
            }
        }

        self.update_fee_estimates();
        self.cached_estimates.read().unwrap().0.clone()
    }

    fn update_fee_estimates(&self) {
        match self.daemon.estimatesmartfee_batch(&CONF_TARGETS) {
            Ok(estimates) => {
                *self.cached_estimates.write().unwrap() = (estimates, Some(Instant::now()));
            }
            Err(err) => {
                warn!("failed estimating feerates: {:?}", err);
            }
        }
    }

    pub fn get_relayfee(&self) -> Result<f64> {
        if let Some(cached) = *self.cached_relayfee.read().unwrap() {
            return Ok(cached);
        }

        let relayfee = self.daemon.get_relayfee()?;
        self.cached_relayfee.write().unwrap().replace(relayfee);
        Ok(relayfee)
    }

    pub fn get_total_coin_supply(&self) -> Result<f64> {
        // Get the total coin supply directly from the daemon
        // This uses the gettxoutsetinfo RPC call which returns accurate information
        // about the current UTXO set, including the total amount of coins
        let txout_set_info = self.daemon.gettxoutsetinfo()?;

        // Return the total amount from the txoutsetinfo
        Ok(txout_set_info.total_amount)
    }


}

    #[cfg(feature = "liquid")]
    pub fn new(
        chain: Arc<ChainQuery>,
        mempool: Arc<RwLock<Mempool>>,
        daemon: Arc<Daemon>,
        config: Arc<Config>,
        asset_db: Option<Arc<RwLock<AssetRegistry>>>,
    ) -> Self {
        Query {
            chain,
            mempool,
            daemon,
            config,
            asset_db,
            cached_estimates: RwLock::new((HashMap::new(), None)),
            cached_relayfee: RwLock::new(None),
        }
    }

    #[cfg(feature = "liquid")]
    pub fn lookup_asset(&self, asset_id: &AssetId) -> Result<Option<LiquidAsset>> {
        lookup_asset(&self, self.asset_db.as_ref(), asset_id, None)
    }

    #[cfg(feature = "liquid")]
    pub fn list_registry_assets(
        &self,
        start_index: usize,
        limit: usize,
        sorting: AssetSorting,
    ) -> Result<(usize, Vec<LiquidAsset>)> {
        let asset_db = match &self.asset_db {
            None => return Ok((0, vec![])),
            Some(db) => db.read().unwrap(),
        };
        let (total_num, results) = asset_db.list(start_index, limit, sorting);
        // Attach on-chain information alongside the registry metadata
        let results = results
            .into_iter()
            .map(|(asset_id, metadata)| {
                Ok(lookup_asset(&self, None, asset_id, Some(metadata))?
                    .chain_err(|| "missing registered asset")?)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok((total_num, results))
    }
