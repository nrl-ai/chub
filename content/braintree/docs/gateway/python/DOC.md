---
name: gateway
description: "Braintree Python SDK for payment gateway, PayPal, and subscriptions"
metadata:
  languages: "python"
  versions: "4.40.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "braintree,gateway,payments,paypal,subscriptions"
---

# Braintree Python SDK Context

## Golden Rule

**ALWAYS use the official Braintree Python SDK package: `braintree`**

```bash
pip install braintree
```

**DO NOT use:**
- Deprecated or unofficial packages
- Direct API calls without the SDK
- Outdated versions (version 3.40.0+ required for SSL certificate support)

The official package is `braintree` and is maintained by Braintree (a PayPal service).

## Installation

```bash
pip install braintree
```

The SDK requires Python 3.8 or higher. The SDK also depends on the `requests` library.

## Environment Configuration

Create a `.env` file with your Braintree credentials:

```bash
BRAINTREE_ENVIRONMENT=Sandbox
BRAINTREE_MERCHANT_ID=your_merchant_id
BRAINTREE_PUBLIC_KEY=your_public_key
BRAINTREE_PRIVATE_KEY=your_private_key
```

Load environment variables using `python-dotenv`:

```bash
pip install python-dotenv
```

```python
from dotenv import load_dotenv
import os

load_dotenv()
```

## Initialization

### Basic Gateway Setup

```python
import braintree

gateway = braintree.BraintreeGateway(
    braintree.Configuration(
        braintree.Environment.Sandbox,
        merchant_id="your_merchant_id",
        public_key="your_public_key",
        private_key="your_private_key"
    )
)
```

### Using Environment Variables

```python
import braintree
import os

gateway = braintree.BraintreeGateway(
    braintree.Configuration(
        braintree.Environment.Production if os.getenv('BRAINTREE_ENVIRONMENT') == 'Production'
        else braintree.Environment.Sandbox,
        merchant_id=os.getenv('BRAINTREE_MERCHANT_ID'),
        public_key=os.getenv('BRAINTREE_PUBLIC_KEY'),
        private_key=os.getenv('BRAINTREE_PRIVATE_KEY')
    )
)
```

### Environment Options

```python
# Sandbox environment (for testing)
braintree.Environment.Sandbox

# Production environment (for live transactions)
braintree.Environment.Production
```

## Client Token Generation

Client tokens contain authorization and configuration information for the client SDK.

### Basic Client Token

```python
client_token = gateway.client_token.generate()
```

### Client Token with Customer ID

```python
client_token = gateway.client_token.generate({
    "customer_id": "customer_123"
})
```

### Flask Endpoint for Client Token

```python
from flask import Flask, jsonify

app = Flask(__name__)

@app.route("/client_token", methods=["GET"])
def get_client_token():
    try:
        token = gateway.client_token.generate()
        return jsonify({"client_token": token})
    except Exception as e:
        return jsonify({"error": str(e)}), 500
```

### Django View for Client Token

```python
from django.http import JsonResponse

def client_token(request):
    try:
        token = gateway.client_token.generate()
        return JsonResponse({"client_token": token})
    except Exception as e:
        return JsonResponse({"error": str(e)}, status=500)
```

## Transactions

### Creating a Transaction

**Basic Sale:**
```python
result = gateway.transaction.sale({
    "amount": "10.00",
    "payment_method_nonce": nonce_from_the_client,
    "options": {
        "submit_for_settlement": True
    }
})

if result.is_success:
    print("Transaction ID: " + result.transaction.id)
else:
    print("Error: " + result.message)
```

### Transaction with Device Data

```python
result = gateway.transaction.sale({
    "amount": "10.00",
    "payment_method_nonce": nonce_from_the_client,
    "device_data": device_data_from_the_client,
    "options": {
        "submit_for_settlement": True
    }
})
```

### Transaction with Customer Information

```python
result = gateway.transaction.sale({
    "amount": "100.00",
    "payment_method_nonce": nonce_from_the_client,
    "customer": {
        "first_name": "John",
        "last_name": "Doe",
        "email": "john.doe@example.com",
        "phone": "312-555-1234"
    },
    "billing": {
        "first_name": "John",
        "last_name": "Doe",
        "street_address": "123 Main St",
        "locality": "Chicago",
        "region": "IL",
        "postal_code": "60614",
        "country_code_alpha2": "US"
    },
    "shipping": {
        "first_name": "John",
        "last_name": "Doe",
        "street_address": "123 Main St",
        "locality": "Chicago",
        "region": "IL",
        "postal_code": "60614",
        "country_code_alpha2": "US"
    },
    "options": {
        "submit_for_settlement": True
    }
})
```

### Transaction with Stored Payment Method

```python
result = gateway.transaction.sale({
    "amount": "10.00",
    "payment_method_token": "the_token",
    "options": {
        "submit_for_settlement": True
    }
})
```

### Transaction with Customer ID

```python
result = gateway.transaction.sale({
    "amount": "10.00",
    "customer_id": "customer_123",
    "options": {
        "submit_for_settlement": True
    }
})
```

### Flask Checkout Endpoint

```python
from flask import Flask, request, jsonify

app = Flask(__name__)

@app.route("/checkout", methods=["POST"])
def checkout():
    nonce_from_the_client = request.form.get("payment_method_nonce")
    amount = request.form.get("amount")

    result = gateway.transaction.sale({
        "amount": amount,
        "payment_method_nonce": nonce_from_the_client,
        "options": {
            "submit_for_settlement": True
        }
    })

    if result.is_success:
        return jsonify({
            "success": True,
            "transaction_id": result.transaction.id
        })
    else:
        return jsonify({
            "success": False,
            "message": result.message
        }), 500
```

### Finding a Transaction

```python
transaction = gateway.transaction.find("the_transaction_id")
print(transaction.amount)
print(transaction.status)
print(transaction.credit_card_details.last_4)
```

### Refunding a Transaction

**Full Refund:**
```python
result = gateway.transaction.refund("the_transaction_id")

if result.is_success:
    print("Refund successful")
```

**Partial Refund:**
```python
result = gateway.transaction.refund("the_transaction_id", "10.00")

if result.is_success:
    print("Partial refund successful")
```

### Voiding a Transaction

```python
result = gateway.transaction.void("the_transaction_id")

if result.is_success:
    print("Transaction voided")
else:
    print("Error: " + result.message)
```

### Submitting for Settlement

**Full Amount:**
```python
result = gateway.transaction.submit_for_settlement("transaction_id")

if result.is_success:
    print("Submitted for settlement")
```

**Partial Amount:**
```python
result = gateway.transaction.submit_for_settlement("transaction_id", "50.00")

if result.is_success:
    print("Submitted for partial settlement")
```

## Customer Management

### Creating a Customer

**Basic Customer:**
```python
result = gateway.customer.create({
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@example.com",
    "phone": "312-555-1234"
})

if result.is_success:
    print("Customer ID: " + result.customer.id)
```

### Creating Customer with Company Information

```python
result = gateway.customer.create({
    "first_name": "John",
    "last_name": "Doe",
    "company": "Acme Corp",
    "email": "john.doe@example.com",
    "phone": "312-555-1234",
    "fax": "614-555-5678",
    "website": "www.example.com"
})
```

### Creating Customer with Payment Method

```python
result = gateway.customer.create({
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@example.com",
    "payment_method_nonce": nonce_from_the_client
})

if result.is_success:
    customer_id = result.customer.id
    payment_token = result.customer.payment_methods[0].token
    print(f"Customer ID: {customer_id}")
    print(f"Payment Method Token: {payment_token}")
```

### Creating Customer with Credit Card

```python
result = gateway.customer.create({
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@example.com",
    "credit_card": {
        "number": "4111111111111111",
        "expiration_date": "05/2025",
        "cvv": "123",
        "billing_address": {
            "postal_code": "60614"
        }
    }
})

if result.is_success:
    token = result.customer.payment_methods[0].token
    print("Payment method token: " + token)
```

### Finding a Customer

```python
customer = gateway.customer.find("the_customer_id")
print(customer.first_name)
print(customer.last_name)
print(customer.email)
print(customer.payment_methods)
```

### Updating a Customer

```python
result = gateway.customer.update("the_customer_id", {
    "first_name": "Jane",
    "last_name": "Smith",
    "email": "jane.smith@example.com"
})

if result.is_success:
    print("Customer updated")
```

### Deleting a Customer

```python
result = gateway.customer.delete("the_customer_id")

if result.is_success:
    print("Customer deleted")
```

### Searching for Customers

```python
collection = gateway.customer.search(
    braintree.CustomerSearch.first_name == "John",
    braintree.CustomerSearch.last_name == "Doe"
)

for customer in collection.items:
    print(customer.id)
    print(customer.email)
```

## Payment Method Management

### Creating a Payment Method

```python
result = gateway.payment_method.create({
    "customer_id": "customer_123",
    "payment_method_nonce": nonce_from_the_client
})

if result.is_success:
    print("Payment Method Token: " + result.payment_method.token)
```

### Creating Payment Method with Billing Address

```python
result = gateway.payment_method.create({
    "customer_id": "customer_123",
    "payment_method_nonce": nonce_from_the_client,
    "billing_address": {
        "street_address": "123 Main St",
        "locality": "Chicago",
        "region": "IL",
        "postal_code": "60614"
    }
})
```

### Creating Payment Method with Verification

```python
result = gateway.payment_method.create({
    "customer_id": "customer_123",
    "payment_method_nonce": nonce_from_the_client,
    "options": {
        "verify_card": True
    }
})

if result.is_success:
    print("Payment method verified and created")
```

### Finding a Payment Method

```python
payment_method = gateway.payment_method.find("the_token")
print(payment_method.token)
print(payment_method.card_type)
```

### Updating a Payment Method

**Set as Default:**
```python
result = gateway.payment_method.update("the_token", {
    "options": {
        "make_default": True
    }
})

if result.is_success:
    print("Payment method is now default")
```

**Update Billing Address:**
```python
result = gateway.payment_method.update("the_token", {
    "billing_address": {
        "street_address": "456 Oak Ave",
        "locality": "Chicago",
        "region": "IL",
        "postal_code": "60614",
        "options": {
            "update_existing": True
        }
    }
})
```

### Deleting a Payment Method

```python
result = gateway.payment_method.delete("the_token")

if result.is_success:
    print("Payment method deleted")
```

## Subscription Billing

### Creating a Subscription

**Basic Subscription:**
```python
result = gateway.subscription.create({
    "payment_method_token": "the_token",
    "plan_id": "plan_id"
})

if result.is_success:
    print("Subscription ID: " + result.subscription.id)
```

### Creating Subscription with Price Override

```python
result = gateway.subscription.create({
    "payment_method_token": "the_token",
    "plan_id": "plan_id",
    "price": "15.00"
})
```

### Creating Subscription with Add-ons and Discounts

```python
result = gateway.subscription.create({
    "payment_method_token": "the_token",
    "plan_id": "plan_id",
    "add_ons": {
        "add": [
            {
                "inherited_from_id": "add_on_id_1",
                "amount": "10.00"
            }
        ]
    },
    "discounts": {
        "add": [
            {
                "inherited_from_id": "discount_id_1",
                "amount": "5.00"
            }
        ]
    }
})
```

### Creating Subscription with Trial Period

```python
result = gateway.subscription.create({
    "payment_method_token": "the_token",
    "plan_id": "plan_id",
    "trial_duration": 14,
    "trial_duration_unit": braintree.Subscription.TrialDurationUnit.Day,
    "trial_period": True
})
```

### Finding a Subscription

```python
subscription = gateway.subscription.find("subscription_id")
print(subscription.status)
print(subscription.price)
print(subscription.plan_id)
```

### Updating a Subscription

```python
result = gateway.subscription.update("subscription_id", {
    "price": "15.00"
})

if result.is_success:
    print("Subscription updated")
```

### Updating Subscription Plan

```python
result = gateway.subscription.update("subscription_id", {
    "plan_id": "new_plan_id",
    "price": "20.00"
})
```

### Canceling a Subscription

```python
result = gateway.subscription.cancel("subscription_id")

if result.is_success:
    print("Subscription canceled")
```

### Retrieving All Plans

```python
plans = gateway.plan.all()

for plan in plans:
    print(plan.id)
    print(plan.name)
    print(plan.price)
```

## Webhooks

### Parsing Webhook Notifications

```python
webhook_notification = gateway.webhook_notification.parse(
    request.form["bt_signature"],
    request.form["bt_payload"]
)

print(webhook_notification.kind)
print(webhook_notification.timestamp)
```

### Flask Webhook Endpoint

```python
from flask import Flask, request

app = Flask(__name__)

@app.route('/webhooks', methods=['POST'])
def webhooks():
    try:
        webhook_notification = gateway.webhook_notification.parse(
            request.form["bt_signature"],
            request.form["bt_payload"]
        )

        # Handle different webhook types
        if webhook_notification.kind == braintree.WebhookNotification.Kind.SubscriptionCanceled:
            print(f"Subscription {webhook_notification.subscription.id} canceled")
        elif webhook_notification.kind == braintree.WebhookNotification.Kind.SubscriptionChargedSuccessfully:
            print(f"Subscription {webhook_notification.subscription.id} charged")
        elif webhook_notification.kind == braintree.WebhookNotification.Kind.DisputeOpened:
            print(f"Dispute opened for transaction {webhook_notification.dispute.transaction.id}")

        return "", 200
    except Exception as e:
        print(f"Error: {e}")
        return "", 400
```

### Django Webhook View

```python
from django.http import HttpResponse
from django.views.decorators.csrf import csrf_exempt

@csrf_exempt
def webhook(request):
    try:
        webhook_notification = gateway.webhook_notification.parse(
            request.POST["bt_signature"],
            request.POST["bt_payload"]
        )

        # Handle webhook
        if webhook_notification.kind == braintree.WebhookNotification.Kind.SubscriptionWentPastDue:
            subscription_id = webhook_notification.subscription.id
            # Handle past due subscription

        return HttpResponse(status=200)
    except Exception as e:
        return HttpResponse(status=400)
```

### Handling Subscription Webhooks

```python
webhook_notification = gateway.webhook_notification.parse(
    signature,
    payload
)

if webhook_notification.kind == braintree.WebhookNotification.Kind.SubscriptionWentPastDue:
    subscription_id = webhook_notification.subscription.id
    # Handle past due subscription
elif webhook_notification.kind == braintree.WebhookNotification.Kind.SubscriptionChargedSuccessfully:
    subscription_id = webhook_notification.subscription.id
    transaction_id = webhook_notification.subscription.transactions[0].id
    # Handle successful charge
```

### Verifying Webhook Signatures

```python
verification_string = gateway.webhook_notification.verify("challenge_string")
print(verification_string)
```

### Testing Webhooks

```python
sample_notification = gateway.webhook_testing.sample_notification(
    braintree.WebhookNotification.Kind.SubscriptionWentPastDue,
    "subscription_id"
)

webhook_notification = gateway.webhook_notification.parse(
    sample_notification['bt_signature'],
    sample_notification['bt_payload']
)

print(webhook_notification.kind)
print(webhook_notification.subscription.id)
```

## Advanced Features

### Searching Transactions

```python
collection = gateway.transaction.search(
    braintree.TransactionSearch.amount >= "10.00",
    braintree.TransactionSearch.amount <= "100.00",
    braintree.TransactionSearch.status == braintree.Transaction.Status.Settled,
    braintree.TransactionSearch.created_at >= "2024-01-01"
)

for transaction in collection.items:
    print(transaction.id)
    print(transaction.amount)
```

### Transaction with Custom Fields

```python
result = gateway.transaction.sale({
    "amount": "10.00",
    "payment_method_nonce": nonce_from_the_client,
    "custom_fields": {
        "order_id": "12345",
        "user_type": "premium"
    },
    "options": {
        "submit_for_settlement": True
    }
})
```

### Transaction with Order ID

```python
result = gateway.transaction.sale({
    "amount": "10.00",
    "payment_method_nonce": nonce_from_the_client,
    "order_id": "order_123",
    "options": {
        "submit_for_settlement": True
    }
})
```

### Holding in Escrow

```python
result = gateway.transaction.sale({
    "amount": "100.00",
    "payment_method_nonce": nonce_from_the_client,
    "service_fee_amount": "10.00",
    "merchant_account_id": "submerchant_account_id",
    "options": {
        "submit_for_settlement": True,
        "hold_in_escrow": True
    }
})
```

### Transaction with Descriptor

```python
result = gateway.transaction.sale({
    "amount": "10.00",
    "payment_method_nonce": nonce_from_the_client,
    "descriptor": {
        "name": "Company*Product",
        "phone": "8004567890",
        "url": "example.com"
    },
    "options": {
        "submit_for_settlement": True
    }
})
```

### Storing Payment Method on Success

```python
result = gateway.transaction.sale({
    "amount": "10.00",
    "payment_method_nonce": nonce_from_the_client,
    "customer_id": "customer_123",
    "options": {
        "submit_for_settlement": True,
        "store_in_vault_on_success": True
    }
})

if result.is_success:
    payment_token = result.transaction.credit_card_details.token
    print(f"Payment method stored with token: {payment_token}")
```

### Cloning a Transaction

```python
result = gateway.transaction.clone_transaction(
    "original_transaction_id",
    {
        "amount": "20.00",
        "options": {
            "submit_for_settlement": True
        }
    }
)

if result.is_success:
    print("Cloned transaction ID: " + result.transaction.id)
```

### Dispute Management

**Finding a Dispute:**
```python
dispute = gateway.dispute.find("dispute_id")
print(dispute.status)
print(dispute.amount)
print(dispute.reason)
```

**Accept Dispute:**
```python
result = gateway.dispute.accept("dispute_id")
```

**Add Text Evidence:**
```python
result = gateway.dispute.add_text_evidence("dispute_id", "evidence_text")
```

**Finalize Dispute:**
```python
result = gateway.dispute.finalize("dispute_id")
```

### Merchant Account Creation

```python
result = gateway.merchant_account.create({
    "individual": {
        "first_name": "John",
        "last_name": "Doe",
        "email": "john.doe@example.com",
        "date_of_birth": "1980-01-01",
        "ssn": "123-45-6789",
        "address": {
            "street_address": "123 Main St",
            "locality": "Chicago",
            "region": "IL",
            "postal_code": "60614"
        }
    },
    "business": {
        "legal_name": "John Doe Inc",
        "dba_name": "JD Services",
        "tax_id": "12-3456789",
        "address": {
            "street_address": "123 Main St",
            "locality": "Chicago",
            "region": "IL",
            "postal_code": "60614"
        }
    },
    "funding": {
        "destination": braintree.MerchantAccount.FundingDestination.Bank,
        "routing_number": "021000021",
        "account_number": "1234567890"
    },
    "tos_accepted": True,
    "master_merchant_account_id": "master_merchant_account_id"
})

if result.is_success:
    print("Merchant Account ID: " + result.merchant_account.id)
```

## Error Handling

### Checking Result Success

```python
result = gateway.transaction.sale({
    "amount": "10.00",
    "payment_method_nonce": nonce_from_the_client,
    "options": {
        "submit_for_settlement": True
    }
})

if result.is_success:
    print("Transaction ID:", result.transaction.id)
else:
    print("Error:", result.message)

    # Check validation errors
    for error in result.errors.deep_errors:
        print("Error:", error.code)
        print("Message:", error.message)
        print("Attribute:", error.attribute)
```

### Handling Validation Errors

```python
result = gateway.customer.create({
    "email": "invalid-email"
})

if not result.is_success:
    for error in result.errors.deep_errors:
        print(f"Error Code: {error.code}")
        print(f"Message: {error.message}")
        print(f"Attribute: {error.attribute}")
```

### Handling Not Found Errors

```python
try:
    customer = gateway.customer.find("nonexistent_id")
except braintree.exceptions.NotFoundError:
    print("Customer not found")
```

### Handling Authentication Errors

```python
try:
    result = gateway.transaction.sale({
        "amount": "10.00",
        "payment_method_nonce": nonce_from_the_client
    })
except braintree.exceptions.AuthenticationError:
    print("Invalid API credentials")
except braintree.exceptions.AuthorizationError:
    print("Not authorized to perform this action")
```

## Common Patterns

### Complete Flask Application Example

```python
from flask import Flask, request, jsonify, render_template
import braintree
import os

app = Flask(__name__)

gateway = braintree.BraintreeGateway(
    braintree.Configuration(
        braintree.Environment.Sandbox,
        merchant_id=os.getenv('BRAINTREE_MERCHANT_ID'),
        public_key=os.getenv('BRAINTREE_PUBLIC_KEY'),
        private_key=os.getenv('BRAINTREE_PRIVATE_KEY')
    )
)

@app.route('/')
def index():
    return render_template('index.html')

@app.route('/client_token', methods=['GET'])
def get_client_token():
    try:
        token = gateway.client_token.generate()
        return jsonify({"client_token": token})
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route('/checkout', methods=['POST'])
def checkout():
    nonce_from_the_client = request.form.get("payment_method_nonce")
    amount = request.form.get("amount")

    result = gateway.transaction.sale({
        "amount": amount,
        "payment_method_nonce": nonce_from_the_client,
        "options": {
            "submit_for_settlement": True
        }
    })

    if result.is_success:
        return jsonify({
            "success": True,
            "transaction_id": result.transaction.id
        })
    else:
        return jsonify({
            "success": False,
            "message": result.message
        }), 500

if __name__ == '__main__':
    app.run(debug=True, port=3000)
```

### Customer Creation with Payment Method Workflow

```python
# Step 1: Create customer
result = gateway.customer.create({
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@example.com",
    "payment_method_nonce": nonce_from_the_client
})

if result.is_success:
    customer_id = result.customer.id
    payment_token = result.customer.payment_methods[0].token

    # Step 2: Create subscription
    sub_result = gateway.subscription.create({
        "payment_method_token": payment_token,
        "plan_id": "monthly_plan"
    })

    if sub_result.is_success:
        print("Subscription ID:", sub_result.subscription.id)
```

### Stored Payment Method Workflow

```python
# Step 1: Get customer
customer = gateway.customer.find("customer_id")

# Step 2: Find default payment method
default_payment_method = None
for pm in customer.payment_methods:
    if pm.default:
        default_payment_method = pm
        break

# Step 3: Use default payment method for transaction
if default_payment_method:
    result = gateway.transaction.sale({
        "amount": "25.00",
        "payment_method_token": default_payment_method.token,
        "options": {
            "submit_for_settlement": True
        }
    })

    if result.is_success:
        print("Transaction ID:", result.transaction.id)
```

### Django Integration Example

```python
# views.py
from django.http import JsonResponse
from django.views.decorators.csrf import csrf_exempt
import braintree
import os

gateway = braintree.BraintreeGateway(
    braintree.Configuration(
        braintree.Environment.Sandbox,
        merchant_id=os.getenv('BRAINTREE_MERCHANT_ID'),
        public_key=os.getenv('BRAINTREE_PUBLIC_KEY'),
        private_key=os.getenv('BRAINTREE_PRIVATE_KEY')
    )
)

def client_token(request):
    try:
        token = gateway.client_token.generate()
        return JsonResponse({"client_token": token})
    except Exception as e:
        return JsonResponse({"error": str(e)}, status=500)

@csrf_exempt
def checkout(request):
    if request.method == 'POST':
        nonce = request.POST.get('payment_method_nonce')
        amount = request.POST.get('amount')

        result = gateway.transaction.sale({
            "amount": amount,
            "payment_method_nonce": nonce,
            "options": {
                "submit_for_settlement": True
            }
        })

        if result.is_success:
            return JsonResponse({
                "success": True,
                "transaction_id": result.transaction.id
            })
        else:
            return JsonResponse({
                "success": False,
                "message": result.message
            }, status=500)
```

### Subscription Renewal Handler

```python
def handle_subscription_renewal(subscription_id):
    try:
        subscription = gateway.subscription.find(subscription_id)

        if subscription.status == braintree.Subscription.Status.PastDue:
            # Attempt to retry billing
            result = gateway.subscription.retry_charge(subscription_id)

            if result.is_success:
                print(f"Successfully charged subscription {subscription_id}")
            else:
                print(f"Failed to charge subscription: {result.message}")

    except braintree.exceptions.NotFoundError:
        print(f"Subscription {subscription_id} not found")
```
