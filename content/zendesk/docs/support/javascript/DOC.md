---
name: support
description: "Zendesk API JavaScript/Node.js SDK (node-zendesk) for helpdesk, tickets, and customer service integration"
metadata:
  languages: "javascript"
  versions: "6.0.1"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "zendesk,support,helpdesk,tickets,customer-service"
---

# Zendesk API - JavaScript/Node.js SDK (node-zendesk)

## Golden Rule

**ALWAYS use the `node-zendesk` package (version 6.0.1 or later) for Zendesk API integration in JavaScript/Node.js projects.**

```bash
npm install node-zendesk
```

**DO NOT use:**
- `zendesk-node-api` (community alternative)
- `zendesk-node` (outdated)
- `@zendesk/client` (non-existent)
- Direct HTTP requests to Zendesk API endpoints (use the SDK instead)

The `node-zendesk` library is the officially recommended and most actively maintained Node.js client for the Zendesk API, with 10+ years of continuous support.

---

## Installation

```bash
npm install node-zendesk
```

For TypeScript projects, the package includes TypeScript definitions:

```bash
npm install node-zendesk
# TypeScript definitions are included in the package
```

---

## Authentication & Initialization

### API Token Authentication (Recommended)

The most common authentication method uses username, API token, and subdomain:

```javascript
const zendesk = require('node-zendesk');

const client = zendesk.createClient({
  username: 'your_email@example.com',
  token: 'your_api_token',
  subdomain: 'your_subdomain'
});
```

**Environment Variables Example:**

```javascript
require('dotenv').config();
const zendesk = require('node-zendesk');

const client = zendesk.createClient({
  username: process.env.ZENDESK_USERNAME,
  token: process.env.ZENDESK_API_TOKEN,
  subdomain: process.env.ZENDESK_SUBDOMAIN
});
```

**.env file:**

```
ZENDESK_USERNAME=your_email@example.com
ZENDESK_API_TOKEN=your_api_token_here
ZENDESK_SUBDOMAIN=your_company
```

### OAuth Token Authentication

For OAuth-based authentication:

```javascript
const zendesk = require('node-zendesk');

const client = zendesk.createClient({
  token: 'your_oauth_access_token',
  oauth: true,
  subdomain: 'your_subdomain'
});
```

**With Environment Variables:**

```javascript
const client = zendesk.createClient({
  token: process.env.ZENDESK_OAUTH_TOKEN,
  oauth: true,
  subdomain: process.env.ZENDESK_SUBDOMAIN
});
```

### TypeScript/ES6 Import

```typescript
import { createClient } from 'node-zendesk';

const client = createClient({
  username: process.env.ZENDESK_USERNAME!,
  token: process.env.ZENDESK_API_TOKEN!,
  subdomain: process.env.ZENDESK_SUBDOMAIN!
});
```

### User Impersonation

Make requests on behalf of end users (requires proper OAuth scopes):

```javascript
const client = zendesk.createClient({
  username: process.env.ZENDESK_USERNAME,
  token: process.env.ZENDESK_API_TOKEN,
  subdomain: process.env.ZENDESK_SUBDOMAIN,
  asUser: 'end-user@example.com'
});
```

### Custom Configuration

```javascript
const client = zendesk.createClient({
  username: process.env.ZENDESK_USERNAME,
  token: process.env.ZENDESK_API_TOKEN,
  subdomain: process.env.ZENDESK_SUBDOMAIN,
  remoteUri: 'https://custom.zendesk.com/api/v2',  // Custom API endpoint
  debug: true,  // Enable debug logging
  disableGlobalState: true,  // Disable global state (for serverless)
  retry: true,  // Enable automatic retries
  timeout: 30000  // Request timeout in ms (default: 60000)
});
```

---

## Core API Surfaces

### Tickets API

#### List All Tickets

**Basic Example:**

```javascript
// List all tickets
client.tickets.list()
  .then(tickets => {
    console.log('Total tickets:', tickets.length);
    tickets.forEach(ticket => {
      console.log(`#${ticket.id}: ${ticket.subject}`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example with Pagination:**

```javascript
// List tickets with pagination
async function listAllTickets() {
  let page = 1;
  let hasMore = true;

  while (hasMore) {
    const result = await client.tickets.list(page);

    result.forEach(ticket => {
      console.log(`Ticket #${ticket.id}: ${ticket.subject}`);
      console.log(`Status: ${ticket.status}, Priority: ${ticket.priority}`);
      console.log(`Created: ${ticket.created_at}`);
      console.log('---');
    });

    hasMore = result.nextPage !== null;
    page++;
  }
}

listAllTickets().catch(console.error);
```

#### Show Single Ticket

**Basic Example:**

```javascript
const ticketId = 12345;

client.tickets.show(ticketId)
  .then(ticket => {
    console.log('Subject:', ticket.subject);
    console.log('Status:', ticket.status);
    console.log('Requester ID:', ticket.requester_id);
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function getTicketDetails(ticketId) {
  try {
    const ticket = await client.tickets.show(ticketId);

    console.log('=== Ticket Details ===');
    console.log(`ID: ${ticket.id}`);
    console.log(`Subject: ${ticket.subject}`);
    console.log(`Description: ${ticket.description}`);
    console.log(`Status: ${ticket.status}`);
    console.log(`Priority: ${ticket.priority}`);
    console.log(`Type: ${ticket.type}`);
    console.log(`Requester ID: ${ticket.requester_id}`);
    console.log(`Assignee ID: ${ticket.assignee_id}`);
    console.log(`Group ID: ${ticket.group_id}`);
    console.log(`Created: ${ticket.created_at}`);
    console.log(`Updated: ${ticket.updated_at}`);
    console.log(`Tags: ${ticket.tags.join(', ')}`);

    // Custom fields
    if (ticket.custom_fields && ticket.custom_fields.length > 0) {
      console.log('Custom Fields:');
      ticket.custom_fields.forEach(field => {
        console.log(`  ${field.id}: ${field.value}`);
      });
    }

    return ticket;
  } catch (error) {
    console.error('Failed to fetch ticket:', error.message);
    throw error;
  }
}

getTicketDetails(12345);
```

#### Create Ticket

**Basic Example:**

```javascript
const newTicket = {
  subject: 'Help with product installation',
  comment: {
    body: 'I need assistance installing the product on my system.'
  },
  requester: {
    name: 'John Doe',
    email: 'john.doe@example.com'
  },
  priority: 'normal',
  status: 'new'
};

client.tickets.create(newTicket)
  .then(ticket => {
    console.log('Ticket created successfully!');
    console.log('Ticket ID:', ticket.id);
    console.log('Subject:', ticket.subject);
  })
  .catch(error => {
    console.error('Error creating ticket:', error.message);
  });
```

**Advanced Example with Custom Fields:**

```javascript
async function createDetailedTicket() {
  const ticket = {
    subject: 'Technical Support Request',
    comment: {
      body: 'Detailed description of the issue...\n\nSteps to reproduce:\n1. Step one\n2. Step two',
      public: true
    },
    requester: {
      name: 'Jane Smith',
      email: 'jane.smith@example.com',
      locale_id: 1  // English (United States)
    },
    assignee_id: 67890,
    group_id: 11111,
    priority: 'high',
    status: 'open',
    type: 'problem',
    tags: ['technical', 'urgent', 'product-bug'],
    custom_fields: [
      { id: 12345, value: 'Custom value 1' },
      { id: 67890, value: 'Custom value 2' }
    ],
    due_at: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString()  // 7 days from now
  };

  try {
    const createdTicket = await client.tickets.create(ticket);
    console.log('Ticket created:', createdTicket.id);
    return createdTicket;
  } catch (error) {
    console.error('Failed to create ticket:', error.message);
    throw error;
  }
}

createDetailedTicket();
```

#### Update Ticket

**Basic Example:**

```javascript
const ticketId = 12345;
const updates = {
  status: 'solved',
  comment: {
    body: 'This issue has been resolved.',
    public: true
  }
};

client.tickets.update(ticketId, updates)
  .then(ticket => {
    console.log('Ticket updated successfully!');
    console.log('New status:', ticket.status);
  })
  .catch(error => {
    console.error('Error updating ticket:', error.message);
  });
```

**Advanced Example:**

```javascript
async function updateTicketWithComment(ticketId) {
  const updates = {
    status: 'pending',
    priority: 'high',
    assignee_id: 98765,
    tags: ['escalated', 'requires-attention'],
    comment: {
      body: 'This ticket has been escalated to senior support.',
      author_id: 12345,
      public: false  // Internal comment
    },
    custom_fields: [
      { id: 11111, value: 'Updated value' }
    ]
  };

  try {
    const updatedTicket = await client.tickets.update(ticketId, updates);
    console.log('Ticket updated:', updatedTicket.id);
    console.log('New status:', updatedTicket.status);
    console.log('New priority:', updatedTicket.priority);
    return updatedTicket;
  } catch (error) {
    console.error('Failed to update ticket:', error.message);
    throw error;
  }
}

updateTicketWithComment(12345);
```

#### Delete Ticket

**Basic Example:**

```javascript
const ticketId = 12345;

client.tickets.delete(ticketId)
  .then(() => {
    console.log('Ticket deleted successfully');
  })
  .catch(error => {
    console.error('Error deleting ticket:', error.message);
  });
```

**Advanced Example with Confirmation:**

```javascript
async function deleteTicketSafely(ticketId) {
  try {
    // First, verify the ticket exists
    const ticket = await client.tickets.show(ticketId);
    console.log(`About to delete ticket #${ticket.id}: ${ticket.subject}`);

    // Delete the ticket
    await client.tickets.delete(ticketId);
    console.log('Ticket deleted successfully');

    return true;
  } catch (error) {
    if (error.statusCode === 404) {
      console.log('Ticket not found');
    } else {
      console.error('Failed to delete ticket:', error.message);
    }
    return false;
  }
}

deleteTicketSafely(12345);
```

#### List Ticket Comments

**Basic Example:**

```javascript
const ticketId = 12345;

client.tickets.getComments(ticketId)
  .then(comments => {
    console.log(`Found ${comments.length} comments`);
    comments.forEach(comment => {
      console.log(`Author: ${comment.author_id}`);
      console.log(`Body: ${comment.body}`);
      console.log(`Public: ${comment.public}`);
      console.log('---');
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function analyzeTicketConversation(ticketId) {
  try {
    const comments = await client.tickets.getComments(ticketId);

    const publicComments = comments.filter(c => c.public);
    const privateComments = comments.filter(c => !c.public);

    console.log('=== Ticket Conversation Analysis ===');
    console.log(`Total comments: ${comments.length}`);
    console.log(`Public comments: ${publicComments.length}`);
    console.log(`Private/Internal notes: ${privateComments.length}`);

    console.log('\n=== Comment History ===');
    comments.forEach((comment, index) => {
      console.log(`\n[Comment ${index + 1}]`);
      console.log(`ID: ${comment.id}`);
      console.log(`Author ID: ${comment.author_id}`);
      console.log(`Created: ${comment.created_at}`);
      console.log(`Type: ${comment.public ? 'Public' : 'Internal'}`);
      console.log(`Body: ${comment.body.substring(0, 100)}...`);

      if (comment.attachments && comment.attachments.length > 0) {
        console.log('Attachments:');
        comment.attachments.forEach(att => {
          console.log(`  - ${att.file_name} (${att.size} bytes)`);
        });
      }
    });

    return comments;
  } catch (error) {
    console.error('Error fetching comments:', error.message);
    throw error;
  }
}

analyzeTicketConversation(12345);
```

---

### Users API

#### List Users

**Basic Example:**

```javascript
client.users.list()
  .then(users => {
    console.log('Total users:', users.length);
    users.forEach(user => {
      console.log(`${user.name} (${user.email})`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example with Filtering:**

```javascript
async function listActiveAgents() {
  try {
    const users = await client.users.list();

    // Filter for active agents
    const agents = users.filter(user =>
      user.role === 'agent' && user.active === true
    );

    console.log(`Found ${agents.length} active agents:`);
    agents.forEach(agent => {
      console.log(`\nName: ${agent.name}`);
      console.log(`Email: ${agent.email}`);
      console.log(`Role: ${agent.role}`);
      console.log(`Timezone: ${agent.time_zone}`);
      console.log(`Locale: ${agent.locale}`);
      console.log(`Last Login: ${agent.last_login_at}`);
    });

    return agents;
  } catch (error) {
    console.error('Error fetching users:', error.message);
    throw error;
  }
}

listActiveAgents();
```

#### Show User

**Basic Example:**

```javascript
const userId = 67890;

client.users.show(userId)
  .then(user => {
    console.log('Name:', user.name);
    console.log('Email:', user.email);
    console.log('Role:', user.role);
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function getUserProfile(userId) {
  try {
    const user = await client.users.show(userId);

    console.log('=== User Profile ===');
    console.log(`ID: ${user.id}`);
    console.log(`Name: ${user.name}`);
    console.log(`Email: ${user.email}`);
    console.log(`Phone: ${user.phone || 'N/A'}`);
    console.log(`Role: ${user.role}`);
    console.log(`Active: ${user.active}`);
    console.log(`Verified: ${user.verified}`);
    console.log(`Timezone: ${user.time_zone}`);
    console.log(`Locale: ${user.locale}`);
    console.log(`Organization ID: ${user.organization_id}`);
    console.log(`Created: ${user.created_at}`);
    console.log(`Updated: ${user.updated_at}`);
    console.log(`Last Login: ${user.last_login_at}`);
    console.log(`Tags: ${user.tags.join(', ')}`);

    if (user.user_fields) {
      console.log('Custom User Fields:', user.user_fields);
    }

    return user;
  } catch (error) {
    console.error('Error fetching user:', error.message);
    throw error;
  }
}

getUserProfile(67890);
```

#### Create User

**Basic Example:**

```javascript
const newUser = {
  name: 'John Doe',
  email: 'john.doe@example.com',
  role: 'end-user'
};

client.users.create(newUser)
  .then(user => {
    console.log('User created successfully!');
    console.log('User ID:', user.id);
    console.log('Name:', user.name);
  })
  .catch(error => {
    console.error('Error creating user:', error.message);
  });
```

**Advanced Example:**

```javascript
async function createAgentUser() {
  const user = {
    name: 'Sarah Agent',
    email: 'sarah.agent@company.com',
    role: 'agent',
    phone: '+1-555-123-4567',
    time_zone: 'America/New_York',
    locale: 'en-US',
    verified: true,
    tags: ['support-team', 'tier-2'],
    user_fields: {
      department: 'Technical Support',
      employee_id: 'EMP-12345'
    },
    organization_id: 98765
  };

  try {
    const createdUser = await client.users.create(user);
    console.log('Agent created:', createdUser.id);
    console.log('Name:', createdUser.name);
    console.log('Email:', createdUser.email);
    return createdUser;
  } catch (error) {
    console.error('Failed to create user:', error.message);
    throw error;
  }
}

createAgentUser();
```

#### Update User

**Basic Example:**

```javascript
const userId = 67890;
const updates = {
  name: 'Jane Smith',
  phone: '+1-555-999-8888'
};

client.users.update(userId, updates)
  .then(user => {
    console.log('User updated successfully!');
    console.log('Name:', user.name);
    console.log('Phone:', user.phone);
  })
  .catch(error => {
    console.error('Error updating user:', error.message);
  });
```

**Advanced Example:**

```javascript
async function updateUserProfile(userId) {
  const updates = {
    name: 'Jane Smith-Johnson',
    phone: '+1-555-987-6543',
    time_zone: 'Pacific/Auckland',
    locale: 'en-GB',
    tags: ['vip', 'premium-support'],
    user_fields: {
      department: 'Engineering',
      location: 'Remote'
    },
    organization_id: 11111
  };

  try {
    const updatedUser = await client.users.update(userId, updates);
    console.log('User updated:', updatedUser.id);
    console.log('New name:', updatedUser.name);
    console.log('New timezone:', updatedUser.time_zone);
    return updatedUser;
  } catch (error) {
    console.error('Failed to update user:', error.message);
    throw error;
  }
}

updateUserProfile(67890);
```

#### Delete User

**Basic Example:**

```javascript
const userId = 67890;

client.users.delete(userId)
  .then(() => {
    console.log('User deleted successfully');
  })
  .catch(error => {
    console.error('Error deleting user:', error.message);
  });
```

#### Search Users

**Basic Example:**

```javascript
client.users.search({ query: 'john@example.com' })
  .then(users => {
    console.log('Found users:', users.length);
    users.forEach(user => {
      console.log(`${user.name} - ${user.email}`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function searchUsersByName(searchTerm) {
  try {
    const users = await client.users.search({
      query: `name:${searchTerm}`
    });

    console.log(`Found ${users.length} users matching "${searchTerm}"`);

    users.forEach(user => {
      console.log('\n---');
      console.log(`ID: ${user.id}`);
      console.log(`Name: ${user.name}`);
      console.log(`Email: ${user.email}`);
      console.log(`Role: ${user.role}`);
      console.log(`Organization: ${user.organization_id}`);
    });

    return users;
  } catch (error) {
    console.error('Search failed:', error.message);
    throw error;
  }
}

searchUsersByName('John');
```

#### Get Current User

**Basic Example:**

```javascript
client.users.me()
  .then(user => {
    console.log('Current user:', user.name);
    console.log('Email:', user.email);
    console.log('Role:', user.role);
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

---

### Organizations API

#### List Organizations

**Basic Example:**

```javascript
client.organizations.list()
  .then(organizations => {
    console.log('Total organizations:', organizations.length);
    organizations.forEach(org => {
      console.log(`${org.name} (ID: ${org.id})`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function analyzeOrganizations() {
  try {
    const organizations = await client.organizations.list();

    console.log(`Total organizations: ${organizations.length}`);

    organizations.forEach(org => {
      console.log('\n=== Organization ===');
      console.log(`ID: ${org.id}`);
      console.log(`Name: ${org.name}`);
      console.log(`Domain Names: ${org.domain_names?.join(', ') || 'None'}`);
      console.log(`Details: ${org.details || 'N/A'}`);
      console.log(`Notes: ${org.notes || 'N/A'}`);
      console.log(`Tags: ${org.tags?.join(', ') || 'None'}`);
      console.log(`Created: ${org.created_at}`);

      if (org.organization_fields) {
        console.log('Custom Fields:', org.organization_fields);
      }
    });

    return organizations;
  } catch (error) {
    console.error('Error fetching organizations:', error.message);
    throw error;
  }
}

analyzeOrganizations();
```

#### Show Organization

**Basic Example:**

```javascript
const orgId = 98765;

client.organizations.show(orgId)
  .then(org => {
    console.log('Name:', org.name);
    console.log('Details:', org.details);
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function getOrganizationDetails(orgId) {
  try {
    const org = await client.organizations.show(orgId);

    console.log('=== Organization Details ===');
    console.log(`ID: ${org.id}`);
    console.log(`Name: ${org.name}`);
    console.log(`Domain Names: ${org.domain_names?.join(', ') || 'None'}`);
    console.log(`Details: ${org.details || 'N/A'}`);
    console.log(`Notes: ${org.notes || 'N/A'}`);
    console.log(`Group ID: ${org.group_id || 'N/A'}`);
    console.log(`Tags: ${org.tags?.join(', ') || 'None'}`);
    console.log(`Created: ${org.created_at}`);
    console.log(`Updated: ${org.updated_at}`);

    if (org.organization_fields) {
      console.log('\nCustom Organization Fields:');
      Object.entries(org.organization_fields).forEach(([key, value]) => {
        console.log(`  ${key}: ${value}`);
      });
    }

    return org;
  } catch (error) {
    console.error('Error fetching organization:', error.message);
    throw error;
  }
}

getOrganizationDetails(98765);
```

#### Create Organization

**Basic Example:**

```javascript
const newOrg = {
  name: 'Acme Corporation',
  domain_names: ['acme.com']
};

client.organizations.create(newOrg)
  .then(org => {
    console.log('Organization created!');
    console.log('ID:', org.id);
    console.log('Name:', org.name);
  })
  .catch(error => {
    console.error('Error creating organization:', error.message);
  });
```

**Advanced Example:**

```javascript
async function createOrganization() {
  const organization = {
    name: 'TechStart Solutions Inc.',
    domain_names: ['techstart.com', 'techstart.io'],
    details: 'Premium enterprise customer',
    notes: 'VIP support tier, 24/7 coverage',
    tags: ['enterprise', 'vip', 'priority-support'],
    group_id: 12345,
    organization_fields: {
      industry: 'Technology',
      account_tier: 'Enterprise',
      annual_revenue: '10M+'
    }
  };

  try {
    const createdOrg = await client.organizations.create(organization);
    console.log('Organization created:', createdOrg.id);
    console.log('Name:', createdOrg.name);
    console.log('Domains:', createdOrg.domain_names.join(', '));
    return createdOrg;
  } catch (error) {
    console.error('Failed to create organization:', error.message);
    throw error;
  }
}

createOrganization();
```

#### Update Organization

**Basic Example:**

```javascript
const orgId = 98765;
const updates = {
  name: 'Acme Corporation Ltd.',
  details: 'Updated company details'
};

client.organizations.update(orgId, updates)
  .then(org => {
    console.log('Organization updated!');
    console.log('Name:', org.name);
  })
  .catch(error => {
    console.error('Error updating organization:', error.message);
  });
```

**Advanced Example:**

```javascript
async function updateOrganization(orgId) {
  const updates = {
    name: 'Acme Global Corporation',
    domain_names: ['acme.com', 'acme.global', 'acmeglobal.com'],
    details: 'Global enterprise customer with multiple subsidiaries',
    notes: 'Upgraded to platinum tier - assign dedicated account manager',
    tags: ['enterprise', 'platinum', 'global', 'strategic-account'],
    organization_fields: {
      account_tier: 'Platinum',
      contract_renewal: '2025-12-31',
      dedicated_support: 'yes'
    }
  };

  try {
    const updatedOrg = await client.organizations.update(orgId, updates);
    console.log('Organization updated:', updatedOrg.id);
    console.log('New name:', updatedOrg.name);
    console.log('Tags:', updatedOrg.tags.join(', '));
    return updatedOrg;
  } catch (error) {
    console.error('Failed to update organization:', error.message);
    throw error;
  }
}

updateOrganization(98765);
```

#### Delete Organization

**Basic Example:**

```javascript
const orgId = 98765;

client.organizations.delete(orgId)
  .then(() => {
    console.log('Organization deleted successfully');
  })
  .catch(error => {
    console.error('Error deleting organization:', error.message);
  });
```

---

### Search API

#### Basic Search

**Basic Example:**

```javascript
client.search.query('type:ticket status:open')
  .then(results => {
    console.log('Found tickets:', results.length);
    results.forEach(ticket => {
      console.log(`#${ticket.id}: ${ticket.subject}`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function searchOpenTickets() {
  try {
    const query = 'type:ticket status:open priority:high';
    const results = await client.search.query(query);

    console.log(`Found ${results.length} high-priority open tickets`);

    results.forEach(ticket => {
      console.log('\n---');
      console.log(`Ticket #${ticket.id}`);
      console.log(`Subject: ${ticket.subject}`);
      console.log(`Priority: ${ticket.priority}`);
      console.log(`Status: ${ticket.status}`);
      console.log(`Created: ${ticket.created_at}`);
      console.log(`Assignee: ${ticket.assignee_id || 'Unassigned'}`);
    });

    return results;
  } catch (error) {
    console.error('Search failed:', error.message);
    throw error;
  }
}

searchOpenTickets();
```

#### Search Tickets

**Basic Example:**

```javascript
client.search.query('type:ticket subject:login')
  .then(tickets => {
    console.log('Tickets mentioning "login":', tickets.length);
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example with Multiple Filters:**

```javascript
async function advancedTicketSearch() {
  const searchParams = {
    type: 'ticket',
    status: 'open',
    priority: 'urgent',
    tags: 'bug',
    created_at: '>2025-01-01'
  };

  // Build query string
  const query = Object.entries(searchParams)
    .map(([key, value]) => `${key}:${value}`)
    .join(' ');

  try {
    const tickets = await client.search.query(query);

    console.log(`Search Query: ${query}`);
    console.log(`Results: ${tickets.length} tickets\n`);

    tickets.forEach(ticket => {
      console.log(`#${ticket.id}: ${ticket.subject}`);
      console.log(`  Status: ${ticket.status} | Priority: ${ticket.priority}`);
      console.log(`  Tags: ${ticket.tags.join(', ')}`);
      console.log(`  Created: ${ticket.created_at}`);
    });

    return tickets;
  } catch (error) {
    console.error('Search error:', error.message);
    throw error;
  }
}

advancedTicketSearch();
```

#### Search Users

**Basic Example:**

```javascript
client.search.query('type:user email:*@example.com')
  .then(users => {
    console.log('Found users:', users.length);
    users.forEach(user => {
      console.log(`${user.name} - ${user.email}`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function findAgentsByOrganization(orgId) {
  const query = `type:user role:agent organization_id:${orgId}`;

  try {
    const agents = await client.search.query(query);

    console.log(`Found ${agents.length} agents in organization ${orgId}`);

    agents.forEach(agent => {
      console.log('\n---');
      console.log(`Name: ${agent.name}`);
      console.log(`Email: ${agent.email}`);
      console.log(`Role: ${agent.role}`);
      console.log(`Active: ${agent.active}`);
      console.log(`Last Login: ${agent.last_login_at || 'Never'}`);
    });

    return agents;
  } catch (error) {
    console.error('User search failed:', error.message);
    throw error;
  }
}

findAgentsByOrganization(98765);
```

#### Search Organizations

**Basic Example:**

```javascript
client.search.query('type:organization name:Acme')
  .then(orgs => {
    console.log('Found organizations:', orgs.length);
    orgs.forEach(org => {
      console.log(`${org.name} (ID: ${org.id})`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function searchOrganizationsByDomain(domain) {
  const query = `type:organization ${domain}`;

  try {
    const organizations = await client.search.query(query);

    console.log(`Found ${organizations.length} organizations with domain "${domain}"`);

    organizations.forEach(org => {
      console.log('\n=== Organization ===');
      console.log(`ID: ${org.id}`);
      console.log(`Name: ${org.name}`);
      console.log(`Domains: ${org.domain_names?.join(', ') || 'None'}`);
      console.log(`Details: ${org.details || 'N/A'}`);
      console.log(`Created: ${org.created_at}`);
    });

    return organizations;
  } catch (error) {
    console.error('Organization search failed:', error.message);
    throw error;
  }
}

searchOrganizationsByDomain('acme.com');
```

---

### Groups API

#### List Groups

**Basic Example:**

```javascript
client.groups.list()
  .then(groups => {
    console.log('Total groups:', groups.length);
    groups.forEach(group => {
      console.log(`${group.name} (ID: ${group.id})`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function listAllGroups() {
  try {
    const groups = await client.groups.list();

    console.log(`Found ${groups.length} groups\n`);

    groups.forEach(group => {
      console.log('=== Group ===');
      console.log(`ID: ${group.id}`);
      console.log(`Name: ${group.name}`);
      console.log(`Description: ${group.description || 'N/A'}`);
      console.log(`Created: ${group.created_at}`);
      console.log(`Updated: ${group.updated_at}`);
      console.log('---');
    });

    return groups;
  } catch (error) {
    console.error('Error fetching groups:', error.message);
    throw error;
  }
}

listAllGroups();
```

#### Show Group

**Basic Example:**

```javascript
const groupId = 12345;

client.groups.show(groupId)
  .then(group => {
    console.log('Group:', group.name);
    console.log('Description:', group.description);
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

#### Create Group

**Basic Example:**

```javascript
const newGroup = {
  name: 'Technical Support Team'
};

client.groups.create(newGroup)
  .then(group => {
    console.log('Group created!');
    console.log('ID:', group.id);
    console.log('Name:', group.name);
  })
  .catch(error => {
    console.error('Error creating group:', error.message);
  });
```

**Advanced Example:**

```javascript
async function createSupportGroup() {
  const group = {
    name: 'Enterprise Support Team',
    description: 'Dedicated support team for enterprise customers'
  };

  try {
    const createdGroup = await client.groups.create(group);
    console.log('Group created:', createdGroup.id);
    console.log('Name:', createdGroup.name);
    console.log('Description:', createdGroup.description);
    return createdGroup;
  } catch (error) {
    console.error('Failed to create group:', error.message);
    throw error;
  }
}

createSupportGroup();
```

#### Update Group

**Basic Example:**

```javascript
const groupId = 12345;
const updates = {
  name: 'Updated Group Name',
  description: 'Updated description'
};

client.groups.update(groupId, updates)
  .then(group => {
    console.log('Group updated!');
    console.log('Name:', group.name);
  })
  .catch(error => {
    console.error('Error updating group:', error.message);
  });
```

#### Delete Group

**Basic Example:**

```javascript
const groupId = 12345;

client.groups.delete(groupId)
  .then(() => {
    console.log('Group deleted successfully');
  })
  .catch(error => {
    console.error('Error deleting group:', error.message);
  });
```

---

### Attachments API

#### Upload Attachment

**Basic Example:**

```javascript
const fs = require('fs');
const path = require('path');

const filePath = path.resolve('./documents/file.pdf');

client.attachments.upload(filePath, { filename: 'file.pdf' })
  .then(upload => {
    console.log('File uploaded successfully!');
    console.log('Upload token:', upload.token);
    console.log('Attachment:', upload.attachment);
  })
  .catch(error => {
    console.error('Error uploading file:', error.message);
  });
```

**Advanced Example with Ticket Creation:**

```javascript
const fs = require('fs');
const path = require('path');

async function createTicketWithAttachment() {
  try {
    // Step 1: Upload the file
    const filePath = path.resolve('./screenshots/bug-screenshot.png');
    const upload = await client.attachments.upload(filePath, {
      filename: 'bug-screenshot.png'
    });

    console.log('File uploaded. Token:', upload.token);

    // Step 2: Create ticket with attachment
    const ticket = {
      subject: 'Bug Report: UI Issue',
      comment: {
        body: 'Please see the attached screenshot showing the UI bug.',
        uploads: [upload.token]  // Reference the upload token
      },
      requester: {
        name: 'Bug Reporter',
        email: 'reporter@example.com'
      },
      priority: 'high',
      tags: ['bug', 'ui']
    };

    const createdTicket = await client.tickets.create(ticket);

    console.log('Ticket created with attachment!');
    console.log('Ticket ID:', createdTicket.id);
    console.log('Subject:', createdTicket.subject);

    return createdTicket;
  } catch (error) {
    console.error('Error:', error.message);
    throw error;
  }
}

createTicketWithAttachment();
```

#### Upload Multiple Attachments

**Advanced Example:**

```javascript
const fs = require('fs');
const path = require('path');

async function createTicketWithMultipleAttachments() {
  try {
    // Upload multiple files
    const files = [
      './documents/report.pdf',
      './screenshots/screen1.png',
      './screenshots/screen2.png'
    ];

    const uploadPromises = files.map(file => {
      const filePath = path.resolve(file);
      const filename = path.basename(file);
      return client.attachments.upload(filePath, { filename });
    });

    const uploads = await Promise.all(uploadPromises);
    const tokens = uploads.map(u => u.token);

    console.log(`Uploaded ${tokens.length} files`);

    // Create ticket with all attachments
    const ticket = {
      subject: 'Detailed Bug Report',
      comment: {
        body: 'Please find the attached report and screenshots.',
        uploads: tokens
      },
      requester: {
        name: 'QA Tester',
        email: 'qa@example.com'
      },
      priority: 'normal'
    };

    const createdTicket = await client.tickets.create(ticket);

    console.log('Ticket created with multiple attachments!');
    console.log('Ticket ID:', createdTicket.id);

    return createdTicket;
  } catch (error) {
    console.error('Error:', error.message);
    throw error;
  }
}

createTicketWithMultipleAttachments();
```

#### Show Attachment

**Basic Example:**

```javascript
const attachmentId = 123456789;

client.attachments.show(attachmentId)
  .then(attachment => {
    console.log('Filename:', attachment.file_name);
    console.log('Size:', attachment.size, 'bytes');
    console.log('Content Type:', attachment.content_type);
    console.log('URL:', attachment.content_url);
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

#### Delete Attachment Upload

**Basic Example:**

```javascript
const uploadToken = 'upload_token_here';

client.attachments.deleteUpload(uploadToken)
  .then(() => {
    console.log('Upload deleted successfully');
  })
  .catch(error => {
    console.error('Error deleting upload:', error.message);
  });
```

---

### Macros API

#### List Macros

**Basic Example:**

```javascript
client.macros.list()
  .then(macros => {
    console.log('Total macros:', macros.length);
    macros.forEach(macro => {
      console.log(`${macro.title} (ID: ${macro.id})`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function listActiveMacros() {
  try {
    const macros = await client.macros.list();

    const activeMacros = macros.filter(m => m.active);

    console.log(`Found ${activeMacros.length} active macros\n`);

    activeMacros.forEach(macro => {
      console.log('=== Macro ===');
      console.log(`ID: ${macro.id}`);
      console.log(`Title: ${macro.title}`);
      console.log(`Description: ${macro.description || 'N/A'}`);
      console.log(`Active: ${macro.active}`);
      console.log(`Created: ${macro.created_at}`);
      console.log('---');
    });

    return activeMacros;
  } catch (error) {
    console.error('Error fetching macros:', error.message);
    throw error;
  }
}

listActiveMacros();
```

#### Show Macro

**Basic Example:**

```javascript
const macroId = 54321;

client.macros.show(macroId)
  .then(macro => {
    console.log('Title:', macro.title);
    console.log('Description:', macro.description);
    console.log('Actions:', macro.actions);
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

#### Apply Macro to Ticket

**Advanced Example:**

```javascript
async function applyMacroToTicket(ticketId, macroId) {
  try {
    const result = await client.macros.applyTicket(ticketId, macroId);

    console.log('Macro applied successfully!');
    console.log('Result:', result);

    // Update the ticket with the macro's changes
    const updates = result.ticket;
    const updatedTicket = await client.tickets.update(ticketId, updates);

    console.log('Ticket updated with macro actions');
    console.log('New status:', updatedTicket.status);

    return updatedTicket;
  } catch (error) {
    console.error('Error applying macro:', error.message);
    throw error;
  }
}

applyMacroToTicket(12345, 54321);
```

---

### Views API

#### List Views

**Basic Example:**

```javascript
client.views.list()
  .then(views => {
    console.log('Total views:', views.length);
    views.forEach(view => {
      console.log(`${view.title} (ID: ${view.id})`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function listActiveViews() {
  try {
    const views = await client.views.list();

    const activeViews = views.filter(v => v.active);

    console.log(`Found ${activeViews.length} active views\n`);

    activeViews.forEach(view => {
      console.log('=== View ===');
      console.log(`ID: ${view.id}`);
      console.log(`Title: ${view.title}`);
      console.log(`Description: ${view.description || 'N/A'}`);
      console.log(`Active: ${view.active}`);
      console.log(`Position: ${view.position}`);
      console.log('---');
    });

    return activeViews;
  } catch (error) {
    console.error('Error fetching views:', error.message);
    throw error;
  }
}

listActiveViews();
```

#### Execute View

**Basic Example:**

```javascript
const viewId = 11111;

client.views.execute(viewId)
  .then(result => {
    console.log('View executed!');
    console.log('Tickets:', result.rows.length);
    result.rows.forEach(ticket => {
      console.log(`#${ticket.id}: ${ticket.subject}`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function executeViewAndAnalyze(viewId) {
  try {
    const result = await client.views.execute(viewId);

    console.log('=== View Execution Results ===');
    console.log(`Total tickets: ${result.rows.length}`);
    console.log(`Columns: ${result.columns.map(c => c.title).join(', ')}`);

    // Analyze ticket statuses
    const statusCounts = {};
    result.rows.forEach(ticket => {
      const status = ticket.status;
      statusCounts[status] = (statusCounts[status] || 0) + 1;
    });

    console.log('\n=== Status Distribution ===');
    Object.entries(statusCounts).forEach(([status, count]) => {
      console.log(`${status}: ${count}`);
    });

    // Show first 10 tickets
    console.log('\n=== Tickets ===');
    result.rows.slice(0, 10).forEach(ticket => {
      console.log(`#${ticket.id}: ${ticket.subject}`);
      console.log(`  Status: ${ticket.status} | Priority: ${ticket.priority}`);
    });

    return result;
  } catch (error) {
    console.error('Error executing view:', error.message);
    throw error;
  }
}

executeViewAndAnalyze(11111);
```

---

### Triggers API

#### List Triggers

**Basic Example:**

```javascript
client.triggers.list()
  .then(triggers => {
    console.log('Total triggers:', triggers.length);
    triggers.forEach(trigger => {
      console.log(`${trigger.title} (ID: ${trigger.id})`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function listActiveTriggers() {
  try {
    const triggers = await client.triggers.list();

    const activeTriggers = triggers.filter(t => t.active);

    console.log(`Found ${activeTriggers.length} active triggers\n`);

    activeTriggers.forEach(trigger => {
      console.log('=== Trigger ===');
      console.log(`ID: ${trigger.id}`);
      console.log(`Title: ${trigger.title}`);
      console.log(`Description: ${trigger.description || 'N/A'}`);
      console.log(`Active: ${trigger.active}`);
      console.log(`Position: ${trigger.position}`);
      console.log(`Conditions:`, JSON.stringify(trigger.conditions, null, 2));
      console.log(`Actions:`, JSON.stringify(trigger.actions, null, 2));
      console.log('---');
    });

    return activeTriggers;
  } catch (error) {
    console.error('Error fetching triggers:', error.message);
    throw error;
  }
}

listActiveTriggers();
```

---

### Automations API

#### List Automations

**Basic Example:**

```javascript
client.automations.list()
  .then(automations => {
    console.log('Total automations:', automations.length);
    automations.forEach(automation => {
      console.log(`${automation.title} (ID: ${automation.id})`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function listActiveAutomations() {
  try {
    const automations = await client.automations.list();

    const activeAutomations = automations.filter(a => a.active);

    console.log(`Found ${activeAutomations.length} active automations\n`);

    activeAutomations.forEach(automation => {
      console.log('=== Automation ===');
      console.log(`ID: ${automation.id}`);
      console.log(`Title: ${automation.title}`);
      console.log(`Active: ${automation.active}`);
      console.log(`Position: ${automation.position}`);
      console.log(`Conditions:`, JSON.stringify(automation.conditions, null, 2));
      console.log(`Actions:`, JSON.stringify(automation.actions, null, 2));
      console.log('---');
    });

    return activeAutomations;
  } catch (error) {
    console.error('Error fetching automations:', error.message);
    throw error;
  }
}

listActiveAutomations();
```

---

### Brands API

#### List Brands

**Basic Example:**

```javascript
client.brands.list()
  .then(brands => {
    console.log('Total brands:', brands.length);
    brands.forEach(brand => {
      console.log(`${brand.name} (ID: ${brand.id})`);
    });
  })
  .catch(error => {
    console.error('Error:', error.message);
  });
```

**Advanced Example:**

```javascript
async function listAllBrands() {
  try {
    const brands = await client.brands.list();

    console.log(`Found ${brands.length} brands\n`);

    brands.forEach(brand => {
      console.log('=== Brand ===');
      console.log(`ID: ${brand.id}`);
      console.log(`Name: ${brand.name}`);
      console.log(`Subdomain: ${brand.subdomain}`);
      console.log(`Host Mapping: ${brand.host_mapping || 'N/A'}`);
      console.log(`Active: ${brand.active}`);
      console.log(`Default: ${brand.default}`);
      console.log(`Created: ${brand.created_at}`);
      console.log('---');
    });

    return brands;
  } catch (error) {
    console.error('Error fetching brands:', error.message);
    throw error;
  }
}

listAllBrands();
```

---

## Error Handling

### Basic Error Handling

```javascript
client.tickets.show(99999)
  .then(ticket => {
    console.log('Ticket:', ticket.subject);
  })
  .catch(error => {
    console.error('Error occurred:', error.message);
    console.error('Status code:', error.statusCode);
  });
```

### Advanced Error Handling

```javascript
async function handleZendeskErrors() {
  try {
    const ticket = await client.tickets.show(99999);
    return ticket;
  } catch (error) {
    // Check error type
    if (error.statusCode === 404) {
      console.error('Ticket not found');
    } else if (error.statusCode === 401) {
      console.error('Authentication failed - check credentials');
    } else if (error.statusCode === 403) {
      console.error('Forbidden - insufficient permissions');
    } else if (error.statusCode === 429) {
      console.error('Rate limit exceeded - retry after:', error.retry);
    } else if (error.statusCode >= 500) {
      console.error('Zendesk server error - try again later');
    } else {
      console.error('Unknown error:', error.message);
    }

    // Log full error details
    console.error('Error details:', {
      statusCode: error.statusCode,
      message: error.message,
      stack: error.stack
    });

    throw error;
  }
}
```

### Retry Logic

```javascript
async function fetchTicketWithRetry(ticketId, maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const ticket = await client.tickets.show(ticketId);
      return ticket;
    } catch (error) {
      if (error.statusCode === 429 && attempt < maxRetries) {
        // Rate limited - wait and retry
        const waitTime = Math.pow(2, attempt) * 1000; // Exponential backoff
        console.log(`Rate limited. Retrying in ${waitTime}ms...`);
        await new Promise(resolve => setTimeout(resolve, waitTime));
      } else if (attempt === maxRetries) {
        console.error(`Failed after ${maxRetries} attempts`);
        throw error;
      } else if (error.statusCode >= 500 && attempt < maxRetries) {
        // Server error - retry
        console.log(`Server error. Attempt ${attempt}/${maxRetries}`);
        await new Promise(resolve => setTimeout(resolve, 1000));
      } else {
        throw error;
      }
    }
  }
}
```

---

## Complete Working Examples

### Basic Support Ticket System

```javascript
require('dotenv').config();
const zendesk = require('node-zendesk');

const client = zendesk.createClient({
  username: process.env.ZENDESK_USERNAME,
  token: process.env.ZENDESK_API_TOKEN,
  subdomain: process.env.ZENDESK_SUBDOMAIN
});

async function createSupportTicket(customerEmail, subject, description) {
  try {
    // Check if user exists
    let user;
    const users = await client.users.search({ query: customerEmail });

    if (users.length > 0) {
      user = users[0];
      console.log('Found existing user:', user.name);
    } else {
      // Create new user
      user = await client.users.create({
        name: customerEmail.split('@')[0],
        email: customerEmail,
        role: 'end-user'
      });
      console.log('Created new user:', user.name);
    }

    // Create ticket
    const ticket = await client.tickets.create({
      subject: subject,
      comment: {
        body: description
      },
      requester_id: user.id,
      priority: 'normal',
      status: 'new'
    });

    console.log('Ticket created successfully!');
    console.log('Ticket ID:', ticket.id);
    console.log('Subject:', ticket.subject);

    return ticket;
  } catch (error) {
    console.error('Error creating support ticket:', error.message);
    throw error;
  }
}

// Usage
createSupportTicket(
  'customer@example.com',
  'Cannot access my account',
  'I am unable to log into my account. I keep getting an error message.'
);
```

### Ticket Management Dashboard

```javascript
require('dotenv').config();
const zendesk = require('node-zendesk');

const client = zendesk.createClient({
  username: process.env.ZENDESK_USERNAME,
  token: process.env.ZENDESK_API_TOKEN,
  subdomain: process.env.ZENDESK_SUBDOMAIN
});

async function getDashboardStats() {
  try {
    // Search for different ticket categories
    const openTickets = await client.search.query('type:ticket status:open');
    const pendingTickets = await client.search.query('type:ticket status:pending');
    const solvedTickets = await client.search.query('type:ticket status:solved created>2025-01-01');
    const urgentTickets = await client.search.query('type:ticket priority:urgent status<solved');

    console.log('=== Support Dashboard ===');
    console.log(`Open Tickets: ${openTickets.length}`);
    console.log(`Pending Tickets: ${pendingTickets.length}`);
    console.log(`Solved This Year: ${solvedTickets.length}`);
    console.log(`Urgent Tickets: ${urgentTickets.length}`);

    // List urgent tickets
    if (urgentTickets.length > 0) {
      console.log('\n=== Urgent Tickets Requiring Attention ===');
      urgentTickets.forEach(ticket => {
        console.log(`#${ticket.id}: ${ticket.subject}`);
        console.log(`  Status: ${ticket.status} | Created: ${ticket.created_at}`);
        console.log(`  Assignee: ${ticket.assignee_id || 'Unassigned'}`);
      });
    }

    return {
      open: openTickets.length,
      pending: pendingTickets.length,
      solved: solvedTickets.length,
      urgent: urgentTickets.length
    };
  } catch (error) {
    console.error('Error fetching dashboard stats:', error.message);
    throw error;
  }
}

getDashboardStats();
```

### Bulk Ticket Operations

```javascript
require('dotenv').config();
const zendesk = require('node-zendesk');

const client = zendesk.createClient({
  username: process.env.ZENDESK_USERNAME,
  token: process.env.ZENDESK_API_TOKEN,
  subdomain: process.env.ZENDESK_SUBDOMAIN
});

async function bulkUpdateTickets(ticketIds, updates) {
  try {
    console.log(`Updating ${ticketIds.length} tickets...`);

    const updatePromises = ticketIds.map(async (ticketId) => {
      try {
        const ticket = await client.tickets.update(ticketId, updates);
        console.log(`✓ Updated ticket #${ticketId}`);
        return { success: true, ticketId, ticket };
      } catch (error) {
        console.error(`✗ Failed to update ticket #${ticketId}:`, error.message);
        return { success: false, ticketId, error: error.message };
      }
    });

    const results = await Promise.all(updatePromises);

    const successful = results.filter(r => r.success).length;
    const failed = results.filter(r => !r.success).length;

    console.log(`\n=== Bulk Update Complete ===`);
    console.log(`Successful: ${successful}`);
    console.log(`Failed: ${failed}`);

    return results;
  } catch (error) {
    console.error('Bulk update error:', error.message);
    throw error;
  }
}

// Usage: Close multiple tickets at once
const ticketIds = [12345, 12346, 12347];
const updates = {
  status: 'solved',
  comment: {
    body: 'This ticket has been resolved and closed.',
    public: false
  }
};

bulkUpdateTickets(ticketIds, updates);
```
