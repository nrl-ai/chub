---
name: crm
description: "HubSpot Python SDK for managing CRM contacts, companies, deals, and marketing automation via the HubSpot API."
metadata:
  languages: "python"
  versions: "12.0.0"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "hubspot,crm,marketing,contacts,automation"
---

# HubSpot Python SDK Coding Guidelines

You are a HubSpot API coding expert. Help me write code using the HubSpot API with the official Python SDK.

Official SDK documentation and code samples:
https://developers.hubspot.com/docs/api/overview

## Golden Rule: Use the Correct and Current SDK

Always use the official HubSpot Python SDK (`hubspot-api-client`), which is the standard library for all HubSpot API interactions. Do not use deprecated or unofficial packages.

- **Library Name:** HubSpot Python SDK
- **PyPI Package:** `hubspot-api-client`
- **Current Version:** 12.0.0
- **Deprecated Packages:** `hubspot-python`, `hubspot3` (do not use these)

**Installation:**

- **Correct:** `pip install hubspot-api-client`
- **Incorrect:** `pip install hubspot-python` or `pip install hubspot3`

**APIs and Usage:**

- **Correct:** `from hubspot import HubSpot`
- **Correct:** `api_client = HubSpot(access_token=token)`
- **Correct:** `api_client.crm.contacts.basic_api.create(...)`
- **Incorrect:** `HubspotClient` or `HubspotAPI`
- **Incorrect:** Legacy v2 API endpoints
- **Incorrect:** Using `hapikey` (removed after v5.1.0)

## Installation

Install the SDK via pip:

```bash
pip install hubspot-api-client
```

The SDK requires Python 3.9 or higher.

For development with additional dependencies:

```bash
pip install hubspot-api-client[dev]
```

## Authentication

HubSpot API uses access tokens for authentication. There are three types of access tokens:

1. **Private App Access Tokens** (recommended for server-side integrations)
2. **OAuth Access Tokens** (for public apps and integrations)
3. **Legacy API Keys** (deprecated, will be revoked November 19, 2025)

### Environment Variable Configuration

Set your access token as an environment variable:

```bash
# .env file
HUBSPOT_ACCESS_TOKEN=your_access_token_here
```

**Creating a Private App:**

1. Navigate to Settings > Integrations > Private Apps in your HubSpot account
2. Click "Create private app"
3. Configure the required scopes (permissions)
4. Click "Create app" and copy the access token

## Initialization

### Basic Initialization

```python
from hubspot import HubSpot
import os

# Initialize with access token from environment variable
api_client = HubSpot(access_token=os.getenv('HUBSPOT_ACCESS_TOKEN'))
```

### Initialization with OAuth

```python
from hubspot import HubSpot

api_client = HubSpot(access_token='YOUR_OAUTH_ACCESS_TOKEN')

# Update token when refreshed
api_client.access_token = 'NEW_ACCESS_TOKEN'
```

### Alternative Initialization Methods

```python
from hubspot import HubSpot

# Option 1: Set token during initialization
api_client = HubSpot(access_token='your_access_token')

# Option 2: Set token after initialization
api_client = HubSpot()
api_client.access_token = 'your_access_token'

# Option 3: Using Client.create method
import hubspot
api_client = hubspot.Client.create(access_token='your_access_token')
```

## CRM API - Contacts

### Create a Contact

**Basic Example:**

```python
from hubspot import HubSpot
from hubspot.crm.contacts import SimplePublicObjectInputForCreate, ApiException
import os

api_client = HubSpot(access_token=os.getenv('HUBSPOT_ACCESS_TOKEN'))

def create_contact():
    properties = {
        "email": "example@company.com",
        "firstname": "John",
        "lastname": "Doe",
        "phone": "555-0100",
        "company": "Example Company",
        "website": "example.com"
    }

    simple_public_object_input_for_create = SimplePublicObjectInputForCreate(
        properties=properties
    )

    try:
        api_response = api_client.crm.contacts.basic_api.create(
            simple_public_object_input_for_create=simple_public_object_input_for_create
        )
        print(f"Contact created with ID: {api_response.id}")
        return api_response
    except ApiException as e:
        print(f"Exception when creating contact: {e}")

create_contact()
```

**Advanced Example with Custom Properties:**

```python
from hubspot.crm.contacts import SimplePublicObjectInputForCreate, ApiException

def create_contact_advanced():
    properties = {
        "email": "jane@example.com",
        "firstname": "Jane",
        "lastname": "Smith",
        "phone": "555-0200",
        "company": "Tech Corp",
        "jobtitle": "Software Engineer",
        "lifecyclestage": "lead",
        "hs_lead_status": "NEW",
        # Custom properties
        "custom_field": "custom_value",
        "industry": "Technology"
    }

    simple_public_object_input = SimplePublicObjectInputForCreate(properties=properties)

    try:
        api_response = api_client.crm.contacts.basic_api.create(
            simple_public_object_input_for_create=simple_public_object_input
        )
        print(f"Contact created with ID: {api_response.id}")
        print(f"Properties: {api_response.properties}")
        return api_response
    except ApiException as e:
        if e.status == 409:
            print("Contact with this email already exists")
        else:
            print(f"Exception when creating contact: {e}")

create_contact_advanced()
```

### Get a Contact

**By ID:**

```python
from hubspot.crm.contacts import ApiException

def get_contact(contact_id):
    properties = ["email", "firstname", "lastname", "phone", "company"]

    try:
        api_response = api_client.crm.contacts.basic_api.get_by_id(
            contact_id=contact_id,
            properties=properties
        )
        print(f"Contact: {api_response.properties}")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching contact: {e}")

get_contact("12345")
```

**By Email:**

```python
from hubspot.crm.contacts import PublicObjectSearchRequest, ApiException

def get_contact_by_email(email):
    filter_obj = {
        "propertyName": "email",
        "operator": "EQ",
        "value": email
    }

    filter_group = {
        "filters": [filter_obj]
    }

    sort = {
        "propertyName": "createdate",
        "direction": "DESCENDING"
    }

    public_object_search_request = PublicObjectSearchRequest(
        filter_groups=[filter_group],
        sorts=[sort],
        properties=["email", "firstname", "lastname"],
        limit=1
    )

    try:
        api_response = api_client.crm.contacts.search_api.do_search(
            public_object_search_request=public_object_search_request
        )

        if api_response.total > 0:
            return api_response.results[0]
        return None
    except ApiException as e:
        print(f"Exception when searching contact: {e}")

get_contact_by_email("example@company.com")
```

### Update a Contact

```python
from hubspot.crm.contacts import SimplePublicObjectInput, ApiException

def update_contact(contact_id):
    properties = {
        "firstname": "Updated Name",
        "phone": "555-9999",
        "lifecyclestage": "opportunity"
    }

    simple_public_object_input = SimplePublicObjectInput(properties=properties)

    try:
        api_response = api_client.crm.contacts.basic_api.update(
            contact_id=contact_id,
            simple_public_object_input=simple_public_object_input
        )
        print(f"Contact updated: {api_response.id}")
        return api_response
    except ApiException as e:
        print(f"Exception when updating contact: {e}")

update_contact("12345")
```

### List Contacts

**Basic Pagination:**

```python
from hubspot.crm.contacts import ApiException

def list_contacts():
    limit = 10
    properties = ["email", "firstname", "lastname", "company"]

    try:
        api_response = api_client.crm.contacts.basic_api.get_page(
            limit=limit,
            properties=properties
        )

        print(f"Found {len(api_response.results)} contacts")
        for contact in api_response.results:
            print(f"{contact.properties.get('firstname')} {contact.properties.get('lastname')}")

        return api_response
    except ApiException as e:
        print(f"Exception when listing contacts: {e}")

list_contacts()
```

**Get All Contacts (with pagination handling):**

```python
from hubspot.crm.contacts import ApiException

def get_all_contacts():
    all_contacts = []
    after = None
    limit = 100
    properties = ["email", "firstname", "lastname"]

    try:
        while True:
            api_response = api_client.crm.contacts.basic_api.get_page(
                limit=limit,
                after=after,
                properties=properties
            )

            all_contacts.extend(api_response.results)

            if api_response.paging and api_response.paging.next:
                after = api_response.paging.next.after
            else:
                break

        print(f"Total contacts retrieved: {len(all_contacts)}")
        return all_contacts
    except ApiException as e:
        print(f"Exception when getting all contacts: {e}")

get_all_contacts()
```

**Using get_all method:**

```python
from hubspot.crm.contacts import ApiException

def get_all_contacts_simple():
    try:
        # The get_all method handles pagination automatically
        all_contacts = api_client.crm.contacts.get_all()
        print(f"Total contacts: {len(all_contacts)}")
        return all_contacts
    except ApiException as e:
        print(f"Exception when getting all contacts: {e}")

get_all_contacts_simple()
```

### Delete a Contact

```python
from hubspot.crm.contacts import ApiException

def delete_contact(contact_id):
    try:
        api_client.crm.contacts.basic_api.archive(contact_id=contact_id)
        print(f"Contact deleted: {contact_id}")
    except ApiException as e:
        print(f"Exception when deleting contact: {e}")

delete_contact("12345")
```

## CRM API - Companies

### Create a Company

```python
from hubspot.crm.companies import SimplePublicObjectInputForCreate, ApiException

def create_company():
    properties = {
        "name": "Example Company",
        "domain": "example.com",
        "city": "San Francisco",
        "state": "California",
        "industry": "Technology",
        "phone": "555-0100",
        "numberofemployees": "50",
        "description": "A technology company"
    }

    simple_public_object_input = SimplePublicObjectInputForCreate(properties=properties)

    try:
        api_response = api_client.crm.companies.basic_api.create(
            simple_public_object_input_for_create=simple_public_object_input
        )
        print(f"Company created with ID: {api_response.id}")
        return api_response
    except ApiException as e:
        print(f"Exception when creating company: {e}")

create_company()
```

### Get a Company

```python
from hubspot.crm.companies import ApiException

def get_company(company_id):
    properties = ["name", "domain", "city", "industry", "phone"]

    try:
        api_response = api_client.crm.companies.basic_api.get_by_id(
            company_id=company_id,
            properties=properties
        )
        print(f"Company: {api_response.properties}")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching company: {e}")

get_company("12345")
```

### Update a Company

```python
from hubspot.crm.companies import SimplePublicObjectInput, ApiException

def update_company(company_id):
    properties = {
        "name": "Updated Company Name",
        "numberofemployees": "100",
        "annualrevenue": "1000000"
    }

    simple_public_object_input = SimplePublicObjectInput(properties=properties)

    try:
        api_response = api_client.crm.companies.basic_api.update(
            company_id=company_id,
            simple_public_object_input=simple_public_object_input
        )
        print(f"Company updated: {api_response.id}")
        return api_response
    except ApiException as e:
        print(f"Exception when updating company: {e}")

update_company("12345")
```

### List Companies

```python
from hubspot.crm.companies import ApiException

def list_companies():
    limit = 10
    properties = ["name", "domain", "city", "industry"]

    try:
        api_response = api_client.crm.companies.basic_api.get_page(
            limit=limit,
            properties=properties
        )

        print(f"Found {len(api_response.results)} companies")
        return api_response
    except ApiException as e:
        print(f"Exception when listing companies: {e}")

list_companies()
```

## CRM API - Deals

### Create a Deal

```python
from hubspot.crm.deals import SimplePublicObjectInputForCreate, ApiException

def create_deal():
    properties = {
        "dealname": "New Deal",
        "dealstage": "appointmentscheduled",
        "amount": "10000",
        "closedate": "2025-12-31",
        "pipeline": "default",
        "hubspot_owner_id": "12345"
    }

    simple_public_object_input = SimplePublicObjectInputForCreate(properties=properties)

    try:
        api_response = api_client.crm.deals.basic_api.create(
            simple_public_object_input_for_create=simple_public_object_input
        )
        print(f"Deal created with ID: {api_response.id}")
        return api_response
    except ApiException as e:
        print(f"Exception when creating deal: {e}")

create_deal()
```

### Get a Deal

```python
from hubspot.crm.deals import ApiException

def get_deal(deal_id):
    properties = ["dealname", "dealstage", "amount", "closedate"]

    try:
        api_response = api_client.crm.deals.basic_api.get_by_id(
            deal_id=deal_id,
            properties=properties
        )
        print(f"Deal: {api_response.properties}")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching deal: {e}")

get_deal("12345")
```

### Update a Deal

```python
from hubspot.crm.deals import SimplePublicObjectInput, ApiException

def update_deal(deal_id):
    properties = {
        "dealstage": "closedwon",
        "amount": "15000",
        "closedate": "2025-11-30"
    }

    simple_public_object_input = SimplePublicObjectInput(properties=properties)

    try:
        api_response = api_client.crm.deals.basic_api.update(
            deal_id=deal_id,
            simple_public_object_input=simple_public_object_input
        )
        print(f"Deal updated: {api_response.id}")
        return api_response
    except ApiException as e:
        print(f"Exception when updating deal: {e}")

update_deal("12345")
```

### List Deals

```python
from hubspot.crm.deals import ApiException

def list_deals():
    limit = 10
    properties = ["dealname", "dealstage", "amount"]

    try:
        api_response = api_client.crm.deals.basic_api.get_page(
            limit=limit,
            properties=properties
        )

        print(f"Found {len(api_response.results)} deals")
        return api_response
    except ApiException as e:
        print(f"Exception when listing deals: {e}")

list_deals()
```

## CRM API - Tickets

### Create a Ticket

```python
from hubspot.crm.tickets import SimplePublicObjectInputForCreate, ApiException

def create_ticket():
    properties = {
        "subject": "Customer Support Request",
        "content": "Customer needs help with product setup",
        "hs_pipeline": "0",
        "hs_pipeline_stage": "1",
        "hs_ticket_priority": "HIGH",
        "hubspot_owner_id": "12345"
    }

    simple_public_object_input = SimplePublicObjectInputForCreate(properties=properties)

    try:
        api_response = api_client.crm.tickets.basic_api.create(
            simple_public_object_input_for_create=simple_public_object_input
        )
        print(f"Ticket created with ID: {api_response.id}")
        return api_response
    except ApiException as e:
        print(f"Exception when creating ticket: {e}")

create_ticket()
```

### Get a Ticket

```python
from hubspot.crm.tickets import ApiException

def get_ticket(ticket_id):
    properties = ["subject", "content", "hs_pipeline_stage", "hs_ticket_priority"]

    try:
        api_response = api_client.crm.tickets.basic_api.get_by_id(
            ticket_id=ticket_id,
            properties=properties
        )
        print(f"Ticket: {api_response.properties}")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching ticket: {e}")

get_ticket("12345")
```

### Update a Ticket

```python
from hubspot.crm.tickets import SimplePublicObjectInput, ApiException

def update_ticket(ticket_id):
    properties = {
        "hs_pipeline_stage": "4",  # Closed
        "hs_ticket_priority": "LOW"
    }

    simple_public_object_input = SimplePublicObjectInput(properties=properties)

    try:
        api_response = api_client.crm.tickets.basic_api.update(
            ticket_id=ticket_id,
            simple_public_object_input=simple_public_object_input
        )
        print(f"Ticket updated: {api_response.id}")
        return api_response
    except ApiException as e:
        print(f"Exception when updating ticket: {e}")

update_ticket("12345")
```

## CRM API - Associations

Associations link objects together (e.g., contacts to companies, deals to contacts).

### Create Association

**Associate Contact with Company:**

```python
from hubspot.crm.associations import ApiException

def associate_contact_with_company(contact_id, company_id):
    try:
        api_response = api_client.crm.contacts.associations_api.create(
            contact_id=contact_id,
            to_object_type="companies",
            to_object_id=company_id,
            association_type="contact_to_company"
        )
        print("Association created")
        return api_response
    except ApiException as e:
        print(f"Exception when creating association: {e}")

associate_contact_with_company("12345", "67890")
```

**Associate Deal with Contact:**

```python
from hubspot.crm.associations import ApiException

def associate_deal_with_contact(deal_id, contact_id):
    try:
        api_response = api_client.crm.deals.associations_api.create(
            deal_id=deal_id,
            to_object_type="contacts",
            to_object_id=contact_id,
            association_type="deal_to_contact"
        )
        print("Association created")
        return api_response
    except ApiException as e:
        print(f"Exception when creating association: {e}")

associate_deal_with_contact("12345", "67890")
```

### Get Associations

```python
from hubspot.crm.associations import ApiException

def get_contact_associations(contact_id):
    try:
        # Get associated companies
        companies = api_client.crm.contacts.associations_api.get_all(
            contact_id=contact_id,
            to_object_type="companies"
        )

        # Get associated deals
        deals = api_client.crm.contacts.associations_api.get_all(
            contact_id=contact_id,
            to_object_type="deals"
        )

        print(f"Associated companies: {companies.results}")
        print(f"Associated deals: {deals.results}")

        return {"companies": companies, "deals": deals}
    except ApiException as e:
        print(f"Exception when fetching associations: {e}")

get_contact_associations("12345")
```

### Remove Association

```python
from hubspot.crm.associations import ApiException

def remove_association(contact_id, company_id):
    try:
        api_client.crm.contacts.associations_api.archive(
            contact_id=contact_id,
            to_object_type="companies",
            to_object_id=company_id,
            association_type="contact_to_company"
        )
        print("Association removed")
    except ApiException as e:
        print(f"Exception when removing association: {e}")

remove_association("12345", "67890")
```

## CRM API - Batch Operations

Batch operations allow you to create, update, or read up to 100 records in a single API call.

### Batch Create Contacts

```python
from hubspot.crm.contacts import BatchInputSimplePublicObjectInputForCreate, SimplePublicObjectInputForCreate, ApiException

def batch_create_contacts():
    inputs = [
        SimplePublicObjectInputForCreate(
            properties={
                "email": "contact1@example.com",
                "firstname": "Contact",
                "lastname": "One"
            }
        ),
        SimplePublicObjectInputForCreate(
            properties={
                "email": "contact2@example.com",
                "firstname": "Contact",
                "lastname": "Two"
            }
        ),
        SimplePublicObjectInputForCreate(
            properties={
                "email": "contact3@example.com",
                "firstname": "Contact",
                "lastname": "Three"
            }
        )
    ]

    batch_input = BatchInputSimplePublicObjectInputForCreate(inputs=inputs)

    try:
        api_response = api_client.crm.contacts.batch_api.create(
            batch_input_simple_public_object_input_for_create=batch_input
        )
        print(f"Created {len(api_response.results)} contacts")
        return api_response
    except ApiException as e:
        print(f"Exception when batch creating contacts: {e}")

batch_create_contacts()
```

### Batch Update Contacts

```python
from hubspot.crm.contacts import BatchInputSimplePublicObjectBatchInput, SimplePublicObjectBatchInput, ApiException

def batch_update_contacts(contact_ids):
    inputs = []
    for contact_id in contact_ids:
        inputs.append(
            SimplePublicObjectBatchInput(
                id=contact_id,
                properties={
                    "lifecyclestage": "customer",
                    "hs_lead_status": "CONNECTED"
                }
            )
        )

    batch_input = BatchInputSimplePublicObjectBatchInput(inputs=inputs)

    try:
        api_response = api_client.crm.contacts.batch_api.update(
            batch_input_simple_public_object_batch_input=batch_input
        )
        print(f"Updated {len(api_response.results)} contacts")
        return api_response
    except ApiException as e:
        print(f"Exception when batch updating contacts: {e}")

batch_update_contacts(["12345", "67890", "11111"])
```

### Batch Read Contacts

```python
from hubspot.crm.contacts import BatchReadInputSimplePublicObjectId, SimplePublicObjectId, ApiException

def batch_read_contacts(contact_ids):
    inputs = [SimplePublicObjectId(id=contact_id) for contact_id in contact_ids]

    batch_input = BatchReadInputSimplePublicObjectId(
        properties=["email", "firstname", "lastname", "phone"],
        inputs=inputs
    )

    try:
        api_response = api_client.crm.contacts.batch_api.read(
            batch_read_input_simple_public_object_id=batch_input
        )
        print(f"Retrieved {len(api_response.results)} contacts")
        return api_response
    except ApiException as e:
        print(f"Exception when batch reading contacts: {e}")

batch_read_contacts(["12345", "67890", "11111"])
```

### Batch Upsert Contacts

```python
from hubspot.crm.contacts import BatchInputSimplePublicObjectInputForCreate, SimplePublicObjectInputForCreate, ApiException

def batch_upsert_contacts():
    inputs = [
        SimplePublicObjectInputForCreate(
            properties={
                "email": "existing@example.com",
                "firstname": "Updated",
                "lastname": "Name"
            },
            id_property="email"
        ),
        SimplePublicObjectInputForCreate(
            properties={
                "email": "new@example.com",
                "firstname": "New",
                "lastname": "Contact"
            },
            id_property="email"
        )
    ]

    batch_input = BatchInputSimplePublicObjectInputForCreate(inputs=inputs)

    try:
        api_response = api_client.crm.contacts.batch_api.upsert(
            batch_input_simple_public_object_input_for_create=batch_input
        )
        print(f"Upserted {len(api_response.results)} contacts")
        return api_response
    except ApiException as e:
        print(f"Exception when batch upserting contacts: {e}")

batch_upsert_contacts()
```

## CRM API - Search

The Search API allows you to filter, sort, and search across CRM objects.

### Basic Search

```python
from hubspot.crm.contacts import PublicObjectSearchRequest, ApiException

def search_contacts():
    filter_obj = {
        "propertyName": "lifecyclestage",
        "operator": "EQ",
        "value": "lead"
    }

    filter_group = {
        "filters": [filter_obj]
    }

    public_object_search_request = PublicObjectSearchRequest(
        filter_groups=[filter_group],
        properties=["email", "firstname", "lastname", "lifecyclestage"],
        limit=10
    )

    try:
        api_response = api_client.crm.contacts.search_api.do_search(
            public_object_search_request=public_object_search_request
        )
        print(f"Found {api_response.total} contacts")
        print(f"Returned {len(api_response.results)} results")
        return api_response
    except ApiException as e:
        print(f"Exception when searching contacts: {e}")

search_contacts()
```

### Advanced Search with Multiple Filters

```python
from hubspot.crm.contacts import PublicObjectSearchRequest, ApiException

def advanced_search_contacts():
    filter_group = {
        "filters": [
            {
                "propertyName": "lifecyclestage",
                "operator": "EQ",
                "value": "lead"
            },
            {
                "propertyName": "createdate",
                "operator": "GTE",
                "value": "2025-01-01"
            }
        ]
    }

    sort = {
        "propertyName": "createdate",
        "direction": "DESCENDING"
    }

    public_object_search_request = PublicObjectSearchRequest(
        filter_groups=[filter_group],
        sorts=[sort],
        properties=["email", "firstname", "lastname", "createdate", "lifecyclestage"],
        limit=100,
        after=0
    )

    try:
        api_response = api_client.crm.contacts.search_api.do_search(
            public_object_search_request=public_object_search_request
        )
        print(f"Found {api_response.total} matching contacts")
        return api_response
    except ApiException as e:
        print(f"Exception when searching contacts: {e}")

advanced_search_contacts()
```

### Search with Association Filters

```python
from hubspot.crm.contacts import PublicObjectSearchRequest, ApiException

def search_contacts_by_company(company_id):
    filter_obj = {
        "propertyName": "associations.company",
        "operator": "EQ",
        "value": company_id
    }

    filter_group = {
        "filters": [filter_obj]
    }

    public_object_search_request = PublicObjectSearchRequest(
        filter_groups=[filter_group],
        properties=["email", "firstname", "lastname"],
        limit=100
    )

    try:
        api_response = api_client.crm.contacts.search_api.do_search(
            public_object_search_request=public_object_search_request
        )
        print(f"Found {len(api_response.results)} contacts for company")
        return api_response
    except ApiException as e:
        print(f"Exception when searching contacts by company: {e}")

search_contacts_by_company("12345")
```

### Search Operators

Available operators for search filters:

- `EQ` - Equal to
- `NEQ` - Not equal to
- `LT` - Less than
- `LTE` - Less than or equal to
- `GT` - Greater than
- `GTE` - Greater than or equal to
- `IN` - In list of values
- `NOT_IN` - Not in list of values
- `HAS_PROPERTY` - Has property value set
- `NOT_HAS_PROPERTY` - Does not have property value set
- `CONTAINS_TOKEN` - Contains token (for text)
- `NOT_CONTAINS_TOKEN` - Does not contain token

## CRM API - Properties

### Get All Contact Properties

```python
from hubspot.crm.properties import ApiException

def get_all_contact_properties():
    try:
        api_response = api_client.crm.properties.core_api.get_all(
            object_type="contacts"
        )
        print(f"Found {len(api_response.results)} contact properties")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching contact properties: {e}")

get_all_contact_properties()
```

### Create Custom Property

```python
from hubspot.crm.properties import PropertyCreate, ApiException

def create_custom_property():
    property_create = PropertyCreate(
        name="favorite_color",
        label="Favorite Color",
        type="string",
        field_type="text",
        group_name="contactinformation",
        description="The contact's favorite color"
    )

    try:
        api_response = api_client.crm.properties.core_api.create(
            object_type="contacts",
            property_create=property_create
        )
        print(f"Custom property created: {api_response.name}")
        return api_response
    except ApiException as e:
        print(f"Exception when creating custom property: {e}")

create_custom_property()
```

### Create Dropdown Property

```python
from hubspot.crm.properties import PropertyCreate, OptionInput, ApiException

def create_dropdown_property():
    options = [
        OptionInput(label="Bronze", value="bronze", display_order=0),
        OptionInput(label="Silver", value="silver", display_order=1),
        OptionInput(label="Gold", value="gold", display_order=2),
        OptionInput(label="Platinum", value="platinum", display_order=3)
    ]

    property_create = PropertyCreate(
        name="customer_tier",
        label="Customer Tier",
        type="enumeration",
        field_type="select",
        group_name="contactinformation",
        options=options
    )

    try:
        api_response = api_client.crm.properties.core_api.create(
            object_type="contacts",
            property_create=property_create
        )
        print(f"Dropdown property created: {api_response.name}")
        return api_response
    except ApiException as e:
        print(f"Exception when creating dropdown property: {e}")

create_dropdown_property()
```

### Update Property

```python
from hubspot.crm.properties import PropertyUpdate, ApiException

def update_property(property_name):
    property_update = PropertyUpdate(
        label="Updated Label",
        description="Updated description"
    )

    try:
        api_response = api_client.crm.properties.core_api.update(
            object_type="contacts",
            property_name=property_name,
            property_update=property_update
        )
        print(f"Property updated: {api_response.name}")
        return api_response
    except ApiException as e:
        print(f"Exception when updating property: {e}")

update_property("favorite_color")
```

## Marketing API - Emails

### Get All Marketing Emails

```python
from hubspot.marketing.emails import ApiException

def get_marketing_emails():
    try:
        api_response = api_client.marketing.emails.emails_api.get_page()
        print(f"Found {len(api_response.results)} marketing emails")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching marketing emails: {e}")

get_marketing_emails()
```

### Get Email by ID

```python
from hubspot.marketing.emails import ApiException

def get_marketing_email(email_id):
    try:
        api_response = api_client.marketing.emails.emails_api.get_by_id(
            email_id=email_id
        )
        print(f"Email: {api_response.name}")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching marketing email: {e}")

get_marketing_email("12345")
```

## Marketing API - Forms

### Get All Forms

```python
from hubspot.marketing.forms import ApiException

def get_all_forms():
    try:
        api_response = api_client.marketing.forms.forms_api.get_page()
        print(f"Found {len(api_response.results)} forms")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching forms: {e}")

get_all_forms()
```

### Get Form by ID

```python
from hubspot.marketing.forms import ApiException

def get_form(form_id):
    try:
        api_response = api_client.marketing.forms.forms_api.get_by_id(
            form_id=form_id
        )
        print(f"Form: {api_response.name}")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching form: {e}")

get_form("12345")
```

### Submit Form Data

```python
import requests

def submit_form_data(form_guid, portal_id):
    form_data = {
        "fields": [
            {
                "name": "email",
                "value": "test@example.com"
            },
            {
                "name": "firstname",
                "value": "John"
            },
            {
                "name": "lastname",
                "value": "Doe"
            }
        ],
        "context": {
            "pageUri": "https://example.com/contact",
            "pageName": "Contact Us"
        }
    }

    try:
        response = requests.post(
            f"https://api.hsforms.com/submissions/v3/integration/submit/{portal_id}/{form_guid}",
            json=form_data
        )
        response.raise_for_status()
        print("Form submitted successfully")
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error submitting form: {e}")

submit_form_data("form-guid-here", "12345")
```

## Events API - Timeline Events

### Create Timeline Event

```python
import requests
import os

def create_timeline_event(event_type_id, object_id):
    event_data = {
        "eventTypeId": event_type_id,
        "objectId": object_id,
        "extraData": {
            "eventTitle": "Purchase Completed",
            "eventDescription": "Customer completed a purchase of $99.99"
        },
        "timestamp": "2025-11-07T12:00:00Z"
    }

    headers = {
        "Authorization": f"Bearer {os.getenv('HUBSPOT_ACCESS_TOKEN')}",
        "Content-Type": "application/json"
    }

    try:
        response = requests.post(
            "https://api.hubapi.com/crm/v3/timeline/events",
            json=event_data,
            headers=headers
        )
        response.raise_for_status()
        print(f"Timeline event created: {response.json()['id']}")
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error creating timeline event: {e}")

create_timeline_event("event_type_id_here", "12345")
```

## Automation API - Workflows

### Get All Workflows

```python
import requests
import os

def get_all_workflows():
    headers = {
        "Authorization": f"Bearer {os.getenv('HUBSPOT_ACCESS_TOKEN')}"
    }

    try:
        response = requests.get(
            "https://api.hubapi.com/automation/v3/workflows",
            headers=headers
        )
        response.raise_for_status()
        workflows = response.json()['workflows']
        print(f"Found {len(workflows)} workflows")
        return workflows
    except requests.exceptions.RequestException as e:
        print(f"Error fetching workflows: {e}")

get_all_workflows()
```

### Get Workflow by ID

```python
import requests
import os

def get_workflow(workflow_id):
    headers = {
        "Authorization": f"Bearer {os.getenv('HUBSPOT_ACCESS_TOKEN')}"
    }

    try:
        response = requests.get(
            f"https://api.hubapi.com/automation/v3/workflows/{workflow_id}",
            headers=headers
        )
        response.raise_for_status()
        workflow = response.json()
        print(f"Workflow: {workflow['name']}")
        return workflow
    except requests.exceptions.RequestException as e:
        print(f"Error fetching workflow: {e}")

get_workflow("12345")
```

### Enroll Contact in Workflow

```python
import requests
import os

def enroll_contact_in_workflow(workflow_id, contact_email):
    headers = {
        "Authorization": f"Bearer {os.getenv('HUBSPOT_ACCESS_TOKEN')}",
        "Content-Type": "application/json"
    }

    try:
        response = requests.post(
            f"https://api.hubapi.com/automation/v2/workflows/{workflow_id}/enrollments/contacts/{contact_email}",
            headers=headers,
            json={}
        )
        response.raise_for_status()
        print("Contact enrolled in workflow")
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error enrolling contact in workflow: {e}")

enroll_contact_in_workflow("12345", "contact@example.com")
```

## Webhooks API

### Create Webhook Subscription

```python
from hubspot.webhooks import SubscriptionCreateRequest, ApiException

def create_webhook_subscription(app_id):
    subscription_create = SubscriptionCreateRequest(
        event_type="contact.creation",
        active=True
    )

    try:
        api_response = api_client.webhooks.subscriptions_api.create(
            app_id=app_id,
            subscription_create_request=subscription_create
        )
        print(f"Webhook subscription created: {api_response.id}")
        return api_response
    except ApiException as e:
        print(f"Exception when creating webhook subscription: {e}")

create_webhook_subscription(12345)
```

### Get All Webhook Subscriptions

```python
from hubspot.webhooks import ApiException

def get_webhook_subscriptions(app_id):
    try:
        api_response = api_client.webhooks.subscriptions_api.get_all(
            app_id=app_id
        )
        print(f"Found {len(api_response.results)} webhook subscriptions")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching webhook subscriptions: {e}")

get_webhook_subscriptions(12345)
```

### Delete Webhook Subscription

```python
from hubspot.webhooks import ApiException

def delete_webhook_subscription(app_id, subscription_id):
    try:
        api_client.webhooks.subscriptions_api.archive(
            subscription_id=subscription_id,
            app_id=app_id
        )
        print("Webhook subscription deleted")
    except ApiException as e:
        print(f"Exception when deleting webhook subscription: {e}")

delete_webhook_subscription(12345, 67890)
```

## Lists API

### Get All Lists

```python
import requests
import os

def get_all_lists():
    headers = {
        "Authorization": f"Bearer {os.getenv('HUBSPOT_ACCESS_TOKEN')}"
    }

    params = {
        "count": 100,
        "offset": 0
    }

    try:
        response = requests.get(
            "https://api.hubapi.com/contacts/v1/lists",
            headers=headers,
            params=params
        )
        response.raise_for_status()
        lists = response.json()['lists']
        print(f"Found {len(lists)} lists")
        return lists
    except requests.exceptions.RequestException as e:
        print(f"Error fetching lists: {e}")

get_all_lists()
```

### Add Contact to List

```python
import requests
import os

def add_contact_to_list(list_id, contact_id):
    headers = {
        "Authorization": f"Bearer {os.getenv('HUBSPOT_ACCESS_TOKEN')}",
        "Content-Type": "application/json"
    }

    data = {
        "vids": [contact_id]
    }

    try:
        response = requests.post(
            f"https://api.hubapi.com/contacts/v1/lists/{list_id}/add",
            headers=headers,
            json=data
        )
        response.raise_for_status()
        print("Contact added to list")
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error adding contact to list: {e}")

add_contact_to_list(123, 456)
```

### Remove Contact from List

```python
import requests
import os

def remove_contact_from_list(list_id, contact_id):
    headers = {
        "Authorization": f"Bearer {os.getenv('HUBSPOT_ACCESS_TOKEN')}",
        "Content-Type": "application/json"
    }

    data = {
        "vids": [contact_id]
    }

    try:
        response = requests.post(
            f"https://api.hubapi.com/contacts/v1/lists/{list_id}/remove",
            headers=headers,
            json=data
        )
        response.raise_for_status()
        print("Contact removed from list")
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error removing contact from list: {e}")

remove_contact_from_list(123, 456)
```

## Owners API

### Get All Owners

```python
from hubspot.crm.owners import ApiException

def get_all_owners():
    try:
        api_response = api_client.crm.owners.owners_api.get_page()
        print(f"Found {len(api_response.results)} owners")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching owners: {e}")

get_all_owners()
```

### Get Owner by ID

```python
from hubspot.crm.owners import ApiException

def get_owner(owner_id):
    try:
        api_response = api_client.crm.owners.owners_api.get_by_id(
            owner_id=owner_id
        )
        print(f"Owner: {api_response.email}")
        return api_response
    except ApiException as e:
        print(f"Exception when fetching owner: {e}")

get_owner("12345")
```

## Error Handling

### Comprehensive Error Handling

```python
from hubspot.crm.contacts import ApiException

def handle_api_call():
    try:
        api_response = api_client.crm.contacts.basic_api.get_by_id(
            contact_id="12345"
        )
        return api_response
    except ApiException as e:
        if e.status == 401:
            print("Authentication failed - check your access token")
        elif e.status == 404:
            print("Resource not found")
        elif e.status == 409:
            print("Conflict - resource already exists")
        elif e.status == 429:
            print("Rate limit exceeded - wait before retrying")
        elif e.status >= 500:
            print("HubSpot server error - try again later")
        else:
            print(f"API error: {e}")
        raise

handle_api_call()
```

### Retry Logic

```python
import time
from hubspot.crm.contacts import ApiException

def api_call_with_retry(api_call, max_retries=3):
    for attempt in range(1, max_retries + 1):
        try:
            return api_call()
        except ApiException as e:
            if e.status == 429 and attempt < max_retries:
                retry_after = int(e.headers.get('Retry-After', 1))
                print(f"Rate limited. Retrying after {retry_after} seconds...")
                time.sleep(retry_after)
            else:
                raise

# Usage
result = api_call_with_retry(
    lambda: api_client.crm.contacts.basic_api.get_by_id(contact_id="12345")
)
```

## Rate Limits

HubSpot API has the following rate limits:

- **Private Apps:** 100 requests per 10 seconds per account
- **OAuth Apps:** 100 requests per 10 seconds per app per account
- **Search API:** 5 requests per second, 1000 requests per day (per account)
- **Batch API:** 100 objects per request, same rate limits as standard endpoints

Monitor rate limit headers in responses:

```python
from hubspot.crm.contacts import ApiException

def check_rate_limits():
    try:
        api_response = api_client.crm.contacts.basic_api.get_page(limit=1)

        # Rate limit information is in the response headers
        # These are typically accessed via the underlying HTTP client
        print("Request successful")
        return api_response
    except ApiException as e:
        if e.status == 429:
            print("Rate limit exceeded")
            print(f"Retry after: {e.headers.get('Retry-After')} seconds")

check_rate_limits()
```

## Pagination

Most list endpoints return paginated results. Use the `after` parameter for pagination:

```python
from hubspot.crm.contacts import ApiException

def paginate_through_contacts():
    after = None
    all_contacts = []

    while True:
        try:
            api_response = api_client.crm.contacts.basic_api.get_page(
                limit=100,
                after=after,
                properties=["email", "firstname", "lastname"]
            )

            all_contacts.extend(api_response.results)

            if api_response.paging and api_response.paging.next:
                after = api_response.paging.next.after
            else:
                break
        except ApiException as e:
            print(f"Exception during pagination: {e}")
            break

    return all_contacts

all_contacts = paginate_through_contacts()
print(f"Total contacts: {len(all_contacts)}")
```

## OAuth Implementation

### Initialize OAuth Provider

```python
from hubspot import HubSpot
import os

api_client = HubSpot(
    access_token=None,  # Will be set after OAuth flow
    client_id=os.getenv('HUBSPOT_CLIENT_ID'),
    client_secret=os.getenv('HUBSPOT_CLIENT_SECRET'),
    redirect_uri='https://yourapp.com/oauth-callback'
)
```

### Get Authorization URL

```python
import os

def get_authorization_url():
    scopes = ['contacts', 'crm.objects.contacts.read']

    auth_url = (
        f"https://app.hubspot.com/oauth/authorize"
        f"?client_id={os.getenv('HUBSPOT_CLIENT_ID')}"
        f"&redirect_uri={os.getenv('REDIRECT_URI')}"
        f"&scope={' '.join(scopes)}"
    )

    return auth_url

print(get_authorization_url())
```

### Exchange Authorization Code for Tokens

```python
from hubspot.auth.oauth import ApiException

def get_tokens(authorization_code):
    try:
        token_response = api_client.auth.oauth.tokens_api.create(
            grant_type='authorization_code',
            code=authorization_code,
            redirect_uri=os.getenv('REDIRECT_URI'),
            client_id=os.getenv('HUBSPOT_CLIENT_ID'),
            client_secret=os.getenv('HUBSPOT_CLIENT_SECRET')
        )

        return {
            "access_token": token_response.access_token,
            "refresh_token": token_response.refresh_token,
            "expires_in": token_response.expires_in
        }
    except ApiException as e:
        print(f"Error exchanging code for tokens: {e}")

tokens = get_tokens("authorization_code_here")
```

### Refresh Access Token

```python
from hubspot.auth.oauth import ApiException
import os

def refresh_access_token(refresh_token):
    try:
        token_response = api_client.auth.oauth.tokens_api.create(
            grant_type='refresh_token',
            redirect_uri=None,
            refresh_token=refresh_token,
            client_id=os.getenv('HUBSPOT_CLIENT_ID'),
            client_secret=os.getenv('HUBSPOT_CLIENT_SECRET')
        )

        return {
            "access_token": token_response.access_token,
            "refresh_token": token_response.refresh_token,
            "expires_in": token_response.expires_in
        }
    except ApiException as e:
        print(f"Error refreshing token: {e}")

new_tokens = refresh_access_token("refresh_token_here")
```

## Custom Objects

### Create Custom Object Schema

```python
from hubspot.crm.schemas import ObjectSchemaEgg, ObjectTypePropertyCreate, ApiException

def create_custom_object_schema():
    properties = [
        ObjectTypePropertyCreate(
            name="model",
            label="Model",
            type="string",
            field_type="text"
        ),
        ObjectTypePropertyCreate(
            name="make",
            label="Make",
            type="string",
            field_type="text"
        ),
        ObjectTypePropertyCreate(
            name="year",
            label="Year",
            type="number",
            field_type="number"
        )
    ]

    schema_egg = ObjectSchemaEgg(
        name="cars",
        labels={
            "singular": "Car",
            "plural": "Cars"
        },
        primary_display_property="model",
        required_properties=["model", "make"],
        searchable_properties=["model", "make", "year"],
        properties=properties
    )

    try:
        api_response = api_client.crm.schemas.core_api.create(
            object_schema_egg=schema_egg
        )
        print(f"Custom object schema created: {api_response.name}")
        return api_response
    except ApiException as e:
        print(f"Exception when creating custom object schema: {e}")

create_custom_object_schema()
```

### Create Custom Object Instance

```python
from hubspot.crm.objects import SimplePublicObjectInputForCreate, ApiException

def create_custom_object(object_type):
    object_data = SimplePublicObjectInputForCreate(
        properties={
            "model": "Model 3",
            "make": "Tesla",
            "year": "2024"
        }
    )

    try:
        api_response = api_client.crm.objects.basic_api.create(
            object_type=object_type,
            simple_public_object_input_for_create=object_data
        )
        print(f"Custom object created: {api_response.id}")
        return api_response
    except ApiException as e:
        print(f"Exception when creating custom object: {e}")

create_custom_object("cars")
```

## Complete Example Application

Here's a complete example that demonstrates multiple API operations:

```python
from hubspot import HubSpot
from hubspot.crm.contacts import SimplePublicObjectInputForCreate, PublicObjectSearchRequest
from hubspot.crm.companies import SimplePublicObjectInputForCreate as CompanyInput
from hubspot.crm.deals import SimplePublicObjectInputForCreate as DealInput
from hubspot.crm.contacts import ApiException
import os
from dotenv import load_dotenv

load_dotenv()

api_client = HubSpot(access_token=os.getenv('HUBSPOT_ACCESS_TOKEN'))

def main():
    try:
        # Create a contact
        print("Creating contact...")
        new_contact = api_client.crm.contacts.basic_api.create(
            simple_public_object_input_for_create=SimplePublicObjectInputForCreate(
                properties={
                    "email": "john.doe@example.com",
                    "firstname": "John",
                    "lastname": "Doe",
                    "phone": "555-0100",
                    "company": "Example Corp"
                }
            )
        )
        print(f"Contact created with ID: {new_contact.id}")

        # Create a company
        print("\nCreating company...")
        new_company = api_client.crm.companies.basic_api.create(
            simple_public_object_input_for_create=CompanyInput(
                properties={
                    "name": "Example Corp",
                    "domain": "example.com",
                    "city": "San Francisco",
                    "industry": "Technology"
                }
            )
        )
        print(f"Company created with ID: {new_company.id}")

        # Associate contact with company
        print("\nAssociating contact with company...")
        api_client.crm.contacts.associations_api.create(
            contact_id=new_contact.id,
            to_object_type="companies",
            to_object_id=new_company.id,
            association_type="contact_to_company"
        )
        print("Association created successfully")

        # Create a deal
        print("\nCreating deal...")
        new_deal = api_client.crm.deals.basic_api.create(
            simple_public_object_input_for_create=DealInput(
                properties={
                    "dealname": "Example Deal",
                    "dealstage": "appointmentscheduled",
                    "amount": "10000",
                    "closedate": "2025-12-31"
                }
            )
        )
        print(f"Deal created with ID: {new_deal.id}")

        # Associate deal with contact
        print("\nAssociating deal with contact...")
        api_client.crm.deals.associations_api.create(
            deal_id=new_deal.id,
            to_object_type="contacts",
            to_object_id=new_contact.id,
            association_type="deal_to_contact"
        )
        print("Deal associated with contact")

        # Search for contacts
        print("\nSearching for leads...")
        search_results = api_client.crm.contacts.search_api.do_search(
            public_object_search_request=PublicObjectSearchRequest(
                filter_groups=[
                    {
                        "filters": [
                            {
                                "propertyName": "email",
                                "operator": "CONTAINS_TOKEN",
                                "value": "example.com"
                            }
                        ]
                    }
                ],
                properties=["email", "firstname", "lastname", "company"],
                limit=10
            )
        )
        print(f"Found {search_results.total} contacts")

        # List all owners
        print("\nFetching owners...")
        owners = api_client.crm.owners.owners_api.get_page()
        print(f"Found {len(owners.results)} owners")

        print("\nAll operations completed successfully!")

    except ApiException as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()
```

## Environment Variables Template

```bash
# .env file
HUBSPOT_ACCESS_TOKEN=your_private_app_access_token
HUBSPOT_CLIENT_ID=your_client_id
HUBSPOT_CLIENT_SECRET=your_client_secret
REDIRECT_URI=https://yourapp.com/oauth-callback
```

## API Scopes

When creating a private app or OAuth app, configure the following scopes based on your needs:

**CRM:**
- `crm.objects.contacts.read` / `crm.objects.contacts.write`
- `crm.objects.companies.read` / `crm.objects.companies.write`
- `crm.objects.deals.read` / `crm.objects.deals.write`
- `crm.objects.owners.read`
- `crm.schemas.contacts.read` / `crm.schemas.contacts.write`

**Marketing:**
- `forms` - Read and write forms
- `content` - Read and write marketing emails

**Automation:**
- `automation` - Access to workflows

**Timeline:**
- `timeline` - Create timeline events

## Migration from Legacy API Keys

If you're using legacy API keys (deprecated), migrate to private apps:

**Old (Deprecated):**
```python
# DON'T USE THIS - hapikey is no longer supported
api_client = HubSpot(api_key='your-api-key')  # Removed after v5.1.0
```

**New (Correct):**
```python
api_client = HubSpot(access_token=os.getenv('HUBSPOT_ACCESS_TOKEN'))
```

Legacy API keys will be revoked on November 19, 2025.
