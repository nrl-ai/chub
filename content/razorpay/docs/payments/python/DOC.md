---
name: payments
description: "Razorpay Python SDK coding guidelines for building payment systems with orders, payments, refunds, and subscriptions"
metadata:
  languages: "python"
  versions: "1.4.2"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "razorpay,payments,india,checkout,upi"
---

# Razorpay Python Coding Guidelines

You are a Razorpay payment gateway coding expert. Help me with writing code using the Razorpay Python SDK for building payment systems with orders, payments, refunds, subscriptions, customers, and invoices.

Please follow the following guidelines when generating code.

You can find the official SDK documentation and code samples here:
https://razorpay.com/docs/payments/server-integration/python/

## Golden Rule: Use the Correct and Current SDK

Always use the official Razorpay Python SDK, which is the standard library for all Razorpay API interactions.

**Library Name:** Razorpay Python SDK
**Python Package:** `razorpay`
**Current Version:** 1.4.2
**Supported Python Versions:** Python 3.12+

**Installation:**
- **Correct:** `pip install razorpay`

**APIs and Usage:**
- **Correct:** `import razorpay`
- **Correct:** `client = razorpay.Client(auth=("YOUR_KEY_ID", "YOUR_KEY_SECRET"))`
- **Correct:** `client.order.create(data={...})` for creating orders
- **Correct:** `client.payment.fetch(payment_id)` for fetching payments
- **Incorrect:** Using unofficial Razorpay libraries or wrappers
- **Incorrect:** Exposing credentials in client-side applications

## Authentication and Initialization

The Razorpay Python library requires your Key ID and Key Secret for authentication. You can obtain these from the Razorpay Dashboard at Settings > API Keys.

### Environment Variables (Recommended)

Set up environment variables for secure credential management:

```python
import os
import razorpay
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

# Initialize Razorpay client
client = razorpay.Client(auth=(
    os.environ.get('RAZORPAY_KEY_ID'),
    os.environ.get('RAZORPAY_KEY_SECRET')
))
```

### Direct Initialization

```python
import razorpay

# Initialize client with credentials
client = razorpay.Client(auth=(
    'rzp_test_xxxxxxxxxx',
    'your_key_secret_here'
))
```

### Client Configuration Options

```python
import razorpay

# Enable retry on network errors
client = razorpay.Client(auth=(
    os.environ.get('RAZORPAY_KEY_ID'),
    os.environ.get('RAZORPAY_KEY_SECRET')
))

# Enable automatic retry
client.enable_retry(True)

# Set custom app details
client.set_app_details({
    "title": "My Application",
    "version": "1.0.0"
})
```

## Orders API

### Create Order

Create an order before accepting payments from customers:

```python
import razorpay
import os

client = razorpay.Client(auth=(
    os.environ.get('RAZORPAY_KEY_ID'),
    os.environ.get('RAZORPAY_KEY_SECRET')
))

def create_order():
    try:
        data = {
            'amount': 50000,  # Amount in paise (50000 paise = ₹500)
            'currency': 'INR',
            'receipt': 'receipt_order_12345',
            'notes': {
                'key1': 'value1',
                'key2': 'value2'
            }
        }

        order = client.order.create(data=data)
        print('Order ID:', order['id'])
        print('Order:', order)
        return order
    except Exception as e:
        print('Error creating order:', str(e))
        raise
```

### Fetch Order Details

```python
def fetch_order(order_id):
    try:
        order = client.order.fetch(order_id)
        print('Order details:', order)
        return order
    except Exception as e:
        print('Error fetching order:', str(e))
        raise
```

### Fetch All Orders

```python
def fetch_all_orders():
    try:
        options = {
            'count': 10,           # Number of orders to fetch (default: 10, max: 100)
            'skip': 0,             # Number of orders to skip (for pagination)
            'from': 1640995200,    # Unix timestamp (fetch orders from this time)
            'to': 1672531199       # Unix timestamp (fetch orders till this time)
        }

        orders = client.order.all(options)
        print('Orders:', orders)
        return orders
    except Exception as e:
        print('Error fetching orders:', str(e))
        raise
```

### Fetch Payments for an Order

```python
def fetch_order_payments(order_id):
    try:
        payments = client.order.payments(order_id)
        print('Payments for order:', payments)
        return payments
    except Exception as e:
        print('Error fetching order payments:', str(e))
        raise
```

### Update Order

```python
def update_order(order_id):
    try:
        data = {
            'notes': {
                'updated_key': 'updated_value'
            }
        }

        order = client.order.edit(order_id, data)
        print('Order updated:', order)
        return order
    except Exception as e:
        print('Error updating order:', str(e))
        raise
```

## Payments API

### Fetch Payment Details

```python
def fetch_payment(payment_id):
    try:
        payment = client.payment.fetch(payment_id)
        print('Payment details:', payment)
        return payment
    except Exception as e:
        print('Error fetching payment:', str(e))
        raise
```

### Fetch All Payments

```python
def fetch_all_payments():
    try:
        options = {
            'count': 10,
            'skip': 0,
            'from': 1640995200,
            'to': 1672531199
        }

        payments = client.payment.all(options)
        print('Payments:', payments)
        return payments
    except Exception as e:
        print('Error fetching payments:', str(e))
        raise
```

### Capture Payment

Capture an authorized payment:

```python
def capture_payment(payment_id, amount):
    try:
        data = {
            'amount': amount,  # Amount in paise
            'currency': 'INR'
        }

        payment = client.payment.capture(payment_id, amount)
        print('Payment captured:', payment)
        return payment
    except Exception as e:
        print('Error capturing payment:', str(e))
        raise
```

### Update Payment

```python
def update_payment(payment_id):
    try:
        data = {
            'notes': {
                'note_key_1': 'updated value 1',
                'note_key_2': 'updated value 2'
            }
        }

        payment = client.payment.edit(payment_id, data)
        print('Payment updated:', payment)
        return payment
    except Exception as e:
        print('Error updating payment:', str(e))
        raise
```

### Fetch Card Details

```python
def fetch_card_details(payment_id):
    try:
        card = client.payment.fetch_card_details(payment_id)
        print('Card details:', card)
        return card
    except Exception as e:
        print('Error fetching card details:', str(e))
        raise
```

## Refunds API

### Create Refund

```python
def create_refund(payment_id):
    try:
        data = {
            'amount': 10000,  # Amount in paise to refund
            'speed': 'normal',  # 'normal' or 'optimum'
            'notes': {
                'notes_key_1': 'Refund reason'
            },
            'receipt': 'Receipt No. 31'
        }

        refund = client.payment.refund(payment_id, data)
        print('Refund created:', refund)
        return refund
    except Exception as e:
        print('Error creating refund:', str(e))
        raise
```

### Fetch Refund Details

```python
def fetch_refund(refund_id):
    try:
        refund = client.refund.fetch(refund_id)
        print('Refund details:', refund)
        return refund
    except Exception as e:
        print('Error fetching refund:', str(e))
        raise
```

### Fetch All Refunds for a Payment

```python
def fetch_payment_refunds(payment_id):
    try:
        refunds = client.payment.refunds(payment_id)
        print('Refunds:', refunds)
        return refunds
    except Exception as e:
        print('Error fetching refunds:', str(e))
        raise
```

### Fetch All Refunds

```python
def fetch_all_refunds():
    try:
        options = {
            'count': 10,
            'skip': 0,
            'from': 1640995200,
            'to': 1672531199
        }

        refunds = client.refund.all(options)
        print('All refunds:', refunds)
        return refunds
    except Exception as e:
        print('Error fetching all refunds:', str(e))
        raise
```

### Update Refund

```python
def update_refund(refund_id):
    try:
        data = {
            'notes': {
                'updated_note': 'Updated refund note'
            }
        }

        refund = client.refund.edit(refund_id, data)
        print('Refund updated:', refund)
        return refund
    except Exception as e:
        print('Error updating refund:', str(e))
        raise
```

## Customers API

### Create Customer

```python
def create_customer():
    try:
        data = {
            'name': 'John Doe',
            'email': 'john.doe@example.com',
            'contact': '+919876543210',
            'fail_existing': 0,  # 0 to return existing customer if exists, 1 to fail
            'notes': {
                'customer_type': 'premium'
            }
        }

        customer = client.customer.create(data)
        print('Customer created:', customer)
        return customer
    except Exception as e:
        print('Error creating customer:', str(e))
        raise
```

### Fetch Customer Details

```python
def fetch_customer(customer_id):
    try:
        customer = client.customer.fetch(customer_id)
        print('Customer details:', customer)
        return customer
    except Exception as e:
        print('Error fetching customer:', str(e))
        raise
```

### Update Customer

```python
def update_customer(customer_id):
    try:
        data = {
            'name': 'Jane Doe',
            'email': 'jane.doe@example.com',
            'contact': '+919876543211'
        }

        customer = client.customer.edit(customer_id, data)
        print('Customer updated:', customer)
        return customer
    except Exception as e:
        print('Error updating customer:', str(e))
        raise
```

### Fetch All Customers

```python
def fetch_all_customers():
    try:
        options = {
            'count': 10,
            'skip': 0
        }

        customers = client.customer.all(options)
        print('Customers:', customers)
        return customers
    except Exception as e:
        print('Error fetching customers:', str(e))
        raise
```

## Subscriptions API

### Create Subscription

```python
import time

def create_subscription():
    try:
        data = {
            'plan_id': 'plan_xxxxxxxxxxxxx',
            'customer_notify': 1,
            'quantity': 1,
            'total_count': 12,
            'start_at': int(time.time()) + 86400,  # Start after 1 day
            'addons': [
                {
                    'item': {
                        'name': 'Extra Storage',
                        'amount': 5000,
                        'currency': 'INR'
                    }
                }
            ],
            'notes': {
                'subscription_type': 'premium'
            }
        }

        subscription = client.subscription.create(data)
        print('Subscription created:', subscription)
        return subscription
    except Exception as e:
        print('Error creating subscription:', str(e))
        raise
```

### Fetch Subscription Details

```python
def fetch_subscription(subscription_id):
    try:
        subscription = client.subscription.fetch(subscription_id)
        print('Subscription details:', subscription)
        return subscription
    except Exception as e:
        print('Error fetching subscription:', str(e))
        raise
```

### Cancel Subscription

```python
def cancel_subscription(subscription_id):
    try:
        data = {
            'cancel_at_cycle_end': 0  # 0 to cancel immediately, 1 to cancel at end of cycle
        }

        subscription = client.subscription.cancel(subscription_id, data)
        print('Subscription cancelled:', subscription)
        return subscription
    except Exception as e:
        print('Error cancelling subscription:', str(e))
        raise
```

### Fetch All Subscriptions

```python
def fetch_all_subscriptions():
    try:
        options = {
            'count': 10,
            'skip': 0
        }

        subscriptions = client.subscription.all(options)
        print('Subscriptions:', subscriptions)
        return subscriptions
    except Exception as e:
        print('Error fetching subscriptions:', str(e))
        raise
```

### Update Subscription

```python
def update_subscription(subscription_id):
    try:
        data = {
            'plan_id': 'plan_yyyyyyyyyyy',
            'quantity': 2,
            'remaining_count': 6
        }

        subscription = client.subscription.edit(subscription_id, data)
        print('Subscription updated:', subscription)
        return subscription
    except Exception as e:
        print('Error updating subscription:', str(e))
        raise
```

## Plans API

### Create Plan

```python
def create_plan():
    try:
        data = {
            'period': 'monthly',
            'interval': 1,
            'item': {
                'name': 'Premium Plan',
                'amount': 99900,  # Amount in paise
                'currency': 'INR',
                'description': 'Premium monthly subscription'
            },
            'notes': {
                'plan_type': 'premium'
            }
        }

        plan = client.plan.create(data)
        print('Plan created:', plan)
        return plan
    except Exception as e:
        print('Error creating plan:', str(e))
        raise
```

### Fetch Plan Details

```python
def fetch_plan(plan_id):
    try:
        plan = client.plan.fetch(plan_id)
        print('Plan details:', plan)
        return plan
    except Exception as e:
        print('Error fetching plan:', str(e))
        raise
```

### Fetch All Plans

```python
def fetch_all_plans():
    try:
        options = {
            'count': 10,
            'skip': 0
        }

        plans = client.plan.all(options)
        print('Plans:', plans)
        return plans
    except Exception as e:
        print('Error fetching plans:', str(e))
        raise
```

## Invoices API

### Create Invoice

```python
import time

def create_invoice():
    try:
        data = {
            'type': 'invoice',
            'description': 'Invoice for the month of January 2025',
            'customer': {
                'name': 'John Doe',
                'email': 'john.doe@example.com',
                'contact': '+919876543210'
            },
            'line_items': [
                {
                    'name': 'Premium Subscription',
                    'description': 'Monthly subscription',
                    'amount': 99900,
                    'currency': 'INR',
                    'quantity': 1
                }
            ],
            'currency': 'INR',
            'email_notify': 1,
            'sms_notify': 1,
            'date': int(time.time())
        }

        invoice = client.invoice.create(data)
        print('Invoice created:', invoice)
        return invoice
    except Exception as e:
        print('Error creating invoice:', str(e))
        raise
```

### Fetch Invoice Details

```python
def fetch_invoice(invoice_id):
    try:
        invoice = client.invoice.fetch(invoice_id)
        print('Invoice details:', invoice)
        return invoice
    except Exception as e:
        print('Error fetching invoice:', str(e))
        raise
```

### Fetch All Invoices

```python
def fetch_all_invoices():
    try:
        options = {
            'count': 10,
            'skip': 0
        }

        invoices = client.invoice.all(options)
        print('Invoices:', invoices)
        return invoices
    except Exception as e:
        print('Error fetching invoices:', str(e))
        raise
```

### Cancel Invoice

```python
def cancel_invoice(invoice_id):
    try:
        invoice = client.invoice.cancel(invoice_id)
        print('Invoice cancelled:', invoice)
        return invoice
    except Exception as e:
        print('Error cancelling invoice:', str(e))
        raise
```

### Issue Invoice

```python
def issue_invoice(invoice_id):
    try:
        invoice = client.invoice.issue(invoice_id)
        print('Invoice issued:', invoice)
        return invoice
    except Exception as e:
        print('Error issuing invoice:', str(e))
        raise
```

### Update Invoice

```python
def update_invoice(invoice_id):
    try:
        data = {
            'notes': {
                'updated_note': 'Updated invoice note'
            }
        }

        invoice = client.invoice.edit(invoice_id, data)
        print('Invoice updated:', invoice)
        return invoice
    except Exception as e:
        print('Error updating invoice:', str(e))
        raise
```

## Payment Verification

### Verify Payment Signature

Verify payment signatures to ensure payment authenticity:

```python
import hmac
import hashlib

# Method 1: Using Razorpay's built-in utility
def verify_payment_signature(order_id, payment_id, signature):
    try:
        params_dict = {
            'razorpay_order_id': order_id,
            'razorpay_payment_id': payment_id,
            'razorpay_signature': signature
        }

        # This will raise SignatureVerificationError if signature is invalid
        client.utility.verify_payment_signature(params_dict)
        print('Payment signature verified successfully')
        return True
    except razorpay.errors.SignatureVerificationError as e:
        print('Invalid payment signature:', str(e))
        return False
    except Exception as e:
        print('Error verifying payment signature:', str(e))
        return False

# Method 2: Manual verification
def verify_payment_signature_manual(order_id, payment_id, signature):
    try:
        key_secret = os.environ.get('RAZORPAY_KEY_SECRET')

        # Create the expected signature
        message = f"{order_id}|{payment_id}"
        generated_signature = hmac.new(
            key_secret.encode(),
            message.encode(),
            hashlib.sha256
        ).hexdigest()

        # Compare signatures
        is_valid = hmac.compare_digest(generated_signature, signature)

        if is_valid:
            print('Payment signature verified successfully')
            return True
        else:
            print('Invalid payment signature')
            return False
    except Exception as e:
        print('Error verifying payment signature:', str(e))
        return False
```

## Webhook Handling and Validation

### Flask Webhook Integration

Handle and validate incoming Razorpay webhooks:

```python
from flask import Flask, request, jsonify
import razorpay
import os
import hmac
import hashlib

app = Flask(__name__)

client = razorpay.Client(auth=(
    os.environ.get('RAZORPAY_KEY_ID'),
    os.environ.get('RAZORPAY_KEY_SECRET')
))

def validate_webhook_signature(webhook_body, signature, secret):
    """Validate webhook signature"""
    try:
        generated_signature = hmac.new(
            secret.encode(),
            webhook_body,
            hashlib.sha256
        ).hexdigest()

        return hmac.compare_digest(generated_signature, signature)
    except Exception as e:
        print('Error validating webhook:', str(e))
        return False

@app.route('/webhook', methods=['POST'])
def handle_webhook():
    # Get the webhook signature from headers
    signature = request.headers.get('X-Razorpay-Signature')
    webhook_secret = os.environ.get('RAZORPAY_WEBHOOK_SECRET')

    # Get raw request body
    webhook_body = request.get_data()

    # Validate signature
    is_valid = validate_webhook_signature(webhook_body, signature, webhook_secret)

    if not is_valid:
        return jsonify({'error': 'Invalid signature'}), 400

    # Parse webhook event
    event = request.get_json()

    print('Webhook event:', event.get('event'))

    # Handle different event types
    event_type = event.get('event')

    if event_type == 'payment.authorized':
        handle_payment_authorized(event['payload']['payment']['entity'])
    elif event_type == 'payment.captured':
        handle_payment_captured(event['payload']['payment']['entity'])
    elif event_type == 'payment.failed':
        handle_payment_failed(event['payload']['payment']['entity'])
    elif event_type == 'order.paid':
        handle_order_paid(event['payload']['order']['entity'])
    elif event_type == 'refund.created':
        handle_refund_created(event['payload']['refund']['entity'])
    elif event_type == 'subscription.activated':
        handle_subscription_activated(event['payload']['subscription']['entity'])
    elif event_type == 'subscription.cancelled':
        handle_subscription_cancelled(event['payload']['subscription']['entity'])
    elif event_type == 'invoice.paid':
        handle_invoice_paid(event['payload']['invoice']['entity'])
    else:
        print('Unhandled event:', event_type)

    return jsonify({'status': 'ok'}), 200

def handle_payment_authorized(payment):
    print('Payment authorized:', payment['id'])
    # Capture the payment or perform other actions

def handle_payment_captured(payment):
    print('Payment captured:', payment['id'])
    # Update order status, send confirmation email, etc.

def handle_payment_failed(payment):
    print('Payment failed:', payment['id'])
    # Notify customer, update order status, etc.

def handle_order_paid(order):
    print('Order paid:', order['id'])
    # Process order fulfillment

def handle_refund_created(refund):
    print('Refund created:', refund['id'])
    # Update records, notify customer

def handle_subscription_activated(subscription):
    print('Subscription activated:', subscription['id'])
    # Grant access to subscription features

def handle_subscription_cancelled(subscription):
    print('Subscription cancelled:', subscription['id'])
    # Revoke access, send cancellation email

def handle_invoice_paid(invoice):
    print('Invoice paid:', invoice['id'])
    # Update invoice status, send receipt

if __name__ == '__main__':
    app.run(port=3000, debug=True)
```

### Using Razorpay's Webhook Verification Utility

```python
@app.route('/webhook', methods=['POST'])
def handle_webhook_with_utility():
    signature = request.headers.get('X-Razorpay-Signature')
    webhook_secret = os.environ.get('RAZORPAY_WEBHOOK_SECRET')
    webhook_body = request.get_data().decode('utf-8')

    try:
        # Verify webhook signature using Razorpay utility
        client.utility.verify_webhook_signature(
            webhook_body,
            signature,
            webhook_secret
        )

        # Process webhook event
        event = request.get_json()
        # Handle event...

        return jsonify({'status': 'ok'}), 200
    except razorpay.errors.SignatureVerificationError:
        return jsonify({'error': 'Invalid signature'}), 400
```

## Transfers API (Route)

### Create Transfer

Transfer funds to linked accounts:

```python
def create_transfer(payment_id):
    try:
        data = {
            'transfers': [
                {
                    'account': 'acc_xxxxxxxxxxxxx',
                    'amount': 10000,  # Amount in paise
                    'currency': 'INR',
                    'notes': {
                        'name': 'Transfer to vendor',
                        'roll_no': 'IEC2011025'
                    },
                    'linked_account_notes': ['branch'],
                    'on_hold': 0,  # 0 to transfer immediately, 1 to hold
                    'on_hold_until': None
                }
            ]
        }

        transfer = client.payment.transfer(payment_id, data)
        print('Transfer created:', transfer)
        return transfer
    except Exception as e:
        print('Error creating transfer:', str(e))
        raise
```

### Fetch Transfer Details

```python
def fetch_transfer(transfer_id):
    try:
        transfer = client.transfer.fetch(transfer_id)
        print('Transfer details:', transfer)
        return transfer
    except Exception as e:
        print('Error fetching transfer:', str(e))
        raise
```

### Fetch All Transfers

```python
def fetch_all_transfers():
    try:
        options = {
            'count': 10,
            'skip': 0
        }

        transfers = client.transfer.all(options)
        print('Transfers:', transfers)
        return transfers
    except Exception as e:
        print('Error fetching transfers:', str(e))
        raise
```

## Virtual Accounts API

### Create Virtual Account

```python
import time

def create_virtual_account():
    try:
        data = {
            'receivers': {
                'types': ['bank_account']
            },
            'description': 'Virtual Account for customer XYZ',
            'customer_id': 'cust_xxxxxxxxxxxxx',
            'close_by': int(time.time()) + 86400 * 30,  # Close after 30 days
            'notes': {
                'purpose': 'Rent collection'
            }
        }

        virtual_account = client.virtual_account.create(data)
        print('Virtual Account created:', virtual_account)
        return virtual_account
    except Exception as e:
        print('Error creating virtual account:', str(e))
        raise
```

### Fetch Virtual Account Details

```python
def fetch_virtual_account(va_id):
    try:
        virtual_account = client.virtual_account.fetch(va_id)
        print('Virtual Account details:', virtual_account)
        return virtual_account
    except Exception as e:
        print('Error fetching virtual account:', str(e))
        raise
```

### Fetch All Virtual Accounts

```python
def fetch_all_virtual_accounts():
    try:
        options = {
            'count': 10,
            'skip': 0
        }

        virtual_accounts = client.virtual_account.all(options)
        print('Virtual Accounts:', virtual_accounts)
        return virtual_accounts
    except Exception as e:
        print('Error fetching virtual accounts:', str(e))
        raise
```

### Close Virtual Account

```python
def close_virtual_account(va_id):
    try:
        virtual_account = client.virtual_account.close(va_id)
        print('Virtual Account closed:', virtual_account)
        return virtual_account
    except Exception as e:
        print('Error closing virtual account:', str(e))
        raise
```

## Error Handling

### Comprehensive Error Handling

```python
import razorpay.errors as errors

def safe_api_call():
    try:
        data = {
            'amount': 50000,
            'currency': 'INR',
            'receipt': 'receipt_12345'
        }

        order = client.order.create(data=data)

        return {
            'success': True,
            'data': order
        }
    except errors.BadRequestError as e:
        print('Bad Request Error:', e)
        return {
            'success': False,
            'error': {
                'code': 400,
                'message': str(e)
            }
        }
    except errors.UnauthorizedError as e:
        print('Unauthorized Error:', e)
        return {
            'success': False,
            'error': {
                'code': 401,
                'message': 'Invalid API credentials'
            }
        }
    except errors.ServerError as e:
        print('Server Error:', e)
        return {
            'success': False,
            'error': {
                'code': 500,
                'message': 'Razorpay server error'
            }
        }
    except errors.GatewayError as e:
        print('Gateway Error:', e)
        return {
            'success': False,
            'error': {
                'code': 502,
                'message': 'Gateway error occurred'
            }
        }
    except Exception as e:
        print('Unknown error:', e)
        return {
            'success': False,
            'error': {
                'code': 500,
                'message': str(e)
            }
        }
```

### Specific Error Handling

```python
def handle_specific_errors():
    try:
        payment = client.payment.fetch('pay_xxxxxxxxxxxxx')
        return payment
    except errors.BadRequestError as e:
        print('Bad Request - Invalid parameters')
        raise
    except errors.UnauthorizedError as e:
        print('Unauthorized - Invalid API credentials')
        raise
    except errors.NotFoundError as e:
        print('Not Found - Resource does not exist')
        raise
    except errors.ServerError as e:
        print('Internal Server Error - Razorpay issue')
        raise
    except Exception as e:
        print('Unknown error:', str(e))
        raise
```

## Complete Payment Flow Example

### Flask Server Integration

```python
from flask import Flask, request, jsonify
import razorpay
import os
import time

app = Flask(__name__)

client = razorpay.Client(auth=(
    os.environ.get('RAZORPAY_KEY_ID'),
    os.environ.get('RAZORPAY_KEY_SECRET')
))

# Step 1: Create order
@app.route('/create-order', methods=['POST'])
def create_order():
    try:
        data = request.get_json()
        amount = data.get('amount')
        currency = data.get('currency', 'INR')

        order_data = {
            'amount': amount * 100,  # Convert to paise
            'currency': currency,
            'receipt': f"receipt_{int(time.time())}",
            'notes': {
                'description': 'Order payment'
            }
        }

        order = client.order.create(data=order_data)

        return jsonify({
            'success': True,
            'order': {
                'id': order['id'],
                'amount': order['amount'],
                'currency': order['currency']
            },
            'key_id': os.environ.get('RAZORPAY_KEY_ID')
        }), 200
    except Exception as e:
        print('Error creating order:', str(e))
        return jsonify({
            'success': False,
            'message': 'Failed to create order'
        }), 500

# Step 2: Verify payment
@app.route('/verify-payment', methods=['POST'])
def verify_payment():
    try:
        data = request.get_json()
        razorpay_order_id = data.get('razorpay_order_id')
        razorpay_payment_id = data.get('razorpay_payment_id')
        razorpay_signature = data.get('razorpay_signature')

        params_dict = {
            'razorpay_order_id': razorpay_order_id,
            'razorpay_payment_id': razorpay_payment_id,
            'razorpay_signature': razorpay_signature
        }

        # Verify signature
        client.utility.verify_payment_signature(params_dict)

        # Payment is successful, fetch payment details
        payment = client.payment.fetch(razorpay_payment_id)

        return jsonify({
            'success': True,
            'message': 'Payment verified successfully',
            'payment': {
                'id': payment['id'],
                'amount': payment['amount'],
                'status': payment['status']
            }
        }), 200
    except razorpay.errors.SignatureVerificationError:
        return jsonify({
            'success': False,
            'message': 'Payment verification failed'
        }), 400
    except Exception as e:
        print('Error verifying payment:', str(e))
        return jsonify({
            'success': False,
            'message': 'Payment verification error'
        }), 500

# Step 3: Fetch payment details
@app.route('/payment/<payment_id>', methods=['GET'])
def get_payment(payment_id):
    try:
        payment = client.payment.fetch(payment_id)
        return jsonify({
            'success': True,
            'payment': payment
        }), 200
    except Exception as e:
        print('Error fetching payment:', str(e))
        return jsonify({
            'success': False,
            'message': 'Failed to fetch payment'
        }), 500

if __name__ == '__main__':
    app.run(port=3000, debug=True)
```

## Security Best Practices

### Never Expose Credentials

```python
# Good - Use environment variables
import os

client = razorpay.Client(auth=(
    os.environ.get('RAZORPAY_KEY_ID'),
    os.environ.get('RAZORPAY_KEY_SECRET')
))

# Bad - Hardcoded credentials
client = razorpay.Client(auth=(
    'rzp_test_xxxxxxxxxxxx',
    'your_secret_here'
))
```

### Always Validate Webhooks

```python
# Always verify webhook signatures before processing
is_valid = validate_webhook_signature(
    webhook_body,
    signature,
    os.environ.get('RAZORPAY_WEBHOOK_SECRET')
)

if not is_valid:
    return jsonify({'error': 'Invalid signature'}), 400
```

### Verify Payment Signatures

```python
# Always verify payment signatures on the server
params_dict = {
    'razorpay_order_id': order_id,
    'razorpay_payment_id': payment_id,
    'razorpay_signature': signature
}

client.utility.verify_payment_signature(params_dict)
```

## Useful Links

- **Documentation:** https://razorpay.com/docs/
- **API Reference:** https://razorpay.com/docs/api/
- **Python SDK Docs:** https://razorpay.com/docs/payments/server-integration/python/
- **PyPI Package:** https://pypi.org/project/razorpay/
- **GitHub Repository:** https://github.com/razorpay/razorpay-python
- **Dashboard:** https://dashboard.razorpay.com/
- **Support:** https://razorpay.com/support/

## Notes

This guide covers the core functionality of the Razorpay Python SDK. The SDK provides comprehensive support for all Razorpay services including payments, orders, refunds, subscriptions, customers, invoices, transfers, and virtual accounts. Always use environment variables for credentials and validate all payment signatures and webhook signatures to ensure security. The SDK requires Python 3.12 or higher and provides utilities for payment verification and webhook signature validation.
