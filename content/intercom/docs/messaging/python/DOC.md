---
name: messaging
description: "Intercom Python SDK for customer messaging and chat support"
metadata:
  languages: "python"
  versions: "4.0.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "intercom,messaging,customer,chat,support"
---

# Intercom Python SDK (v4.0.0)

## Golden Rule

**ALWAYS use `python-intercom` version 4.0.0 or later.**

```bash
pip install python-intercom
```

**DO NOT use these deprecated or unofficial packages:**
- `intercom-python-2` (community fork)
- Old versions of `python-intercom` < 4.0.0 (different API structure)

The official Intercom Python SDK is `python-intercom`, maintained by Intercom at https://github.com/intercom/python-intercom

## Installation

```bash
pip install python-intercom
```

For Poetry projects:

```bash
poetry add python-intercom
```

**Environment Setup:**

```bash
# .env file
INTERCOM_ACCESS_TOKEN=your_access_token_here
```

**Loading environment variables:**

```python
import os
from dotenv import load_dotenv

load_dotenv()

token = os.getenv('INTERCOM_ACCESS_TOKEN')
```

## Initialization

**Basic Client Setup:**

```python
from intercom import Intercom

client = Intercom(
    token=os.getenv('INTERCOM_ACCESS_TOKEN')
)
```

**With custom configuration:**

```python
from intercom import Intercom

client = Intercom(
    token=os.getenv('INTERCOM_ACCESS_TOKEN'),
    timeout=120,  # Default is 60 seconds
    max_retries=3,  # Default is 2
)
```

**Async Client:**

```python
from intercom import AsyncIntercom
import asyncio

async def main():
    client = AsyncIntercom(
        token=os.getenv('INTERCOM_ACCESS_TOKEN')
    )

    contact = await client.contacts.find(id='65f9a5e4f5e5b40001234567')
    print(contact.email)

asyncio.run(main())
```

**API Version Selection:**

```python
client = Intercom(
    token=os.getenv('INTERCOM_ACCESS_TOKEN'),
    api_version='2.11',  # Default is latest stable
)
```

## Core API Surfaces

### Contacts

Contacts represent users, leads, and visitors in Intercom. They are the primary entities for customer communication.

**Create a contact:**

```python
contact = client.contacts.create(
    email='[email protected]',
    name='John Doe',
    phone='+1234567890',
    role='user',  # 'user' or 'lead'
)

print(contact.id)
```

**Create with custom attributes:**

```python
contact = client.contacts.create(
    email='[email protected]',
    name='Jane Smith',
    role='user',
    custom_attributes={
        'plan': 'premium',
        'signup_date': '2025-01-15',
        'total_spend': 599.99,
        'active': True,
    }
)
```

**Retrieve a contact:**

```python
contact = client.contacts.find(id='65f9a5e4f5e5b40001234567')

print(contact.email, contact.name)
```

**Update a contact:**

```python
updated = client.contacts.update(
    id='65f9a5e4f5e5b40001234567',
    name='John Updated',
    custom_attributes={
        'plan': 'enterprise',
    }
)
```

**Search contacts:**

```python
results = client.contacts.search(
    query={
        'field': 'email',
        'operator': '=',
        'value': '[email protected]',
    }
)

for contact in results.data:
    print(contact.name, contact.email)
```

**Search with filters:**

```python
results = client.contacts.search(
    query={
        'operator': 'AND',
        'value': [
            {
                'field': 'role',
                'operator': '=',
                'value': 'user',
            },
            {
                'field': 'created_at',
                'operator': '>',
                'value': 1704067200,  # Unix timestamp
            },
        ],
    },
    pagination={
        'per_page': 50,
    }
)
```

**List all contacts:**

```python
response = client.contacts.list(per_page=150)

# Iterate through paginated results
for contact in response:
    print(contact.email)
```

**Delete a contact:**

```python
client.contacts.delete(id='65f9a5e4f5e5b40001234567')
```

**Archive a contact:**

```python
archived = client.contacts.archive(id='65f9a5e4f5e5b40001234567')
```

**Unarchive a contact:**

```python
unarchived = client.contacts.unarchive(id='65f9a5e4f5e5b40001234567')
```

**Merge contacts:**

```python
merged = client.contacts.merge(
    from_='65f9a5e4f5e5b40001234567',
    into='65f9a5e4f5e5b40009876543'
)
```

### Companies

Companies group contacts together by organization. They're useful for B2B use cases.

**Create a company:**

```python
company = client.companies.create(
    company_id='company_123',
    name='Acme Corporation',
    website='https://acme.com',
    industry='Technology',
    size=500,
    custom_attributes={
        'plan_level': 'enterprise',
        'mrr': 50000,
    }
)
```

**Retrieve a company:**

```python
company = client.companies.find(company_id='company_123')
```

**Update a company:**

```python
updated = client.companies.update(
    company_id='company_123',
    name='Acme Corp',
    size=600,
    custom_attributes={
        'mrr': 60000,
    }
)
```

**List companies:**

```python
response = client.companies.list(per_page=50)

for company in response:
    print(company.name, company.company_id)
```

**Attach contact to company:**

```python
client.contacts.update(
    id='65f9a5e4f5e5b40001234567',
    companies=[
        {
            'company_id': 'company_123',
        }
    ]
)
```

**List company contacts:**

```python
contacts = client.companies.list_contacts(company_id='company_123')

for contact in contacts:
    print(contact.email)
```

**Delete a company:**

```python
client.companies.delete(company_id='company_123')
```

### Conversations

Conversations are threads of communication between contacts and your team.

**Create a conversation:**

```python
conversation = client.conversations.create(
    from_={
        'type': 'user',
        'id': '65f9a5e4f5e5b40001234567',
    },
    body='Hello, I need help with my account.'
)

print(conversation.id)
```

**Create conversation with user:**

```python
conversation = client.conversations.create_conversation(
    user_id='65f9a5e4f5e5b40001234567',
    body='I have a question about billing.'
)
```

**Retrieve a conversation:**

```python
conversation = client.conversations.find(id='123456')

print(conversation.state)  # 'open', 'closed', 'snoozed'
```

**Reply to conversation as admin:**

```python
reply = client.conversations.reply(
    id='123456',
    message_type='comment',
    type='admin',
    admin_id='987654',
    body='Thanks for reaching out! How can I help?'
)
```

**Reply with attachment:**

```python
reply = client.conversations.reply(
    id='123456',
    message_type='comment',
    type='admin',
    admin_id='987654',
    body='Here is the document you requested.',
    attachment_urls=[
        'https://example.com/document.pdf',
    ]
)
```

**Reply as user:**

```python
reply = client.conversations.reply(
    id='123456',
    message_type='comment',
    type='user',
    user_id='65f9a5e4f5e5b40001234567',
    body='Thank you for your help!'
)
```

**Search conversations:**

```python
results = client.conversations.search(
    query={
        'field': 'state',
        'operator': '=',
        'value': 'open',
    },
    pagination={
        'per_page': 50,
    }
)

for conv in results.conversations:
    print(conv.id, conv.created_at)
```

**Search with multiple filters:**

```python
results = client.conversations.search(
    query={
        'operator': 'AND',
        'value': [
            {
                'field': 'state',
                'operator': '=',
                'value': 'open',
            },
            {
                'field': 'updated_at',
                'operator': '>',
                'value': 1704067200,
            },
        ],
    },
    sort={
        'field': 'updated_at',
        'order': 'descending',
    }
)
```

**List conversations:**

```python
response = client.conversations.list()

for conversation in response:
    print(conversation.id, conversation.state)
```

**Close a conversation:**

```python
closed = client.conversations.close(
    id='123456',
    type='admin',
    admin_id='987654'
)
```

**Open a conversation:**

```python
opened = client.conversations.open(
    id='123456',
    type='admin',
    admin_id='987654'
)
```

**Snooze a conversation:**

```python
snoozed = client.conversations.snooze(
    id='123456',
    type='admin',
    admin_id='987654',
    snoozed_until=1704153600  # Unix timestamp
)
```

**Assign conversation to admin:**

```python
assigned = client.conversations.assign(
    id='123456',
    type='admin',
    admin_id='987654',
    assignee_id='111222'
)
```

**Assign to team:**

```python
assigned = client.conversations.assign(
    id='123456',
    type='admin',
    admin_id='987654',
    assignee_id='333444',
    assignment_type='team'
)
```

**Convert conversation to ticket:**

```python
ticket = client.conversations.convert_to_ticket(
    id='123456',
    ticket_type_id='100'
)
```

**Add tag to conversation:**

```python
client.conversations.attach_tag(
    conversation_id='123456',
    id='tag_789',
    admin_id='987654'
)
```

**Remove tag from conversation:**

```python
client.conversations.detach_tag(
    conversation_id='123456',
    id='tag_789',
    admin_id='987654'
)
```

### Messages

Send messages to contacts via Intercom.

**Send a message to contact:**

```python
message = client.messages.create(
    message_type='email',
    from_={
        'type': 'admin',
        'id': '987654',
    },
    to={
        'type': 'user',
        'id': '65f9a5e4f5e5b40001234567',
    },
    subject='Welcome to our platform',
    body='Thanks for signing up! Here are some tips to get started.'
)
```

**Send message by email:**

```python
message = client.messages.create(
    message_type='email',
    from_={
        'type': 'admin',
        'id': '987654',
    },
    to={
        'type': 'user',
        'email': '[email protected]',
    },
    subject='Your monthly report',
    body='Here is your activity summary for January.'
)
```

**Send in-app message:**

```python
message = client.messages.create(
    message_type='inapp',
    from_={
        'type': 'admin',
        'id': '987654',
    },
    to={
        'type': 'user',
        'id': '65f9a5e4f5e5b40001234567',
    },
    body='Check out our new features!'
)
```

**Send message with custom data:**

```python
message = client.messages.create(
    message_type='email',
    from_={
        'type': 'admin',
        'id': '987654',
    },
    to={
        'type': 'user',
        'id': '65f9a5e4f5e5b40001234567',
    },
    subject='Order confirmation',
    body='Your order has been confirmed.',
    template='plain',
    create_conversation_without_contact_reply=False
)
```

### Data Events

Track user behavior and custom events.

**Submit a data event:**

```python
import time

client.events.create(
    event_name='purchased_item',
    created_at=int(time.time()),
    user_id='65f9a5e4f5e5b40001234567',
    metadata={
        'item_name': 'Premium Plan',
        'item_price': 99.99,
        'currency': 'USD',
        'quantity': 1,
    }
)
```

**Event with email identifier:**

```python
client.events.create(
    event_name='signed_up',
    created_at=int(time.time()),
    email='[email protected]',
    metadata={
        'source': 'landing_page',
        'campaign': 'winter_2025',
    }
)
```

**Complex event metadata:**

```python
client.events.create(
    event_name='completed_onboarding',
    created_at=int(time.time()),
    user_id='65f9a5e4f5e5b40001234567',
    metadata={
        'steps_completed': 5,
        'time_taken_seconds': 320,
        'skipped_steps': ['profile_picture'],
        'completion_rate': 0.95,
        'device': 'mobile',
    }
)
```

**List events for user:**

```python
events = client.events.list(
    type='user',
    user_id='65f9a5e4f5e5b40001234567'
)

for event in events.events:
    print(event.event_name, event.created_at)
```

**Event summaries:**

```python
summary = client.events.summaries(
    user_id='65f9a5e4f5e5b40001234567',
    event_name='purchased_item'
)

print(summary.count, summary.first, summary.last)
```

### Tags

Organize and categorize contacts, companies, and conversations.

**Create a tag:**

```python
tag = client.tags.create(name='VIP Customer')

print(tag.id)
```

**Retrieve a tag:**

```python
tag = client.tags.find(id='tag_789')
```

**List all tags:**

```python
tags = client.tags.list()

for tag in tags.data:
    print(tag.name, tag.id)
```

**Tag a contact:**

```python
client.contacts.tag(
    contact_id='65f9a5e4f5e5b40001234567',
    id='tag_789'
)
```

**Tag a company:**

```python
client.companies.tag(
    company_id='company_123',
    id='tag_789'
)
```

**Untag a contact:**

```python
client.contacts.untag(
    contact_id='65f9a5e4f5e5b40001234567',
    id='tag_789'
)
```

**Delete a tag:**

```python
client.tags.delete(id='tag_789')
```

### Data Attributes

Define custom attributes for contacts and companies.

**Create a contact attribute:**

```python
attribute = client.data_attributes.create(
    name='subscription_tier',
    model='contact',
    data_type='string',
    options=['free', 'pro', 'enterprise'],
    description='Customer subscription level'
)
```

**Create a company attribute:**

```python
attribute = client.data_attributes.create(
    name='annual_revenue',
    model='company',
    data_type='float',
    description='Company annual revenue in USD'
)
```

**Create boolean attribute:**

```python
attribute = client.data_attributes.create(
    name='is_beta_tester',
    model='contact',
    data_type='boolean',
    description='Whether user is enrolled in beta program'
)
```

**Create date attribute:**

```python
attribute = client.data_attributes.create(
    name='trial_end_date',
    model='contact',
    data_type='date',
    description='Date when trial period ends'
)
```

**List data attributes:**

```python
attributes = client.data_attributes.list(model='contact')

for attr in attributes.data:
    print(attr.name, attr.data_type)
```

**Update an attribute:**

```python
updated = client.data_attributes.update(
    id='attr_123',
    description='Updated description',
    options=['free', 'pro', 'enterprise', 'custom']
)
```

### Notes

Add notes to contacts and companies for internal reference.

**Create a note for contact:**

```python
note = client.notes.create(
    contact_id='65f9a5e4f5e5b40001234567',
    admin_id='987654',
    body='Customer requested custom integration. Follow up next week.'
)
```

**Create a note for company:**

```python
note = client.notes.create(
    company_id='company_123',
    admin_id='987654',
    body='Contract renewal coming up in Q2 2025.'
)
```

**Retrieve a note:**

```python
note = client.notes.find(id='note_456')
```

**List notes for contact:**

```python
notes = client.contacts.list_notes(id='65f9a5e4f5e5b40001234567')

for note in notes.data:
    print(note.body, note.created_at)
```

### Segments

Query and retrieve segments (groups of contacts).

**List all segments:**

```python
segments = client.segments.list()

for segment in segments.segments:
    print(segment.name, segment.count)
```

**Retrieve a segment:**

```python
segment = client.segments.find(id='segment_999')

print(segment.name, segment.person_type)
```

**List contacts in segment:**

```python
contacts = client.segments.list_contacts(id='segment_999')

for contact in contacts:
    print(contact.email)
```

### Tickets

Manage customer support tickets.

**Create a ticket:**

```python
ticket = client.tickets.create(
    contact_id='65f9a5e4f5e5b40001234567',
    ticket_type_id='100',
    contacts=[
        {
            'id': '65f9a5e4f5e5b40001234567',
        }
    ],
    ticket_attributes={
        '_default_title_': 'Payment issue',
        '_default_description_': 'Customer unable to process payment',
    }
)
```

**Retrieve a ticket:**

```python
ticket = client.tickets.find(id='ticket_888')

print(ticket.ticket_state)
```

**Update a ticket:**

```python
updated = client.tickets.update(
    id='ticket_888',
    ticket_attributes={
        '_default_title_': 'Payment issue - RESOLVED',
    },
    ticket_state='submitted'
)
```

**Reply to ticket:**

```python
reply = client.tickets.reply(
    id='ticket_888',
    admin_id='987654',
    message_type='comment',
    body='Issue has been resolved. Payment processed successfully.'
)
```

**Search tickets:**

```python
results = client.tickets.search(
    query={
        'field': 'ticket_state',
        'operator': '=',
        'value': 'submitted',
    }
)

for ticket in results.tickets:
    print(ticket.id, ticket.ticket_attributes)
```

### Admins

Manage admin users and teams.

**List all admins:**

```python
admins = client.admins.list()

for admin in admins.admins:
    print(admin.name, admin.email)
```

**Retrieve an admin:**

```python
admin = client.admins.find(id='987654')

print(admin.name, admin.away_mode_enabled)
```

**Retrieve current admin:**

```python
me = client.admins.me()

print(me.name, me.email)
```

**Set away mode:**

```python
updated = client.admins.set_away_mode(
    admin_id='987654',
    away_mode_enabled=True,
    away_mode_reassign=True
)
```

**List teams:**

```python
teams = client.teams.list()

for team in teams.teams:
    print(team.name, team.id)
```

**Retrieve a team:**

```python
team = client.teams.find(id='333444')
```

### Articles

Manage help center articles.

**Create an article:**

```python
article = client.articles.create(
    title='Getting Started Guide',
    description='Learn how to use our platform',
    body='<h1>Welcome</h1><p>This guide will help you get started...</p>',
    author_id=987654,
    state='published'
)
```

**Retrieve an article:**

```python
article = client.articles.find(id='article_555')
```

**Update an article:**

```python
updated = client.articles.update(
    id='article_555',
    title='Getting Started Guide (Updated)',
    body='<h1>Welcome</h1><p>This updated guide...</p>'
)
```

**List articles:**

```python
response = client.articles.list(per_page=50)

for article in response:
    print(article.title, article.state)
```

**Delete an article:**

```python
client.articles.delete(id='article_555')
```

### Subscription Types

Manage subscription preferences for contacts.

**List subscription types:**

```python
types = client.subscription_types.list()

for sub_type in types.data:
    print(sub_type.id, sub_type.content_type)
```

**Subscribe contact:**

```python
client.contacts.subscribe(
    id='65f9a5e4f5e5b40001234567',
    subscription_type_id='sub_123'
)
```

**Unsubscribe contact:**

```python
client.contacts.unsubscribe(
    id='65f9a5e4f5e5b40001234567',
    subscription_type_id='sub_123'
)
```

## Error Handling

**Basic error handling:**

```python
from intercom import ApiError

try:
    contact = client.contacts.find(id='invalid_id')
except ApiError as e:
    print(f'Status: {e.status_code}')
    print(f'Message: {e.message}')
    print(f'Body: {e.body}')
```

**Handle specific error codes:**

```python
try:
    client.contacts.create(email='[email protected]')
except ApiError as e:
    if e.status_code == 400:
        print('Bad request:', e.message)
    elif e.status_code == 401:
        print('Unauthorized - check your token')
    elif e.status_code == 404:
        print('Resource not found')
    elif e.status_code == 429:
        print('Rate limit exceeded')
    else:
        print('Error:', e.message)
```

**Retry on rate limit:**

```python
import time
from intercom import ApiError

def create_contact_with_retry(data, max_retries=3):
    for attempt in range(1, max_retries + 1):
        try:
            return client.contacts.create(**data)
        except ApiError as e:
            if e.status_code == 429 and attempt < max_retries:
                delay = 2 ** attempt
                print(f'Rate limited, retrying in {delay}s...')
                time.sleep(delay)
                continue
            raise
```

## Pagination

**Auto-pagination with iterator:**

```python
response = client.contacts.list()

for contact in response:
    print(contact.email)
    # Automatically fetches next page when needed
```

**Manual pagination:**

```python
has_more = True
starting_after = None

while has_more:
    response = client.contacts.list(
        per_page=50,
        starting_after=starting_after
    )

    for contact in response.data:
        print(contact.email)

    has_more = response.pages and response.pages.next
    starting_after = response.pages.next.starting_after if has_more else None
```

**Get all pages at once:**

```python
all_contacts = []
response = client.contacts.list()

for contact in response:
    all_contacts.append(contact)

print(f'Total contacts: {len(all_contacts)}')
```

## Async Client

**Basic async usage:**

```python
import asyncio
from intercom import AsyncIntercom

async def main():
    client = AsyncIntercom(
        token=os.getenv('INTERCOM_ACCESS_TOKEN')
    )

    contact = await client.contacts.find(id='65f9a5e4f5e5b40001234567')
    print(contact.email)

    await client.close()

asyncio.run(main())
```

**Async context manager:**

```python
async def main():
    async with AsyncIntercom(token=os.getenv('INTERCOM_ACCESS_TOKEN')) as client:
        contact = await client.contacts.create(
            email='[email protected]',
            name='Async User'
        )
        print(contact.id)

asyncio.run(main())
```

**Concurrent operations:**

```python
async def create_multiple_contacts():
    async with AsyncIntercom(token=os.getenv('INTERCOM_ACCESS_TOKEN')) as client:
        tasks = [
            client.contacts.create(email=f'user{i}@example.com', name=f'User {i}')
            for i in range(10)
        ]

        contacts = await asyncio.gather(*tasks)

        for contact in contacts:
            print(contact.email)

asyncio.run(create_multiple_contacts())
```

**Async pagination:**

```python
async def list_all_contacts():
    async with AsyncIntercom(token=os.getenv('INTERCOM_ACCESS_TOKEN')) as client:
        response = await client.contacts.list()

        async for contact in response:
            print(contact.email)

asyncio.run(list_all_contacts())
```

## Advanced Features

**Custom headers:**

```python
contact = client.contacts.create(
    email='[email protected]',
    name='Test User',
    headers={
        'X-Custom-Header': 'custom-value',
    }
)
```

**Request timeout:**

```python
contact = client.contacts.find(
    id='65f9a5e4f5e5b40001234567',
    timeout=30
)
```

**Disable automatic retries:**

```python
client = Intercom(
    token=os.getenv('INTERCOM_ACCESS_TOKEN'),
    max_retries=0  # Disable retries
)
```

**Custom retry configuration:**

```python
client = Intercom(
    token=os.getenv('INTERCOM_ACCESS_TOKEN'),
    max_retries=5,
    timeout=90
)
```

## Webhooks

**Verify webhook signature:**

```python
import hmac
import hashlib

def verify_webhook(body, signature, secret):
    computed_hash = hmac.new(
        secret.encode('utf-8'),
        body.encode('utf-8'),
        hashlib.sha256
    ).hexdigest()

    return signature == computed_hash

# Flask example
from flask import Flask, request, jsonify

app = Flask(__name__)

@app.route('/webhooks/intercom', methods=['POST'])
def webhook():
    signature = request.headers.get('X-Hub-Signature')
    body = request.get_data(as_text=True)

    is_valid = verify_webhook(
        body,
        signature,
        os.getenv('INTERCOM_WEBHOOK_SECRET')
    )

    if not is_valid:
        return jsonify({'error': 'Invalid signature'}), 401

    event = request.json
    print(f'Webhook event: {event["topic"]}')

    return jsonify({'status': 'ok'})
```

**Handle webhook events:**

```python
@app.route('/webhooks/intercom', methods=['POST'])
def webhook():
    event = request.json
    topic = event.get('topic')
    data = event.get('data')

    if topic == 'contact.created':
        print(f'New contact: {data["item"]["email"]}')

    elif topic == 'conversation.user.created':
        print(f'New conversation: {data["item"]["id"]}')

    elif topic == 'conversation.admin.replied':
        print(f'Admin replied to: {data["item"]["id"]}')

    elif topic == 'user.tag.created':
        print(f'User tagged: {data["item"]["user"]["email"]}')

    else:
        print(f'Unknown event: {topic}')

    return jsonify({'status': 'ok'})
```

**Django webhook handler:**

```python
from django.http import JsonResponse
from django.views.decorators.csrf import csrf_exempt
import json

@csrf_exempt
def intercom_webhook(request):
    if request.method == 'POST':
        signature = request.headers.get('X-Hub-Signature')
        body = request.body.decode('utf-8')

        is_valid = verify_webhook(
            body,
            signature,
            os.getenv('INTERCOM_WEBHOOK_SECRET')
        )

        if not is_valid:
            return JsonResponse({'error': 'Invalid signature'}, status=401)

        event = json.loads(body)
        topic = event.get('topic')

        # Handle event
        print(f'Event: {topic}')

        return JsonResponse({'status': 'ok'})

    return JsonResponse({'error': 'Method not allowed'}, status=405)
```

## Rate Limiting

Intercom enforces rate limits on API requests. The SDK handles retries automatically.

**Default retry behavior:**

```python
# SDK automatically retries on 429 (rate limit) responses
# with exponential backoff (2 retries by default)

client = Intercom(
    token=os.getenv('INTERCOM_ACCESS_TOKEN'),
    max_retries=2  # Default
)
```

**Handle rate limits manually:**

```python
import time
from intercom import ApiError

def make_request_with_backoff(request_fn, max_retries=5):
    for i in range(max_retries):
        try:
            return request_fn()
        except ApiError as e:
            if e.status_code == 429:
                if i < max_retries - 1:
                    wait_time = 2 ** i
                    print(f'Rate limited. Waiting {wait_time}s...')
                    time.sleep(wait_time)
                    continue
            raise

# Usage
contact = make_request_with_backoff(
    lambda: client.contacts.create(email='[email protected]')
)
```

## Bulk Operations

**Bulk create contacts:**

```python
contacts_data = [
    {'email': '[email protected]', 'name': 'User 1'},
    {'email': '[email protected]', 'name': 'User 2'},
    {'email': '[email protected]', 'name': 'User 3'},
]

results = []
for data in contacts_data:
    try:
        contact = client.contacts.create(**data)
        results.append({'success': True, 'data': contact})
    except ApiError as e:
        results.append({'success': False, 'error': str(e)})

succeeded = sum(1 for r in results if r['success'])
print(f'Created {succeeded} contacts')
```

**Bulk update with concurrency (async):**

```python
import asyncio
from intercom import AsyncIntercom, ApiError

async def bulk_update_contacts(updates):
    async with AsyncIntercom(token=os.getenv('INTERCOM_ACCESS_TOKEN')) as client:
        tasks = [
            client.contacts.update(**data)
            for data in updates
        ]

        results = await asyncio.gather(*tasks, return_exceptions=True)

        succeeded = sum(1 for r in results if not isinstance(r, Exception))
        failed = len(results) - succeeded

        print(f'Success: {succeeded}, Failed: {failed}')
        return results

# Usage
updates = [
    {'id': '1', 'name': 'Updated 1'},
    {'id': '2', 'name': 'Updated 2'},
    {'id': '3', 'name': 'Updated 3'},
]

asyncio.run(bulk_update_contacts(updates))
```

**Rate-limited bulk operations:**

```python
import time

def bulk_operation_with_rate_limit(items, operation, requests_per_second=5):
    results = []
    delay = 1.0 / requests_per_second

    for item in items:
        try:
            result = operation(item)
            results.append({'success': True, 'data': result})
        except ApiError as e:
            results.append({'success': False, 'error': str(e)})

        time.sleep(delay)

    return results

# Usage
contact_data = [
    {'email': '[email protected]', 'name': 'User 1'},
    {'email': '[email protected]', 'name': 'User 2'},
]

results = bulk_operation_with_rate_limit(
    contact_data,
    lambda data: client.contacts.create(**data),
    requests_per_second=5
)
```

## Environment-Specific Configuration

**Development:**

```python
client = Intercom(
    token=os.getenv('INTERCOM_ACCESS_TOKEN'),
    timeout=30,
    max_retries=1
)
```

**Production:**

```python
client = Intercom(
    token=os.getenv('INTERCOM_ACCESS_TOKEN'),
    timeout=120,
    max_retries=3
)
```

**Testing with mock:**

```python
from unittest.mock import Mock

# Mock client for testing
mock_client = Mock(spec=Intercom)
mock_client.contacts.create.return_value = Mock(
    id='test_id',
    email='[email protected]',
    name='Test User'
)
```

## Complete Examples

**User onboarding workflow:**

```python
import time
from intercom import ApiError

def onboard_user(email, name, plan):
    try:
        # Create contact
        contact = client.contacts.create(
            email=email,
            name=name,
            role='user',
            custom_attributes={
                'plan': plan,
                'signup_date': time.strftime('%Y-%m-%d'),
            }
        )

        # Track signup event
        client.events.create(
            event_name='signed_up',
            created_at=int(time.time()),
            user_id=contact.id,
            metadata={
                'plan': plan,
                'source': 'web',
            }
        )

        # Tag as new user
        new_user_tag = client.tags.create(name='New User')
        client.contacts.tag(
            contact_id=contact.id,
            id=new_user_tag.id
        )

        # Send welcome message
        client.messages.create(
            message_type='email',
            from_={
                'type': 'admin',
                'id': os.getenv('ADMIN_ID'),
            },
            to={
                'type': 'user',
                'id': contact.id,
            },
            subject='Welcome to our platform!',
            body=f'Hi {name}, thanks for signing up!'
        )

        print(f'User onboarded successfully: {contact.id}')
        return contact

    except ApiError as e:
        print(f'Onboarding failed: {e}')
        raise
```

**Customer support workflow:**

```python
def handle_support_request(user_id, subject, message):
    try:
        # Create conversation
        conversation = client.conversations.create(
            from_={
                'type': 'user',
                'id': user_id,
            },
            body=message
        )

        # Assign to team
        client.conversations.assign(
            id=conversation.id,
            type='admin',
            admin_id=os.getenv('ADMIN_ID'),
            assignee_id=os.getenv('SUPPORT_TEAM_ID'),
            assignment_type='team'
        )

        # Add priority tag if urgent
        if 'urgent' in subject.lower():
            urgent_tag = client.tags.create(name='Urgent')
            client.conversations.attach_tag(
                conversation_id=conversation.id,
                id=urgent_tag.id,
                admin_id=os.getenv('ADMIN_ID')
            )

        # Track support event
        client.events.create(
            event_name='support_request_created',
            created_at=int(time.time()),
            user_id=user_id,
            metadata={
                'conversation_id': conversation.id,
                'subject': subject,
                'urgent': 'urgent' in subject.lower(),
            }
        )

        return conversation

    except ApiError as e:
        print(f'Support request failed: {e}')
        raise
```

**Company and contact management:**

```python
def add_contact_to_company(contact_email, company_id):
    try:
        # Find or create contact
        try:
            search_results = client.contacts.search(
                query={
                    'field': 'email',
                    'operator': '=',
                    'value': contact_email,
                }
            )
            contact = search_results.data[0]
        except (ApiError, IndexError):
            contact = client.contacts.create(
                email=contact_email,
                role='user'
            )

        # Attach to company
        client.contacts.update(
            id=contact.id,
            companies=[
                {
                    'company_id': company_id,
                }
            ]
        )

        # Get company details
        company = client.companies.find(company_id=company_id)

        print(f'Added {contact_email} to {company.name}')

        return {'contact': contact, 'company': company}

    except ApiError as e:
        print(f'Failed to add contact to company: {e}')
        raise
```

**Analytics and reporting:**

```python
def generate_user_report(user_id):
    try:
        # Get user details
        contact = client.contacts.find(id=user_id)

        # Get user events
        events = client.events.list(
            type='user',
            user_id=user_id
        )

        # Get conversations
        conversations = client.conversations.search(
            query={
                'field': 'contact_ids',
                'operator': '=',
                'value': user_id,
            }
        )

        # Get notes
        notes = client.contacts.list_notes(id=user_id)

        report = {
            'contact': {
                'email': contact.email,
                'name': contact.name,
                'created_at': contact.created_at,
                'custom_attributes': contact.custom_attributes,
            },
            'events': [
                {
                    'name': e.event_name,
                    'created_at': e.created_at,
                    'metadata': e.metadata,
                }
                for e in events.events
            ],
            'conversations': {
                'total': conversations.total_count,
                'open': sum(1 for c in conversations.conversations if c.state == 'open'),
                'closed': sum(1 for c in conversations.conversations if c.state == 'closed'),
            },
            'notes': [
                {
                    'body': n.body,
                    'created_at': n.created_at,
                }
                for n in notes.data
            ],
        }

        return report

    except ApiError as e:
        print(f'Report generation failed: {e}')
        raise
```

**Async batch processing:**

```python
import asyncio
from intercom import AsyncIntercom

async def process_contacts_batch(contact_ids):
    async with AsyncIntercom(token=os.getenv('INTERCOM_ACCESS_TOKEN')) as client:
        # Fetch all contacts concurrently
        tasks = [client.contacts.find(id=contact_id) for contact_id in contact_ids]
        contacts = await asyncio.gather(*tasks, return_exceptions=True)

        # Process successful results
        valid_contacts = [c for c in contacts if not isinstance(c, Exception)]

        # Update contacts concurrently
        update_tasks = [
            client.contacts.update(
                id=contact.id,
                custom_attributes={'last_processed': int(time.time())}
            )
            for contact in valid_contacts
        ]

        results = await asyncio.gather(*update_tasks, return_exceptions=True)

        succeeded = sum(1 for r in results if not isinstance(r, Exception))
        print(f'Processed {succeeded}/{len(contact_ids)} contacts')

        return results

# Usage
contact_ids = ['id1', 'id2', 'id3', 'id4', 'id5']
asyncio.run(process_contacts_batch(contact_ids))
```
