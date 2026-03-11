//! Compact block utilities for efficient block streaming
//!
//! This module provides optimized compact block generation
//! with different detail levels for different use cases.

use bitcoin::consensus::encode::serialize;
use bitcoin::hashes::Hash;

use crate::config::Config;
use crate::new_index::{ChainQuery, Query};
use crate::util::ScriptToAddr;

use super::proto::{CompactBlock, CompactTx, CompactInput, CompactOutput};

/// Level of detail for compact blocks
#[derive(Debug, Clone, Copy)]
pub enum CompactBlockLevel {
    /// Minimal - just block header info (for sync status checks)
    /// Fields: height, hash, prev_hash, time
    Minimal,
    
    /// Headers - includes serialized header (for header chain sync)
    /// Fields: all Minimal + serialized header bytes
    Headers,
    
    /// Standard - includes transaction hashes (for wallet sync)
    /// Fields: all Headers + tx hashes only
    Standard,
    
    /// Full - includes full transaction details (for detailed queries)
    /// Fields: all Standard + full inputs/outputs with addresses
    Full,
}

/// Builder for compact blocks with caching support
pub struct CompactBlockBuilder<'a> {
    chain: &'a ChainQuery,
    query: Option<&'a Query>,
    config: &'a Config,
}

impl<'a> CompactBlockBuilder<'a> {
    pub fn new(chain: &'a ChainQuery, config: &'a Config) -> Self {
        CompactBlockBuilder {
            chain,
            query: None,
            config,
        }
    }

    pub fn with_query(mut self, query: &'a Query) -> Self {
        self.query = Some(query);
        self
    }

    /// Build a compact block at the specified height with the given detail level
    pub fn build(&self, height: usize, level: CompactBlockLevel) -> Option<CompactBlock> {
        let header = self.chain.header_by_height(height)?;
        let hash = header.hash();

        match level {
            CompactBlockLevel::Minimal => {
                Some(CompactBlock {
                    proto_version: 1,
                    height: height as u64,
                    hash: hash.to_byte_array().to_vec(),
                    prev_hash: header.header().prev_blockhash.to_byte_array().to_vec(),
                    time: header.header().time,
                    header: Vec::new(),
                    vtx: Vec::new(),
                })
            }
            
            CompactBlockLevel::Headers => {
                let header_bytes = serialize(&header.header());
                
                Some(CompactBlock {
                    proto_version: 1,
                    height: height as u64,
                    hash: hash.to_byte_array().to_vec(),
                    prev_hash: header.header().prev_blockhash.to_byte_array().to_vec(),
                    time: header.header().time,
                    header: header_bytes,
                    vtx: Vec::new(),
                })
            }
            
            CompactBlockLevel::Standard => {
                let header_bytes = serialize(&header.header());
                
                // Get transaction IDs only
                let txids = self.chain.get_block_txids(&hash)?;
                
                let vtx: Vec<CompactTx> = txids.iter()
                    .enumerate()
                    .map(|(index, txid)| CompactTx {
                        index: index as u64,
                        hash: txid.to_byte_array().to_vec(),
                        fee: 0,
                        inputs: Vec::new(),
                        outputs: Vec::new(),
                    })
                    .collect();
                
                Some(CompactBlock {
                    proto_version: 1,
                    height: height as u64,
                    hash: hash.to_byte_array().to_vec(),
                    prev_hash: header.header().prev_blockhash.to_byte_array().to_vec(),
                    time: header.header().time,
                    header: header_bytes,
                    vtx,
                })
            }
            
            CompactBlockLevel::Full => {
                let query = self.query?;
                let header_bytes = serialize(&header.header());
                let txids = self.chain.get_block_txids(&hash)?;
                
                let mut vtx = Vec::with_capacity(txids.len());
                
                for (index, txid) in txids.iter().enumerate() {
                    if let Some(tx) = query.lookup_txn(txid) {
                        let mut inputs = Vec::with_capacity(tx.input.len());
                        let mut outputs = Vec::with_capacity(tx.output.len());
                        
                        // Process inputs (minimal info for now)
                        for txin in &tx.input {
                            inputs.push(CompactInput {
                                prev_tx_hash: txin.previous_output.txid.to_byte_array().to_vec(),
                                prev_index: txin.previous_output.vout,
                                value: 0,
                                address: String::new(),
                            });
                        }
                        
                        // Process outputs with addresses
                        for txout in &tx.output {
                            #[cfg(not(feature = "liquid"))]
                            let value = txout.value.to_sat();
                            #[cfg(feature = "liquid")]
                            let value = txout.value.explicit().unwrap_or(0);
                            
                            let address = txout.script_pubkey
                                .to_address_str(self.config.network_type)
                                .unwrap_or_default();
                            
                            outputs.push(CompactOutput {
                                value,
                                script_pub_key: txout.script_pubkey.to_bytes(),
                                address,
                            });
                        }
                        
                        vtx.push(CompactTx {
                            index: index as u64,
                            hash: txid.to_byte_array().to_vec(),
                            fee: 0, // Would need to calculate
                            inputs,
                            outputs,
                        });
                    }
                }
                
                Some(CompactBlock {
                    proto_version: 1,
                    height: height as u64,
                    hash: hash.to_byte_array().to_vec(),
                    prev_hash: header.header().prev_blockhash.to_byte_array().to_vec(),
                    time: header.header().time,
                    header: header_bytes,
                    vtx,
                })
            }
        }
    }

    /// Build compact blocks for a range of heights
    /// Uses an iterator to allow streaming without loading all blocks into memory
    pub fn build_range(
        &self,
        start: usize,
        end: usize,
        level: CompactBlockLevel,
    ) -> impl Iterator<Item = CompactBlock> + '_ {
        (start..=end).filter_map(move |height| self.build(height, level))
    }
}

/// Utility function to quickly build a minimal compact block
pub fn build_minimal_block(chain: &ChainQuery, height: usize) -> Option<CompactBlock> {
    let header = chain.header_by_height(height)?;
    let hash = header.hash();
    
    Some(CompactBlock {
        proto_version: 1,
        height: height as u64,
        hash: hash.to_byte_array().to_vec(),
        prev_hash: header.header().prev_blockhash.to_byte_array().to_vec(),
        time: header.header().time,
        header: Vec::new(),
        vtx: Vec::new(),
    })
}

/// Utility function to quickly build a header-only compact block
pub fn build_header_block(chain: &ChainQuery, height: usize) -> Option<CompactBlock> {
    let header = chain.header_by_height(height)?;
    let hash = header.hash();
    let header_bytes = serialize(&header.header());
    
    Some(CompactBlock {
        proto_version: 1,
        height: height as u64,
        hash: hash.to_byte_array().to_vec(),
        prev_hash: header.header().prev_blockhash.to_byte_array().to_vec(),
        time: header.header().time,
        header: header_bytes,
        vtx: Vec::new(),
    })
}
