---
name: messaging
description: "Intercom JavaScript SDK for customer messaging and chat support"
metadata:
  languages: "javascript"
  versions: "6.4.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "intercom,messaging,customer,chat,support"
---

# Intercom JavaScript SDK (v6.4.0)

## Golden Rule

**ALWAYS use `intercom-client` version 6.4.0 or later.**

```bash
npm install intercom-client
```

**DO NOT use these deprecated or unofficial packages:**
- `intercom.io` (deprecated)
- `@ludovicbret/intercom-client` (community fork)
- Any package not named exactly `intercom-client`

The official Intercom TypeScript/JavaScript SDK is `intercom-client`, maintained by Intercom at https://github.com/intercom/intercom-node

## Installation

```bash
npm install intercom-client
```

For TypeScript projects, types are included in the package.

**Environment Setup:**

```bash
# .env file
INTERCOM_ACCESS_TOKEN=your_access_token_here
```

**Loading environment variables:**

```javascript
// Using dotenv
import 'dotenv/config';
// or
require('dotenv').config();
```

## Initialization

**Basic Client Setup:**

```javascript
import { IntercomClient } from 'intercom-client';

const client = new IntercomClient({
  token: process.env.INTERCOM_ACCESS_TOKEN
});
```

**TypeScript with explicit typing:**

```typescript
import { IntercomClient, Intercom } from 'intercom-client';

const client: IntercomClient = new IntercomClient({
  token: process.env.INTERCOM_ACCESS_TOKEN
});
```

**With custom configuration:**

```javascript
const client = new IntercomClient({
  token: process.env.INTERCOM_ACCESS_TOKEN,
  timeoutInSeconds: 120, // Default is 60
  maxRetries: 3, // Default is 2
});
```

**API Version Selection:**

```javascript
// Use specific API version
const client = new IntercomClient({
  token: process.env.INTERCOM_ACCESS_TOKEN,
  apiVersion: '2.11', // Default is latest stable (2.11)
});
```

## Core API Surfaces

### Contacts

Contacts represent users, leads, and visitors in Intercom. They are the primary entities for customer communication.

**Create a contact:**

```javascript
const contact = await client.contacts.create({
  email: '[email protected]',
  name: 'John Doe',
  phone: '+1234567890',
  role: 'user', // 'user' or 'lead'
});

console.log(contact.id);
```

**Create with custom attributes:**

```javascript
const contact = await client.contacts.create({
  email: '[email protected]',
  name: 'Jane Smith',
  role: 'user',
  custom_attributes: {
    plan: 'premium',
    signup_date: '2025-01-15',
    total_spend: 599.99,
    active: true,
  },
});
```

**Retrieve a contact:**

```javascript
const contact = await client.contacts.find({
  id: '65f9a5e4f5e5b40001234567',
});

console.log(contact.email, contact.name);
```

**Update a contact:**

```javascript
const updated = await client.contacts.update({
  id: '65f9a5e4f5e5b40001234567',
  name: 'John Updated',
  custom_attributes: {
    plan: 'enterprise',
  },
});
```

**Search contacts:**

```javascript
const results = await client.contacts.search({
  query: {
    field: 'email',
    operator: '=',
    value: '[email protected]',
  },
});

for (const contact of results.data) {
  console.log(contact.name, contact.email);
}
```

**Search with filters:**

```javascript
const results = await client.contacts.search({
  query: {
    operator: 'AND',
    value: [
      {
        field: 'role',
        operator: '=',
        value: 'user',
      },
      {
        field: 'created_at',
        operator: '>',
        value: 1704067200, // Unix timestamp
      },
    ],
  },
  pagination: {
    per_page: 50,
  },
});
```

**List all contacts:**

```javascript
const response = await client.contacts.list({
  per_page: 150,
});

// Iterate through paginated results
for await (const contact of response) {
  console.log(contact.email);
}
```

**Delete a contact:**

```javascript
await client.contacts.delete({
  id: '65f9a5e4f5e5b40001234567',
});
```

**Archive a contact:**

```javascript
const archived = await client.contacts.archive({
  id: '65f9a5e4f5e5b40001234567',
});
```

**Unarchive a contact:**

```javascript
const unarchived = await client.contacts.unarchive({
  id: '65f9a5e4f5e5b40001234567',
});
```

**Merge contacts:**

```javascript
const merged = await client.contacts.merge({
  from: '65f9a5e4f5e5b40001234567',
  into: '65f9a5e4f5e5b40009876543',
});
```

### Companies

Companies group contacts together by organization. They're useful for B2B use cases.

**Create a company:**

```javascript
const company = await client.companies.create({
  company_id: 'company_123',
  name: 'Acme Corporation',
  website: 'https://acme.com',
  industry: 'Technology',
  size: 500,
  custom_attributes: {
    plan_level: 'enterprise',
    mrr: 50000,
  },
});
```

**Retrieve a company:**

```javascript
const company = await client.companies.find({
  company_id: 'company_123',
});
```

**Update a company:**

```javascript
const updated = await client.companies.update({
  company_id: 'company_123',
  name: 'Acme Corp',
  size: 600,
  custom_attributes: {
    mrr: 60000,
  },
});
```

**List companies:**

```javascript
const response = await client.companies.list({
  per_page: 50,
});

for await (const company of response) {
  console.log(company.name, company.company_id);
}
```

**Attach contact to company:**

```javascript
await client.contacts.update({
  id: '65f9a5e4f5e5b40001234567',
  companies: [
    {
      company_id: 'company_123',
    },
  ],
});
```

**List company contacts:**

```javascript
const contacts = await client.companies.listContacts({
  company_id: 'company_123',
});

for await (const contact of contacts) {
  console.log(contact.email);
}
```

**Delete a company:**

```javascript
await client.companies.delete({
  company_id: 'company_123',
});
```

### Conversations

Conversations are threads of communication between contacts and your team.

**Create a conversation:**

```javascript
const conversation = await client.conversations.create({
  from: {
    type: 'user',
    id: '65f9a5e4f5e5b40001234567',
  },
  body: 'Hello, I need help with my account.',
});

console.log(conversation.id);
```

**Create conversation with user:**

```javascript
const conversation = await client.conversations.createConversation({
  userId: '65f9a5e4f5e5b40001234567',
  body: 'I have a question about billing.',
});
```

**Retrieve a conversation:**

```javascript
const conversation = await client.conversations.find({
  id: '123456',
});

console.log(conversation.state); // 'open', 'closed', 'snoozed'
```

**Reply to conversation as admin:**

```javascript
const reply = await client.conversations.reply({
  id: '123456',
  message_type: 'comment',
  type: 'admin',
  admin_id: '987654',
  body: 'Thanks for reaching out! How can I help?',
});
```

**Reply with attachment:**

```javascript
const reply = await client.conversations.reply({
  id: '123456',
  message_type: 'comment',
  type: 'admin',
  admin_id: '987654',
  body: 'Here is the document you requested.',
  attachment_urls: [
    'https://example.com/document.pdf',
  ],
});
```

**Reply as user:**

```javascript
const reply = await client.conversations.reply({
  id: '123456',
  message_type: 'comment',
  type: 'user',
  user_id: '65f9a5e4f5e5b40001234567',
  body: 'Thank you for your help!',
});
```

**Search conversations:**

```javascript
const results = await client.conversations.search({
  query: {
    field: 'state',
    operator: '=',
    value: 'open',
  },
  pagination: {
    per_page: 50,
  },
});

for (const conv of results.conversations) {
  console.log(conv.id, conv.created_at);
}
```

**Search with multiple filters:**

```javascript
const results = await client.conversations.search({
  query: {
    operator: 'AND',
    value: [
      {
        field: 'state',
        operator: '=',
        value: 'open',
      },
      {
        field: 'updated_at',
        operator: '>',
        value: 1704067200,
      },
    ],
  },
  sort: {
    field: 'updated_at',
    order: 'descending',
  },
});
```

**List conversations:**

```javascript
const response = await client.conversations.list();

for await (const conversation of response) {
  console.log(conversation.id, conversation.state);
}
```

**Close a conversation:**

```javascript
const closed = await client.conversations.close({
  id: '123456',
  type: 'admin',
  admin_id: '987654',
});
```

**Open a conversation:**

```javascript
const opened = await client.conversations.open({
  id: '123456',
  type: 'admin',
  admin_id: '987654',
});
```

**Snooze a conversation:**

```javascript
const snoozed = await client.conversations.snooze({
  id: '123456',
  type: 'admin',
  admin_id: '987654',
  snoozed_until: 1704153600, // Unix timestamp
});
```

**Assign conversation to admin:**

```javascript
const assigned = await client.conversations.assign({
  id: '123456',
  type: 'admin',
  admin_id: '987654',
  assignee_id: '111222',
});
```

**Assign to team:**

```javascript
const assigned = await client.conversations.assign({
  id: '123456',
  type: 'admin',
  admin_id: '987654',
  assignee_id: '333444',
  assignment_type: 'team',
});
```

**Convert conversation to ticket:**

```javascript
const ticket = await client.conversations.convertToTicket({
  id: '123456',
  ticket_type_id: '100',
});
```

**Add tag to conversation:**

```javascript
await client.conversations.attachTag({
  conversationId: '123456',
  id: 'tag_789',
  adminId: '987654',
});
```

**Remove tag from conversation:**

```javascript
await client.conversations.detachTag({
  conversationId: '123456',
  id: 'tag_789',
  adminId: '987654',
});
```

### Messages

Send messages to contacts via Intercom.

**Send a message to contact:**

```javascript
const message = await client.messages.create({
  message_type: 'email',
  from: {
    type: 'admin',
    id: '987654',
  },
  to: {
    type: 'user',
    id: '65f9a5e4f5e5b40001234567',
  },
  subject: 'Welcome to our platform',
  body: 'Thanks for signing up! Here are some tips to get started.',
});
```

**Send message by email:**

```javascript
const message = await client.messages.create({
  message_type: 'email',
  from: {
    type: 'admin',
    id: '987654',
  },
  to: {
    type: 'user',
    email: '[email protected]',
  },
  subject: 'Your monthly report',
  body: 'Here is your activity summary for January.',
});
```

**Send in-app message:**

```javascript
const message = await client.messages.create({
  message_type: 'inapp',
  from: {
    type: 'admin',
    id: '987654',
  },
  to: {
    type: 'user',
    id: '65f9a5e4f5e5b40001234567',
  },
  body: 'Check out our new features!',
});
```

**Send message with custom data:**

```javascript
const message = await client.messages.create({
  message_type: 'email',
  from: {
    type: 'admin',
    id: '987654',
  },
  to: {
    type: 'user',
    id: '65f9a5e4f5e5b40001234567',
  },
  subject: 'Order confirmation',
  body: 'Your order has been confirmed.',
  template: 'plain',
  create_conversation_without_contact_reply: false,
});
```

### Data Events

Track user behavior and custom events.

**Submit a data event:**

```javascript
await client.events.create({
  event_name: 'purchased_item',
  created_at: Math.floor(Date.now() / 1000),
  user_id: '65f9a5e4f5e5b40001234567',
  metadata: {
    item_name: 'Premium Plan',
    item_price: 99.99,
    currency: 'USD',
    quantity: 1,
  },
});
```

**Event with email identifier:**

```javascript
await client.events.create({
  event_name: 'signed_up',
  created_at: Math.floor(Date.now() / 1000),
  email: '[email protected]',
  metadata: {
    source: 'landing_page',
    campaign: 'winter_2025',
  },
});
```

**Complex event metadata:**

```javascript
await client.events.create({
  event_name: 'completed_onboarding',
  created_at: Math.floor(Date.now() / 1000),
  user_id: '65f9a5e4f5e5b40001234567',
  metadata: {
    steps_completed: 5,
    time_taken_seconds: 320,
    skipped_steps: ['profile_picture'],
    completion_rate: 0.95,
    device: 'mobile',
  },
});
```

**List events for user:**

```javascript
const events = await client.events.list({
  type: 'user',
  user_id: '65f9a5e4f5e5b40001234567',
});

for (const event of events.events) {
  console.log(event.event_name, event.created_at);
}
```

**Event summaries:**

```javascript
const summary = await client.events.summaries({
  user_id: '65f9a5e4f5e5b40001234567',
  event_name: 'purchased_item',
});

console.log(summary.count, summary.first, summary.last);
```

### Tags

Organize and categorize contacts, companies, and conversations.

**Create a tag:**

```javascript
const tag = await client.tags.create({
  name: 'VIP Customer',
});

console.log(tag.id);
```

**Retrieve a tag:**

```javascript
const tag = await client.tags.find({
  id: 'tag_789',
});
```

**List all tags:**

```javascript
const tags = await client.tags.list();

for (const tag of tags.data) {
  console.log(tag.name, tag.id);
}
```

**Tag a contact:**

```javascript
await client.contacts.tag({
  contactId: '65f9a5e4f5e5b40001234567',
  id: 'tag_789',
});
```

**Tag a company:**

```javascript
await client.companies.tag({
  companyId: 'company_123',
  id: 'tag_789',
});
```

**Untag a contact:**

```javascript
await client.contacts.untag({
  contactId: '65f9a5e4f5e5b40001234567',
  id: 'tag_789',
});
```

**Delete a tag:**

```javascript
await client.tags.delete({
  id: 'tag_789',
});
```

### Data Attributes

Define custom attributes for contacts and companies.

**Create a contact attribute:**

```javascript
const attribute = await client.dataAttributes.create({
  name: 'subscription_tier',
  model: 'contact',
  data_type: 'string',
  options: ['free', 'pro', 'enterprise'],
  description: 'Customer subscription level',
});
```

**Create a company attribute:**

```javascript
const attribute = await client.dataAttributes.create({
  name: 'annual_revenue',
  model: 'company',
  data_type: 'float',
  description: 'Company annual revenue in USD',
});
```

**Create boolean attribute:**

```javascript
const attribute = await client.dataAttributes.create({
  name: 'is_beta_tester',
  model: 'contact',
  data_type: 'boolean',
  description: 'Whether user is enrolled in beta program',
});
```

**Create date attribute:**

```javascript
const attribute = await client.dataAttributes.create({
  name: 'trial_end_date',
  model: 'contact',
  data_type: 'date',
  description: 'Date when trial period ends',
});
```

**List data attributes:**

```javascript
const attributes = await client.dataAttributes.list({
  model: 'contact',
});

for (const attr of attributes.data) {
  console.log(attr.name, attr.data_type);
}
```

**Update an attribute:**

```javascript
const updated = await client.dataAttributes.update({
  id: 'attr_123',
  description: 'Updated description',
  options: ['free', 'pro', 'enterprise', 'custom'],
});
```

### Notes

Add notes to contacts and companies for internal reference.

**Create a note for contact:**

```javascript
const note = await client.notes.create({
  contact_id: '65f9a5e4f5e5b40001234567',
  admin_id: '987654',
  body: 'Customer requested custom integration. Follow up next week.',
});
```

**Create a note for company:**

```javascript
const note = await client.notes.create({
  company_id: 'company_123',
  admin_id: '987654',
  body: 'Contract renewal coming up in Q2 2025.',
});
```

**Retrieve a note:**

```javascript
const note = await client.notes.find({
  id: 'note_456',
});
```

**List notes for contact:**

```javascript
const notes = await client.contacts.listNotes({
  id: '65f9a5e4f5e5b40001234567',
});

for (const note of notes.data) {
  console.log(note.body, note.created_at);
}
```

### Segments

Query and retrieve segments (groups of contacts).

**List all segments:**

```javascript
const segments = await client.segments.list();

for (const segment of segments.segments) {
  console.log(segment.name, segment.count);
}
```

**Retrieve a segment:**

```javascript
const segment = await client.segments.find({
  id: 'segment_999',
});

console.log(segment.name, segment.person_type);
```

**List contacts in segment:**

```javascript
const contacts = await client.segments.listContacts({
  id: 'segment_999',
});

for await (const contact of contacts) {
  console.log(contact.email);
}
```

### Tickets

Manage customer support tickets.

**Create a ticket:**

```javascript
const ticket = await client.tickets.create({
  contact_id: '65f9a5e4f5e5b40001234567',
  ticket_type_id: '100',
  contacts: [
    {
      id: '65f9a5e4f5e5b40001234567',
    },
  ],
  ticket_attributes: {
    _default_title_: 'Payment issue',
    _default_description_: 'Customer unable to process payment',
  },
});
```

**Retrieve a ticket:**

```javascript
const ticket = await client.tickets.find({
  id: 'ticket_888',
});

console.log(ticket.ticket_state);
```

**Update a ticket:**

```javascript
const updated = await client.tickets.update({
  id: 'ticket_888',
  ticket_attributes: {
    _default_title_: 'Payment issue - RESOLVED',
  },
  ticket_state: 'submitted',
});
```

**Reply to ticket:**

```javascript
const reply = await client.tickets.reply({
  id: 'ticket_888',
  admin_id: '987654',
  message_type: 'comment',
  body: 'Issue has been resolved. Payment processed successfully.',
});
```

**Search tickets:**

```javascript
const results = await client.tickets.search({
  query: {
    field: 'ticket_state',
    operator: '=',
    value: 'submitted',
  },
});

for (const ticket of results.tickets) {
  console.log(ticket.id, ticket.ticket_attributes);
}
```

### Admins

Manage admin users and teams.

**List all admins:**

```javascript
const admins = await client.admins.list();

for (const admin of admins.admins) {
  console.log(admin.name, admin.email);
}
```

**Retrieve an admin:**

```javascript
const admin = await client.admins.find({
  id: '987654',
});

console.log(admin.name, admin.away_mode_enabled);
```

**Retrieve current admin:**

```javascript
const me = await client.admins.me();

console.log(me.name, me.email);
```

**Set away mode:**

```javascript
const updated = await client.admins.setAwayMode({
  admin_id: '987654',
  away_mode_enabled: true,
  away_mode_reassign: true,
});
```

**List teams:**

```javascript
const teams = await client.teams.list();

for (const team of teams.teams) {
  console.log(team.name, team.id);
}
```

**Retrieve a team:**

```javascript
const team = await client.teams.find({
  id: '333444',
});
```

### Articles

Manage help center articles.

**Create an article:**

```javascript
const article = await client.articles.create({
  title: 'Getting Started Guide',
  description: 'Learn how to use our platform',
  body: '<h1>Welcome</h1><p>This guide will help you get started...</p>',
  author_id: 987654,
  state: 'published',
});
```

**Retrieve an article:**

```javascript
const article = await client.articles.find({
  id: 'article_555',
});
```

**Update an article:**

```javascript
const updated = await client.articles.update({
  id: 'article_555',
  title: 'Getting Started Guide (Updated)',
  body: '<h1>Welcome</h1><p>This updated guide...</p>',
});
```

**List articles:**

```javascript
const response = await client.articles.list({
  per_page: 50,
});

for await (const article of response) {
  console.log(article.title, article.state);
}
```

**Delete an article:**

```javascript
await client.articles.delete({
  id: 'article_555',
});
```

### Subscription Types

Manage subscription preferences for contacts.

**List subscription types:**

```javascript
const types = await client.subscriptionTypes.list();

for (const type of types.data) {
  console.log(type.id, type.content_type);
}
```

**Subscribe contact:**

```javascript
await client.contacts.subscribe({
  id: '65f9a5e4f5e5b40001234567',
  subscription_type_id: 'sub_123',
});
```

**Unsubscribe contact:**

```javascript
await client.contacts.unsubscribe({
  id: '65f9a5e4f5e5b40001234567',
  subscription_type_id: 'sub_123',
});
```

## Error Handling

**Basic error handling:**

```javascript
import { IntercomError } from 'intercom-client';

try {
  const contact = await client.contacts.find({ id: 'invalid_id' });
} catch (err) {
  if (err instanceof IntercomError) {
    console.error('Status:', err.statusCode);
    console.error('Message:', err.message);
    console.error('Body:', err.body);
  }
}
```

**Handle specific error codes:**

```javascript
try {
  await client.contacts.create({ email: '[email protected]' });
} catch (err) {
  if (err instanceof IntercomError) {
    switch (err.statusCode) {
      case 400:
        console.error('Bad request:', err.message);
        break;
      case 401:
        console.error('Unauthorized - check your token');
        break;
      case 404:
        console.error('Resource not found');
        break;
      case 429:
        console.error('Rate limit exceeded');
        break;
      default:
        console.error('Error:', err.message);
    }
  }
}
```

**Retry on rate limit:**

```javascript
async function createContactWithRetry(data, maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await client.contacts.create(data);
    } catch (err) {
      if (err instanceof IntercomError && err.statusCode === 429) {
        if (attempt < maxRetries) {
          const delay = Math.pow(2, attempt) * 1000;
          console.log(`Rate limited, retrying in ${delay}ms...`);
          await new Promise(resolve => setTimeout(resolve, delay));
          continue;
        }
      }
      throw err;
    }
  }
}
```

## Pagination

**Auto-pagination with for-await:**

```javascript
const response = await client.contacts.list();

for await (const contact of response) {
  console.log(contact.email);
  // Automatically fetches next page when needed
}
```

**Manual pagination:**

```javascript
let hasMore = true;
let startingAfter = null;

while (hasMore) {
  const response = await client.contacts.list({
    per_page: 50,
    starting_after: startingAfter,
  });

  for (const contact of response.data) {
    console.log(contact.email);
  }

  hasMore = response.pages && response.pages.next;
  startingAfter = response.pages?.next?.starting_after;
}
```

**Get all pages at once:**

```javascript
const allContacts = [];
const response = await client.contacts.list();

for await (const contact of response) {
  allContacts.push(contact);
}

console.log(`Total contacts: ${allContacts.length}`);
```

## Advanced Features

**Custom headers:**

```javascript
const contact = await client.contacts.create(
  {
    email: '[email protected]',
    name: 'Test User',
  },
  {
    headers: {
      'X-Custom-Header': 'custom-value',
    },
  }
);
```

**Request timeout:**

```javascript
const contact = await client.contacts.find(
  { id: '65f9a5e4f5e5b40001234567' },
  {
    timeoutInSeconds: 30,
  }
);
```

**Abort requests:**

```javascript
const controller = new AbortController();

// Cancel after 5 seconds
setTimeout(() => controller.abort(), 5000);

try {
  const contact = await client.contacts.create(
    {
      email: '[email protected]',
    },
    {
      abortSignal: controller.signal,
    }
  );
} catch (err) {
  if (err.name === 'AbortError') {
    console.log('Request was aborted');
  }
}
```

**Access raw response:**

```javascript
const response = await client.contacts
  .withRawResponse()
  .find({ id: '65f9a5e4f5e5b40001234567' });

console.log('Status:', response.statusCode);
console.log('Headers:', response.headers);
console.log('Body:', response.body);
```

**Disable automatic retries:**

```javascript
const client = new IntercomClient({
  token: process.env.INTERCOM_ACCESS_TOKEN,
  maxRetries: 0, // Disable retries
});
```

**Custom retry configuration:**

```javascript
const client = new IntercomClient({
  token: process.env.INTERCOM_ACCESS_TOKEN,
  maxRetries: 5,
  timeoutInSeconds: 90,
});
```

## Type Definitions

**Using request types:**

```typescript
import { Intercom } from 'intercom-client';

const createRequest: Intercom.ContactCreateRequest = {
  email: '[email protected]',
  name: 'Jane Doe',
  role: 'user',
  custom_attributes: {
    plan: 'premium',
  },
};

const contact = await client.contacts.create(createRequest);
```

**Using response types:**

```typescript
import { Intercom } from 'intercom-client';

const contact: Intercom.Contact = await client.contacts.find({
  id: '65f9a5e4f5e5b40001234567',
});

console.log(contact.email, contact.created_at);
```

**Conversation types:**

```typescript
import { Intercom } from 'intercom-client';

const conversation: Intercom.Conversation = await client.conversations.find({
  id: '123456',
});

const state: Intercom.ConversationState = conversation.state;
// 'open' | 'closed' | 'snoozed'
```

**Event metadata typing:**

```typescript
interface PurchaseMetadata {
  item_name: string;
  item_price: number;
  currency: string;
  quantity: number;
}

await client.events.create({
  event_name: 'purchased_item',
  created_at: Math.floor(Date.now() / 1000),
  user_id: '65f9a5e4f5e5b40001234567',
  metadata: {
    item_name: 'Premium Plan',
    item_price: 99.99,
    currency: 'USD',
    quantity: 1,
  } as PurchaseMetadata,
});
```

## Webhooks

**Verify webhook signature:**

```javascript
import crypto from 'crypto';

function verifyWebhook(body, signature, secret) {
  const hash = crypto
    .createHmac('sha256', secret)
    .update(body)
    .digest('hex');

  return signature === hash;
}

// Express example
app.post('/webhooks/intercom', (req, res) => {
  const signature = req.headers['x-hub-signature'];
  const isValid = verifyWebhook(
    JSON.stringify(req.body),
    signature,
    process.env.INTERCOM_WEBHOOK_SECRET
  );

  if (!isValid) {
    return res.status(401).send('Invalid signature');
  }

  const event = req.body;
  console.log('Webhook event:', event.topic);

  res.sendStatus(200);
});
```

**Handle webhook events:**

```javascript
app.post('/webhooks/intercom', (req, res) => {
  const { topic, data } = req.body;

  switch (topic) {
    case 'contact.created':
      console.log('New contact:', data.item.email);
      break;

    case 'conversation.user.created':
      console.log('New conversation:', data.item.id);
      break;

    case 'conversation.admin.replied':
      console.log('Admin replied to:', data.item.id);
      break;

    case 'user.tag.created':
      console.log('User tagged:', data.item.user.email);
      break;

    default:
      console.log('Unknown event:', topic);
  }

  res.sendStatus(200);
});
```

## Rate Limiting

Intercom enforces rate limits on API requests. The SDK handles retries automatically.

**Default retry behavior:**

```javascript
// SDK automatically retries on 429 (rate limit) responses
// with exponential backoff (2 retries by default)

const client = new IntercomClient({
  token: process.env.INTERCOM_ACCESS_TOKEN,
  maxRetries: 2, // Default
});
```

**Rate limit headers:**

```javascript
const response = await client.contacts
  .withRawResponse()
  .list();

console.log('Rate limit:', response.headers['x-ratelimit-limit']);
console.log('Remaining:', response.headers['x-ratelimit-remaining']);
console.log('Reset:', response.headers['x-ratelimit-reset']);
```

**Handle rate limits manually:**

```javascript
async function makeRequestWithBackoff(requestFn, maxRetries = 5) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await requestFn();
    } catch (err) {
      if (err instanceof IntercomError && err.statusCode === 429) {
        const resetHeader = err.headers?.['x-ratelimit-reset'];
        const resetTime = resetHeader ? parseInt(resetHeader) * 1000 : null;

        if (resetTime && i < maxRetries - 1) {
          const waitTime = Math.max(resetTime - Date.now(), 0);
          console.log(`Rate limited. Waiting ${waitTime}ms...`);
          await new Promise(resolve => setTimeout(resolve, waitTime));
          continue;
        }
      }
      throw err;
    }
  }
}

// Usage
await makeRequestWithBackoff(() =>
  client.contacts.create({ email: '[email protected]' })
);
```

## Bulk Operations

**Bulk create contacts:**

```javascript
const contacts = [
  { email: '[email protected]', name: 'User 1' },
  { email: '[email protected]', name: 'User 2' },
  { email: '[email protected]', name: 'User 3' },
];

const results = await Promise.all(
  contacts.map(data => client.contacts.create(data))
);

console.log(`Created ${results.length} contacts`);
```

**Bulk update with error handling:**

```javascript
const updates = [
  { id: '1', name: 'Updated 1' },
  { id: '2', name: 'Updated 2' },
  { id: '3', name: 'Updated 3' },
];

const results = await Promise.allSettled(
  updates.map(data => client.contacts.update(data))
);

const succeeded = results.filter(r => r.status === 'fulfilled').length;
const failed = results.filter(r => r.status === 'rejected').length;

console.log(`Success: ${succeeded}, Failed: ${failed}`);
```

**Rate-limited bulk operations:**

```javascript
async function bulkOperationWithRateLimit(items, operation, requestsPerSecond = 5) {
  const results = [];
  const delay = 1000 / requestsPerSecond;

  for (const item of items) {
    try {
      const result = await operation(item);
      results.push({ success: true, data: result });
    } catch (err) {
      results.push({ success: false, error: err });
    }

    await new Promise(resolve => setTimeout(resolve, delay));
  }

  return results;
}

// Usage
const contactData = [
  { email: '[email protected]', name: 'User 1' },
  { email: '[email protected]', name: 'User 2' },
  // ... more contacts
];

const results = await bulkOperationWithRateLimit(
  contactData,
  data => client.contacts.create(data),
  5 // 5 requests per second
);
```

## Environment-Specific Configuration

**Development:**

```javascript
const client = new IntercomClient({
  token: process.env.INTERCOM_ACCESS_TOKEN,
  timeoutInSeconds: 30,
  maxRetries: 1,
});
```

**Production:**

```javascript
const client = new IntercomClient({
  token: process.env.INTERCOM_ACCESS_TOKEN,
  timeoutInSeconds: 120,
  maxRetries: 3,
});
```

**Testing with mock:**

```javascript
// For testing, you can inject a custom httpx client
import { IntercomClient } from 'intercom-client';

const mockClient = new IntercomClient({
  token: 'test_token',
  // Use mock HTTP client for tests
});
```

## Complete Examples

**User onboarding workflow:**

```javascript
async function onboardUser(email, name, plan) {
  try {
    // Create contact
    const contact = await client.contacts.create({
      email,
      name,
      role: 'user',
      custom_attributes: {
        plan,
        signup_date: new Date().toISOString(),
      },
    });

    // Track signup event
    await client.events.create({
      event_name: 'signed_up',
      created_at: Math.floor(Date.now() / 1000),
      user_id: contact.id,
      metadata: {
        plan,
        source: 'web',
      },
    });

    // Tag as new user
    const newUserTag = await client.tags.create({
      name: 'New User',
    });

    await client.contacts.tag({
      contactId: contact.id,
      id: newUserTag.id,
    });

    // Send welcome message
    await client.messages.create({
      message_type: 'email',
      from: {
        type: 'admin',
        id: process.env.ADMIN_ID,
      },
      to: {
        type: 'user',
        id: contact.id,
      },
      subject: 'Welcome to our platform!',
      body: `Hi ${name}, thanks for signing up!`,
    });

    console.log('User onboarded successfully:', contact.id);
    return contact;
  } catch (err) {
    console.error('Onboarding failed:', err);
    throw err;
  }
}
```

**Customer support workflow:**

```javascript
async function handleSupportRequest(userId, subject, message) {
  try {
    // Create conversation
    const conversation = await client.conversations.create({
      from: {
        type: 'user',
        id: userId,
      },
      body: message,
    });

    // Assign to team
    await client.conversations.assign({
      id: conversation.id,
      type: 'admin',
      admin_id: process.env.ADMIN_ID,
      assignee_id: process.env.SUPPORT_TEAM_ID,
      assignment_type: 'team',
    });

    // Add priority tag if urgent
    if (subject.toLowerCase().includes('urgent')) {
      const urgentTag = await client.tags.create({
        name: 'Urgent',
      });

      await client.conversations.attachTag({
        conversationId: conversation.id,
        id: urgentTag.id,
        adminId: process.env.ADMIN_ID,
      });
    }

    // Track support event
    await client.events.create({
      event_name: 'support_request_created',
      created_at: Math.floor(Date.now() / 1000),
      user_id: userId,
      metadata: {
        conversation_id: conversation.id,
        subject,
        urgent: subject.toLowerCase().includes('urgent'),
      },
    });

    return conversation;
  } catch (err) {
    console.error('Support request failed:', err);
    throw err;
  }
}
```

**Company and contact management:**

```javascript
async function addContactToCompany(contactEmail, companyId) {
  try {
    // Find or create contact
    let contact;
    try {
      const searchResults = await client.contacts.search({
        query: {
          field: 'email',
          operator: '=',
          value: contactEmail,
        },
      });
      contact = searchResults.data[0];
    } catch (err) {
      contact = await client.contacts.create({
        email: contactEmail,
        role: 'user',
      });
    }

    // Attach to company
    await client.contacts.update({
      id: contact.id,
      companies: [
        {
          company_id: companyId,
        },
      ],
    });

    // Get company details
    const company = await client.companies.find({
      company_id: companyId,
    });

    console.log(`Added ${contactEmail} to ${company.name}`);

    return { contact, company };
  } catch (err) {
    console.error('Failed to add contact to company:', err);
    throw err;
  }
}
```

**Analytics and reporting:**

```javascript
async function generateUserReport(userId) {
  try {
    // Get user details
    const contact = await client.contacts.find({ id: userId });

    // Get user events
    const events = await client.events.list({
      type: 'user',
      user_id: userId,
    });

    // Get conversations
    const conversations = await client.conversations.search({
      query: {
        field: 'contact_ids',
        operator: '=',
        value: userId,
      },
    });

    // Get notes
    const notes = await client.contacts.listNotes({ id: userId });

    const report = {
      contact: {
        email: contact.email,
        name: contact.name,
        created_at: contact.created_at,
        custom_attributes: contact.custom_attributes,
      },
      events: events.events.map(e => ({
        name: e.event_name,
        created_at: e.created_at,
        metadata: e.metadata,
      })),
      conversations: {
        total: conversations.total_count,
        open: conversations.conversations.filter(c => c.state === 'open').length,
        closed: conversations.conversations.filter(c => c.state === 'closed').length,
      },
      notes: notes.data.map(n => ({
        body: n.body,
        created_at: n.created_at,
      })),
    };

    return report;
  } catch (err) {
    console.error('Report generation failed:', err);
    throw err;
  }
}
```
