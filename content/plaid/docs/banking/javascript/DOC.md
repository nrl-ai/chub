---
name: banking
description: "Plaid API Coding Guidelines for JavaScript/TypeScript using the official Plaid libraries and SDKs"
metadata:
  languages: "javascript"
  versions: "39.1.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "plaid,banking,fintech,payments,financial-data"
---

# Plaid API Coding Guidelines (JavaScript/TypeScript)

You are a Plaid API coding expert. Help me with writing code using the Plaid API calling the official libraries and SDKs.

## Golden Rule: Use the Correct and Current SDK

Always use the official Plaid Node.js SDK for all Plaid API interactions.

- **Library Name:** Plaid Node.js SDK
- **NPM Package:** `plaid`
- **Current Version:** 39.1.0

**Installation:**

```bash
npm install plaid
```

**Import Patterns:**

```typescript
// Correct - ES6 import
import { Configuration, PlaidApi, PlaidEnvironments } from 'plaid';

// Correct - CommonJS
const { Configuration, PlaidApi, PlaidEnvironments } = require('plaid');
```

**Important Notes:**

- The plaid-node client library is updated monthly
- Always use a recent version of the library for new endpoints and fields support
- This library uses semantic versioning with breaking changes indicated by major version bumps
- The library is generated from the OpenAPI spec

## Initialization and Authentication

The Plaid library requires creating a `Configuration` object and `PlaidApi` client instance for all API calls.

```typescript
import { Configuration, PlaidApi, PlaidEnvironments } from 'plaid';

const configuration = new Configuration({
  basePath: PlaidEnvironments.sandbox,
  baseOptions: {
    headers: {
      'PLAID-CLIENT-ID': process.env.PLAID_CLIENT_ID,
      'PLAID-SECRET': process.env.PLAID_SECRET,
      'Plaid-Version': '2020-09-14',
    },
  },
});

const plaidClient = new PlaidApi(configuration);
```

### Environment Configuration

Plaid has multiple environments for different use cases:

```typescript
// Sandbox - for testing with stateful test data
const configuration = new Configuration({
  basePath: PlaidEnvironments.sandbox,
  baseOptions: {
    headers: {
      'PLAID-CLIENT-ID': process.env.PLAID_CLIENT_ID,
      'PLAID-SECRET': process.env.PLAID_SANDBOX_SECRET,
      'Plaid-Version': '2020-09-14',
    },
  },
});

// Development - for testing with live credentials
const configuration = new Configuration({
  basePath: PlaidEnvironments.development,
  baseOptions: {
    headers: {
      'PLAID-CLIENT-ID': process.env.PLAID_CLIENT_ID,
      'PLAID-SECRET': process.env.PLAID_DEVELOPMENT_SECRET,
      'Plaid-Version': '2020-09-14',
    },
  },
});

// Production - for live users
const configuration = new Configuration({
  basePath: PlaidEnvironments.production,
  baseOptions: {
    headers: {
      'PLAID-CLIENT-ID': process.env.PLAID_CLIENT_ID,
      'PLAID-SECRET': process.env.PLAID_PRODUCTION_SECRET,
      'Plaid-Version': '2020-09-14',
    },
  },
});
```

### Environment Variables Setup

```bash
# .env file
PLAID_CLIENT_ID=your_client_id_here
PLAID_SANDBOX_SECRET=your_sandbox_secret_here
PLAID_DEVELOPMENT_SECRET=your_development_secret_here
PLAID_PRODUCTION_SECRET=your_production_secret_here
```

```typescript
// Load environment variables
import dotenv from 'dotenv';
dotenv.config();

const configuration = new Configuration({
  basePath: PlaidEnvironments.sandbox,
  baseOptions: {
    headers: {
      'PLAID-CLIENT-ID': process.env.PLAID_CLIENT_ID,
      'PLAID-SECRET': process.env.PLAID_SANDBOX_SECRET,
      'Plaid-Version': '2020-09-14',
    },
  },
});
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

```typescript
import {
  Configuration,
  PlaidApi,
  PlaidEnvironments,
  Products,
  CountryCode
} from 'plaid';

const plaidClient = new PlaidApi(configuration);

async function createLinkToken(userId: string) {
  const request = {
    user: {
      client_user_id: userId,
    },
    client_name: 'My Application',
    products: [Products.Auth, Products.Transactions],
    country_codes: [CountryCode.Us],
    language: 'en',
    webhook: 'https://your-domain.com/plaid/webhook',
  };

  try {
    const response = await plaidClient.linkTokenCreate(request);
    const linkToken = response.data.link_token;
    return linkToken;
  } catch (error) {
    console.error('Error creating link token:', error);
    throw error;
  }
}
```

### Step 2: Exchange Public Token for Access Token

After the user completes the Link flow, exchange the public_token for a permanent access_token:

```typescript
async function exchangePublicToken(publicToken: string) {
  try {
    const response = await plaidClient.itemPublicTokenExchange({
      public_token: publicToken,
    });

    const accessToken = response.data.access_token;
    const itemId = response.data.item_id;

    // Store these securely in your database
    console.log('Access Token:', accessToken);
    console.log('Item ID:', itemId);

    return { accessToken, itemId };
  } catch (error) {
    console.error('Error exchanging public token:', error);
    throw error;
  }
}
```

## Product APIs

### Auth - Account and Routing Numbers

Retrieve bank account and routing numbers for ACH transfers:

```typescript
async function getAuthData(accessToken: string) {
  try {
    const request = {
      access_token: accessToken,
    };

    const response = await plaidClient.authGet(request);
    const accounts = response.data.accounts;
    const numbers = response.data.numbers;

    console.log('Accounts:', accounts);
    console.log('Account Numbers:', numbers.ach);
    console.log('Routing Numbers:', numbers.ach[0].routing);

    return { accounts, numbers };
  } catch (error) {
    console.error('Error getting auth data:', error);
    throw error;
  }
}
```

**Advanced Auth with Options:**

```typescript
async function getAuthDataWithOptions(accessToken: string, accountIds?: string[]) {
  try {
    const request = {
      access_token: accessToken,
      options: {
        account_ids: accountIds, // Optional: filter specific accounts
      },
    };

    const response = await plaidClient.authGet(request);
    return response.data;
  } catch (error) {
    console.error('Error getting auth data:', error);
    throw error;
  }
}
```

### Accounts - Retrieve Account Information

Get account details including balances and metadata:

```typescript
async function getAccounts(accessToken: string) {
  try {
    const request = {
      access_token: accessToken,
    };

    const response = await plaidClient.accountsGet(request);
    const accounts = response.data.accounts;

    accounts.forEach(account => {
      console.log('Account ID:', account.account_id);
      console.log('Name:', account.name);
      console.log('Type:', account.type);
      console.log('Subtype:', account.subtype);
      console.log('Current Balance:', account.balances.current);
      console.log('Available Balance:', account.balances.available);
    });

    return accounts;
  } catch (error) {
    console.error('Error getting accounts:', error);
    throw error;
  }
}
```

### Balance - Real-time Balance Information

Get up-to-date balance information:

```typescript
async function getBalance(accessToken: string) {
  try {
    const request = {
      access_token: accessToken,
    };

    const response = await plaidClient.accountsBalanceGet(request);
    const accounts = response.data.accounts;

    accounts.forEach(account => {
      console.log(`${account.name}: $${account.balances.current}`);
      console.log(`Available: $${account.balances.available}`);
      console.log(`Currency: ${account.balances.iso_currency_code}`);
    });

    return accounts;
  } catch (error) {
    console.error('Error getting balance:', error);
    throw error;
  }
}
```

**Advanced Balance with Account Filtering:**

```typescript
async function getBalanceForSpecificAccounts(
  accessToken: string,
  accountIds: string[]
) {
  try {
    const request = {
      access_token: accessToken,
      options: {
        account_ids: accountIds,
      },
    };

    const response = await plaidClient.accountsBalanceGet(request);
    return response.data.accounts;
  } catch (error) {
    console.error('Error getting balance:', error);
    throw error;
  }
}
```

### Identity - Account Holder Information

Retrieve identity information for account holders:

```typescript
async function getIdentity(accessToken: string) {
  try {
    const request = {
      access_token: accessToken,
    };

    const response = await plaidClient.identityGet(request);
    const accounts = response.data.accounts;

    accounts.forEach(account => {
      console.log('Account:', account.name);
      account.owners.forEach(owner => {
        console.log('Name:', owner.names);
        console.log('Email:', owner.emails);
        console.log('Phone:', owner.phone_numbers);
        console.log('Address:', owner.addresses);
      });
    });

    return response.data;
  } catch (error) {
    console.error('Error getting identity:', error);
    throw error;
  }
}
```

### Transactions - Transaction History

Plaid provides two methods for retrieving transactions: `/transactions/get` (legacy) and `/transactions/sync` (recommended).

#### Transactions Sync (Recommended)

The `/transactions/sync` endpoint provides incremental updates and is the recommended approach:

```typescript
async function syncTransactions(accessToken: string, cursor?: string) {
  try {
    const request = {
      access_token: accessToken,
      cursor: cursor || undefined,
      count: 100, // Number of transactions to fetch (max 500)
    };

    const response = await plaidClient.transactionsSync(request);

    console.log('Added transactions:', response.data.added.length);
    console.log('Modified transactions:', response.data.modified.length);
    console.log('Removed transactions:', response.data.removed.length);
    console.log('Next cursor:', response.data.next_cursor);
    console.log('Has more:', response.data.has_more);

    // Process transactions
    response.data.added.forEach(transaction => {
      console.log('Date:', transaction.date);
      console.log('Name:', transaction.name);
      console.log('Amount:', transaction.amount);
      console.log('Category:', transaction.category);
    });

    return response.data;
  } catch (error) {
    console.error('Error syncing transactions:', error);
    throw error;
  }
}
```

**Paginated Transaction Sync:**

```typescript
async function getAllTransactions(accessToken: string) {
  let cursor: string | undefined = undefined;
  let allTransactions: any[] = [];
  let hasMore = true;

  try {
    while (hasMore) {
      const request = {
        access_token: accessToken,
        cursor: cursor,
        count: 500, // Use maximum for efficiency
      };

      const response = await plaidClient.transactionsSync(request);

      allTransactions = allTransactions.concat(response.data.added);
      cursor = response.data.next_cursor;
      hasMore = response.data.has_more;

      console.log(`Fetched ${response.data.added.length} transactions`);
    }

    console.log(`Total transactions: ${allTransactions.length}`);
    return allTransactions;
  } catch (error) {
    console.error('Error getting all transactions:', error);
    throw error;
  }
}
```

#### Transactions Get (Legacy)

For retrieving transactions within a specific date range:

```typescript
async function getTransactions(
  accessToken: string,
  startDate: string,
  endDate: string
) {
  try {
    const request = {
      access_token: accessToken,
      start_date: startDate, // Format: 'YYYY-MM-DD'
      end_date: endDate,     // Format: 'YYYY-MM-DD'
      options: {
        count: 250,
        offset: 0,
      },
    };

    const response = await plaidClient.transactionsGet(request);
    const transactions = response.data.transactions;
    const totalTransactions = response.data.total_transactions;

    console.log(`Retrieved ${transactions.length} of ${totalTransactions}`);

    return transactions;
  } catch (error) {
    console.error('Error getting transactions:', error);
    throw error;
  }
}
```

**Paginated Transactions Get:**

```typescript
async function getAllTransactionsInRange(
  accessToken: string,
  startDate: string,
  endDate: string
) {
  let offset = 0;
  const batchSize = 500;
  let allTransactions: any[] = [];
  let totalTransactions = 0;

  try {
    do {
      const request = {
        access_token: accessToken,
        start_date: startDate,
        end_date: endDate,
        options: {
          count: batchSize,
          offset: offset,
        },
      };

      const response = await plaidClient.transactionsGet(request);
      const transactions = response.data.transactions;
      totalTransactions = response.data.total_transactions;

      allTransactions = allTransactions.concat(transactions);
      offset += transactions.length;

      console.log(`Fetched ${allTransactions.length} of ${totalTransactions}`);
    } while (allTransactions.length < totalTransactions);

    return allTransactions;
  } catch (error) {
    console.error('Error getting all transactions:', error);
    throw error;
  }
}
```

### Investments - Holdings and Transactions

Retrieve investment account holdings and transactions:

```typescript
async function getInvestmentHoldings(accessToken: string) {
  try {
    const request = {
      access_token: accessToken,
    };

    const response = await plaidClient.investmentsHoldingsGet(request);
    const holdings = response.data.holdings;
    const securities = response.data.securities;

    holdings.forEach(holding => {
      const security = securities.find(s => s.security_id === holding.security_id);
      console.log('Security:', security?.name);
      console.log('Ticker:', security?.ticker_symbol);
      console.log('Quantity:', holding.quantity);
      console.log('Institution Price:', holding.institution_price);
      console.log('Value:', holding.institution_value);
    });

    return { holdings, securities };
  } catch (error) {
    console.error('Error getting investment holdings:', error);
    throw error;
  }
}
```

**Investment Transactions:**

```typescript
async function getInvestmentTransactions(
  accessToken: string,
  startDate: string,
  endDate: string
) {
  try {
    const request = {
      access_token: accessToken,
      start_date: startDate,
      end_date: endDate,
    };

    const response = await plaidClient.investmentsTransactionsGet(request);
    const transactions = response.data.investment_transactions;

    transactions.forEach(transaction => {
      console.log('Date:', transaction.date);
      console.log('Name:', transaction.name);
      console.log('Type:', transaction.type);
      console.log('Amount:', transaction.amount);
      console.log('Quantity:', transaction.quantity);
      console.log('Price:', transaction.price);
    });

    return transactions;
  } catch (error) {
    console.error('Error getting investment transactions:', error);
    throw error;
  }
}
```

### Liabilities - Loan and Credit Card Data

Access loan balances, interest rates, and credit card information:

```typescript
async function getLiabilities(accessToken: string) {
  try {
    const request = {
      access_token: accessToken,
    };

    const response = await plaidClient.liabilitiesGet(request);
    const liabilities = response.data.liabilities;

    // Credit cards
    liabilities.credit?.forEach(card => {
      console.log('Credit Card:', card.name);
      console.log('APRs:', card.aprs);
      console.log('Last Payment:', card.last_payment_amount);
      console.log('Minimum Payment:', card.minimum_payment_amount);
    });

    // Student loans
    liabilities.student?.forEach(loan => {
      console.log('Student Loan:', loan.account_id);
      console.log('Interest Rate:', loan.interest_rate_percentage);
      console.log('Origination Date:', loan.origination_date);
      console.log('Outstanding Balance:', loan.outstanding_interest_amount);
    });

    // Mortgages
    liabilities.mortgage?.forEach(mortgage => {
      console.log('Mortgage:', mortgage.account_id);
      console.log('Interest Rate:', mortgage.interest_rate.percentage);
      console.log('Origination Date:', mortgage.origination_date);
      console.log('Maturity Date:', mortgage.maturity_date);
    });

    return liabilities;
  } catch (error) {
    console.error('Error getting liabilities:', error);
    throw error;
  }
}
```

### Payment Initiation (UK and Europe)

Create and manage payments:

```typescript
import { PaymentInitiationPaymentCreateRequest } from 'plaid';

async function createPayment(accessToken: string) {
  try {
    const request: PaymentInitiationPaymentCreateRequest = {
      recipient_id: 'recipient_id_from_previous_call',
      reference: 'Invoice #12345',
      amount: {
        currency: 'GBP',
        value: 100.00,
      },
    };

    const response = await plaidClient.paymentInitiationPaymentCreate(request);
    const paymentId = response.data.payment_id;

    console.log('Payment ID:', paymentId);
    return paymentId;
  } catch (error) {
    console.error('Error creating payment:', error);
    throw error;
  }
}
```

**Get Payment Status:**

```typescript
async function getPaymentStatus(paymentId: string) {
  try {
    const request = {
      payment_id: paymentId,
    };

    const response = await plaidClient.paymentInitiationPaymentGet(request);
    const payment = response.data;

    console.log('Status:', payment.status);
    console.log('Amount:', payment.amount);
    console.log('Last Updated:', payment.last_status_update);

    return payment;
  } catch (error) {
    console.error('Error getting payment status:', error);
    throw error;
  }
}
```

## Items Management

An Item represents a user's connection to a financial institution.

### Get Item Information

```typescript
async function getItem(accessToken: string) {
  try {
    const request = {
      access_token: accessToken,
    };

    const response = await plaidClient.itemGet(request);
    const item = response.data.item;

    console.log('Item ID:', item.item_id);
    console.log('Institution ID:', item.institution_id);
    console.log('Available Products:', item.available_products);
    console.log('Billed Products:', item.billed_products);
    console.log('Error:', item.error);

    return item;
  } catch (error) {
    console.error('Error getting item:', error);
    throw error;
  }
}
```

### Remove Item

```typescript
async function removeItem(accessToken: string) {
  try {
    const request = {
      access_token: accessToken,
    };

    const response = await plaidClient.itemRemove(request);
    console.log('Item removed successfully');
    return response.data;
  } catch (error) {
    console.error('Error removing item:', error);
    throw error;
  }
}
```

### Update Item Webhook

```typescript
async function updateWebhook(accessToken: string, newWebhook: string) {
  try {
    const request = {
      access_token: accessToken,
      webhook: newWebhook,
    };

    const response = await plaidClient.itemWebhookUpdate(request);
    const item = response.data.item;

    console.log('Webhook updated to:', newWebhook);
    return item;
  } catch (error) {
    console.error('Error updating webhook:', error);
    throw error;
  }
}
```

## Webhooks

Plaid sends webhook notifications for various events. Configure webhooks via `/link/token/create` or the Plaid Dashboard.

### Webhook Handler Example

```typescript
import express from 'express';

const app = express();
app.use(express.json());

app.post('/plaid/webhook', async (req, res) => {
  const webhook = req.body;

  console.log('Webhook Type:', webhook.webhook_type);
  console.log('Webhook Code:', webhook.webhook_code);

  switch (webhook.webhook_type) {
    case 'TRANSACTIONS':
      await handleTransactionsWebhook(webhook);
      break;
    case 'ITEM':
      await handleItemWebhook(webhook);
      break;
    case 'AUTH':
      await handleAuthWebhook(webhook);
      break;
    default:
      console.log('Unknown webhook type:', webhook.webhook_type);
  }

  res.json({ status: 'received' });
});

async function handleTransactionsWebhook(webhook: any) {
  if (webhook.webhook_code === 'SYNC_UPDATES_AVAILABLE') {
    const itemId = webhook.item_id;
    console.log('New transactions available for item:', itemId);
    // Fetch new transactions using transactionsSync
  }
}

async function handleItemWebhook(webhook: any) {
  if (webhook.webhook_code === 'ERROR') {
    console.log('Item error:', webhook.error);
    // Handle item error
  }
}

async function handleAuthWebhook(webhook: any) {
  if (webhook.webhook_code === 'AUTOMATICALLY_VERIFIED') {
    console.log('Account automatically verified:', webhook.account_id);
  }
}
```

## Sandbox Testing

The Sandbox environment provides test data and utilities for development.

### Fire a Test Webhook

```typescript
async function fireSandboxWebhook(accessToken: string) {
  try {
    const request = {
      access_token: accessToken,
      webhook_code: 'SYNC_UPDATES_AVAILABLE',
    };

    const response = await plaidClient.sandboxItemFireWebhook(request);
    console.log('Webhook fired:', response.data);
    return response.data;
  } catch (error) {
    console.error('Error firing webhook:', error);
    throw error;
  }
}
```

### Reset Sandbox Item Login

```typescript
async function resetSandboxItem(accessToken: string) {
  try {
    const request = {
      access_token: accessToken,
    };

    const response = await plaidClient.sandboxItemResetLogin(request);
    console.log('Item login reset');
    return response.data;
  } catch (error) {
    console.error('Error resetting item:', error);
    throw error;
  }
}
```

### Set Verification Status (Sandbox)

```typescript
async function setVerificationStatus(
  accessToken: string,
  accountId: string,
  verificationStatus: string
) {
  try {
    const request = {
      access_token: accessToken,
      account_id: accountId,
      verification_status: verificationStatus,
    };

    const response = await plaidClient.sandboxItemSetVerificationStatus(request);
    console.log('Verification status set');
    return response.data;
  } catch (error) {
    console.error('Error setting verification status:', error);
    throw error;
  }
}
```

## Error Handling

Plaid errors include an `error_type`, `error_code`, and HTTP status code.

```typescript
import { PlaidError } from 'plaid';

async function makeApiCall(accessToken: string) {
  try {
    const response = await plaidClient.accountsGet({
      access_token: accessToken,
    });
    return response.data;
  } catch (error: any) {
    if (error.response) {
      const plaidError = error.response.data;

      console.log('Error Type:', plaidError.error_type);
      console.log('Error Code:', plaidError.error_code);
      console.log('Error Message:', plaidError.error_message);
      console.log('Display Message:', plaidError.display_message);
      console.log('HTTP Status:', error.response.status);

      switch (plaidError.error_type) {
        case 'ITEM_ERROR':
          if (plaidError.error_code === 'ITEM_LOGIN_REQUIRED') {
            console.log('User needs to re-authenticate');
            // Trigger Link update mode
          }
          break;
        case 'RATE_LIMIT_EXCEEDED':
          console.log('Rate limit exceeded, retry after delay');
          // Implement exponential backoff
          break;
        case 'API_ERROR':
          console.log('Plaid API error, retry request');
          // Retry with idempotency key if available
          break;
        case 'INVALID_REQUEST':
          console.log('Invalid request parameters');
          break;
        case 'INVALID_INPUT':
          console.log('Invalid input data');
          break;
        case 'INSTITUTION_ERROR':
          console.log('Institution is down or experiencing issues');
          break;
        default:
          console.log('Unexpected error type');
      }
    } else {
      console.error('Network or unexpected error:', error);
    }
    throw error;
  }
}
```

### Retry Logic with Exponential Backoff

```typescript
async function makeApiCallWithRetry<T>(
  apiCall: () => Promise<T>,
  maxRetries = 3
): Promise<T> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await apiCall();
    } catch (error: any) {
      const isLastAttempt = attempt === maxRetries;
      const shouldRetry = error.response?.data?.error_type === 'RATE_LIMIT_EXCEEDED' ||
                          error.response?.data?.error_type === 'API_ERROR';

      if (!shouldRetry || isLastAttempt) {
        throw error;
      }

      const delay = Math.pow(2, attempt) * 1000; // Exponential backoff
      console.log(`Retrying after ${delay}ms (attempt ${attempt}/${maxRetries})`);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  throw new Error('Max retries exceeded');
}

// Usage
const accounts = await makeApiCallWithRetry(() =>
  plaidClient.accountsGet({ access_token: accessToken })
);
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

```typescript
// Efficient transactions sync with maximum count
async function efficientTransactionSync(accessToken: string, cursor?: string) {
  const request = {
    access_token: accessToken,
    cursor: cursor,
    count: 500, // Use maximum to reduce number of requests
  };

  const response = await plaidClient.transactionsSync(request);
  return response.data;
}
```

## Complete Integration Example

```typescript
import express from 'express';
import { Configuration, PlaidApi, PlaidEnvironments, Products, CountryCode } from 'plaid';
import dotenv from 'dotenv';

dotenv.config();

const app = express();
app.use(express.json());

// Initialize Plaid client
const configuration = new Configuration({
  basePath: PlaidEnvironments.sandbox,
  baseOptions: {
    headers: {
      'PLAID-CLIENT-ID': process.env.PLAID_CLIENT_ID,
      'PLAID-SECRET': process.env.PLAID_SANDBOX_SECRET,
      'Plaid-Version': '2020-09-14',
    },
  },
});

const plaidClient = new PlaidApi(configuration);

// Store access tokens (use a database in production)
const accessTokenStore = new Map<string, string>();

// Create link token
app.post('/api/create_link_token', async (req, res) => {
  try {
    const { userId } = req.body;

    const response = await plaidClient.linkTokenCreate({
      user: {
        client_user_id: userId,
      },
      client_name: 'My Financial App',
      products: [Products.Auth, Products.Transactions],
      country_codes: [CountryCode.Us],
      language: 'en',
      webhook: 'https://your-domain.com/plaid/webhook',
    });

    res.json({ link_token: response.data.link_token });
  } catch (error) {
    console.error('Error creating link token:', error);
    res.status(500).json({ error: 'Failed to create link token' });
  }
});

// Exchange public token
app.post('/api/exchange_public_token', async (req, res) => {
  try {
    const { public_token, userId } = req.body;

    const response = await plaidClient.itemPublicTokenExchange({
      public_token: public_token,
    });

    const accessToken = response.data.access_token;
    const itemId = response.data.item_id;

    // Store access token securely (use database in production)
    accessTokenStore.set(userId, accessToken);

    res.json({ success: true, item_id: itemId });
  } catch (error) {
    console.error('Error exchanging public token:', error);
    res.status(500).json({ error: 'Failed to exchange public token' });
  }
});

// Get accounts
app.get('/api/accounts/:userId', async (req, res) => {
  try {
    const { userId } = req.params;
    const accessToken = accessTokenStore.get(userId);

    if (!accessToken) {
      return res.status(404).json({ error: 'No access token found' });
    }

    const response = await plaidClient.accountsGet({
      access_token: accessToken,
    });

    res.json({ accounts: response.data.accounts });
  } catch (error) {
    console.error('Error getting accounts:', error);
    res.status(500).json({ error: 'Failed to get accounts' });
  }
});

// Get transactions
app.get('/api/transactions/:userId', async (req, res) => {
  try {
    const { userId } = req.params;
    const accessToken = accessTokenStore.get(userId);

    if (!accessToken) {
      return res.status(404).json({ error: 'No access token found' });
    }

    const response = await plaidClient.transactionsSync({
      access_token: accessToken,
      count: 100,
    });

    res.json({
      added: response.data.added,
      modified: response.data.modified,
      removed: response.data.removed,
      next_cursor: response.data.next_cursor,
      has_more: response.data.has_more,
    });
  } catch (error) {
    console.error('Error getting transactions:', error);
    res.status(500).json({ error: 'Failed to get transactions' });
  }
});

// Webhook handler
app.post('/plaid/webhook', async (req, res) => {
  const webhook = req.body;
  console.log('Received webhook:', webhook);

  if (webhook.webhook_type === 'TRANSACTIONS') {
    if (webhook.webhook_code === 'SYNC_UPDATES_AVAILABLE') {
      // Fetch new transactions
      console.log('New transactions available for item:', webhook.item_id);
    }
  }

  res.json({ status: 'received' });
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});
```

## Useful Links

- **Documentation:** https://plaid.com/docs/
- **API Reference:** https://plaid.com/docs/api/
- **Dashboard:** https://dashboard.plaid.com/
- **GitHub Repository:** https://github.com/plaid/plaid-node
- **Quickstart Guide:** https://plaid.com/docs/quickstart/
- **Changelog:** https://plaid.com/docs/changelog/
