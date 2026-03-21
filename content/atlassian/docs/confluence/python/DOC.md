---
name: confluence
description: "Confluence Cloud API coding guidelines for Python using the atlassian-python-api library"
metadata:
  languages: "python"
  versions: "4.0.7"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "atlassian,confluence,wiki,documentation,collaboration"
---

# Confluence Cloud API Coding Guidelines (Python)

You are a **Confluence Cloud API coding expert**. Help me write correct, idiomatic Python code that calls the Atlassian Confluence Cloud REST API using the atlassian-python-api library.

Use **only official Atlassian sources** for API behavior, fields, and constraints. This guide summarizes key patterns for Python applications.

> Ground truth: Atlassian Confluence Cloud REST API documentation at developer.atlassian.com/cloud/confluence/

## Golden Rule: Use atlassian-python-api Library

**CRITICAL:** Use the `atlassian-python-api` package (version 4.0.7 or later) for interacting with Confluence Cloud REST API. This is the most comprehensive and actively maintained community library for Atlassian products.

**DO NOT use:**
- `pyatlassian` (different package, less maintained)
- `confluence-api` (JavaScript package, not Python)
- Direct REST API calls with `requests` without a client library (error-prone)

**Install:**
```bash
pip install atlassian-python-api
# or
poetry add atlassian-python-api
# or
uv pip install atlassian-python-api
```

The library supports Python 3.8+ and provides comprehensive coverage of Confluence Cloud and Server APIs.

## Authentication

Confluence Cloud requires API token authentication. Username/password authentication is deprecated.

### API Token Authentication - Required for Cloud

Generate an API token from your Atlassian Account Settings (https://id.atlassian.com/manage-profile/security/api-tokens).

```python
from atlassian import Confluence

confluence = Confluence(
    url='https://your-domain.atlassian.net',
    username='your-email@example.com',
    password='your-api-token',  # This is your API token, not your password
    cloud=True
)
```

**IMPORTANT:** For Confluence Cloud, you must:
1. Use your email address as the username
2. Use an API token (not your account password) as the password
3. Set `cloud=True`

### Environment Variables Setup

```bash
# .env file
CONFLUENCE_URL=https://your-domain.atlassian.net
CONFLUENCE_USERNAME=your-email@example.com
CONFLUENCE_API_TOKEN=your_api_token_here
```

**Loading in Python:**

```python
import os
from dotenv import load_dotenv
from atlassian import Confluence

load_dotenv()

confluence = Confluence(
    url=os.getenv('CONFLUENCE_URL'),
    username=os.getenv('CONFLUENCE_USERNAME'),
    password=os.getenv('CONFLUENCE_API_TOKEN'),
    cloud=True
)
```

### OAuth 2.0 Authentication

For apps using OAuth 2.0 access tokens:

```python
from atlassian import Confluence

confluence = Confluence(
    url='https://your-domain.atlassian.net',
    token='your-oauth-access-token',
    cloud=True
)
```

### Server/Data Center Authentication (On-Premise)

For self-hosted Confluence instances:

```python
from atlassian import Confluence

confluence = Confluence(
    url='https://confluence.company.com',
    username='your-username',
    password='your-password',
    cloud=False  # Important for Server/Data Center
)
```

## Initialization Patterns

### Basic Initialization

```python
from atlassian import Confluence

confluence = Confluence(
    url='https://your-domain.atlassian.net',
    username='your-email@example.com',
    password='your-api-token',
    cloud=True
)
```

### With Timeout Configuration

```python
confluence = Confluence(
    url='https://your-domain.atlassian.net',
    username='your-email@example.com',
    password='your-api-token',
    cloud=True,
    timeout=60  # Seconds
)
```

### With Proxy Support

```python
confluence = Confluence(
    url='https://your-domain.atlassian.net',
    username='your-email@example.com',
    password='your-api-token',
    cloud=True,
    proxies={
        'http': 'http://proxy.company.com:8080',
        'https': 'https://proxy.company.com:8080',
    }
)
```

### Verify SSL Certificates

```python
# Disable SSL verification (not recommended for production)
confluence = Confluence(
    url='https://your-domain.atlassian.net',
    username='your-email@example.com',
    password='your-api-token',
    cloud=True,
    verify_ssl=False
)
```

## Pages API

### Get Page by ID

```python
page = confluence.get_page_by_id(
    page_id='123456789',
    expand='body.storage,version,space'
)

print(f"Title: {page['title']}")
print(f"Space: {page['space']['key']}")
print(f"Version: {page['version']['number']}")
print(f"Content: {page['body']['storage']['value']}")
```

### Get Page by Title

```python
page = confluence.get_page_by_title(
    space='DEMO',
    title='Getting Started'
)

if page:
    print(f"Page ID: {page['id']}")
    print(f"Title: {page['title']}")
else:
    print("Page not found")
```

### Get Page ID Only

```python
page_id = confluence.get_page_id(
    space='DEMO',
    title='Getting Started'
)

if page_id:
    print(f"Found page: {page_id}")
```

### Get All Pages from Space

```python
pages = confluence.get_all_pages_from_space(
    space='DEMO',
    start=0,
    limit=100,
    status='current',
    expand='version,space'
)

for page in pages:
    print(f"{page['title']} - {page['id']}")
```

### Get All Pages as Generator (Memory Efficient)

```python
for page in confluence.get_all_pages_from_space_as_generator(
    space='DEMO',
    expand='body.storage,version'
):
    print(f"Processing: {page['title']}")
    # Process each page without loading all into memory
```

### Create a Page

```python
page = confluence.create_page(
    space='DEMO',
    title='Getting Started Guide',
    body='<h1>Welcome</h1><p>This is the introduction to our project.</p>',
    parent_id=None,  # Optional: specify parent page ID
    type='page',
    representation='storage'
)

print(f"Created page ID: {page['id']}")
print(f"URL: {page['_links']['webui']}")
```

### Create a Child Page

```python
parent_id = confluence.get_page_id(space='DEMO', title='Parent Page')

child_page = confluence.create_page(
    space='DEMO',
    title='Child Page',
    body='<p>This is a child page.</p>',
    parent_id=parent_id
)

print(f"Created child page: {child_page['id']}")
```

### Update a Page

**IMPORTANT:** Always increment the version number when updating.

```python
# Get current page to retrieve version
page = confluence.get_page_by_id(
    page_id='123456789',
    expand='version,space'
)

# Update with incremented version
updated = confluence.update_page(
    page_id='123456789',
    title='Updated Title',
    body='<h1>Updated Content</h1><p>New information here.</p>',
    version_number=page['version']['number'] + 1,
    representation='storage'
)

print(f"Updated to version {updated['version']['number']}")
```

### Update Page Content Only (Keep Title)

```python
page = confluence.get_page_by_id(
    page_id='123456789',
    expand='version,body.storage'
)

confluence.update_page(
    page_id='123456789',
    title=page['title'],  # Keep existing title
    body='<p>Only the content changed.</p>',
    version_number=page['version']['number'] + 1
)
```

### Append to Page Content

```python
page = confluence.get_page_by_id(
    page_id='123456789',
    expand='version,body.storage'
)

current_content = page['body']['storage']['value']
new_content = current_content + '<p>Additional content appended.</p>'

confluence.update_page(
    page_id='123456789',
    title=page['title'],
    body=new_content,
    version_number=page['version']['number'] + 1
)
```

### Delete a Page

```python
confluence.remove_page(
    page_id='123456789',
    status=None  # Use 'trashed' for soft delete
)

print("Page deleted successfully")
```

### Move Page to Trash (Soft Delete)

```python
confluence.set_page_property(
    page_id='123456789',
    data={'status': 'trashed'}
)
```

### Get Page Children

```python
children = confluence.get_page_child_by_type(
    page_id='123456789',
    type='page',
    start=0,
    limit=50
)

for child in children:
    print(f"Child: {child['title']} ({child['id']})")
```

### Get Page Descendants

```python
descendants = confluence.get_page_descendants(
    page_id='123456789',
    type='page'
)

for page in descendants:
    print(f"Descendant: {page['title']}")
```

### Get Page Ancestors

```python
page = confluence.get_page_by_id(
    page_id='123456789',
    expand='ancestors'
)

for ancestor in page.get('ancestors', []):
    print(f"Ancestor: {ancestor['title']} ({ancestor['id']})")
```

### Get Page History

```python
history = confluence.history(page_id='123456789')

print(f"Created by: {history['createdBy']['displayName']}")
print(f"Created date: {history['createdDate']}")
print(f"Latest version: {history['latest']['number']}")
```

### Get Page Version Information

```python
version = confluence.get_page_version(
    page_id='123456789',
    version_number=None  # None for latest, or specify version number
)

print(f"Version: {version['number']}")
print(f"Modified by: {version['by']['displayName']}")
print(f"Modified when: {version['when']}")
```

## Spaces API

### Get Space

```python
space = confluence.get_space(
    space_key='DEMO',
    expand='description.plain,homepage'
)

print(f"Name: {space['name']}")
print(f"Key: {space['key']}")
print(f"Type: {space['type']}")
if 'description' in space:
    print(f"Description: {space['description']['plain']['value']}")
```

### Get All Spaces

```python
spaces = confluence.get_all_spaces(
    start=0,
    limit=100,
    expand='description.plain'
)

for space in spaces['results']:
    print(f"{space['key']}: {space['name']}")
```

### Get All Spaces as Generator

```python
for space in confluence.get_all_spaces_as_generator():
    print(f"Space: {space['key']} - {space['name']}")
```

### Create a Space

```python
space = confluence.create_space(
    space_key='PROJ',
    space_name='Project Galaxy',
    description='Documentation for Project Galaxy'
)

print(f"Created space: {space['key']}")
print(f"URL: {space['_links']['webui']}")
```

### Update Space

```python
confluence.update_space(
    space_key='DEMO',
    name='Demo Space - Updated',
    description='Updated description'
)
```

### Delete Space

```python
confluence.delete_space(space_key='OLDSPACE')
print("Space deleted successfully")
```

### Get Space Content

```python
content = confluence.get_space_content(
    space_key='DEMO',
    depth='all',
    start=0,
    limit=100
)

for item in content.get('page', {}).get('results', []):
    print(f"Page: {item['title']} ({item['id']})")
```

### Get Space Homepage

```python
space = confluence.get_space(
    space_key='DEMO',
    expand='homepage'
)

if 'homepage' in space:
    homepage = space['homepage']
    print(f"Homepage: {homepage['title']} ({homepage['id']})")
```

## Search API

### CQL Search (Confluence Query Language)

```python
results = confluence.cql(
    cql='type=page AND space=DEMO AND title~"getting started"',
    start=0,
    limit=20,
    expand='content.space,content.version'
)

for result in results.get('results', []):
    content = result.get('content', {})
    print(f"{content.get('title')} - {content.get('id')}")
```

### Common CQL Query Patterns

```python
# Find all pages in a space
results = confluence.cql('type=page AND space=DEMO')

# Find pages by creator
results = confluence.cql('type=page AND creator=currentUser()')

# Find recently modified content
results = confluence.cql(
    'type=page AND lastModified > now("-7d") ORDER BY lastModified DESC',
    limit=10
)

# Find pages with specific label
results = confluence.cql('type=page AND label="documentation"')

# Complex search with AND/OR
results = confluence.cql(
    'type=page AND space IN (DEMO, PROJ) AND (title~"guide" OR text~"tutorial")'
)

# Find pages modified by specific user
results = confluence.cql('type=page AND contributor="john.doe@example.com"')

# Find pages created this month
results = confluence.cql('type=page AND created >= startOfMonth()')
```

### Search Content by Title

```python
# Using get_page_by_title for exact match
page = confluence.get_page_by_title(
    space='DEMO',
    title='Installation'
)

# Using CQL for partial match
results = confluence.cql('type=page AND title~"installation"')
```

### Full-Text Search

```python
results = confluence.cql(
    'type=page AND text~"kubernetes deployment"',
    limit=50
)

for result in results.get('results', []):
    content = result.get('content', {})
    print(f"Found: {content.get('title')}")
```

## Attachments API

### Get Attachments for Page

```python
attachments = confluence.get_attachments_from_content(
    page_id='123456789',
    start=0,
    limit=50,
    filename=None,  # Optional: filter by filename
    media_type=None  # Optional: filter by media type
)

for attachment in attachments.get('results', []):
    print(f"{attachment['title']} - {attachment['extensions']['fileSize']} bytes")
    print(f"Download: {attachment['_links']['download']}")
```

### Attach File to Page

```python
# Upload a file
result = confluence.attach_file(
    filename='/path/to/document.pdf',
    page_id='123456789',
    title='Document',  # Optional: display name
    comment='Uploaded via API'
)

print(f"Uploaded: {result['title']}")
```

### Attach File from Content

```python
# Upload file from bytes or file object
with open('/path/to/image.png', 'rb') as f:
    result = confluence.attach_content(
        content=f.read(),
        name='screenshot.png',
        content_type='image/png',
        page_id='123456789',
        comment='Screenshot of the dashboard'
    )
```

### Update Existing Attachment

```python
# Re-upload with same filename updates the attachment
result = confluence.attach_file(
    filename='/path/to/updated-document.pdf',
    page_id='123456789',
    title='Document',
    comment='Updated version'
)
```

### Download Attachment

```python
# Get attachment info first
attachments = confluence.get_attachments_from_content(page_id='123456789')

for attachment in attachments.get('results', []):
    if attachment['title'] == 'document.pdf':
        download_link = attachment['_links']['download']

        # Download the file
        import requests
        response = requests.get(
            f"{confluence.url}{download_link}",
            auth=(confluence.username, confluence.password)
        )

        with open('downloaded-document.pdf', 'wb') as f:
            f.write(response.content)

        break
```

### Delete Attachment

```python
confluence.delete_attachment(
    page_id='123456789',
    filename='old-document.pdf'
)
```

## Labels API

### Get Labels for Content

```python
labels = confluence.get_page_labels(page_id='123456789')

for label in labels.get('results', []):
    print(f"Label: {label['name']}")
```

### Add Label to Content

```python
confluence.add_label(
    page_id='123456789',
    label='documentation'
)

print("Label added successfully")
```

### Add Multiple Labels

```python
labels = ['documentation', 'getting-started', 'tutorial']

for label in labels:
    confluence.add_label(
        page_id='123456789',
        label=label
    )
```

### Remove Label from Content

```python
confluence.remove_label(
    page_id='123456789',
    label='old-label'
)
```

### Search Pages by Label

```python
results = confluence.cql('type=page AND label="documentation"')

for result in results.get('results', []):
    content = result.get('content', {})
    print(f"{content.get('title')} - {content.get('id')}")
```

## Comments API

### Get Comments for Page

```python
comments = confluence.get_page_comments(
    page_id='123456789',
    expand='body.storage',
    depth='all'
)

for comment in comments:
    print(f"Comment: {comment['title']}")
    print(f"Content: {comment['body']['storage']['value']}")
```

### Add Comment to Page

```python
comment = confluence.create_page(
    space='DEMO',
    title='Re: Page Title',  # Comment title
    body='<p>This is a helpful comment!</p>',
    parent_id='123456789',
    type='comment'
)

print(f"Comment added: {comment['id']}")
```

## User and Group APIs

### Get Current User

```python
user = confluence.get_current_user()

print(f"Display Name: {user['displayName']}")
print(f"Email: {user['email']}")
print(f"Account ID: {user['accountId']}")
```

### Get User by Username (Server/Data Center)

```python
user = confluence.get_user_details_by_username(
    username='john.doe'
)
```

### Get User by Account ID (Cloud)

```python
user = confluence.get_user_details_by_accountid(
    account_id='5a1234567890123456789012'
)

print(f"Display Name: {user['displayName']}")
```

### Get Group Members

```python
members = confluence.get_group_members(
    group_name='confluence-administrators',
    start=0,
    limit=50
)

for member in members:
    print(f"Member: {member['displayName']}")
```

### Check if User is in Group

```python
# Get all group members and check
members = confluence.get_group_members('confluence-users')
user_ids = [m['accountId'] for m in members]

is_member = 'user-account-id' in user_ids
print(f"User is member: {is_member}")
```

## Content Properties (Metadata)

### Get Content Properties

```python
properties = confluence.get_page_properties(page_id='123456789')

for prop in properties:
    print(f"Key: {prop['key']}")
    print(f"Value: {prop['value']}")
```

### Get Specific Content Property

```python
property_value = confluence.get_page_property(
    page_id='123456789',
    page_property_key='custom-metadata'
)

print(f"Property value: {property_value}")
```

### Set Content Property

```python
confluence.set_page_property(
    page_id='123456789',
    data={
        'key': 'custom-metadata',
        'value': {
            'lastReviewed': '2025-11-07',
            'reviewer': 'john.doe@example.com',
            'status': 'approved'
        }
    }
)
```

### Update Content Property

```python
# Get current property
current = confluence.get_page_property(
    page_id='123456789',
    page_property_key='custom-metadata'
)

# Update with new data
current['value']['status'] = 'needs-review'
current['version']['number'] += 1

confluence.update_page_property(
    page_id='123456789',
    page_property_key='custom-metadata',
    data=current
)
```

### Delete Content Property

```python
confluence.delete_page_property(
    page_id='123456789',
    page_property_key='custom-metadata'
)
```

## Advanced Page Operations

### Copy a Page

```python
# Get source page
source = confluence.get_page_by_id(
    page_id='123456789',
    expand='body.storage,space'
)

# Create copy
copy = confluence.create_page(
    space=source['space']['key'],
    title=f"{source['title']} (Copy)",
    body=source['body']['storage']['value'],
    parent_id=None
)

print(f"Created copy: {copy['id']}")
```

### Copy Page with Children

```python
def copy_page_tree(confluence, source_page_id, target_space, parent_id=None):
    """Recursively copy a page and all its children."""
    # Get source page
    source = confluence.get_page_by_id(
        page_id=source_page_id,
        expand='body.storage,space'
    )

    # Create copy
    copy = confluence.create_page(
        space=target_space,
        title=source['title'],
        body=source['body']['storage']['value'],
        parent_id=parent_id
    )

    # Copy children
    children = confluence.get_page_child_by_type(
        page_id=source_page_id,
        type='page'
    )

    for child in children:
        copy_page_tree(confluence, child['id'], target_space, copy['id'])

    return copy
```

### Move Page to Different Parent

```python
# Get current page
page = confluence.get_page_by_id(
    page_id='123456789',
    expand='version,body.storage,space,ancestors'
)

# Update with new parent
confluence.update_page(
    page_id='123456789',
    title=page['title'],
    body=page['body']['storage']['value'],
    parent_id='new-parent-id',
    version_number=page['version']['number'] + 1
)
```

### Export Page as PDF

```python
# Get page content in export view
page = confluence.get_page_by_id(
    page_id='123456789',
    expand='body.export_view'
)

export_html = page['body']['export_view']['value']

# Use a PDF library to convert HTML to PDF
# Example with pdfkit (requires wkhtmltopdf)
import pdfkit
pdfkit.from_string(export_html, 'output.pdf')
```

## Content Restrictions (Permissions)

### Get Content Restrictions

```python
restrictions = confluence.get_content_restrictions(
    content_id='123456789',
    expand='read.restrictions.user,update.restrictions.user'
)

print("Read restrictions:", restrictions.get('read'))
print("Update restrictions:", restrictions.get('update'))
```

### Add Read Restriction

```python
confluence.add_content_restriction(
    content_id='123456789',
    operation='read',
    restrictions={
        'user': [
            {'accountId': '5a1234567890123456789012'}
        ],
        'group': [
            {'name': 'confluence-users'}
        ]
    }
)
```

### Remove Content Restrictions

```python
confluence.remove_content_restriction(
    content_id='123456789',
    operation='read'
)
```

## Macros in Content

### Table of Contents Macro

```python
content = '''
<h1>Table of Contents</h1>
<ac:structured-macro ac:name="toc" ac:schema-version="1">
    <ac:parameter ac:name="maxLevel">3</ac:parameter>
</ac:structured-macro>
<h2>Section 1</h2>
<p>Content here...</p>
'''

confluence.create_page(
    space='DEMO',
    title='Documentation Index',
    body=content
)
```

### Status Macro

```python
content = '''
<p>Project status:
    <ac:structured-macro ac:name="status" ac:schema-version="1">
        <ac:parameter ac:name="colour">Green</ac:parameter>
        <ac:parameter ac:name="title">Active</ac:parameter>
    </ac:structured-macro>
</p>
'''
```

### Code Block Macro

```python
content = '''
<ac:structured-macro ac:name="code" ac:schema-version="1">
    <ac:parameter ac:name="language">python</ac:parameter>
    <ac:plain-text-body><![CDATA[
def greet(name):
    return f"Hello, {name}!"

print(greet("World"))
    ]]></ac:plain-text-body>
</ac:structured-macro>
'''
```

### Info Panel Macro

```python
content = '''
<ac:structured-macro ac:name="info" ac:schema-version="1">
    <ac:rich-text-body>
        <p>This is important information for users to know.</p>
    </ac:rich-text-body>
</ac:structured-macro>
'''
```

### Warning Panel Macro

```python
content = '''
<ac:structured-macro ac:name="warning" ac:schema-version="1">
    <ac:rich-text-body>
        <p><strong>Warning:</strong> This action cannot be undone.</p>
    </ac:rich-text-body>
</ac:structured-macro>
'''
```

### Note Panel Macro

```python
content = '''
<ac:structured-macro ac:name="note" ac:schema-version="1">
    <ac:rich-text-body>
        <p>Remember to save your work frequently.</p>
    </ac:rich-text-body>
</ac:structured-macro>
'''
```

### Excerpt Macro

```python
content = '''
<ac:structured-macro ac:name="excerpt" ac:schema-version="1">
    <ac:rich-text-body>
        <p>This is an excerpt that can be included in other pages.</p>
    </ac:rich-text-body>
</ac:structured-macro>
'''
```

### Include Page Macro

```python
content = '''
<ac:structured-macro ac:name="include" ac:schema-version="1">
    <ac:parameter ac:name=""><ac:link>
        <ri:page ri:content-title="Page to Include"/>
    </ac:link></ac:parameter>
</ac:structured-macro>
'''
```

## Error Handling

### Basic Error Handling

```python
from atlassian import Confluence
from requests.exceptions import HTTPError

try:
    page = confluence.get_page_by_id(page_id='invalid-id')
except HTTPError as e:
    print(f"HTTP Error: {e.response.status_code}")
    print(f"Message: {e.response.text}")
except Exception as e:
    print(f"Error: {str(e)}")
```

### Common HTTP Status Codes

- `400 Bad Request`: Invalid parameters or request body
- `401 Unauthorized`: Invalid or missing authentication
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Content or space does not exist
- `409 Conflict`: Version conflict (update with wrong version number)
- `429 Too Many Requests`: Rate limit exceeded

### Handling Version Conflicts

```python
def update_page_with_retry(confluence, page_id, new_content, max_retries=3):
    """Update page with automatic retry on version conflicts."""
    import time

    for attempt in range(max_retries):
        try:
            # Get current version
            page = confluence.get_page_by_id(
                page_id=page_id,
                expand='version,body.storage'
            )

            # Update with incremented version
            updated = confluence.update_page(
                page_id=page_id,
                title=page['title'],
                body=new_content,
                version_number=page['version']['number'] + 1
            )

            return updated

        except HTTPError as e:
            if e.response.status_code == 409 and attempt < max_retries - 1:
                print(f"Conflict detected, retrying ({attempt + 1}/{max_retries})...")
                time.sleep(1)
                continue
            raise

    raise Exception(f"Failed after {max_retries} retries")
```

### Rate Limit Handling

```python
import time
from requests.exceptions import HTTPError

def api_call_with_backoff(api_call, *args, max_retries=5, **kwargs):
    """Execute API call with exponential backoff on rate limits."""
    for attempt in range(max_retries):
        try:
            return api_call(*args, **kwargs)
        except HTTPError as e:
            if e.response.status_code == 429 and attempt < max_retries - 1:
                retry_after = int(e.response.headers.get('Retry-After', 2 ** attempt))
                print(f"Rate limited, waiting {retry_after}s before retry...")
                time.sleep(retry_after)
                continue
            raise

    raise Exception(f"Max retries ({max_retries}) exceeded")

# Usage
page = api_call_with_backoff(
    confluence.get_page_by_id,
    page_id='123456789'
)
```

### Comprehensive Error Handler

```python
from requests.exceptions import HTTPError, Timeout, ConnectionError

def safe_api_call(func, *args, **kwargs):
    """Wrapper for safe API calls with comprehensive error handling."""
    try:
        return func(*args, **kwargs)
    except HTTPError as e:
        status = e.response.status_code
        if status == 401:
            print("Authentication failed. Check your API token.")
        elif status == 403:
            print("Permission denied. Check user permissions.")
        elif status == 404:
            print("Resource not found.")
        elif status == 409:
            print("Version conflict. Refresh and try again.")
        elif status == 429:
            print("Rate limit exceeded. Slow down requests.")
        else:
            print(f"HTTP Error {status}: {e.response.text}")
        raise
    except Timeout:
        print("Request timed out. Try again later.")
        raise
    except ConnectionError:
        print("Connection error. Check network connectivity.")
        raise
    except Exception as e:
        print(f"Unexpected error: {str(e)}")
        raise
```

## Bulk Operations

### Get Multiple Pages by IDs

```python
def get_pages_by_ids(confluence, page_ids):
    """Fetch multiple pages by their IDs."""
    pages = []

    for page_id in page_ids:
        try:
            page = confluence.get_page_by_id(
                page_id=page_id,
                expand='body.storage,version'
            )
            pages.append(page)
        except HTTPError as e:
            print(f"Failed to fetch page {page_id}: {e}")

    return pages
```

### Batch Create Pages

```python
import time

def create_multiple_pages(confluence, space_key, page_data):
    """Create multiple pages in a space."""
    created = []

    for data in page_data:
        page = confluence.create_page(
            space=space_key,
            title=data['title'],
            body=data['content'],
            parent_id=data.get('parent_id')
        )
        created.append(page)

        # Rate limit protection: wait between requests
        time.sleep(0.2)

    return created

# Usage
pages = [
    {'title': 'Page 1', 'content': '<p>Content 1</p>'},
    {'title': 'Page 2', 'content': '<p>Content 2</p>'},
    {'title': 'Page 3', 'content': '<p>Content 3</p>'},
]

created_pages = create_multiple_pages(confluence, 'DEMO', pages)
```

### Batch Update Pages

```python
def update_multiple_pages(confluence, updates):
    """Update multiple pages."""
    results = []

    for update in updates:
        page = confluence.get_page_by_id(
            page_id=update['id'],
            expand='version,body.storage'
        )

        updated = confluence.update_page(
            page_id=update['id'],
            title=update.get('title', page['title']),
            body=update['content'],
            version_number=page['version']['number'] + 1
        )

        results.append(updated)
        time.sleep(0.2)

    return results
```

### Export All Pages from Space

```python
import json

def export_space_to_json(confluence, space_key, output_file):
    """Export all pages from a space to JSON file."""
    pages_data = []

    for page in confluence.get_all_pages_from_space_as_generator(
        space=space_key,
        expand='body.storage,version'
    ):
        pages_data.append({
            'id': page['id'],
            'title': page['title'],
            'content': page['body']['storage']['value'],
            'version': page['version']['number']
        })

    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(pages_data, f, indent=2, ensure_ascii=False)

    print(f"Exported {len(pages_data)} pages to {output_file}")
```

### Clone Space to Another Space

```python
def clone_space(confluence, source_space, target_space):
    """Clone all pages from source space to target space."""
    # Get all pages from source
    for page in confluence.get_all_pages_from_space_as_generator(
        space=source_space,
        expand='body.storage'
    ):
        try:
            confluence.create_page(
                space=target_space,
                title=page['title'],
                body=page['body']['storage']['value']
            )
            print(f"Cloned: {page['title']}")
            time.sleep(0.2)
        except Exception as e:
            print(f"Failed to clone {page['title']}: {e}")
```

## Performance and Optimization

### Use Expand Parameter Wisely

```python
# Bad - expands everything (slow)
page = confluence.get_page_by_id(
    page_id='123456789',
    expand='body.storage,body.view,body.editor,version,space,history,ancestors,descendants,container'
)

# Good - only what you need (fast)
page = confluence.get_page_by_id(
    page_id='123456789',
    expand='body.storage,version'
)
```

### Pagination Best Practices

```python
def get_all_pages_paginated(confluence, space_key, batch_size=100):
    """Get all pages with manual pagination control."""
    all_pages = []
    start = 0

    while True:
        batch = confluence.get_all_pages_from_space(
            space=space_key,
            start=start,
            limit=batch_size
        )

        if not batch:
            break

        all_pages.extend(batch)

        if len(batch) < batch_size:
            break

        start += batch_size

    return all_pages
```

### Use Generators for Large Datasets

```python
# Memory efficient - processes one page at a time
def process_all_pages(confluence, space_key):
    for page in confluence.get_all_pages_from_space_as_generator(space=space_key):
        # Process each page
        process_page(page)
        # Memory is released after each iteration

# Memory inefficient - loads all pages into memory
def process_all_pages_bad(confluence, space_key):
    all_pages = confluence.get_all_pages_from_space(space=space_key, limit=10000)
    for page in all_pages:
        process_page(page)
```

### Caching Strategies

```python
from functools import lru_cache
import time

class CachedConfluence:
    def __init__(self, confluence):
        self.confluence = confluence
        self.cache = {}
        self.cache_ttl = 300  # 5 minutes

    def get_page_cached(self, page_id):
        """Get page with caching."""
        cache_key = f"page_{page_id}"

        if cache_key in self.cache:
            cached_data, cached_time = self.cache[cache_key]
            if time.time() - cached_time < self.cache_ttl:
                return cached_data

        # Fetch fresh data
        page = self.confluence.get_page_by_id(page_id=page_id)
        self.cache[cache_key] = (page, time.time())

        return page

    def invalidate_cache(self, page_id=None):
        """Invalidate cache for specific page or all."""
        if page_id:
            cache_key = f"page_{page_id}"
            self.cache.pop(cache_key, None)
        else:
            self.cache.clear()
```

## Common Patterns

### Create or Update Page

```python
def create_or_update_page(confluence, space_key, title, content):
    """Create page if it doesn't exist, update if it does."""
    existing = confluence.get_page_by_title(
        space=space_key,
        title=title
    )

    if existing:
        # Update existing page
        page = confluence.get_page_by_id(
            page_id=existing['id'],
            expand='version'
        )

        return confluence.update_page(
            page_id=existing['id'],
            title=title,
            body=content,
            version_number=page['version']['number'] + 1
        )
    else:
        # Create new page
        return confluence.create_page(
            space=space_key,
            title=title,
            body=content
        )
```

### Build Table of Contents

```python
def build_table_of_contents(confluence, space_key):
    """Generate HTML table of contents for all pages in space."""
    pages = confluence.get_all_pages_from_space(space=space_key)

    toc_html = '<h2>Table of Contents</h2><ul>'

    for page in pages:
        toc_html += f'''
        <li>
            <ac:link>
                <ri:page ri:content-title="{page['title']}"/>
            </ac:link>
        </li>
        '''

    toc_html += '</ul>'

    return toc_html
```

### Sync Local Files to Confluence

```python
import os
from pathlib import Path

def sync_markdown_to_confluence(confluence, directory, space_key, parent_id=None):
    """Sync Markdown files to Confluence pages."""
    import markdown

    for md_file in Path(directory).glob('*.md'):
        # Read Markdown file
        with open(md_file, 'r', encoding='utf-8') as f:
            md_content = f.read()

        # Convert Markdown to HTML
        html_content = markdown.markdown(md_content)

        # Extract title from filename
        title = md_file.stem.replace('-', ' ').title()

        # Create or update page
        create_or_update_page(
            confluence,
            space_key=space_key,
            title=title,
            content=html_content
        )

        print(f"Synced: {title}")
```

## Content Body Formats

Confluence supports multiple body representations:

```python
# Get page with different body formats
page = confluence.get_page_by_id(
    page_id='123456789',
    expand='body.storage,body.view,body.export_view'
)

# Storage format (XHTML for create/update)
storage = page['body']['storage']['value']

# View format (HTML for rendering)
view = page['body']['view']['value']

# Export format (HTML optimized for export)
export = page['body']['export_view']['value']
```

## Common Mistakes to Avoid

- **Not incrementing version number** when updating (causes 409 conflict)
- **Using password instead of API token** for Cloud (causes 401 error)
- **Forgetting cloud=True** for Confluence Cloud instances
- **Not using expand parameter** to get needed fields (results in missing data)
- **Using storage format incorrectly** (must be valid XHTML)
- **Not handling pagination** for large result sets (misses data beyond first page)
- **Hardcoding credentials** in code instead of environment variables (security risk)
- **Not implementing rate limiting** in bulk operations (causes 429 errors)
- **Creating duplicate pages** without checking if title exists first
- **Not escaping HTML** in content (causes malformed XML errors)
- **Using get_all_pages_from_space** without limit for large spaces (memory issues)
- **Not using generators** for processing large datasets (runs out of memory)
- **Ignoring HTTP errors** without proper error handling (silent failures)
- **Not validating page exists** before operations (causes 404 errors)

## Reference Links

- **atlassian-python-api Documentation**: https://atlassian-python-api.readthedocs.io/
- **Confluence Module Docs**: https://atlassian-python-api.readthedocs.io/confluence.html
- **Confluence Cloud REST API v2**: https://developer.atlassian.com/cloud/confluence/rest/v2/
- **Confluence Cloud REST API v1**: https://developer.atlassian.com/cloud/confluence/rest/v1/
- **CQL (Confluence Query Language)**: https://developer.atlassian.com/cloud/confluence/advanced-searching-using-cql/
- **Storage Format Guide**: https://confluence.atlassian.com/doc/confluence-storage-format-790796544.html
- **Atlassian API Tokens**: https://id.atlassian.com/manage-profile/security/api-tokens
- **Rate Limits**: https://developer.atlassian.com/cloud/confluence/rate-limiting/
- **PyPI Package**: https://pypi.org/project/atlassian-python-api/
- **GitHub Repository**: https://github.com/atlassian-api/atlassian-python-api
