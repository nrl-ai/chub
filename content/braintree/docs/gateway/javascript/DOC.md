---
name: gateway
description: "Braintree Node.js SDK for payment gateway, PayPal, and subscriptions"
metadata:
  languages: "javascript"
  versions: "3.34.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "braintree,gateway,payments,paypal,subscriptions"
---

# Braintree Node.js SDK Context

## Golden Rule

**ALWAYS use the official Braintree Node.js SDK package: `braintree`**

```bash
npm install braintree
```

**DO NOT use:**
- Deprecated or unofficial packages
- Direct API calls without the SDK
- Outdated versions (version 3.40.0+ required for SSL certificate support)

The official package is `braintree` and is maintained by Braintree (a PayPal service).

## Installation

```bash
npm install braintree
```

For Express.js applications, you'll also need:

```bash
npm install express body-parser
```

## Environment Configuration

Create a `.env` file with your Braintree credentials:

```bash
BRAINTREE_ENVIRONMENT=Sandbox
BRAINTREE_MERCHANT_ID=your_merchant_id
BRAINTREE_PUBLIC_KEY=your_public_key
BRAINTREE_PRIVATE_KEY=your_private_key
```

Load environment variables:

```javascript
require('dotenv').config();
```

## Initialization

### Basic Gateway Setup

```javascript
const braintree = require("braintree");

const gateway = new braintree.BraintreeGateway({
  environment: braintree.Environment.Sandbox,
  merchantId: "your_merchant_id",
  publicKey: "your_public_key",
  privateKey: "your_private_key"
});
```

### Using Environment Variables

```javascript
const braintree = require("braintree");

const gateway = new braintree.BraintreeGateway({
  environment: process.env.BRAINTREE_ENVIRONMENT === 'Production'
    ? braintree.Environment.Production
    : braintree.Environment.Sandbox,
  merchantId: process.env.BRAINTREE_MERCHANT_ID,
  publicKey: process.env.BRAINTREE_PUBLIC_KEY,
  privateKey: process.env.BRAINTREE_PRIVATE_KEY
});
```

### Environment Options

```javascript
// Sandbox environment (for testing)
braintree.Environment.Sandbox

// Production environment (for live transactions)
braintree.Environment.Production
```

## Client Token Generation

Client tokens contain authorization and configuration information for the client SDK.

### Basic Client Token

**Callback:**
```javascript
gateway.clientToken.generate({}, (err, response) => {
  const clientToken = response.clientToken;
});
```

**Promise:**
```javascript
gateway.clientToken.generate({})
  .then(response => {
    const clientToken = response.clientToken;
  });
```

### Client Token with Customer ID

```javascript
gateway.clientToken.generate({
  customerId: "customer_123"
}, (err, response) => {
  const clientToken = response.clientToken;
});
```

### Express.js Endpoint for Client Token

```javascript
const express = require('express');
const app = express();

app.get("/client_token", (req, res) => {
  gateway.clientToken.generate({}, (err, response) => {
    if (err) {
      res.status(500).send(err);
    } else {
      res.send(response.clientToken);
    }
  });
});
```

## Transactions

### Creating a Transaction

**Basic Sale (Callback):**
```javascript
gateway.transaction.sale({
  amount: "10.00",
  paymentMethodNonce: nonceFromTheClient,
  options: {
    submitForSettlement: true
  }
}, (err, result) => {
  if (result.success) {
    console.log("Transaction ID: " + result.transaction.id);
  } else {
    console.log(result.message);
  }
});
```

**Basic Sale (Promise):**
```javascript
gateway.transaction.sale({
  amount: "10.00",
  paymentMethodNonce: nonceFromTheClient,
  options: {
    submitForSettlement: true
  }
}).then(result => {
  if (result.success) {
    console.log("Transaction ID: " + result.transaction.id);
  } else {
    console.log(result.message);
  }
});
```

### Transaction with Device Data

```javascript
gateway.transaction.sale({
  amount: "10.00",
  paymentMethodNonce: nonceFromTheClient,
  deviceData: deviceDataFromTheClient,
  options: {
    submitForSettlement: true
  }
}, (err, result) => {
  // Handle result
});
```

### Transaction with Customer Information

```javascript
gateway.transaction.sale({
  amount: "100.00",
  paymentMethodNonce: nonceFromTheClient,
  customer: {
    firstName: "John",
    lastName: "Doe",
    email: "john.doe@example.com",
    phone: "312-555-1234"
  },
  billing: {
    firstName: "John",
    lastName: "Doe",
    streetAddress: "123 Main St",
    locality: "Chicago",
    region: "IL",
    postalCode: "60614",
    countryCodeAlpha2: "US"
  },
  shipping: {
    firstName: "John",
    lastName: "Doe",
    streetAddress: "123 Main St",
    locality: "Chicago",
    region: "IL",
    postalCode: "60614",
    countryCodeAlpha2: "US"
  },
  options: {
    submitForSettlement: true
  }
}, (err, result) => {
  // Handle result
});
```

### Transaction with Stored Payment Method

```javascript
gateway.transaction.sale({
  amount: "10.00",
  paymentMethodToken: "the_token",
  options: {
    submitForSettlement: true
  }
}, (err, result) => {
  // Handle result
});
```

### Express.js Checkout Endpoint

```javascript
app.post("/checkout", (req, res) => {
  const nonceFromTheClient = req.body.payment_method_nonce;
  const amount = req.body.amount;

  gateway.transaction.sale({
    amount: amount,
    paymentMethodNonce: nonceFromTheClient,
    options: {
      submitForSettlement: true
    }
  }, (err, result) => {
    if (result.success) {
      res.send({
        success: true,
        transactionId: result.transaction.id
      });
    } else {
      res.status(500).send({
        success: false,
        message: result.message
      });
    }
  });
});
```

### Finding a Transaction

**Callback:**
```javascript
gateway.transaction.find("theTransactionId", (err, transaction) => {
  console.log(transaction.amount);
  console.log(transaction.status);
});
```

**Promise:**
```javascript
gateway.transaction.find("theTransactionId")
  .then(transaction => {
    console.log(transaction.amount);
    console.log(transaction.status);
  });
```

### Refunding a Transaction

**Full Refund (Callback):**
```javascript
gateway.transaction.refund("theTransactionId", (err, result) => {
  if (result.success) {
    console.log("Refund successful");
  }
});
```

**Partial Refund (Callback):**
```javascript
gateway.transaction.refund("theTransactionId", "10.00", (err, result) => {
  if (result.success) {
    console.log("Partial refund successful");
  }
});
```

**Promise:**
```javascript
gateway.transaction.refund("theTransactionId", "10.00")
  .then(result => {
    if (result.success) {
      console.log("Refund successful");
    }
  });
```

### Voiding a Transaction

**Callback:**
```javascript
gateway.transaction.void("theTransactionId", (err, result) => {
  if (result.success) {
    console.log("Transaction voided");
  } else {
    console.log(result.message);
  }
});
```

**Promise:**
```javascript
gateway.transaction.void("theTransactionId")
  .then(result => {
    if (result.success) {
      console.log("Transaction voided");
    }
  });
```

## Customer Management

### Creating a Customer

**Basic Customer (Callback):**
```javascript
gateway.customer.create({
  firstName: "John",
  lastName: "Doe",
  email: "john.doe@example.com",
  phone: "312-555-1234"
}, (err, result) => {
  if (result.success) {
    console.log("Customer ID: " + result.customer.id);
  }
});
```

**Promise:**
```javascript
gateway.customer.create({
  firstName: "John",
  lastName: "Doe",
  email: "john.doe@example.com"
}).then(result => {
  if (result.success) {
    console.log("Customer ID: " + result.customer.id);
  }
});
```

### Creating Customer with Company Information

```javascript
gateway.customer.create({
  firstName: "John",
  lastName: "Doe",
  company: "Acme Corp",
  email: "john.doe@example.com",
  phone: "312-555-1234",
  fax: "614-555-5678",
  website: "www.example.com"
}, (err, result) => {
  // Handle result
});
```

### Creating Customer with Payment Method

```javascript
gateway.customer.create({
  firstName: "John",
  lastName: "Doe",
  email: "john.doe@example.com",
  paymentMethodNonce: nonceFromTheClient
}, (err, result) => {
  if (result.success) {
    console.log("Customer ID: " + result.customer.id);
    console.log("Payment Method Token: " + result.customer.paymentMethods[0].token);
  }
});
```

### Finding a Customer

**Callback:**
```javascript
gateway.customer.find("theCustomerId", (err, customer) => {
  console.log(customer.firstName);
  console.log(customer.lastName);
  console.log(customer.email);
});
```

**Promise:**
```javascript
gateway.customer.find("theCustomerId")
  .then(customer => {
    console.log(customer.firstName);
    console.log(customer.paymentMethods);
  });
```

### Updating a Customer

**Callback:**
```javascript
gateway.customer.update("theCustomerId", {
  firstName: "Jane",
  lastName: "Smith",
  email: "jane.smith@example.com"
}, (err, result) => {
  if (result.success) {
    console.log("Customer updated");
  }
});
```

**Promise:**
```javascript
gateway.customer.update("theCustomerId", {
  firstName: "Jane",
  lastName: "Smith"
}).then(result => {
  if (result.success) {
    console.log("Customer updated");
  }
});
```

### Deleting a Customer

**Callback:**
```javascript
gateway.customer.delete("theCustomerId", (err) => {
  if (!err) {
    console.log("Customer deleted");
  }
});
```

**Promise:**
```javascript
gateway.customer.delete("theCustomerId")
  .then(() => {
    console.log("Customer deleted");
  });
```

### Searching for Customers

```javascript
const stream = gateway.customer.search((search) => {
  search.firstName().is("John");
  search.lastName().is("Doe");
});

stream.on("data", (customer) => {
  console.log(customer.id);
  console.log(customer.email);
});

stream.on("end", () => {
  console.log("Search complete");
});
```

## Payment Method Management

### Creating a Payment Method

**Callback:**
```javascript
gateway.paymentMethod.create({
  customerId: "customer_123",
  paymentMethodNonce: nonceFromTheClient
}, (err, result) => {
  if (result.success) {
    console.log("Payment Method Token: " + result.paymentMethod.token);
  }
});
```

**Promise:**
```javascript
gateway.paymentMethod.create({
  customerId: "customer_123",
  paymentMethodNonce: nonceFromTheClient
}).then(result => {
  if (result.success) {
    console.log("Payment Method Token: " + result.paymentMethod.token);
  }
});
```

### Creating Payment Method with Billing Address

```javascript
gateway.paymentMethod.create({
  customerId: "customer_123",
  paymentMethodNonce: nonceFromTheClient,
  billingAddress: {
    streetAddress: "123 Main St",
    locality: "Chicago",
    region: "IL",
    postalCode: "60614"
  }
}, (err, result) => {
  // Handle result
});
```

### Finding a Payment Method

**Callback:**
```javascript
gateway.paymentMethod.find("the_token", (err, paymentMethod) => {
  console.log(paymentMethod.token);
});
```

**Promise:**
```javascript
gateway.paymentMethod.find("the_token")
  .then(paymentMethod => {
    console.log(paymentMethod.token);
  });
```

### Updating a Payment Method

**Set as Default (Callback):**
```javascript
gateway.paymentMethod.update("the_token", {
  options: {
    makeDefault: true
  }
}, (err, result) => {
  if (result.success) {
    console.log("Payment method is now default");
  }
});
```

**Update Billing Address (Callback):**
```javascript
gateway.paymentMethod.update("the_token", {
  billingAddress: {
    streetAddress: "456 Oak Ave",
    locality: "Chicago",
    region: "IL",
    postalCode: "60614",
    options: {
      updateExisting: true
    }
  }
}, (err, result) => {
  // Handle result
});
```

**Promise:**
```javascript
gateway.paymentMethod.update("the_token", {
  options: {
    makeDefault: true
  }
}).then(result => {
  if (result.success) {
    console.log("Payment method updated");
  }
});
```

### Deleting a Payment Method

**Callback:**
```javascript
gateway.paymentMethod.delete("the_token", (err) => {
  if (!err) {
    console.log("Payment method deleted");
  }
});
```

**Promise:**
```javascript
gateway.paymentMethod.delete("the_token")
  .then(() => {
    console.log("Payment method deleted");
  })
  .catch(err => {
    console.log(err);
  });
```

## Subscription Billing

### Creating a Subscription

**Basic Subscription (Callback):**
```javascript
gateway.subscription.create({
  paymentMethodToken: "the_token",
  planId: "plan_id"
}, (err, result) => {
  if (result.success) {
    console.log("Subscription ID: " + result.subscription.id);
  }
});
```

**Promise:**
```javascript
gateway.subscription.create({
  paymentMethodToken: "the_token",
  planId: "plan_id"
}).then(result => {
  if (result.success) {
    console.log("Subscription ID: " + result.subscription.id);
  }
});
```

### Creating Subscription with Add-ons and Discounts

```javascript
gateway.subscription.create({
  paymentMethodToken: "the_token",
  planId: "plan_id",
  addOns: {
    add: [
      {
        inheritedFromId: "add_on_id_1",
        amount: "10.00"
      }
    ]
  },
  discounts: {
    add: [
      {
        inheritedFromId: "discount_id_1",
        amount: "5.00"
      }
    ]
  }
}, (err, result) => {
  // Handle result
});
```

### Creating Subscription with Trial Period

```javascript
gateway.subscription.create({
  paymentMethodToken: "the_token",
  planId: "plan_id",
  trialDuration: 14,
  trialDurationUnit: "day",
  trialPeriod: true
}, (err, result) => {
  // Handle result
});
```

### Finding a Subscription

**Callback:**
```javascript
gateway.subscription.find("subscription_id", (err, subscription) => {
  console.log(subscription.status);
  console.log(subscription.price);
});
```

**Promise:**
```javascript
gateway.subscription.find("subscription_id")
  .then(subscription => {
    console.log(subscription.status);
  });
```

### Updating a Subscription

**Callback:**
```javascript
gateway.subscription.update("subscription_id", {
  price: "15.00"
}, (err, result) => {
  if (result.success) {
    console.log("Subscription updated");
  }
});
```

**Promise:**
```javascript
gateway.subscription.update("subscription_id", {
  price: "15.00",
  planId: "new_plan_id"
}).then(result => {
  if (result.success) {
    console.log("Subscription updated");
  }
});
```

### Canceling a Subscription

**Callback:**
```javascript
gateway.subscription.cancel("subscription_id", (err, result) => {
  if (result.success) {
    console.log("Subscription canceled");
  }
});
```

**Promise:**
```javascript
gateway.subscription.cancel("subscription_id")
  .then(result => {
    if (result.success) {
      console.log("Subscription canceled");
    }
  });
```

### Retrieving All Plans

**Callback:**
```javascript
gateway.plan.all((err, result) => {
  result.plans.forEach(plan => {
    console.log(plan.id);
    console.log(plan.name);
    console.log(plan.price);
  });
});
```

**Promise:**
```javascript
gateway.plan.all()
  .then(result => {
    result.plans.forEach(plan => {
      console.log(plan.id);
    });
  });
```

## Webhooks

### Parsing Webhook Notifications

**Callback:**
```javascript
gateway.webhookNotification.parse(
  req.body.bt_signature,
  req.body.bt_payload,
  (err, webhookNotification) => {
    if (err) {
      console.error(err);
      return;
    }

    console.log(webhookNotification.kind);
    console.log(webhookNotification.timestamp);
  }
);
```

**Promise:**
```javascript
gateway.webhookNotification.parse(
  req.body.bt_signature,
  req.body.bt_payload
).then(webhookNotification => {
  console.log(webhookNotification.kind);
  console.log(webhookNotification.timestamp);
});
```

### Express.js Webhook Endpoint

```javascript
const express = require('express');
const bodyParser = require('body-parser');
const app = express();

app.use(bodyParser.urlencoded({ extended: false }));

app.post('/webhooks', (req, res) => {
  gateway.webhookNotification.parse(
    req.body.bt_signature,
    req.body.bt_payload,
    (err, webhookNotification) => {
      if (err) {
        console.error(err);
        return res.sendStatus(400);
      }

      // Handle different webhook types
      if (webhookNotification.kind === braintree.WebhookNotification.Kind.SubscriptionCanceled) {
        console.log("Subscription " + webhookNotification.subscription.id + " canceled");
      } else if (webhookNotification.kind === braintree.WebhookNotification.Kind.SubscriptionChargedSuccessfully) {
        console.log("Subscription " + webhookNotification.subscription.id + " charged");
      } else if (webhookNotification.kind === braintree.WebhookNotification.Kind.DisputeOpened) {
        console.log("Dispute opened for transaction " + webhookNotification.dispute.transaction.id);
      }

      res.sendStatus(200);
    }
  );
});
```

### Handling Subscription Webhooks

```javascript
gateway.webhookNotification.parse(
  signature,
  payload,
  (err, webhookNotification) => {
    if (webhookNotification.kind === braintree.WebhookNotification.Kind.SubscriptionWentPastDue) {
      const subscriptionId = webhookNotification.subscription.id;
      // Handle past due subscription
    }
  }
);
```

### Verifying Webhook Signatures

```javascript
gateway.webhookNotification.verify(
  "challenge_string",
  (err, response) => {
    console.log(response);
  }
);
```

### Testing Webhooks

```javascript
const sampleNotification = gateway.webhookTesting.sampleNotification(
  braintree.WebhookNotification.Kind.SubscriptionWentPastDue,
  "subscription_id"
);

gateway.webhookNotification.parse(
  sampleNotification.bt_signature,
  sampleNotification.bt_payload
).then(webhookNotification => {
  console.log(webhookNotification.kind);
  console.log(webhookNotification.subscription.id);
});
```

## Advanced Features

### Searching Transactions

```javascript
const stream = gateway.transaction.search((search) => {
  search.amount().min("10.00");
  search.amount().max("100.00");
  search.status().is(braintree.Transaction.Status.Settled);
  search.createdAt().min(new Date("2024-01-01"));
});

stream.on("data", (transaction) => {
  console.log(transaction.id);
  console.log(transaction.amount);
});

stream.on("end", () => {
  console.log("Search complete");
});
```

### Creating Customer with Multiple Payment Methods

```javascript
gateway.customer.create({
  firstName: "John",
  lastName: "Doe",
  email: "john.doe@example.com",
  creditCard: {
    number: "4111111111111111",
    expirationDate: "05/2025",
    cvv: "123",
    billingAddress: {
      postalCode: "60614"
    }
  }
}, (err, result) => {
  if (result.success) {
    const token = result.customer.paymentMethods[0].token;
    console.log("Payment method token: " + token);
  }
});
```

### Transaction with Custom Fields

```javascript
gateway.transaction.sale({
  amount: "10.00",
  paymentMethodNonce: nonceFromTheClient,
  customFields: {
    order_id: "12345",
    user_type: "premium"
  },
  options: {
    submitForSettlement: true
  }
}, (err, result) => {
  // Handle result
});
```

### Holding in Escrow

```javascript
gateway.transaction.sale({
  amount: "10.00",
  paymentMethodNonce: nonceFromTheClient,
  serviceFeeAmount: "1.00",
  merchantAccountId: "submerchant_account_id",
  options: {
    submitForSettlement: true,
    holdInEscrow: true
  }
}, (err, result) => {
  // Handle result
});
```

### Transaction with Descriptor

```javascript
gateway.transaction.sale({
  amount: "10.00",
  paymentMethodNonce: nonceFromTheClient,
  descriptor: {
    name: "Company*Product",
    phone: "8004567890",
    url: "example.com"
  },
  options: {
    submitForSettlement: true
  }
}, (err, result) => {
  // Handle result
});
```

### Cloning a Transaction

```javascript
gateway.transaction.cloneTransaction(
  "original_transaction_id",
  {
    amount: "20.00",
    options: {
      submitForSettlement: true
    }
  },
  (err, result) => {
    if (result.success) {
      console.log("Cloned transaction ID: " + result.transaction.id);
    }
  }
);
```

### Settlement

**Submit for Settlement:**
```javascript
gateway.transaction.submitForSettlement("transaction_id", (err, result) => {
  if (result.success) {
    console.log("Submitted for settlement");
  }
});
```

**Submit with Amount:**
```javascript
gateway.transaction.submitForSettlement("transaction_id", "50.00", (err, result) => {
  if (result.success) {
    console.log("Submitted for partial settlement");
  }
});
```

### Dispute Management

**Finding a Dispute:**
```javascript
gateway.dispute.find("dispute_id", (err, dispute) => {
  console.log(dispute.status);
  console.log(dispute.amount);
  console.log(dispute.reason);
});
```

**Accept Dispute:**
```javascript
gateway.dispute.accept("dispute_id", (err, result) => {
  // Handle result
});
```

**Add Text Evidence:**
```javascript
gateway.dispute.addTextEvidence("dispute_id", "evidence_text", (err, result) => {
  // Handle result
});
```

### Merchant Account Creation

```javascript
gateway.merchantAccount.create({
  individual: {
    firstName: "John",
    lastName: "Doe",
    email: "john.doe@example.com",
    dateOfBirth: "1980-01-01",
    ssn: "123-45-6789",
    address: {
      streetAddress: "123 Main St",
      locality: "Chicago",
      region: "IL",
      postalCode: "60614"
    }
  },
  business: {
    legalName: "John Doe Inc",
    dbaName: "JD Services",
    taxId: "12-3456789",
    address: {
      streetAddress: "123 Main St",
      locality: "Chicago",
      region: "IL",
      postalCode: "60614"
    }
  },
  funding: {
    destination: braintree.MerchantAccount.FundingDestination.Bank,
    routingNumber: "021000021",
    accountNumber: "1234567890"
  },
  tosAccepted: true,
  masterMerchantAccountId: "master_merchant_account_id"
}, (err, result) => {
  if (result.success) {
    console.log("Merchant Account ID: " + result.merchantAccount.id);
  }
});
```

## Error Handling

### Checking Result Success

```javascript
gateway.transaction.sale({
  amount: "10.00",
  paymentMethodNonce: nonceFromTheClient,
  options: {
    submitForSettlement: true
  }
}, (err, result) => {
  if (err) {
    console.error("Error:", err);
    return;
  }

  if (result.success) {
    console.log("Transaction ID:", result.transaction.id);
  } else {
    console.log("Error:", result.message);

    // Check validation errors
    if (result.errors) {
      console.log("Validation errors:");
      for (let key in result.errors.deepErrors()) {
        console.log(result.errors.deepErrors()[key]);
      }
    }
  }
});
```

### Handling Validation Errors

```javascript
gateway.customer.create({
  email: "invalid-email"
}, (err, result) => {
  if (!result.success) {
    result.errors.deepErrors().forEach((error) => {
      console.log("Error:", error.code);
      console.log("Message:", error.message);
      console.log("Attribute:", error.attribute);
    });
  }
});
```

### Promise Error Handling

```javascript
gateway.transaction.sale({
  amount: "10.00",
  paymentMethodNonce: nonceFromTheClient,
  options: {
    submitForSettlement: true
  }
})
.then(result => {
  if (result.success) {
    console.log("Transaction ID:", result.transaction.id);
  } else {
    console.log("Error:", result.message);
  }
})
.catch(err => {
  console.error("Unexpected error:", err);
});
```

## Common Patterns

### Complete Express.js Server Example

```javascript
const express = require('express');
const bodyParser = require('body-parser');
const braintree = require('braintree');

const app = express();
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: false }));

const gateway = new braintree.BraintreeGateway({
  environment: braintree.Environment.Sandbox,
  merchantId: process.env.BRAINTREE_MERCHANT_ID,
  publicKey: process.env.BRAINTREE_PUBLIC_KEY,
  privateKey: process.env.BRAINTREE_PRIVATE_KEY
});

// Generate client token
app.get('/client_token', (req, res) => {
  gateway.clientToken.generate({}, (err, response) => {
    if (err) {
      res.status(500).send(err);
    } else {
      res.send(response.clientToken);
    }
  });
});

// Process payment
app.post('/checkout', (req, res) => {
  const nonceFromTheClient = req.body.payment_method_nonce;
  const amount = req.body.amount;

  gateway.transaction.sale({
    amount: amount,
    paymentMethodNonce: nonceFromTheClient,
    options: {
      submitForSettlement: true
    }
  }, (err, result) => {
    if (result.success) {
      res.send({
        success: true,
        transactionId: result.transaction.id
      });
    } else {
      res.status(500).send({
        success: false,
        message: result.message
      });
    }
  });
});

app.listen(3000, () => {
  console.log('Server running on port 3000');
});
```

### Customer Creation with Payment Method Workflow

```javascript
// Step 1: Create customer
gateway.customer.create({
  firstName: "John",
  lastName: "Doe",
  email: "john.doe@example.com",
  paymentMethodNonce: nonceFromTheClient
}, (err, result) => {
  if (result.success) {
    const customerId = result.customer.id;
    const paymentToken = result.customer.paymentMethods[0].token;

    // Step 2: Create subscription
    gateway.subscription.create({
      paymentMethodToken: paymentToken,
      planId: "monthly_plan"
    }, (err, subResult) => {
      if (subResult.success) {
        console.log("Subscription ID:", subResult.subscription.id);
      }
    });
  }
});
```

### Stored Payment Method Workflow

```javascript
// Step 1: Get customer
gateway.customer.find("customer_id", (err, customer) => {
  // Step 2: Use default payment method for transaction
  const defaultPaymentMethod = customer.paymentMethods.find(pm => pm.default);

  gateway.transaction.sale({
    amount: "25.00",
    paymentMethodToken: defaultPaymentMethod.token,
    options: {
      submitForSettlement: true
    }
  }, (err, result) => {
    if (result.success) {
      console.log("Transaction ID:", result.transaction.id);
    }
  });
});
```
