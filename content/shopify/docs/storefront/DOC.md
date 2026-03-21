---
name: storefront
description: "Shopify Storefront API library for building ecommerce integrations with OAuth and the official JavaScript SDK"
metadata:
  languages: "javascript"
  versions: "12.1.0"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "shopify,storefront,ecommerce,api,oauth"
---

# Shopify API Library - Comprehensive Coding Guide

## 1. Golden Rule

**Always use the official `@shopify/shopify-api` package.** 

**Installation command:**
```bash
pnpm add @shopify/shopify-api
```

**Warning:** Do not use deprecated packages like `shopify-api-node` or unofficial alternatives. This library is the official TypeScript/JavaScript SDK for Shopify's Admin API. 

## 2. Installation

### Package Installation

```bash
# npm
npm install @shopify/shopify-api

# yarn  
yarn add @shopify/shopify-api

# pnpm (recommended)
pnpm add @shopify/shopify-api
``` 

### Environment Variables

```bash
# Required environment variables
SHOPIFY_API_KEY=your_api_key_from_partners_dashboard
SHOPIFY_API_SECRET=your_api_secret_from_partners_dashboard
SHOPIFY_APP_URL=https://your-app-domain.com
SHOPIFY_SCOPES=read_products,write_products
``` 

## 3. Initialization

### Runtime Adapter Import (Required First Step)

**Critical:** Import the appropriate runtime adapter before using the library.

```typescript
// Node.js
import '@shopify/shopify-api/adapters/node';

// Cloudflare Workers
import '@shopify/shopify-api/adapters/cf-worker';

// Web API (generic runtimes)
import '@shopify/shopify-api/adapters/web-api';
``` 

### Basic Configuration

```typescript
import '@shopify/shopify-api/adapters/node';
import { shopifyApi, ApiVersion } from '@shopify/shopify-api';

const shopify = shopifyApi({
  apiKey: process.env.SHOPIFY_API_KEY,
  apiSecretKey: process.env.SHOPIFY_API_SECRET,
  scopes: ['read_products', 'write_products'],
  hostName: process.env.SHOPIFY_APP_URL,
  apiVersion: ApiVersion.July25,
});
``` 

### Custom Store App Configuration

```typescript
const shopify = shopifyApi({
  apiSecretKey: "App_API_secret_key",
  apiVersion: ApiVersion.April23,
  isCustomStoreApp: true,
  adminApiAccessToken: "Admin_API_Access_Token",
  isEmbeddedApp: false,
  hostName: "my-shop.myshopify.com",
});
```

## 4. Core API Surfaces

### Authentication & OAuth

#### Minimal Example - Begin OAuth
```typescript
await shopify.auth.begin({
  shop: 'my-shop.myshopify.com',
  callbackPath: '/auth/callback',
  isOnline: true,
  rawRequest: req,
  rawResponse: res,
});
``` 

#### Advanced Example - OAuth Callback
```typescript
const callbackResponse = await shopify.auth.callback({
  rawRequest: req,
  rawResponse: res,
});

const { session } = callbackResponse;
// Store session for future API calls
```

### REST API Client

#### Minimal Example
```typescript
const client = new shopify.clients.Rest({ session });
const response = await client.get({
  path: 'products',
});
```

#### Advanced Example with REST Resources
```typescript
import { restResources } from '@shopify/shopify-api/rest/admin/2023-07';

const shopify = shopifyApi({
  // ... other config
  restResources,
});

// Using REST resources
const products = await shopify.rest.Product.all({
  session,
  limit: 50,
  status: 'active',
});
``` 

### GraphQL API Client

#### Minimal Example
```typescript
const client = new shopify.clients.Graphql({ session });
const response = await client.request(`
  query getProducts($first: Int!) {
    products(first: $first) {
      edges {
        node {
          id
          title
        }
      }
    }
  }
`, {
  variables: { first: 10 }
});
```  

#### Advanced Example with Types
```typescript
const response = await client.request(
  `#graphql
  query productHandles($first: Int!) {
    products(first: $first) {
      edges {
        node {
          handle
        }
      }
    }
  }`,
  {
    variables: {
      first: 10,
    },
  },
);
``` 

### Webhooks

#### Minimal Example - Registration
```typescript
await shopify.webhooks.register({
  session,
});
```

#### Advanced Example - Processing
```typescript
const handleWebhookRequest = async (
  topic: string,
  shop: string,
  webhookRequestBody: string,
  webhookId: string,
  apiVersion: string,
) => {
  // Process webhook event
};

shopify.webhooks.addHandlers({
  PRODUCTS_CREATE: [
    {
      deliveryMethod: DeliveryMethod.Http,
      callbackUrl: '/webhooks',
      callback: handleWebhookRequest,
    },
  ],
});
``` 

### Billing

#### Minimal Example - Check Billing
```typescript
const billingCheck = await shopify.billing.check({
  session,
  plans: ['Basic Plan'],
  isTest: true,
});
```

#### Advanced Example - Request Payment
```typescript
const billingResponse = await shopify.billing.request({
  session,
  plan: 'Premium Plan',
  isTest: true,
  returnUrl: 'https://myapp.com/billing/callback',
});
``` 

## 5. Advanced Features

### Error Handling

```typescript
import { 
  ShopifyError, 
  HttpResponseError, 
  BillingError 
} from '@shopify/shopify-api';

try {
  const response = await client.request(query);
} catch (error) {
  if (error instanceof HttpResponseError) {
    console.log('HTTP Error:', error.response.status);
  } else if (error instanceof BillingError) {
    console.log('Billing Error:', error.message);
  }
}
```

### Retries and Timeouts

```typescript
const client = new shopify.clients.Graphql({
  session,
  retries: 3,
});

const response = await client.request(query, {
  retries: 2,
});
```  

### Logging and Debugging

```typescript
const shopify = shopifyApi({
  // ... other config
  logger: {
    level: LogSeverity.Debug,
    timestamps: true,
    httpRequests: true,
    log: async (severity, message) => {
      console.log(`[${severity}] ${message}`);
    },
  },
});
```

### Pagination

```typescript
let pageInfo;
do {
  const response = await shopify.rest.Product.all({
    ...pageInfo?.nextPage?.query,
    session,
    limit: 10,
  });

  const pageProducts = response.data;
  pageInfo = response.pageInfo;
} while (pageInfo?.nextPage);
``` 

## 6. TypeScript Usage

### Type Imports

```typescript
import {
  Session,
  ApiVersion,
  BillingInterval,
  DeliveryMethod,
  LogSeverity,
} from '@shopify/shopify-api';
```

### Type-Safe GraphQL

```typescript
// Install codegen preset
// pnpm add --save-dev @shopify/api-codegen-preset

// After running graphql-codegen, queries are automatically typed
const response = await client.request(
  `#graphql
  query getProduct($id: ID!) {
    product(id: $id) {
      id
      title
      handle
    }
  }`,
  {
    variables: { id: "gid://shopify/Product/123" }
  }
);

// response.data.product is now fully typed
```

### Session Types

```typescript
const session: Session = {
  id: 'session-id',
  shop: 'my-shop.myshopify.com',
  state: 'state-value',
  isOnline: true,
  accessToken: 'access-token',
  scope: 'read_products,write_products',
};
```

## 7. Best Practices

### Session Management

```typescript
// Always validate sessions before API calls
const sessionId = await shopify.session.getCurrentId({
  isOnline: true,
  rawRequest: req,
  rawResponse: res,
});

const session = await getSessionFromStorage(sessionId);
if (!session) {
  // Redirect to OAuth
  return;
}
``` 

### Rate Limit Handling

```typescript
// Use built-in retry logic
const client = new shopify.clients.Graphql({
  session,
  retries: 3, // Will retry on rate limit errors
});
```

### Custom Store Apps

```typescript
// For custom store apps, create sessions manually
const session = shopify.session.customAppSession("my-shop.myshopify.com");

const productCount = await shopify.rest.Product.count({ session });
``` 

## 8. Production Checklist

### Pin SDK Version

```json
{
  "dependencies": {
    "@shopify/shopify-api": "^12.0.0"
  }
}
``` 

### Robust Error Handling

```typescript
try {
  const response = await client.request(query);
  return response.data;
} catch (error) {
  if (error instanceof HttpResponseError) {
    // Log and handle HTTP errors
    logger.error('API Error:', error.response.status);
    throw new Error('API request failed');
  }
  throw error;
}
```

### Environment Configuration

```typescript
const shopify = shopifyApi({
  apiKey: process.env.SHOPIFY_API_KEY!,
  apiSecretKey: process.env.SHOPIFY_API_SECRET!,
  scopes: process.env.SHOPIFY_SCOPES!.split(','),
  hostName: process.env.SHOPIFY_APP_URL!,
  apiVersion: ApiVersion.July25, // Use stable versions
  logger: {
    level: process.env.NODE_ENV === 'production' 
      ? LogSeverity.Error 
      : LogSeverity.Debug,
  },
});
```

### Webhook Security

```typescript
// Always validate webhook HMAC
await shopify.webhooks.process({
  rawBody: req.body,
  rawRequest: req,
  rawResponse: res,
});
```

## Notes

This guide covers the `@shopify/shopify-api` library which provides comprehensive TypeScript/JavaScript support for Shopify's Admin API, including OAuth flows, REST/GraphQL clients, webhook processing, and billing functionality.  The library uses a factory pattern centered around `shopifyApi()` and requires runtime adapters for cross-platform compatibility. 