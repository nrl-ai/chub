---
name: payments
description: "Square Python SDK coding guide for payments, POS, and commerce checkout"
metadata:
  languages: "python"
  versions: "43.2.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "square,payments,pos,commerce,checkout"
---

# Square Python SDK Coding Guide

## 1. Golden Rule

**Always use the official Square Python SDK package:**
- Package name: `squareup` (Python library for Square API)
- Official repository: https://github.com/square/square-python-sdk

**Never use deprecated libraries:**
- `squareconnect` (DEPRECATED - replaced by `squareup`)
- `squareup_legacy` (legacy version for migration purposes only)

**Current SDK Version:** v43.2.0

**API Versioning:** Square uses date-based API versioning (e.g., 2025-08-20). Your account is automatically pinned to an API version. You can override this when initializing the client.

## 2. Installation

```bash
pip install squareup
```

```bash
poetry add squareup
```

```bash
uv add squareup
```

**Requirements:** Python 3.8+ (Python 3.10+ recommended for production)

### Environment Variables

```bash
# Required
SQUARE_TOKEN=EAAAl...          # Your Square access token
SQUARE_ENVIRONMENT=sandbox     # or "production"

# Optional
SQUARE_LOCATION_ID=L88917...  # Default location ID for operations
```

**CRITICAL:** Never commit access tokens to version control. Use environment variables or secure secret management systems. Use sandbox tokens for development and testing.

## 3. Initialization

### Basic Initialization

```python
import os
from square import Square

client = Square(
    token=os.environ.get("SQUARE_TOKEN"),
)
```

### With Environment Specification

```python
import os
from square import Square
from square.environment import Environment

client = Square(
    environment=Environment.SANDBOX,
    token=os.environ.get("SQUARE_TOKEN"),
)
```

### Advanced Configuration

```python
import os
from square import Square
from square.environment import Environment

client = Square(
    environment=Environment.PRODUCTION,
    token=os.environ.get("SQUARE_TOKEN"),
    timeout=60,                    # 60 second timeout
    max_retries=3,                 # Maximum retry attempts
    square_version="2025-08-20",   # Override API version
    user_agent_detail="MyApp/1.0.0",
)
```

### Async Client Initialization

```python
import os
from square import AsyncSquare
from square.environment import Environment

async_client = AsyncSquare(
    environment=Environment.SANDBOX,
    token=os.environ.get("SQUARE_TOKEN"),
)
```

### Error Handling Setup

```python
from square import Square
from square.core.api_error import ApiError

client = Square(
    token=os.environ.get("SQUARE_TOKEN"),
)

try:
    # Make API calls here
    pass
except ApiError as e:
    for error in e.errors:
        print(f"Category: {error.category}")
        print(f"Code: {error.code}")
        print(f"Detail: {error.detail}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## 4. Core API Surfaces

### Locations API

Get information about business locations.

**List All Locations:**

```python
try:
    response = client.locations.list()
    locations = response.locations

    for location in locations:
        print(f"{location.id}: {location.name}")
        print(f"Address: {location.address.address_line_1}")
        print(f"City: {location.address.locality}")

except ApiError as e:
    print("Error listing locations:", e.errors)
```

**Retrieve Single Location:**

```python
def get_location(location_id: str):
    try:
        response = client.locations.retrieve(location_id)
        return response.location
    except ApiError as e:
        print("Error retrieving location:", e.errors)
        raise
```

**Async List Locations:**

```python
import asyncio

async def list_locations_async():
    try:
        response = await async_client.locations.list()
        return response.locations
    except ApiError as e:
        print("Error listing locations:", e.errors)
        raise

# Usage
locations = asyncio.run(list_locations_async())
```

### Payments API

Process and manage payments.

**Create Payment (Minimal):**

```python
import uuid

def create_payment(source_id: str, amount: int, location_id: str):
    try:
        response = client.payments.create(
            source_id=source_id,
            idempotency_key=str(uuid.uuid4()),
            amount_money={
                "amount": amount,
                "currency": "USD",
            },
            location_id=location_id,
        )
        return response.payment
    except ApiError as e:
        print("Payment error:", e.errors)
        raise
```

**Create Payment (Advanced):**

```python
import uuid

def create_advanced_payment(payment_data: dict):
    try:
        response = client.payments.create(
            source_id=payment_data["source_id"],
            idempotency_key=str(uuid.uuid4()),
            amount_money={
                "amount": payment_data["amount"],
                "currency": payment_data.get("currency", "USD"),
            },
            location_id=payment_data["location_id"],
            customer_id=payment_data.get("customer_id"),
            reference_id=payment_data.get("reference_id"),
            note=payment_data.get("note"),
            autocomplete=True,
            buyer_email_address=payment_data.get("email"),
            billing_address={
                "address_line_1": payment_data.get("address_line_1"),
                "locality": payment_data.get("city"),
                "administrative_district_level_1": payment_data.get("state"),
                "postal_code": payment_data.get("postal_code"),
                "country": payment_data.get("country", "US"),
            },
        )
        return response.payment
    except ApiError as e:
        print("Payment creation failed:", e.errors)
        raise
```

**List Payments:**

```python
def list_payments(location_id: str, begin_time: str, end_time: str):
    try:
        response = client.payments.list(
            location_id=location_id,
            begin_time=begin_time,
            end_time=end_time,
            sort_order="DESC",
            limit=100,
        )
        return response.payments
    except ApiError as e:
        print("Error listing payments:", e.errors)
        raise
```

**Get Payment:**

```python
def get_payment(payment_id: str):
    try:
        response = client.payments.retrieve(payment_id)
        return response.payment
    except ApiError as e:
        print("Error retrieving payment:", e.errors)
        raise
```

**Complete Payment:**

```python
def complete_payment(payment_id: str):
    try:
        response = client.payments.complete(payment_id)
        return response.payment
    except ApiError as e:
        print("Error completing payment:", e.errors)
        raise
```

**Cancel Payment:**

```python
def cancel_payment(payment_id: str):
    try:
        response = client.payments.cancel(payment_id)
        return response.payment
    except ApiError as e:
        print("Error canceling payment:", e.errors)
        raise
```

**Async Create Payment:**

```python
import asyncio
import uuid

async def create_payment_async(source_id: str, amount: int, location_id: str):
    try:
        response = await async_client.payments.create(
            source_id=source_id,
            idempotency_key=str(uuid.uuid4()),
            amount_money={
                "amount": amount,
                "currency": "USD",
            },
            location_id=location_id,
        )
        return response.payment
    except ApiError as e:
        print("Payment error:", e.errors)
        raise
```

### Refunds API

Manage payment refunds.

**Create Refund (Minimal):**

```python
import uuid

def create_refund(payment_id: str, amount: int, currency: str = "USD"):
    try:
        response = client.refunds.create(
            idempotency_key=str(uuid.uuid4()),
            payment_id=payment_id,
            amount_money={
                "amount": amount,
                "currency": currency,
            },
        )
        return response.refund
    except ApiError as e:
        print("Refund error:", e.errors)
        raise
```

**Create Refund (Advanced):**

```python
import uuid

def create_advanced_refund(refund_data: dict):
    try:
        response = client.refunds.create(
            idempotency_key=str(uuid.uuid4()),
            payment_id=refund_data["payment_id"],
            amount_money={
                "amount": refund_data["amount"],
                "currency": refund_data.get("currency", "USD"),
            },
            reason=refund_data.get("reason"),
            location_id=refund_data.get("location_id"),
        )
        return response.refund
    except ApiError as e:
        print("Refund creation failed:", e.errors)
        raise
```

**Get Refund:**

```python
def get_refund(refund_id: str):
    try:
        response = client.refunds.retrieve(refund_id)
        return response.refund
    except ApiError as e:
        print("Error retrieving refund:", e.errors)
        raise
```

**List Refunds:**

```python
def list_refunds(location_id: str, begin_time: str, end_time: str):
    try:
        response = client.refunds.list(
            location_id=location_id,
            begin_time=begin_time,
            end_time=end_time,
            sort_order="DESC",
        )
        return response.refunds
    except ApiError as e:
        print("Error listing refunds:", e.errors)
        raise
```

### Orders API

Create and manage orders.

**Create Order (Minimal):**

```python
import uuid

def create_order(location_id: str):
    try:
        response = client.orders.create(
            idempotency_key=str(uuid.uuid4()),
            order={
                "location_id": location_id,
                "line_items": [
                    {
                        "name": "Item Name",
                        "quantity": "1",
                        "base_price_money": {
                            "amount": 1000,
                            "currency": "USD",
                        },
                    },
                ],
            },
        )
        return response.order
    except ApiError as e:
        print("Order creation error:", e.errors)
        raise
```

**Create Order (Advanced):**

```python
import uuid

def create_advanced_order(order_data: dict):
    try:
        line_items = [
            {
                "name": item["name"],
                "quantity": str(item["quantity"]),
                "base_price_money": {
                    "amount": item["price"],
                    "currency": item.get("currency", "USD"),
                },
                "note": item.get("note"),
            }
            for item in order_data["line_items"]
        ]

        taxes = [
            {
                "name": "Sales Tax",
                "percentage": "8.5",
                "scope": "ORDER",
            }
        ]

        discounts = [
            {
                "name": discount["name"],
                "percentage": discount["percentage"],
                "scope": "ORDER",
            }
            for discount in order_data.get("discounts", [])
        ]

        response = client.orders.create(
            idempotency_key=str(uuid.uuid4()),
            order={
                "location_id": order_data["location_id"],
                "reference_id": order_data.get("reference_id"),
                "customer_id": order_data.get("customer_id"),
                "line_items": line_items,
                "taxes": taxes,
                "discounts": discounts,
            },
        )
        return response.order
    except ApiError as e:
        print("Order creation failed:", e.errors)
        raise
```

**Retrieve Order:**

```python
def retrieve_order(order_id: str):
    try:
        response = client.orders.retrieve(order_id)
        return response.order
    except ApiError as e:
        print("Error retrieving order:", e.errors)
        raise
```

**Update Order:**

```python
def update_order(order_id: str, updates: dict):
    try:
        response = client.orders.update(
            order_id=order_id,
            order=updates,
        )
        return response.order
    except ApiError as e:
        print("Error updating order:", e.errors)
        raise
```

**Search Orders:**

```python
def search_orders(location_ids: list, query: dict):
    try:
        search_query = {
            "filter": {
                "state_filter": {
                    "states": ["OPEN", "COMPLETED"],
                },
            },
            "sort": {
                "sort_field": "CREATED_AT",
                "sort_order": "DESC",
            },
        }

        if query.get("date_time_filter"):
            search_query["filter"]["date_time_filter"] = query["date_time_filter"]

        if query.get("customer_id"):
            search_query["filter"]["customer_filter"] = {
                "customer_ids": [query["customer_id"]],
            }

        response = client.orders.search(
            location_ids=location_ids,
            query=search_query,
            limit=query.get("limit", 100),
        )
        return response.orders
    except ApiError as e:
        print("Error searching orders:", e.errors)
        raise
```

### Customers API

Manage customer profiles.

**Create Customer (Minimal):**

```python
import uuid

def create_customer(email: str, given_name: str, family_name: str):
    try:
        response = client.customers.create(
            idempotency_key=str(uuid.uuid4()),
            email_address=email,
            given_name=given_name,
            family_name=family_name,
        )
        return response.customer
    except ApiError as e:
        print("Customer creation error:", e.errors)
        raise
```

**Create Customer (Advanced):**

```python
import uuid

def create_advanced_customer(customer_data: dict):
    try:
        response = client.customers.create(
            idempotency_key=str(uuid.uuid4()),
            given_name=customer_data["given_name"],
            family_name=customer_data["family_name"],
            email_address=customer_data.get("email_address"),
            phone_number=customer_data.get("phone_number"),
            address={
                "address_line_1": customer_data.get("address_line_1"),
                "address_line_2": customer_data.get("address_line_2"),
                "locality": customer_data.get("city"),
                "administrative_district_level_1": customer_data.get("state"),
                "postal_code": customer_data.get("postal_code"),
                "country": customer_data.get("country", "US"),
            },
            reference_id=customer_data.get("reference_id"),
            note=customer_data.get("note"),
            birthday=customer_data.get("birthday"),
            company_name=customer_data.get("company_name"),
        )
        return response.customer
    except ApiError as e:
        print("Customer creation failed:", e.errors)
        raise
```

**List Customers with Pagination:**

```python
def list_customers(cursor: str = None, limit: int = 100):
    try:
        response = client.customers.list(
            cursor=cursor,
            limit=limit,
            sort_field="CREATED_AT",
            sort_order="DESC",
        )
        return {
            "customers": response.customers,
            "cursor": response.cursor,
        }
    except ApiError as e:
        print("Error listing customers:", e.errors)
        raise
```

**List All Customers with Auto-Pagination:**

```python
def list_all_customers():
    try:
        all_customers = []
        response = client.customers.list(limit=100)

        # Auto-pagination: iterate over all pages
        for customer in response:
            all_customers.append(customer)

        return all_customers
    except ApiError as e:
        print("Error listing customers:", e.errors)
        raise
```

**Retrieve Customer:**

```python
def retrieve_customer(customer_id: str):
    try:
        response = client.customers.retrieve(customer_id)
        return response.customer
    except ApiError as e:
        print("Error retrieving customer:", e.errors)
        raise
```

**Update Customer:**

```python
def update_customer(customer_id: str, updates: dict):
    try:
        response = client.customers.update(
            customer_id=customer_id,
            **updates
        )
        return response.customer
    except ApiError as e:
        print("Error updating customer:", e.errors)
        raise
```

**Delete Customer:**

```python
def delete_customer(customer_id: str):
    try:
        response = client.customers.delete(customer_id)
        return response
    except ApiError as e:
        print("Error deleting customer:", e.errors)
        raise
```

**Search Customers:**

```python
def search_customers(query: dict):
    try:
        search_query = {
            "filter": {},
        }

        if query.get("email"):
            search_query["filter"]["email_address"] = {
                "exact": query["email"],
            }

        if query.get("phone"):
            search_query["filter"]["phone_number"] = {
                "exact": query["phone"],
            }

        if query.get("created_at"):
            search_query["filter"]["created_at"] = query["created_at"]

        response = client.customers.search(
            query=search_query,
            limit=query.get("limit", 100),
        )
        return response.customers
    except ApiError as e:
        print("Error searching customers:", e.errors)
        raise
```

### Catalog API

Manage items, categories, taxes, and modifiers.

**Create Catalog Item (Minimal):**

```python
import uuid

def create_catalog_item(name: str, price: int):
    try:
        item_id = f"#{name.replace(' ', '_')}"

        response = client.catalog.upsert_catalog_object(
            idempotency_key=str(uuid.uuid4()),
            object={
                "type": "ITEM",
                "id": item_id,
                "item_data": {
                    "name": name,
                    "variations": [
                        {
                            "type": "ITEM_VARIATION",
                            "id": f"{item_id}_variation",
                            "item_variation_data": {
                                "name": "Regular",
                                "pricing_type": "FIXED_PRICING",
                                "price_money": {
                                    "amount": price,
                                    "currency": "USD",
                                },
                            },
                        },
                    ],
                },
            },
        )
        return response.catalog_object
    except ApiError as e:
        print("Catalog item creation error:", e.errors)
        raise
```

**Create Catalog Item (Advanced):**

```python
import uuid

def create_advanced_catalog_item(item_data: dict):
    try:
        item_id = f"#{item_data['name'].replace(' ', '_')}"

        variations = [
            {
                "type": "ITEM_VARIATION",
                "id": f"{item_id}_var_{i}",
                "item_variation_data": {
                    "name": var["name"],
                    "sku": var.get("sku"),
                    "pricing_type": "FIXED_PRICING",
                    "price_money": {
                        "amount": var["price"],
                        "currency": var.get("currency", "USD"),
                    },
                    "track_inventory": var.get("track_inventory", False),
                },
            }
            for i, var in enumerate(item_data["variations"])
        ]

        response = client.catalog.upsert_catalog_object(
            idempotency_key=str(uuid.uuid4()),
            object={
                "type": "ITEM",
                "id": item_id,
                "item_data": {
                    "name": item_data["name"],
                    "description": item_data.get("description"),
                    "category_id": item_data.get("category_id"),
                    "tax_ids": item_data.get("tax_ids"),
                    "variations": variations,
                },
            },
        )
        return response.catalog_object
    except ApiError as e:
        print("Catalog item creation failed:", e.errors)
        raise
```

**List Catalog:**

```python
def list_catalog(types: list = None, cursor: str = None):
    try:
        if types is None:
            types = ["ITEM"]

        response = client.catalog.list(
            cursor=cursor,
            types=",".join(types),
        )
        return {
            "objects": response.objects,
            "cursor": response.cursor,
        }
    except ApiError as e:
        print("Error listing catalog:", e.errors)
        raise
```

**Retrieve Catalog Object:**

```python
def retrieve_catalog_object(object_id: str):
    try:
        response = client.catalog.retrieve(
            object_id=object_id,
            include_related_objects=True,
        )
        return response.object
    except ApiError as e:
        print("Error retrieving catalog object:", e.errors)
        raise
```

**Search Catalog Items:**

```python
def search_catalog_items(query: dict):
    try:
        response = client.catalog.search_catalog_items(
            text_filter=query.get("text"),
            category_ids=query.get("category_ids"),
            stock_levels=query.get("stock_levels"),
            enabled_location_ids=query.get("location_ids"),
            limit=query.get("limit", 100),
        )
        return response.items
    except ApiError as e:
        print("Error searching catalog:", e.errors)
        raise
```

**Delete Catalog Object:**

```python
def delete_catalog_object(object_id: str):
    try:
        response = client.catalog.delete_catalog_object(object_id)
        return response
    except ApiError as e:
        print("Error deleting catalog object:", e.errors)
        raise
```

**Batch Upsert Catalog Objects:**

```python
import uuid

def batch_upsert_catalog_objects(objects: list):
    try:
        response = client.catalog.batch_upsert_catalog_objects(
            idempotency_key=str(uuid.uuid4()),
            batches=[
                {
                    "objects": objects,
                },
            ],
        )
        return response.objects
    except ApiError as e:
        print("Batch upsert error:", e.errors)
        raise
```

### Inventory API

Track and manage inventory.

**Retrieve Inventory Count:**

```python
def retrieve_inventory_count(catalog_object_id: str, location_ids: list):
    try:
        response = client.inventory.retrieve_inventory_count(
            catalog_object_id=catalog_object_id,
            location_ids=",".join(location_ids),
        )
        return response.counts
    except ApiError as e:
        print("Error retrieving inventory:", e.errors)
        raise
```

**Batch Retrieve Inventory Counts:**

```python
def batch_retrieve_inventory_counts(catalog_object_ids: list, location_ids: list):
    try:
        response = client.inventory.batch_retrieve_inventory_counts(
            catalog_object_ids=catalog_object_ids,
            location_ids=location_ids,
        )
        return response.counts
    except ApiError as e:
        print("Error retrieving inventory counts:", e.errors)
        raise
```

**Adjust Inventory:**

```python
import uuid
from datetime import datetime

def adjust_inventory(catalog_object_id: str, location_id: str, adjustment: dict):
    try:
        response = client.inventory.batch_change_inventory(
            idempotency_key=str(uuid.uuid4()),
            changes=[
                {
                    "type": "ADJUSTMENT",
                    "adjustment": {
                        "catalog_object_id": catalog_object_id,
                        "location_id": location_id,
                        "quantity": str(adjustment["quantity"]),
                        "from_state": "IN_STOCK",
                        "to_state": "IN_STOCK",
                        "occurred_at": datetime.utcnow().isoformat(),
                    },
                },
            ],
        )
        return response.counts
    except ApiError as e:
        print("Error adjusting inventory:", e.errors)
        raise
```

### Invoices API

Create and manage invoices.

**Create Invoice (Minimal):**

```python
import uuid
from datetime import datetime, timedelta

def create_invoice(location_id: str, customer_id: str, order_id: str):
    try:
        due_date = (datetime.now() + timedelta(days=7)).strftime("%Y-%m-%d")

        response = client.invoices.create(
            invoice={
                "location_id": location_id,
                "order_id": order_id,
                "primary_recipient": {
                    "customer_id": customer_id,
                },
                "payment_requests": [
                    {
                        "request_type": "BALANCE",
                        "due_date": due_date,
                    },
                ],
            },
            idempotency_key=str(uuid.uuid4()),
        )
        return response.invoice
    except ApiError as e:
        print("Invoice creation error:", e.errors)
        raise
```

**Create Invoice (Advanced):**

```python
import uuid

def create_advanced_invoice(invoice_data: dict):
    try:
        payment_requests = [
            {
                "request_type": "BALANCE",
                "due_date": invoice_data["due_date"],
                "reminders": [
                    {
                        "relative_scheduled_days": -1,
                        "message": "Payment reminder",
                    },
                ],
            }
        ]

        if invoice_data.get("fixed_amount"):
            payment_requests[0]["fixed_amount_requested_money"] = {
                "amount": invoice_data["fixed_amount"],
                "currency": invoice_data.get("currency", "USD"),
            }

        response = client.invoices.create(
            invoice={
                "location_id": invoice_data["location_id"],
                "order_id": invoice_data["order_id"],
                "primary_recipient": {
                    "customer_id": invoice_data["customer_id"],
                    "given_name": invoice_data.get("given_name"),
                    "family_name": invoice_data.get("family_name"),
                    "email_address": invoice_data.get("email_address"),
                },
                "payment_requests": payment_requests,
                "delivery_method": "EMAIL",
                "invoice_number": invoice_data.get("invoice_number"),
                "title": invoice_data.get("title"),
                "description": invoice_data.get("description"),
            },
            idempotency_key=str(uuid.uuid4()),
        )
        return response.invoice
    except ApiError as e:
        print("Invoice creation failed:", e.errors)
        raise
```

**Publish Invoice:**

```python
def publish_invoice(invoice_id: str):
    try:
        response = client.invoices.publish(
            invoice_id=invoice_id,
            version=0,
        )
        return response.invoice
    except ApiError as e:
        print("Error publishing invoice:", e.errors)
        raise
```

**Get Invoice:**

```python
def get_invoice(invoice_id: str):
    try:
        response = client.invoices.retrieve(invoice_id)
        return response.invoice
    except ApiError as e:
        print("Error retrieving invoice:", e.errors)
        raise
```

**Search Invoices:**

```python
def search_invoices(location_ids: list, query: dict):
    try:
        search_query = {
            "location_ids": location_ids,
            "filter": {},
            "sort": {
                "field": "INVOICE_SORT_DATE",
                "order": "DESC",
            },
        }

        if query.get("customer_ids"):
            search_query["filter"]["customer_ids"] = query["customer_ids"]

        if query.get("states"):
            search_query["filter"]["state_filter"] = query["states"]

        response = client.invoices.search(
            query=search_query,
            limit=query.get("limit", 100),
        )
        return response.invoices
    except ApiError as e:
        print("Error searching invoices:", e.errors)
        raise
```

### Subscriptions API

Manage recurring payments and subscriptions.

**Create Subscription (Minimal):**

```python
import uuid

def create_subscription(location_id: str, customer_id: str, plan_id: str):
    try:
        response = client.subscriptions.create(
            idempotency_key=str(uuid.uuid4()),
            location_id=location_id,
            plan_id=plan_id,
            customer_id=customer_id,
        )
        return response.subscription
    except ApiError as e:
        print("Subscription creation error:", e.errors)
        raise
```

**Create Subscription (Advanced):**

```python
import uuid

def create_advanced_subscription(subscription_data: dict):
    try:
        price_override = None
        if subscription_data.get("price_override"):
            price_override = {
                "amount": subscription_data["price_override"],
                "currency": subscription_data.get("currency", "USD"),
            }

        response = client.subscriptions.create(
            idempotency_key=str(uuid.uuid4()),
            location_id=subscription_data["location_id"],
            plan_id=subscription_data["plan_id"],
            customer_id=subscription_data["customer_id"],
            start_date=subscription_data.get("start_date"),
            tax_percentage=subscription_data.get("tax_percentage"),
            price_override_money=price_override,
            card_id=subscription_data.get("card_id"),
        )
        return response.subscription
    except ApiError as e:
        print("Subscription creation failed:", e.errors)
        raise
```

**Retrieve Subscription:**

```python
def retrieve_subscription(subscription_id: str):
    try:
        response = client.subscriptions.retrieve(
            subscription_id=subscription_id,
            include="actions",
        )
        return response.subscription
    except ApiError as e:
        print("Error retrieving subscription:", e.errors)
        raise
```

**Cancel Subscription:**

```python
def cancel_subscription(subscription_id: str):
    try:
        response = client.subscriptions.cancel(subscription_id)
        return response.subscription
    except ApiError as e:
        print("Error canceling subscription:", e.errors)
        raise
```

**Search Subscriptions:**

```python
def search_subscriptions(query: dict):
    try:
        search_query = {
            "filter": {},
        }

        if query.get("customer_ids"):
            search_query["filter"]["customer_ids"] = query["customer_ids"]

        if query.get("location_ids"):
            search_query["filter"]["location_ids"] = query["location_ids"]

        response = client.subscriptions.search(
            query=search_query,
            limit=query.get("limit", 100),
        )
        return response.subscriptions
    except ApiError as e:
        print("Error searching subscriptions:", e.errors)
        raise
```

### Checkout API

Create hosted checkout pages.

**Create Checkout (Minimal):**

```python
import uuid

def create_checkout(location_id: str, order_id: str):
    try:
        response = client.checkout.create(
            location_id=location_id,
            idempotency_key=str(uuid.uuid4()),
            order={
                "order_id": order_id,
            },
        )
        return response.checkout.checkout_page_url
    except ApiError as e:
        print("Checkout creation error:", e.errors)
        raise
```

**Create Checkout (Advanced):**

```python
import uuid

def create_advanced_checkout(location_id: str, checkout_data: dict):
    try:
        response = client.checkout.create(
            location_id=location_id,
            idempotency_key=str(uuid.uuid4()),
            order={
                "order_id": checkout_data["order_id"],
                "location_id": location_id,
            },
            ask_for_shipping_address=checkout_data.get("ask_for_shipping_address", False),
            merchant_support_email=checkout_data.get("merchant_email"),
            pre_populate_buyer_email=checkout_data.get("buyer_email"),
            redirect_url=checkout_data.get("redirect_url"),
            additional_recipients=checkout_data.get("additional_recipients"),
        )
        return response.checkout
    except ApiError as e:
        print("Checkout creation failed:", e.errors)
        raise
```

### Bookings API

Manage appointments and bookings.

**Create Booking (Minimal):**

```python
import uuid

def create_booking(location_id: str, customer_id: str, start_at: str,
                   service_variation_id: str, team_member_id: str):
    try:
        response = client.bookings.create(
            idempotency_key=str(uuid.uuid4()),
            booking={
                "location_id": location_id,
                "customer_id": customer_id,
                "start_at": start_at,
                "appointment_segments": [
                    {
                        "duration_minutes": 60,
                        "service_variation_id": service_variation_id,
                        "team_member_id": team_member_id,
                    },
                ],
            },
        )
        return response.booking
    except ApiError as e:
        print("Booking creation error:", e.errors)
        raise
```

**Create Booking (Advanced):**

```python
import uuid

def create_advanced_booking(booking_data: dict):
    try:
        appointment_segments = [
            {
                "duration_minutes": segment["duration_minutes"],
                "service_variation_id": segment["service_variation_id"],
                "team_member_id": segment["team_member_id"],
            }
            for segment in booking_data["appointment_segments"]
        ]

        response = client.bookings.create(
            idempotency_key=str(uuid.uuid4()),
            booking={
                "location_id": booking_data["location_id"],
                "customer_id": booking_data["customer_id"],
                "customer_note": booking_data.get("customer_note"),
                "seller_note": booking_data.get("seller_note"),
                "start_at": booking_data["start_at"],
                "appointment_segments": appointment_segments,
            },
        )
        return response.booking
    except ApiError as e:
        print("Booking creation failed:", e.errors)
        raise
```

**List Bookings:**

```python
def list_bookings(location_id: str, start_at_min: str, start_at_max: str):
    try:
        response = client.bookings.list(
            location_id=location_id,
            start_at_min=start_at_min,
            start_at_max=start_at_max,
            limit=100,
        )
        return response.bookings
    except ApiError as e:
        print("Error listing bookings:", e.errors)
        raise
```

**Retrieve Booking:**

```python
def retrieve_booking(booking_id: str):
    try:
        response = client.bookings.retrieve(booking_id)
        return response.booking
    except ApiError as e:
        print("Error retrieving booking:", e.errors)
        raise
```

**Cancel Booking:**

```python
import uuid

def cancel_booking(booking_id: str):
    try:
        response = client.bookings.cancel(
            booking_id=booking_id,
            idempotency_key=str(uuid.uuid4()),
        )
        return response.booking
    except ApiError as e:
        print("Error canceling booking:", e.errors)
        raise
```

### Terminal API

Create checkouts for Square Terminal devices.

**Create Terminal Checkout:**

```python
import uuid

def create_terminal_checkout(device_id: str, amount: int, location_id: str):
    try:
        response = client.terminal.create_terminal_checkout(
            idempotency_key=str(uuid.uuid4()),
            checkout={
                "amount_money": {
                    "amount": amount,
                    "currency": "USD",
                },
                "device_options": {
                    "device_id": device_id,
                },
            },
        )
        return response.checkout
    except ApiError as e:
        print("Terminal checkout error:", e.errors)
        raise
```

**Get Terminal Checkout:**

```python
def get_terminal_checkout(checkout_id: str):
    try:
        response = client.terminal.get_terminal_checkout(checkout_id)
        return response.checkout
    except ApiError as e:
        print("Error retrieving terminal checkout:", e.errors)
        raise
```

**Cancel Terminal Checkout:**

```python
def cancel_terminal_checkout(checkout_id: str):
    try:
        response = client.terminal.cancel_terminal_checkout(checkout_id)
        return response.checkout
    except ApiError as e:
        print("Error canceling terminal checkout:", e.errors)
        raise
```

### Webhooks

Handle webhook notifications from Square.

**Create Webhook Subscription:**

```python
def create_webhook_subscription(notification_url: str, event_types: list):
    try:
        response = client.webhook_subscriptions.create(
            subscription={
                "name": "My Webhook",
                "notification_url": notification_url,
                "event_types": event_types,
            },
        )
        return response.subscription
    except ApiError as e:
        print("Webhook creation error:", e.errors)
        raise
```

**List Webhook Subscriptions:**

```python
def list_webhook_subscriptions():
    try:
        response = client.webhook_subscriptions.list()
        return response.subscriptions
    except ApiError as e:
        print("Error listing webhooks:", e.errors)
        raise
```

**Verify Webhook Signature:**

```python
import hmac
import hashlib

def verify_webhook_signature(body: str, signature: str, signature_key: str, notification_url: str) -> bool:
    message = notification_url + body
    hmac_obj = hmac.new(
        signature_key.encode('utf-8'),
        message.encode('utf-8'),
        hashlib.sha256
    )
    expected_signature = hmac_obj.digest().hex()

    return hmac.compare_digest(expected_signature, signature)
```

**Handle Webhook (Flask):**

```python
from flask import Flask, request

app = Flask(__name__)

@app.route('/webhooks/square', methods=['POST'])
def handle_square_webhook():
    signature = request.headers.get('x-square-hmacsha256-signature')
    body = request.get_data(as_text=True)

    is_valid = verify_webhook_signature(
        body,
        signature,
        os.environ.get('SQUARE_WEBHOOK_SECRET'),
        'https://yourdomain.com/webhooks/square'
    )

    if not is_valid:
        return 'Invalid signature', 400

    event = request.json

    if event['type'] == 'payment.created':
        print('Payment created:', event['data']['object']['payment'])
    elif event['type'] == 'order.created':
        print('Order created:', event['data']['object']['order'])
    elif event['type'] == 'customer.created':
        print('Customer created:', event['data']['object']['customer'])
    else:
        print('Unhandled event type:', event['type'])

    return 'OK', 200
```

### Common Webhook Events

```python
# Payment events
PAYMENT_CREATED = "payment.created"
PAYMENT_UPDATED = "payment.updated"

# Order events
ORDER_CREATED = "order.created"
ORDER_UPDATED = "order.updated"
ORDER_FULFILLMENT_UPDATED = "order.fulfillment.updated"

# Customer events
CUSTOMER_CREATED = "customer.created"
CUSTOMER_UPDATED = "customer.updated"
CUSTOMER_DELETED = "customer.deleted"

# Invoice events
INVOICE_CREATED = "invoice.created"
INVOICE_PUBLISHED = "invoice.published"
INVOICE_PAYMENT_MADE = "invoice.payment_made"

# Subscription events
SUBSCRIPTION_CREATED = "subscription.created"
SUBSCRIPTION_STARTED = "subscription.started"
SUBSCRIPTION_CANCELED = "subscription.canceled"

# Booking events
BOOKING_CREATED = "booking.created"
BOOKING_UPDATED = "booking.updated"

# Inventory events
INVENTORY_COUNT_UPDATED = "inventory.count.updated"

# Catalog events
CATALOG_VERSION_UPDATED = "catalog.version.updated"
```

### Pagination

Handle paginated results with auto-pagination.

```python
def get_all_customers():
    try:
        all_customers = []
        response = client.customers.list(limit=100)

        # Auto-pagination using iterator
        for customer in response:
            all_customers.append(customer)

        return all_customers
    except ApiError as e:
        print("Error fetching customers:", e.errors)
        raise
```

### Manual Pagination

```python
def get_all_customers_manual():
    all_customers = []
    cursor = None

    while True:
        try:
            response = client.customers.list(
                cursor=cursor,
                limit=100,
            )

            if response.customers:
                all_customers.extend(response.customers)

            cursor = response.cursor
            if not cursor:
                break

        except ApiError as e:
            print("Error fetching customers:", e.errors)
            raise

    return all_customers
```

### Idempotency

Use idempotency keys to prevent duplicate operations.

```python
import uuid

def idempotent_payment(source_id: str, amount: int, location_id: str):
    idempotency_key = str(uuid.uuid4())

    try:
        response = client.payments.create(
            source_id=source_id,
            idempotency_key=idempotency_key,
            amount_money={
                "amount": amount,
                "currency": "USD",
            },
            location_id=location_id,
        )
        return response.payment
    except ApiError as e:
        print("Payment failed:", e.errors)
        raise
```

### Testing with Sandbox

```python
import os
from square import Square
from square.environment import Environment

# Always use sandbox for development
client = Square(
    environment=Environment.SANDBOX,
    token=os.environ.get("SQUARE_SANDBOX_TOKEN"),
)

# Test payment source IDs for sandbox:
# cnon:card-nonce-ok - successful charge
# cnon:card-nonce-declined - declined charge
# cnon:card-nonce-dishonoured - card verification failed
```

### Complete Example: Process Payment with Customer

```python
import os
import uuid
from square import Square
from square.core.api_error import ApiError
from square.environment import Environment

client = Square(
    environment=Environment.SANDBOX,
    token=os.environ.get("SQUARE_TOKEN"),
)

def process_payment_with_customer(payment_details: dict):
    try:
        # 1. Create or retrieve customer
        customer = None
        try:
            customer_response = client.customers.create(
                idempotency_key=str(uuid.uuid4()),
                given_name=payment_details["given_name"],
                family_name=payment_details["family_name"],
                email_address=payment_details["email_address"],
            )
            customer = customer_response.customer
        except ApiError:
            print("Customer already exists, using existing")

        # 2. Create order
        order_response = client.orders.create(
            idempotency_key=str(uuid.uuid4()),
            order={
                "location_id": payment_details["location_id"],
                "customer_id": customer.id if customer else None,
                "line_items": [
                    {
                        "name": payment_details["item_name"],
                        "quantity": "1",
                        "base_price_money": {
                            "amount": payment_details["amount"],
                            "currency": "USD",
                        },
                    },
                ],
            },
        )

        # 3. Create payment
        payment_response = client.payments.create(
            source_id=payment_details["source_id"],
            idempotency_key=str(uuid.uuid4()),
            amount_money={
                "amount": payment_details["amount"],
                "currency": "USD",
            },
            order_id=order_response.order.id,
            customer_id=customer.id if customer else None,
            location_id=payment_details["location_id"],
            autocomplete=True,
        )

        return {
            "payment": payment_response.payment,
            "order": order_response.order,
            "customer": customer,
        }

    except ApiError as e:
        print("Transaction failed:")
        for error in e.errors:
            print(f"  {error.category}: {error.code}")
            print(f"  {error.detail}")
        raise

# Usage
result = process_payment_with_customer({
    "source_id": "cnon:card-nonce-ok",
    "amount": 1000,
    "location_id": "L88917AVBK2S5",
    "given_name": "John",
    "family_name": "Doe",
    "email_address": "john.doe@example.com",
    "item_name": "Premium Service",
})

print("Payment successful!")
print(f"Payment ID: {result['payment'].id}")
print(f"Order ID: {result['order'].id}")
if result['customer']:
    print(f"Customer ID: {result['customer'].id}")
```
