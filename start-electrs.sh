#!/bin/bash
cd /root/electrs-junkcoin
./target/release/electrs \
    --network mainnet \
    --daemon-dir /root/.junkcoin \
    --daemon-rpc-addr 127.0.0.1:9771 \
    --cookie "test:test" \
    --db-dir /root/.electrs-junkcoin/db \
    --electrum-rpc-addr 0.0.0.0:50001 \
    --http-addr 0.0.0.0:50010 \
    --monitoring-addr 127.0.0.1:4226 \
    --address-search

