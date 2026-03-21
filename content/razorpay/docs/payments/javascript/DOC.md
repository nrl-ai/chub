---
name: payments
description: "Razorpay Node.js SDK coding guidelines for building payment systems with orders, payments, refunds, and subscriptions"
metadata:
  languages: "javascript"
  versions: "2.9.6"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "razorpay,payments,india,checkout,upi"
---

# Razorpay Node.js Coding Guidelines

You are a Razorpay payment gateway coding expert. Help me with writing code using the Razorpay Node.js SDK for building payment systems with orders, payments, refunds, subscriptions, customers, and invoices.

Please follow the following guidelines when generating code.

You can find the official SDK documentation and code samples here:
https://razorpay.com/docs/payments/server-integration/nodejs/

## Golden Rule: Use the Correct and Current SDK

Always use the official Razorpay Node.js SDK, which is the standard library for all Razorpay API interactions.

**Library Name:** Razorpay Node.js SDK
**NPM Package:** `razorpay`
**Current Version:** 2.9.6
**Supported Node.js Versions:** Node.js 10+

**Installation:**
- **Correct:** `npm install razorpay` or `yarn add razorpay`

**APIs and Usage:**
- **Correct:** `const Razorpay = require('razorpay')`
- **Correct:** `const instance = new Razorpay({ key_id: 'YOUR_KEY_ID', key_secret: 'YOUR_KEY_SECRET' })`
- **Correct:** `instance.orders.create({...})` for creating orders
- **Correct:** `instance.payments.fetch(paymentId)` for fetching payments
- **Incorrect:** Using unofficial Razorpay libraries or wrappers
- **Incorrect:** Exposing credentials in front-end applications

## Authentication and Initialization

The Razorpay Node.js library requires your Key ID and Key Secret for authentication. You can obtain these from the Razorpay Dashboard at Settings > API Keys.

### Environment Variables (Recommended)

Set up environment variables for secure credential management:

```javascript
// Load environment variables
require('dotenv').config();

// Initialize Razorpay instance
const Razorpay = require('razorpay');

const instance = new Razorpay({
  key_id: process.env.RAZORPAY_KEY_ID,
  key_secret: process.env.RAZORPAY_KEY_SECRET
});
```

### Direct Initialization

```javascript
const Razorpay = require('razorpay');

const instance = new Razorpay({
  key_id: 'rzp_test_xxxxxxxxxx',
  key_secret: 'your_key_secret_here'
});
```

### Client Configuration Options

```javascript
const instance = new Razorpay({
  key_id: process.env.RAZORPAY_KEY_ID,
  key_secret: process.env.RAZORPAY_KEY_SECRET,
  headers: {
    'X-Razorpay-Account': 'acc_xxxxxxxxxxxxx'  // Optional: For partner integrations
  }
});
```

## Orders API

### Create Order

Create an order before accepting payments from customers:

```javascript
const Razorpay = require('razorpay');
const instance = new Razorpay({
  key_id: process.env.RAZORPAY_KEY_ID,
  key_secret: process.env.RAZORPAY_KEY_SECRET
});

async function createOrder() {
  try {
    const options = {
      amount: 50000,  // Amount in paise (50000 paise = ₹500)
      currency: "INR",
      receipt: "receipt_order_12345",
      notes: {
        key1: "value1",
        key2: "value2"
      }
    };

    const order = await instance.orders.create(options);
    console.log('Order ID:', order.id);
    console.log('Order:', order);
    return order;
  } catch (error) {
    console.error('Error creating order:', error);
    throw error;
  }
}
```

### Fetch Order Details

```javascript
async function fetchOrder(orderId) {
  try {
    const order = await instance.orders.fetch(orderId);
    console.log('Order details:', order);
    return order;
  } catch (error) {
    console.error('Error fetching order:', error);
    throw error;
  }
}
```

### Fetch All Orders

```javascript
async function fetchAllOrders() {
  try {
    const options = {
      count: 10,           // Number of orders to fetch (default: 10, max: 100)
      skip: 0,             // Number of orders to skip (for pagination)
      from: 1640995200,    // Unix timestamp (fetch orders from this time)
      to: 1672531199       // Unix timestamp (fetch orders till this time)
    };

    const orders = await instance.orders.all(options);
    console.log('Orders:', orders);
    return orders;
  } catch (error) {
    console.error('Error fetching orders:', error);
    throw error;
  }
}
```

### Fetch Payments for an Order

```javascript
async function fetchOrderPayments(orderId) {
  try {
    const payments = await instance.orders.fetchPayments(orderId);
    console.log('Payments for order:', payments);
    return payments;
  } catch (error) {
    console.error('Error fetching order payments:', error);
    throw error;
  }
}
```

## Payments API

### Fetch Payment Details

```javascript
async function fetchPayment(paymentId) {
  try {
    const payment = await instance.payments.fetch(paymentId);
    console.log('Payment details:', payment);
    return payment;
  } catch (error) {
    console.error('Error fetching payment:', error);
    throw error;
  }
}
```

### Fetch All Payments

```javascript
async function fetchAllPayments() {
  try {
    const options = {
      count: 10,
      skip: 0,
      from: 1640995200,
      to: 1672531199
    };

    const payments = await instance.payments.all(options);
    console.log('Payments:', payments);
    return payments;
  } catch (error) {
    console.error('Error fetching payments:', error);
    throw error;
  }
}
```

### Capture Payment

Capture an authorized payment:

```javascript
async function capturePayment(paymentId, amount) {
  try {
    const payment = await instance.payments.capture(paymentId, amount, "INR");
    console.log('Payment captured:', payment);
    return payment;
  } catch (error) {
    console.error('Error capturing payment:', error);
    throw error;
  }
}
```

### Update Payment

```javascript
async function updatePayment(paymentId) {
  try {
    const notes = {
      notes: {
        note_key_1: "updated value 1",
        note_key_2: "updated value 2"
      }
    };

    const payment = await instance.payments.edit(paymentId, notes);
    console.log('Payment updated:', payment);
    return payment;
  } catch (error) {
    console.error('Error updating payment:', error);
    throw error;
  }
}
```

## Refunds API

### Create Refund

```javascript
async function createRefund(paymentId) {
  try {
    const options = {
      amount: 10000,  // Amount in paise to refund
      speed: "normal",  // "normal" or "optimum"
      notes: {
        notes_key_1: "Refund reason"
      },
      receipt: "Receipt No. 31"
    };

    const refund = await instance.payments.refund(paymentId, options);
    console.log('Refund created:', refund);
    return refund;
  } catch (error) {
    console.error('Error creating refund:', error);
    throw error;
  }
}
```

### Fetch Refund Details

```javascript
async function fetchRefund(paymentId, refundId) {
  try {
    const refund = await instance.payments.fetchRefund(paymentId, refundId);
    console.log('Refund details:', refund);
    return refund;
  } catch (error) {
    console.error('Error fetching refund:', error);
    throw error;
  }
}
```

### Fetch All Refunds for a Payment

```javascript
async function fetchPaymentRefunds(paymentId) {
  try {
    const refunds = await instance.payments.fetchMultipleRefund(paymentId);
    console.log('Refunds:', refunds);
    return refunds;
  } catch (error) {
    console.error('Error fetching refunds:', error);
    throw error;
  }
}
```

### Fetch All Refunds

```javascript
async function fetchAllRefunds() {
  try {
    const options = {
      count: 10,
      skip: 0,
      from: 1640995200,
      to: 1672531199
    };

    const refunds = await instance.refunds.all(options);
    console.log('All refunds:', refunds);
    return refunds;
  } catch (error) {
    console.error('Error fetching all refunds:', error);
    throw error;
  }
}
```

## Customers API

### Create Customer

```javascript
async function createCustomer() {
  try {
    const customerData = {
      name: "John Doe",
      email: "john.doe@example.com",
      contact: "+919876543210",
      fail_existing: 0,  // 0 to return existing customer if exists, 1 to fail
      notes: {
        customer_type: "premium"
      }
    };

    const customer = await instance.customers.create(customerData);
    console.log('Customer created:', customer);
    return customer;
  } catch (error) {
    console.error('Error creating customer:', error);
    throw error;
  }
}
```

### Fetch Customer Details

```javascript
async function fetchCustomer(customerId) {
  try {
    const customer = await instance.customers.fetch(customerId);
    console.log('Customer details:', customer);
    return customer;
  } catch (error) {
    console.error('Error fetching customer:', error);
    throw error;
  }
}
```

### Update Customer

```javascript
async function updateCustomer(customerId) {
  try {
    const updates = {
      name: "Jane Doe",
      email: "jane.doe@example.com",
      contact: "+919876543211"
    };

    const customer = await instance.customers.edit(customerId, updates);
    console.log('Customer updated:', customer);
    return customer;
  } catch (error) {
    console.error('Error updating customer:', error);
    throw error;
  }
}
```

### Fetch All Customers

```javascript
async function fetchAllCustomers() {
  try {
    const options = {
      count: 10,
      skip: 0
    };

    const customers = await instance.customers.all(options);
    console.log('Customers:', customers);
    return customers;
  } catch (error) {
    console.error('Error fetching customers:', error);
    throw error;
  }
}
```

## Subscriptions API

### Create Subscription

```javascript
async function createSubscription() {
  try {
    const subscriptionData = {
      plan_id: "plan_xxxxxxxxxxxxx",
      customer_notify: 1,
      quantity: 1,
      total_count: 12,
      start_at: Math.floor(Date.now() / 1000) + 86400,  // Start after 1 day
      addons: [
        {
          item: {
            name: "Extra Storage",
            amount: 5000,
            currency: "INR"
          }
        }
      ],
      notes: {
        subscription_type: "premium"
      }
    };

    const subscription = await instance.subscriptions.create(subscriptionData);
    console.log('Subscription created:', subscription);
    return subscription;
  } catch (error) {
    console.error('Error creating subscription:', error);
    throw error;
  }
}
```

### Fetch Subscription Details

```javascript
async function fetchSubscription(subscriptionId) {
  try {
    const subscription = await instance.subscriptions.fetch(subscriptionId);
    console.log('Subscription details:', subscription);
    return subscription;
  } catch (error) {
    console.error('Error fetching subscription:', error);
    throw error;
  }
}
```

### Cancel Subscription

```javascript
async function cancelSubscription(subscriptionId) {
  try {
    const options = {
      cancel_at_cycle_end: 0  // 0 to cancel immediately, 1 to cancel at end of cycle
    };

    const subscription = await instance.subscriptions.cancel(subscriptionId, options);
    console.log('Subscription cancelled:', subscription);
    return subscription;
  } catch (error) {
    console.error('Error cancelling subscription:', error);
    throw error;
  }
}
```

### Fetch All Subscriptions

```javascript
async function fetchAllSubscriptions() {
  try {
    const options = {
      count: 10,
      skip: 0
    };

    const subscriptions = await instance.subscriptions.all(options);
    console.log('Subscriptions:', subscriptions);
    return subscriptions;
  } catch (error) {
    console.error('Error fetching subscriptions:', error);
    throw error;
  }
}
```

## Plans API

### Create Plan

```javascript
async function createPlan() {
  try {
    const planData = {
      period: "monthly",
      interval: 1,
      item: {
        name: "Premium Plan",
        amount: 99900,  // Amount in paise
        currency: "INR",
        description: "Premium monthly subscription"
      },
      notes: {
        plan_type: "premium"
      }
    };

    const plan = await instance.plans.create(planData);
    console.log('Plan created:', plan);
    return plan;
  } catch (error) {
    console.error('Error creating plan:', error);
    throw error;
  }
}
```

### Fetch Plan Details

```javascript
async function fetchPlan(planId) {
  try {
    const plan = await instance.plans.fetch(planId);
    console.log('Plan details:', plan);
    return plan;
  } catch (error) {
    console.error('Error fetching plan:', error);
    throw error;
  }
}
```

### Fetch All Plans

```javascript
async function fetchAllPlans() {
  try {
    const options = {
      count: 10,
      skip: 0
    };

    const plans = await instance.plans.all(options);
    console.log('Plans:', plans);
    return plans;
  } catch (error) {
    console.error('Error fetching plans:', error);
    throw error;
  }
}
```

## Invoices API

### Create Invoice

```javascript
async function createInvoice() {
  try {
    const invoiceData = {
      type: "invoice",
      description: "Invoice for the month of January 2025",
      customer: {
        name: "John Doe",
        email: "john.doe@example.com",
        contact: "+919876543210"
      },
      line_items: [
        {
          name: "Premium Subscription",
          description: "Monthly subscription",
          amount: 99900,
          currency: "INR",
          quantity: 1
        }
      ],
      currency: "INR",
      email_notify: 1,
      sms_notify: 1,
      date: Math.floor(Date.now() / 1000)
    };

    const invoice = await instance.invoices.create(invoiceData);
    console.log('Invoice created:', invoice);
    return invoice;
  } catch (error) {
    console.error('Error creating invoice:', error);
    throw error;
  }
}
```

### Fetch Invoice Details

```javascript
async function fetchInvoice(invoiceId) {
  try {
    const invoice = await instance.invoices.fetch(invoiceId);
    console.log('Invoice details:', invoice);
    return invoice;
  } catch (error) {
    console.error('Error fetching invoice:', error);
    throw error;
  }
}
```

### Fetch All Invoices

```javascript
async function fetchAllInvoices() {
  try {
    const options = {
      count: 10,
      skip: 0
    };

    const invoices = await instance.invoices.all(options);
    console.log('Invoices:', invoices);
    return invoices;
  } catch (error) {
    console.error('Error fetching invoices:', error);
    throw error;
  }
}
```

### Cancel Invoice

```javascript
async function cancelInvoice(invoiceId) {
  try {
    const invoice = await instance.invoices.cancel(invoiceId);
    console.log('Invoice cancelled:', invoice);
    return invoice;
  } catch (error) {
    console.error('Error cancelling invoice:', error);
    throw error;
  }
}
```

## Payment Verification

### Verify Payment Signature

Verify payment signatures to ensure payment authenticity:

```javascript
const crypto = require('crypto');
const { validatePaymentVerification } = require('razorpay/dist/utils/razorpay-utils');

// Method 1: Using Razorpay's built-in utility
function verifyPaymentSignature(orderId, paymentId, signature) {
  try {
    const isValid = validatePaymentVerification(
      { order_id: orderId, payment_id: paymentId },
      signature,
      process.env.RAZORPAY_KEY_SECRET
    );

    if (isValid) {
      console.log('Payment signature verified successfully');
      return true;
    } else {
      console.log('Invalid payment signature');
      return false;
    }
  } catch (error) {
    console.error('Error verifying payment signature:', error);
    return false;
  }
}

// Method 2: Manual verification
function verifyPaymentSignatureManual(orderId, paymentId, signature) {
  try {
    const generatedSignature = crypto
      .createHmac('sha256', process.env.RAZORPAY_KEY_SECRET)
      .update(`${orderId}|${paymentId}`)
      .digest('hex');

    const isValid = generatedSignature === signature;

    if (isValid) {
      console.log('Payment signature verified successfully');
      return true;
    } else {
      console.log('Invalid payment signature');
      return false;
    }
  } catch (error) {
    console.error('Error verifying payment signature:', error);
    return false;
  }
}
```

## Webhook Handling and Validation

### Express.js Webhook Integration

Handle and validate incoming Razorpay webhooks:

```javascript
const express = require('express');
const crypto = require('crypto');
const { validateWebhookSignature } = require('razorpay/dist/utils/razorpay-utils');

const app = express();

// IMPORTANT: Use express.text() to get raw body for webhook signature validation
app.use(express.json({
  verify: (req, res, buf) => {
    req.rawBody = buf.toString();
  }
}));

// Webhook validation middleware
function validateWebhook(req, res, next) {
  const signature = req.headers['x-razorpay-signature'];
  const webhookSecret = process.env.RAZORPAY_WEBHOOK_SECRET;

  try {
    const isValid = validateWebhookSignature(
      req.rawBody,
      signature,
      webhookSecret
    );

    if (!isValid) {
      return res.status(400).json({ error: 'Invalid signature' });
    }

    next();
  } catch (error) {
    console.error('Webhook validation error:', error);
    return res.status(400).json({ error: 'Webhook validation failed' });
  }
}

// Webhook endpoint
app.post('/webhook', validateWebhook, (req, res) => {
  const event = req.body;

  console.log('Webhook event:', event.event);

  switch (event.event) {
    case 'payment.authorized':
      handlePaymentAuthorized(event.payload.payment.entity);
      break;

    case 'payment.captured':
      handlePaymentCaptured(event.payload.payment.entity);
      break;

    case 'payment.failed':
      handlePaymentFailed(event.payload.payment.entity);
      break;

    case 'order.paid':
      handleOrderPaid(event.payload.order.entity);
      break;

    case 'refund.created':
      handleRefundCreated(event.payload.refund.entity);
      break;

    case 'subscription.activated':
      handleSubscriptionActivated(event.payload.subscription.entity);
      break;

    case 'subscription.cancelled':
      handleSubscriptionCancelled(event.payload.subscription.entity);
      break;

    case 'invoice.paid':
      handleInvoicePaid(event.payload.invoice.entity);
      break;

    default:
      console.log('Unhandled event:', event.event);
  }

  res.json({ status: 'ok' });
});

function handlePaymentAuthorized(payment) {
  console.log('Payment authorized:', payment.id);
  // Capture the payment or perform other actions
}

function handlePaymentCaptured(payment) {
  console.log('Payment captured:', payment.id);
  // Update order status, send confirmation email, etc.
}

function handlePaymentFailed(payment) {
  console.log('Payment failed:', payment.id);
  // Notify customer, update order status, etc.
}

function handleOrderPaid(order) {
  console.log('Order paid:', order.id);
  // Process order fulfillment
}

function handleRefundCreated(refund) {
  console.log('Refund created:', refund.id);
  // Update records, notify customer
}

function handleSubscriptionActivated(subscription) {
  console.log('Subscription activated:', subscription.id);
  // Grant access to subscription features
}

function handleSubscriptionCancelled(subscription) {
  console.log('Subscription cancelled:', subscription.id);
  // Revoke access, send cancellation email
}

function handleInvoicePaid(invoice) {
  console.log('Invoice paid:', invoice.id);
  // Update invoice status, send receipt
}

app.listen(3000, () => {
  console.log('Webhook server listening on port 3000');
});
```

### Manual Webhook Signature Validation

```javascript
const crypto = require('crypto');

function validateWebhookManual(webhookBody, signature, secret) {
  try {
    const generatedSignature = crypto
      .createHmac('sha256', secret)
      .update(webhookBody)
      .digest('hex');

    return generatedSignature === signature;
  } catch (error) {
    console.error('Error validating webhook:', error);
    return false;
  }
}

// Usage
app.post('/webhook', (req, res) => {
  const signature = req.headers['x-razorpay-signature'];
  const isValid = validateWebhookManual(
    req.rawBody,
    signature,
    process.env.RAZORPAY_WEBHOOK_SECRET
  );

  if (!isValid) {
    return res.status(400).json({ error: 'Invalid signature' });
  }

  // Process webhook event
  res.json({ status: 'ok' });
});
```

## Transfers API (Route)

### Create Transfer

Transfer funds to linked accounts:

```javascript
async function createTransfer(paymentId) {
  try {
    const transferData = {
      transfers: [
        {
          account: "acc_xxxxxxxxxxxxx",
          amount: 10000,  // Amount in paise
          currency: "INR",
          notes: {
            name: "Transfer to vendor",
            roll_no: "IEC2011025"
          },
          linked_account_notes: ["branch"],
          on_hold: 0,  // 0 to transfer immediately, 1 to hold
          on_hold_until: null
        }
      ]
    };

    const transfer = await instance.payments.transfer(paymentId, transferData);
    console.log('Transfer created:', transfer);
    return transfer;
  } catch (error) {
    console.error('Error creating transfer:', error);
    throw error;
  }
}
```

### Fetch Transfer Details

```javascript
async function fetchTransfer(transferId) {
  try {
    const transfer = await instance.transfers.fetch(transferId);
    console.log('Transfer details:', transfer);
    return transfer;
  } catch (error) {
    console.error('Error fetching transfer:', error);
    throw error;
  }
}
```

## Virtual Accounts API

### Create Virtual Account

```javascript
async function createVirtualAccount() {
  try {
    const vaData = {
      receivers: {
        types: ["bank_account"]
      },
      description: "Virtual Account for customer XYZ",
      customer_id: "cust_xxxxxxxxxxxxx",
      close_by: Math.floor(Date.now() / 1000) + 86400 * 30,  // Close after 30 days
      notes: {
        purpose: "Rent collection"
      }
    };

    const virtualAccount = await instance.virtualAccounts.create(vaData);
    console.log('Virtual Account created:', virtualAccount);
    return virtualAccount;
  } catch (error) {
    console.error('Error creating virtual account:', error);
    throw error;
  }
}
```

### Fetch Virtual Account Details

```javascript
async function fetchVirtualAccount(vaId) {
  try {
    const virtualAccount = await instance.virtualAccounts.fetch(vaId);
    console.log('Virtual Account details:', virtualAccount);
    return virtualAccount;
  } catch (error) {
    console.error('Error fetching virtual account:', error);
    throw error;
  }
}
```

## Error Handling

### Comprehensive Error Handling

```javascript
async function safeApiCall() {
  try {
    const order = await instance.orders.create({
      amount: 50000,
      currency: "INR",
      receipt: "receipt_12345"
    });

    return {
      success: true,
      data: order
    };
  } catch (error) {
    console.error('Razorpay API Error:', {
      statusCode: error.statusCode,
      error: error.error,
      description: error.error?.description,
      field: error.error?.field,
      reason: error.error?.reason
    });

    return {
      success: false,
      error: {
        code: error.statusCode,
        message: error.error?.description || 'An error occurred',
        field: error.error?.field
      }
    };
  }
}
```

### Common Error Codes

Handle specific error scenarios:

```javascript
async function handleErrors() {
  try {
    const payment = await instance.payments.fetch('pay_xxxxxxxxxxxxx');
    return payment;
  } catch (error) {
    switch (error.statusCode) {
      case 400:
        console.error('Bad Request - Invalid parameters');
        break;
      case 401:
        console.error('Unauthorized - Invalid API credentials');
        break;
      case 404:
        console.error('Not Found - Resource does not exist');
        break;
      case 500:
        console.error('Internal Server Error - Razorpay issue');
        break;
      default:
        console.error('Unknown error:', error);
    }
    throw error;
  }
}
```

## Complete Payment Flow Example

### Server-Side Integration

```javascript
const express = require('express');
const Razorpay = require('razorpay');
const crypto = require('crypto');
const { validatePaymentVerification } = require('razorpay/dist/utils/razorpay-utils');

const app = express();
app.use(express.json());

const instance = new Razorpay({
  key_id: process.env.RAZORPAY_KEY_ID,
  key_secret: process.env.RAZORPAY_KEY_SECRET
});

// Step 1: Create order
app.post('/create-order', async (req, res) => {
  try {
    const { amount, currency, receipt } = req.body;

    const options = {
      amount: amount * 100,  // Convert to paise
      currency: currency || 'INR',
      receipt: receipt || `receipt_${Date.now()}`,
      notes: {
        description: 'Order payment'
      }
    };

    const order = await instance.orders.create(options);

    res.json({
      success: true,
      order: {
        id: order.id,
        amount: order.amount,
        currency: order.currency
      },
      key_id: process.env.RAZORPAY_KEY_ID
    });
  } catch (error) {
    console.error('Error creating order:', error);
    res.status(500).json({
      success: false,
      message: 'Failed to create order'
    });
  }
});

// Step 2: Verify payment
app.post('/verify-payment', async (req, res) => {
  try {
    const { razorpay_order_id, razorpay_payment_id, razorpay_signature } = req.body;

    const isValid = validatePaymentVerification(
      { order_id: razorpay_order_id, payment_id: razorpay_payment_id },
      razorpay_signature,
      process.env.RAZORPAY_KEY_SECRET
    );

    if (isValid) {
      // Payment is successful, update database
      const payment = await instance.payments.fetch(razorpay_payment_id);

      res.json({
        success: true,
        message: 'Payment verified successfully',
        payment: {
          id: payment.id,
          amount: payment.amount,
          status: payment.status
        }
      });
    } else {
      res.status(400).json({
        success: false,
        message: 'Payment verification failed'
      });
    }
  } catch (error) {
    console.error('Error verifying payment:', error);
    res.status(500).json({
      success: false,
      message: 'Payment verification error'
    });
  }
});

// Step 3: Fetch payment details
app.get('/payment/:id', async (req, res) => {
  try {
    const payment = await instance.payments.fetch(req.params.id);
    res.json({
      success: true,
      payment
    });
  } catch (error) {
    console.error('Error fetching payment:', error);
    res.status(500).json({
      success: false,
      message: 'Failed to fetch payment'
    });
  }
});

app.listen(3000, () => {
  console.log('Server running on port 3000');
});
```

## Security Best Practices

### Never Expose Credentials

```javascript
// Good - Use environment variables
const instance = new Razorpay({
  key_id: process.env.RAZORPAY_KEY_ID,
  key_secret: process.env.RAZORPAY_KEY_SECRET
});

// Bad - Hardcoded credentials
const instance = new Razorpay({
  key_id: 'rzp_test_xxxxxxxxxxxx',
  key_secret: 'your_secret_here'
});
```

### Always Validate Webhooks

```javascript
// Always verify webhook signatures before processing
const isValid = validateWebhookSignature(
  req.rawBody,
  req.headers['x-razorpay-signature'],
  process.env.RAZORPAY_WEBHOOK_SECRET
);

if (!isValid) {
  return res.status(400).json({ error: 'Invalid signature' });
}
```

### Verify Payment Signatures

```javascript
// Always verify payment signatures on the server
const isValid = validatePaymentVerification(
  { order_id: orderId, payment_id: paymentId },
  signature,
  process.env.RAZORPAY_KEY_SECRET
);
```

## Useful Links

- **Documentation:** https://razorpay.com/docs/
- **API Reference:** https://razorpay.com/docs/api/
- **Node.js SDK Docs:** https://razorpay.com/docs/payments/server-integration/nodejs/
- **NPM Package:** https://www.npmjs.com/package/razorpay
- **GitHub Repository:** https://github.com/razorpay/razorpay-node
- **Dashboard:** https://dashboard.razorpay.com/
- **Support:** https://razorpay.com/support/

## Notes

This guide covers the core functionality of the Razorpay Node.js SDK. The SDK provides a promise-based API for all Razorpay services including payments, orders, refunds, subscriptions, customers, invoices, transfers, and virtual accounts. Always use environment variables for credentials and validate all payment signatures and webhook signatures to ensure security.
