//! gRPC server setup and management

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

use tokio::sync::oneshot;
use tonic::transport::{Server, Identity, ServerTlsConfig};

use crate::config::Config;
use crate::new_index::Query;

use super::proto::compact_tx_streamer_server::CompactTxStreamerServer;
use super::ranking::RankingManager;
use super::service::LightwalletService;

/// Handle for the gRPC server thread
pub struct GrpcHandle {
    shutdown_tx: oneshot::Sender<()>,
    thread: thread::JoinHandle<()>,
}

impl GrpcHandle {
    /// Stop the gRPC server
    pub fn stop(self) {
        let _ = self.shutdown_tx.send(());
        self.thread.join().expect("gRPC server thread panicked");
    }
}

/// Start the gRPC server in a background thread
pub fn start_grpc_server(
    config: Arc<Config>,
    query: Arc<Query>,
    addr: SocketAddr,
    ranking: Arc<RankingManager>,
) -> GrpcHandle {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let thread = thread::spawn(move || {
        let worker_threads = num_cpus::get().max(2); // At least 2 threads, at most num CPUs
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(worker_threads)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        log::info!("gRPC server starting with {} worker threads", worker_threads);

        runtime.block_on(async move {
            run_grpc_server(config, query, addr, ranking, shutdown_rx).await;
        });
    });

    GrpcHandle {
        shutdown_tx,
        thread,
    }
}

async fn run_grpc_server(
    config: Arc<Config>,
    query: Arc<Query>,
    addr: SocketAddr,
    ranking: Arc<RankingManager>,
    shutdown_rx: oneshot::Receiver<()>,
) {
    let service = LightwalletService::new(query, config.clone(), ranking);
    let server = CompactTxStreamerServer::new(service);

    // Check if TLS is configured
    let use_tls = config.grpc_tls_cert.is_some() && config.grpc_tls_key.is_some();

    if use_tls {
        let cert_path = config.grpc_tls_cert.as_ref().unwrap();
        let key_path = config.grpc_tls_key.as_ref().unwrap();

        log::info!("gRPC server with TLS listening on {}", addr);
        log::info!("  TLS cert: {:?}", cert_path);
        log::info!("  TLS key: {:?}", key_path);

        // Load certificate and key
        let cert = match std::fs::read_to_string(cert_path) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to read TLS certificate: {}", e);
                return;
            }
        };
        let key = match std::fs::read_to_string(key_path) {
            Ok(k) => k,
            Err(e) => {
                log::error!("Failed to read TLS key: {}", e);
                return;
            }
        };

        let identity = Identity::from_pem(cert, key);
        let tls_config = ServerTlsConfig::new().identity(identity);

        let result = Server::builder()
            .tls_config(tls_config)
            .expect("Failed to configure TLS")
            .add_service(server)
            .serve_with_shutdown(addr, async {
                let _ = shutdown_rx.await;
                log::info!("gRPC server shutting down");
            })
            .await;

        if let Err(e) = result {
            log::error!("gRPC server error: {}", e);
        }
    } else {
        // Plain text server
        log::info!("gRPC server listening on {} (no TLS)", addr);

        let result = Server::builder()
            .add_service(server)
            .serve_with_shutdown(addr, async {
                let _ = shutdown_rx.await;
                log::info!("gRPC server shutting down");
            })
            .await;

        if let Err(e) = result {
            log::error!("gRPC server error: {}", e);
        }
    }
}
