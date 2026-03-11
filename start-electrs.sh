#!/bin/bash
# Performance-optimized configuration for dedoo-electrs
# Tuned for high-traffic production environment

cd /root/dedoo-electrs-oldchain

./target/release/electrs \
    --network testnet \
    -vvvv \
    --daemon-dir /root/.junkcoin \
    --daemon-rpc-addr 127.0.0.1:19772 \
    --cookie "senasagara:AnakUcings" \
    --db-dir /root/.dedoo-electrs/db \
    --electrum-rpc-addr 0.0.0.0:50001 \
    --http-addr 0.0.0.0:50010 \
    --grpc-addr 0.0.0.0:9067 \
    --monitoring-addr 127.0.0.1:4227 \
    --address-search \
    \
    # === Database Performance Tuning ===
    --db-max-open-files=8192 \
    --db-write-buffer-size=512 \
    --db-compaction-parallelism=4 \
    \
    # === Connection Limits ===
    --electrum-max-connections=200 \
    --electrum-channel-buffer-size=20 \
    \
    # === HTTP Server Tuning ===
    --http-worker-threads=8 \
    \
    # === Mempool Tuning ===
    --mempool-backlog-stats-ttl=30 \
    \
    # === Fee Estimates Caching ===
    --fee-estimates-cache-ttl=300 \
    --relay-fee-cache-ttl=300


