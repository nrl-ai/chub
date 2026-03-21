---
name: payments
description: "Square SDK coding guide for payments, POS, and commerce checkout"
metadata:
  languages: "javascript"
  versions: "43.2.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "square,payments,pos,commerce,checkout"
---

# Square SDK Coding Guide

## 1. Golden Rule

**Always use the official Square SDK package:**
- Package name: `square` (Node.js/TypeScript library for Square API)
- Official repository: https://github.com/square/square-nodejs-sdk

**Never use deprecated libraries:**
- `square-connect` (DEPRECATED - replaced by `square`)
- `connect-nodejs-sdk` (DEPRECATED - replaced by `square`)

**Current SDK Version:** v43.2.0

**API Versioning:** Square uses date-based API versioning (e.g., 2025-08-20). Your account is automatically pinned to an API version. You can override this when initializing the client or in the Square Dashboard.

## 2. Installation

```bash
npm install square
```

```bash
yarn add square
```

```bash
pnpm add square
```

**Requirements:** Node.js 16+ (Node.js 18+ recommended for production)

### Environment Variables

```bash
# Required
SQUARE_ACCESS_TOKEN=EAAAl...  # Your Square access token
SQUARE_ENVIRONMENT=sandbox    # or "production"

# Optional
SQUARE_LOCATION_ID=L88917...  # Default location ID for operations
```

**CRITICAL:** Never commit access tokens to version control. Use environment variables or secure secret management systems. Use sandbox tokens for development and testing.

## 3. Initialization

### Basic Initialization

```javascript
const { SquareClient, SquareEnvironment } = require("square");

const client = new SquareClient({
  token: process.env.SQUARE_ACCESS_TOKEN,
  environment: SquareEnvironment.Sandbox,
});
```

### TypeScript Initialization

```typescript
import { SquareClient, SquareEnvironment, SquareError } from "square";

const client = new SquareClient({
  token: process.env.SQUARE_ACCESS_TOKEN!,
  environment: SquareEnvironment.Production,
});
```

### Advanced Configuration

```javascript
const { SquareClient, SquareEnvironment } = require("square");

const client = new SquareClient({
  token: process.env.SQUARE_ACCESS_TOKEN,
  environment: SquareEnvironment.Production,
  timeout: 60000,              // 60 second timeout
  squareVersion: "2025-08-20", // Override API version
  additionalHeaders: {
    "User-Agent": "MyApp/1.0.0",
  },
});
```

### ES Module Import

```javascript
import { SquareClient, SquareEnvironment } from "square";

const client = new SquareClient({
  token: process.env.SQUARE_ACCESS_TOKEN,
  environment: SquareEnvironment.Sandbox,
});
```

### Error Handling Setup

```javascript
const { SquareClient, SquareError } = require("square");

async function makeSquareRequest() {
  try {
    // Make API calls here
  } catch (error) {
    if (error instanceof SquareError) {
      // Handle Square-specific errors
      error.errors.forEach(e => {
        console.log(`Category: ${e.category}`);
        console.log(`Code: ${e.code}`);
        console.log(`Detail: ${e.detail}`);
      });
    } else {
      console.log("Unexpected error:", error);
    }
  }
}
```

## 4. Core API Surfaces

### Locations API

Get information about business locations.

**List All Locations:**

```javascript
async function listLocations() {
  try {
    const response = await client.locations.list();
    const locations = response.locations;

    locations.forEach(location => {
      console.log(`${location.id}: ${location.name}`);
      console.log(`Address: ${location.address.addressLine1}`);
      console.log(`City: ${location.address.locality}`);
    });

    return locations;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error listing locations:", error.errors);
    }
    throw error;
  }
}
```

**Retrieve Single Location:**

```javascript
async function getLocation(locationId) {
  try {
    const response = await client.locations.retrieve(locationId);
    return response.location;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving location:", error.errors);
    }
    throw error;
  }
}
```

### Payments API

Process and manage payments.

**Create Payment (Minimal):**

```javascript
import { randomUUID } from 'crypto';

async function createPayment(sourceId, amountMoney, locationId) {
  try {
    const response = await client.payments.create({
      sourceId: sourceId,
      idempotencyKey: randomUUID(),
      amountMoney: {
        amount: BigInt(amountMoney),
        currency: "USD",
      },
      locationId: locationId,
    });

    return response.payment;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Payment error:", error.errors);
    }
    throw error;
  }
}
```

**Create Payment (Advanced):**

```javascript
import { randomUUID } from 'crypto';

async function createAdvancedPayment(paymentData) {
  try {
    const response = await client.payments.create({
      sourceId: paymentData.sourceId,
      idempotencyKey: randomUUID(),
      amountMoney: {
        amount: BigInt(paymentData.amount),
        currency: paymentData.currency || "USD",
      },
      locationId: paymentData.locationId,
      customerId: paymentData.customerId,
      referenceId: paymentData.referenceId,
      note: paymentData.note,
      autocomplete: true,
      buyerEmailAddress: paymentData.email,
      billingAddress: {
        addressLine1: paymentData.addressLine1,
        locality: paymentData.city,
        administrativeDistrictLevel1: paymentData.state,
        postalCode: paymentData.postalCode,
        country: paymentData.country || "US",
      },
    });

    return response.payment;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Payment creation failed:", error.errors);
    }
    throw error;
  }
}
```

**List Payments:**

```javascript
async function listPayments(locationId, beginTime, endTime) {
  try {
    const response = await client.payments.list({
      locationId: locationId,
      beginTime: beginTime,
      endTime: endTime,
      sortOrder: "DESC",
      limit: 100,
    });

    return response.payments;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error listing payments:", error.errors);
    }
    throw error;
  }
}
```

**Get Payment:**

```javascript
async function getPayment(paymentId) {
  try {
    const response = await client.payments.retrieve(paymentId);
    return response.payment;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving payment:", error.errors);
    }
    throw error;
  }
}
```

**Complete Payment (for delayed capture):**

```javascript
async function completePayment(paymentId) {
  try {
    const response = await client.payments.complete(paymentId);
    return response.payment;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error completing payment:", error.errors);
    }
    throw error;
  }
}
```

**Cancel Payment:**

```javascript
async function cancelPayment(paymentId) {
  try {
    const response = await client.payments.cancel(paymentId);
    return response.payment;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error canceling payment:", error.errors);
    }
    throw error;
  }
}
```

### Refunds API

Manage payment refunds.

**Create Refund (Minimal):**

```javascript
import { randomUUID } from 'crypto';

async function createRefund(paymentId, amountMoney, currency) {
  try {
    const response = await client.refunds.create({
      idempotencyKey: randomUUID(),
      paymentId: paymentId,
      amountMoney: {
        amount: BigInt(amountMoney),
        currency: currency || "USD",
      },
    });

    return response.refund;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Refund error:", error.errors);
    }
    throw error;
  }
}
```

**Create Refund (Advanced):**

```javascript
import { randomUUID } from 'crypto';

async function createAdvancedRefund(refundData) {
  try {
    const response = await client.refunds.create({
      idempotencyKey: randomUUID(),
      paymentId: refundData.paymentId,
      amountMoney: {
        amount: BigInt(refundData.amount),
        currency: refundData.currency || "USD",
      },
      reason: refundData.reason,
      locationId: refundData.locationId,
    });

    return response.refund;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Refund creation failed:", error.errors);
    }
    throw error;
  }
}
```

**Get Refund:**

```javascript
async function getRefund(refundId) {
  try {
    const response = await client.refunds.retrieve(refundId);
    return response.refund;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving refund:", error.errors);
    }
    throw error;
  }
}
```

**List Refunds:**

```javascript
async function listRefunds(locationId, beginTime, endTime) {
  try {
    const response = await client.refunds.list({
      locationId: locationId,
      beginTime: beginTime,
      endTime: endTime,
      sortOrder: "DESC",
    });

    return response.refunds;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error listing refunds:", error.errors);
    }
    throw error;
  }
}
```

### Orders API

Create and manage orders.

**Create Order (Minimal):**

```javascript
import { randomUUID } from 'crypto';

async function createOrder(locationId) {
  try {
    const response = await client.orders.create({
      idempotencyKey: randomUUID(),
      order: {
        locationId: locationId,
        lineItems: [
          {
            name: "Item Name",
            quantity: "1",
            basePriceMoney: {
              amount: BigInt(1000),
              currency: "USD",
            },
          },
        ],
      },
    });

    return response.order;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Order creation error:", error.errors);
    }
    throw error;
  }
}
```

**Create Order (Advanced):**

```javascript
import { randomUUID } from 'crypto';

async function createAdvancedOrder(orderData) {
  try {
    const response = await client.orders.create({
      idempotencyKey: randomUUID(),
      order: {
        locationId: orderData.locationId,
        referenceId: orderData.referenceId,
        customerId: orderData.customerId,
        lineItems: orderData.lineItems.map(item => ({
          name: item.name,
          quantity: item.quantity.toString(),
          basePriceMoney: {
            amount: BigInt(item.price),
            currency: item.currency || "USD",
          },
          note: item.note,
        })),
        taxes: [
          {
            name: "Sales Tax",
            percentage: "8.5",
            scope: "ORDER",
          },
        ],
        discounts: orderData.discounts?.map(discount => ({
          name: discount.name,
          percentage: discount.percentage,
          scope: "ORDER",
        })) || [],
      },
    });

    return response.order;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Order creation failed:", error.errors);
    }
    throw error;
  }
}
```

**Retrieve Order:**

```javascript
async function retrieveOrder(orderId) {
  try {
    const response = await client.orders.retrieve(orderId);
    return response.order;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving order:", error.errors);
    }
    throw error;
  }
}
```

**Update Order:**

```javascript
async function updateOrder(orderId, updates) {
  try {
    const response = await client.orders.update(orderId, {
      order: updates,
    });

    return response.order;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error updating order:", error.errors);
    }
    throw error;
  }
}
```

**Search Orders:**

```javascript
async function searchOrders(locationIds, query) {
  try {
    const response = await client.orders.search({
      locationIds: locationIds,
      query: {
        filter: {
          stateFilter: {
            states: ["OPEN", "COMPLETED"],
          },
          dateTimeFilter: query.dateTimeFilter,
          customerFilter: query.customerId ? {
            customerIds: [query.customerId],
          } : undefined,
        },
        sort: {
          sortField: "CREATED_AT",
          sortOrder: "DESC",
        },
      },
      limit: query.limit || 100,
    });

    return response.orders;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error searching orders:", error.errors);
    }
    throw error;
  }
}
```

### Customers API

Manage customer profiles.

**Create Customer (Minimal):**

```javascript
import { randomUUID } from 'crypto';

async function createCustomer(emailAddress, givenName, familyName) {
  try {
    const response = await client.customers.create({
      idempotencyKey: randomUUID(),
      emailAddress: emailAddress,
      givenName: givenName,
      familyName: familyName,
    });

    return response.customer;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Customer creation error:", error.errors);
    }
    throw error;
  }
}
```

**Create Customer (Advanced):**

```javascript
import { randomUUID } from 'crypto';

async function createAdvancedCustomer(customerData) {
  try {
    const response = await client.customers.create({
      idempotencyKey: randomUUID(),
      givenName: customerData.givenName,
      familyName: customerData.familyName,
      emailAddress: customerData.emailAddress,
      phoneNumber: customerData.phoneNumber,
      address: {
        addressLine1: customerData.addressLine1,
        addressLine2: customerData.addressLine2,
        locality: customerData.city,
        administrativeDistrictLevel1: customerData.state,
        postalCode: customerData.postalCode,
        country: customerData.country || "US",
      },
      referenceId: customerData.referenceId,
      note: customerData.note,
      birthday: customerData.birthday,
      companyName: customerData.companyName,
    });

    return response.customer;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Customer creation failed:", error.errors);
    }
    throw error;
  }
}
```

**List Customers:**

```javascript
async function listCustomers(cursor = null, limit = 100) {
  try {
    const response = await client.customers.list({
      cursor: cursor,
      limit: limit,
      sortField: "CREATED_AT",
      sortOrder: "DESC",
    });

    return {
      customers: response.customers,
      cursor: response.cursor,
    };
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error listing customers:", error.errors);
    }
    throw error;
  }
}
```

**Retrieve Customer:**

```javascript
async function retrieveCustomer(customerId) {
  try {
    const response = await client.customers.retrieve(customerId);
    return response.customer;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving customer:", error.errors);
    }
    throw error;
  }
}
```

**Update Customer:**

```javascript
async function updateCustomer(customerId, updates) {
  try {
    const response = await client.customers.update(customerId, updates);
    return response.customer;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error updating customer:", error.errors);
    }
    throw error;
  }
}
```

**Delete Customer:**

```javascript
async function deleteCustomer(customerId) {
  try {
    const response = await client.customers.delete(customerId);
    return response;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error deleting customer:", error.errors);
    }
    throw error;
  }
}
```

**Search Customers:**

```javascript
async function searchCustomers(query) {
  try {
    const response = await client.customers.search({
      query: {
        filter: {
          emailAddress: query.email ? {
            exact: query.email,
          } : undefined,
          phoneNumber: query.phone ? {
            exact: query.phone,
          } : undefined,
          createdAt: query.createdAt,
        },
      },
      limit: query.limit || 100,
    });

    return response.customers;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error searching customers:", error.errors);
    }
    throw error;
  }
}
```

### Catalog API

Manage items, categories, taxes, and modifiers.

**Create Catalog Item (Minimal):**

```javascript
import { randomUUID } from 'crypto';

async function createCatalogItem(name, price) {
  try {
    const response = await client.catalog.upsertCatalogObject({
      idempotencyKey: randomUUID(),
      object: {
        type: "ITEM",
        id: `#${name.replace(/\s/g, '_')}`,
        itemData: {
          name: name,
          variations: [
            {
              type: "ITEM_VARIATION",
              id: `#${name.replace(/\s/g, '_')}_variation`,
              itemVariationData: {
                name: "Regular",
                pricingType: "FIXED_PRICING",
                priceMoney: {
                  amount: BigInt(price),
                  currency: "USD",
                },
              },
            },
          ],
        },
      },
    });

    return response.catalogObject;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Catalog item creation error:", error.errors);
    }
    throw error;
  }
}
```

**Create Catalog Item (Advanced):**

```javascript
import { randomUUID } from 'crypto';

async function createAdvancedCatalogItem(itemData) {
  try {
    const response = await client.catalog.upsertCatalogObject({
      idempotencyKey: randomUUID(),
      object: {
        type: "ITEM",
        id: `#${itemData.name.replace(/\s/g, '_')}`,
        itemData: {
          name: itemData.name,
          description: itemData.description,
          categoryId: itemData.categoryId,
          taxIds: itemData.taxIds,
          variations: itemData.variations.map((variation, index) => ({
            type: "ITEM_VARIATION",
            id: `#${itemData.name.replace(/\s/g, '_')}_var_${index}`,
            itemVariationData: {
              name: variation.name,
              sku: variation.sku,
              pricingType: "FIXED_PRICING",
              priceMoney: {
                amount: BigInt(variation.price),
                currency: variation.currency || "USD",
              },
              trackInventory: variation.trackInventory || false,
            },
          })),
        },
      },
    });

    return response.catalogObject;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Catalog item creation failed:", error.errors);
    }
    throw error;
  }
}
```

**List Catalog:**

```javascript
async function listCatalog(types = ["ITEM"], cursor = null) {
  try {
    const response = await client.catalog.list({
      cursor: cursor,
      types: types,
    });

    return {
      objects: response.objects,
      cursor: response.cursor,
    };
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error listing catalog:", error.errors);
    }
    throw error;
  }
}
```

**Retrieve Catalog Object:**

```javascript
async function retrieveCatalogObject(objectId) {
  try {
    const response = await client.catalog.retrieve(objectId, {
      includeRelatedObjects: true,
    });

    return response.object;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving catalog object:", error.errors);
    }
    throw error;
  }
}
```

**Search Catalog Items:**

```javascript
async function searchCatalogItems(query) {
  try {
    const response = await client.catalog.searchCatalogItems({
      textFilter: query.text,
      categoryIds: query.categoryIds,
      stockLevels: query.stockLevels,
      enabledLocationIds: query.locationIds,
      limit: query.limit || 100,
    });

    return response.items;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error searching catalog:", error.errors);
    }
    throw error;
  }
}
```

**Delete Catalog Object:**

```javascript
async function deleteCatalogObject(objectId) {
  try {
    const response = await client.catalog.deleteCatalogObject(objectId);
    return response;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error deleting catalog object:", error.errors);
    }
    throw error;
  }
}
```

**Batch Upsert Catalog Objects:**

```javascript
import { randomUUID } from 'crypto';

async function batchUpsertCatalogObjects(objects) {
  try {
    const response = await client.catalog.batchUpsertCatalogObjects({
      idempotencyKey: randomUUID(),
      batches: [
        {
          objects: objects,
        },
      ],
    });

    return response.objects;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Batch upsert error:", error.errors);
    }
    throw error;
  }
}
```

### Inventory API

Track and manage inventory.

**Retrieve Inventory Count:**

```javascript
async function retrieveInventoryCount(catalogObjectId, locationIds) {
  try {
    const response = await client.inventory.retrieveInventoryCount(
      catalogObjectId,
      {
        locationIds: locationIds.join(','),
      }
    );

    return response.counts;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving inventory:", error.errors);
    }
    throw error;
  }
}
```

**Batch Retrieve Inventory Counts:**

```javascript
async function batchRetrieveInventoryCounts(catalogObjectIds, locationIds) {
  try {
    const response = await client.inventory.batchRetrieveInventoryCounts({
      catalogObjectIds: catalogObjectIds,
      locationIds: locationIds,
    });

    return response.counts;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving inventory counts:", error.errors);
    }
    throw error;
  }
}
```

**Adjust Inventory:**

```javascript
import { randomUUID } from 'crypto';

async function adjustInventory(catalogObjectId, locationId, adjustment) {
  try {
    const response = await client.inventory.batchChangeInventory({
      idempotencyKey: randomUUID(),
      changes: [
        {
          type: "ADJUSTMENT",
          adjustment: {
            catalogObjectId: catalogObjectId,
            locationId: locationId,
            quantity: adjustment.quantity.toString(),
            fromState: "IN_STOCK",
            toState: "IN_STOCK",
            occurredAt: new Date().toISOString(),
          },
        },
      ],
    });

    return response.counts;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error adjusting inventory:", error.errors);
    }
    throw error;
  }
}
```

### Invoices API

Create and manage invoices.

**Create Invoice (Minimal):**

```javascript
import { randomUUID } from 'crypto';

async function createInvoice(locationId, customerId, orderId) {
  try {
    const response = await client.invoices.create({
      invoice: {
        locationId: locationId,
        orderId: orderId,
        primaryRecipient: {
          customerId: customerId,
        },
        paymentRequests: [
          {
            requestType: "BALANCE",
            dueDate: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)
              .toISOString()
              .split('T')[0],
          },
        ],
      },
      idempotencyKey: randomUUID(),
    });

    return response.invoice;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Invoice creation error:", error.errors);
    }
    throw error;
  }
}
```

**Create Invoice (Advanced):**

```javascript
import { randomUUID } from 'crypto';

async function createAdvancedInvoice(invoiceData) {
  try {
    const response = await client.invoices.create({
      invoice: {
        locationId: invoiceData.locationId,
        orderId: invoiceData.orderId,
        primaryRecipient: {
          customerId: invoiceData.customerId,
          givenName: invoiceData.givenName,
          familyName: invoiceData.familyName,
          emailAddress: invoiceData.emailAddress,
        },
        paymentRequests: [
          {
            requestType: "BALANCE",
            dueDate: invoiceData.dueDate,
            fixedAmountRequestedMoney: invoiceData.fixedAmount ? {
              amount: BigInt(invoiceData.fixedAmount),
              currency: invoiceData.currency || "USD",
            } : undefined,
            reminders: [
              {
                relativeScheduledDays: -1,
                message: "Payment reminder",
              },
            ],
          },
        ],
        deliveryMethod: "EMAIL",
        invoiceNumber: invoiceData.invoiceNumber,
        title: invoiceData.title,
        description: invoiceData.description,
      },
      idempotencyKey: randomUUID(),
    });

    return response.invoice;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Invoice creation failed:", error.errors);
    }
    throw error;
  }
}
```

**Publish Invoice:**

```javascript
async function publishInvoice(invoiceId) {
  try {
    const response = await client.invoices.publish(invoiceId, {
      version: 0,
    });

    return response.invoice;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error publishing invoice:", error.errors);
    }
    throw error;
  }
}
```

**Get Invoice:**

```javascript
async function getInvoice(invoiceId) {
  try {
    const response = await client.invoices.retrieve(invoiceId);
    return response.invoice;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving invoice:", error.errors);
    }
    throw error;
  }
}
```

**Search Invoices:**

```javascript
async function searchInvoices(locationIds, query) {
  try {
    const response = await client.invoices.search({
      query: {
        locationIds: locationIds,
        filter: {
          customerIds: query.customerIds,
          stateFilter: query.states,
        },
        sort: {
          field: "INVOICE_SORT_DATE",
          order: "DESC",
        },
      },
      limit: query.limit || 100,
    });

    return response.invoices;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error searching invoices:", error.errors);
    }
    throw error;
  }
}
```

### Subscriptions API

Manage recurring payments and subscriptions.

**Create Subscription (Minimal):**

```javascript
import { randomUUID } from 'crypto';

async function createSubscription(locationId, customerId, planId) {
  try {
    const response = await client.subscriptions.create({
      idempotencyKey: randomUUID(),
      locationId: locationId,
      planId: planId,
      customerId: customerId,
    });

    return response.subscription;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Subscription creation error:", error.errors);
    }
    throw error;
  }
}
```

**Create Subscription (Advanced):**

```javascript
import { randomUUID } from 'crypto';

async function createAdvancedSubscription(subscriptionData) {
  try {
    const response = await client.subscriptions.create({
      idempotencyKey: randomUUID(),
      locationId: subscriptionData.locationId,
      planId: subscriptionData.planId,
      customerId: subscriptionData.customerId,
      startDate: subscriptionData.startDate,
      taxPercentage: subscriptionData.taxPercentage,
      priceOverrideMoney: subscriptionData.priceOverride ? {
        amount: BigInt(subscriptionData.priceOverride),
        currency: subscriptionData.currency || "USD",
      } : undefined,
      cardId: subscriptionData.cardId,
    });

    return response.subscription;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Subscription creation failed:", error.errors);
    }
    throw error;
  }
}
```

**Retrieve Subscription:**

```javascript
async function retrieveSubscription(subscriptionId) {
  try {
    const response = await client.subscriptions.retrieve(subscriptionId, {
      include: "actions",
    });

    return response.subscription;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving subscription:", error.errors);
    }
    throw error;
  }
}
```

**Cancel Subscription:**

```javascript
async function cancelSubscription(subscriptionId) {
  try {
    const response = await client.subscriptions.cancel(subscriptionId);
    return response.subscription;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error canceling subscription:", error.errors);
    }
    throw error;
  }
}
```

**Search Subscriptions:**

```javascript
async function searchSubscriptions(query) {
  try {
    const response = await client.subscriptions.search({
      query: {
        filter: {
          customerIds: query.customerIds,
          locationIds: query.locationIds,
        },
      },
      limit: query.limit || 100,
    });

    return response.subscriptions;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error searching subscriptions:", error.errors);
    }
    throw error;
  }
}
```

### Checkout API

Create hosted checkout pages.

**Create Checkout (Minimal):**

```javascript
import { randomUUID } from 'crypto';

async function createCheckout(locationId, orderId) {
  try {
    const response = await client.checkout.create(locationId, {
      idempotencyKey: randomUUID(),
      order: {
        orderId: orderId,
      },
    });

    return response.checkout.checkoutPageUrl;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Checkout creation error:", error.errors);
    }
    throw error;
  }
}
```

**Create Checkout (Advanced):**

```javascript
import { randomUUID } from 'crypto';

async function createAdvancedCheckout(locationId, checkoutData) {
  try {
    const response = await client.checkout.create(locationId, {
      idempotencyKey: randomUUID(),
      order: {
        orderId: checkoutData.orderId,
        locationId: locationId,
      },
      askForShippingAddress: checkoutData.askForShippingAddress || false,
      merchantSupportEmail: checkoutData.merchantEmail,
      prePopulateBuyerEmail: checkoutData.buyerEmail,
      redirectUrl: checkoutData.redirectUrl,
      additionalRecipients: checkoutData.additionalRecipients,
    });

    return response.checkout;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Checkout creation failed:", error.errors);
    }
    throw error;
  }
}
```

### Bookings API

Manage appointments and bookings.

**Create Booking (Minimal):**

```javascript
import { randomUUID } from 'crypto';

async function createBooking(locationId, customerId, startAt, serviceVariationId, teamMemberId) {
  try {
    const response = await client.bookings.create({
      idempotencyKey: randomUUID(),
      booking: {
        locationId: locationId,
        customerId: customerId,
        startAt: startAt,
        appointmentSegments: [
          {
            durationMinutes: 60,
            serviceVariationId: serviceVariationId,
            teamMemberId: teamMemberId,
          },
        ],
      },
    });

    return response.booking;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Booking creation error:", error.errors);
    }
    throw error;
  }
}
```

**Create Booking (Advanced):**

```javascript
import { randomUUID } from 'crypto';

async function createAdvancedBooking(bookingData) {
  try {
    const response = await client.bookings.create({
      idempotencyKey: randomUUID(),
      booking: {
        locationId: bookingData.locationId,
        customerId: bookingData.customerId,
        customerNote: bookingData.customerNote,
        sellerNote: bookingData.sellerNote,
        startAt: bookingData.startAt,
        appointmentSegments: bookingData.appointmentSegments.map(segment => ({
          durationMinutes: segment.durationMinutes,
          serviceVariationId: segment.serviceVariationId,
          teamMemberId: segment.teamMemberId,
        })),
      },
    });

    return response.booking;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Booking creation failed:", error.errors);
    }
    throw error;
  }
}
```

**List Bookings:**

```javascript
async function listBookings(locationId, startAtMin, startAtMax) {
  try {
    const response = await client.bookings.list({
      locationId: locationId,
      startAtMin: startAtMin,
      startAtMax: startAtMax,
      limit: 100,
    });

    return response.bookings;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error listing bookings:", error.errors);
    }
    throw error;
  }
}
```

**Retrieve Booking:**

```javascript
async function retrieveBooking(bookingId) {
  try {
    const response = await client.bookings.retrieve(bookingId);
    return response.booking;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving booking:", error.errors);
    }
    throw error;
  }
}
```

**Cancel Booking:**

```javascript
async function cancelBooking(bookingId) {
  try {
    const response = await client.bookings.cancel(bookingId, {
      idempotencyKey: randomUUID(),
    });

    return response.booking;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error canceling booking:", error.errors);
    }
    throw error;
  }
}
```

### Terminal API

Create checkouts for Square Terminal devices.

**Create Terminal Checkout:**

```javascript
import { randomUUID } from 'crypto';

async function createTerminalCheckout(deviceId, amountMoney, locationId) {
  try {
    const response = await client.terminal.createTerminalCheckout({
      idempotencyKey: randomUUID(),
      checkout: {
        amountMoney: {
          amount: BigInt(amountMoney),
          currency: "USD",
        },
        deviceOptions: {
          deviceId: deviceId,
        },
      },
    });

    return response.checkout;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Terminal checkout error:", error.errors);
    }
    throw error;
  }
}
```

**Get Terminal Checkout:**

```javascript
async function getTerminalCheckout(checkoutId) {
  try {
    const response = await client.terminal.getTerminalCheckout(checkoutId);
    return response.checkout;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error retrieving terminal checkout:", error.errors);
    }
    throw error;
  }
}
```

**Cancel Terminal Checkout:**

```javascript
async function cancelTerminalCheckout(checkoutId) {
  try {
    const response = await client.terminal.cancelTerminalCheckout(checkoutId);
    return response.checkout;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error canceling terminal checkout:", error.errors);
    }
    throw error;
  }
}
```

### Webhooks

Handle webhook notifications from Square.

**Create Webhook Subscription:**

```javascript
async function createWebhookSubscription(notificationUrl, eventTypes) {
  try {
    const response = await client.webhookSubscriptions.create({
      subscription: {
        name: "My Webhook",
        notificationUrl: notificationUrl,
        eventTypes: eventTypes,
      },
    });

    return response.subscription;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Webhook creation error:", error.errors);
    }
    throw error;
  }
}
```

**List Webhook Subscriptions:**

```javascript
async function listWebhookSubscriptions() {
  try {
    const response = await client.webhookSubscriptions.list();
    return response.subscriptions;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Error listing webhooks:", error.errors);
    }
    throw error;
  }
}
```

**Verify Webhook Signature:**

```javascript
const crypto = require('crypto');

function verifyWebhookSignature(body, signature, signatureKey, notificationUrl) {
  const hmac = crypto.createHmac('sha256', signatureKey);
  hmac.update(notificationUrl + body);
  const hash = hmac.digest('base64');

  return hash === signature;
}
```

**Handle Webhook (Express):**

```javascript
const express = require('express');
const app = express();

app.post('/webhooks/square', express.raw({ type: 'application/json' }), (req, res) => {
  const signature = req.headers['x-square-hmacsha256-signature'];
  const body = req.body.toString('utf8');

  const isValid = verifyWebhookSignature(
    body,
    signature,
    process.env.SQUARE_WEBHOOK_SECRET,
    'https://yourdomain.com/webhooks/square'
  );

  if (!isValid) {
    return res.status(400).send('Invalid signature');
  }

  const event = JSON.parse(body);

  switch (event.type) {
    case 'payment.created':
      console.log('Payment created:', event.data.object.payment);
      break;
    case 'order.created':
      console.log('Order created:', event.data.object.order);
      break;
    case 'customer.created':
      console.log('Customer created:', event.data.object.customer);
      break;
    default:
      console.log('Unhandled event type:', event.type);
  }

  res.status(200).send('OK');
});
```

### Common Webhook Events

```javascript
// Payment events
"payment.created"
"payment.updated"

// Order events
"order.created"
"order.updated"
"order.fulfillment.updated"

// Customer events
"customer.created"
"customer.updated"
"customer.deleted"

// Invoice events
"invoice.created"
"invoice.published"
"invoice.payment_made"

// Subscription events
"subscription.created"
"subscription.started"
"subscription.canceled"

// Booking events
"booking.created"
"booking.updated"

// Inventory events
"inventory.count.updated"

// Catalog events
"catalog.version.updated"
```

### Pagination

Handle paginated results with cursor-based pagination.

```javascript
async function getAllCustomers() {
  const allCustomers = [];
  let cursor = null;

  do {
    const response = await client.customers.list({
      cursor: cursor,
      limit: 100,
    });

    if (response.customers) {
      allCustomers.push(...response.customers);
    }

    cursor = response.cursor;
  } while (cursor);

  return allCustomers;
}
```

### Idempotency

Use idempotency keys to prevent duplicate operations.

```javascript
import { randomUUID } from 'crypto';

async function idempotentPayment(sourceId, amount, locationId) {
  const idempotencyKey = randomUUID();

  try {
    const response = await client.payments.create({
      sourceId: sourceId,
      idempotencyKey: idempotencyKey,
      amountMoney: {
        amount: BigInt(amount),
        currency: "USD",
      },
      locationId: locationId,
    });

    return response.payment;
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Payment failed:", error.errors);
    }
    throw error;
  }
}
```

### Error Categories

```javascript
// Common Square error categories:
// - API_ERROR: API-level error
// - AUTHENTICATION_ERROR: Authentication failure
// - INVALID_REQUEST_ERROR: Invalid request parameters
// - RATE_LIMIT_ERROR: Too many requests
// - PAYMENT_METHOD_ERROR: Payment method issue
// - REFUND_ERROR: Refund processing error
```

### Testing with Sandbox

```javascript
const { SquareClient, SquareEnvironment } = require("square");

// Always use sandbox for development
const client = new SquareClient({
  token: process.env.SQUARE_SANDBOX_TOKEN,
  environment: SquareEnvironment.Sandbox,
});

// Test payment source IDs for sandbox:
// cnon:card-nonce-ok - successful charge
// cnon:card-nonce-declined - declined charge
// cnon:card-nonce-dishonoured - card verification failed
```

### Complete Example: Process Payment with Customer

```javascript
const { SquareClient, SquareEnvironment, SquareError } = require("square");
const { randomUUID } = require('crypto');

const client = new SquareClient({
  token: process.env.SQUARE_ACCESS_TOKEN,
  environment: SquareEnvironment.Sandbox,
});

async function processPaymentWithCustomer(paymentDetails) {
  try {
    // 1. Create or retrieve customer
    let customer;
    try {
      const customerResponse = await client.customers.create({
        idempotencyKey: randomUUID(),
        givenName: paymentDetails.givenName,
        familyName: paymentDetails.familyName,
        emailAddress: paymentDetails.emailAddress,
      });
      customer = customerResponse.customer;
    } catch (error) {
      console.log("Customer already exists, using existing");
    }

    // 2. Create order
    const orderResponse = await client.orders.create({
      idempotencyKey: randomUUID(),
      order: {
        locationId: paymentDetails.locationId,
        customerId: customer.id,
        lineItems: [
          {
            name: paymentDetails.itemName,
            quantity: "1",
            basePriceMoney: {
              amount: BigInt(paymentDetails.amount),
              currency: "USD",
            },
          },
        ],
      },
    });

    // 3. Create payment
    const paymentResponse = await client.payments.create({
      sourceId: paymentDetails.sourceId,
      idempotencyKey: randomUUID(),
      amountMoney: {
        amount: BigInt(paymentDetails.amount),
        currency: "USD",
      },
      orderId: orderResponse.order.id,
      customerId: customer.id,
      locationId: paymentDetails.locationId,
      autocomplete: true,
    });

    return {
      payment: paymentResponse.payment,
      order: orderResponse.order,
      customer: customer,
    };
  } catch (error) {
    if (error instanceof SquareError) {
      console.error("Transaction failed:");
      error.errors.forEach(e => {
        console.log(`  ${e.category}: ${e.code}`);
        console.log(`  ${e.detail}`);
      });
    }
    throw error;
  }
}

// Usage
processPaymentWithCustomer({
  sourceId: "cnon:card-nonce-ok",
  amount: 1000,
  locationId: "L88917AVBK2S5",
  givenName: "John",
  familyName: "Doe",
  emailAddress: "john.doe@example.com",
  itemName: "Premium Service",
}).then(result => {
  console.log("Payment successful!");
  console.log("Payment ID:", result.payment.id);
  console.log("Order ID:", result.order.id);
  console.log("Customer ID:", result.customer.id);
});
```
