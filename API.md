# Dedoo-Electrs API Documentation

> Complete API reference for dedoo-electrs - A Junkcoin blockchain indexer with gRPC lightwalletd API

## Overview

Dedoo-electrs is a blockchain indexer for Junkcoin that provides:
- **REST API** - HTTP endpoints for blockchain data
- **Electrum RPC** - Electrum protocol compatibility  
- **gRPC API** - Lightwalletd-compatible API with wallet ranking extensions

---

## Quick Start

```bash
# Start dedoo-electrs
./dedoo-electrs \
  --daemon-rpc-addr 127.0.0.1:19772 \
  --http-addr 127.0.0.1:50010 \
  --electrum-rpc-addr 127.0.0.1:50001 \
  --grpc-addr 127.0.0.1:9067

# With TLS for gRPC
./dedoo-electrs \
  --grpc-addr 0.0.0.0:9067 \
  --grpc-tls-cert /path/cert.pem \
  --grpc-tls-key /path/key.pem
```

---

## CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `--network` | Network type (mainnet/testnet) | mainnet |
| `--daemon-rpc-addr` | Junkcoin RPC address | 127.0.0.1:19771 |
| `--daemon-dir` | Junkcoin data directory | ~/.junkcoin |
| `--db-dir` | Index database directory | ./db |
| `--http-addr` | REST API address | 127.0.0.1:3000 |
| `--electrum-rpc-addr` | Electrum RPC address | 127.0.0.1:50001 |
| `--grpc-addr` | gRPC server address | disabled |
| `--grpc-tls-cert` | TLS certificate path (PEM) | none |
| `--grpc-tls-key` | TLS private key path (PEM) | none |
| `--light-mode` | Reduced memory usage | false |

---

## gRPC API (Port 9067)

### Connection

```bash
# Plain text
grpcurl -plaintext localhost:9067 <method>

# With TLS
grpcurl -insecure localhost:9067 <method>

# List available services
grpcurl -plaintext localhost:9067 list
```

### Service: `junkcoin.lightwallet.CompactTxStreamer`

---

### Server Information

#### GetLightdInfo

Get server information.

```protobuf
rpc GetLightdInfo(Empty) returns (LightdInfo)
```

**Response:**
```json
{
  "version": "0.4.1",
  "vendor": "dedoo-electrs",
  "chainName": "main",
  "blockHeight": 1234567,
  "taddrSupport": true,
  "saplingActivationHeight": 0
}
```

**Example:**
```bash
grpcurl -plaintext localhost:9067 \
  junkcoin.lightwallet.CompactTxStreamer/GetLightdInfo
```

---

### Block Operations

#### GetLatestBlock

Get the current chain tip.

```protobuf
rpc GetLatestBlock(Empty) returns (BlockID)
```

**Response:**
```json
{
  "height": 1234567,
  "hash": "base64_encoded_hash"
}
```

#### GetBlock

Get a compact block at specific height.

```protobuf
rpc GetBlock(BlockID) returns (CompactBlock)
```

**Request:**
```json
{
  "height": 1234567
}
```

**Response:**
```json
{
  "protoVersion": 1,
  "height": 1234567,
  "hash": "base64_encoded_hash",
  "prevHash": "base64_encoded_prev_hash",
  "time": 1703260800,
  "header": "base64_encoded_header",
  "vtx": [
    {
      "index": 0,
      "hash": "base64_encoded_txid",
      "fee": 10000,
      "inputs": [],
      "outputs": []
    }
  ]
}
```

#### GetBlockRange (Streaming)

Stream compact blocks for a range of heights.

```protobuf
rpc GetBlockRange(BlockRange) returns (stream CompactBlock)
```

**Request:**
```json
{
  "start": {"height": 100000},
  "end": {"height": 100100}
}
```

**Example:**
```bash
grpcurl -plaintext -d '{"start":{"height":100000},"end":{"height":100010}}' \
  localhost:9067 junkcoin.lightwallet.CompactTxStreamer/GetBlockRange
```

---

### Transaction Operations

#### GetTransaction

Get raw transaction bytes.

```protobuf
rpc GetTransaction(TxFilter) returns (RawTransaction)
```

**Request:**
```json
{
  "hash": "base64_encoded_txid"
}
```

**Response:**
```json
{
  "data": "base64_encoded_raw_tx",
  "height": 1234567
}
```

#### SendTransaction

Broadcast a signed transaction.

```protobuf
rpc SendTransaction(RawTransaction) returns (SendResponse)
```

**Request:**
```json
{
  "data": "base64_encoded_raw_tx"
}
```

**Response:**
```json
{
  "txid": "hex_encoded_txid"
}
```

---

### Address Operations

#### GetBalance

Get address balance.

```protobuf
rpc GetBalance(GetBalanceArg) returns (GetBalanceReply)
```

**Request:**
```json
{
  "address": "JXxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
}
```

**Response:**
```json
{
  "confirmedBalance": 100000000,
  "pendingBalance": 5000000
}
```

#### GetAddressUtxos

Get unspent transaction outputs for addresses.

```protobuf
rpc GetAddressUtxos(GetAddressUtxosArg) returns (GetAddressUtxosReply)
```

**Request:**
```json
{
  "addresses": ["JXaddr1", "JXaddr2"],
  "startHeight": 0,
  "maxEntries": 100
}
```

**Response:**
```json
{
  "addressUtxos": [
    {
      "address": "JXaddr1",
      "txid": "base64_encoded_txid",
      "index": 0,
      "valueSat": 100000000,
      "height": 1234567
    }
  ]
}
```

#### GetAddressUtxosStream (Streaming)

Stream UTXOs for addresses.

```protobuf
rpc GetAddressUtxosStream(GetAddressUtxosArg) returns (stream AddressUtxo)
```

#### GetAddressStats

Get detailed address statistics.

```protobuf
rpc GetAddressStats(GetAddressStatsArg) returns (GetAddressStatsReply)
```

**Request:**
```json
{
  "address": "JXxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
}
```

**Response:**
```json
{
  "fundedTxoCount": 50,
  "fundedTxoSum": 500000000,
  "spentTxoCount": 30,
  "spentTxoSum": 400000000,
  "txCount": 80,
  "balance": 100000000,
  "firstSeenTxTime": 1609459200,
  "lastSeenTxTime": 1703260800
}
```

#### GetTxHistory

Get transaction history with balance changes.

```protobuf
rpc GetTxHistory(GetTxHistoryArg) returns (GetTxHistoryReply)
```

**Request:**
```json
{
  "address": "JXxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "limit": 25,
  "offset": 0
}
```

**Response:**
```json
{
  "transactions": [
    {
      "txid": "base64_encoded_txid",
      "height": 1234567,
      "time": 1703260800,
      "balanceChange": 50000000,
      "fee": 10000
    },
    {
      "txid": "base64_encoded_txid",
      "height": 1234500,
      "time": 1703200000,
      "balanceChange": -25000000,
      "fee": 10000
    }
  ],
  "totalCount": 80
}
```

**Note:** `balanceChange` is positive for incoming, negative for outgoing.

---

### Wallet Ranking (Junkcoin Extension)

#### GetTopWallets

Get richest wallets sorted by balance.

```protobuf
rpc GetTopWallets(GetTopWalletsArg) returns (GetTopWalletsReply)
```

**Request:**
```json
{
  "limit": 100,
  "offset": 0
}
```

**Response:**
```json
{
  "wallets": [
    {
      "address": "scripthash:a1b2c3d4",
      "balance": 1000000000000,
      "txCount": 5000,
      "firstSeenTime": 1609459200,
      "lastSeenTime": 1703260800,
      "rank": 1
    }
  ],
  "totalWallets": 50000,
  "totalSupply": 21000000000000
}
```

**Example:**
```bash
grpcurl -plaintext -d '{"limit":100}' localhost:9067 \
  junkcoin.lightwallet.CompactTxStreamer/GetTopWallets
```

#### GetWalletRank

Get a specific wallet's rank.

```protobuf
rpc GetWalletRank(GetWalletRankArg) returns (GetWalletRankReply)
```

**Request:**
```json
{
  "address": "JXxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
}
```

**Response:**
```json
{
  "rank": 42,
  "percentile": 99.9,
  "totalWallets": 50000,
  "balance": 500000000000,
  "txCount": 1000
}
```

#### GetRichList

Get filtered rich list.

```protobuf
rpc GetRichList(GetRichListArg) returns (GetRichListReply)
```

**Request:**
```json
{
  "limit": 100,
  "offset": 0,
  "minBalance": 1000000000
}
```

---

### Network Information

#### GetMempool

Get mempool transactions.

```protobuf
rpc GetMempool(GetMempoolArg) returns (GetMempoolReply)
```

**Request:**
```json
{
  "limit": 100
}
```

**Response:**
```json
{
  "txids": ["base64_txid1", "base64_txid2"],
  "totalCount": 50
}
```

#### GetSupplyInfo

Get total and circulating supply.

```protobuf
rpc GetSupplyInfo(Empty) returns (GetSupplyInfoReply)
```

**Response:**
```json
{
  "totalSupply": 21000000000000,
  "circulatingSupply": 18500000000000,
  "height": 1234567
}
```

---

## REST API (Port 50010)

### Blocks

| Endpoint | Description |
|----------|-------------|
| `GET /blocks/tip/height` | Current block height |
| `GET /blocks/tip/hash` | Current block hash |
| `GET /block/:hash` | Block by hash |
| `GET /block-height/:height` | Block hash at height |

### Transactions

| Endpoint | Description |
|----------|-------------|
| `GET /tx/:txid` | Transaction details |
| `GET /tx/:txid/hex` | Raw transaction hex |
| `GET /tx/:txid/status` | Confirmation status |
| `POST /tx` | Broadcast transaction |

### Addresses

| Endpoint | Description |
|----------|-------------|
| `GET /address/:addr` | Address info |
| `GET /address/:addr/txs` | Address transactions |
| `GET /address/:addr/utxo` | Address UTXOs |
| `GET /scripthash/:hash/utxo` | Scripthash UTXOs |

---

## Electrum RPC (Port 50001)

Compatible with Electrum protocol. Supports:

- `blockchain.scripthash.get_balance`
- `blockchain.scripthash.get_history`
- `blockchain.scripthash.listunspent`
- `blockchain.scripthash.subscribe`
- `blockchain.transaction.get`
- `blockchain.transaction.broadcast`
- `blockchain.headers.subscribe`
- `server.banner`
- `server.features`
- `server.version`

---

## Background Services

### Ranking Indexer

Automatically indexes wallet balances on startup:

```
[INFO] Starting background wallet ranking indexer (low priority)...
[INFO] Performance settings: batch_size=500, yield_ms=10
[INFO] Ranking indexer: 5000 scripthashes (1200/sec), 1800 with balance
[INFO] Full scan complete: 50000 scripthashes, 18000 with positive balance
[INFO] Entering incremental update mode (checking every 30s)
```

**Features:**
- Low priority thread (nice 10)
- Batch processing with yields
- Incremental updates on new blocks

---

## Error Codes

| gRPC Code | Description |
|-----------|-------------|
| `NOT_FOUND` | Block/transaction not found |
| `INVALID_ARGUMENT` | Invalid address format |
| `INTERNAL` | Server error |
| `UNAVAILABLE` | Service temporarily unavailable |

---

## Data Types

### Balance Values

All balance values are in **satoshis** (1 JKC = 100,000,000 satoshis).

### Hashes

- Transaction IDs: 32 bytes, little-endian
- Block hashes: 32 bytes, little-endian
- Scripthashes: SHA256 of scriptPubKey

### Timestamps

Unix timestamps in seconds.

---

## Version

- **Dedoo-Electrs Version:** 0.4.1
- **Protocol Version:** 1
- **gRPC Service:** junkcoin.lightwallet.CompactTxStreamer
