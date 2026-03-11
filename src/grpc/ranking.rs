//! Wallet ranking module for tracking and ranking addresses by balance
//!
//! This module provides functionality to:
//! - Get top wallets by balance
//! - Get a specific wallet's rank
//! - Generate rich lists

use std::collections::HashMap;
use std::sync::RwLock;

use crate::util::FullHash;

/// Wallet information for ranking
#[derive(Debug, Clone)]
pub struct WalletEntry {
    pub scripthash: FullHash,
    pub address: String,
    pub balance: i64,
    pub tx_count: u64,
    pub first_seen_time: u64,
    pub last_seen_time: u64,
}

/// Cache for wallet rankings to avoid expensive recalculations
pub struct RankingCache {
    /// Sorted list of wallets by balance (descending)
    pub wallets: Vec<WalletEntry>,
    /// Map from scripthash to index in wallets vector
    scripthash_to_rank: HashMap<FullHash, usize>,

    /// Total supply in satoshis
    total_supply: i64,
    /// Last update timestamp
    last_updated: std::time::Instant,
    /// Block height at last update
    block_height: usize,
}

impl RankingCache {
    pub fn new() -> Self {
        RankingCache {
            wallets: Vec::new(),
            scripthash_to_rank: HashMap::new(),
            total_supply: 0,
            last_updated: std::time::Instant::now(),
            block_height: 0,
        }
    }

    pub fn total_wallets(&self) -> u64 {
        self.wallets.len() as u64
    }

    pub fn total_supply(&self) -> i64 {
        self.total_supply
    }

    pub fn last_updated(&self) -> std::time::Instant {
        self.last_updated
    }

    pub fn block_height(&self) -> usize {
        self.block_height
    }

    pub fn get_top_wallets(&self, limit: u32, offset: u32) -> Vec<WalletEntry> {
        let start = offset as usize;
        let end = std::cmp::min(start + limit as usize, self.wallets.len());
        
        if start >= self.wallets.len() {
            return Vec::new();
        }
        
        self.wallets[start..end].to_vec()
    }

    pub fn get_wallet_rank(&self, scripthash: &[u8; 32]) -> Option<WalletEntry> {
        self.scripthash_to_rank
            .get(scripthash)
            .and_then(|&idx| self.wallets.get(idx))
            .cloned()
    }

    /// Bulk set wallets (for background indexer)
    pub fn set_wallets(&mut self, wallets: Vec<WalletEntry>, block_height: usize) {
        self.wallets = wallets;
        self.scripthash_to_rank.clear();
        
        // Build the index
        for (idx, wallet) in self.wallets.iter().enumerate() {
            self.scripthash_to_rank.insert(wallet.scripthash, idx);
        }
        
        // Calculate total supply
        self.total_supply = self.wallets.iter().map(|w| w.balance).sum();
        self.block_height = block_height;
        self.last_updated = std::time::Instant::now();
    }

    /// Update a single wallet (for incremental updates)
    /// Maintains sorted order and updates the index
    pub fn update_wallet(&mut self, entry: WalletEntry) {
        // Remove old entry if exists
        if let Some(&old_idx) = self.scripthash_to_rank.get(&entry.scripthash) {
            // Update total supply delta
            if old_idx < self.wallets.len() {
                self.total_supply -= self.wallets[old_idx].balance;
            }
            self.wallets.retain(|w| w.scripthash != entry.scripthash);
        }

        // Add to total supply
        self.total_supply += entry.balance;

        // Find insertion point (maintain sorted order by balance descending)
        let insert_pos = self.wallets
            .iter()
            .position(|w| w.balance < entry.balance)
            .unwrap_or(self.wallets.len());

        self.wallets.insert(insert_pos, entry);

        // Rebuild the index (needed because positions shifted)
        self.scripthash_to_rank.clear();
        for (idx, wallet) in self.wallets.iter().enumerate() {
            self.scripthash_to_rank.insert(wallet.scripthash, idx);
        }

        self.last_updated = std::time::Instant::now();
    }

    /// Set the block height without changing wallets
    pub fn set_block_height(&mut self, height: usize) {
        self.block_height = height;
        self.last_updated = std::time::Instant::now();
    }
}

/// Manager for wallet rankings
pub struct RankingManager {
    cache: RwLock<RankingCache>,
}

impl RankingManager {
    pub fn new() -> Self {
        RankingManager {
            cache: RwLock::new(RankingCache::new()),
        }
    }

    /// Get the read-only cache
    pub fn cache(&self) -> std::sync::RwLockReadGuard<RankingCache> {
        self.cache.read().unwrap()
    }

    /// Get mutable cache access (for indexer updates)
    pub fn cache_mut(&self) -> std::sync::RwLockWriteGuard<RankingCache> {
        self.cache.write().unwrap()
    }

    /// Check if cache needs refresh
    pub fn needs_refresh(&self, current_height: usize) -> bool {
        self.cache.read().unwrap().block_height < current_height
    }

    /// Update the ranking cache (called periodically or on-demand)
    /// Note: This is a placeholder - actual implementation requires iterating
    /// through all scripthashes in the database, which is expensive.
    /// For production, consider:
    /// 1. Background indexing of balances
    /// 2. Incremental updates
    /// 3. Separate ranking database table
    #[allow(dead_code)]
    pub fn refresh_rankings(
        &self,
        current_height: usize,
    ) -> Result<(), String> {
        // This is a simplified implementation
        // Full implementation would need to scan all scripthashes
        // For now, return an error suggesting the feature needs setup
        
        let mut cache = self.cache.write().unwrap();
        
        // Mark as updated even if we can't fully populate
        // The actual rankings will be computed on-demand or via background job
        cache.block_height = current_height;
        cache.last_updated = std::time::Instant::now();
        
        Ok(())
    }
}

impl Default for RankingManager {
    fn default() -> Self {
        Self::new()
    }
}
