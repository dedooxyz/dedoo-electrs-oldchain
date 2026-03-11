use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

use bitcoin::hashes::Hash;
use futures::Stream;
use tonic::{Request, Response, Status};

use crate::config::Config;
use crate::new_index::{compute_script_hash, Query};

use super::proto::*;
use super::proto::compact_tx_streamer_server::CompactTxStreamer;
use super::ranking::RankingManager;

/// Implementation of the CompactTxStreamer gRPC service
pub struct LightwalletService {
    query: Arc<Query>,
    config: Arc<Config>,
    ranking: Arc<RankingManager>,
}

impl LightwalletService {
    pub fn new(query: Arc<Query>, config: Arc<Config>, ranking: Arc<RankingManager>) -> Self {
        LightwalletService {
            query,
            config,
            ranking,
        }
    }

    /// Convert an address string to scripthash for querying
    fn address_to_scripthash(&self, address: &str) -> Result<[u8; 32], Status> {
        let network = self.config.network_type;
        let script = bitcoin::Address::from_str(address)
            .map_err(|e| Status::invalid_argument(format!("Invalid address {}: {}", address, e)))?
            .require_network(network.into())
            .map_err(|e| Status::invalid_argument(format!("Wrong network for address {}: {}", address, e)))?
            .script_pubkey();
        Ok(compute_script_hash(&script))
    }

    /// Helper to get network name string
    fn network_name(&self) -> &str {
        match self.config.network_type {
            crate::chain::Network::Bitcoin => "main",
            crate::chain::Network::Testnet => "test",
            crate::chain::Network::Regtest => "regtest",
            crate::chain::Network::Signet => "signet",
            #[cfg(feature = "liquid")]
            crate::chain::Network::Liquid => "liquid",
            #[cfg(feature = "liquid")]
            crate::chain::Network::LiquidTestnet => "liquidtestnet",
            #[cfg(feature = "liquid")]
            crate::chain::Network::LiquidRegtest => "liquidregtest",
        }
    }
}

#[tonic::async_trait]
impl CompactTxStreamer for LightwalletService {
    async fn get_lightd_info(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<LightdInfo>, Status> {
        let chain = self.query.chain();
        let height = chain.best_height();
        let hash = chain.best_hash();

        log::info!("gRPC GetLightdInfo: height={}, hash={}", height, hash);

        Ok(Response::new(LightdInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            vendor: "dedoo-electrs".to_string(),
            taddr_support: true,
            chain_name: self.network_name().to_string(),
            block_height: height as u64,
            best_block_hash: hash.to_string(),
            estimated_height: height as u64,
        }))
    }

    // ========== Block Services ==========

    async fn get_latest_block(
        &self,
        _request: Request<ChainSpec>,
    ) -> Result<Response<BlockId>, Status> {
        let chain = self.query.chain();
        let height = chain.best_height();
        let hash = chain.best_hash();

        Ok(Response::new(BlockId {
            height: height as u64,
            hash: hash.to_byte_array().to_vec(),
        }))
    }

    async fn get_block(
        &self,
        request: Request<BlockId>,
    ) -> Result<Response<CompactBlock>, Status> {
        let block_id = request.into_inner();
        let height = block_id.height as usize;
        
        log::debug!("gRPC GetBlock: height={}", height);
        
        let chain = self.query.chain();
        let header = chain.header_by_height(height)
            .ok_or_else(|| Status::not_found(format!("Block not found at height {}", height)))?;
        
        let hash = header.hash();
        let header_bytes = bitcoin::consensus::encode::serialize(header.header());

        // Get transaction IDs
        let txids = chain.get_block_txids(&hash)
            .ok_or_else(|| Status::internal("Failed to get block transactions"))?;

        let mut compact_txs = Vec::new();
        for (index, txid) in txids.iter().enumerate() {
            compact_txs.push(CompactTx {
                index: index as u64,
                hash: txid.to_byte_array().to_vec(),
                fee: 0, 
                inputs: Vec::new(),
                outputs: Vec::new(),
            });
        }

        Ok(Response::new(CompactBlock {
            proto_version: 1,
            height: height as u64,
            hash: hash.to_byte_array().to_vec(),
            prev_hash: header.header().prev_blockhash.to_byte_array().to_vec(),
            time: header.header().time,
            header: header_bytes,
            vtx: compact_txs,
        }))
    }

    type GetBlockRangeStream = Pin<Box<dyn Stream<Item = Result<CompactBlock, Status>> + Send>>;

    async fn get_block_range(
        &self,
        request: Request<BlockRange>,
    ) -> Result<Response<Self::GetBlockRangeStream>, Status> {
        let range = request.into_inner();
        let start = range.start.map(|b| b.height).unwrap_or(0) as usize;
        let end = range.end.map(|b| b.height).unwrap_or(start as u64) as usize;

        let query = self.query.clone();
        let config = self.config.clone();

        let stream = async_stream::try_stream! {
            for height in start..=end {
                let chain = query.chain();
                
                // Use compact block builder for optimized streaming
                use super::compact::{CompactBlockBuilder, CompactBlockLevel};
                
                let builder = CompactBlockBuilder::new(&chain, &config);
                
                if let Some(compact_block) = builder.build(height, CompactBlockLevel::Headers) {
                    yield compact_block;
                }
                
                // Yield periodically to avoid blocking
                if height % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }

    // ========== Transaction Services ==========

    async fn get_transaction(
        &self,
        request: Request<TxFilter>,
    ) -> Result<Response<RawTransaction>, Status> {
        let filter = request.into_inner();
        let txid: bitcoin::Txid = bitcoin::hashes::Hash::from_slice(&filter.hash)
            .map_err(|_| Status::invalid_argument("Invalid txid"))?;

        let tx = self.query.lookup_txn(&txid)
            .ok_or_else(|| Status::not_found("Transaction not found"))?;

        let height = self.query.chain().tx_confirming_block(&txid)
            .map(|b| b.height as u64)
            .unwrap_or(0);

        Ok(Response::new(RawTransaction {
            data: bitcoin::consensus::encode::serialize(&tx),
            height,
        }))
    }

    async fn send_transaction(
        &self,
        request: Request<RawTransaction>,
    ) -> Result<Response<SendResponse>, Status> {
        let tx_data = request.into_inner().data;
        let tx: bitcoin::Transaction = bitcoin::consensus::encode::deserialize(&tx_data)
            .map_err(|e| Status::invalid_argument(format!("Failed to deserialize transaction: {}", e)))?;

        let hex_tx = bitcoin::consensus::encode::serialize_hex(&tx);
        let txid = self.query.broadcast_raw(&hex_tx)
            .map_err(|e| Status::internal(format!("Failed to broadcast: {}", e)))?;

        log::info!("gRPC SendTransaction: broadcasted txid={}", txid);

        Ok(Response::new(SendResponse {
            error_message: String::new(),
            error_code: 0,
            tx_hash: txid.to_byte_array().to_vec(),
        }))
    }

    // ========== Address Services ==========

    async fn get_address_utxos(
        &self,
        request: Request<GetAddressUtxosArg>,
    ) -> Result<Response<GetAddressUtxosReply>, Status> {
        let args = request.into_inner();
        log::debug!("gRPC GetAddressUtxos: addresses={:?}", args.addresses);
        let mut all_utxos = Vec::new();

        for address in &args.addresses {
            let scripthash = self.address_to_scripthash(address)?;
            
            let utxos = self.query
                .utxo(&scripthash)
                .map_err(|e| Status::internal(format!("Failed to get UTXOs: {}", e)))?;

            for utxo in utxos {
                // Filter by start height if specified
                if let Some(confirmed) = &utxo.confirmed {
                    if (confirmed.height as u64) < args.start_height {
                        continue;
                    }
                } else if args.start_height > 0 {
                    // Unconfirmed UTXOs have height 0
                    continue;
                }

                all_utxos.push(AddressUtxo {
                    address: address.clone(),
                    txid: utxo.txid.to_byte_array().to_vec(),
                    index: utxo.vout,
                    script: Vec::new(), 
                    value_sat: utxo.value as i64,
                    height: utxo.confirmed.map(|c| c.height as u64).unwrap_or(0),
                });

                // Check max entries limit
                if args.max_entries > 0 && all_utxos.len() >= args.max_entries as usize {
                    break;
                }
            }

            if args.max_entries > 0 && all_utxos.len() >= args.max_entries as usize {
                break;
            }
        }

        Ok(Response::new(GetAddressUtxosReply {
            address_utxos: all_utxos,
        }))
    }

    type GetAddressUtxosStreamStream = Pin<Box<dyn Stream<Item = Result<AddressUtxo, Status>> + Send>>;

    async fn get_address_utxos_stream(
        &self,
        request: Request<GetAddressUtxosArg>,
    ) -> Result<Response<Self::GetAddressUtxosStreamStream>, Status> {
        let args = request.into_inner();
        log::debug!("gRPC GetAddressUtxosStream: addresses={:?}", args.addresses);
        let query = self.query.clone();

        // Pre-compute scripthashes
        let mut addresses_with_hashes = Vec::new();
        for address in args.addresses.clone() {
            let scripthash = self.address_to_scripthash(&address)?;
            addresses_with_hashes.push((address, scripthash));
        }

        let start_height = args.start_height;
        let max_entries = args.max_entries;

        let stream = async_stream::try_stream! {
            let mut count = 0;
            for (address, scripthash) in addresses_with_hashes {
                let utxos = query.utxo(&scripthash)
                    .map_err(|e| Status::internal(format!("Failed to get UTXOs: {}", e)))?;

                for utxo in utxos {
                    if let Some(confirmed) = &utxo.confirmed {
                        if (confirmed.height as u64) < start_height {
                            continue;
                        }
                    } else if start_height > 0 {
                        continue;
                    }

                    yield AddressUtxo {
                        address: address.clone(),
                        txid: utxo.txid.to_byte_array().to_vec(),
                        index: utxo.vout,
                        script: Vec::new(),
                        value_sat: utxo.value as i64,
                        height: utxo.confirmed.map(|c| c.height as u64).unwrap_or(0),
                    };

                    count += 1;
                    if max_entries > 0 && count >= max_entries {
                        return;
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }

    async fn get_balance(
        &self,
        request: Request<GetBalanceArg>,
    ) -> Result<Response<GetBalanceReply>, Status> {
        let args = request.into_inner();
        log::debug!("gRPC GetBalance: address={}", args.address);
        let scripthash = self.address_to_scripthash(&args.address)?;

        let stats = self.query.stats(&scripthash);
        
        let confirmed = (stats.0.funded_txo_sum as i64) - (stats.0.spent_txo_sum as i64);
        let pending = (stats.1.funded_txo_sum as i64) - (stats.1.spent_txo_sum as i64);

        Ok(Response::new(GetBalanceReply {
            confirmed_balance: confirmed,
            pending_balance: pending,
            total_balance: confirmed + pending,
        }))
    }

    async fn get_address_stats(
        &self,
        request: Request<GetAddressStatsArg>,
    ) -> Result<Response<GetAddressStatsReply>, Status> {
        let args = request.into_inner();
        log::debug!("gRPC GetAddressStats: address={}", args.address);
        let scripthash = self.address_to_scripthash(&args.address)?;

        let stats = self.query.stats(&scripthash);
        let txs = self.query.history_txids(&scripthash, 1000);

        // Find first and last transaction timestamps
        let mut first_seen_time: u64 = 0;
        let mut last_seen_time: u64 = 0;

        for (_, blockid) in &txs {
            if let Some(block_id) = blockid {
                let timestamp = block_id.time as u64;
                if first_seen_time == 0 || first_seen_time > timestamp {
                    first_seen_time = timestamp;
                }
                if last_seen_time < timestamp {
                    last_seen_time = timestamp;
                }
            }
        }

        let funded_count = stats.0.funded_txo_count + stats.1.funded_txo_count;
        let funded_sum = stats.0.funded_txo_sum + stats.1.funded_txo_sum;
        let spent_count = stats.0.spent_txo_count + stats.1.spent_txo_count;
        let spent_sum = stats.0.spent_txo_sum + stats.1.spent_txo_sum;
        let tx_count = stats.0.tx_count + stats.1.tx_count;
        let balance = (funded_sum as i64) - (spent_sum as i64);

        Ok(Response::new(GetAddressStatsReply {
            funded_txo_count: funded_count as u64,
            funded_txo_sum: funded_sum,
            spent_txo_count: spent_count as u64,
            spent_txo_sum: spent_sum,
            tx_count: tx_count as u64,
            balance,
            first_seen_tx_time: first_seen_time,
            last_seen_tx_time: last_seen_time,
        }))
    }

    async fn get_tx_history(
        &self,
        request: Request<GetTxHistoryArg>,
    ) -> Result<Response<GetTxHistoryReply>, Status> {
        let args = request.into_inner();
        log::debug!("gRPC GetTxHistory: address={}, limit={}", args.address, args.limit);
        let scripthash = self.address_to_scripthash(&args.address)?;

        let limit = if args.limit == 0 { 25 } else { args.limit as usize };
        let txids = self.query.history_txids(&scripthash, limit + args.offset as usize);

        let mut transactions = Vec::new();
        for (txid, blockid) in txids.into_iter().skip(args.offset as usize).take(limit) {
            let height = blockid.as_ref().map(|b| b.height as u64).unwrap_or(0);
            let time = blockid.as_ref().map(|b| b.time).unwrap_or(0);

            // Calculate balance change for this transaction
            let (balance_change, fee) = if let Some(tx) = self.query.lookup_txn(&txid) {
                use std::collections::BTreeSet;
                use crate::chain::OutPoint;
                
                // Get all previous outputs for inputs
                let outpoints: BTreeSet<OutPoint> = tx.input
                    .iter()
                    .map(|txin| txin.previous_output)
                    .collect();
                let prev_txos = self.query.lookup_txos(&outpoints);

                let mut change: i64 = 0;
                let mut input_sum: u64 = 0;
                let mut output_sum: u64 = 0;

                // Calculate inputs spent from this address
                for txin in &tx.input {
                    if let Some(prev_txout) = prev_txos.get(&txin.previous_output) {
                        #[cfg(not(feature = "liquid"))]
                        let value = prev_txout.value.to_sat();
                        #[cfg(feature = "liquid")]
                        let value = prev_txout.value.explicit().unwrap_or(0);
                        
                        input_sum += value;
                        
                        // Check if this input is from our address
                        let prev_scripthash = compute_script_hash(&prev_txout.script_pubkey);
                        if prev_scripthash == scripthash {
                            change -= value as i64;
                        }
                    }
                }

                // Calculate outputs to this address
                for txout in &tx.output {
                    #[cfg(not(feature = "liquid"))]
                    let value = txout.value.to_sat();
                    #[cfg(feature = "liquid")]
                    let value = txout.value.explicit().unwrap_or(0);
                    
                    output_sum += value;
                    
                    // Check if this output is to our address
                    let out_scripthash = compute_script_hash(&txout.script_pubkey);
                    if out_scripthash == scripthash {
                        change += value as i64;
                    }
                }

                // Calculate fee (inputs - outputs), but only if we have all inputs
                let fee = if input_sum > 0 && input_sum >= output_sum {
                    input_sum - output_sum
                } else {
                    0
                };

                (change, fee)
            } else {
                (0, 0)
            };

            transactions.push(TxHistoryEntry {
                txid: txid.to_byte_array().to_vec(),
                height,
                time,
                balance_change,
                fee,
            });
        }

        let stats = self.query.stats(&scripthash);
        let total_count = (stats.0.tx_count + stats.1.tx_count) as u64;

        Ok(Response::new(GetTxHistoryReply {
            transactions,
            total_count,
        }))
    }

    // ========== Wallet Ranking ==========

    async fn get_top_wallets(
        &self,
        request: Request<GetTopWalletsArg>,
    ) -> Result<Response<GetTopWalletsReply>, Status> {
        let args = request.into_inner();
        let limit = if args.limit == 0 { 100 } else { std::cmp::min(args.limit, 1000) };
        
        let cache = self.ranking.cache();
        let wallets = cache.get_top_wallets(limit, args.offset);

        let wallet_infos: Vec<WalletInfo> = wallets
            .into_iter()
            .enumerate()
            .map(|(idx, w)| WalletInfo {
                address: w.address,
                balance: w.balance,
                tx_count: w.tx_count,
                first_seen_time: w.first_seen_time,
                last_seen_time: w.last_seen_time,
                rank: (args.offset as u64) + (idx as u64) + 1,
            })
            .collect();

        Ok(Response::new(GetTopWalletsReply {
            wallets: wallet_infos,
            total_wallets: cache.total_wallets(),
            total_supply: cache.total_supply() as u64,
        }))
    }

    async fn get_wallet_rank(
        &self,
        request: Request<GetWalletRankArg>,
    ) -> Result<Response<GetWalletRankReply>, Status> {
        let args = request.into_inner();
        log::debug!("gRPC GetWalletRank: address={}", args.address);
        let scripthash = self.address_to_scripthash(&args.address)?;
        
        let cache = self.ranking.cache();
        if let Some(wallet) = cache.get_wallet_rank(&scripthash) {
            let rank = cache.wallets.iter()
                .position(|w| w.scripthash == scripthash)
                .map(|p| p as u64 + 1)
                .unwrap_or(0);
                
            let percentile = if cache.wallets.is_empty() {
                0.0
            } else {
                100.0 * (1.0 - (rank as f64 / cache.wallets.len() as f64))
            };

            Ok(Response::new(GetWalletRankReply {
                wallet: Some(WalletInfo {
                    address: wallet.address,
                    balance: wallet.balance,
                    tx_count: wallet.tx_count,
                    first_seen_time: wallet.first_seen_time,
                    last_seen_time: wallet.last_seen_time,
                    rank,
                }),
                total_wallets: cache.total_wallets(),
                percentile,
            }))
        } else {
            Err(Status::not_found("Address not found in rankings"))
        }
    }

    async fn get_rich_list(
        &self,
        request: Request<GetRichListArg>,
    ) -> Result<Response<GetRichListReply>, Status> {
        let args = request.into_inner();
        log::debug!("gRPC GetRichList: limit={}, min_balance={}", args.limit, args.min_balance);
        let limit = if args.limit == 0 { 100 } else { std::cmp::min(args.limit, 1000) };
        
        let cache = self.ranking.cache();
        let filtered_entries: Vec<RichListEntry> = cache.wallets.iter()
            .enumerate()
            .filter(|(_, w)| w.balance >= args.min_balance)
            .skip(args.offset as usize)
            .take(limit as usize)
            .map(|(idx, w)| RichListEntry {
                address: w.address.clone(),
                balance: w.balance,
                rank: idx as u64 + 1,
            })
            .collect();

        Ok(Response::new(GetRichListReply {
            entries: filtered_entries,
            total_count: cache.total_wallets(),
            total_supply: cache.total_supply() as u64,
        }))
    }

    // ========== Network Services ==========

    async fn get_mempool(
        &self,
        request: Request<GetMempoolArg>,
    ) -> Result<Response<GetMempoolReply>, Status> {
        let args = request.into_inner();
        let mempool = self.query.mempool();
        
        let mut entries = Vec::new();
        let mut total_size = 0;
        
        // Use the public recently added transactions or all txids
        // Since we want all mempool entries, we'll use a new method or access fields if possible
        // For now, let's use the txids() and lookup stats
        let txids = mempool.txids();
        let limit = if args.limit == 0 { txids.len() } else { std::cmp::min(args.limit as usize, txids.len()) };

        for txid in txids.iter().take(limit) {
            if let Some(fee) = mempool.get_tx_fee(txid) {
                // We need vsize. Since vsize isn't directly exposed per-txid in a public method,
                // we'll try to get it from the transaction itself if available, or use 0.
                // In a production environment, we'd add a method to Mempool to get TxFeeInfo.
                let vsize = mempool.lookup_txn(txid).map(|tx| tx.weight().to_wu() / 4).unwrap_or(0) as u32;
                
                entries.push(MempoolEntry {
                    txid: txid.to_byte_array().to_vec(),
                    fee,
                    size: vsize,
                    time: 0, // Not tracked by electrs mempool
                });
                total_size += vsize as u64;
            }
        }

        log::info!("gRPC GetMempool: returning {} entries", entries.len());

        Ok(Response::new(GetMempoolReply {
            entries,
            total_count: txids.len() as u64,
            total_size,
        }))
    }

    async fn get_supply_info(
        &self,
        _request: Request<GetSupplyInfoArg>,
    ) -> Result<Response<GetSupplyInfoReply>, Status> {
        let cache = self.ranking.cache();
        let height = cache.block_height();
        
        let block_hash = self.query.chain().header_by_height(height)
            .map(|h| h.hash().to_string())
            .unwrap_or_default();

        log::info!("gRPC GetSupplyInfo: height={}, supply={}", height, cache.total_supply());

        Ok(Response::new(GetSupplyInfoReply {
            total_supply: cache.total_supply(),
            circulating_supply: cache.total_supply(), 
            block_height: height as u64,
            block_hash,
        }))
    }
}
