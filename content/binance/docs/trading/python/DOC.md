---
name: trading
description: "Binance API Python coding guidelines for trading using official libraries and SDKs"
metadata:
  languages: "python"
  versions: "3.12.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "binance,trading,crypto,exchange,api"
---

# Binance API Python Coding Guidelines

You are a Binance API coding expert. Help me with writing code using the Binance API calling the official libraries and SDKs.

You can find the official documentation here:
https://developers.binance.com/docs/binance-spot-api-docs

## Golden Rule: Use the Correct and Current SDK

Always use the official Binance Connector for Python, which is the standard library for all Binance Spot API interactions. Do not use unofficial or third-party libraries.

- **Library Name:** Binance Connector Python
- **PyPI Package:** `binance-connector`
- **Alternative Official Packages:** `binance-sdk-spot` (newer modular package), `binance-futures-connector`, `binance-pay-connector`
- **Unofficial Libraries:** `python-binance`, `binance.py` (not recommended for production)

**Installation:**

- **Correct:** `pip install binance-connector`
- **Alternative:** `pip install binance-sdk-spot` (for newer modular approach)

**APIs and Usage:**

- **Correct:** `from binance.spot import Spot`
- **Correct:** `client = Spot(api_key='...', api_secret='...')`
- **Correct:** `client.account()` for account information
- **Correct:** `client.new_order()` for placing orders
- **Incorrect:** Using unofficial packages like `python-binance`
- **Incorrect:** Direct HTTP requests without using the connector

## Installation

Install the official Binance connector for Python:

```bash
pip install binance-connector
```

For Python 3.9 or later, you can also use the modular SDK:

```bash
pip install binance-sdk-spot
```

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

Load environment variables in Python:

```python
import os
from dotenv import load_dotenv

load_dotenv()

api_key = os.getenv('BINANCE_API_KEY')
api_secret = os.getenv('BINANCE_API_SECRET')
```

## Initialization

The `binance-connector` library requires creating a `Spot` instance for all API calls.

### Basic Initialization (Public Endpoints)

For public market data endpoints that don't require authentication:

```python
from binance.spot import Spot

# Public endpoints only (no authentication)
client = Spot()

# Use client for public data
exchange_info = client.exchange_info()
print(exchange_info)
```

### Authenticated Initialization (HMAC)

For trading and account endpoints:

```python
import os
from binance.spot import Spot

api_key = os.getenv('BINANCE_API_KEY')
api_secret = os.getenv('BINANCE_API_SECRET')

# Authenticated client with HMAC
client = Spot(api_key=api_key, api_secret=api_secret)

# Use client for authenticated endpoints
account_info = client.account()
print(account_info)
```

### RSA/ED25519 Authentication

For enhanced security using RSA or ED25519 private keys:

```python
from binance.spot import Spot

api_key = os.getenv('BINANCE_API_KEY')

# Read private key from file
with open('/path/to/private_key.pem', 'r') as f:
    private_key = f.read()

# Initialize with RSA key
client = Spot(
    api_key=api_key,
    private_key=private_key,
    private_key_pass='your_passphrase'  # Optional
)

# For ED25519 keys, the connector auto-detects the key type
```

### Client Configuration Options

```python
from binance.spot import Spot

client = Spot(
    api_key=api_key,
    api_secret=api_secret,
    base_url='https://api.binance.com',  # Default base URL
    timeout=10,  # Request timeout in seconds
    proxies=None,  # Proxy configuration
    show_limit_usage=False,  # Show rate limit usage in response
    show_header=False  # Include response headers
)
```

### Testnet Configuration

For testing without real funds:

```python
from binance.spot import Spot

client = Spot(
    api_key=api_key,
    api_secret=api_secret,
    base_url='https://testnet.binance.vision'
)
```

### Proxy Configuration

For requests through a proxy:

```python
from binance.spot import Spot

proxies = {
    'http': 'http://username:password@proxy.example.com:8080',
    'https': 'https://username:password@proxy.example.com:8080'
}

client = Spot(
    api_key=api_key,
    api_secret=api_secret,
    proxies=proxies
)
```

### Logging Configuration

Enable logging for debugging:

```python
import logging
from binance.spot import Spot
from binance.lib.utils import config_logging

# Configure logging
config_logging(logging, logging.DEBUG)

client = Spot(api_key=api_key, api_secret=api_secret)

# All API calls will now be logged
account_info = client.account()
```

## Market Data Endpoints

Market data endpoints provide access to public trading information without authentication.

### Exchange Information

Get exchange trading rules and symbol information:

```python
from binance.spot import Spot

client = Spot()

# Get all exchange information
exchange_info = client.exchange_info()
print(exchange_info)

# Get specific symbol information
exchange_info = client.exchange_info(symbol='BTCUSDT')
print(exchange_info)

# Get information for multiple symbols
exchange_info = client.exchange_info(symbols=['BTCUSDT', 'ETHUSDT'])
print(exchange_info)
```

### Order Book (Depth)

Get current order book for a symbol:

```python
# Get order book with default limit (100)
depth = client.depth('BTCUSDT')
print(depth)

# Get order book with specific limit (5, 10, 20, 50, 100, 500, 1000, 5000)
depth = client.depth('BTCUSDT', limit=10)
print(depth)

# Access bids and asks
bids = depth['bids']
asks = depth['asks']
print(f"Best Bid: {bids[0]}")
print(f"Best Ask: {asks[0]}")
```

### Recent Trades

Get recent trades for a symbol:

```python
# Get recent trades (default limit: 500)
trades = client.trades('BTCUSDT')
print(trades)

# Get specific number of recent trades
trades = client.trades('BTCUSDT', limit=100)
print(trades)

# Process trades
for trade in trades:
    print(f"Price: {trade['price']}, Qty: {trade['qty']}, Time: {trade['time']}")
```

### Historical Trades

Get older trades (requires API key):

```python
client = Spot(api_key=api_key, api_secret=api_secret)

# Get historical trades
historical_trades = client.historical_trades('BTCUSDT', limit=100)
print(historical_trades)

# Get trades from specific ID
historical_trades = client.historical_trades('BTCUSDT', limit=100, fromId=28457)
print(historical_trades)
```

### Aggregate Trades

Get compressed, aggregate trades:

```python
# Get recent aggregate trades
agg_trades = client.agg_trades('BTCUSDT')
print(agg_trades)

# Get aggregate trades with time range
agg_trades = client.agg_trades(
    'BTCUSDT',
    startTime=1609459200000,
    endTime=1609545600000,
    limit=1000
)
print(agg_trades)

# Get aggregate trades from specific ID
agg_trades = client.agg_trades('BTCUSDT', fromId=28457, limit=500)
print(agg_trades)
```

### Candlestick Data (Klines)

Get kline/candlestick bars for a symbol:

```python
# Get recent klines (default limit: 500)
klines = client.klines('BTCUSDT', '1h')
print(klines)

# Get klines with time range
klines = client.klines(
    'BTCUSDT',
    '1d',
    startTime=1609459200000,
    endTime=1609545600000,
    limit=100
)
print(klines)

# Available intervals: 1s, 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M

# Process klines
for kline in klines:
    open_time, open_price, high, low, close, volume, close_time, quote_volume, trades, taker_buy_base, taker_buy_quote, ignore = kline
    print(f"Time: {open_time}, Open: {open_price}, High: {high}, Low: {low}, Close: {close}, Volume: {volume}")
```

### Klines to Pandas DataFrame

Convert klines to pandas DataFrame for analysis:

```python
import pandas as pd
from binance.spot import Spot

client = Spot()

klines = client.klines('BTCUSDT', '1h', limit=100)

# Convert to DataFrame
df = pd.DataFrame(klines, columns=[
    'open_time', 'open', 'high', 'low', 'close', 'volume',
    'close_time', 'quote_volume', 'trades', 'taker_buy_base',
    'taker_buy_quote', 'ignore'
])

# Convert timestamps to datetime
df['open_time'] = pd.to_datetime(df['open_time'], unit='ms')
df['close_time'] = pd.to_datetime(df['close_time'], unit='ms')

# Convert price columns to float
price_cols = ['open', 'high', 'low', 'close', 'volume']
df[price_cols] = df[price_cols].astype(float)

print(df.head())
```

### Current Average Price

Get current average price for a symbol:

```python
avg_price = client.avg_price('BTCUSDT')
print(avg_price)
```

### 24hr Ticker Price Change Statistics

Get 24-hour rolling window price change statistics:

```python
# Get ticker for single symbol
ticker = client.ticker_24hr('BTCUSDT')
print(ticker)

# Get ticker for multiple symbols
ticker = client.ticker_24hr(symbols=['BTCUSDT', 'ETHUSDT'])
print(ticker)

# Get ticker for all symbols (warning: large response)
ticker = client.ticker_24hr()
print(ticker)
```

### Symbol Price Ticker

Get latest price for a symbol:

```python
# Get price for single symbol
price = client.ticker_price('BTCUSDT')
print(price)

# Get price for multiple symbols
prices = client.ticker_price(symbols=['BTCUSDT', 'ETHUSDT'])
print(prices)

# Get price for all symbols
prices = client.ticker_price()
print(prices)
```

### Symbol Order Book Ticker

Get best price/quantity on the order book:

```python
# Get book ticker for single symbol
book_ticker = client.book_ticker('BTCUSDT')
print(book_ticker)

# Get book ticker for multiple symbols
book_ticker = client.book_ticker(symbols=['BTCUSDT', 'ETHUSDT'])
print(book_ticker)

# Get book ticker for all symbols
book_ticker = client.book_ticker()
print(book_ticker)
```

## Trading Endpoints

Trading endpoints require authentication and allow you to place, cancel, and query orders.

### New Order

Place a new order on the exchange:

```python
from binance.spot import Spot

client = Spot(api_key=api_key, api_secret=api_secret)

# Market buy order
order = client.new_order(
    symbol='BTCUSDT',
    side='BUY',
    type='MARKET',
    quantity=0.001
)
print(order)

# Market sell order with quote quantity
order = client.new_order(
    symbol='BTCUSDT',
    side='SELL',
    type='MARKET',
    quoteOrderQty=100
)
print(order)

# Limit buy order
order = client.new_order(
    symbol='BTCUSDT',
    side='BUY',
    type='LIMIT',
    timeInForce='GTC',
    quantity=0.001,
    price=50000
)
print(order)

# Limit sell order
order = client.new_order(
    symbol='BTCUSDT',
    side='SELL',
    type='LIMIT',
    timeInForce='GTC',
    quantity=0.001,
    price=60000
)
print(order)

# Stop-loss order
order = client.new_order(
    symbol='BTCUSDT',
    side='SELL',
    type='STOP_LOSS_LIMIT',
    timeInForce='GTC',
    quantity=0.001,
    price=48000,
    stopPrice=49000
)
print(order)

# Take-profit order
order = client.new_order(
    symbol='BTCUSDT',
    side='SELL',
    type='TAKE_PROFIT_LIMIT',
    timeInForce='GTC',
    quantity=0.001,
    price=62000,
    stopPrice=61000
)
print(order)
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

```python
# Test a limit order
test_order = client.new_order_test(
    symbol='BTCUSDT',
    side='BUY',
    type='LIMIT',
    timeInForce='GTC',
    quantity=0.001,
    price=50000
)
print(test_order)
```

### Query Order

Check an order's status:

```python
# Query by orderId
order = client.get_order('BTCUSDT', orderId=28)
print(order)

# Query by origClientOrderId
order = client.get_order('BTCUSDT', origClientOrderId='myOrder1')
print(order)
```

### Cancel Order

Cancel an active order:

```python
# Cancel by orderId
cancel_result = client.cancel_order('BTCUSDT', orderId=28)
print(cancel_result)

# Cancel by origClientOrderId
cancel_result = client.cancel_order('BTCUSDT', origClientOrderId='myOrder1')
print(cancel_result)
```

### Cancel All Open Orders

Cancel all active orders on a symbol:

```python
cancel_results = client.cancel_open_orders('BTCUSDT')
print(cancel_results)
```

### Cancel and Replace Order

Cancel an existing order and place a new order on the same symbol:

```python
# Cancel and replace by orderId
result = client.cancel_replace(
    symbol='BTCUSDT',
    side='BUY',
    type='LIMIT',
    cancelReplaceMode='STOP_ON_FAILURE',
    timeInForce='GTC',
    cancelOrderId=28,
    quantity=0.002,
    price=51000
)
print(result)
```

### Current Open Orders

Get all open orders on a symbol:

```python
# Get open orders for specific symbol
open_orders = client.get_open_orders('BTCUSDT')
print(open_orders)

# Get all open orders
open_orders = client.get_open_orders()
print(open_orders)
```

### All Orders

Get all account orders (active, canceled, or filled):

```python
# Get all orders for a symbol
all_orders = client.get_orders('BTCUSDT')
print(all_orders)

# Get orders with time range
all_orders = client.get_orders(
    'BTCUSDT',
    startTime=1609459200000,
    endTime=1609545600000,
    limit=500
)
print(all_orders)

# Get orders from specific orderId
all_orders = client.get_orders('BTCUSDT', orderId=28, limit=500)
print(all_orders)
```

### OCO Orders (One-Cancels-the-Other)

Place a pair of orders where if one is executed, the other is canceled:

```python
# Place OCO order
oco_order = client.new_oco_order(
    symbol='BTCUSDT',
    side='SELL',
    quantity=0.001,
    price=62000,
    stopPrice=48000,
    stopLimitPrice=47500,
    stopLimitTimeInForce='GTC'
)
print(oco_order)

# Cancel OCO order
cancel_result = client.cancel_oco_order('BTCUSDT', orderListId=0)
print(cancel_result)

# Query OCO order
oco_order = client.get_oco_order(orderListId=0)
print(oco_order)

# Get all OCO orders
oco_orders = client.get_oco_orders()
print(oco_orders)

# Get open OCO orders
open_oco_orders = client.get_open_oco_orders()
print(open_oco_orders)
```

## Account Endpoints

Account endpoints provide access to account information and require authentication.

### Account Information

Get current account information:

```python
from binance.spot import Spot

client = Spot(api_key=api_key, api_secret=api_secret)

account = client.account()
print(f"Account Type: {account['accountType']}")
print(f"Can Trade: {account['canTrade']}")
print(f"Can Withdraw: {account['canWithdraw']}")
print(f"Can Deposit: {account['canDeposit']}")

# Get non-zero balances
balances = [b for b in account['balances'] if float(b['free']) > 0 or float(b['locked']) > 0]
print(f"Non-zero Balances: {balances}")
```

### Account Trade List

Get trades for a specific symbol:

```python
# Get recent trades
trades = client.my_trades('BTCUSDT')
print(trades)

# Get trades with time range
trades = client.my_trades(
    'BTCUSDT',
    startTime=1609459200000,
    endTime=1609545600000,
    limit=500
)
print(trades)

# Get trades from specific ID
trades = client.my_trades('BTCUSDT', fromId=28457, limit=500)
print(trades)

# Calculate total trading volume
total_volume = sum(float(trade['qty']) for trade in trades)
print(f"Total Volume: {total_volume}")
```

### Current Order Count Usage

Get current order count usage for rate limits:

```python
rate_limit = client.rate_limit_order()
print(rate_limit)
```

### Query Prevented Matches

Get prevented matches from Self-Trade Prevention:

```python
prevented_matches = client.prevented_matches('BTCUSDT')
print(prevented_matches)
```

## WebSocket Streams

WebSocket streams provide real-time market data with low latency.

### WebSocket Stream Client

Initialize a WebSocket stream client:

```python
from binance.websocket.spot.websocket_stream import SpotWebsocketStreamClient

def message_handler(_, message):
    print(f"Received message: {message}")

# Create WebSocket stream client
ws_client = SpotWebsocketStreamClient(on_message=message_handler)

# Start the client
ws_client.start()

# Subscribe to streams (examples below)
```

### Aggregate Trade Streams

Subscribe to aggregate trade updates:

```python
from binance.websocket.spot.websocket_stream import SpotWebsocketStreamClient

def message_handler(_, message):
    print(f"Aggregate Trade: {message}")

ws_client = SpotWebsocketStreamClient(on_message=message_handler)

# Subscribe to BTCUSDT aggregate trades
ws_client.agg_trade(symbol='btcusdt')

# Keep the connection alive
import time
time.sleep(60)

# Stop the client
ws_client.stop()
```

### Trade Streams

Subscribe to raw trade updates:

```python
def message_handler(_, message):
    print(f"Trade: {message}")

ws_client = SpotWebsocketStreamClient(on_message=message_handler)

# Subscribe to BTCUSDT trades
ws_client.trade(symbol='btcusdt')

import time
time.sleep(60)

ws_client.stop()
```

### Kline/Candlestick Streams

Subscribe to candlestick updates:

```python
def message_handler(_, message):
    print(f"Kline: {message}")

ws_client = SpotWebsocketStreamClient(on_message=message_handler)

# Subscribe to BTCUSDT 1-minute klines
ws_client.kline(symbol='btcusdt', interval='1m')

# Available intervals: 1s, 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M

import time
time.sleep(60)

ws_client.stop()
```

### Mini Ticker Streams

Subscribe to 24hr mini ticker updates:

```python
def message_handler(_, message):
    print(f"Mini Ticker: {message}")

ws_client = SpotWebsocketStreamClient(on_message=message_handler)

# Subscribe to BTCUSDT mini ticker
ws_client.mini_ticker(symbol='btcusdt')

# Subscribe to all symbols mini ticker
ws_client.mini_ticker()

import time
time.sleep(60)

ws_client.stop()
```

### Ticker Streams

Subscribe to 24hr ticker updates:

```python
def message_handler(_, message):
    print(f"Ticker: {message}")

ws_client = SpotWebsocketStreamClient(on_message=message_handler)

# Subscribe to BTCUSDT ticker
ws_client.ticker(symbol='btcusdt')

# Subscribe to all symbols ticker
ws_client.ticker()

import time
time.sleep(60)

ws_client.stop()
```

### Individual Symbol Book Ticker Streams

Subscribe to best bid/ask updates:

```python
def message_handler(_, message):
    print(f"Book Ticker: {message}")

ws_client = SpotWebsocketStreamClient(on_message=message_handler)

# Subscribe to BTCUSDT book ticker
ws_client.book_ticker(symbol='btcusdt')

import time
time.sleep(60)

ws_client.stop()
```

### Partial Book Depth Streams

Subscribe to top bids and asks:

```python
def message_handler(_, message):
    print(f"Partial Depth: {message}")

ws_client = SpotWebsocketStreamClient(on_message=message_handler)

# Subscribe to BTCUSDT top 5 levels @ 1000ms
ws_client.partial_depth(symbol='btcusdt', level=5, speed=1000)

# Available levels: 5, 10, 20
# Available speeds: 1000ms, 100ms

import time
time.sleep(60)

ws_client.stop()
```

### Diff Depth Stream

Subscribe to order book price and quantity depth updates:

```python
def message_handler(_, message):
    print(f"Diff Depth: {message}")

ws_client = SpotWebsocketStreamClient(on_message=message_handler)

# Subscribe to BTCUSDT depth updates @ 1000ms
ws_client.diff_depth(symbol='btcusdt', speed=1000)

# Available speeds: 1000ms, 100ms

import time
time.sleep(60)

ws_client.stop()
```

### Multiple Streams

Subscribe to multiple streams with a single connection:

```python
def message_handler(_, message):
    print(f"Message: {message}")

ws_client = SpotWebsocketStreamClient(on_message=message_handler)

# Subscribe to multiple streams
ws_client.agg_trade(symbol='btcusdt')
ws_client.trade(symbol='ethusdt')
ws_client.kline(symbol='bnbusdt', interval='1m')
ws_client.ticker(symbol='adausdt')

import time
time.sleep(60)

ws_client.stop()
```

## WebSocket API

WebSocket API allows bidirectional communication for placing orders, querying data, and more.

### WebSocket API Client

Initialize a WebSocket API client:

```python
from binance.websocket.spot.websocket_api import SpotWebsocketAPIClient

def message_handler(_, message):
    print(f"Message: {message}")

# Create WebSocket API client
ws_client = SpotWebsocketAPIClient(
    api_key=api_key,
    api_secret=api_secret,
    on_message=message_handler
)

# The client automatically connects
```

### Ping/Pong

Test connectivity:

```python
from binance.websocket.spot.websocket_api import SpotWebsocketAPIClient

def message_handler(_, message):
    print(f"Response: {message}")

ws_client = SpotWebsocketAPIClient(on_message=message_handler)

# Send ping
ws_client.ping()

import time
time.sleep(2)

ws_client.stop()
```

### Server Time

Get server time:

```python
def message_handler(_, message):
    print(f"Server Time: {message}")

ws_client = SpotWebsocketAPIClient(on_message=message_handler)

# Get server time
ws_client.time()

import time
time.sleep(2)

ws_client.stop()
```

### Order Book via WebSocket API

Get order book through WebSocket:

```python
def message_handler(_, message):
    print(f"Order Book: {message}")

ws_client = SpotWebsocketAPIClient(on_message=message_handler)

# Get order book
ws_client.depth(symbol='BTCUSDT', limit=10)

import time
time.sleep(2)

ws_client.stop()
```

### Place Order via WebSocket API

Place orders through WebSocket:

```python
def message_handler(_, message):
    print(f"Order Response: {message}")

ws_client = SpotWebsocketAPIClient(
    api_key=api_key,
    api_secret=api_secret,
    on_message=message_handler
)

# Place market buy order
ws_client.new_order(
    symbol='BTCUSDT',
    side='BUY',
    type='MARKET',
    quantity=0.001
)

import time
time.sleep(2)

ws_client.stop()
```

### Account Information via WebSocket API

Get account information through WebSocket:

```python
def message_handler(_, message):
    print(f"Account Info: {message}")

ws_client = SpotWebsocketAPIClient(
    api_key=api_key,
    api_secret=api_secret,
    on_message=message_handler
)

# Get account information
ws_client.account_status()

import time
time.sleep(2)

ws_client.stop()
```

## User Data Streams

User Data Streams provide real-time updates about account and order changes.

### Start User Data Stream

Create a listenKey for user data stream:

```python
from binance.spot import Spot

client = Spot(api_key=api_key, api_secret=api_secret)

# Create listen key
response = client.user_data_stream()
listen_key = response['listenKey']
print(f"Listen Key: {listen_key}")

# Use this listenKey with WebSocket
```

### Keep-Alive User Data Stream

Extend the validity of a listenKey:

```python
# Keep alive (extend validity by 60 minutes)
client.renew_user_data_stream(listenKey=listen_key)
print("Stream kept alive")
```

### Close User Data Stream

Close a user data stream:

```python
# Close stream
client.close_user_data_stream(listenKey=listen_key)
print("Stream closed")
```

### Subscribe to User Data Stream

Connect to user data stream via WebSocket:

```python
from binance.spot import Spot
from binance.websocket.spot.websocket_stream import SpotWebsocketStreamClient

client = Spot(api_key=api_key, api_secret=api_secret)

# Create listen key
response = client.user_data_stream()
listen_key = response['listenKey']

def message_handler(_, message):
    import json
    event = json.loads(message)

    if event['e'] == 'executionReport':
        print(f"Order Update: {event}")
    elif event['e'] == 'outboundAccountPosition':
        print(f"Balance Update: {event}")
    elif event['e'] == 'balanceUpdate':
        print(f"Balance Update: {event}")

# Connect to user data stream
ws_client = SpotWebsocketStreamClient(on_message=message_handler)
ws_client.user_data(listen_key=listen_key)

# Keep stream alive
import time
import threading

def keep_alive():
    while True:
        time.sleep(30 * 60)  # 30 minutes
        try:
            client.renew_user_data_stream(listenKey=listen_key)
            print("Stream kept alive")
        except Exception as e:
            print(f"Keep alive failed: {e}")

# Start keep-alive thread
keep_alive_thread = threading.Thread(target=keep_alive, daemon=True)
keep_alive_thread.start()

# Keep running
try:
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    ws_client.stop()
    client.close_user_data_stream(listenKey=listen_key)
```

## Error Handling

Proper error handling for API requests:

```python
from binance.spot import Spot
from binance.error import ClientError, ServerError

client = Spot(api_key=api_key, api_secret=api_secret)

try:
    account = client.account()
    print(f"Success: {account}")
except ClientError as error:
    # Handle client errors (4xx status codes)
    print(f"Client Error - Status: {error.status_code}, Code: {error.error_code}, Message: {error.error_message}")
except ServerError as error:
    # Handle server errors (5xx status codes)
    print(f"Server Error - Status: {error.status_code}, Message: {error.error_message}")
except Exception as error:
    # Handle other errors
    print(f"Error: {error}")
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

Enable rate limit headers in client configuration:

```python
from binance.spot import Spot

client = Spot(
    api_key=api_key,
    api_secret=api_secret,
    show_limit_usage=True,
    show_header=True
)

response = client.account()

# Access headers
print(f"Used Weight: {response.get('x-mbx-used-weight-1m')}")
print(f"Order Count: {response.get('x-mbx-order-count-10s')}")
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

```python
from binance.spot import Spot

client = Spot(
    api_key=api_key,
    api_secret=api_secret
)

# Manually set recvWindow for specific request
account = client.account(recvWindow=10000)  # 10 seconds
```

### Custom Request Parameters

Pass additional parameters to requests:

```python
# Pass custom client order ID
order = client.new_order(
    symbol='BTCUSDT',
    side='BUY',
    type='LIMIT',
    timeInForce='GTC',
    quantity=0.001,
    price=50000,
    newClientOrderId='my_custom_order_id_123'
)
```

### Base URL Selection

Choose optimal base URL for your region:

```python
# Default (most stable)
client1 = Spot(
    api_key=api_key,
    api_secret=api_secret,
    base_url='https://api.binance.com'
)

# GCP (better for some regions)
client2 = Spot(
    api_key=api_key,
    api_secret=api_secret,
    base_url='https://api-gcp.binance.com'
)

# Numbered endpoints (better performance, less stability)
client3 = Spot(
    api_key=api_key,
    api_secret=api_secret,
    base_url='https://api1.binance.com'
)

# Market data only
client4 = Spot(base_url='https://data-api.binance.vision')
```

### Complete Trading Bot Example

Full example of a simple trading bot:

```python
import os
import time
import logging
from binance.spot import Spot
from binance.error import ClientError, ServerError
from binance.lib.utils import config_logging

# Configure logging
config_logging(logging, logging.INFO)

# Initialize client
api_key = os.getenv('BINANCE_API_KEY')
api_secret = os.getenv('BINANCE_API_SECRET')

client = Spot(api_key=api_key, api_secret=api_secret)

def get_account_balance(asset='USDT'):
    """Get balance for specific asset"""
    try:
        account = client.account()
        for balance in account['balances']:
            if balance['asset'] == asset:
                return float(balance['free'])
        return 0.0
    except (ClientError, ServerError) as error:
        logging.error(f"Error getting balance: {error}")
        return 0.0

def get_current_price(symbol='BTCUSDT'):
    """Get current price for symbol"""
    try:
        ticker = client.ticker_price(symbol=symbol)
        return float(ticker['price'])
    except (ClientError, ServerError) as error:
        logging.error(f"Error getting price: {error}")
        return 0.0

def place_market_order(symbol, side, quantity):
    """Place market order"""
    try:
        order = client.new_order(
            symbol=symbol,
            side=side,
            type='MARKET',
            quantity=quantity
        )
        logging.info(f"Order placed: {order}")
        return order
    except ClientError as error:
        logging.error(f"Order failed: {error.error_message}")
        return None

def place_limit_order(symbol, side, quantity, price):
    """Place limit order"""
    try:
        order = client.new_order(
            symbol=symbol,
            side=side,
            type='LIMIT',
            timeInForce='GTC',
            quantity=quantity,
            price=price
        )
        logging.info(f"Limit order placed: {order}")
        return order
    except ClientError as error:
        logging.error(f"Order failed: {error.error_message}")
        return None

def main():
    """Main trading logic"""
    symbol = 'BTCUSDT'

    # Get account balance
    usdt_balance = get_account_balance('USDT')
    logging.info(f"USDT Balance: {usdt_balance}")

    # Get current price
    current_price = get_current_price(symbol)
    logging.info(f"Current BTC Price: {current_price}")

    # Example: Place limit buy order
    if usdt_balance > 10:
        buy_price = current_price * 0.99  # 1% below current price
        quantity = 0.001
        place_limit_order(symbol, 'BUY', quantity, buy_price)

if __name__ == '__main__':
    main()
```

## Useful Links

- Official Documentation: https://developers.binance.com/docs/binance-spot-api-docs
- REST API Documentation: https://developers.binance.com/docs/binance-spot-api-docs/rest-api
- WebSocket Streams: https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams
- WebSocket API: https://developers.binance.com/docs/binance-spot-api-docs/websocket-api
- GitHub Repository: https://github.com/binance/binance-connector-python
- PyPI Package: https://pypi.org/project/binance-connector/
- API Testnet: https://testnet.binance.vision
- Get API Key: https://www.binance.com/en/my/settings/api-management
- API Documentation (Old): https://binance-docs.github.io/apidocs/spot/en/
