# Junkcoin API Documentation

Complete HTTP API reference for accessing Junkcoin blockchain data.

## Base URL

```
https://api.junk-coin.com
```

## Address Endpoints

### Get Address Information
```
GET /address/{address}
```

Returns balance and transaction statistics for an address.

Example Request:
```bash
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3
```

Response:
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

### Get Address Transactions
```
GET /address/{address}/txs
GET /scripthash/{scripthash}/txs
```

Returns transaction history for an address or scripthash (mempool + confirmed) with pagination support.

Parameters:
- start_index: Optional. Integer. Starting index for pagination. Default: 0.
- limit: Optional. Integer. Maximum number of transactions to return. Default: 25.
- after_txid: Optional. String. Transaction ID after which to start returning results (cursor-based pagination).
- mempool: Optional. Boolean. Whether to include mempool transactions. Default: true.

Example Request:
```bash
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs?limit=10
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs?after_txid=f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs?mempool=false
```

Response:
```json
{
  "transactions": [...],
  "total": 150,
  "start_index": 0,
  "limit": 25,
  "next_page_after_txid": "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16"
}
```

### Get Confirmed Transactions
```
GET /address/{address}/txs/chain[/:last_seen_txid]
```

Returns confirmed transactions for an address (25 per page).

Parameters:
- last_seen_txid: Optional. Get transactions after this txid.

Example Request:
```bash
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs/chain
```

### Get Mempool Transactions
```
GET /address/{address}/txs/mempool
```

Returns unconfirmed transactions for an address (up to 50).

Example Request:
```bash
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs/mempool
```

### Get Address Balance
```
GET /address/{address}/balance
GET /scripthash/{scripthash}/balance
```

Returns formatted balance information for an address or scripthash.

Parameters:
- optimized: Optional. Boolean (true/false). Use optimized calculation method for large addresses. Default: false.

Example Request:
```bash
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/balance
curl https://api.junk-coin.com/scripthash/8b01df4e368ea28f8dc0423bcf7a4923e3a12d307c875e47a0cfbf90b5c39161/balance
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/balance?optimized=true
```

Response:
```json
{
  "confirm_amount": "0.12345678",
  "pending_amount": "0.00000000",
  "amount": "0.12345678",
  "confirm_coin_amount": "0.12345678",
  "pending_coin_amount": "0.00000000",
  "coin_amount": "0.12345678"
}
```

### Get Address UTXO
```
GET /address/{address}/utxo
GET /scripthash/{scripthash}/utxo
```

Returns unspent transaction outputs for an address or scripthash with pagination support.

Parameters:
- start_index: Optional. Integer. Starting index for pagination. Default: 0.
- limit: Optional. Integer. Maximum number of UTXOs to return. Default: config.utxos_limit.

Example Request:
```bash
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/utxo
curl https://api.junk-coin.com/address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/utxo?start_index=0&limit=10
```

Response:
```json
{
  "utxos": [
    {
      "txid": "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
      "vout": 0,
      "value": 500000000,
      "status": {
        "confirmed": true,
        "block_height": 170,
        "block_hash": "00000000d1145790a8694403d4063f323d499e655c83426834d4ce2f8dd4a2ee",
        "block_time": 1231731025
      }
    }
  ],
  "total": 1,
  "start_index": 0,
  "limit": 100
}
```

## Transaction Endpoints

### Get Transaction
```
GET /tx/{txid}
```

Returns detailed transaction information.

Example Request:
```bash
curl https://api.junk-coin.com/tx/f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16
```

Response:
```json
{
  "txid": "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
  "version": 1,
  "locktime": 0,
  "vin": [
    {
      "txid": "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9",
      "vout": 0,
      "sequence": 4294967295,
      "scriptsig": "47304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901"
    }
  ],
  "vout": [
    {
      "value": 10.00000000,
      "n": 0,
      "scriptpubkey": {
        "asm": "OP_DUP OP_HASH160 7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3 OP_EQUALVERIFY OP_CHECKSIG",
        "hex": "76a9147gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC388ac",
        "type": "pubkeyhash",
        "addresses": ["7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3"]
      }
    }
  ],
  "size": 225,
  "weight": 900,
  "fee": 50000,
  "status": {
    "confirmed": true,
    "block_height": 170,
    "block_hash": "00000000d1145790a8694403d4063f323d499e655c83426834d4ce2f8dd4a2ee",
    "block_time": 1231731025
  }
}
```

### Get Transaction Status
```
GET /tx/{txid}/status
```

Returns transaction confirmation status.

Example Request:
```bash
curl https://api.junk-coin.com/tx/f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16/status
```

Response:
```json
{
  "confirmed": true,
  "block_height": 170,
  "block_hash": "00000000d1145790a8694403d4063f323d499e655c83426834d4ce2f8dd4a2ee",
  "block_time": 1231731025
}
```

### Get Raw Transaction
```
GET /tx/{txid}/hex
```

Returns the raw transaction in hexadecimal format.

Example Request:
```bash
curl https://api.junk-coin.com/tx/f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16/hex
```

### Get Transaction Merkle Proof
```
GET /tx/{txid}/merkle-proof
```

Returns merkle inclusion proof for the transaction.

Example Request:
```bash
curl https://api.junk-coin.com/tx/f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16/merkle-proof
```

## Block Endpoints

### Get Block
```
GET /block/{hash}
```

Returns detailed block information.

Example Request:
```bash
curl https://api.junk-coin.com/block/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f
```

Response:
```json
{
  "id": "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
  "height": 0,
  "version": 1,
  "timestamp": 1231006505,
  "tx_count": 1,
  "size": 285,
  "weight": 816,
  "merkle_root": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
  "previousblockhash": null,
  "nonce": 2083236893,
  "bits": 486604799,
  "difficulty": 1
}
```

### Get Block Status
```
GET /block/{hash}/status
```

Returns block confirmation status.

Example Request:
```bash
curl https://api.junk-coin.com/block/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f/status
```

### Get Block Transactions
```
GET /block/{hash}/txs[/:start_index]
```

Returns transactions in the block (25 per page).

Parameters:
- start_index: Optional. Start from this transaction index.

Example Request:
```bash
curl https://api.junk-coin.com/block/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f/txs
```

### Get Block Raw Data
```
GET /block/{hash}/raw
```

Returns the raw block data in hexadecimal format.

Example Request:
```bash
curl https://api.junk-coin.com/block/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f/raw
```

## Blockchain Endpoints

### Get Total Coin Supply
```
GET /blockchain/total-coin
```

Returns the total coin supply information.

Example Request:
```bash
curl https://api.junk-coin.com/blockchain/total-coin
```

Response:
```json
{
  "total_amount": "21000000.00000000",
  "total_amount_float": 21000000.0,
  "height": 680000,
  "block_hash": "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
}
```

### Get Top Holders
```
GET /blockchain/top-holders
```

Returns the top holders (addresses with highest balances) on the blockchain.

Parameters:
- limit: Optional. Number of top holders to return. Default: 100, Maximum: 1000.
- start_index: Optional. Index to start from (for pagination). Default: 0.

Example Request:
```bash
curl https://api.junk-coin.com/blockchain/top-holders
curl https://api.junk-coin.com/blockchain/top-holders?limit=10
curl https://api.junk-coin.com/blockchain/top-holders?start_index=10&limit=10
```

Response:
```json
{
  "holders": [
    {
      "address": "7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3",
      "balance": 1000000000,
      "balance_amount": "10.00000000"
    },
    {
      "address": "7fR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC4",
      "balance": 500000000,
      "balance_amount": "5.00000000"
    },
    ...
  ],
  "total_holders": 100,
  "height": 680000,
  "block_hash": "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
}
```

## Mempool Endpoints

### Get Mempool Transaction IDs
```
GET /mempool/txids
```

Returns transaction IDs in the mempool with pagination support.

Parameters:
- start_index: Optional. Integer. Starting index for pagination. Default: 0.
- limit: Optional. Integer. Maximum number of txids to return. Default: 100.

Example Request:
```bash
curl https://api.junk-coin.com/mempool/txids
curl https://api.junk-coin.com/mempool/txids?start_index=0&limit=50
```

Response:
```json
{
  "txids": ["f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16", "..."],
  "total": 150,
  "start_index": 0,
  "limit": 50
}
```

### Get Mempool Recent Transactions
```
GET /mempool/recent
```

Returns the latest transactions that entered the mempool.

Example Request:
```bash
curl https://api.junk-coin.com/mempool/recent
```

## Developer Code Examples

### Python Examples

#### Get Address Balance
```python
import requests

# Method 1: Using the new balance endpoint
def get_address_formatted_balance(address):
    """Get the formatted balance of a Junkcoin address"""
    try:
        response = requests.get(f'https://api.junk-coin.com/address/{address}/balance')
        if response.status_code == 200:
            return response.json()
        else:
            print(f'Error: {response.status_code}')
            return None
    except Exception as e:
        print(f'Error: {str(e)}')
        return None

# Example usage
balance = get_address_formatted_balance('7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3')
if balance:
    print(f'Confirmed Balance: {balance["confirm_amount"]} JKC')
    print(f'Pending Balance: {balance["pending_amount"]} JKC')
    print(f'Total Balance: {balance["amount"]} JKC')

# Method 2: Using the address stats endpoint
def get_address_balance(address):
    """Get the balance of a Junkcoin address in satoshis"""
    try:
        response = requests.get(f'https://api.junk-coin.com/address/{address}')
        if response.status_code == 200:
            data = response.json()
            confirmed_balance = data['chain_stats']['funded_txo_sum'] - data['chain_stats']['spent_txo_sum']
            unconfirmed_balance = data['mempool_stats']['funded_txo_sum'] - data['mempool_stats']['spent_txo_sum']
            return {
                'confirmed': confirmed_balance,
                'unconfirmed': unconfirmed_balance,
                'total': confirmed_balance + unconfirmed_balance
            }
        else:
            print(f'Error: {response.status_code}')
            return None
    except Exception as e:
        print(f'Error: {str(e)}')
        return None

# Example usage
balance = get_address_balance('7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3')
if balance:
    print(f'Confirmed Balance: {balance["confirmed"]} satoshis')
    print(f'Unconfirmed Balance: {balance["unconfirmed"]} satoshis')
    print(f'Total Balance: {balance["total"]} satoshis')
```

#### Get Transaction History
```python
import requests

def get_address_transactions(address, include_mempool=True):
    """Get transaction history for a Junkcoin address"""
    try:
        # Get confirmed transactions
        confirmed_txs = requests.get(f'https://api.junk-coin.com/address/{address}/txs/chain').json()

        # Get mempool transactions if requested
        mempool_txs = []
        if include_mempool:
            mempool_txs = requests.get(f'https://api.junk-coin.com/address/{address}/txs/mempool').json()

        return {
            'confirmed': confirmed_txs,
            'mempool': mempool_txs
        }
    except Exception as e:
        print(f'Error: {str(e)}')
        return None

# Example usage
txs = get_address_transactions('7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3')
if txs:
    print(f'Confirmed Transactions: {len(txs["confirmed"])}')
    print(f'Mempool Transactions: {len(txs["mempool"])}')
```

### JavaScript Examples

#### Get Address Balance
```javascript
// Method 1: Using the new balance endpoint
async function getAddressFormattedBalance(address) {
    try {
        const response = await fetch(`https://api.junk-coin.com/address/${address}/balance`);
        if (response.ok) {
            const balance = await response.json();
            return balance;
        } else {
            console.error(`Error: ${response.status}`);
            return null;
        }
    } catch (error) {
        console.error('Error:', error);
        return null;
    }
}

// Example usage
getAddressFormattedBalance('7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3')
    .then(balance => {
        if (balance) {
            console.log(`Confirmed Balance: ${balance.confirm_amount} JKC`);
            console.log(`Pending Balance: ${balance.pending_amount} JKC`);
            console.log(`Total Balance: ${balance.amount} JKC`);
        }
    });

// Method 2: Using the address stats endpoint
async function getAddressBalance(address) {
    try {
        const response = await fetch(`https://api.junk-coin.com/address/${address}`);
        if (response.ok) {
            const data = await response.json();
            const confirmedBalance = data.chain_stats.funded_txo_sum - data.chain_stats.spent_txo_sum;
            const unconfirmedBalance = data.mempool_stats.funded_txo_sum - data.mempool_stats.spent_txo_sum;

            return {
                confirmed: confirmedBalance,
                unconfirmed: unconfirmedBalance,
                total: confirmedBalance + unconfirmedBalance
            };
        } else {
            console.error(`Error: ${response.status}`);
            return null;
        }
    } catch (error) {
        console.error('Error:', error);
        return null;
    }
}

// Example usage
getAddressBalance('7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3')
    .then(balance => {
        if (balance) {
            console.log(`Confirmed Balance: ${balance.confirmed} satoshis`);
            console.log(`Unconfirmed Balance: ${balance.unconfirmed} satoshis`);
            console.log(`Total Balance: ${balance.total} satoshis`);
        }
    });
```

#### Get UTXO Set
```javascript
async function getAddressUtxos(address) {
    try {
        const response = await fetch(`https://api.junk-coin.com/address/${address}/utxo`);
        if (response.ok) {
            const utxos = await response.json();
            return utxos.map(utxo => ({
                txid: utxo.txid,
                vout: utxo.vout,
                value: utxo.value,
                status: utxo.status
            }));
        } else {
            console.error(`Error: ${response.status}`);
            return null;
        }
    } catch (error) {
        console.error('Error:', error);
        return null;
    }
}

// Example usage
getAddressUtxos('7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3')
    .then(utxos => {
        if (utxos) {
            console.log('UTXOs:', utxos);
            const total = utxos.reduce((sum, utxo) => sum + utxo.value, 0);
            console.log(`Total Value: ${total} satoshis`);
        }
    });
```

## Error Handling

The API uses standard HTTP status codes:

- 200: Success
- 400: Bad Request - Invalid parameters
- 404: Not Found - Resource doesn't exist
- 429: Too Many Requests - Rate limit exceeded
- 500: Internal Server Error - Server-side error

Error Response Format:
```json
{
  "error": "Detailed error message"
}
```

## Rate Limiting

The API implements rate limiting to ensure fair usage. When rate limits are exceeded, the API will return a 429 status code.

Rate limit headers are included in all responses:
```
X-RateLimit-Limit: Maximum requests per window
X-RateLimit-Remaining: Remaining requests in current window
X-RateLimit-Reset: Time when the rate limit resets (Unix timestamp)
```

## Best Practices

1. Implement proper error handling in your code
2. Cache responses when appropriate
3. Respect rate limits and implement backoff strategies
4. Use pagination for large datasets
5. Monitor API response times and implement timeouts
6. Keep your client libraries updated

## Support

For API support or to report issues, please visit our GitHub repository or contact our support team.
