# Pagination in Electrs API

This document describes how to use pagination in the Electrs API to efficiently retrieve large datasets.

## Why Pagination?

Some API endpoints can potentially return very large amounts of data, especially for addresses with many transactions or UTXOs. Pagination allows you to:

1. Reduce server load by retrieving only the data you need
2. Improve client performance by processing smaller chunks of data
3. Implement infinite scrolling or "load more" functionality in user interfaces

## Endpoints with Pagination Support

### Address Transactions (All)

```
GET /address/{address}/txs
GET /scripthash/{scripthash}/txs
```

These endpoints support both offset-based and cursor-based pagination:

- **Offset-based pagination**: Use `start_index` and `limit` query parameters
- **Cursor-based pagination**: Use the `after_txid` parameter
- **Filtering**: Use the `mempool` parameter to include/exclude mempool transactions

Example:
```
# First page (offset-based)
GET /address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs?start_index=0&limit=10

# Next page (offset-based)
GET /address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs?start_index=10&limit=10

# First page (cursor-based)
GET /address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs?limit=10

# Next page (cursor-based, using the next_page_after_txid from the previous response)
GET /address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs?after_txid=f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16&limit=10

# Only confirmed transactions
GET /address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs?mempool=false
```

The response includes pagination metadata:
```json
{
  "transactions": [...],
  "total": 150,
  "start_index": 0,
  "limit": 10,
  "next_page_after_txid": "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16"
}
```

### Address Transactions (Confirmed Only)

```
GET /address/{address}/txs/chain[/{last_seen_txid}]
GET /scripthash/{scripthash}/txs/chain[/{last_seen_txid}]
```

These endpoints support cursor-based pagination using the `last_seen_txid` parameter. You can also use the `max_txs` query parameter to limit the number of transactions returned.

Example:
```
# First page
GET /address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs/chain?max_txs=10

# Next page (using the last txid from the previous response)
GET /address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/txs/chain/f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16?max_txs=10
```

### Address UTXOs

```
GET /address/{address}/utxo
GET /scripthash/{scripthash}/utxo
```

These endpoints support offset-based pagination using the `start_index` and `limit` query parameters.

Example:
```
# First page (10 UTXOs)
GET /address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/utxo?start_index=0&limit=10

# Second page (next 10 UTXOs)
GET /address/7gR9M3RvDsHupPuSjHiCm2ZjhQAzZqxDC3/utxo?start_index=10&limit=10
```

The response includes pagination metadata:
```json
{
  "utxos": [...],
  "total": 100,
  "start_index": 0,
  "limit": 10
}
```

### Mempool Transactions

```
GET /mempool/txids
```

This endpoint supports offset-based pagination using the `start_index` and `limit` query parameters.

Example:
```
# First page (100 txids)
GET /mempool/txids?start_index=0&limit=100

# Second page (next 100 txids)
GET /mempool/txids?start_index=100&limit=100
```

The response includes pagination metadata:
```json
{
  "txids": [...],
  "total": 1000,
  "start_index": 0,
  "limit": 100
}
```

### Block Transactions

```
GET /block/{hash}/txs[/{start_index}]
```

This endpoint supports offset-based pagination using the `start_index` path parameter. The number of transactions per page is fixed at 25.

Example:
```
# First page (25 transactions)
GET /block/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f/txs

# Second page (next 25 transactions)
GET /block/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f/txs/25
```

## Optimized Balance Calculation for Large Addresses

For addresses with a very large number of transactions, calculating the balance can be resource-intensive. The address balance endpoint supports an `optimized` parameter to use a more efficient calculation method:

```
GET /address/{address}/balance?optimized=true
GET /scripthash/{scripthash}/balance?optimized=true
```

This uses a more efficient algorithm that directly sums the UTXOs instead of processing the full transaction history.

## Best Practices

1. **Always use pagination** for endpoints that might return large datasets
2. **Start with a reasonable page size** (10-100 items) and adjust based on your application's needs
3. **Use the optimized parameter** for balance calculations on addresses with many transactions
4. **Cache results** when appropriate to reduce server load
5. **Implement infinite scrolling or "load more" buttons** in user interfaces rather than loading all data at once

## Implementation Details

Pagination is implemented in different ways depending on the endpoint:

1. **Cursor-based pagination** (using `last_seen_txid`): This is used for transaction history endpoints and is more efficient for large datasets that change frequently.

2. **Offset-based pagination** (using `start_index` and `limit`): This is used for UTXOs and mempool endpoints and is simpler to implement but can be less efficient for very large datasets.

The server enforces maximum limits on the number of items returned per request to prevent abuse and ensure good performance.
