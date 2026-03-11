//! Background ranking indexer for wallet balance ranking
//!
//! This module provides a background thread that scans all scripthashes
//! in the database and builds a sorted ranking of wallets by balance.
//!
//! Features:
//! - Initial full scan on startup
//! - Incremental updates on new blocks
//! - Performance optimizations (batching, yielding, low priority)

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use log::{info, warn, debug};

use crate::new_index::{ChainQuery, Store};
use crate::grpc::ranking::{RankingManager, WalletEntry};
use crate::util::{FullHash, ScriptToAddr};
use crate::chain::Network;

/// Configuration for indexer performance tuning
const BATCH_SIZE: u64 = 500;              // Process this many scripthashes before yielding
const YIELD_DURATION_MS: u64 = 10;        // Milliseconds to sleep between batches
const PROGRESS_LOG_INTERVAL: u64 = 5000;  // Log progress every N scripthashes
const UPDATE_CHECK_INTERVAL_SECS: u64 = 30; // Check for new blocks every N seconds

/// TxHistoryKey structure matching the database format
#[derive(Debug)]
struct TxHistoryKey {
    code: u8,
    hash: FullHash,
    confirmed_height: u32,
    txinfo: [u8; 28],
}

impl TxHistoryKey {
    fn from_bytes(key: &[u8]) -> Option<Self> {
        if key.len() < 1 + 32 + 4 {
            return None;
        }
        
        let code = key[0];
        if code != b'H' {
            return None;
        }
        
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&key[1..33]);
        
        // Height is stored as big-endian u32
        let confirmed_height = u32::from_be_bytes([key[33], key[34], key[35], key[36]]);
        
        let mut txinfo = [0u8; 28];
        if key.len() >= 37 + 28 {
            txinfo.copy_from_slice(&key[37..65]);
        }
        
        Some(TxHistoryKey {
            code,
            hash,
            confirmed_height,
            txinfo,
        })
    }
}

/// Handle for the ranking indexer thread
pub struct RankingIndexerHandle {
    shutdown: Arc<AtomicBool>,
    thread: JoinHandle<()>,
    last_height: Arc<AtomicUsize>,
}

impl RankingIndexerHandle {
    /// Signal the indexer to shutdown
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }

    /// Wait for the indexer thread to complete
    pub fn join(self) {
        let _ = self.thread.join();
    }

    /// Get the last processed block height
    pub fn last_height(&self) -> usize {
        self.last_height.load(Ordering::Relaxed)
    }
}

/// Background indexer that scans all scripthashes and builds wallet rankings
pub struct RankingIndexer {
    store: Arc<Store>,
    chain: Arc<ChainQuery>,
    ranking: Arc<RankingManager>,
    #[allow(dead_code)]
    network: Network,
}

impl RankingIndexer {
    pub fn new(
        store: Arc<Store>,
        chain: Arc<ChainQuery>,
        ranking: Arc<RankingManager>,
        network: Network,
    ) -> Self {
        RankingIndexer {
            store,
            chain,
            ranking,
            network,
        }
    }

    /// Spawn the background indexing thread with incremental update support
    /// Returns a handle that can be used to check status and shutdown
    pub fn spawn(self) -> RankingIndexerHandle {
        let shutdown = Arc::new(AtomicBool::new(false));
        let last_height = Arc::new(AtomicUsize::new(0));
        
        let shutdown_clone = shutdown.clone();
        let last_height_clone = last_height.clone();

        let thread = thread::Builder::new()
            .name("ranking-indexer".to_string())
            .spawn(move || {
                // Set lower thread priority if possible (Linux specific)
                #[cfg(target_os = "linux")]
                {
                    // Nice value 10 = lower priority
                    unsafe {
                        libc::nice(10);
                    }
                }

                info!("Starting background wallet ranking indexer (low priority)...");
                info!("Performance settings: batch_size={}, yield_ms={}", BATCH_SIZE, YIELD_DURATION_MS);
                
                // Initial full scan
                let initial_height = match self.full_scan() {
                    Ok(height) => {
                        info!("Initial ranking scan completed at height {}", height);
                        height
                    }
                    Err(e) => {
                        warn!("Ranking indexer initial scan error: {}", e);
                        0
                    }
                };
                
                last_height_clone.store(initial_height, Ordering::SeqCst);

                // Incremental update loop
                info!("Entering incremental update mode (checking every {}s)", UPDATE_CHECK_INTERVAL_SECS);
                
                while !shutdown_clone.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_secs(UPDATE_CHECK_INTERVAL_SECS));
                    
                    if shutdown_clone.load(Ordering::SeqCst) {
                        break;
                    }

                    let current_height = self.chain.best_height();
                    let last = last_height_clone.load(Ordering::SeqCst);
                    
                    if current_height > last {
                        debug!("New blocks detected: {} -> {}", last, current_height);
                        
                        match self.incremental_update(last, current_height) {
                            Ok(updated) => {
                                if updated > 0 {
                                    info!("Ranking updated {} wallets for blocks {} to {}", 
                                          updated, last + 1, current_height);
                                }
                                last_height_clone.store(current_height, Ordering::SeqCst);
                            }
                            Err(e) => {
                                warn!("Incremental update error: {}", e);
                            }
                        }
                    }
                }

                info!("Ranking indexer shutdown complete");
            })
            .expect("Failed to spawn ranking indexer thread");

        RankingIndexerHandle {
            shutdown,
            thread,
            last_height,
        }
    }

    /// Full scan of all scripthashes (initial indexing)
    /// Returns the block height at completion
    fn full_scan(&self) -> Result<usize, String> {
        let start_time = Instant::now();
        info!("Starting full scan of all scripthashes for ranking...");

        // Track unique scripthashes we've seen
        let mut seen_scripthashes: HashSet<FullHash> = HashSet::new();
        let mut wallet_entries: Vec<WalletEntry> = Vec::new();
        let mut processed = 0u64;
        let mut with_balance = 0u64;
        let mut batch_count = 0u64;

        // Iterate through the history database
        let mut iter = self.store.history_db().raw_iterator();
        iter.seek(b"H");

        while iter.valid() {
            let key = match iter.key() {
                Some(k) => k,
                None => break,
            };

            // Check if we're still in the 'H' prefix
            if !key.starts_with(b"H") {
                break;
            }

            // Parse the key
            if let Some(history_key) = TxHistoryKey::from_bytes(key) {
                // Only process each scripthash once
                if !seen_scripthashes.contains(&history_key.hash) {
                    seen_scripthashes.insert(history_key.hash);
                    processed += 1;
                    batch_count += 1;

                    // Get stats for this scripthash
                    let stats = self.chain.stats(&history_key.hash);
                    let balance = (stats.funded_txo_sum as i64) - (stats.spent_txo_sum as i64);

                    // Only include wallets with positive balance
                    if balance > 0 {
                        with_balance += 1;

                        // Try to resolve scripthash to human-readable address
                        let mut address = format!("scripthash:{}", hex_encode(&history_key.hash[..8]));
                        
                        // Use transaction history to find an output and resolve address
                        // This works even if all UTXOs are spent
                        let history = self.chain.history_txids(&history_key.hash, 5);
                        for (txid, _block_id) in history.iter() {
                            if let Some(tx) = self.chain.lookup_txn(txid, None) {
                                // Check outputs for matching scripthash
                                for txout in tx.output.iter() {
                                    let script_hash = crate::new_index::compute_script_hash(&txout.script_pubkey);
                                    if script_hash == history_key.hash {
                                        if let Some(addr_str) = txout.script_pubkey.to_address_str(self.network) {
                                            address = addr_str;
                                            break;
                                        }
                                    }
                                }
                                if !address.starts_with("scripthash:") {
                                    break; // Found address, stop searching
                                }
                            }
                        }

                        // Get transaction history to find first/last seen times
                        let txs = self.chain.history_txids(&history_key.hash, 2);
                        let mut first_seen = 0u64;
                        let mut last_seen = 0u64;

                        for (_, block_id) in &txs {
                            let timestamp = block_id.time as u64;
                            if first_seen == 0 || first_seen > timestamp {
                                first_seen = timestamp;
                            }
                            if last_seen < timestamp {
                                last_seen = timestamp;
                            }
                        }

                        wallet_entries.push(WalletEntry {
                            scripthash: history_key.hash,
                            address,
                            balance,
                            tx_count: (stats.funded_txo_count + stats.spent_txo_count) as u64,
                            first_seen_time: first_seen,
                            last_seen_time: last_seen,
                        });
                    }

                    // Yield to other threads periodically
                    if batch_count >= BATCH_SIZE {
                        batch_count = 0;
                        thread::sleep(Duration::from_millis(YIELD_DURATION_MS));
                        thread::yield_now();
                    }

                    // Log progress
                    if processed % PROGRESS_LOG_INTERVAL == 0 {
                        let elapsed = start_time.elapsed();
                        let rate = processed as f64 / elapsed.as_secs_f64();
                        info!(
                            "Ranking indexer: {} scripthashes ({:.0}/sec), {} with balance, elapsed: {:.1}s",
                            processed, rate, with_balance, elapsed.as_secs_f64()
                        );
                    }
                }
            }

            iter.next();
        }

        // Sort by balance descending
        info!("Sorting {} wallets by balance...", wallet_entries.len());
        wallet_entries.sort_by(|a, b| b.balance.cmp(&a.balance));

        let elapsed = start_time.elapsed();
        let height = self.chain.best_height();
        
        info!(
            "Full scan complete in {:.1}s: {} scripthashes, {} with positive balance, rate: {:.0}/sec",
            elapsed.as_secs_f64(),
            processed,
            with_balance,
            processed as f64 / elapsed.as_secs_f64()
        );

        // Update the ranking cache with all entries
        {
            let mut cache = self.ranking.cache_mut();
            cache.set_wallets(wallet_entries, height);
        }

        info!(
            "Ranking cache updated with {} wallets at height {}",
            self.ranking.cache().total_wallets(),
            height
        );

        Ok(height)
    }

    /// Incremental update for new blocks
    /// Scans transactions in new blocks and updates affected wallet balances
    /// Returns the number of wallets updated
    fn incremental_update(&self, from_height: usize, to_height: usize) -> Result<usize, String> {
        let mut updated_scripthashes: HashSet<FullHash> = HashSet::new();

        // Scan the history database for entries in the new block range
        let mut iter = self.store.history_db().raw_iterator();
        iter.seek(b"H");

        while iter.valid() {
            let key = match iter.key() {
                Some(k) => k,
                None => break,
            };

            if !key.starts_with(b"H") {
                break;
            }

            if let Some(history_key) = TxHistoryKey::from_bytes(key) {
                // Check if this entry is in the new block range
                let height = history_key.confirmed_height as usize;
                if height > from_height && height <= to_height {
                    updated_scripthashes.insert(history_key.hash);
                }
            }

            iter.next();
        }

        if updated_scripthashes.is_empty() {
            return Ok(0);
        }

        debug!("Found {} scripthashes to update", updated_scripthashes.len());

        // Update balance for each affected scripthash
        let mut updates = Vec::new();
        
        for scripthash in &updated_scripthashes {
            let stats = self.chain.stats(scripthash);
            let balance = (stats.funded_txo_sum as i64) - (stats.spent_txo_sum as i64);

            if balance > 0 {
                let mut address = format!("scripthash:{}", hex_encode(&scripthash[..8]));
                
                // Use transaction history to resolve address
                let history = self.chain.history_txids(scripthash, 5);
                for (txid, _block_id) in history.iter() {
                    if let Some(tx) = self.chain.lookup_txn(txid, None) {
                        for txout in tx.output.iter() {
                            let script_hash = crate::new_index::compute_script_hash(&txout.script_pubkey);
                            if script_hash == *scripthash {
                                if let Some(addr_str) = txout.script_pubkey.to_address_str(self.network) {
                                    address = addr_str;
                                    break;
                                }
                            }
                        }
                        if !address.starts_with("scripthash:") {
                            break;
                        }
                    }
                }
                
                // Get timestamps
                let txs = self.chain.history_txids(scripthash, 2);
                let mut first_seen = 0u64;
                let mut last_seen = 0u64;

                for (_, block_id) in &txs {
                    let timestamp = block_id.time as u64;
                    if first_seen == 0 || first_seen > timestamp {
                        first_seen = timestamp;
                    }
                    if last_seen < timestamp {
                        last_seen = timestamp;
                    }
                }

                updates.push(WalletEntry {
                    scripthash: *scripthash,
                    address,
                    balance,
                    tx_count: (stats.funded_txo_count + stats.spent_txo_count) as u64,
                    first_seen_time: first_seen,
                    last_seen_time: last_seen,
                });
            }
        }

        // Apply updates to the ranking cache
        let updated_count = updates.len();
        {
            let mut cache = self.ranking.cache_mut();
            for entry in updates {
                cache.update_wallet(entry);
            }
            cache.set_block_height(to_height);
        }

        Ok(updated_count)
    }
}

/// Helper function to encode bytes as hex string
fn hex_encode(data: &[u8]) -> String {
    use hex::DisplayHex;
    data.to_lower_hex_string()
}
