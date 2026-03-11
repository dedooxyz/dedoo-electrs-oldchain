//! gRPC server module implementing lightwalletd-style API
//!
//! This module provides a gRPC interface for light wallet clients,
//! compatible with Zcash's lightwalletd protocol with Junkcoin extensions.

mod service;
mod server;
pub mod ranking;
mod indexer;
pub mod compact;

pub use server::start_grpc_server;
pub use server::GrpcHandle;
pub use indexer::{RankingIndexer, RankingIndexerHandle};
pub use ranking::RankingManager;

// Include the generated protobuf code
pub mod proto {
    tonic::include_proto!("junkcoin.lightwallet");
}
