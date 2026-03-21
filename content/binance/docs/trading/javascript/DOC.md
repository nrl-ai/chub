---
name: trading
description: "Binance API JavaScript/TypeScript coding guidelines for trading using official libraries and SDKs"
metadata:
  languages: "javascript"
  versions: "3.6.1"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "binance,trading,crypto,exchange,api"
---

# Binance API JavaScript/TypeScript Coding Guidelines

You are a Binance API coding expert. Help me with writing code using the Binance API calling the official libraries and SDKs.

You can find the official documentation here:
https://developers.binance.com/docs/binance-spot-api-docs

## Golden Rule: Use the Correct and Current SDK

Always use the official Binance Connector for Node.js, which is the standard library for all Binance Spot API interactions. Do not use unofficial or third-party libraries.

- **Library Name:** Binance Connector for Node.js
- **NPM Package:** `@binance/connector`
- **Alternative Official Packages:** `@binance/spot` (newer modular package), `@binance/futures-connector`, `@binance/pay`
- **Unofficial Libraries:** `binance-api-node`, `node-binance-api` (not recommended for production)

**Installation:**

- **Correct:** `npm install @binance/connector`
- **Alternative:** `npm install @binance/spot` (for newer modular approach)

**APIs and Usage:**

- **Correct:** `const { Spot } = require('@binance/connector')`
- **Correct:** `const client = new Spot(apiKey, apiSecret)`
- **Correct:** `client.account()` for account information
- **Correct:** `client.newOrder()` for placing orders
- **Incorrect:** Using unofficial packages like `binance-api-node`
- **Incorrect:** Direct HTTP requests without using the connector

## Installation

Install the official Binance connector for Node.js:

```bash
npm install @binance/connector
```

For TypeScript projects, the package includes built-in type definitions.

**Environment Variables:**

Set your API credentials as environment variables:

```bash
export BINANCE_API_KEY='your_api_key_here'
export BINANCE_API_SECRET='your_api_secret_here'
```

Or create a `.env` file:

```bash
BINANCE_API_KEY=your_api_key_here
BINANCE_API_SECRET=your_api_secret_here
```

## Initialization

The `@binance/connector` library requires creating a `Spot` instance for all API calls.

### Basic Initialization (Public Endpoints)

For public market data endpoints that don't require authentication:

```javascript
const { Spot } = require('@binance/connector')

// Public endpoints only (no authentication)
const client = new Spot()

// Use client for public data
client.exchangeInfo()
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Authenticated Initialization (HMAC)

For trading and account endpoints:

```javascript
const { Spot } = require('@binance/connector')

const apiKey = process.env.BINANCE_API_KEY
const apiSecret = process.env.BINANCE_API_SECRET

// Authenticated client with HMAC
const client = new Spot(apiKey, apiSecret)

// Use client for authenticated endpoints
client.account()
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### RSA/ED25519 Authentication

For enhanced security using RSA or ED25519 private keys:

```javascript
const { Spot, PrivateKeyAlgo } = require('@binance/connector')
const fs = require('fs')

const apiKey = process.env.BINANCE_API_KEY
const privateKey = fs.readFileSync('/path/to/private_key.pem', 'utf8')
const privateKeyPassphrase = 'your_passphrase' // Optional
const privateKeyAlgo = PrivateKeyAlgo.RSA // or PrivateKeyAlgo.ED25519

const client = new Spot(apiKey, '', {
  privateKey,
  privateKeyPassphrase,
  privateKeyAlgo
})
```

### Client Configuration Options

```javascript
const { Spot } = require('@binance/connector')

const client = new Spot(apiKey, apiSecret, {
  baseURL: 'https://api.binance.com', // Default base URL
  timeout: 10000, // Request timeout in milliseconds (default: 10000)
  recvWindow: 5000, // API timing security (default: 5000)
  logger: console // Custom logger (optional)
})
```

### Testnet Configuration

For testing without real funds:

```javascript
const { Spot } = require('@binance/connector')

const client = new Spot(apiKey, apiSecret, {
  baseURL: 'https://testnet.binance.vision'
})
```

### Proxy Configuration

For requests through a proxy:

```javascript
const { Spot } = require('@binance/connector')

const client = new Spot(apiKey, apiSecret, {
  proxy: {
    protocol: 'https',
    host: '127.0.0.1',
    port: 9000,
    auth: {
      username: 'proxy_user',
      password: 'proxy_password'
    }
  }
})
```

## Market Data Endpoints

Market data endpoints provide access to public trading information without authentication.

### Exchange Information

Get exchange trading rules and symbol information:

```javascript
const { Spot } = require('@binance/connector')

const client = new Spot()

// Get all exchange information
client.exchangeInfo()
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get specific symbol information
client.exchangeInfo({ symbol: 'BTCUSDT' })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get information for multiple symbols
client.exchangeInfo({ symbols: ['BTCUSDT', 'ETHUSDT'] })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Order Book (Depth)

Get current order book for a symbol:

```javascript
// Get order book with default limit (100)
client.depth('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get order book with specific limit (5, 10, 20, 50, 100, 500, 1000, 5000)
client.depth('BTCUSDT', { limit: 10 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Recent Trades

Get recent trades for a symbol:

```javascript
// Get recent trades (default limit: 500)
client.trades('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get specific number of recent trades
client.trades('BTCUSDT', { limit: 100 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Historical Trades

Get older trades (requires API key):

```javascript
const client = new Spot(apiKey, apiSecret)

// Get historical trades
client.historicalTrades('BTCUSDT', { limit: 100 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get trades from specific ID
client.historicalTrades('BTCUSDT', { limit: 100, fromId: 28457 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Aggregate Trades

Get compressed, aggregate trades:

```javascript
// Get recent aggregate trades
client.aggTrades('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get aggregate trades with time range
client.aggTrades('BTCUSDT', {
  startTime: 1609459200000,
  endTime: 1609545600000,
  limit: 1000
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get aggregate trades from specific ID
client.aggTrades('BTCUSDT', { fromId: 28457, limit: 500 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Candlestick Data (Klines)

Get kline/candlestick bars for a symbol:

```javascript
// Get recent klines (default limit: 500)
client.klines('BTCUSDT', '1h')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get klines with time range
client.klines('BTCUSDT', '1d', {
  startTime: 1609459200000,
  endTime: 1609545600000,
  limit: 100
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Available intervals: 1s, 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M
```

### Current Average Price

Get current average price for a symbol:

```javascript
client.avgPrice('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### 24hr Ticker Price Change Statistics

Get 24-hour rolling window price change statistics:

```javascript
// Get ticker for single symbol
client.ticker24hr('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get ticker for multiple symbols
client.ticker24hr({ symbols: ['BTCUSDT', 'ETHUSDT'] })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get ticker for all symbols
client.ticker24hr()
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Symbol Price Ticker

Get latest price for a symbol:

```javascript
// Get price for single symbol
client.tickerPrice('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get price for multiple symbols
client.tickerPrice({ symbols: ['BTCUSDT', 'ETHUSDT'] })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get price for all symbols
client.tickerPrice()
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Symbol Order Book Ticker

Get best price/quantity on the order book:

```javascript
// Get book ticker for single symbol
client.bookTicker('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get book ticker for multiple symbols
client.bookTicker({ symbols: ['BTCUSDT', 'ETHUSDT'] })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get book ticker for all symbols
client.bookTicker()
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

## Trading Endpoints

Trading endpoints require authentication and allow you to place, cancel, and query orders.

### New Order

Place a new order on the exchange:

```javascript
const { Spot } = require('@binance/connector')

const client = new Spot(apiKey, apiSecret)

// Market buy order
client.newOrder('BTCUSDT', 'BUY', 'MARKET', {
  quantity: 0.001
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Market sell order with quote quantity
client.newOrder('BTCUSDT', 'SELL', 'MARKET', {
  quoteOrderQty: 100
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Limit buy order
client.newOrder('BTCUSDT', 'BUY', 'LIMIT', {
  quantity: 0.001,
  price: 50000,
  timeInForce: 'GTC'
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Limit sell order
client.newOrder('BTCUSDT', 'SELL', 'LIMIT', {
  quantity: 0.001,
  price: 60000,
  timeInForce: 'GTC'
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Stop-loss order
client.newOrder('BTCUSDT', 'SELL', 'STOP_LOSS_LIMIT', {
  quantity: 0.001,
  price: 48000,
  stopPrice: 49000,
  timeInForce: 'GTC'
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Take-profit order
client.newOrder('BTCUSDT', 'SELL', 'TAKE_PROFIT_LIMIT', {
  quantity: 0.001,
  price: 62000,
  stopPrice: 61000,
  timeInForce: 'GTC'
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

**Order Types:**
- `MARKET` - Market order
- `LIMIT` - Limit order
- `STOP_LOSS` - Stop-loss order
- `STOP_LOSS_LIMIT` - Stop-loss limit order
- `TAKE_PROFIT` - Take-profit order
- `TAKE_PROFIT_LIMIT` - Take-profit limit order
- `LIMIT_MAKER` - Limit maker order

**Time in Force:**
- `GTC` - Good Till Cancel
- `IOC` - Immediate or Cancel
- `FOK` - Fill or Kill

### Test New Order

Test order placement without actually placing the order:

```javascript
// Test a limit order
client.newOrderTest('BTCUSDT', 'BUY', 'LIMIT', {
  quantity: 0.001,
  price: 50000,
  timeInForce: 'GTC'
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Query Order

Check an order's status:

```javascript
// Query by orderId
client.getOrder('BTCUSDT', { orderId: 28 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Query by origClientOrderId
client.getOrder('BTCUSDT', { origClientOrderId: 'myOrder1' })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Cancel Order

Cancel an active order:

```javascript
// Cancel by orderId
client.cancelOrder('BTCUSDT', { orderId: 28 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Cancel by origClientOrderId
client.cancelOrder('BTCUSDT', { origClientOrderId: 'myOrder1' })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Cancel All Open Orders

Cancel all active orders on a symbol:

```javascript
client.cancelOpenOrders('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Cancel and Replace Order

Cancel an existing order and place a new order on the same symbol:

```javascript
// Cancel and replace by orderId
client.cancelReplaceOrder('BTCUSDT', 'BUY', 'LIMIT', 'STOP_ON_FAILURE', {
  cancelOrderId: 28,
  quantity: 0.002,
  price: 51000,
  timeInForce: 'GTC'
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Current Open Orders

Get all open orders on a symbol:

```javascript
// Get open orders for specific symbol
client.openOrders('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get all open orders
client.openOrders()
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### All Orders

Get all account orders (active, canceled, or filled):

```javascript
// Get all orders for a symbol
client.allOrders('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get orders with time range
client.allOrders('BTCUSDT', {
  startTime: 1609459200000,
  endTime: 1609545600000,
  limit: 500
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get orders from specific orderId
client.allOrders('BTCUSDT', { orderId: 28, limit: 500 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### OCO Orders (One-Cancels-the-Other)

Place a pair of orders where if one is executed, the other is canceled:

```javascript
// Place OCO order
client.newOCOOrder('BTCUSDT', 'SELL', 0.001, 62000, 48000, {
  stopLimitPrice: 47500,
  stopLimitTimeInForce: 'GTC'
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Cancel OCO order
client.cancelOCOOrder('BTCUSDT', { orderListId: 0 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Query OCO order
client.getOCOOrder({ orderListId: 0 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get all OCO orders
client.getOCOOrders()
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get open OCO orders
client.getOpenOCOOrders()
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

## Account Endpoints

Account endpoints provide access to account information and require authentication.

### Account Information

Get current account information:

```javascript
const { Spot } = require('@binance/connector')

const client = new Spot(apiKey, apiSecret)

client.account()
  .then(response => {
    const data = response.data
    console.log('Account Type:', data.accountType)
    console.log('Can Trade:', data.canTrade)
    console.log('Can Withdraw:', data.canWithdraw)
    console.log('Can Deposit:', data.canDeposit)
    console.log('Balances:', data.balances)
  })
  .catch(error => console.error(error))
```

### Account Trade List

Get trades for a specific symbol:

```javascript
// Get recent trades
client.myTrades('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get trades with time range
client.myTrades('BTCUSDT', {
  startTime: 1609459200000,
  endTime: 1609545600000,
  limit: 500
})
  .then(response => console.log(response.data))
  .catch(error => console.error(error))

// Get trades from specific ID
client.myTrades('BTCUSDT', { fromId: 28457, limit: 500 })
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Current Order Count Usage

Get current order count usage for rate limits:

```javascript
client.rateLimitOrder()
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

### Query Prevented Matches

Get prevented matches from Self-Trade Prevention:

```javascript
client.preventedMatches('BTCUSDT')
  .then(response => console.log(response.data))
  .catch(error => console.error(error))
```

## WebSocket Streams

WebSocket streams provide real-time market data with low latency.

### WebSocket Stream Client

Initialize a WebSocket stream client:

```javascript
const { WebsocketStream } = require('@binance/connector')

// Create WebSocket stream client
const callbacks = {
  open: () => console.log('WebSocket connected'),
  close: () => console.log('WebSocket disconnected'),
  message: data => console.log('Received:', data)
}

const websocketStreamClient = new WebsocketStream({ callbacks })
```

### Aggregate Trade Streams

Subscribe to aggregate trade updates:

```javascript
const { WebsocketStream } = require('@binance/connector')

const callbacks = {
  open: () => console.log('Connected to aggregate trade stream'),
  close: () => console.log('Disconnected from aggregate trade stream'),
  message: data => console.log('Trade:', JSON.parse(data))
}

const websocketStreamClient = new WebsocketStream({ callbacks })

// Subscribe to BTCUSDT aggregate trades
websocketStreamClient.aggTrade('btcusdt')

// Unsubscribe
// websocketStreamClient.unsubscribe('btcusdt@aggTrade')

// Disconnect
// websocketStreamClient.disconnect()
```

### Trade Streams

Subscribe to raw trade updates:

```javascript
const callbacks = {
  open: () => console.log('Connected to trade stream'),
  close: () => console.log('Disconnected from trade stream'),
  message: data => console.log('Trade:', JSON.parse(data))
}

const websocketStreamClient = new WebsocketStream({ callbacks })

// Subscribe to BTCUSDT trades
websocketStreamClient.trade('btcusdt')
```

### Kline/Candlestick Streams

Subscribe to candlestick updates:

```javascript
const callbacks = {
  open: () => console.log('Connected to kline stream'),
  close: () => console.log('Disconnected from kline stream'),
  message: data => console.log('Kline:', JSON.parse(data))
}

const websocketStreamClient = new WebsocketStream({ callbacks })

// Subscribe to BTCUSDT 1-minute klines
websocketStreamClient.kline('btcusdt', '1m')

// Available intervals: 1s, 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M
```

### Mini Ticker Streams

Subscribe to 24hr mini ticker updates:

```javascript
const callbacks = {
  open: () => console.log('Connected to mini ticker stream'),
  close: () => console.log('Disconnected from mini ticker stream'),
  message: data => console.log('Mini Ticker:', JSON.parse(data))
}

const websocketStreamClient = new WebsocketStream({ callbacks })

// Subscribe to BTCUSDT mini ticker
websocketStreamClient.miniTicker('btcusdt')

// Subscribe to all symbols mini ticker
websocketStreamClient.miniTicker()
```

### Ticker Streams

Subscribe to 24hr ticker updates:

```javascript
const callbacks = {
  open: () => console.log('Connected to ticker stream'),
  close: () => console.log('Disconnected from ticker stream'),
  message: data => console.log('Ticker:', JSON.parse(data))
}

const websocketStreamClient = new WebsocketStream({ callbacks })

// Subscribe to BTCUSDT ticker
websocketStreamClient.ticker('btcusdt')

// Subscribe to all symbols ticker
websocketStreamClient.ticker()
```

### Individual Symbol Book Ticker Streams

Subscribe to best bid/ask updates:

```javascript
const callbacks = {
  open: () => console.log('Connected to book ticker stream'),
  close: () => console.log('Disconnected from book ticker stream'),
  message: data => console.log('Book Ticker:', JSON.parse(data))
}

const websocketStreamClient = new WebsocketStream({ callbacks })

// Subscribe to BTCUSDT book ticker
websocketStreamClient.bookTicker('btcusdt')
```

### Partial Book Depth Streams

Subscribe to top bids and asks:

```javascript
const callbacks = {
  open: () => console.log('Connected to partial depth stream'),
  close: () => console.log('Disconnected from partial depth stream'),
  message: data => console.log('Partial Depth:', JSON.parse(data))
}

const websocketStreamClient = new WebsocketStream({ callbacks })

// Subscribe to BTCUSDT top 5 levels @ 1000ms
websocketStreamClient.partialDepth('btcusdt', 5, 1000)

// Available levels: 5, 10, 20
// Available speeds: 1000ms, 100ms
```

### Diff Depth Stream

Subscribe to order book price and quantity depth updates:

```javascript
const callbacks = {
  open: () => console.log('Connected to diff depth stream'),
  close: () => console.log('Disconnected from diff depth stream'),
  message: data => console.log('Diff Depth:', JSON.parse(data))
}

const websocketStreamClient = new WebsocketStream({ callbacks })

// Subscribe to BTCUSDT depth updates @ 1000ms
websocketStreamClient.diffDepth('btcusdt', 1000)

// Available speeds: 1000ms, 100ms
```

### Combined Streams

Subscribe to multiple streams with a single connection:

```javascript
const callbacks = {
  open: () => console.log('Connected to combined stream'),
  close: () => console.log('Disconnected from combined stream'),
  message: data => console.log('Combined:', JSON.parse(data))
}

const websocketStreamClient = new WebsocketStream({ callbacks })

// Subscribe to multiple streams
websocketStreamClient.subscribe([
  'btcusdt@aggTrade',
  'btcusdt@depth',
  'ethusdt@kline_1m',
  'bnbusdt@ticker'
])
```

## WebSocket API

WebSocket API allows bidirectional communication for placing orders, querying data, and more.

### WebSocket API Client

Initialize a WebSocket API client:

```javascript
const { WebsocketAPI } = require('@binance/connector')

const callbacks = {
  open: (client) => console.log('WebSocket API connected'),
  close: () => console.log('WebSocket API disconnected'),
  message: data => console.log('Message:', data)
}

const websocketAPIClient = new WebsocketAPI(apiKey, apiSecret, { callbacks })
```

### Ping/Pong

Test connectivity:

```javascript
const { WebsocketAPI } = require('@binance/connector')

const callbacks = {
  open: (client) => {
    console.log('WebSocket API connected')

    // Send ping
    client.ping()
  },
  message: data => {
    console.log('Response:', data)
  }
}

const websocketAPIClient = new WebsocketAPI(apiKey, apiSecret, { callbacks })
```

### Server Time

Get server time:

```javascript
const callbacks = {
  open: (client) => {
    console.log('WebSocket API connected')

    // Get server time
    client.time()
  },
  message: data => {
    console.log('Server Time:', data)
  }
}

const websocketAPIClient = new WebsocketAPI(apiKey, apiSecret, { callbacks })
```

### Order Book via WebSocket API

Get order book through WebSocket:

```javascript
const callbacks = {
  open: (client) => {
    console.log('WebSocket API connected')

    // Get order book
    client.depth('BTCUSDT', { limit: 10 })
  },
  message: data => {
    console.log('Order Book:', data)
  }
}

const websocketAPIClient = new WebsocketAPI(apiKey, apiSecret, { callbacks })
```

### Place Order via WebSocket API

Place orders through WebSocket:

```javascript
const callbacks = {
  open: (client) => {
    console.log('WebSocket API connected')

    // Place market buy order
    client.newOrder('BTCUSDT', 'BUY', 'MARKET', { quantity: 0.001 })
  },
  message: data => {
    console.log('Order Response:', data)
  }
}

const websocketAPIClient = new WebsocketAPI(apiKey, apiSecret, { callbacks })
```

### Account Information via WebSocket API

Get account information through WebSocket:

```javascript
const callbacks = {
  open: (client) => {
    console.log('WebSocket API connected')

    // Get account information
    client.account()
  },
  message: data => {
    console.log('Account Info:', data)
  }
}

const websocketAPIClient = new WebsocketAPI(apiKey, apiSecret, { callbacks })
```

## User Data Streams

User Data Streams provide real-time updates about account and order changes.

### Start User Data Stream

Create a listenKey for user data stream:

```javascript
const { Spot } = require('@binance/connector')

const client = new Spot(apiKey, apiSecret)

// Create listen key
client.userDataStream()
  .then(response => {
    const listenKey = response.data.listenKey
    console.log('Listen Key:', listenKey)

    // Use this listenKey with WebSocket
    subscribeToUserDataStream(listenKey)
  })
  .catch(error => console.error(error))
```

### Keep-Alive User Data Stream

Extend the validity of a listenKey:

```javascript
// Keep alive (extend validity by 60 minutes)
client.keepAliveUserDataStream(listenKey)
  .then(response => console.log('Stream kept alive'))
  .catch(error => console.error(error))
```

### Close User Data Stream

Close a user data stream:

```javascript
// Close stream
client.closeUserDataStream(listenKey)
  .then(response => console.log('Stream closed'))
  .catch(error => console.error(error))
```

### Subscribe to User Data Stream

Connect to user data stream via WebSocket:

```javascript
const { WebsocketStream } = require('@binance/connector')

function subscribeToUserDataStream(listenKey) {
  const callbacks = {
    open: () => console.log('Connected to user data stream'),
    close: () => console.log('Disconnected from user data stream'),
    message: data => {
      const event = JSON.parse(data)

      if (event.e === 'executionReport') {
        console.log('Order Update:', event)
      } else if (event.e === 'outboundAccountPosition') {
        console.log('Balance Update:', event)
      } else if (event.e === 'balanceUpdate') {
        console.log('Balance Update:', event)
      }
    }
  }

  const websocketStreamClient = new WebsocketStream({ callbacks })
  websocketStreamClient.userData(listenKey)

  return websocketStreamClient
}
```

### Complete User Data Stream Example

Full example with keep-alive:

```javascript
const { Spot, WebsocketStream } = require('@binance/connector')

const client = new Spot(apiKey, apiSecret)

// Create listen key
client.userDataStream()
  .then(response => {
    const listenKey = response.data.listenKey
    console.log('Listen Key:', listenKey)

    // Connect to user data stream
    const callbacks = {
      open: () => console.log('Connected to user data stream'),
      close: () => console.log('Disconnected from user data stream'),
      message: data => {
        const event = JSON.parse(data)
        console.log('User Data Event:', event)
      }
    }

    const websocketStreamClient = new WebsocketStream({ callbacks })
    websocketStreamClient.userData(listenKey)

    // Keep alive every 30 minutes
    setInterval(() => {
      client.keepAliveUserDataStream(listenKey)
        .then(() => console.log('Stream kept alive'))
        .catch(error => console.error('Keep alive failed:', error))
    }, 30 * 60 * 1000)
  })
  .catch(error => console.error(error))
```

## Error Handling

Proper error handling for API requests:

```javascript
const { Spot } = require('@binance/connector')

const client = new Spot(apiKey, apiSecret)

client.account()
  .then(response => {
    console.log('Success:', response.data)
  })
  .catch(error => {
    if (error.response) {
      // Server responded with error
      console.error('Error Status:', error.response.status)
      console.error('Error Code:', error.response.data.code)
      console.error('Error Message:', error.response.data.msg)
    } else if (error.request) {
      // No response received
      console.error('No response received:', error.request)
    } else {
      // Request setup error
      console.error('Error:', error.message)
    }
  })
```

### Common Error Codes

| Code  | Message                          | Description                              |
|-------|----------------------------------|------------------------------------------|
| -1000 | UNKNOWN                          | An unknown error occurred                |
| -1001 | DISCONNECTED                     | Internal error; unable to process        |
| -1002 | UNAUTHORIZED                     | Invalid API key or signature             |
| -1003 | TOO_MANY_REQUESTS                | Too many requests; rate limit exceeded   |
| -1007 | TIMEOUT                          | Request timeout                          |
| -1021 | TIMESTAMP_OUTSIDE_RECV_WINDOW    | Timestamp outside recvWindow             |
| -1022 | INVALID_SIGNATURE                | Invalid signature                        |
| -1100 | ILLEGAL_CHARS                    | Illegal characters in parameter          |
| -1101 | TOO_MANY_PARAMETERS              | Too many parameters                      |
| -1102 | MANDATORY_PARAM_EMPTY_OR_MALFORMED | Mandatory parameter missing/malformed  |
| -1103 | UNKNOWN_PARAM                    | Unknown parameter                        |
| -1104 | UNREAD_PARAMETERS                | Not all parameters were read             |
| -1111 | BAD_PRECISION                    | Precision over maximum defined           |
| -1112 | NO_DEPTH                         | No orders on book                        |
| -1114 | TIF_NOT_REQUIRED                 | Time in force not required               |
| -1115 | INVALID_TIF                      | Invalid time in force                    |
| -1116 | INVALID_ORDER_TYPE               | Invalid order type                       |
| -1117 | INVALID_SIDE                     | Invalid side                             |
| -1118 | EMPTY_NEW_CL_ORD_ID              | New client order ID empty                |
| -1119 | EMPTY_ORG_CL_ORD_ID              | Original client order ID empty           |
| -1120 | BAD_INTERVAL                     | Invalid interval                         |
| -1121 | BAD_SYMBOL                       | Invalid symbol                           |
| -1125 | INVALID_LISTEN_KEY               | Invalid listen key                       |
| -1127 | MORE_THAN_XX_HOURS               | Lookup interval too large                |
| -1128 | OPTIONAL_PARAMS_BAD_COMBO        | Invalid parameter combination            |
| -1130 | INVALID_PARAMETER                | Invalid parameter                        |
| -2010 | NEW_ORDER_REJECTED               | Order rejected                           |
| -2011 | CANCEL_REJECTED                  | Cancel rejected                          |
| -2013 | NO_SUCH_ORDER                    | Order does not exist                     |
| -2014 | BAD_API_KEY_FMT                  | API key format invalid                   |
| -2015 | REJECTED_MBX_KEY                 | API key rejected                         |

## Rate Limits

Binance implements rate limits to prevent API abuse.

### Rate Limit Types

1. **REQUEST_WEIGHT** - Based on the weight of requests
2. **ORDERS** - Based on the number of orders
3. **RAW_REQUESTS** - Based on the number of requests

### Rate Limit Headers

Response headers include rate limit information:

```javascript
client.account()
  .then(response => {
    console.log('Used Weight:', response.headers['x-mbx-used-weight-1m'])
    console.log('Order Count:', response.headers['x-mbx-order-count-10s'])
  })
  .catch(error => console.error(error))
```

### Best Practices for Rate Limits

- Use WebSocket streams for real-time data instead of polling REST API
- Batch requests when possible
- Monitor rate limit headers
- Implement exponential backoff for retries
- Use `recvWindow` parameter appropriately

## Advanced Configuration

### Timestamp Synchronization

Configure timing window for requests:

```javascript
const { Spot } = require('@binance/connector')

const client = new Spot(apiKey, apiSecret, {
  recvWindow: 10000 // 10 seconds (default: 5000)
})
```

### Custom Logger

Use a custom logger:

```javascript
const winston = require('winston')

const logger = winston.createLogger({
  level: 'info',
  format: winston.format.json(),
  transports: [
    new winston.transports.File({ filename: 'binance.log' })
  ]
})

const client = new Spot(apiKey, apiSecret, {
  logger: logger
})
```

### Base URL Selection

Choose optimal base URL for your region:

```javascript
// Default (most stable)
const client1 = new Spot(apiKey, apiSecret, {
  baseURL: 'https://api.binance.com'
})

// GCP (better for some regions)
const client2 = new Spot(apiKey, apiSecret, {
  baseURL: 'https://api-gcp.binance.com'
})

// Numbered endpoints (better performance, less stability)
const client3 = new Spot(apiKey, apiSecret, {
  baseURL: 'https://api1.binance.com'
})

// Market data only
const client4 = new Spot('', '', {
  baseURL: 'https://data-api.binance.vision'
})
```

### Async/Await Pattern

Use async/await for cleaner code:

```javascript
const { Spot } = require('@binance/connector')

const client = new Spot(apiKey, apiSecret)

async function getAccountBalance() {
  try {
    const response = await client.account()
    const balances = response.data.balances.filter(b => parseFloat(b.free) > 0)
    console.log('Non-zero balances:', balances)
    return balances
  } catch (error) {
    console.error('Error fetching account:', error.response?.data || error.message)
    throw error
  }
}

async function placeLimitOrder(symbol, side, quantity, price) {
  try {
    const response = await client.newOrder(symbol, side, 'LIMIT', {
      quantity,
      price,
      timeInForce: 'GTC'
    })
    console.log('Order placed:', response.data)
    return response.data
  } catch (error) {
    console.error('Error placing order:', error.response?.data || error.message)
    throw error
  }
}

// Usage
(async () => {
  await getAccountBalance()
  await placeLimitOrder('BTCUSDT', 'BUY', 0.001, 50000)
})()
```

## Useful Links

- Official Documentation: https://developers.binance.com/docs/binance-spot-api-docs
- REST API Documentation: https://developers.binance.com/docs/binance-spot-api-docs/rest-api
- WebSocket Streams: https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams
- WebSocket API: https://developers.binance.com/docs/binance-spot-api-docs/websocket-api
- GitHub Repository: https://github.com/binance/binance-connector-node
- API Testnet: https://testnet.binance.vision
- Get API Key: https://www.binance.com/en/my/settings/api-management
- API Documentation (Old): https://binance-docs.github.io/apidocs/spot/en/
