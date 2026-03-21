---
name: crm
description: "HubSpot Node.js SDK for managing CRM contacts, companies, deals, and marketing automation via the HubSpot API."
metadata:
  languages: "javascript"
  versions: "13.4.0"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "hubspot,crm,marketing,contacts,automation"
---

# HubSpot JavaScript/Node.js SDK Coding Guidelines

You are a HubSpot API coding expert. Help me write code using the HubSpot API with the official Node.js SDK.

Official SDK documentation and code samples:
https://developers.hubspot.com/docs/api/overview

## Golden Rule: Use the Correct and Current SDK

Always use the official HubSpot Node.js SDK (`@hubspot/api-client`), which is the standard library for all HubSpot API interactions. Do not use deprecated or unofficial packages.

- **Library Name:** HubSpot Node.js SDK
- **NPM Package:** `@hubspot/api-client`
- **Current Version:** 13.4.0
- **Deprecated Packages:** `hubspot`, `hubspot-api`, `@hubspot/integrations-framework-actions` (do not use these)

**Installation:**

- **Correct:** `npm install @hubspot/api-client`
- **Incorrect:** `npm install hubspot` or `npm install hubspot-api`

**APIs and Usage:**

- **Correct:** `const hubspot = require('@hubspot/api-client')`
- **Correct:** `const hubspotClient = new hubspot.Client({ accessToken: token })`
- **Correct:** `await hubspotClient.crm.contacts.basicApi.create(...)`
- **Incorrect:** `HubspotClient` or `HubspotAPI`
- **Incorrect:** Legacy v2 API endpoints

## Installation

Install the SDK via npm:

```bash
npm install @hubspot/api-client
```

For TypeScript projects, the SDK includes built-in type definitions.

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

```javascript
const hubspot = require('@hubspot/api-client');

// Initialize with access token from environment variable
const hubspotClient = new hubspot.Client({
  accessToken: process.env.HUBSPOT_ACCESS_TOKEN
});
```

### Initialization with OAuth

```javascript
const hubspot = require('@hubspot/api-client');

const hubspotClient = new hubspot.Client({
  accessToken: 'YOUR_OAUTH_ACCESS_TOKEN'
});

// Refresh token when needed
hubspotClient.setAccessToken('NEW_ACCESS_TOKEN');
```

### Initialization with Rate Limiting Options

```javascript
const hubspot = require('@hubspot/api-client');

const hubspotClient = new hubspot.Client({
  accessToken: process.env.HUBSPOT_ACCESS_TOKEN,
  limiterOptions: {
    maxConcurrent: 5,
    minTime: 100
  }
});
```

## CRM API - Contacts

### Create a Contact

**Basic Example:**

```javascript
const hubspot = require('@hubspot/api-client');
const hubspotClient = new hubspot.Client({ accessToken: process.env.HUBSPOT_ACCESS_TOKEN });

async function createContact() {
  const contactObj = {
    properties: {
      email: 'example@company.com',
      firstname: 'John',
      lastname: 'Doe',
      phone: '555-0100',
      company: 'Example Company',
      website: 'example.com'
    }
  };

  try {
    const apiResponse = await hubspotClient.crm.contacts.basicApi.create(contactObj);
    console.log('Contact created:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error creating contact:', error.message);
  }
}

createContact();
```

**Advanced Example with Custom Properties:**

```javascript
async function createContactAdvanced() {
  const contactObj = {
    properties: {
      email: 'jane@example.com',
      firstname: 'Jane',
      lastname: 'Smith',
      phone: '555-0200',
      company: 'Tech Corp',
      jobtitle: 'Software Engineer',
      lifecyclestage: 'lead',
      hs_lead_status: 'NEW',
      // Custom properties
      custom_field: 'custom_value',
      industry: 'Technology'
    }
  };

  try {
    const apiResponse = await hubspotClient.crm.contacts.basicApi.create(contactObj);
    console.log('Contact created with ID:', apiResponse.id);
    console.log('Properties:', apiResponse.properties);
    return apiResponse;
  } catch (error) {
    if (error.statusCode === 409) {
      console.error('Contact with this email already exists');
    } else {
      console.error('Error creating contact:', error.message);
    }
  }
}
```

### Get a Contact

**By ID:**

```javascript
async function getContact(contactId) {
  try {
    const apiResponse = await hubspotClient.crm.contacts.basicApi.getById(
      contactId,
      ['email', 'firstname', 'lastname', 'phone', 'company'] // properties to return
    );
    console.log('Contact:', apiResponse.properties);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching contact:', error.message);
  }
}
```

**By Email:**

```javascript
async function getContactByEmail(email) {
  const filter = { propertyName: 'email', operator: 'EQ', value: email };
  const filterGroup = { filters: [filter] };
  const sort = JSON.stringify({ propertyName: 'createdate', direction: 'DESCENDING' });
  const properties = ['email', 'firstname', 'lastname'];
  const limit = 1;

  try {
    const apiResponse = await hubspotClient.crm.contacts.searchApi.doSearch({
      filterGroups: [filterGroup],
      sorts: [sort],
      properties,
      limit
    });

    if (apiResponse.total > 0) {
      return apiResponse.results[0];
    }
    return null;
  } catch (error) {
    console.error('Error searching contact:', error.message);
  }
}
```

### Update a Contact

```javascript
async function updateContact(contactId) {
  const properties = {
    firstname: 'Updated Name',
    phone: '555-9999',
    lifecyclestage: 'opportunity'
  };

  try {
    const apiResponse = await hubspotClient.crm.contacts.basicApi.update(
      contactId,
      { properties }
    );
    console.log('Contact updated:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error updating contact:', error.message);
  }
}
```

### List Contacts

**Basic Pagination:**

```javascript
async function listContacts() {
  const limit = 10;
  const properties = ['email', 'firstname', 'lastname', 'company'];

  try {
    const apiResponse = await hubspotClient.crm.contacts.basicApi.getPage(
      limit,
      undefined, // after (for pagination)
      properties
    );

    console.log(`Found ${apiResponse.results.length} contacts`);
    apiResponse.results.forEach(contact => {
      console.log(`${contact.properties.firstname} ${contact.properties.lastname}`);
    });

    return apiResponse;
  } catch (error) {
    console.error('Error listing contacts:', error.message);
  }
}
```

**Get All Contacts (with pagination handling):**

```javascript
async function getAllContacts() {
  const allContacts = [];
  let after = undefined;
  const limit = 100;

  try {
    do {
      const apiResponse = await hubspotClient.crm.contacts.basicApi.getPage(
        limit,
        after,
        ['email', 'firstname', 'lastname']
      );

      allContacts.push(...apiResponse.results);
      after = apiResponse.paging?.next?.after;

    } while (after);

    console.log(`Total contacts retrieved: ${allContacts.length}`);
    return allContacts;
  } catch (error) {
    console.error('Error getting all contacts:', error.message);
  }
}
```

### Delete a Contact

```javascript
async function deleteContact(contactId) {
  try {
    await hubspotClient.crm.contacts.basicApi.archive(contactId);
    console.log('Contact deleted:', contactId);
  } catch (error) {
    console.error('Error deleting contact:', error.message);
  }
}
```

## CRM API - Companies

### Create a Company

```javascript
async function createCompany() {
  const companyObj = {
    properties: {
      name: 'Example Company',
      domain: 'example.com',
      city: 'San Francisco',
      state: 'California',
      industry: 'Technology',
      phone: '555-0100',
      numberofemployees: '50',
      description: 'A technology company'
    }
  };

  try {
    const apiResponse = await hubspotClient.crm.companies.basicApi.create(companyObj);
    console.log('Company created:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error creating company:', error.message);
  }
}
```

### Get a Company

```javascript
async function getCompany(companyId) {
  const properties = ['name', 'domain', 'city', 'industry', 'phone'];

  try {
    const apiResponse = await hubspotClient.crm.companies.basicApi.getById(
      companyId,
      properties
    );
    console.log('Company:', apiResponse.properties);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching company:', error.message);
  }
}
```

### Update a Company

```javascript
async function updateCompany(companyId) {
  const properties = {
    name: 'Updated Company Name',
    numberofemployees: '100',
    annualrevenue: '1000000'
  };

  try {
    const apiResponse = await hubspotClient.crm.companies.basicApi.update(
      companyId,
      { properties }
    );
    console.log('Company updated:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error updating company:', error.message);
  }
}
```

### List Companies

```javascript
async function listCompanies() {
  const limit = 10;
  const properties = ['name', 'domain', 'city', 'industry'];

  try {
    const apiResponse = await hubspotClient.crm.companies.basicApi.getPage(
      limit,
      undefined,
      properties
    );

    console.log(`Found ${apiResponse.results.length} companies`);
    return apiResponse;
  } catch (error) {
    console.error('Error listing companies:', error.message);
  }
}
```

## CRM API - Deals

### Create a Deal

```javascript
async function createDeal() {
  const dealObj = {
    properties: {
      dealname: 'New Deal',
      dealstage: 'appointmentscheduled',
      amount: '10000',
      closedate: '2025-12-31',
      pipeline: 'default',
      hubspot_owner_id: '12345'
    }
  };

  try {
    const apiResponse = await hubspotClient.crm.deals.basicApi.create(dealObj);
    console.log('Deal created:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error creating deal:', error.message);
  }
}
```

### Get a Deal

```javascript
async function getDeal(dealId) {
  const properties = ['dealname', 'dealstage', 'amount', 'closedate'];

  try {
    const apiResponse = await hubspotClient.crm.deals.basicApi.getById(
      dealId,
      properties
    );
    console.log('Deal:', apiResponse.properties);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching deal:', error.message);
  }
}
```

### Update a Deal

```javascript
async function updateDeal(dealId) {
  const properties = {
    dealstage: 'closedwon',
    amount: '15000',
    closedate: '2025-11-30'
  };

  try {
    const apiResponse = await hubspotClient.crm.deals.basicApi.update(
      dealId,
      { properties }
    );
    console.log('Deal updated:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error updating deal:', error.message);
  }
}
```

### List Deals

```javascript
async function listDeals() {
  const limit = 10;
  const properties = ['dealname', 'dealstage', 'amount'];

  try {
    const apiResponse = await hubspotClient.crm.deals.basicApi.getPage(
      limit,
      undefined,
      properties
    );

    console.log(`Found ${apiResponse.results.length} deals`);
    return apiResponse;
  } catch (error) {
    console.error('Error listing deals:', error.message);
  }
}
```

## CRM API - Tickets

### Create a Ticket

```javascript
async function createTicket() {
  const ticketObj = {
    properties: {
      subject: 'Customer Support Request',
      content: 'Customer needs help with product setup',
      hs_pipeline: '0',
      hs_pipeline_stage: '1',
      hs_ticket_priority: 'HIGH',
      hubspot_owner_id: '12345'
    }
  };

  try {
    const apiResponse = await hubspotClient.crm.tickets.basicApi.create(ticketObj);
    console.log('Ticket created:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error creating ticket:', error.message);
  }
}
```

### Get a Ticket

```javascript
async function getTicket(ticketId) {
  const properties = ['subject', 'content', 'hs_pipeline_stage', 'hs_ticket_priority'];

  try {
    const apiResponse = await hubspotClient.crm.tickets.basicApi.getById(
      ticketId,
      properties
    );
    console.log('Ticket:', apiResponse.properties);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching ticket:', error.message);
  }
}
```

### Update a Ticket

```javascript
async function updateTicket(ticketId) {
  const properties = {
    hs_pipeline_stage: '4', // Closed
    hs_ticket_priority: 'LOW'
  };

  try {
    const apiResponse = await hubspotClient.crm.tickets.basicApi.update(
      ticketId,
      { properties }
    );
    console.log('Ticket updated:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error updating ticket:', error.message);
  }
}
```

## CRM API - Associations

Associations link objects together (e.g., contacts to companies, deals to contacts).

### Create Association

**Associate Contact with Company:**

```javascript
async function associateContactWithCompany(contactId, companyId) {
  try {
    const apiResponse = await hubspotClient.crm.contacts.associationsApi.create(
      contactId,
      'companies',
      companyId,
      'contact_to_company'
    );
    console.log('Association created');
    return apiResponse;
  } catch (error) {
    console.error('Error creating association:', error.message);
  }
}
```

**Associate Deal with Contact:**

```javascript
async function associateDealWithContact(dealId, contactId) {
  try {
    const apiResponse = await hubspotClient.crm.deals.associationsApi.create(
      dealId,
      'contacts',
      contactId,
      'deal_to_contact'
    );
    console.log('Association created');
    return apiResponse;
  } catch (error) {
    console.error('Error creating association:', error.message);
  }
}
```

### Get Associations

```javascript
async function getContactAssociations(contactId) {
  try {
    const companies = await hubspotClient.crm.contacts.associationsApi.getAll(
      contactId,
      'companies'
    );

    const deals = await hubspotClient.crm.contacts.associationsApi.getAll(
      contactId,
      'deals'
    );

    console.log('Associated companies:', companies.results);
    console.log('Associated deals:', deals.results);

    return { companies, deals };
  } catch (error) {
    console.error('Error fetching associations:', error.message);
  }
}
```

### Remove Association

```javascript
async function removeAssociation(contactId, companyId) {
  try {
    await hubspotClient.crm.contacts.associationsApi.archive(
      contactId,
      'companies',
      companyId,
      'contact_to_company'
    );
    console.log('Association removed');
  } catch (error) {
    console.error('Error removing association:', error.message);
  }
}
```

## CRM API - Batch Operations

Batch operations allow you to create, update, or read up to 100 records in a single API call.

### Batch Create Contacts

```javascript
async function batchCreateContacts() {
  const batchInputs = {
    inputs: [
      {
        properties: {
          email: 'contact1@example.com',
          firstname: 'Contact',
          lastname: 'One'
        }
      },
      {
        properties: {
          email: 'contact2@example.com',
          firstname: 'Contact',
          lastname: 'Two'
        }
      },
      {
        properties: {
          email: 'contact3@example.com',
          firstname: 'Contact',
          lastname: 'Three'
        }
      }
    ]
  };

  try {
    const apiResponse = await hubspotClient.crm.contacts.batchApi.create(batchInputs);
    console.log(`Created ${apiResponse.results.length} contacts`);
    return apiResponse;
  } catch (error) {
    console.error('Error batch creating contacts:', error.message);
  }
}
```

### Batch Update Contacts

```javascript
async function batchUpdateContacts(contactIds) {
  const batchInputs = {
    inputs: contactIds.map(id => ({
      id: id,
      properties: {
        lifecyclestage: 'customer',
        hs_lead_status: 'CONNECTED'
      }
    }))
  };

  try {
    const apiResponse = await hubspotClient.crm.contacts.batchApi.update(batchInputs);
    console.log(`Updated ${apiResponse.results.length} contacts`);
    return apiResponse;
  } catch (error) {
    console.error('Error batch updating contacts:', error.message);
  }
}
```

### Batch Read Contacts

```javascript
async function batchReadContacts(contactIds) {
  const batchInputs = {
    properties: ['email', 'firstname', 'lastname', 'phone'],
    inputs: contactIds.map(id => ({ id }))
  };

  try {
    const apiResponse = await hubspotClient.crm.contacts.batchApi.read(batchInputs);
    console.log(`Retrieved ${apiResponse.results.length} contacts`);
    return apiResponse;
  } catch (error) {
    console.error('Error batch reading contacts:', error.message);
  }
}
```

### Batch Upsert Contacts

```javascript
async function batchUpsertContacts() {
  const batchInputs = {
    inputs: [
      {
        properties: {
          email: 'existing@example.com',
          firstname: 'Updated',
          lastname: 'Name'
        },
        idProperty: 'email'
      },
      {
        properties: {
          email: 'new@example.com',
          firstname: 'New',
          lastname: 'Contact'
        },
        idProperty: 'email'
      }
    ]
  };

  try {
    const apiResponse = await hubspotClient.crm.contacts.batchApi.upsert(batchInputs);
    console.log(`Upserted ${apiResponse.results.length} contacts`);
    return apiResponse;
  } catch (error) {
    console.error('Error batch upserting contacts:', error.message);
  }
}
```

## CRM API - Search

The Search API allows you to filter, sort, and search across CRM objects.

### Basic Search

```javascript
async function searchContacts() {
  const filter = {
    propertyName: 'lifecyclestage',
    operator: 'EQ',
    value: 'lead'
  };

  const filterGroup = {
    filters: [filter]
  };

  const searchRequest = {
    filterGroups: [filterGroup],
    properties: ['email', 'firstname', 'lastname', 'lifecyclestage'],
    limit: 10
  };

  try {
    const apiResponse = await hubspotClient.crm.contacts.searchApi.doSearch(searchRequest);
    console.log(`Found ${apiResponse.total} contacts`);
    console.log(`Returned ${apiResponse.results.length} results`);
    return apiResponse;
  } catch (error) {
    console.error('Error searching contacts:', error.message);
  }
}
```

### Advanced Search with Multiple Filters

```javascript
async function advancedSearchContacts() {
  const searchRequest = {
    filterGroups: [
      {
        filters: [
          {
            propertyName: 'lifecyclestage',
            operator: 'EQ',
            value: 'lead'
          },
          {
            propertyName: 'createdate',
            operator: 'GTE',
            value: '2025-01-01'
          }
        ]
      }
    ],
    sorts: [
      {
        propertyName: 'createdate',
        direction: 'DESCENDING'
      }
    ],
    properties: ['email', 'firstname', 'lastname', 'createdate', 'lifecyclestage'],
    limit: 100,
    after: 0
  };

  try {
    const apiResponse = await hubspotClient.crm.contacts.searchApi.doSearch(searchRequest);
    console.log(`Found ${apiResponse.total} matching contacts`);
    return apiResponse;
  } catch (error) {
    console.error('Error searching contacts:', error.message);
  }
}
```

### Search with Association Filters

```javascript
async function searchContactsByCompany(companyId) {
  const searchRequest = {
    filterGroups: [
      {
        filters: [
          {
            propertyName: 'associations.company',
            operator: 'EQ',
            value: companyId
          }
        ]
      }
    ],
    properties: ['email', 'firstname', 'lastname'],
    limit: 100
  };

  try {
    const apiResponse = await hubspotClient.crm.contacts.searchApi.doSearch(searchRequest);
    console.log(`Found ${apiResponse.results.length} contacts for company`);
    return apiResponse;
  } catch (error) {
    console.error('Error searching contacts by company:', error.message);
  }
}
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

```javascript
async function getAllContactProperties() {
  try {
    const apiResponse = await hubspotClient.crm.properties.coreApi.getAll('contacts');
    console.log(`Found ${apiResponse.results.length} contact properties`);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching contact properties:', error.message);
  }
}
```

### Create Custom Property

```javascript
async function createCustomProperty() {
  const propertyObj = {
    name: 'favorite_color',
    label: 'Favorite Color',
    type: 'string',
    fieldType: 'text',
    groupName: 'contactinformation',
    description: 'The contact\'s favorite color',
    options: []
  };

  try {
    const apiResponse = await hubspotClient.crm.properties.coreApi.create(
      'contacts',
      propertyObj
    );
    console.log('Custom property created:', apiResponse.name);
    return apiResponse;
  } catch (error) {
    console.error('Error creating custom property:', error.message);
  }
}
```

### Create Dropdown Property

```javascript
async function createDropdownProperty() {
  const propertyObj = {
    name: 'customer_tier',
    label: 'Customer Tier',
    type: 'enumeration',
    fieldType: 'select',
    groupName: 'contactinformation',
    options: [
      { label: 'Bronze', value: 'bronze', displayOrder: 0 },
      { label: 'Silver', value: 'silver', displayOrder: 1 },
      { label: 'Gold', value: 'gold', displayOrder: 2 },
      { label: 'Platinum', value: 'platinum', displayOrder: 3 }
    ]
  };

  try {
    const apiResponse = await hubspotClient.crm.properties.coreApi.create(
      'contacts',
      propertyObj
    );
    console.log('Dropdown property created:', apiResponse.name);
    return apiResponse;
  } catch (error) {
    console.error('Error creating dropdown property:', error.message);
  }
}
```

### Update Property

```javascript
async function updateProperty(propertyName) {
  const propertyUpdate = {
    label: 'Updated Label',
    description: 'Updated description'
  };

  try {
    const apiResponse = await hubspotClient.crm.properties.coreApi.update(
      'contacts',
      propertyName,
      propertyUpdate
    );
    console.log('Property updated:', apiResponse.name);
    return apiResponse;
  } catch (error) {
    console.error('Error updating property:', error.message);
  }
}
```

## Marketing API - Emails

### Get All Marketing Emails

```javascript
async function getMarketingEmails() {
  try {
    const apiResponse = await hubspotClient.marketing.emails.emailsApi.getPage();
    console.log(`Found ${apiResponse.results.length} marketing emails`);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching marketing emails:', error.message);
  }
}
```

### Get Email by ID

```javascript
async function getMarketingEmail(emailId) {
  try {
    const apiResponse = await hubspotClient.marketing.emails.emailsApi.getById(emailId);
    console.log('Email:', apiResponse.name);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching marketing email:', error.message);
  }
}
```

## Marketing API - Forms

### Get All Forms

```javascript
async function getAllForms() {
  try {
    const apiResponse = await hubspotClient.marketing.forms.formsApi.getPage();
    console.log(`Found ${apiResponse.results.length} forms`);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching forms:', error.message);
  }
}
```

### Get Form by ID

```javascript
async function getForm(formId) {
  try {
    const apiResponse = await hubspotClient.marketing.forms.formsApi.getById(formId);
    console.log('Form:', apiResponse.name);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching form:', error.message);
  }
}
```

### Submit Form Data

```javascript
async function submitFormData(formGuid, portalId) {
  const formData = {
    fields: [
      {
        name: 'email',
        value: 'test@example.com'
      },
      {
        name: 'firstname',
        value: 'John'
      },
      {
        name: 'lastname',
        value: 'Doe'
      }
    ],
    context: {
      pageUri: 'https://example.com/contact',
      pageName: 'Contact Us'
    }
  };

  try {
    const axios = require('axios');
    const response = await axios.post(
      `https://api.hsforms.com/submissions/v3/integration/submit/${portalId}/${formGuid}`,
      formData
    );
    console.log('Form submitted successfully');
    return response.data;
  } catch (error) {
    console.error('Error submitting form:', error.message);
  }
}
```

## Events API - Timeline Events

### Create Timeline Event Type

```javascript
async function createTimelineEventType() {
  const eventTypeObj = {
    name: 'Custom Event',
    applicationId: 12345, // Your app ID
    objectType: 'CONTACT',
    headerTemplate: '{{eventTitle}}',
    detailTemplate: '{{eventDescription}}'
  };

  try {
    const apiResponse = await hubspotClient.crm.timeline.eventsApi.createEventType(
      eventTypeObj
    );
    console.log('Timeline event type created:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error creating timeline event type:', error.message);
  }
}
```

### Create Timeline Event

```javascript
async function createTimelineEvent(eventTypeId, objectId) {
  const eventData = {
    eventTypeId: eventTypeId,
    objectId: objectId,
    extraData: {
      eventTitle: 'Purchase Completed',
      eventDescription: 'Customer completed a purchase of $99.99'
    },
    timestamp: new Date().toISOString()
  };

  try {
    const apiResponse = await hubspotClient.crm.timeline.eventsApi.create(eventData);
    console.log('Timeline event created:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error creating timeline event:', error.message);
  }
}
```

## Automation API - Workflows

### Get All Workflows

```javascript
async function getAllWorkflows() {
  try {
    const axios = require('axios');
    const response = await axios.get(
      'https://api.hubapi.com/automation/v3/workflows',
      {
        headers: {
          Authorization: `Bearer ${process.env.HUBSPOT_ACCESS_TOKEN}`
        }
      }
    );
    console.log(`Found ${response.data.workflows.length} workflows`);
    return response.data;
  } catch (error) {
    console.error('Error fetching workflows:', error.message);
  }
}
```

### Get Workflow by ID

```javascript
async function getWorkflow(workflowId) {
  try {
    const axios = require('axios');
    const response = await axios.get(
      `https://api.hubapi.com/automation/v3/workflows/${workflowId}`,
      {
        headers: {
          Authorization: `Bearer ${process.env.HUBSPOT_ACCESS_TOKEN}`
        }
      }
    );
    console.log('Workflow:', response.data.name);
    return response.data;
  } catch (error) {
    console.error('Error fetching workflow:', error.message);
  }
}
```

### Enroll Contact in Workflow

```javascript
async function enrollContactInWorkflow(workflowId, contactEmail) {
  try {
    const axios = require('axios');
    const response = await axios.post(
      `https://api.hubapi.com/automation/v2/workflows/${workflowId}/enrollments/contacts/${contactEmail}`,
      {},
      {
        headers: {
          Authorization: `Bearer ${process.env.HUBSPOT_ACCESS_TOKEN}`,
          'Content-Type': 'application/json'
        }
      }
    );
    console.log('Contact enrolled in workflow');
    return response.data;
  } catch (error) {
    console.error('Error enrolling contact in workflow:', error.message);
  }
}
```

## Webhooks API

### Create Webhook Subscription

```javascript
async function createWebhookSubscription() {
  const subscriptionObj = {
    eventType: 'contact.creation',
    propertyName: undefined,
    active: true
  };

  try {
    const apiResponse = await hubspotClient.webhooks.subscriptionsApi.create(
      12345, // Your app ID
      subscriptionObj
    );
    console.log('Webhook subscription created:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error creating webhook subscription:', error.message);
  }
}
```

### Get All Webhook Subscriptions

```javascript
async function getWebhookSubscriptions(appId) {
  try {
    const apiResponse = await hubspotClient.webhooks.subscriptionsApi.getAll(appId);
    console.log(`Found ${apiResponse.results.length} webhook subscriptions`);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching webhook subscriptions:', error.message);
  }
}
```

### Delete Webhook Subscription

```javascript
async function deleteWebhookSubscription(appId, subscriptionId) {
  try {
    await hubspotClient.webhooks.subscriptionsApi.archive(appId, subscriptionId);
    console.log('Webhook subscription deleted');
  } catch (error) {
    console.error('Error deleting webhook subscription:', error.message);
  }
}
```

## Lists API

### Get All Lists

```javascript
async function getAllLists() {
  try {
    const axios = require('axios');
    const response = await axios.get(
      'https://api.hubapi.com/contacts/v1/lists',
      {
        headers: {
          Authorization: `Bearer ${process.env.HUBSPOT_ACCESS_TOKEN}`
        },
        params: {
          count: 100,
          offset: 0
        }
      }
    );
    console.log(`Found ${response.data.lists.length} lists`);
    return response.data;
  } catch (error) {
    console.error('Error fetching lists:', error.message);
  }
}
```

### Add Contact to List

```javascript
async function addContactToList(listId, contactId) {
  try {
    const axios = require('axios');
    const response = await axios.post(
      `https://api.hubapi.com/contacts/v1/lists/${listId}/add`,
      {
        vids: [contactId]
      },
      {
        headers: {
          Authorization: `Bearer ${process.env.HUBSPOT_ACCESS_TOKEN}`,
          'Content-Type': 'application/json'
        }
      }
    );
    console.log('Contact added to list');
    return response.data;
  } catch (error) {
    console.error('Error adding contact to list:', error.message);
  }
}
```

### Remove Contact from List

```javascript
async function removeContactFromList(listId, contactId) {
  try {
    const axios = require('axios');
    const response = await axios.post(
      `https://api.hubapi.com/contacts/v1/lists/${listId}/remove`,
      {
        vids: [contactId]
      },
      {
        headers: {
          Authorization: `Bearer ${process.env.HUBSPOT_ACCESS_TOKEN}`,
          'Content-Type': 'application/json'
        }
      }
    );
    console.log('Contact removed from list');
    return response.data;
  } catch (error) {
    console.error('Error removing contact from list:', error.message);
  }
}
```

## Owners API

### Get All Owners

```javascript
async function getAllOwners() {
  try {
    const apiResponse = await hubspotClient.crm.owners.ownersApi.getPage();
    console.log(`Found ${apiResponse.results.length} owners`);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching owners:', error.message);
  }
}
```

### Get Owner by ID

```javascript
async function getOwner(ownerId) {
  try {
    const apiResponse = await hubspotClient.crm.owners.ownersApi.getById(ownerId);
    console.log('Owner:', apiResponse.email);
    return apiResponse;
  } catch (error) {
    console.error('Error fetching owner:', error.message);
  }
}
```

## Error Handling

### Comprehensive Error Handling

```javascript
async function handleApiCall() {
  try {
    const apiResponse = await hubspotClient.crm.contacts.basicApi.getById('12345');
    return apiResponse;
  } catch (error) {
    if (error.statusCode === 401) {
      console.error('Authentication failed - check your access token');
    } else if (error.statusCode === 404) {
      console.error('Resource not found');
    } else if (error.statusCode === 409) {
      console.error('Conflict - resource already exists');
    } else if (error.statusCode === 429) {
      console.error('Rate limit exceeded - wait before retrying');
    } else if (error.statusCode >= 500) {
      console.error('HubSpot server error - try again later');
    } else {
      console.error('API error:', error.message);
    }
    throw error;
  }
}
```

### Retry Logic

```javascript
async function apiCallWithRetry(apiCall, maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await apiCall();
    } catch (error) {
      if (error.statusCode === 429 && attempt < maxRetries) {
        const retryAfter = error.response?.headers['retry-after'] || 1;
        console.log(`Rate limited. Retrying after ${retryAfter} seconds...`);
        await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
      } else {
        throw error;
      }
    }
  }
}

// Usage
await apiCallWithRetry(() =>
  hubspotClient.crm.contacts.basicApi.getById('12345')
);
```

## Rate Limits

HubSpot API has the following rate limits:

- **Private Apps:** 100 requests per 10 seconds per account
- **OAuth Apps:** 100 requests per 10 seconds per app per account
- **Search API:** 5 requests per second, 1000 requests per day (per account)
- **Batch API:** 100 objects per request, same rate limits as standard endpoints

The SDK automatically handles rate limiting with the `limiterOptions` configuration.

## Pagination

Most list endpoints return paginated results. Use the `after` parameter for pagination:

```javascript
async function paginateThroughContacts() {
  let after = undefined;
  let allContacts = [];

  do {
    const apiResponse = await hubspotClient.crm.contacts.basicApi.getPage(
      100,
      after,
      ['email', 'firstname', 'lastname']
    );

    allContacts.push(...apiResponse.results);
    after = apiResponse.paging?.next?.after;

  } while (after);

  return allContacts;
}
```

## OAuth Implementation

### Initialize OAuth Provider

```javascript
const hubspot = require('@hubspot/api-client');

const hubspotClient = new hubspot.Client({
  clientId: process.env.HUBSPOT_CLIENT_ID,
  clientSecret: process.env.HUBSPOT_CLIENT_SECRET,
  redirectUri: 'https://yourapp.com/oauth-callback'
});
```

### Get Authorization URL

```javascript
function getAuthorizationUrl() {
  const authUrl = hubspotClient.oauth.getAuthorizationUrl(
    process.env.HUBSPOT_CLIENT_ID,
    process.env.REDIRECT_URI,
    'contacts crm.objects.contacts.read'
  );
  return authUrl;
}
```

### Exchange Authorization Code for Tokens

```javascript
async function getTokens(authorizationCode) {
  try {
    const tokenResponse = await hubspotClient.oauth.tokensApi.create(
      'authorization_code',
      authorizationCode,
      process.env.REDIRECT_URI,
      process.env.HUBSPOT_CLIENT_ID,
      process.env.HUBSPOT_CLIENT_SECRET
    );

    return {
      accessToken: tokenResponse.accessToken,
      refreshToken: tokenResponse.refreshToken,
      expiresIn: tokenResponse.expiresIn
    };
  } catch (error) {
    console.error('Error exchanging code for tokens:', error.message);
  }
}
```

### Refresh Access Token

```javascript
async function refreshAccessToken(refreshToken) {
  try {
    const tokenResponse = await hubspotClient.oauth.tokensApi.create(
      'refresh_token',
      undefined,
      undefined,
      process.env.HUBSPOT_CLIENT_ID,
      process.env.HUBSPOT_CLIENT_SECRET,
      refreshToken
    );

    return {
      accessToken: tokenResponse.accessToken,
      refreshToken: tokenResponse.refreshToken,
      expiresIn: tokenResponse.expiresIn
    };
  } catch (error) {
    console.error('Error refreshing token:', error.message);
  }
}
```

## Custom Objects

### Create Custom Object Schema

```javascript
async function createCustomObjectSchema() {
  const schemaObj = {
    name: 'cars',
    labels: {
      singular: 'Car',
      plural: 'Cars'
    },
    primaryDisplayProperty: 'model',
    requiredProperties: ['model', 'make'],
    searchableProperties: ['model', 'make', 'year'],
    properties: [
      {
        name: 'model',
        label: 'Model',
        type: 'string',
        fieldType: 'text'
      },
      {
        name: 'make',
        label: 'Make',
        type: 'string',
        fieldType: 'text'
      },
      {
        name: 'year',
        label: 'Year',
        type: 'number',
        fieldType: 'number'
      }
    ]
  };

  try {
    const apiResponse = await hubspotClient.crm.schemas.coreApi.create(schemaObj);
    console.log('Custom object schema created:', apiResponse.name);
    return apiResponse;
  } catch (error) {
    console.error('Error creating custom object schema:', error.message);
  }
}
```

### Create Custom Object Instance

```javascript
async function createCustomObject(objectType) {
  const objectData = {
    properties: {
      model: 'Model 3',
      make: 'Tesla',
      year: '2024'
    }
  };

  try {
    const apiResponse = await hubspotClient.crm.objects.basicApi.create(
      objectType,
      objectData
    );
    console.log('Custom object created:', apiResponse.id);
    return apiResponse;
  } catch (error) {
    console.error('Error creating custom object:', error.message);
  }
}
```

## Complete Example Application

Here's a complete example that demonstrates multiple API operations:

```javascript
const hubspot = require('@hubspot/api-client');
require('dotenv').config();

const hubspotClient = new hubspot.Client({
  accessToken: process.env.HUBSPOT_ACCESS_TOKEN
});

async function main() {
  try {
    // Create a contact
    console.log('Creating contact...');
    const newContact = await hubspotClient.crm.contacts.basicApi.create({
      properties: {
        email: 'john.doe@example.com',
        firstname: 'John',
        lastname: 'Doe',
        phone: '555-0100',
        company: 'Example Corp'
      }
    });
    console.log(`Contact created with ID: ${newContact.id}`);

    // Create a company
    console.log('\nCreating company...');
    const newCompany = await hubspotClient.crm.companies.basicApi.create({
      properties: {
        name: 'Example Corp',
        domain: 'example.com',
        city: 'San Francisco',
        industry: 'Technology'
      }
    });
    console.log(`Company created with ID: ${newCompany.id}`);

    // Associate contact with company
    console.log('\nAssociating contact with company...');
    await hubspotClient.crm.contacts.associationsApi.create(
      newContact.id,
      'companies',
      newCompany.id,
      'contact_to_company'
    );
    console.log('Association created successfully');

    // Create a deal
    console.log('\nCreating deal...');
    const newDeal = await hubspotClient.crm.deals.basicApi.create({
      properties: {
        dealname: 'Example Deal',
        dealstage: 'appointmentscheduled',
        amount: '10000',
        closedate: '2025-12-31'
      }
    });
    console.log(`Deal created with ID: ${newDeal.id}`);

    // Associate deal with contact
    console.log('\nAssociating deal with contact...');
    await hubspotClient.crm.deals.associationsApi.create(
      newDeal.id,
      'contacts',
      newContact.id,
      'deal_to_contact'
    );
    console.log('Deal associated with contact');

    // Search for contacts
    console.log('\nSearching for leads...');
    const searchResults = await hubspotClient.crm.contacts.searchApi.doSearch({
      filterGroups: [
        {
          filters: [
            {
              propertyName: 'email',
              operator: 'CONTAINS_TOKEN',
              value: 'example.com'
            }
          ]
        }
      ],
      properties: ['email', 'firstname', 'lastname', 'company'],
      limit: 10
    });
    console.log(`Found ${searchResults.total} contacts`);

    // List all owners
    console.log('\nFetching owners...');
    const owners = await hubspotClient.crm.owners.ownersApi.getPage();
    console.log(`Found ${owners.results.length} owners`);

    console.log('\nAll operations completed successfully!');
  } catch (error) {
    console.error('Error:', error.message);
    if (error.body) {
      console.error('Error details:', JSON.stringify(error.body, null, 2));
    }
  }
}

main();
```

## TypeScript Support

The SDK includes full TypeScript definitions:

```typescript
import { Client } from '@hubspot/api-client';

const hubspotClient = new Client({
  accessToken: process.env.HUBSPOT_ACCESS_TOKEN
});

interface ContactProperties {
  email: string;
  firstname: string;
  lastname: string;
  phone?: string;
}

async function createContact(props: ContactProperties) {
  const apiResponse = await hubspotClient.crm.contacts.basicApi.create({
    properties: props
  });
  return apiResponse;
}
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
```javascript
// DON'T USE THIS
const hapikey = 'your-api-key';
```

**New (Correct):**
```javascript
const hubspotClient = new hubspot.Client({
  accessToken: process.env.HUBSPOT_ACCESS_TOKEN
});
```

Legacy API keys will be revoked on November 19, 2025.
