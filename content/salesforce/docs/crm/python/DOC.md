---
name: crm
description: "Salesforce Python SDK (simple-salesforce) coding guidelines for Salesforce REST API interactions"
metadata:
  languages: "python"
  versions: "1.12.9"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "salesforce,crm,soql,enterprise,api"
---

# Salesforce Python SDK (simple-salesforce) Coding Guidelines

You are a Salesforce API coding expert. Help me with writing code using the Salesforce API calling the official libraries and SDKs.

You can find the official SDK documentation and code samples here:
https://simple-salesforce.readthedocs.io/

## Golden Rule: Use the Correct and Current SDK

Always use simple-salesforce to interact with Salesforce APIs. Simple-salesforce is the most widely used Python library for Salesforce REST API interactions.

- **Library Name:** simple-salesforce
- **PyPI Package:** `simple-salesforce`
- **Alternative Library:** `salesforce-api` (less popular)
- **Legacy Libraries**: Do not use deprecated or unofficial Salesforce Python packages

**Installation:**

- **Correct:** `pip install simple-salesforce`
- **With environment variables support:** `pip install simple-salesforce python-dotenv`

**APIs and Usage:**

- **Correct:** `from simple_salesforce import Salesforce`
- **Correct:** `sf = Salesforce(username='...', password='...', security_token='...')`
- **Correct:** `sf.query("SELECT Id, Name FROM Account")`
- **Correct:** `sf.Account.create({'Name': 'Test'})`
- **Incorrect:** Using unofficial Salesforce client libraries
- **Incorrect:** Direct REST API calls without SDK

## Installation

Install simple-salesforce via pip:

```bash
pip install simple-salesforce
```

For environment variable support:

```bash
pip install simple-salesforce python-dotenv
```

## Environment Variables

Set up your Salesforce credentials using environment variables:

```bash
export SALESFORCE_USERNAME="user@example.org"
export SALESFORCE_PASSWORD="your_password"
export SALESFORCE_SECURITY_TOKEN="your_security_token"
export SALESFORCE_DOMAIN="login" # or "test" for sandbox
```

Or create a `.env` file:

```
SALESFORCE_USERNAME=user@example.org
SALESFORCE_PASSWORD=your_password
SALESFORCE_SECURITY_TOKEN=your_security_token
SALESFORCE_DOMAIN=login
```

Load environment variables in code:

```python
import os
from dotenv import load_dotenv

load_dotenv()

username = os.getenv('SALESFORCE_USERNAME')
password = os.getenv('SALESFORCE_PASSWORD')
security_token = os.getenv('SALESFORCE_SECURITY_TOKEN')
```

## Initialization

Simple-salesforce requires creating a `Salesforce` instance for all API calls.

### Basic Connection (Username/Password/Token)

```python
from simple_salesforce import Salesforce

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token'
)
```

### Connection with Environment Variables

```python
import os
from dotenv import load_dotenv
from simple_salesforce import Salesforce

load_dotenv()

sf = Salesforce(
    username=os.getenv('SALESFORCE_USERNAME'),
    password=os.getenv('SALESFORCE_PASSWORD'),
    security_token=os.getenv('SALESFORCE_SECURITY_TOKEN')
)
```

### Connection to Sandbox

```python
from simple_salesforce import Salesforce

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token',
    domain='test'  # Use 'test' for sandbox
)
```

### Connection with Custom Domain

```python
from simple_salesforce import Salesforce

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token',
    domain='mydomain'  # Your My Domain
)
```

## Authentication

### Username/Password/Token Authentication

```python
from simple_salesforce import Salesforce

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token'
)

print(f"Session ID: {sf.session_id}")
print(f"Instance URL: {sf.sf_instance}")
```

### SalesforceLogin (No Token Required)

```python
from simple_salesforce import Salesforce, SalesforceLogin

session_id, instance = SalesforceLogin(
    username='user@example.org',
    password='your_password'
)

sf = Salesforce(instance=instance, session_id=session_id)
```

### Session ID Authentication (OAuth)

```python
from simple_salesforce import Salesforce

sf = Salesforce(
    instance='na1.salesforce.com',
    session_id='your_session_id_from_oauth'
)
```

### OAuth 2.0 Access Token

```python
from simple_salesforce import Salesforce

sf = Salesforce(
    instance_url='https://na1.salesforce.com',
    session_id='your_oauth_access_token'
)
```

### Using Requests Session

```python
import requests
from simple_salesforce import Salesforce

session = requests.Session()
session.headers.update({'Custom-Header': 'value'})

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token',
    session=session
)
```

## SOQL Queries

### Basic Query

```python
from simple_salesforce import Salesforce

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token'
)

result = sf.query("SELECT Id, Name FROM Account LIMIT 10")

print(f"Total records: {result['totalSize']}")

for record in result['records']:
    print(f"Account: {record['Name']}")
```

### Query with WHERE Clause

```python
result = sf.query(
    "SELECT Id, Name, Industry FROM Account WHERE Industry = 'Technology'"
)

print(f"Found {result['totalSize']} Technology accounts")

for record in result['records']:
    print(f"{record['Name']} - {record['Industry']}")
```

### Query with Relationships

```python
# Parent to child relationship
result = sf.query(
    "SELECT Id, Name, (SELECT Id, Name FROM Contacts) FROM Account LIMIT 5"
)

for account in result['records']:
    print(f"Account: {account['Name']}")
    if account.get('Contacts'):
        for contact in account['Contacts']['records']:
            print(f"  Contact: {contact['Name']}")
```

### Query with Child to Parent Relationship

```python
result = sf.query(
    "SELECT Id, Name, Account.Name FROM Contact LIMIT 10"
)

for contact in result['records']:
    account_name = contact.get('Account', {}).get('Name', 'N/A')
    print(f"{contact['Name']} - Account: {account_name}")
```

### Query All (Including Deleted Records)

```python
result = sf.query_all(
    "SELECT Id, Name, IsDeleted FROM Account WHERE IsDeleted = true"
)

print(f"Deleted accounts: {result['totalSize']}")
```

### Query with Automatic Pagination

```python
# query_all handles pagination automatically
result = sf.query_all("SELECT Id, Name FROM Account")

print(f"Total records: {result['totalSize']}")

for record in result['records']:
    print(f"Account: {record['Name']}")
```

### Query All Iterator (Memory Efficient)

```python
# Use query_all_iter for large datasets
for record in sf.query_all_iter("SELECT Id, Name FROM Account"):
    print(f"Account: {record['Name']}")
```

### Manual Pagination

```python
result = sf.query("SELECT Id, Name FROM Account")
all_records = result['records']

while not result['done']:
    result = sf.query_more(result['nextRecordsUrl'], identifier_is_url=True)
    all_records.extend(result['records'])

print(f"Total records retrieved: {len(all_records)}")
```

### Safe Query with Parameter Formatting

```python
from simple_salesforce.format import format_soql

# Safe parameter substitution
last_name = "O'Brien"
query = format_soql("SELECT Id, Email FROM Contact WHERE LastName = {}", last_name)
result = sf.query(query)

# LIKE queries
name_pattern = "John"
query = format_soql(
    "SELECT Id, Name FROM Contact WHERE Name LIKE '{:like}%'",
    name_pattern
)
result = sf.query(query)

# Multiple parameters
query = format_soql(
    "SELECT Id, Name FROM Account WHERE Industry = {} AND AnnualRevenue > {}",
    "Technology",
    1000000
)
result = sf.query(query)
```

### Query with Date Filters

```python
from datetime import datetime, timedelta

# Query records created in the last 7 days
seven_days_ago = (datetime.now() - timedelta(days=7)).strftime('%Y-%m-%dT%H:%M:%SZ')
query = f"SELECT Id, Name FROM Account WHERE CreatedDate > {seven_days_ago}"
result = sf.query(query)
```

## SOSL Search

### Basic Search

```python
result = sf.search(
    "FIND {United*} IN NAME FIELDS RETURNING Account(Id, Name), Contact(Id, Name)"
)

print(f"Search results: {result['searchRecords']}")

for record in result['searchRecords']:
    print(f"{record['attributes']['type']}: {record['Name']}")
```

## CRUD Operations

### Create Single Record

```python
from simple_salesforce import Salesforce

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token'
)

result = sf.Account.create({
    'Name': 'New Account',
    'Industry': 'Technology',
    'BillingCity': 'San Francisco'
})

print(f"Created account ID: {result['id']}")
print(f"Success: {result['success']}")
```

### Create with Error Handling

```python
try:
    result = sf.Account.create({
        'Name': 'Test Account',
        'Industry': 'Technology'
    })
    print(f"Created: {result['id']}")
except Exception as e:
    print(f"Error creating account: {e}")
```

### Retrieve Single Record by ID

```python
account = sf.Account.get('001XXXXXXXXXXXXXXX')

print(f"Account Name: {account['Name']}")
print(f"Industry: {account['Industry']}")
print(f"Created Date: {account['CreatedDate']}")
```

### Retrieve Specific Fields

```python
account = sf.Account.get('001XXXXXXXXXXXXXXX', fields=['Id', 'Name', 'Industry'])

print(f"Name: {account['Name']}")
print(f"Industry: {account['Industry']}")
```

### Update Single Record

```python
result = sf.Account.update('001XXXXXXXXXXXXXXX', {
    'Name': 'Updated Account Name',
    'Industry': 'Media'
})

print(f"Updated: {result}")
```

### Update Multiple Fields

```python
result = sf.Account.update('001XXXXXXXXXXXXXXX', {
    'Name': 'New Name',
    'Industry': 'Finance',
    'BillingCity': 'New York',
    'Phone': '555-1234'
})
```

### Upsert (Update or Insert)

```python
# Upsert using external ID
result = sf.Account.upsert('ExternalId__c/EXT001', {
    'Name': 'Upserted Account',
    'Industry': 'Technology',
    'ExternalId__c': 'EXT001'
})

if result == 204:
    print("Record updated")
elif result == 201:
    print("Record created")
```

### Upsert with Response Details

```python
try:
    result = sf.Contact.upsert('Email/[email protected]', {
        'FirstName': 'John',
        'LastName': 'Doe',
        'Email': '[email protected]'
    }, raw_response=True)

    if result.status_code == 201:
        print("Contact created")
    elif result.status_code == 204:
        print("Contact updated")
except Exception as e:
    print(f"Upsert error: {e}")
```

### Delete Single Record

```python
result = sf.Account.delete('001XXXXXXXXXXXXXXX')

print(f"Deleted: {result}")
```

### Delete with Error Handling

```python
try:
    sf.Account.delete('001XXXXXXXXXXXXXXX')
    print("Account deleted successfully")
except Exception as e:
    print(f"Error deleting account: {e}")
```

## Bulk Operations

### Bulk Insert

```python
from simple_salesforce import Salesforce

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token'
)

# Prepare data
accounts = []
for i in range(1000):
    accounts.append({
        'Name': f'Bulk Account {i}',
        'Industry': 'Technology'
    })

# Bulk insert using Bulk 2.0 API
result = sf.bulk2.Account.insert(accounts)

print(f"Job ID: {result['job_id']}")
print(f"Records processed: {result['numberRecordsProcessed']}")
```

### Bulk Insert from CSV

```python
# Insert from CSV file
result = sf.bulk2.Account.insert('./accounts.csv')

print(f"Job ID: {result['job_id']}")
print(f"Records processed: {result['numberRecordsProcessed']}")
```

### Bulk Update

```python
# Prepare update data
updates = []
for account_id in account_ids:
    updates.append({
        'Id': account_id,
        'Industry': 'Finance'
    })

result = sf.bulk2.Account.update(updates)

print(f"Records updated: {result['numberRecordsProcessed']}")
```

### Bulk Update from CSV

```python
result = sf.bulk2.Account.update('./account_updates.csv')

print(f"Job ID: {result['job_id']}")
```

### Bulk Delete

```python
# Prepare delete data (IDs only)
deletes = []
for account_id in account_ids_to_delete:
    deletes.append({'Id': account_id})

result = sf.bulk2.Account.delete(deletes)

print(f"Records deleted: {result['numberRecordsProcessed']}")
```

### Bulk Upsert

```python
upserts = []
for i in range(100):
    upserts.append({
        'ExternalId__c': f'EXT{i}',
        'Name': f'Bulk Upsert Account {i}',
        'Industry': 'Technology'
    })

result = sf.bulk2.Account.upsert(upserts, 'ExternalId__c')

print(f"Records processed: {result['numberRecordsProcessed']}")
```

### Get Bulk Job Results

```python
job_id = 'bulk_job_id_from_insert_or_update'

# Get successful records
successful = sf.bulk2.Account.get_successful_records(job_id)
print(f"Successful records: {successful}")

# Get failed records
failed = sf.bulk2.Account.get_failed_records(job_id)
print(f"Failed records: {failed}")

# Save failed records to CSV
sf.bulk2.Account.get_failed_records(job_id, file='failed_records.csv')
```

### Bulk Query

```python
# Bulk 2.0 query for large datasets
result = sf.bulk2.Account.query("SELECT Id, Name, Industry FROM Account")

print(f"Job ID: {result['job_id']}")

# Get query results
query_results = sf.bulk2.Account.get_query_results(result['job_id'])

for record in query_results:
    print(f"Account: {record['Name']}")
```

## SObject Metadata

### Describe SObject

```python
metadata = sf.Account.describe()

print(f"Label: {metadata['label']}")
print(f"Updateable: {metadata['updateable']}")
print(f"Deletable: {metadata['deletable']}")
print(f"Number of fields: {len(metadata['fields'])}")

for field in metadata['fields']:
    print(f"  {field['name']} ({field['type']})")
```

### Describe All SObjects

```python
result = sf.describe()

for sobject in result['sobjects']:
    print(f"{sobject['name']} - {sobject['label']}")
```

### Get Field Information

```python
metadata = sf.Contact.describe()

for field in metadata['fields']:
    if field['name'] == 'Email':
        print(f"Type: {field['type']}")
        print(f"Length: {field['length']}")
        print(f"Required: {field['nillable'] == False}")
        print(f"Updateable: {field['updateable']}")
```

### Get Picklist Values

```python
metadata = sf.Account.describe()

for field in metadata['fields']:
    if field['name'] == 'Industry' and field['type'] == 'picklist':
        print("Industry picklist values:")
        for value in field['picklistValues']:
            print(f"  - {value['value']}")
```

## Apex REST API

### Call Custom Apex REST Endpoint (GET)

```python
# Call GET endpoint
result = sf.apexecute('MyApexRestService', method='GET')

print(f"Response: {result}")
```

### Call Custom Apex REST Endpoint (POST)

```python
payload = {
    'name': 'Test',
    'value': 123,
    'active': True
}

result = sf.apexecute('MyApexRestService', method='POST', data=payload)

print(f"Response: {result}")
```

### Call Custom Apex REST Endpoint (PUT)

```python
payload = {
    'name': 'Updated Name',
    'value': 456
}

result = sf.apexecute('MyApexRestService/recordId', method='PUT', data=payload)

print(f"Response: {result}")
```

### Call Custom Apex REST Endpoint (DELETE)

```python
result = sf.apexecute('MyApexRestService/recordId', method='DELETE')

print(f"Response: {result}")
```

### Call with URL Parameters

```python
result = sf.apexecute('MyApexRestService', method='GET', params={'filter': 'active'})

print(f"Response: {result}")
```

## Generic REST API Requests

### GET Request

```python
# Make a generic GET request
result = sf.restful('sobjects/Account/001XXXXXXXXXXXXXXX')

print(f"Account: {result}")
```

### POST Request

```python
data = {
    'Name': 'New Account',
    'Industry': 'Technology'
}

result = sf.restful('sobjects/Account', method='POST', data=data)

print(f"Created: {result}")
```

### Custom API Endpoint

```python
# Call a custom REST endpoint
result = sf.restful('services/apexrest/CustomEndpoint', method='GET')

print(f"Response: {result}")
```

## Advanced Features

### Set API Version

```python
sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token',
    version='58.0'
)
```

### Disable SSL Verification (Not Recommended)

```python
import urllib3
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token',
    security_token_bypass=True,
    session=None
)
```

### Custom Timeout

```python
import requests
from simple_salesforce import Salesforce

session = requests.Session()
session.timeout = 60  # 60 seconds

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token',
    session=session
)
```

### Proxies

```python
import requests
from simple_salesforce import Salesforce

session = requests.Session()
session.proxies = {
    'http': 'http://proxy.example.com:8080',
    'https': 'https://proxy.example.com:8080'
}

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token',
    session=session
)
```

### Custom Headers

```python
import requests
from simple_salesforce import Salesforce

session = requests.Session()
session.headers.update({
    'Custom-Header': 'value',
    'Another-Header': 'another-value'
})

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token',
    session=session
)
```

## Composite API

### Composite Request

```python
composite_request = {
    'compositeRequest': [
        {
            'method': 'POST',
            'url': '/services/data/v58.0/sobjects/Account',
            'referenceId': 'NewAccount',
            'body': {
                'Name': 'Composite Account'
            }
        },
        {
            'method': 'POST',
            'url': '/services/data/v58.0/sobjects/Contact',
            'referenceId': 'NewContact',
            'body': {
                'FirstName': 'John',
                'LastName': 'Doe',
                'AccountId': '@{NewAccount.id}'
            }
        }
    ]
}

result = sf.restful(
    'composite',
    method='POST',
    data=composite_request
)

print(f"Composite results: {result}")
```

## Working with Attachments

### Upload Attachment

```python
import base64

with open('document.pdf', 'rb') as f:
    file_data = base64.b64encode(f.read()).decode()

result = sf.Attachment.create({
    'Name': 'document.pdf',
    'ParentId': '001XXXXXXXXXXXXXXX',  # Record ID
    'Body': file_data,
    'ContentType': 'application/pdf'
})

print(f"Attachment ID: {result['id']}")
```

### Download Attachment

```python
import base64

attachment = sf.Attachment.get('00PXXXXXXXXXXXXXXX')
file_data = base64.b64decode(attachment['Body'])

with open('downloaded_file.pdf', 'wb') as f:
    f.write(file_data)
```

## Working with ContentVersion (Files)

### Upload File

```python
import base64

with open('document.pdf', 'rb') as f:
    file_data = base64.b64encode(f.read()).decode()

result = sf.ContentVersion.create({
    'Title': 'My Document',
    'PathOnClient': 'document.pdf',
    'VersionData': file_data,
    'FirstPublishLocationId': '001XXXXXXXXXXXXXXX'  # Record ID
})

print(f"ContentVersion ID: {result['id']}")
```

### Query Files

```python
result = sf.query(
    "SELECT Id, Title, FileType FROM ContentDocument WHERE Title LIKE '%Report%'"
)

for doc in result['records']:
    print(f"Document: {doc['Title']} ({doc['FileType']})")
```

## Error Handling

### Basic Error Handling

```python
from simple_salesforce import Salesforce, SalesforceAuthenticationFailed

try:
    sf = Salesforce(
        username='user@example.org',
        password='wrong_password',
        security_token='token'
    )
except SalesforceAuthenticationFailed as e:
    print(f"Authentication failed: {e}")
```

### CRUD Error Handling

```python
from simple_salesforce.exceptions import SalesforceError

try:
    result = sf.Account.create({
        'Name': ''  # Required field missing
    })
except SalesforceError as e:
    print(f"Error code: {e.status}")
    print(f"Error message: {e.content}")
```

### Query Error Handling

```python
try:
    result = sf.query("SELECT Id, InvalidField FROM Account")
except SalesforceError as e:
    print(f"Query error: {e}")
```

### Generic Exception Handling

```python
import requests

try:
    result = sf.Account.update('001XXXXXXXXXXXXXXX', {
        'Name': 'Updated Name'
    })
except requests.exceptions.Timeout:
    print("Request timed out")
except requests.exceptions.ConnectionError:
    print("Connection error")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## Best Practices

### Batch Processing

```python
def process_accounts_in_batches(account_ids, batch_size=200):
    for i in range(0, len(account_ids), batch_size):
        batch = account_ids[i:i + batch_size]

        # Build query for batch
        ids_str = "','".join(batch)
        query = f"SELECT Id, Name FROM Account WHERE Id IN ('{ids_str}')"

        result = sf.query(query)

        for record in result['records']:
            # Process each record
            print(f"Processing: {record['Name']}")
```

### Retry Logic

```python
import time
from simple_salesforce.exceptions import SalesforceError

def create_with_retry(sobject, data, max_retries=3):
    for attempt in range(max_retries):
        try:
            result = getattr(sf, sobject).create(data)
            return result
        except SalesforceError as e:
            if attempt < max_retries - 1:
                wait_time = 2 ** attempt  # Exponential backoff
                print(f"Retry {attempt + 1} after {wait_time}s")
                time.sleep(wait_time)
            else:
                raise
```

### Connection Pooling

```python
import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry
from simple_salesforce import Salesforce

def create_sf_session():
    session = requests.Session()

    retry_strategy = Retry(
        total=3,
        status_forcelist=[429, 500, 502, 503, 504],
        method_whitelist=["HEAD", "GET", "OPTIONS", "POST", "PATCH", "PUT"]
    )

    adapter = HTTPAdapter(max_retries=retry_strategy)
    session.mount("https://", adapter)
    session.mount("http://", adapter)

    return session

sf = Salesforce(
    username='user@example.org',
    password='your_password',
    security_token='your_security_token',
    session=create_sf_session()
)
```

### Efficient Field Selection

```python
# Only query fields you need
result = sf.query(
    "SELECT Id, Name, Industry FROM Account LIMIT 100"
)

# Instead of:
# result = sf.query("SELECT Id, Name, Industry, ... (all fields) FROM Account LIMIT 100")
```

### Use Bulk API for Large Data Sets

```python
# For < 2000 records, use regular API
if len(records) < 2000:
    for record in records:
        sf.Account.create(record)
else:
    # For >= 2000 records, use Bulk API
    sf.bulk2.Account.insert(records)
```

## Complete Example Application

```python
import os
from dotenv import load_dotenv
from simple_salesforce import Salesforce, SalesforceAuthenticationFailed
from simple_salesforce.exceptions import SalesforceError

# Load environment variables
load_dotenv()

def main():
    try:
        # Initialize connection
        sf = Salesforce(
            username=os.getenv('SALESFORCE_USERNAME'),
            password=os.getenv('SALESFORCE_PASSWORD'),
            security_token=os.getenv('SALESFORCE_SECURITY_TOKEN')
        )

        print("Connected to Salesforce successfully!")

        # Create account
        account_result = sf.Account.create({
            'Name': 'Example Corp',
            'Industry': 'Technology',
            'BillingCity': 'San Francisco'
        })
        account_id = account_result['id']
        print(f"Created account: {account_id}")

        # Create contact
        contact_result = sf.Contact.create({
            'FirstName': 'John',
            'LastName': 'Doe',
            'Email': '[email protected]',
            'AccountId': account_id
        })
        contact_id = contact_result['id']
        print(f"Created contact: {contact_id}")

        # Query accounts with contacts
        result = sf.query(f"""
            SELECT Id, Name, Industry,
                   (SELECT Id, Name, Email FROM Contacts)
            FROM Account
            WHERE Id = '{account_id}'
        """)

        for account in result['records']:
            print(f"\nAccount: {account['Name']}")
            if account.get('Contacts'):
                for contact in account['Contacts']['records']:
                    print(f"  Contact: {contact['Name']} - {contact['Email']}")

        # Update account
        sf.Account.update(account_id, {
            'Industry': 'Finance'
        })
        print(f"\nUpdated account industry")

        # Clean up - delete records
        sf.Contact.delete(contact_id)
        sf.Account.delete(account_id)
        print("\nCleaned up test data")

    except SalesforceAuthenticationFailed as e:
        print(f"Authentication failed: {e}")
    except SalesforceError as e:
        print(f"Salesforce API error: {e}")
    except Exception as e:
        print(f"Unexpected error: {e}")

if __name__ == '__main__':
    main()
```

## Useful Links

- Documentation: https://simple-salesforce.readthedocs.io/
- GitHub: https://github.com/simple-salesforce/simple-salesforce
- PyPI: https://pypi.org/project/simple-salesforce/
- Salesforce Developer Docs: https://developer.salesforce.com/
- Salesforce REST API Guide: https://developer.salesforce.com/docs/atlas.en-us.api_rest.meta/api_rest/
- SOQL Reference: https://developer.salesforce.com/docs/atlas.en-us.soql_sosl.meta/soql_sosl/
