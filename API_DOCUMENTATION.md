# Junkcoin Electrs Complete API Documentation

This document provides a comprehensive list of all available API endpoints in the Junkcoin Electrs implementation with detailed parameters, examples, and responses.

## Table of Contents

- [Block Endpoints](#block-endpoints)
- [Transaction Endpoints](#transaction-endpoints)
- [Address/Scripthash Endpoints](#addressscripthash-endpoints)
- [Mempool Endpoints](#mempool-endpoints)
- [Blockchain Endpoints](#blockchain-endpoints)
- [Fee Estimation Endpoints](#fee-estimation-endpoints)
- [Internal Endpoints](#internal-endpoints)
- [Transaction Testing Endpoints](#transaction-testing-endpoints)
- [Response Formats](#response-formats)

## Block Endpoints

### Get Block Tip Hash

```
GET /blocks/tip/hash
```

Returns the hash of the latest block in the blockchain.

**Parameters:** None

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/blocks/tip/hash
```

**Example Response:**
```
290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3
```

### Get Block Tip Height

```
GET /blocks/tip/height
```

Returns the height of the latest block in the blockchain.

**Parameters:** None

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/blocks/tip/height
```

**Example Response:**
```
437550
```

### Get Blocks

```
GET /blocks/{start_height}
```

Returns a list of blocks starting from the specified height (or latest if not specified).

**Parameters:**
- `start_height`: Optional. The height to start from (integer). If not provided, starts from the latest block.

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/blocks/437550
```

**Example Response:**
```json
[
  {
    "id": "290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3",
    "height": 437550,
    "version": 536870912,
    "timestamp": 1703123456,
    "tx_count": 1,
    "size": 285,
    "weight": 1140,
    "merkle_root": "abc123...",
    "previousblockhash": "def456...",
    "mediantime": 1703123400,
    "nonce": 123456789,
    "bits": "1a00ffff",
    "difficulty": 1.0
  }
]
```

### Get Block by Hash

```
GET /block/{hash}
```

Returns detailed information about a block with the given hash.

**Parameters:**
- `hash`: Block hash (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/block/290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3
```

**Example Response:**
```json
{
  "id": "290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3",
  "height": 437550,
  "version": 536870912,
  "timestamp": 1703123456,
  "tx_count": 1,
  "size": 285,
  "weight": 1140,
  "merkle_root": "abc123...",
  "previousblockhash": "def456...",
  "mediantime": 1703123400,
  "nonce": 123456789,
  "bits": "1a00ffff",
  "difficulty": 1.0
}
```

### Get Block Hash by Height

```
GET /block-height/{height}
```

Returns the block hash for the given height.

**Parameters:**
- `height`: Block height (integer)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/block-height/437550
```

**Example Response:**
```
290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3
```

### Get Block Header

```
GET /block/{hash}/header
```

Returns the block header for the given hash.

**Parameters:**
- `hash`: Block hash (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/block/290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3/header
```

**Example Response:**
```
0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49ffff001d1dac2b7c
```

### Get Block Status

```
GET /block/{hash}/status
```

Returns the confirmation status of a block.

**Parameters:**
- `hash`: Block hash (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/block/290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3/status
```

**Example Response:**
```json
{
  "in_best_chain": true,
  "height": 437550,
  "next_best": "abc123..."
}
```

### Get Block Transactions

```
GET /block/{hash}/txs/{start_index}
```

Returns transactions in a block, with optional pagination.

**Parameters:**
- `hash`: Block hash (string, 64 characters hex)
- `start_index`: Optional. Starting index for pagination (integer, default: 0)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/block/290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3/txs/0
```

### Get Block Transaction IDs

```
GET /block/{hash}/txids
```

Returns all transaction IDs in a block.

**Parameters:**
- `hash`: Block hash (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/block/290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3/txids
```

### Get Block Raw

```
GET /block/{hash}/raw
```

Returns the raw block data in binary format.

**Parameters:**
- `hash`: Block hash (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/block/290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3/raw
```

## Transaction Endpoints

### Get Transaction

```
GET /tx/{txid}
```

Returns detailed information about a transaction.

**Parameters:**
- `txid`: Transaction ID (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/tx/abc123def456...
```

**Example Response:**
```json
{
  "txid": "abc123def456...",
  "version": 1,
  "locktime": 0,
  "vin": [
    {
      "txid": "def456abc123...",
      "vout": 0,
      "prevout": {
        "scriptpubkey": "76a914...",
        "scriptpubkey_asm": "OP_DUP OP_HASH160 ... OP_EQUALVERIFY OP_CHECKSIG",
        "scriptpubkey_type": "p2pkh",
        "scriptpubkey_address": "7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3",
        "value": 1000000000
      },
      "scriptsig": "483045...",
      "scriptsig_asm": "3045... 03...",
      "witness": null,
      "is_coinbase": false,
      "sequence": 4294967295
    }
  ],
  "vout": [
    {
      "scriptpubkey": "76a914...",
      "scriptpubkey_asm": "OP_DUP OP_HASH160 ... OP_EQUALVERIFY OP_CHECKSIG",
      "scriptpubkey_type": "p2pkh",
      "scriptpubkey_address": "7fR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC4",
      "value": 999990000
    }
  ],
  "size": 225,
  "weight": 900,
  "fee": 10000,
  "status": {
    "confirmed": true,
    "block_height": 437550,
    "block_hash": "290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3",
    "block_time": 1703123456
  }
}
```

### Get Transaction Hex

```
GET /tx/{txid}/hex
```

Returns the raw transaction data in hexadecimal format.

**Parameters:**
- `txid`: Transaction ID (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/tx/abc123def456.../hex
```

**Example Response:**
```
0100000001def456abc123...
```

### Get Transaction Raw

```
GET /tx/{txid}/raw
```

Returns the raw transaction data in binary format.

**Parameters:**
- `txid`: Transaction ID (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/tx/abc123def456.../raw
```

### Get Transaction Status

```
GET /tx/{txid}/status
```

Returns the confirmation status of a transaction.

**Parameters:**
- `txid`: Transaction ID (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/tx/abc123def456.../status
```

**Example Response:**
```json
{
  "confirmed": true,
  "block_height": 437550,
  "block_hash": "290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3",
  "block_time": 1703123456
}
```

### Get Transaction Output Spending Status

```
GET /tx/{txid}/outspend/{vout}
```

Returns information about whether a specific transaction output has been spent.

**Parameters:**
- `txid`: Transaction ID (string, 64 characters hex)
- `vout`: Output index (integer)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/tx/abc123def456.../outspend/0
```

**Example Response:**
```json
{
  "spent": true,
  "txid": "def456abc123...",
  "vin": 0,
  "status": {
    "confirmed": true,
    "block_height": 437551,
    "block_hash": "123abc456def...",
    "block_time": 1703123500
  }
}
```

### Get Transaction Outputs Spending Status

```
GET /tx/{txid}/outspends
```

Returns spending information for all outputs of a transaction.

**Parameters:**
- `txid`: Transaction ID (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/tx/abc123def456.../outspends
```

**Example Response:**
```json
[
  {
    "spent": true,
    "txid": "def456abc123...",
    "vin": 0,
    "status": {
      "confirmed": true,
      "block_height": 437551,
      "block_hash": "123abc456def...",
      "block_time": 1703123500
    }
  },
  {
    "spent": false,
    "txid": null,
    "vin": null,
    "status": null
  }
]
```

### Get Transaction Merkle Proof

```
GET /tx/{txid}/merkle-proof
```

Returns the merkle proof for a transaction.

**Parameters:**
- `txid`: Transaction ID (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/tx/abc123def456.../merkle-proof
```

**Example Response:**
```json
{
  "block_height": 437550,
  "merkle": [
    "def456abc123...",
    "789ghi012jkl..."
  ],
  "pos": 1
}
```

## Address/Scripthash Endpoints

### Get Address Information

```
GET /address/{address}
```

Returns balance and transaction statistics for an address.

**Parameters:**
- `address`: Bitcoin address (string)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3
```

**Example Response:**
```json
{
  "address": "7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3",
  "chain_stats": {
    "funded_txo_count": 10,
    "funded_txo_sum": 1000000000,
    "spent_txo_count": 5,
    "spent_txo_sum": 500000000,
    "tx_count": 15
  },
  "mempool_stats": {
    "funded_txo_count": 0,
    "funded_txo_sum": 0,
    "spent_txo_count": 0,
    "spent_txo_sum": 0,
    "tx_count": 0
  }
}
```

### Get Scripthash Information

```
GET /scripthash/{scripthash}
```

Returns balance and transaction statistics for a scripthash.

**Parameters:**
- `scripthash`: Script hash (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/scripthash/abc123def456...
```

**Example Response:**
```json
{
  "scripthash": "abc123def456...",
  "chain_stats": {
    "funded_txo_count": 10,
    "funded_txo_sum": 1000000000,
    "spent_txo_count": 5,
    "spent_txo_sum": 500000000,
    "tx_count": 15
  },
  "mempool_stats": {
    "funded_txo_count": 0,
    "funded_txo_sum": 0,
    "spent_txo_count": 0,
    "spent_txo_sum": 0,
    "tx_count": 0
  }
}
```

### Get Address Transactions

```
GET /address/{address}/txs
```

Returns transactions for an address with pagination support.

**Parameters:**
- `address`: Bitcoin address (string)
- `start_index`: Optional. Starting index for pagination (integer, default: 0)
- `limit`: Optional. Number of transactions to return (integer, default: 25)
- `after_txid`: Optional. Return transactions after this txid (string)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs?start_index=0&limit=10
```

**Example Response:**
```json
[
  {
    "txid": "abc123def456...",
    "version": 1,
    "locktime": 0,
    "vin": [...],
    "vout": [...],
    "size": 225,
    "weight": 900,
    "fee": 10000,
    "status": {
      "confirmed": true,
      "block_height": 437550,
      "block_hash": "290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3",
      "block_time": 1703123456
    }
  }
]
```

### Get Address Chain Transactions

```
GET /address/{address}/txs/chain/{last_seen_txid}
```

Returns confirmed transactions for an address, with optional pagination from a specific transaction.

**Parameters:**
- `address`: Bitcoin address (string)
- `last_seen_txid`: Optional. Last seen transaction ID for pagination (string)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs/chain/abc123def456...
```

### Get Address Mempool Transactions

```
GET /address/{address}/txs/mempool
```

Returns unconfirmed transactions for an address.

**Parameters:**
- `address`: Bitcoin address (string)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs/mempool
```

### Get Address UTXOs

```
GET /address/{address}/utxo
```

Returns unspent transaction outputs (UTXOs) for an address with pagination support.

**Parameters:**
- `address`: Bitcoin address (string)
- `start_index`: Optional. Starting index for pagination (integer, default: 0)
- `limit`: Optional. Number of UTXOs to return (integer, default: all)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/utxo?start_index=0&limit=10
```

**Example Response:**
```json
[
  {
    "txid": "abc123def456...",
    "vout": 0,
    "status": {
      "confirmed": true,
      "block_height": 437550,
      "block_hash": "290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3",
      "block_time": 1703123456
    },
    "value": 1000000000
  }
]
```

### Get Address Balance

```
GET /address/{address}/balance
```

Returns the balance information for an address.

**Parameters:**
- `address`: Bitcoin address (string)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/balance
```

**Example Response:**
```json
{
  "confirm_amount": "10.00000000",
  "pending_amount": "0.00000000",
  "amount": "10.00000000",
  "confirm_coin_amount": "10.00000000",
  "pending_coin_amount": "0.00000000",
  "coin_amount": "10.00000000"
}
```

### Get Address Stats

```
GET /address/{address}/stats
```

Returns detailed statistics for an address.

**Parameters:**
- `address`: Bitcoin address (string)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/stats
```

**Example Response:**
```json
{
  "funded_txo_count": 10,
  "funded_txo_sum": 1000000000,
  "spent_txo_count": 5,
  "spent_txo_sum": 500000000,
  "tx_count": 15,
  "balance": 500000000,
  "first_seen_tx_time": 1703000000,
  "last_seen_tx_time": 1703123456
}
```

### Get Scripthash Transactions

```
GET /scripthash/{scripthash}/txs
```

Returns transactions for a scripthash with pagination support.

**Parameters:**
- `scripthash`: Script hash (string, 64 characters hex)
- `start_index`: Optional. Starting index for pagination (integer, default: 0)
- `limit`: Optional. Number of transactions to return (integer, default: 25)
- `after_txid`: Optional. Return transactions after this txid (string)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/scripthash/abc123def456.../txs?start_index=0&limit=10
```

### Get Scripthash Chain Transactions

```
GET /scripthash/{scripthash}/txs/chain/{last_seen_txid}
```

Returns confirmed transactions for a scripthash.

**Parameters:**
- `scripthash`: Script hash (string, 64 characters hex)
- `last_seen_txid`: Optional. Last seen transaction ID for pagination (string)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/scripthash/abc123def456.../txs/chain/def456abc123...
```

### Get Scripthash Mempool Transactions

```
GET /scripthash/{scripthash}/txs/mempool
```

Returns unconfirmed transactions for a scripthash.

**Parameters:**
- `scripthash`: Script hash (string, 64 characters hex)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/scripthash/abc123def456.../txs/mempool
```

### Get Scripthash UTXOs

```
GET /scripthash/{scripthash}/utxo
```

Returns unspent transaction outputs (UTXOs) for a scripthash with pagination support.

**Parameters:**
- `scripthash`: Script hash (string, 64 characters hex)
- `start_index`: Optional. Starting index for pagination (integer, default: 0)
- `limit`: Optional. Number of UTXOs to return (integer, default: all)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/scripthash/abc123def456.../utxo?start_index=0&limit=10
```

### Address Search

```
GET /address-prefix/{prefix}
```

Searches for addresses that start with the given prefix.

**Parameters:**
- `prefix`: Address prefix to search for (string, minimum 3 characters)

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/address-prefix/7gR
```

**Example Response:**
```json
[
  "7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3",
  "7gRabc123def456...",
  "7gRdef456abc123..."
]
```

## Mempool Endpoints

### Get Mempool

```
GET /mempool
```

Returns mempool information including transaction count and size.

**Parameters:** None

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/mempool
```

**Example Response:**
```json
{
  "count": 1234,
  "vsize": 567890,
  "total_fee": 12345678,
  "fee_histogram": [
    [1.0, 100],
    [2.0, 200],
    [5.0, 300]
  ]
}
```

### Get Mempool Transaction IDs

```
GET /mempool/txids
```

Returns all transaction IDs currently in the mempool.

**Parameters:** None

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/mempool/txids
```

**Example Response:**
```json
[
  "abc123def456...",
  "def456abc123...",
  "789ghi012jkl..."
]
```

### Get Recent Mempool Transactions

```
GET /mempool/recent
```

Returns recent transactions from the mempool.

**Parameters:** None

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/mempool/recent
```

**Example Response:**
```json
[
  {
    "txid": "abc123def456...",
    "fee": 10000,
    "vsize": 225,
    "value": 1000000000
  }
]
```

## Blockchain Endpoints

### Get Total Coin Supply

```
GET /blockchain/getsupply
```

Returns the total coin supply information.

**Parameters:** None

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/blockchain/getsupply
```

**Example Response:**
```json
{
  "total_amount": "18416576.32358400",
  "total_amount_float": 18416576.32358400,
  "height": 437550,
  "block_hash": "290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3"
}
```

## Fee Estimation Endpoints

### Get Fee Estimates

```
GET /fee-estimates
```

Returns fee estimates for different confirmation targets.

**Parameters:** None

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/fee-estimates
```

**Example Response:**
```json
{
  "1": 20.0,
  "2": 15.0,
  "3": 12.0,
  "6": 8.0,
  "10": 5.0,
  "20": 3.0,
  "144": 1.0,
  "504": 1.0,
  "1008": 1.0
}
```

## Internal Endpoints

### Broadcast Transaction

```
POST /tx
```

Broadcasts a raw transaction to the network.

**Parameters:**
- Body: Raw transaction in hexadecimal format (string)

**Example Request:**
```bash
curl -X POST https://junk-api.s3na.xyz/tx \
  -H "Content-Type: text/plain" \
  -d "0100000001abc123def456..."
```

**Example Response:**
```
abc123def456...
```

### Get Sync Status

```
GET /sync
```

Returns the synchronization status of the node.

**Parameters:** None

**Example Request:**
```bash
curl https://junk-api.s3na.xyz/sync
```

**Example Response:**
```json
{
  "height": 437550,
  "hash": "290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3",
  "progress": 1.0
}
```

## Response Formats

### Common Response Fields

All API responses include appropriate HTTP status codes and headers:

- **200 OK**: Successful request
- **400 Bad Request**: Invalid parameters or malformed request
- **404 Not Found**: Resource not found
- **500 Internal Server Error**: Server error

### Cache Headers

Responses include cache control headers:

- **Static resources** (confirmed blocks, transactions): `Cache-Control: public, max-age=157784630` (5 years)
- **Dynamic resources** (mempool, tips): `Cache-Control: public, max-age=10` (10 seconds)
- **Recent mempool**: `Cache-Control: public, max-age=5` (5 seconds)

### Content Types

- **JSON responses**: `Content-Type: application/json`
- **Text responses**: `Content-Type: text/plain`
- **Binary responses**: `Content-Type: application/octet-stream`

### Pagination

Many endpoints support pagination with the following parameters:

- `start_index`: Starting index (integer, default: 0)
- `limit`: Number of items to return (integer, varies by endpoint)
- `after_txid`: Return items after this transaction ID (string)

### Error Responses

Error responses follow this format:

```json
{
  "error": "Error message describing what went wrong"
}
```

### Transaction Status Object

```json
{
  "confirmed": true,
  "block_height": 437550,
  "block_hash": "290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3",
  "block_time": 1703123456
}
```

For unconfirmed transactions:

```json
{
  "confirmed": false,
  "block_height": null,
  "block_hash": null,
  "block_time": null
}
```

### Address/Scripthash Stats Object

```json
{
  "funded_txo_count": 10,
  "funded_txo_sum": 1000000000,
  "spent_txo_count": 5,
  "spent_txo_sum": 500000000,
  "tx_count": 15
}
```

### UTXO Object

```json
{
  "txid": "abc123def456...",
  "vout": 0,
  "status": {
    "confirmed": true,
    "block_height": 437550,
    "block_hash": "290e8c6c848b7bfde4f203446aaf2486231ae5ace072aca6432ec7b3e68702a3",
    "block_time": 1703123456
  },
  "value": 1000000000
}
```

## Rate Limiting

The API does not currently implement rate limiting, but it's recommended to:

- Limit requests to reasonable intervals
- Use pagination for large datasets
- Cache responses when appropriate
- Avoid unnecessary repeated requests

## CORS Support

The API supports Cross-Origin Resource Sharing (CORS) if configured. The `Access-Control-Allow-Origin` header will be included in responses when CORS is enabled.

## Base URL

The API is accessible at:
```
https://junk-api.s3na.xyz
```

## Blockchain Agnostic Design

This implementation is designed to be blockchain-agnostic. All blockchain-specific operations use configuration parameters to adapt to different blockchain types. The API endpoints remain consistent across different blockchain implementations.

## Notes

1. **Address vs Scripthash**: Both address and scripthash endpoints provide the same functionality. Addresses are converted to scripthashes internally.

2. **Pagination**: When using pagination, the total number of items may change between requests due to new transactions or blocks.

3. **Mempool**: Mempool data is volatile and changes frequently. Use appropriate cache headers.

4. **Binary Data**: Raw endpoints return binary data. Use appropriate tools to handle binary responses.

5. **Hex Encoding**: Transaction and block hashes are returned in hexadecimal format with lowercase letters.

6. **Satoshi Values**: All monetary values are returned in satoshis (smallest unit). To convert to coin units, divide by 100,000,000.

7. **Timestamps**: All timestamps are Unix timestamps (seconds since epoch).

## Example Usage Patterns

### Get Address Balance and Transactions
```bash
# Get address info
curl https://junk-api.s3na.xyz/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3

# Get address transactions with pagination
curl https://junk-api.s3na.xyz/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs?limit=10

# Get address UTXOs
curl https://junk-api.s3na.xyz/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/utxo
```

### Monitor New Blocks
```bash
# Get latest block height
curl https://junk-api.s3na.xyz/blocks/tip/height

# Get latest block details
curl https://junk-api.s3na.xyz/blocks/tip/hash | xargs -I {} curl https://junk-api.s3na.xyz/block/{}
```

### Track Transaction Status
```bash
# Get transaction details
curl https://junk-api.s3na.xyz/tx/abc123def456...

# Check if transaction outputs are spent
curl https://junk-api.s3na.xyz/tx/abc123def456.../outspends
```

This completes the comprehensive API documentation for the Junkcoin Electrs implementation.