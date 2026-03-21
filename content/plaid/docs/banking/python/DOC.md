---
name: banking
description: "Plaid API Coding Guidelines for Python using the official Plaid libraries and SDKs"
metadata:
  languages: "python"
  versions: "37.1.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "plaid,banking,fintech,payments,financial-data"
---

# Plaid API Coding Guidelines (Python)

You are a Plaid API coding expert. Help me with writing code using the Plaid API calling the official libraries and SDKs.

## Golden Rule: Use the Correct and Current SDK

Always use the official Plaid Python SDK for all Plaid API interactions.

- **Library Name:** Plaid Python SDK
- **PyPI Package:** `plaid-python`
- **Current Version:** 37.1.0

**Installation:**

```bash
pip install plaid-python
```

**Important Notes:**

- The plaid-python client library is updated monthly
- This library only supports Python 3 (Python >=3.6)
- This release only supports the latest Plaid API version: 2020-09-14
- The library is generated from the Plaid OpenAPI spec
- Always use a recent version for new endpoints and fields support

## Initialization and Authentication

The Plaid library requires creating a `Configuration` object, `ApiClient`, and `PlaidApi` instance for all API calls.

```python
import plaid
from plaid.api import plaid_api

configuration = plaid.Configuration(
    host=plaid.Environment.Sandbox,
    api_key={
        'clientId': 'your_client_id',
        'secret': 'your_sandbox_secret',
    }
)

api_client = plaid.ApiClient(configuration)
client = plaid_api.PlaidApi(api_client)
```

### Environment Configuration

Plaid has multiple environments for different use cases:

```python
import plaid
from plaid.api import plaid_api

# Sandbox - for testing with stateful test data
configuration = plaid.Configuration(
    host=plaid.Environment.Sandbox,
    api_key={
        'clientId': 'your_client_id',
        'secret': 'your_sandbox_secret',
    }
)

# Development - for testing with live credentials
configuration = plaid.Configuration(
    host=plaid.Environment.Development,
    api_key={
        'clientId': 'your_client_id',
        'secret': 'your_development_secret',
    }
)

# Production - for live users
configuration = plaid.Configuration(
    host=plaid.Environment.Production,
    api_key={
        'clientId': 'your_client_id',
        'secret': 'your_production_secret',
    }
)

api_client = plaid.ApiClient(configuration)
client = plaid_api.PlaidApi(api_client)
```

### Environment Variables Setup

```python
import os
import plaid
from plaid.api import plaid_api

# Load from environment variables
configuration = plaid.Configuration(
    host=plaid.Environment.Sandbox,
    api_key={
        'clientId': os.environ['PLAID_CLIENT_ID'],
        'secret': os.environ['PLAID_SANDBOX_SECRET'],
    }
)

api_client = plaid.ApiClient(configuration)
client = plaid_api.PlaidApi(api_client)
```

**.env file:**

```bash
PLAID_CLIENT_ID=your_client_id_here
PLAID_SANDBOX_SECRET=your_sandbox_secret_here
PLAID_DEVELOPMENT_SECRET=your_development_secret_here
PLAID_PRODUCTION_SECRET=your_production_secret_here
```

**Using python-dotenv:**

```python
import os
from dotenv import load_dotenv
import plaid
from plaid.api import plaid_api

load_dotenv()

configuration = plaid.Configuration(
    host=plaid.Environment.Sandbox,
    api_key={
        'clientId': os.environ['PLAID_CLIENT_ID'],
        'secret': os.environ['PLAID_SANDBOX_SECRET'],
    }
)

api_client = plaid.ApiClient(configuration)
client = plaid_api.PlaidApi(api_client)
```

## Core Plaid Flow

The standard Plaid integration flow follows these steps:

1. Create a link_token
2. Initialize Plaid Link on the frontend
3. Receive a public_token from Link
4. Exchange the public_token for an access_token
5. Use the access_token to make API requests

### Step 1: Create Link Token

Create a temporary link_token to authenticate your app with Plaid Link:

```python
import plaid
from plaid.api import plaid_api
from plaid.model.link_token_create_request import LinkTokenCreateRequest
from plaid.model.link_token_create_request_user import LinkTokenCreateRequestUser
from plaid.model.products import Products
from plaid.model.country_code import CountryCode

configuration = plaid.Configuration(
    host=plaid.Environment.Sandbox,
    api_key={
        'clientId': os.environ['PLAID_CLIENT_ID'],
        'secret': os.environ['PLAID_SANDBOX_SECRET'],
    }
)

api_client = plaid.ApiClient(configuration)
client = plaid_api.PlaidApi(api_client)

def create_link_token(user_id: str) -> str:
    try:
        request = LinkTokenCreateRequest(
            user=LinkTokenCreateRequestUser(
                client_user_id=user_id
            ),
            client_name='My Application',
            products=[Products('auth'), Products('transactions')],
            country_codes=[CountryCode('US')],
            language='en',
            webhook='https://your-domain.com/plaid/webhook'
        )

        response = client.link_token_create(request)
        link_token = response['link_token']

        print(f'Link token: {link_token}')
        return link_token
    except plaid.ApiException as e:
        print(f'Error creating link token: {e}')
        raise
```

### Step 2: Exchange Public Token for Access Token

After the user completes the Link flow, exchange the public_token for a permanent access_token:

```python
from plaid.model.item_public_token_exchange_request import ItemPublicTokenExchangeRequest

def exchange_public_token(public_token: str) -> dict:
    try:
        request = ItemPublicTokenExchangeRequest(
            public_token=public_token
        )

        response = client.item_public_token_exchange(request)
        access_token = response['access_token']
        item_id = response['item_id']

        # Store these securely in your database
        print(f'Access Token: {access_token}')
        print(f'Item ID: {item_id}')

        return {
            'access_token': access_token,
            'item_id': item_id
        }
    except plaid.ApiException as e:
        print(f'Error exchanging public token: {e}')
        raise
```

## Product APIs

### Auth - Account and Routing Numbers

Retrieve bank account and routing numbers for ACH transfers:

```python
from plaid.model.auth_get_request import AuthGetRequest

def get_auth_data(access_token: str) -> dict:
    try:
        request = AuthGetRequest(
            access_token=access_token
        )

        response = client.auth_get(request)
        accounts = response['accounts']
        numbers = response['numbers']

        print('Accounts:', accounts)
        print('Account Numbers:', numbers['ach'])

        for ach in numbers['ach']:
            print(f'Account: {ach["account_id"]}')
            print(f'Account Number: {ach["account"]}')
            print(f'Routing Number: {ach["routing"]}')

        return {
            'accounts': accounts,
            'numbers': numbers
        }
    except plaid.ApiException as e:
        print(f'Error getting auth data: {e}')
        raise
```

**Advanced Auth with Options:**

```python
from plaid.model.auth_get_request import AuthGetRequest
from plaid.model.auth_get_request_options import AuthGetRequestOptions

def get_auth_data_with_options(access_token: str, account_ids: list = None) -> dict:
    try:
        options = AuthGetRequestOptions(
            account_ids=account_ids
        ) if account_ids else None

        request = AuthGetRequest(
            access_token=access_token,
            options=options
        )

        response = client.auth_get(request)
        return response.to_dict()
    except plaid.ApiException as e:
        print(f'Error getting auth data: {e}')
        raise
```

### Accounts - Retrieve Account Information

Get account details including balances and metadata:

```python
from plaid.model.accounts_get_request import AccountsGetRequest

def get_accounts(access_token: str) -> list:
    try:
        request = AccountsGetRequest(
            access_token=access_token
        )

        response = client.accounts_get(request)
        accounts = response['accounts']

        for account in accounts:
            print(f'Account ID: {account["account_id"]}')
            print(f'Name: {account["name"]}')
            print(f'Type: {account["type"]}')
            print(f'Subtype: {account["subtype"]}')
            print(f'Current Balance: ${account["balances"]["current"]}')
            print(f'Available Balance: ${account["balances"]["available"]}')
            print('---')

        return accounts
    except plaid.ApiException as e:
        print(f'Error getting accounts: {e}')
        raise
```

### Balance - Real-time Balance Information

Get up-to-date balance information:

```python
from plaid.model.accounts_balance_get_request import AccountsBalanceGetRequest

def get_balance(access_token: str) -> list:
    try:
        request = AccountsBalanceGetRequest(
            access_token=access_token
        )

        response = client.accounts_balance_get(request)
        accounts = response['accounts']

        for account in accounts:
            balances = account['balances']
            print(f'{account["name"]}: ${balances["current"]}')
            print(f'Available: ${balances["available"]}')
            print(f'Currency: {balances["iso_currency_code"]}')
            print('---')

        return accounts
    except plaid.ApiException as e:
        print(f'Error getting balance: {e}')
        raise
```

**Advanced Balance with Account Filtering:**

```python
from plaid.model.accounts_balance_get_request import AccountsBalanceGetRequest
from plaid.model.accounts_balance_get_request_options import AccountsBalanceGetRequestOptions

def get_balance_for_specific_accounts(access_token: str, account_ids: list) -> list:
    try:
        options = AccountsBalanceGetRequestOptions(
            account_ids=account_ids
        )

        request = AccountsBalanceGetRequest(
            access_token=access_token,
            options=options
        )

        response = client.accounts_balance_get(request)
        return response['accounts']
    except plaid.ApiException as e:
        print(f'Error getting balance: {e}')
        raise
```

### Identity - Account Holder Information

Retrieve identity information for account holders:

```python
from plaid.model.identity_get_request import IdentityGetRequest

def get_identity(access_token: str) -> dict:
    try:
        request = IdentityGetRequest(
            access_token=access_token
        )

        response = client.identity_get(request)
        accounts = response['accounts']

        for account in accounts:
            print(f'Account: {account["name"]}')
            for owner in account['owners']:
                print(f'Names: {owner["names"]}')
                print(f'Emails: {owner["emails"]}')
                print(f'Phone Numbers: {owner["phone_numbers"]}')
                print(f'Addresses: {owner["addresses"]}')
                print('---')

        return response.to_dict()
    except plaid.ApiException as e:
        print(f'Error getting identity: {e}')
        raise
```

### Transactions - Transaction History

Plaid provides two methods for retrieving transactions: `/transactions/get` (legacy) and `/transactions/sync` (recommended).

#### Transactions Sync (Recommended)

The `/transactions/sync` endpoint provides incremental updates and is the recommended approach:

```python
from plaid.model.transactions_sync_request import TransactionsSyncRequest

def sync_transactions(access_token: str, cursor: str = None) -> dict:
    try:
        request = TransactionsSyncRequest(
            access_token=access_token,
            cursor=cursor,
            count=100  # Number of transactions to fetch (max 500)
        )

        response = client.transactions_sync(request)

        print(f'Added transactions: {len(response["added"])}')
        print(f'Modified transactions: {len(response["modified"])}')
        print(f'Removed transactions: {len(response["removed"])}')
        print(f'Next cursor: {response["next_cursor"]}')
        print(f'Has more: {response["has_more"]}')

        # Process transactions
        for transaction in response['added']:
            print(f'Date: {transaction["date"]}')
            print(f'Name: {transaction["name"]}')
            print(f'Amount: ${transaction["amount"]}')
            print(f'Category: {transaction["category"]}')
            print('---')

        return response.to_dict()
    except plaid.ApiException as e:
        print(f'Error syncing transactions: {e}')
        raise
```

**Paginated Transaction Sync:**

```python
from plaid.model.transactions_sync_request import TransactionsSyncRequest

def get_all_transactions(access_token: str) -> list:
    cursor = None
    all_transactions = []
    has_more = True

    try:
        while has_more:
            request = TransactionsSyncRequest(
                access_token=access_token,
                cursor=cursor,
                count=500  # Use maximum for efficiency
            )

            response = client.transactions_sync(request)

            all_transactions.extend(response['added'])
            cursor = response['next_cursor']
            has_more = response['has_more']

            print(f'Fetched {len(response["added"])} transactions')

        print(f'Total transactions: {len(all_transactions)}')
        return all_transactions
    except plaid.ApiException as e:
        print(f'Error getting all transactions: {e}')
        raise
```

#### Transactions Get (Legacy)

For retrieving transactions within a specific date range:

```python
from plaid.model.transactions_get_request import TransactionsGetRequest
from plaid.model.transactions_get_request_options import TransactionsGetRequestOptions
from datetime import datetime, timedelta

def get_transactions(access_token: str, start_date: str, end_date: str) -> list:
    try:
        options = TransactionsGetRequestOptions(
            count=250,
            offset=0
        )

        request = TransactionsGetRequest(
            access_token=access_token,
            start_date=datetime.strptime(start_date, '%Y-%m-%d').date(),
            end_date=datetime.strptime(end_date, '%Y-%m-%d').date(),
            options=options
        )

        response = client.transactions_get(request)
        transactions = response['transactions']
        total_transactions = response['total_transactions']

        print(f'Retrieved {len(transactions)} of {total_transactions}')

        return transactions
    except plaid.ApiException as e:
        print(f'Error getting transactions: {e}')
        raise
```

**Paginated Transactions Get:**

```python
from plaid.model.transactions_get_request import TransactionsGetRequest
from plaid.model.transactions_get_request_options import TransactionsGetRequestOptions
from datetime import datetime

def get_all_transactions_in_range(access_token: str, start_date: str, end_date: str) -> list:
    offset = 0
    batch_size = 500
    all_transactions = []
    total_transactions = 0

    try:
        while True:
            options = TransactionsGetRequestOptions(
                count=batch_size,
                offset=offset
            )

            request = TransactionsGetRequest(
                access_token=access_token,
                start_date=datetime.strptime(start_date, '%Y-%m-%d').date(),
                end_date=datetime.strptime(end_date, '%Y-%m-%d').date(),
                options=options
            )

            response = client.transactions_get(request)
            transactions = response['transactions']
            total_transactions = response['total_transactions']

            all_transactions.extend(transactions)
            offset += len(transactions)

            print(f'Fetched {len(all_transactions)} of {total_transactions}')

            if len(all_transactions) >= total_transactions:
                break

        return all_transactions
    except plaid.ApiException as e:
        print(f'Error getting all transactions: {e}')
        raise
```

### Investments - Holdings and Transactions

Retrieve investment account holdings and transactions:

```python
from plaid.model.investments_holdings_get_request import InvestmentsHoldingsGetRequest

def get_investment_holdings(access_token: str) -> dict:
    try:
        request = InvestmentsHoldingsGetRequest(
            access_token=access_token
        )

        response = client.investments_holdings_get(request)
        holdings = response['holdings']
        securities = response['securities']

        # Create security lookup dictionary
        security_map = {s['security_id']: s for s in securities}

        for holding in holdings:
            security = security_map.get(holding['security_id'])
            if security:
                print(f'Security: {security["name"]}')
                print(f'Ticker: {security.get("ticker_symbol", "N/A")}')
            print(f'Quantity: {holding["quantity"]}')
            print(f'Institution Price: ${holding["institution_price"]}')
            print(f'Value: ${holding["institution_value"]}')
            print('---')

        return {
            'holdings': holdings,
            'securities': securities
        }
    except plaid.ApiException as e:
        print(f'Error getting investment holdings: {e}')
        raise
```

**Investment Transactions:**

```python
from plaid.model.investments_transactions_get_request import InvestmentsTransactionsGetRequest
from datetime import datetime

def get_investment_transactions(access_token: str, start_date: str, end_date: str) -> list:
    try:
        request = InvestmentsTransactionsGetRequest(
            access_token=access_token,
            start_date=datetime.strptime(start_date, '%Y-%m-%d').date(),
            end_date=datetime.strptime(end_date, '%Y-%m-%d').date()
        )

        response = client.investments_transactions_get(request)
        transactions = response['investment_transactions']

        for transaction in transactions:
            print(f'Date: {transaction["date"]}')
            print(f'Name: {transaction["name"]}')
            print(f'Type: {transaction["type"]}')
            print(f'Amount: ${transaction["amount"]}')
            print(f'Quantity: {transaction["quantity"]}')
            print(f'Price: ${transaction["price"]}')
            print('---')

        return transactions
    except plaid.ApiException as e:
        print(f'Error getting investment transactions: {e}')
        raise
```

### Liabilities - Loan and Credit Card Data

Access loan balances, interest rates, and credit card information:

```python
from plaid.model.liabilities_get_request import LiabilitiesGetRequest

def get_liabilities(access_token: str) -> dict:
    try:
        request = LiabilitiesGetRequest(
            access_token=access_token
        )

        response = client.liabilities_get(request)
        liabilities = response['liabilities']

        # Credit cards
        if 'credit' in liabilities and liabilities['credit']:
            print('Credit Cards:')
            for card in liabilities['credit']:
                print(f'  Name: {card.get("name", "N/A")}')
                print(f'  APRs: {card.get("aprs", [])}')
                print(f'  Last Payment: ${card.get("last_payment_amount", 0)}')
                print(f'  Minimum Payment: ${card.get("minimum_payment_amount", 0)}')
                print('---')

        # Student loans
        if 'student' in liabilities and liabilities['student']:
            print('Student Loans:')
            for loan in liabilities['student']:
                print(f'  Account ID: {loan["account_id"]}')
                print(f'  Interest Rate: {loan.get("interest_rate_percentage", 0)}%')
                print(f'  Origination Date: {loan.get("origination_date", "N/A")}')
                print(f'  Outstanding Interest: ${loan.get("outstanding_interest_amount", 0)}')
                print('---')

        # Mortgages
        if 'mortgage' in liabilities and liabilities['mortgage']:
            print('Mortgages:')
            for mortgage in liabilities['mortgage']:
                print(f'  Account ID: {mortgage["account_id"]}')
                if 'interest_rate' in mortgage:
                    print(f'  Interest Rate: {mortgage["interest_rate"].get("percentage", 0)}%')
                print(f'  Origination Date: {mortgage.get("origination_date", "N/A")}')
                print(f'  Maturity Date: {mortgage.get("maturity_date", "N/A")}')
                print('---')

        return liabilities
    except plaid.ApiException as e:
        print(f'Error getting liabilities: {e}')
        raise
```

### Payment Initiation (UK and Europe)

Create and manage payments:

```python
from plaid.model.payment_initiation_payment_create_request import PaymentInitiationPaymentCreateRequest
from plaid.model.payment_amount import PaymentAmount

def create_payment(recipient_id: str) -> str:
    try:
        amount = PaymentAmount(
            currency='GBP',
            value=100.00
        )

        request = PaymentInitiationPaymentCreateRequest(
            recipient_id=recipient_id,
            reference='Invoice #12345',
            amount=amount
        )

        response = client.payment_initiation_payment_create(request)
        payment_id = response['payment_id']

        print(f'Payment ID: {payment_id}')
        return payment_id
    except plaid.ApiException as e:
        print(f'Error creating payment: {e}')
        raise
```

**Get Payment Status:**

```python
from plaid.model.payment_initiation_payment_get_request import PaymentInitiationPaymentGetRequest

def get_payment_status(payment_id: str) -> dict:
    try:
        request = PaymentInitiationPaymentGetRequest(
            payment_id=payment_id
        )

        response = client.payment_initiation_payment_get(request)
        payment = response.to_dict()

        print(f'Status: {payment["status"]}')
        print(f'Amount: {payment["amount"]}')
        print(f'Last Updated: {payment.get("last_status_update", "N/A")}')

        return payment
    except plaid.ApiException as e:
        print(f'Error getting payment status: {e}')
        raise
```

## Items Management

An Item represents a user's connection to a financial institution.

### Get Item Information

```python
from plaid.model.item_get_request import ItemGetRequest

def get_item(access_token: str) -> dict:
    try:
        request = ItemGetRequest(
            access_token=access_token
        )

        response = client.item_get(request)
        item = response['item']

        print(f'Item ID: {item["item_id"]}')
        print(f'Institution ID: {item.get("institution_id", "N/A")}')
        print(f'Available Products: {item.get("available_products", [])}')
        print(f'Billed Products: {item.get("billed_products", [])}')
        if 'error' in item and item['error']:
            print(f'Error: {item["error"]}')

        return item
    except plaid.ApiException as e:
        print(f'Error getting item: {e}')
        raise
```

### Remove Item

```python
from plaid.model.item_remove_request import ItemRemoveRequest

def remove_item(access_token: str) -> dict:
    try:
        request = ItemRemoveRequest(
            access_token=access_token
        )

        response = client.item_remove(request)
        print('Item removed successfully')
        return response.to_dict()
    except plaid.ApiException as e:
        print(f'Error removing item: {e}')
        raise
```

### Update Item Webhook

```python
from plaid.model.item_webhook_update_request import ItemWebhookUpdateRequest

def update_webhook(access_token: str, new_webhook: str) -> dict:
    try:
        request = ItemWebhookUpdateRequest(
            access_token=access_token,
            webhook=new_webhook
        )

        response = client.item_webhook_update(request)
        item = response['item']

        print(f'Webhook updated to: {new_webhook}')
        return item
    except plaid.ApiException as e:
        print(f'Error updating webhook: {e}')
        raise
```

## Webhooks

Plaid sends webhook notifications for various events. Configure webhooks via `/link/token/create` or the Plaid Dashboard.

### Webhook Handler Example (Flask)

```python
from flask import Flask, request, jsonify
import plaid
from plaid.api import plaid_api

app = Flask(__name__)

# Initialize Plaid client
configuration = plaid.Configuration(
    host=plaid.Environment.Sandbox,
    api_key={
        'clientId': os.environ['PLAID_CLIENT_ID'],
        'secret': os.environ['PLAID_SANDBOX_SECRET'],
    }
)

api_client = plaid.ApiClient(configuration)
client = plaid_api.PlaidApi(api_client)

@app.route('/plaid/webhook', methods=['POST'])
def plaid_webhook():
    webhook = request.json

    print(f'Webhook Type: {webhook["webhook_type"]}')
    print(f'Webhook Code: {webhook["webhook_code"]}')

    webhook_type = webhook['webhook_type']

    if webhook_type == 'TRANSACTIONS':
        handle_transactions_webhook(webhook)
    elif webhook_type == 'ITEM':
        handle_item_webhook(webhook)
    elif webhook_type == 'AUTH':
        handle_auth_webhook(webhook)
    else:
        print(f'Unknown webhook type: {webhook_type}')

    return jsonify({'status': 'received'}), 200

def handle_transactions_webhook(webhook):
    if webhook['webhook_code'] == 'SYNC_UPDATES_AVAILABLE':
        item_id = webhook['item_id']
        print(f'New transactions available for item: {item_id}')
        # Fetch new transactions using transactions_sync

def handle_item_webhook(webhook):
    if webhook['webhook_code'] == 'ERROR':
        print(f'Item error: {webhook.get("error", {})}')
        # Handle item error

def handle_auth_webhook(webhook):
    if webhook['webhook_code'] == 'AUTOMATICALLY_VERIFIED':
        print(f'Account automatically verified: {webhook.get("account_id", "N/A")}')

if __name__ == '__main__':
    app.run(port=5000)
```

## Sandbox Testing

The Sandbox environment provides test data and utilities for development.

### Fire a Test Webhook

```python
from plaid.model.sandbox_item_fire_webhook_request import SandboxItemFireWebhookRequest

def fire_sandbox_webhook(access_token: str) -> dict:
    try:
        request = SandboxItemFireWebhookRequest(
            access_token=access_token,
            webhook_code='SYNC_UPDATES_AVAILABLE'
        )

        response = client.sandbox_item_fire_webhook(request)
        print(f'Webhook fired: {response}')
        return response.to_dict()
    except plaid.ApiException as e:
        print(f'Error firing webhook: {e}')
        raise
```

### Reset Sandbox Item Login

```python
from plaid.model.sandbox_item_reset_login_request import SandboxItemResetLoginRequest

def reset_sandbox_item(access_token: str) -> dict:
    try:
        request = SandboxItemResetLoginRequest(
            access_token=access_token
        )

        response = client.sandbox_item_reset_login(request)
        print('Item login reset')
        return response.to_dict()
    except plaid.ApiException as e:
        print(f'Error resetting item: {e}')
        raise
```

### Set Verification Status (Sandbox)

```python
from plaid.model.sandbox_item_set_verification_status_request import SandboxItemSetVerificationStatusRequest

def set_verification_status(access_token: str, account_id: str, verification_status: str) -> dict:
    try:
        request = SandboxItemSetVerificationStatusRequest(
            access_token=access_token,
            account_id=account_id,
            verification_status=verification_status
        )

        response = client.sandbox_item_set_verification_status(request)
        print('Verification status set')
        return response.to_dict()
    except plaid.ApiException as e:
        print(f'Error setting verification status: {e}')
        raise
```

## Error Handling

Plaid errors include an `error_type`, `error_code`, and HTTP status code.

```python
import plaid
from plaid.api import plaid_api
from plaid.model.accounts_get_request import AccountsGetRequest

def make_api_call(access_token: str):
    try:
        request = AccountsGetRequest(
            access_token=access_token
        )

        response = client.accounts_get(request)
        return response.to_dict()
    except plaid.ApiException as e:
        error = e.body

        print(f'Error Type: {error.get("error_type", "N/A")}')
        print(f'Error Code: {error.get("error_code", "N/A")}')
        print(f'Error Message: {error.get("error_message", "N/A")}')
        print(f'Display Message: {error.get("display_message", "N/A")}')
        print(f'HTTP Status: {e.status}')

        error_type = error.get('error_type')
        error_code = error.get('error_code')

        if error_type == 'ITEM_ERROR':
            if error_code == 'ITEM_LOGIN_REQUIRED':
                print('User needs to re-authenticate')
                # Trigger Link update mode
        elif error_type == 'RATE_LIMIT_EXCEEDED':
            print('Rate limit exceeded, retry after delay')
            # Implement exponential backoff
        elif error_type == 'API_ERROR':
            print('Plaid API error, retry request')
            # Retry with idempotency key if available
        elif error_type == 'INVALID_REQUEST':
            print('Invalid request parameters')
        elif error_type == 'INVALID_INPUT':
            print('Invalid input data')
        elif error_type == 'INSTITUTION_ERROR':
            print('Institution is down or experiencing issues')
        else:
            print('Unexpected error type')

        raise
```

### Retry Logic with Exponential Backoff

```python
import time
import plaid

def make_api_call_with_retry(api_call, max_retries=3):
    for attempt in range(1, max_retries + 1):
        try:
            return api_call()
        except plaid.ApiException as e:
            is_last_attempt = attempt == max_retries
            error = e.body
            error_type = error.get('error_type')

            should_retry = error_type in ['RATE_LIMIT_EXCEEDED', 'API_ERROR']

            if not should_retry or is_last_attempt:
                raise

            delay = 2 ** attempt  # Exponential backoff
            print(f'Retrying after {delay}s (attempt {attempt}/{max_retries})')
            time.sleep(delay)

    raise Exception('Max retries exceeded')

# Usage
def get_accounts_safe(access_token: str):
    def api_call():
        request = AccountsGetRequest(access_token=access_token)
        return client.accounts_get(request)

    return make_api_call_with_retry(api_call)
```

## Rate Limits

Plaid enforces rate limits to ensure API stability:

- `/auth/get`: 15 requests per Item per minute (Production)
- `/institutions/get`: 25 requests per client per minute (Production), 10 requests per client per minute (Sandbox)
- Most other endpoints: Custom limits based on your account

To reduce rate limit errors:

- Increase the `count` parameter in `/transactions/sync` to the maximum of 500
- Cache responses when appropriate
- Implement exponential backoff retry logic
- Use webhooks instead of polling for updates

```python
from plaid.model.transactions_sync_request import TransactionsSyncRequest

def efficient_transaction_sync(access_token: str, cursor: str = None) -> dict:
    # Use maximum count to reduce number of requests
    request = TransactionsSyncRequest(
        access_token=access_token,
        cursor=cursor,
        count=500
    )

    response = client.transactions_sync(request)
    return response.to_dict()
```

## Complete Integration Example (Flask)

```python
import os
from flask import Flask, request, jsonify
from dotenv import load_dotenv
import plaid
from plaid.api import plaid_api
from plaid.model.link_token_create_request import LinkTokenCreateRequest
from plaid.model.link_token_create_request_user import LinkTokenCreateRequestUser
from plaid.model.item_public_token_exchange_request import ItemPublicTokenExchangeRequest
from plaid.model.accounts_get_request import AccountsGetRequest
from plaid.model.transactions_sync_request import TransactionsSyncRequest
from plaid.model.products import Products
from plaid.model.country_code import CountryCode

load_dotenv()

app = Flask(__name__)

# Initialize Plaid client
configuration = plaid.Configuration(
    host=plaid.Environment.Sandbox,
    api_key={
        'clientId': os.environ['PLAID_CLIENT_ID'],
        'secret': os.environ['PLAID_SANDBOX_SECRET'],
    }
)

api_client = plaid.ApiClient(configuration)
client = plaid_api.PlaidApi(api_client)

# Store access tokens (use a database in production)
access_token_store = {}

@app.route('/api/create_link_token', methods=['POST'])
def create_link_token():
    try:
        user_id = request.json.get('user_id')

        link_request = LinkTokenCreateRequest(
            user=LinkTokenCreateRequestUser(
                client_user_id=user_id
            ),
            client_name='My Financial App',
            products=[Products('auth'), Products('transactions')],
            country_codes=[CountryCode('US')],
            language='en',
            webhook='https://your-domain.com/plaid/webhook'
        )

        response = client.link_token_create(link_request)
        return jsonify({'link_token': response['link_token']})
    except plaid.ApiException as e:
        print(f'Error creating link token: {e}')
        return jsonify({'error': 'Failed to create link token'}), 500

@app.route('/api/exchange_public_token', methods=['POST'])
def exchange_public_token():
    try:
        public_token = request.json.get('public_token')
        user_id = request.json.get('user_id')

        exchange_request = ItemPublicTokenExchangeRequest(
            public_token=public_token
        )

        response = client.item_public_token_exchange(exchange_request)
        access_token = response['access_token']
        item_id = response['item_id']

        # Store access token securely (use database in production)
        access_token_store[user_id] = access_token

        return jsonify({'success': True, 'item_id': item_id})
    except plaid.ApiException as e:
        print(f'Error exchanging public token: {e}')
        return jsonify({'error': 'Failed to exchange public token'}), 500

@app.route('/api/accounts/<user_id>', methods=['GET'])
def get_accounts(user_id):
    try:
        access_token = access_token_store.get(user_id)

        if not access_token:
            return jsonify({'error': 'No access token found'}), 404

        accounts_request = AccountsGetRequest(
            access_token=access_token
        )

        response = client.accounts_get(accounts_request)
        return jsonify({'accounts': response['accounts']})
    except plaid.ApiException as e:
        print(f'Error getting accounts: {e}')
        return jsonify({'error': 'Failed to get accounts'}), 500

@app.route('/api/transactions/<user_id>', methods=['GET'])
def get_transactions(user_id):
    try:
        access_token = access_token_store.get(user_id)

        if not access_token:
            return jsonify({'error': 'No access token found'}), 404

        transactions_request = TransactionsSyncRequest(
            access_token=access_token,
            count=100
        )

        response = client.transactions_sync(transactions_request)

        return jsonify({
            'added': response['added'],
            'modified': response['modified'],
            'removed': response['removed'],
            'next_cursor': response['next_cursor'],
            'has_more': response['has_more']
        })
    except plaid.ApiException as e:
        print(f'Error getting transactions: {e}')
        return jsonify({'error': 'Failed to get transactions'}), 500

@app.route('/plaid/webhook', methods=['POST'])
def plaid_webhook():
    webhook = request.json
    print(f'Received webhook: {webhook}')

    if webhook['webhook_type'] == 'TRANSACTIONS':
        if webhook['webhook_code'] == 'SYNC_UPDATES_AVAILABLE':
            print(f'New transactions available for item: {webhook["item_id"]}')

    return jsonify({'status': 'received'}), 200

if __name__ == '__main__':
    app.run(port=5000, debug=True)
```

## Useful Links

- **Documentation:** https://plaid.com/docs/
- **API Reference:** https://plaid.com/docs/api/
- **Dashboard:** https://dashboard.plaid.com/
- **GitHub Repository:** https://github.com/plaid/plaid-python
- **Quickstart Guide:** https://plaid.com/docs/quickstart/
- **Changelog:** https://plaid.com/docs/changelog/
