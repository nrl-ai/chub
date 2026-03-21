---
name: database
description: "Airtable Python SDK (pyairtable) — use the official pyairtable package for Airtable API operations"
metadata:
  languages: "python"
  versions: "3.1.1"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "airtable,database,low-code,spreadsheet,api"
---

# Airtable Python SDK (pyairtable) - Version 3.1.1

## Golden Rule

**ALWAYS use the official `pyairtable` package (version 3.1.1 or later)**

```bash
pip install pyairtable
```

**DO NOT use:**
- Deprecated packages like `airtable` (v0.4.8, last updated 2021)
- `airtable-python-wrapper` (v0.15.3, now renamed to pyairtable)
- `python-airtable` or other unofficial wrappers
- The old API key authentication method (deprecated as of February 1, 2024)

**ALWAYS use Personal Access Tokens (PATs) for authentication**, not the deprecated API keys.

**NOTE:** `pyairtable` is the current and actively maintained name for what was previously called `airtable-python-wrapper`.

## Installation

```bash
pip install pyairtable
```

### Environment Variable Setup

Create a `.env` file:

```bash
AIRTABLE_API_KEY=your_personal_access_token_here
```

For production applications, use proper secret management systems.

## Authentication & Initialization

### Personal Access Token Setup

1. Visit https://airtable.com/create/tokens to create a Personal Access Token
2. Name your token (e.g., "My App Token")
3. Add required scopes:
   - `data.records:read` - to read records
   - `data.records:write` - to create/update/delete records
   - `schema.bases:read` - to read base structure
   - `schema.bases:write` - to modify base structure (optional)
4. Select base access level (specific bases or all workspace bases)
5. Copy the token immediately (shown only once)

### Basic Configuration

**Option 1: Direct Token Initialization**

```python
from pyairtable import Api

api = Api('your_personal_access_token')
table = api.table('appExampleBaseId', 'tblExampleTableId')
```

**Option 2: Environment Variable**

```python
import os
from pyairtable import Api

api = Api(os.environ['AIRTABLE_API_KEY'])
table = api.table('appExampleBaseId', 'Table Name')
```

**Option 3: Using Base Instance**

```python
from pyairtable import Api

api = Api(os.environ['AIRTABLE_API_KEY'])
base = api.base('appExampleBaseId')
table = base.table('Table Name')
```

### Advanced Configuration

```python
from pyairtable import Api, retry_strategy

# Custom timeout (connect_timeout, read_timeout)
api = Api('token', timeout=(2, 5))

# Enable retry strategy for rate limiting
api = Api('token', retry_strategy=True)

# Custom retry strategy
custom_retry = retry_strategy(
    total=10,
    status_forcelist=(429, 500, 502, 503, 504),
    backoff_factor=0.2
)
api = Api('token', retry_strategy=custom_retry)

# Disable retries
api = Api('token', retry_strategy=None)
```

### Finding Your Base and Table IDs

1. Go to https://airtable.com/api
2. Select your base
3. Base ID format: `appXXXXXXXXXXXXXX`
4. Table ID format: `tblXXXXXXXXXXXXXX` (or use table name as string)

## Core API Surfaces

### Reading Records

#### Get Single Record by ID

```python
from pyairtable import Api

api = Api(os.environ['AIRTABLE_API_KEY'])
table = api.table('appBaseId', 'Tasks')

# Get single record
record = table.get('recwPQIfs4wKPyc9D')

print(f"Record ID: {record['id']}")
print(f"Name: {record['fields']['Name']}")
print(f"Status: {record['fields']['Status']}")
print(f"Created: {record['createdTime']}")

# Access all fields
fields = record['fields']
for field_name, value in fields.items():
    print(f"{field_name}: {value}")
```

#### Get All Records

```python
# Get all records from table
all_records = table.all()

for record in all_records:
    print(f"ID: {record['id']}")
    print(f"Name: {record['fields']['Name']}")

# With view parameter
view_records = table.all(view='Active Tasks')

# With specific fields only
limited_fields = table.all(fields=['Name', 'Status', 'Priority'])

# With max records limit
limited_records = table.all(max_records=50)

# Combine multiple parameters
filtered_records = table.all(
    view='Active Tasks',
    fields=['Name', 'Status'],
    max_records=100
)
```

#### Get First Matching Record

```python
# Get first record matching formula
first_record = table.first(formula="{Status} = 'Active'")

if first_record:
    print(f"Found: {first_record['fields']['Name']}")
else:
    print("No matching record found")

# With view
first_in_view = table.first(view='High Priority')

# With sort
first_sorted = table.first(
    formula="{Priority} = 'High'",
    sort=['Due Date']
)
```

#### Iterate Through Pages

```python
# Iterate through paginated results
for page in table.iterate(page_size=100):
    for record in page:
        print(f"Processing: {record['fields']['Name']}")

# With formula filter
for page in table.iterate(
    formula="{Status} = 'Active'",
    page_size=50
):
    process_records(page)
```

### Filtering Records

#### Using Formula Parameter

```python
# Simple equality
active_tasks = table.all(formula="{Status} = 'Active'")

# Multiple conditions with AND
high_priority = table.all(
    formula="AND({Status} = 'Active', {Priority} = 'High')"
)

# Multiple conditions with OR
urgent_or_blocked = table.all(
    formula="OR({Status} = 'Urgent', {Status} = 'Blocked')"
)

# Not empty check
with_titles = table.all(formula="NOT({Title} = '')")

# Greater than comparison
recent_tasks = table.all(formula="{Created} > '2025-01-01'")

# Dynamic formula with variables
email = 'user@example.com'
user_records = table.all(formula=f"{{Email}} = '{email}'")

# Complex nested formula
complex = table.all(
    formula="""
    AND(
        {Status} = 'In Progress',
        {Priority} = 'High',
        {Assignee} != '',
        {Due Date} <= TODAY()
    )
    """
)

# String search (case-insensitive)
search_term = 'urgent'
search_results = table.all(
    formula=f"SEARCH(LOWER('{search_term}'), LOWER({{Notes}})) > 0"
)
```

#### Using Formula Helpers

```python
from pyairtable.formulas import match, EQUAL, AND, OR, NOT, IF

# Simple match
formula = match({'Status': 'Active'})
records = table.all(formula=formula)

# Multiple field match (AND)
formula = match({'Status': 'Active', 'Priority': 'High'})
records = table.all(formula=formula)

# Using formula operators
formula = AND(
    EQUAL('Status', 'Active'),
    EQUAL('Priority', 'High')
)
records = table.all(formula=str(formula))

# OR condition
formula = OR(
    EQUAL('Status', 'Urgent'),
    EQUAL('Status', 'Blocked')
)
records = table.all(formula=str(formula))

# NOT condition
formula = NOT(EQUAL('Status', 'Done'))
records = table.all(formula=str(formula))

# Complex formula with IF
from pyairtable.formulas import IF, GT

formula = IF(
    GT('Price', 100),
    'Expensive',
    'Affordable'
)
```

### Sorting Records

```python
# Sort by single field (ascending)
sorted_asc = table.all(sort=['Name'])

# Sort by single field (descending)
sorted_desc = table.all(sort=['-Created'])

# Sort by multiple fields
multi_sort = table.all(
    sort=['-Priority', 'Due Date', 'Name']
)

# Combine with filter and fields
filtered_sorted = table.all(
    formula="{Status} = 'Active'",
    sort=['-Priority', 'Due Date'],
    fields=['Name', 'Status', 'Priority']
)
```

### Creating Records

#### Create Single Record

```python
# Basic create
new_record = table.create({
    'Name': 'New Task',
    'Status': 'To Do',
    'Priority': 'Medium'
})

print(f"Created record: {new_record['id']}")
print(f"Name: {new_record['fields']['Name']}")

# Create with all field types
record = table.create({
    'Name': 'Complete Task',
    'Description': 'Long description here',
    'Status': 'In Progress',
    'Priority': 'High',
    'Due Date': '2025-12-31',
    'Completed': False,
    'Tags': ['Important', 'Client'],
    'Progress': 50
})

# With typecast (converts strings to appropriate types)
record_typecast = table.create({
    'Name': 'Task with Typecast',
    'Due Date': '2025-12-31',  # String converted to date
    'Count': '42'  # String converted to number
}, typecast=True)
```

#### Batch Create (Multiple Records)

```python
# Create up to 10 records at once
records = table.batch_create([
    {'Name': 'Task 1', 'Status': 'To Do'},
    {'Name': 'Task 2', 'Status': 'In Progress'},
    {'Name': 'Task 3', 'Status': 'Done'},
    {'Name': 'Task 4', 'Status': 'To Do'},
    {'Name': 'Task 5', 'Status': 'Review'}
])

print(f"Created {len(records)} records")

for record in records:
    print(f"Created: {record['id']} - {record['fields']['Name']}")

# With typecast
records_typecast = table.batch_create([
    {'Name': 'Task 1', 'Count': '10'},
    {'Name': 'Task 2', 'Count': '20'}
], typecast=True)

# Create more than 10 records (automatic batching)
large_batch = [
    {'Name': f'Task {i}', 'Status': 'To Do'}
    for i in range(1, 51)
]

# Process in batches of 10
all_created = []
for i in range(0, len(large_batch), 10):
    batch = large_batch[i:i+10]
    created = table.batch_create(batch)
    all_created.extend(created)

print(f"Created {len(all_created)} records total")
```

### Updating Records

#### Update Single Record (Partial Update)

```python
# Update specific fields only
updated = table.update('recXXXXXXXXXXXXXX', {
    'Status': 'In Progress',
    'Progress': 50
})

print(f"Updated: {updated['fields']['Status']}")

# Update multiple fields
updated = table.update('recXXXXXXXXXXXXXX', {
    'Status': 'Done',
    'Completed': True,
    'Completed Date': '2025-10-25',
    'Notes': 'Task completed successfully'
})

# With typecast
updated = table.update('recXXXXXXXXXXXXXX', {
    'Count': '100',
    'Due Date': '2025-12-31'
}, typecast=True)
```

#### Replace Record (Full Update)

```python
# Replace entire record (fields not specified will be cleared)
replaced = table.update(
    'recXXXXXXXXXXXXXX',
    {
        'Name': 'Completely New Task',
        'Status': 'To Do'
    },
    replace=True
)

# All other fields will be cleared/empty
```

#### Batch Update (Multiple Records)

```python
# Update up to 10 records at once
updates = [
    {
        'id': 'recXXXXXXXXXXXXXX',
        'fields': {'Status': 'Done'}
    },
    {
        'id': 'recYYYYYYYYYYYYYY',
        'fields': {'Status': 'In Progress', 'Progress': 75}
    },
    {
        'id': 'recZZZZZZZZZZZZZZ',
        'fields': {'Status': 'To Do'}
    }
]

updated_records = table.batch_update(updates)
print(f"Updated {len(updated_records)} records")

# With typecast
updated_typecast = table.batch_update([
    {
        'id': 'recXXXXXXXXXXXXXX',
        'fields': {'Count': '100'}
    }
], typecast=True)

# Batch update more than 10 records
record_ids = ['rec1', 'rec2', 'rec3', ...]  # List of many IDs
all_updated = []

for i in range(0, len(record_ids), 10):
    batch_ids = record_ids[i:i+10]
    batch_updates = [
        {
            'id': record_id,
            'fields': {'Status': 'Archived'}
        }
        for record_id in batch_ids
    ]
    updated = table.batch_update(batch_updates)
    all_updated.extend(updated)
```

#### Upsert Operations (Update or Insert)

```python
# Upsert based on key fields
upsert_result = table.batch_upsert(
    [
        {'Email': 'john@example.com', 'Name': 'John Doe', 'Status': 'Active'},
        {'Email': 'jane@example.com', 'Name': 'Jane Smith', 'Status': 'Active'}
    ],
    key_fields=['Email']
)

print(f"Created: {len(upsert_result['createdRecords'])}")
print(f"Updated: {len(upsert_result['updatedRecords'])}")

# Upsert with multiple key fields
upsert_result = table.batch_upsert(
    [
        {
            'First Name': 'John',
            'Last Name': 'Doe',
            'Email': 'john@example.com',
            'Company': 'Acme Inc'
        }
    ],
    key_fields=['First Name', 'Last Name']
)

# With typecast
upsert_typecast = table.batch_upsert(
    [{'Email': 'user@example.com', 'Count': '50'}],
    key_fields=['Email'],
    typecast=True
)
```

### Deleting Records

#### Delete Single Record

```python
# Delete by record ID
deleted = table.delete('recwPQIfs4wKPyc9D')

print(f"Deleted record: {deleted['id']}")
print(f"Deleted: {deleted['deleted']}")  # True

# Check if deletion was successful
if deleted['deleted']:
    print("Record successfully deleted")
```

#### Batch Delete (Multiple Records)

```python
# Delete up to 10 records at once
deleted_records = table.batch_delete([
    'recXXXXXXXXXXXXXX',
    'recYYYYYYYYYYYYYY',
    'recZZZZZZZZZZZZZZ'
])

print(f"Deleted {len(deleted_records)} records")

for record in deleted_records:
    print(f"Deleted: {record['id']}")

# Delete more than 10 records
record_ids = ['rec1', 'rec2', 'rec3', ...]  # Many record IDs
all_deleted = []

for i in range(0, len(record_ids), 10):
    batch = record_ids[i:i+10]
    deleted = table.batch_delete(batch)
    all_deleted.extend(deleted)

print(f"Deleted {len(all_deleted)} records total")
```

### Working with Different Field Types

#### Text Fields

```python
record = table.create({
    'Single Line Text': 'Short text',
    'Long Text': 'This is a much longer text\nwith multiple lines',
    'Email': 'user@example.com',
    'URL': 'https://example.com',
    'Phone': '+1-555-0100'
})
```

#### Number Fields

```python
record = table.create({
    'Number': 42,
    'Currency': 99.99,
    'Percent': 0.75,
    'Rating': 5
})
```

#### Date and Time Fields

```python
from datetime import datetime, date

record = table.create({
    'Date': '2025-10-25',
    'DateTime': '2025-10-25T14:30:00.000Z',
    'Date Object': date(2025, 10, 25).isoformat(),
    'DateTime Object': datetime.now().isoformat()
})

# Access date fields
date_value = record['fields']['Date']
print(f"Date: {date_value}")
```

#### Checkbox (Boolean) Fields

```python
record = table.create({
    'Completed': True,
    'Active': False
})

# Access checkbox
is_completed = record['fields']['Completed']
if is_completed:
    print('Task is completed')
```

#### Single Select Fields

```python
record = table.create({
    'Status': 'In Progress',  # Must match exact option name
    'Priority': 'High'
})
```

#### Multiple Select Fields

```python
record = table.create({
    'Tags': ['Important', 'Urgent', 'Client Work']
})

# Access multiple select
tags = record['fields']['Tags']
print(f"Tags: {', '.join(tags)}")
```

#### Attachment Fields

```python
record = table.create({
    'Attachments': [
        {'url': 'https://example.com/image.jpg'},
        {'url': 'https://example.com/document.pdf'}
    ]
})

# Access attachments
attachments = record['fields']['Attachments']
for attachment in attachments:
    print(f"File: {attachment['filename']}")
    print(f"URL: {attachment['url']}")
    print(f"Size: {attachment['size']}")
    print(f"Type: {attachment['type']}")
```

#### Upload Attachments

```python
# Upload from file path
result = table.upload_attachment(
    'recAdw9EjV90xbZ',
    'Attachments',
    '/tmp/example.jpg'
)

# Upload from bytes/content
with open('/tmp/photo.jpg', 'rb') as f:
    content = f.read()

result = table.upload_attachment(
    'recAdw9EjV90xbZ',
    'Attachments',
    'photo.jpg',
    content=content,
    content_type='image/jpeg'
)
```

#### Linked Record Fields

```python
# Link to existing records by their IDs
record = table.create({
    'Name': 'Task with Links',
    'Related Tasks': [
        'recXXXXXXXXXXXXXX',
        'recYYYYYYYYYYYYYY'
    ]
})

# Access linked records
linked_records = record['fields']['Related Tasks']
print(f"Linked record IDs: {linked_records}")
```

#### Collaborator Fields

```python
record = table.create({
    'Assignee': {
        'id': 'usrXXXXXXXXXXXXXX',
        'email': 'user@example.com'
    },
    'Collaborators': [
        {'id': 'usrXXXXXXXXXXXXXX'},
        {'id': 'usrYYYYYYYYYYYYYY'}
    ]
})
```

### Working with Comments

```python
# Get all comments on a record
comments = table.comments('recMNxslc6jG0XedV')

for comment in comments:
    print(f"Author: {comment['author']['email']}")
    print(f"Text: {comment['text']}")
    print(f"Created: {comment['createdTime']}")

# Add comment
comment = table.add_comment(
    'recMNxslc6jG0XedV',
    'This is a comment'
)

# Add comment with user mention
comment = table.add_comment(
    'recMNxslc6jG0XedV',
    'Hello, @[usrVMNxslc6jG0Xed]! Please review this.'
)
```

## ORM (Object-Relational Mapping)

pyairtable provides ORM-style classes for type-safe database operations.

### Basic ORM Usage

```python
from pyairtable.orm import Model, fields
from pyairtable import Api

api = Api(os.environ['AIRTABLE_API_KEY'])

# Define model
class Task(Model):
    name = fields.TextField('Name')
    status = fields.SelectField('Status')
    priority = fields.SelectField('Priority')
    due_date = fields.DateField('Due Date')
    completed = fields.CheckboxField('Completed')
    tags = fields.MultipleSelectField('Tags')

    class Meta:
        base_id = 'appYourBaseId'
        table_name = 'Tasks'
        api_key = os.environ['AIRTABLE_API_KEY']

# Create record
task = Task(
    name='New Task',
    status='To Do',
    priority='High',
    due_date='2025-12-31'
)
task.save()

print(f"Created task: {task.id}")

# Retrieve all records
all_tasks = Task.all()

for task in all_tasks:
    print(f"{task.name}: {task.status}")

# Find by ID
task = Task.from_id('recXXXXXXXXXXXXXX')

# Update record
task.status = 'Done'
task.completed = True
task.save()

# Delete record
task.delete()
```

### ORM with Relationships

```python
from pyairtable.orm import Model, fields

class Project(Model):
    name = fields.TextField('Name')
    description = fields.TextField('Description')
    tasks = fields.LinkField('Tasks', 'Task', lazy=True)

    class Meta:
        base_id = 'appYourBaseId'
        table_name = 'Projects'
        api_key = os.environ['AIRTABLE_API_KEY']

class Task(Model):
    name = fields.TextField('Name')
    status = fields.SelectField('Status')
    project = fields.LinkField('Project', 'Project', lazy=True)

    class Meta:
        base_id = 'appYourBaseId'
        table_name = 'Tasks'
        api_key = os.environ['AIRTABLE_API_KEY']

# Create linked records
project = Project(name='New Project', description='Project description')
project.save()

task = Task(name='Task 1', status='To Do')
task.project = [project]
task.save()

# Access linked records
for task in project.tasks:
    print(f"Task: {task.name}")
```

## Schema Management

### Get Base Schema

```python
from pyairtable import Api

api = Api(os.environ['AIRTABLE_API_KEY'])
base = api.base('appYourBaseId')

# Get complete base schema
schema = base.schema()

# List all tables
for table_schema in schema.tables:
    print(f"Table: {table_schema.name}")
    print(f"ID: {table_schema.id}")

# Get specific table schema
table_schema = schema.table('tblXXXXXXXXXXXXXX')
print(f"Table name: {table_schema.name}")

# List fields
for field in table_schema.fields:
    print(f"Field: {field.name} ({field.type})")
```

### Get Table Schema

```python
table = api.table('appYourBaseId', 'Tasks')

# Get table schema
schema = table.schema()

print(f"Table name: {schema.name}")
print(f"Table ID: {schema.id}")
print(f"Primary field: {schema.primary_field_id}")

# List all fields
for field in schema.fields:
    print(f"Field: {field.name}")
    print(f"Type: {field.type}")
    print(f"ID: {field.id}")

    # Access field options
    if hasattr(field, 'options'):
        print(f"Options: {field.options}")
```

### Create Table

```python
base = api.base('appYourBaseId')

# Create new table with fields
new_table = base.create_table(
    'Employees',
    fields=[
        {
            'name': 'Name',
            'type': 'singleLineText'
        },
        {
            'name': 'Email',
            'type': 'email'
        },
        {
            'name': 'Department',
            'type': 'singleSelect',
            'options': {
                'choices': [
                    {'name': 'Engineering'},
                    {'name': 'Sales'},
                    {'name': 'Marketing'}
                ]
            }
        },
        {
            'name': 'Start Date',
            'type': 'date'
        },
        {
            'name': 'Active',
            'type': 'checkbox'
        }
    ]
)

print(f"Created table: {new_table.id}")
```

### Create Field

```python
table = api.table('appYourBaseId', 'Tasks')

# Create single line text field
field = table.create_field('Description', 'singleLineText')

# Create single select field with options
field = table.create_field(
    'Status',
    'singleSelect',
    options={
        'choices': [
            {'name': 'To Do'},
            {'name': 'In Progress'},
            {'name': 'Done'}
        ]
    }
)

# Create multiple select field
field = table.create_field(
    'Tags',
    'multipleSelects',
    options={
        'choices': [
            {'name': 'Important'},
            {'name': 'Urgent'},
            {'name': 'Low Priority'}
        ]
    }
)

# Create number field
field = table.create_field(
    'Progress',
    'number',
    options={'precision': 0}
)

# Create date field
field = table.create_field('Due Date', 'date')

# Create checkbox field
field = table.create_field('Completed', 'checkbox')
```

## Workspace and Base Management

### List All Bases

```python
api = Api(os.environ['AIRTABLE_API_KEY'])

# List all accessible bases
bases = api.bases()

for base in bases:
    print(f"Base: {base.name}")
    print(f"ID: {base.id}")
```

### Create Base

```python
# Create new base in workspace
new_base = api.create_base(
    workspace_id='wspMhESAta6clCCwF',
    name='My New Project Base',
    tables=[
        {
            'name': 'Tasks',
            'fields': [
                {'name': 'Name', 'type': 'singleLineText'},
                {'name': 'Status', 'type': 'singleSelect', 'options': {
                    'choices': [{'name': 'To Do'}, {'name': 'Done'}]
                }}
            ]
        }
    ]
)

print(f"Created base: {new_base.id}")
```

### Workspace Operations

```python
workspace = api.workspace('wspmhESAta6clCCwF')

# Create base in workspace
new_base = workspace.create_base(
    'New Project',
    tables=[
        {
            'name': 'Table 1',
            'fields': [{'name': 'Name', 'type': 'singleLineText'}]
        }
    ]
)

# Get workspace collaborators
collaborators = workspace.collaborators()

for collab in collaborators:
    print(f"User: {collab.user.email}")
    print(f"Permission: {collab.permission_level}")

# Move base to different workspace
workspace.move_base('appCwFmhESAta6clC', 'wspTargetWorkspace')
```

## Webhooks

### List Webhooks

```python
base = api.base('appYourBaseId')

# Get all webhooks
webhooks = base.webhooks()

for webhook in webhooks:
    print(f"Webhook ID: {webhook.id}")
    print(f"URL: {webhook.notification_url}")
```

### Get Webhook

```python
# Get specific webhook
webhook = base.webhook('ach00000000000001')

print(f"Webhook URL: {webhook.notification_url}")
print(f"Cursor: {webhook.cursor}")
```

### Create Webhook

```python
# Create webhook
webhook = base.add_webhook(
    'https://example.com/webhook',
    {
        'options': {
            'filters': {
                'dataTypes': ['tableData']
            }
        }
    }
)

print(f"Created webhook: {webhook.id}")
```

## Enterprise Features

### Enterprise API Access

```python
# Access enterprise features
enterprise = api.enterprise('entUBq2RGdihxl3vU')

# Get enterprise info
info = enterprise.info()
print(f"Enterprise: {info.workspace_ids}")

# Iterate through audit log
for page in enterprise.audit_log(sort_asc=True, page_size=50):
    for event in page.events:
        print(f"Event: {event.action}")
        print(f"Actor: {event.actor.email}")
        print(f"Timestamp: {event.timestamp}")

# User management
users = enterprise.users(['usrID1', 'email@example.com'])

for user in users:
    print(f"User: {user.email}")
    print(f"State: {user.state}")

# Grant admin access
enterprise.grant_admin('usrID1', 'usrID2')

# Remove from enterprise
enterprise.remove_user(['usrID1'])
```

## Complete Examples

### Example 1: Task Management System

```python
import os
from pyairtable import Api
from datetime import datetime, timedelta

api = Api(os.environ['AIRTABLE_API_KEY'])
table = api.table('appTaskManager', 'Tasks')

def create_task(name, description, priority='Medium', assignee=None, due_date=None):
    """Create a new task"""
    task_data = {
        'Name': name,
        'Description': description,
        'Status': 'To Do',
        'Priority': priority,
        'Created': datetime.now().isoformat()
    }

    if assignee:
        task_data['Assignee'] = assignee

    if due_date:
        task_data['Due Date'] = due_date

    record = table.create(task_data)
    print(f"Created task: {record['id']}")
    return record

def get_active_tasks():
    """Get all tasks that are not done or cancelled"""
    records = table.all(
        formula="AND({Status} != 'Done', {Status} != 'Cancelled')",
        sort=['-Priority', 'Due Date']
    )

    tasks = []
    for record in records:
        tasks.append({
            'id': record['id'],
            'name': record['fields']['Name'],
            'status': record['fields']['Status'],
            'priority': record['fields'].get('Priority', 'Medium'),
            'assignee': record['fields'].get('Assignee'),
            'due_date': record['fields'].get('Due Date')
        })

    return tasks

def update_task_status(task_id, new_status):
    """Update task status"""
    updated = table.update(task_id, {
        'Status': new_status,
        'Last Modified': datetime.now().isoformat()
    })

    if new_status == 'Done':
        table.update(task_id, {
            'Completed': True,
            'Completed Date': datetime.now().isoformat()
        })

    print(f"Updated task {task_id} to {new_status}")
    return updated

def get_overdue_tasks():
    """Get all overdue tasks"""
    today = datetime.now().date().isoformat()

    records = table.all(
        formula=f"AND({{Status}} != 'Done', {{Due Date}} < '{today}')",
        sort=['Due Date']
    )

    return records

def bulk_update_status(task_ids, new_status):
    """Update multiple tasks at once"""
    updates = [
        {
            'id': task_id,
            'fields': {
                'Status': new_status,
                'Last Modified': datetime.now().isoformat()
            }
        }
        for task_id in task_ids
    ]

    # Process in batches of 10
    all_updated = []
    for i in range(0, len(updates), 10):
        batch = updates[i:i+10]
        updated = table.batch_update(batch)
        all_updated.extend(updated)

    print(f"Updated {len(all_updated)} tasks")
    return all_updated

def delete_old_completed_tasks(days=30):
    """Delete completed tasks older than specified days"""
    cutoff_date = (datetime.now() - timedelta(days=days)).date().isoformat()

    records = table.all(
        formula=f"AND({{Status}} = 'Done', {{Completed Date}} < '{cutoff_date}')"
    )

    record_ids = [record['id'] for record in records]

    # Delete in batches of 10
    for i in range(0, len(record_ids), 10):
        batch = record_ids[i:i+10]
        table.batch_delete(batch)

    print(f"Deleted {len(record_ids)} old completed tasks")

def get_tasks_by_assignee(assignee_email):
    """Get all tasks for a specific assignee"""
    records = table.all(
        formula=f"{{Assignee}} = '{assignee_email}'",
        sort=['-Priority', 'Due Date']
    )

    return records

# Usage examples
if __name__ == '__main__':
    # Create new task
    task = create_task(
        name='Complete project documentation',
        description='Write comprehensive docs for the project',
        priority='High',
        due_date='2025-11-01'
    )

    # Get active tasks
    active = get_active_tasks()
    print(f"Found {len(active)} active tasks")

    # Update task status
    # update_task_status(task['id'], 'In Progress')

    # Get overdue tasks
    overdue = get_overdue_tasks()
    print(f"Found {len(overdue)} overdue tasks")

    # Bulk update
    # task_ids = ['rec1', 'rec2', 'rec3']
    # bulk_update_status(task_ids, 'Done')
```

### Example 2: Contact Management with Upsert

```python
import os
from pyairtable import Api

api = Api(os.environ['AIRTABLE_API_KEY'])
table = api.table('appContactManager', 'Contacts')

def find_contact_by_email(email):
    """Find contact by email address"""
    record = table.first(formula=f"{{Email}} = '{email}'")
    return record

def upsert_contact(contact_data):
    """Create or update contact based on email"""
    result = table.batch_upsert(
        [contact_data],
        key_fields=['Email']
    )

    if result['createdRecords']:
        print(f"Created new contact: {result['createdRecords'][0]['id']}")
        return {'action': 'created', 'record': result['createdRecords'][0]}
    else:
        print(f"Updated existing contact: {result['updatedRecords'][0]['id']}")
        return {'action': 'updated', 'record': result['updatedRecords'][0]}

def get_contacts_by_company(company_name):
    """Get all contacts from a specific company"""
    records = table.all(
        formula=f"{{Company}} = '{company_name}'",
        sort=['Last Name', 'First Name']
    )

    contacts = []
    for record in records:
        contacts.append({
            'id': record['id'],
            'name': f"{record['fields']['First Name']} {record['fields']['Last Name']}",
            'email': record['fields']['Email'],
            'phone': record['fields'].get('Phone'),
            'company': record['fields']['Company']
        })

    return contacts

def batch_import_contacts(contacts_list):
    """Import multiple contacts with upsert"""
    result = table.batch_upsert(
        contacts_list,
        key_fields=['Email']
    )

    print(f"Created: {len(result['createdRecords'])}")
    print(f"Updated: {len(result['updatedRecords'])}")

    return result

def export_all_contacts():
    """Export all contacts to list"""
    all_contacts = []

    for page in table.iterate(page_size=100):
        for record in page:
            all_contacts.append({
                'id': record['id'],
                'first_name': record['fields'].get('First Name'),
                'last_name': record['fields'].get('Last Name'),
                'email': record['fields'].get('Email'),
                'phone': record['fields'].get('Phone'),
                'company': record['fields'].get('Company'),
                'created': record['createdTime']
            })

    return all_contacts

def tag_contacts(contact_ids, tags):
    """Add tags to multiple contacts"""
    updates = [
        {
            'id': contact_id,
            'fields': {'Tags': tags}
        }
        for contact_id in contact_ids
    ]

    # Process in batches of 10
    all_updated = []
    for i in range(0, len(updates), 10):
        batch = updates[i:i+10]
        updated = table.batch_update(batch)
        all_updated.extend(updated)

    return all_updated

# Usage examples
if __name__ == '__main__':
    # Upsert single contact
    result = upsert_contact({
        'First Name': 'John',
        'Last Name': 'Doe',
        'Email': 'john.doe@example.com',
        'Phone': '+1-555-0100',
        'Company': 'Acme Inc'
    })

    # Find contact by email
    contact = find_contact_by_email('john.doe@example.com')
    if contact:
        print(f"Found: {contact['fields']['First Name']} {contact['fields']['Last Name']}")

    # Get contacts by company
    acme_contacts = get_contacts_by_company('Acme Inc')
    print(f"Found {len(acme_contacts)} contacts at Acme Inc")

    # Batch import
    contacts_to_import = [
        {
            'First Name': 'Jane',
            'Last Name': 'Smith',
            'Email': 'jane@example.com',
            'Company': 'Tech Corp'
        },
        {
            'First Name': 'Bob',
            'Last Name': 'Johnson',
            'Email': 'bob@example.com',
            'Company': 'Startup LLC'
        }
    ]
    batch_import_contacts(contacts_to_import)

    # Export all contacts
    all_contacts = export_all_contacts()
    print(f"Exported {len(all_contacts)} contacts")
```

### Example 3: E-commerce Inventory Management

```python
import os
from pyairtable import Api
from datetime import datetime

api = Api(os.environ['AIRTABLE_API_KEY'])
products_table = api.table('appInventory', 'Products')
orders_table = api.table('appInventory', 'Orders')

def check_inventory(product_id):
    """Check product availability"""
    product = products_table.get(product_id)

    return {
        'id': product['id'],
        'name': product['fields']['Name'],
        'sku': product['fields']['SKU'],
        'quantity': product['fields']['Quantity in Stock'],
        'available': product['fields']['Quantity in Stock'] > 0,
        'price': product['fields']['Price']
    }

def update_stock_quantity(product_id, quantity_change):
    """Update stock after purchase or restock"""
    product = products_table.get(product_id)
    current_stock = product['fields']['Quantity in Stock']
    new_stock = current_stock + quantity_change

    if new_stock < 0:
        raise ValueError(f"Insufficient stock. Available: {current_stock}")

    updated = products_table.update(product_id, {
        'Quantity in Stock': new_stock,
        'Last Updated': datetime.now().isoformat()
    })

    print(f"Updated stock for {product['fields']['Name']}: {current_stock} -> {new_stock}")
    return updated

def get_low_stock_products(threshold=10):
    """Get products below stock threshold"""
    records = products_table.all(
        formula=f"{{Quantity in Stock}} < {threshold}",
        sort=['Quantity in Stock']
    )

    low_stock = []
    for record in records:
        low_stock.append({
            'id': record['id'],
            'name': record['fields']['Name'],
            'sku': record['fields']['SKU'],
            'quantity': record['fields']['Quantity in Stock'],
            'reorder_level': record['fields'].get('Reorder Level', threshold)
        })

    return low_stock

def create_order(order_data):
    """Create order and update inventory"""
    # Create order record
    order = orders_table.create({
        'Order Number': order_data['order_number'],
        'Customer Name': order_data['customer_name'],
        'Customer Email': order_data['customer_email'],
        'Status': 'Pending',
        'Total Amount': order_data['total_amount'],
        'Order Date': datetime.now().isoformat()
    })

    # Update inventory for each product
    try:
        for item in order_data['items']:
            update_stock_quantity(item['product_id'], -item['quantity'])

        # Update order status to confirmed
        orders_table.update(order['id'], {'Status': 'Confirmed'})

        print(f"Created order: {order['id']}")
        return order
    except ValueError as e:
        # Rollback: delete order if inventory update fails
        orders_table.delete(order['id'])
        raise e

def get_orders_by_status(status):
    """Get all orders with specific status"""
    records = orders_table.all(
        formula=f"{{Status}} = '{status}'",
        sort=['-Order Date']
    )

    orders = []
    for record in records:
        orders.append({
            'id': record['id'],
            'order_number': record['fields']['Order Number'],
            'customer': record['fields']['Customer Name'],
            'total': record['fields']['Total Amount'],
            'date': record['fields']['Order Date']
        })

    return orders

def restock_products(restock_list):
    """Bulk restock multiple products"""
    updates = []

    for item in restock_list:
        product = products_table.get(item['product_id'])
        current_stock = product['fields']['Quantity in Stock']
        new_stock = current_stock + item['quantity']

        updates.append({
            'id': item['product_id'],
            'fields': {
                'Quantity in Stock': new_stock,
                'Last Restocked': datetime.now().isoformat()
            }
        })

    # Process in batches of 10
    all_updated = []
    for i in range(0, len(updates), 10):
        batch = updates[i:i+10]
        updated = products_table.batch_update(batch)
        all_updated.extend(updated)

    print(f"Restocked {len(all_updated)} products")
    return all_updated

def get_sales_report(start_date, end_date):
    """Get orders within date range"""
    records = orders_table.all(
        formula=f"AND({{Order Date}} >= '{start_date}', {{Order Date}} <= '{end_date}')",
        sort=['Order Date']
    )

    total_sales = sum(
        record['fields']['Total Amount']
        for record in records
        if 'Total Amount' in record['fields']
    )

    return {
        'total_orders': len(records),
        'total_sales': total_sales,
        'orders': records
    }

# Usage examples
if __name__ == '__main__':
    # Check inventory
    inventory = check_inventory('recProductId123')
    print(f"{inventory['name']}: {inventory['quantity']} in stock")

    # Get low stock products
    low_stock = get_low_stock_products(threshold=15)
    print(f"Found {len(low_stock)} products below threshold")

    # Create order
    order_data = {
        'order_number': 'ORD-2025-001',
        'customer_name': 'John Doe',
        'customer_email': 'john@example.com',
        'total_amount': 299.99,
        'items': [
            {'product_id': 'recProduct1', 'quantity': 2},
            {'product_id': 'recProduct2', 'quantity': 1}
        ]
    }
    # create_order(order_data)

    # Get pending orders
    pending = get_orders_by_status('Pending')
    print(f"Found {len(pending)} pending orders")

    # Restock products
    restock_list = [
        {'product_id': 'recProduct1', 'quantity': 50},
        {'product_id': 'recProduct2', 'quantity': 30}
    ]
    # restock_products(restock_list)
```

## Rate Limits and Error Handling

Airtable enforces a rate limit of **5 requests per second per base**.

### Automatic Retry with Rate Limiting

```python
from pyairtable import Api, retry_strategy

# Enable automatic retry for rate limits
api = Api(
    os.environ['AIRTABLE_API_KEY'],
    retry_strategy=True
)

# Custom retry configuration
custom_retry = retry_strategy(
    total=10,
    status_forcelist=(429, 500, 502, 503, 504),
    backoff_factor=0.2
)

api = Api(
    os.environ['AIRTABLE_API_KEY'],
    retry_strategy=custom_retry
)
```

### Manual Error Handling

```python
from pyairtable import Api
from pyairtable.api.types import APIError
import time

api = Api(os.environ['AIRTABLE_API_KEY'])
table = api.table('appBaseId', 'Tasks')

# Basic error handling
try:
    record = table.get('recXXXXXXXXXXXXXX')
except APIError as e:
    if e.status_code == 404:
        print('Record not found')
    elif e.status_code == 401:
        print('Authentication failed')
    elif e.status_code == 429:
        print('Rate limit exceeded')
    else:
        print(f'API error: {e}')
except Exception as e:
    print(f'Unexpected error: {e}')

# Retry with exponential backoff
def create_with_retry(record_data, max_retries=3):
    for attempt in range(max_retries):
        try:
            return table.create(record_data)
        except APIError as e:
            if e.status_code == 429 and attempt < max_retries - 1:
                wait_time = 2 ** attempt
                print(f"Rate limited. Retrying in {wait_time}s...")
                time.sleep(wait_time)
            else:
                raise

# Comprehensive error handling
def safe_create(record_data):
    try:
        record = table.create(record_data)
        return {'success': True, 'record': record}
    except APIError as e:
        return {
            'success': False,
            'error': {
                'message': str(e),
                'status_code': e.status_code,
                'type': e.type
            }
        }
    except Exception as e:
        return {
            'success': False,
            'error': {
                'message': str(e),
                'type': 'unknown'
            }
        }
```

## Common Formulas Reference

```python
# Exact match
formula = "{Status} = 'Active'"

# Not equal
formula = "{Status} != 'Done'"

# Greater than / Less than
formula = "{Price} > 100"
formula = "{Stock} <= 10"

# String contains (case-insensitive)
formula = "SEARCH('urgent', LOWER({Notes})) > 0"

# Is empty
formula = "{Email} = ''"
formula = "OR({Email} = BLANK())"

# Is not empty
formula = "NOT({Email} = '')"
formula = "{Email} != ''"

# AND condition
formula = "AND({Status} = 'Active', {Priority} = 'High')"

# OR condition
formula = "OR({Status} = 'Urgent', {Priority} = 'High')"

# Date comparisons
formula = "{Created} > '2025-01-01'"
formula = "{Due Date} < TODAY()"
formula = "{Modified} >= DATEADD(TODAY(), -7, 'days')"

# Multiple conditions
formula = "AND({Status} = 'Active', OR({Priority} = 'High', {Due Date} < TODAY()))"

# Check if field is in a list
formula = "OR({Status} = 'Active', {Status} = 'In Progress', {Status} = 'Review')"

# Numeric range
formula = "AND({Price} >= 10, {Price} <= 100)"

# Using variables
email = 'user@example.com'
formula = f"{{Email}} = '{email}'"

search_term = 'urgent'
formula = f"SEARCH(LOWER('{search_term}'), LOWER({{Notes}})) > 0"
```
