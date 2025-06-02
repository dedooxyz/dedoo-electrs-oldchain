# Electrs API Documentation

## Overview

Electrs is a Bitcoin/Altcoin blockchain index server that implements two main APIs:

1. **REST API** - A modern HTTP-based API for accessing blockchain data
2. **Electrum RPC API** - A JSON-RPC based API compatible with Electrum clients

## Server Configuration

The server can be configured with the following parameters:

- **HTTP REST API**: Default address `127.0.0.1:3000`
- **Electrum RPC API**: Default address `127.0.0.1:50001`
- **Monitoring API**: Default address `127.0.0.1:4224` (Prometheus metrics)

## REST API

### General Information

- **Content Type**: All JSON responses use `application/json` content type
- **Cache Control**: Responses include `Cache-Control` headers with appropriate TTL values:
  - `TTL_LONG`: 157,784,630 seconds (5 years) for static resources
  - `TTL_SHORT`: 10 seconds for volatile resources
  - `TTL_MEMPOOL_RECENT`: 5 seconds for mempool data

### Pagination

Many endpoints support pagination with the following query parameters:

- `start_index`: Starting index for pagination (0-based)
- `limit`: Maximum number of items to return
- `after_txid`: For cursor-based pagination in transaction lists

Default pagination limits:
- `CHAIN_TXS_PER_PAGE`: 25 transactions per page
- `MAX_MEMPOOL_TXS`: 50 mempool transactions
- `BLOCK_LIMIT`: 10 blocks
- `ADDRESS_SEARCH_LIMIT`: 10 addresses

For Liquid assets (when feature is enabled):
- `ASSETS_PER_PAGE`: 25 assets per page
- `ASSETS_MAX_PER_PAGE`: 100 assets maximum per page

### Endpoints

#### Block Information

1. **Get Latest Block Hash**
   - **Endpoint**: `GET /blocks/tip/hash`
   - **Response**: Plain text with the latest block hash
   - **TTL**: Short (10 seconds)

2. **Get Latest Block Height**
   - **Endpoint**: `GET /blocks/tip/height`
   - **Response**: Plain text with the latest block height
   - **TTL**: Short (10 seconds)

3. **Get Block List**
   - **Endpoint**: `GET /blocks[/<start_height>]`
   - **Parameters**:
     - `start_height` (optional): Starting block height
   - **Response**: JSON array of block information
   - **TTL**: Based on block depth

4. **Get Block by Height**
   - **Endpoint**: `GET /block-height/<height>`
   - **Parameters**:
     - `height`: Block height
   - **Response**: Plain text with the block hash
   - **TTL**: Based on block depth

5. **Get Block Details**
   - **Endpoint**: `GET /block/<hash>`
   - **Parameters**:
     - `hash`: Block hash
   - **Response**: JSON object with block details
   - **Response Fields**:
     ```json
     {
       "id": "block_hash",
       "height": 123456,
       "version": 536870912,
       "timestamp": 1598918400,
       "tx_count": 2000,
       "size": 1234567,
       "weight": 4000000,
       "merkle_root": "merkle_root_hash",
       "previousblockhash": "previous_block_hash",
       "mediantime": 1598917500,
       "nonce": 123456789,
       "bits": 386604799,
       "difficulty": 16105.35009435
     }
     ```
   - **TTL**: Long (5 years)

6. **Get Block Transactions**
   - **Endpoint**: `GET /block/<hash>/txs[/<start_index>]`
   - **Parameters**:
     - `hash`: Block hash
     - `start_index` (optional): Starting transaction index (must be a multiple of 25)
   - **Response**: JSON array of transaction objects
   - **Pagination**: Returns 25 transactions per page
   - **TTL**: Based on block depth

#### Transaction Information

1. **Get Transaction Details**
   - **Endpoint**: `GET /tx/<txid>`
   - **Parameters**:
     - `txid`: Transaction ID
   - **Response**: JSON object with transaction details
   - **Response Fields**:
     ```json
     {
       "txid": "transaction_id",
       "version": 2,
       "locktime": 0,
       "vin": [
         {
           "txid": "input_txid",
           "vout": 0,
           "prevout": {
             "scriptpubkey": "script_hex",
             "scriptpubkey_asm": "asm_script",
             "scriptpubkey_type": "p2pkh",
             "scriptpubkey_address": "address",
             "value": 100000000
           },
           "scriptsig": "script_hex",
           "scriptsig_asm": "asm_script",
           "witness": ["witness_data"],
           "is_coinbase": false,
           "sequence": 4294967295
         }
       ],
       "vout": [
         {
           "scriptpubkey": "script_hex",
           "scriptpubkey_asm": "asm_script",
           "scriptpubkey_type": "p2pkh",
           "scriptpubkey_address": "address",
           "value": 100000000
         }
       ],
       "size": 225,
       "weight": 900,
       "fee": 1000,
       "status": {
         "confirmed": true,
         "block_height": 123456,
         "block_hash": "block_hash",
         "block_time": 1598918400
       }
     }
     ```
   - **TTL**: Based on confirmation depth

2. **Get Transaction Merkle Proof**
   - **Endpoint**: `GET /tx/<txid>/merkle-proof`
   - **Parameters**:
     - `txid`: Transaction ID
   - **Response**: JSON object with merkle proof
   - **Response Fields**:
     ```json
     {
       "block_height": 123456,
       "merkle": ["hash1", "hash2", "hash3"],
       "pos": 5
     }
     ```
   - **TTL**: Based on confirmation depth

3. **Get Transaction Outspends**
   - **Endpoint**: `GET /tx/<txid>/outspends`
   - **Parameters**:
     - `txid`: Transaction ID
   - **Response**: JSON array of spending information for each output
   - **Response Fields**:
     ```json
     [
       {
         "spent": true,
         "txid": "spending_txid",
         "vin": 0,
         "status": {
           "confirmed": true,
           "block_height": 123456,
           "block_hash": "block_hash",
           "block_time": 1598918400
         }
       },
       {
         "spent": false
       }
     ]
     ```
   - **TTL**: Short (10 seconds)

#### Address/ScriptHash Information

1. **Get Address/ScriptHash Information**
   - **Endpoint**: `GET /address/<address>` or `GET /scripthash/<scripthash>`
   - **Parameters**:
     - `address`: Bitcoin address
     - `scripthash`: Script hash (hex)
   - **Response**: JSON object with address statistics
   - **Response Fields**:
     ```json
     {
       "address": "bitcoin_address",
       "chain_stats": {
         "funded_txo_count": 10,
         "funded_txo_sum": 1000000000,
         "spent_txo_count": 5,
         "spent_txo_sum": 500000000,
         "tx_count": 15
       },
       "mempool_stats": {
         "funded_txo_count": 1,
         "funded_txo_sum": 100000000,
         "spent_txo_count": 0,
         "spent_txo_sum": 0,
         "tx_count": 1
       }
     }
     ```
   - **TTL**: Short (10 seconds)

2. **Get Address/ScriptHash Transactions**
   - **Endpoint**: `GET /address/<address>/txs` or `GET /scripthash/<scripthash>/txs`
   - **Parameters**:
     - `address`: Bitcoin address
     - `scripthash`: Script hash (hex)
     - `start_index` (query, optional): Starting index for pagination
     - `limit` (query, optional): Maximum number of transactions to return
     - `after_txid` (query, optional): Return transactions after this txid
     - `mempool` (query, optional): Include mempool transactions (default: true)
   - **Response**: JSON object with transactions and pagination metadata
   - **Response Fields**:
     ```json
     {
       "transactions": [...],
       "total": 100,
       "start_index": 0,
       "limit": 25,
       "next_page_after_txid": "last_txid_in_page"
     }
     ```
   - **TTL**: Short (10 seconds)

3. **Get Address/ScriptHash UTXOs**
   - **Endpoint**: `GET /address/<address>/utxo` or `GET /scripthash/<scripthash>/utxo`
   - **Parameters**:
     - `address`: Bitcoin address
     - `scripthash`: Script hash (hex)
     - `start_index` (query, optional): Starting index for pagination
     - `limit` (query, optional): Maximum number of UTXOs to return
   - **Response**: JSON object with UTXOs and pagination metadata
   - **Response Fields**:
     ```json
     {
       "utxos": [
         {
           "txid": "transaction_id",
           "vout": 0,
           "value": 100000000,
           "status": {
             "confirmed": true,
             "block_height": 123456,
             "block_hash": "block_hash",
             "block_time": 1598918400
           }
         }
       ],
       "total": 10,
       "start_index": 0,
       "limit": 25
     }
     ```
   - **TTL**: Short (10 seconds)

4. **Get Address/ScriptHash Balance**
   - **Endpoint**: `GET /address/<address>/balance` or `GET /scripthash/<scripthash>/balance`
   - **Parameters**:
     - `address`: Bitcoin address
     - `scripthash`: Script hash (hex)
   - **Response**: JSON object with balance information
   - **Response Fields**:
     ```json
     {
       "confirm_amount": "1.00000000",
       "pending_amount": "0.50000000",
       "amount": "1.50000000",
       "confirm_coin_amount": "1.00000000",
       "pending_coin_amount": "0.50000000",
       "coin_amount": "1.50000000"
     }
     ```
   - **TTL**: Short (10 seconds)

5. **Get Address/ScriptHash Detailed Stats**
   - **Endpoint**: `GET /address/<address>/stats` or `GET /scripthash/<scripthash>/stats`
   - **Parameters**:
     - `address`: Bitcoin address
     - `scripthash`: Script hash (hex)
   - **Response**: JSON object with detailed statistics
   - **Response Fields**:
     ```json
     {
       "funded_txo_count": 10,
       "funded_txo_sum": 1000000000,
       "spent_txo_count": 5,
       "spent_txo_sum": 500000000,
       "tx_count": 15,
       "balance": 500000000,
       "first_seen_tx_time": 1598918400,
       "last_seen_tx_time": 1598918400
     }
     ```
   - **TTL**: Short (10 seconds)

6. **Search Addresses by Prefix**
   - **Endpoint**: `GET /address-prefix/<prefix>`
   - **Parameters**:
     - `prefix`: Address prefix to search for
   - **Response**: JSON array of matching addresses
   - **Limit**: Returns up to 10 matching addresses
   - **TTL**: Short (10 seconds)

#### Mempool Information

1. **Get Mempool Transaction IDs**
   - **Endpoint**: `GET /mempool/txids`
   - **Parameters**:
     - `start_index` (query, optional): Starting index for pagination
     - `limit` (query, optional): Maximum number of txids to return (default: 100)
   - **Response**: JSON object with txids and pagination metadata
   - **Response Fields**:
     ```json
     {
       "txids": ["txid1", "txid2", "txid3"],
       "total": 1000,
       "start_index": 0,
       "limit": 100
     }
     ```
   - **TTL**: Short (10 seconds)

2. **Get Recent Mempool Transactions**
   - **Endpoint**: `GET /mempool/recent`
   - **Response**: JSON array of recent transactions
   - **Limit**: Returns up to 50 recent transactions
   - **TTL**: Very short (5 seconds)

3. **Get Mempool Fee Histogram**
   - **Endpoint**: `GET /fee-estimates`
   - **Response**: JSON object mapping confirmation targets to fee rates (in sat/vB)
   - **Response Fields**:
     ```json
     {
       "1": 102.5,
       "2": 95.0,
       "3": 80.0,
       "6": 65.0,
       "10": 50.0,
       "20": 30.0,
       "144": 20.0,
       "504": 10.0,
       "1008": 5.0
     }
     ```
   - **TTL**: Short (10 seconds)

#### Liquid-specific Endpoints (when feature is enabled)

1. **Get Asset Registry List**
   - **Endpoint**: `GET /assets/registry`
   - **Parameters**:
     - `start_index` (query, optional): Starting index for pagination
     - `limit` (query, optional): Maximum number of assets to return
   - **Response**: JSON array of asset information
   - **Pagination**: Returns 25 assets per page by default, up to 100 maximum
   - **TTL**: No caching (Cache-Control: no-store)

2. **Get Asset Details**
   - **Endpoint**: `GET /asset/<asset_id>`
   - **Parameters**:
     - `asset_id`: Asset ID
   - **Response**: JSON object with asset details
   - **TTL**: Short (10 seconds)

## Electrum RPC API

The Electrum RPC API follows the JSON-RPC 2.0 specification. Requests and responses are sent over TCP.

### Protocol Information

- **Protocol Version**: 1.4
- **Server Version**: `electrs-esplora <version>`

### Request Format

```json
{
  "jsonrpc": "2.0",
  "id": <request_id>,
  "method": "<method_name>",
  "params": [<param1>, <param2>, ...]
}
```

### Response Format

```json
{
  "jsonrpc": "2.0",
  "id": <request_id>,
  "result": <result_data>
}
```

### Error Response Format

```json
{
  "jsonrpc": "2.0",
  "id": <request_id>,
  "error": "<error_message>"
}
```

### Methods

#### Server Information

1. **server.version**
   - **Parameters**: None
   - **Returns**: Array with server name and protocol version
   - **Example Response**: `["electrs-esplora 0.9.0", [1, 4]]`

2. **server.banner**
   - **Parameters**: None
   - **Returns**: Server banner text
   - **Example Response**: `"Welcome to Electrs!"`

3. **server.donation_address**
   - **Parameters**: None
   - **Returns**: Donation address (always null)
   - **Example Response**: `null`

4. **server.peers.subscribe**
   - **Parameters**: None
   - **Returns**: Array of known Electrum servers
   - **Example Response**: `[]` (empty if discovery is disabled)

5. **server.ping**
   - **Parameters**: None
   - **Returns**: Null
   - **Example Response**: `null`

6. **server.features** (when discovery is enabled)
   - **Parameters**: None
   - **Returns**: Server features object
   - **Example Response**: JSON object with server features

7. **server.add_peer** (when discovery is enabled)
   - **Parameters**: `[features]`
   - **Returns**: Boolean indicating success
   - **Example Response**: `true`

#### Blockchain Information

1. **blockchain.block.header**
   - **Parameters**: `[height, cp_height?]`
     - `height`: Block height
     - `cp_height` (optional): Checkpoint height
   - **Returns**: Block header in hex format, with optional merkle proof
   - **Example Response**: `"0100000000000000000000..."`

2. **blockchain.block.headers**
   - **Parameters**: `[start_height, count, cp_height?]`
     - `start_height`: Starting block height
     - `count`: Number of headers to return (max 2016)
     - `cp_height` (optional): Checkpoint height
   - **Returns**: Object with headers and optional merkle proof
   - **Example Response**:
     ```json
     {
       "count": 10,
       "hex": "0100000000000000000000...",
       "max": 2016
     }
     ```

3. **blockchain.estimatefee**
   - **Parameters**: `[blocks_count]`
     - `blocks_count`: Target confirmation blocks
   - **Returns**: Estimated fee rate in BTC/kB
   - **Example Response**: `0.00001`

4. **blockchain.headers.subscribe**
   - **Parameters**: None
   - **Returns**: Information about the latest block header
   - **Example Response**:
     ```json
     {
       "hex": "0100000000000000000000...",
       "height": 680000
     }
     ```
   - **Notifications**: Client receives updates when new blocks are found

5. **blockchain.relayfee**
   - **Parameters**: None
   - **Returns**: Minimum relay fee in BTC/kB
   - **Example Response**: `0.00001`

#### ScriptHash Operations

1. **blockchain.scripthash.get_balance**
   - **Parameters**: `[scripthash]`
     - `scripthash`: Script hash in hex format
   - **Returns**: Object with confirmed and unconfirmed balance
   - **Example Response**:
     ```json
     {
       "confirmed": 1000000,
       "unconfirmed": 500000
     }
     ```

2. **blockchain.scripthash.get_history**
   - **Parameters**: `[scripthash]`
     - `scripthash`: Script hash in hex format
   - **Returns**: Array of transaction history
   - **Example Response**:
     ```json
     [
       {
         "tx_hash": "transaction_id",
         "height": 680000
       }
     ]
     ```

3. **blockchain.scripthash.get_mempool**
   - **Parameters**: `[scripthash]`
     - `scripthash`: Script hash in hex format
   - **Returns**: Array of unconfirmed transactions
   - **Example Response**:
     ```json
     [
       {
         "tx_hash": "transaction_id",
         "height": 0,
         "fee": 1000
       }
     ]
     ```

4. **blockchain.scripthash.listunspent**
   - **Parameters**: `[scripthash]`
     - `scripthash`: Script hash in hex format
   - **Returns**: Array of unspent outputs
   - **Example Response**:
     ```json
     [
       {
         "tx_hash": "transaction_id",
         "tx_pos": 0,
         "height": 680000,
         "value": 1000000
       }
     ]
     ```

5. **blockchain.scripthash.subscribe**
   - **Parameters**: `[scripthash]`
     - `scripthash`: Script hash in hex format
   - **Returns**: Status hash (or null if no history)
   - **Example Response**: `"hash_string"`
   - **Notifications**: Client receives updates when scripthash status changes

#### Transaction Operations

1. **blockchain.transaction.broadcast**
   - **Parameters**: `[raw_tx]`
     - `raw_tx`: Raw transaction in hex format
   - **Returns**: Transaction ID if successful
   - **Example Response**: `"transaction_id"`

2. **blockchain.transaction.get**
   - **Parameters**: `[tx_hash, verbose?]`
     - `tx_hash`: Transaction ID
     - `verbose` (optional): If true, returns detailed transaction object
   - **Returns**: Raw transaction in hex format or detailed transaction object
   - **Example Response**: `"0100000001..."`

3. **blockchain.transaction.get_merkle**
   - **Parameters**: `[tx_hash, height]`
     - `tx_hash`: Transaction ID
     - `height`: Block height
   - **Returns**: Merkle proof for the transaction
   - **Example Response**:
     ```json
     {
       "block_height": 680000,
       "merkle": ["hash1", "hash2", "hash3"],
       "pos": 5
     }
     ```

4. **blockchain.transaction.id_from_pos**
   - **Parameters**: `[height, position, merkle?]`
     - `height`: Block height
     - `position`: Transaction position in block
     - `merkle` (optional): If true, includes merkle proof
   - **Returns**: Transaction ID and optional merkle proof
   - **Example Response**:
     ```json
     {
       "tx_hash": "transaction_id",
       "merkle": ["hash1", "hash2", "hash3"]
     }
     ```

#### Mempool Operations

1. **mempool.get_fee_histogram**
   - **Parameters**: None
   - **Returns**: Fee rate histogram for mempool transactions
   - **Example Response**:
     ```json
     [[1, 12000], [2, 10000], [5, 8000], [10, 5000]]
     ```

### Subscription Notifications

Electrum RPC supports real-time notifications for subscribed resources:

1. **blockchain.headers.subscribe**
   - **Notification Format**:
     ```json
     {
       "jsonrpc": "2.0",
       "method": "blockchain.headers.subscribe",
       "params": [{"hex": "...", "height": 680000}]
     }
     ```

2. **blockchain.scripthash.subscribe**
   - **Notification Format**:
     ```json
     {
       "jsonrpc": "2.0",
       "method": "blockchain.scripthash.subscribe",
       "params": ["scripthash", "status_hash"]
     }
     ```

## Pagination Mechanisms

### REST API Pagination

The REST API supports several pagination mechanisms:

1. **Index-based Pagination**
   - Used by most endpoints
   - Query parameters:
     - `start_index`: Starting index (0-based)
     - `limit`: Maximum number of items to return

2. **Cursor-based Pagination**
   - Used primarily for transaction lists
   - Query parameters:
     - `after_txid`: Return transactions after this transaction ID

3. **Path-based Pagination**
   - Used by some endpoints like block transactions
   - Format: `/block/<hash>/txs/<start_index>`
   - `start_index` must be a multiple of `CHAIN_TXS_PER_PAGE` (25)

### Electrum RPC Pagination

The Electrum RPC API has built-in pagination for methods that return large datasets:

1. **blockchain.block.headers**
   - Limited to `MAX_HEADERS` (2016) headers per request

2. **blockchain.scripthash.get_history**
   - Limited by the server's `txs_limit` configuration

## Error Handling

### REST API Errors

REST API errors are returned with appropriate HTTP status codes:

- `400 Bad Request`: Invalid parameters
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server-side error

Error responses include a plain text error message.

### Electrum RPC Errors

Electrum RPC errors follow the JSON-RPC 2.0 specification:

```json
{
  "jsonrpc": "2.0",
  "id": <request_id>,
  "error": "<error_message>"
}
```

## Cache Control

The REST API uses different cache TTL values based on the volatility of the data:

- `TTL_LONG` (157,784,630 seconds / 5 years): Static resources like confirmed blocks
- `TTL_SHORT` (10 seconds): Volatile resources like address balances
- `TTL_MEMPOOL_RECENT` (5 seconds): Highly volatile resources like mempool data

Block-related resources use dynamic TTL values based on confirmation depth:
- Recent blocks: Short TTL
- Blocks with 10+ confirmations: Long TTL (considered final)

## Monitoring

Electrs provides a Prometheus-compatible monitoring endpoint at the configured monitoring address (default: `127.0.0.1:4224`).

Available metrics include:
- Server performance statistics
- Client connection counts
- Subscription counts
- RPC method latency
- Mempool statistics
- Indexing performance

## Liquid-specific Features

When compiled with the `liquid` feature flag, Electrs provides additional functionality for the Liquid sidechain:

- Asset registry and metadata
- Confidential transactions support
- Peg-in/peg-out information
- Asset issuance details

These features are exposed through both the REST API and Electrum RPC API with appropriate extensions.
