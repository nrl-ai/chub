---
name: marketing
description: "Mailchimp Marketing API Node.js SDK Coding Guidelines for email marketing, audience management, and campaign management"
metadata:
  languages: "javascript"
  versions: "3.0.80"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "mailchimp,marketing,email,campaigns,audience"
---

# Mailchimp Marketing API Node.js SDK Coding Guidelines (JavaScript/TypeScript)

You are a Mailchimp Marketing API coding expert. Help me with writing code using the official Mailchimp Marketing Node.js library for email marketing, audience management, campaign management, and e-commerce integrations. Please follow the following guidelines when generating code.

You can find the official documentation and code samples here: https://mailchimp.com/developer/marketing/

## Golden Rule: Use the Correct and Current SDK

Always use the official Mailchimp Marketing Node.js SDK, which is the standard library for all Mailchimp Marketing API interactions.

- **Primary Package:** `@mailchimp/mailchimp_marketing`
- **GitHub Repository:** https://github.com/mailchimp/mailchimp-marketing-node
- **Current Version:** 3.0.80

**Installation:**

```bash
npm install @mailchimp/mailchimp_marketing
```

**APIs and Usage:**

- **Correct:** `const mailchimp = require('@mailchimp/mailchimp_marketing')`
- **Correct:** `mailchimp.setConfig({ apiKey: '...', server: '...' })`
- **Correct:** `await mailchimp.lists.getAllLists()`
- **Correct:** `await mailchimp.campaigns.create({ ... })`
- **Incorrect:** Using legacy or unofficial packages like `mailchimp`, `mailchimp-api-v3`, or `node-mailchimp`

## Prerequisites and Setup

Before using the Mailchimp Marketing Node.js library, ensure you have:

- Node.js installed (version 12 or higher recommended)
- A Mailchimp account with an API key
- Your Mailchimp server prefix (e.g., `us19`, `us6`)

## Finding Your Server Prefix

The server prefix is required for API authentication. To find it:

1. Log into your Mailchimp account
2. Look at the URL in your browser: `https://us19.admin.mailchimp.com/`
3. The `us19` part is your server prefix

Alternatively, your API key ends with the server prefix: if your key is `abc123-us6`, then `us6` is your server prefix.

## API Key Configuration

**Never hardcode your API key.** Always use environment variables:

```bash
# .env file
MAILCHIMP_API_KEY=your_api_key_here
MAILCHIMP_SERVER_PREFIX=us19
```

```javascript
require('dotenv').config();

const mailchimp = require('@mailchimp/mailchimp_marketing');

mailchimp.setConfig({
  apiKey: process.env.MAILCHIMP_API_KEY,
  server: process.env.MAILCHIMP_SERVER_PREFIX,
});
```

## Initialization

### Basic Authentication (API Key)

```javascript
const mailchimp = require('@mailchimp/mailchimp_marketing');

mailchimp.setConfig({
  apiKey: process.env.MAILCHIMP_API_KEY,
  server: process.env.MAILCHIMP_SERVER_PREFIX,
});

// Test connection
async function testConnection() {
  try {
    const response = await mailchimp.ping.get();
    console.log('Connected to Mailchimp:', response);
  } catch (error) {
    console.error('Connection failed:', error);
  }
}
```

### OAuth2 Authentication

For applications that access Mailchimp on behalf of other users:

```javascript
const mailchimp = require('@mailchimp/mailchimp_marketing');

mailchimp.setConfig({
  accessToken: process.env.MAILCHIMP_ACCESS_TOKEN,
  server: process.env.MAILCHIMP_SERVER_PREFIX,
});
```

## Audiences (Lists) Management

### Get All Lists

```javascript
async function getAllLists() {
  try {
    const response = await mailchimp.lists.getAllLists();
    console.log('Total lists:', response.total_items);

    response.lists.forEach(list => {
      console.log(`List: ${list.name} (ID: ${list.id})`);
      console.log(`Members: ${list.stats.member_count}`);
    });

    return response.lists;
  } catch (error) {
    console.error('Error fetching lists:', error);
  }
}
```

### Get Specific List Information

```javascript
async function getListInfo(listId) {
  try {
    const response = await mailchimp.lists.getList(listId);
    console.log('List Name:', response.name);
    console.log('Total Members:', response.stats.member_count);
    console.log('Unsubscribe Count:', response.stats.unsubscribe_count);
    console.log('Open Rate:', response.stats.open_rate);
    console.log('Click Rate:', response.stats.click_rate);
    return response;
  } catch (error) {
    console.error('Error fetching list info:', error);
  }
}
```

### Create a New List

```javascript
async function createList() {
  try {
    const response = await mailchimp.lists.createList({
      name: 'My New Newsletter',
      contact: {
        company: 'My Company',
        address1: '123 Main St',
        city: 'New York',
        state: 'NY',
        zip: '10001',
        country: 'US',
      },
      permission_reminder: 'You signed up for updates on our website.',
      campaign_defaults: {
        from_name: 'My Company',
        from_email: 'hello@mycompany.com',
        subject: 'Newsletter',
        language: 'en',
      },
      email_type_option: true,
    });

    console.log('List created:', response.id);
    return response;
  } catch (error) {
    console.error('Error creating list:', error);
  }
}
```

## List Members Management

### Add a Member to a List

```javascript
async function addMemberToList(listId, email, firstName, lastName) {
  try {
    const response = await mailchimp.lists.addListMember(listId, {
      email_address: email,
      status: 'subscribed',
      merge_fields: {
        FNAME: firstName,
        LNAME: lastName,
      },
    });

    console.log(`Added ${email} to list`);
    return response;
  } catch (error) {
    console.error('Error adding member:', error);
    if (error.status === 400) {
      console.error('Member might already exist or email is invalid');
    }
  }
}
```

### Add or Update a Member

Use this to avoid errors when a member already exists:

```javascript
const crypto = require('crypto');

async function addOrUpdateMember(listId, email, firstName, lastName, tags = []) {
  try {
    // Create MD5 hash of lowercase email for subscriber_hash
    const subscriberHash = crypto
      .createHash('md5')
      .update(email.toLowerCase())
      .digest('hex');

    const response = await mailchimp.lists.setListMember(
      listId,
      subscriberHash,
      {
        email_address: email,
        status_if_new: 'subscribed',
        merge_fields: {
          FNAME: firstName,
          LNAME: lastName,
        },
        tags: tags,
      }
    );

    console.log(`Added/Updated ${email}`);
    return response;
  } catch (error) {
    console.error('Error adding/updating member:', error);
  }
}
```

### Get List Members

```javascript
async function getListMembers(listId, count = 100) {
  try {
    const response = await mailchimp.lists.getListMembersInfo(listId, {
      count: count,
      offset: 0,
    });

    console.log(`Total members: ${response.total_items}`);

    response.members.forEach(member => {
      console.log(`${member.email_address} - ${member.status}`);
    });

    return response.members;
  } catch (error) {
    console.error('Error fetching members:', error);
  }
}
```

### Get Specific Member Information

```javascript
const crypto = require('crypto');

async function getMemberInfo(listId, email) {
  try {
    const subscriberHash = crypto
      .createHash('md5')
      .update(email.toLowerCase())
      .digest('hex');

    const response = await mailchimp.lists.getListMember(
      listId,
      subscriberHash
    );

    console.log('Member:', response.email_address);
    console.log('Status:', response.status);
    console.log('Member since:', response.timestamp_opt);
    console.log('Tags:', response.tags);

    return response;
  } catch (error) {
    console.error('Error fetching member:', error);
  }
}
```

### Update Member Information

```javascript
const crypto = require('crypto');

async function updateMember(listId, email, updates) {
  try {
    const subscriberHash = crypto
      .createHash('md5')
      .update(email.toLowerCase())
      .digest('hex');

    const response = await mailchimp.lists.updateListMember(
      listId,
      subscriberHash,
      updates
    );

    console.log('Member updated:', response.email_address);
    return response;
  } catch (error) {
    console.error('Error updating member:', error);
  }
}

// Example usage
updateMember('list123', 'user@example.com', {
  merge_fields: {
    FNAME: 'Jane',
    LNAME: 'Smith',
  },
  status: 'subscribed',
});
```

### Delete a Member

```javascript
const crypto = require('crypto');

async function deleteMember(listId, email) {
  try {
    const subscriberHash = crypto
      .createHash('md5')
      .update(email.toLowerCase())
      .digest('hex');

    await mailchimp.lists.deleteListMember(listId, subscriberHash);
    console.log(`Deleted ${email} from list`);
  } catch (error) {
    console.error('Error deleting member:', error);
  }
}
```

### Batch Subscribe or Unsubscribe

```javascript
async function batchSubscribe(listId, members) {
  try {
    const response = await mailchimp.lists.batchListMembers(listId, {
      members: members.map(member => ({
        email_address: member.email,
        status: 'subscribed',
        merge_fields: {
          FNAME: member.firstName,
          LNAME: member.lastName,
        },
      })),
      update_existing: true,
    });

    console.log('New members:', response.new_members.length);
    console.log('Updated members:', response.updated_members.length);
    console.log('Errors:', response.errors.length);

    return response;
  } catch (error) {
    console.error('Error batch subscribing:', error);
  }
}

// Example usage
batchSubscribe('list123', [
  { email: 'user1@example.com', firstName: 'John', lastName: 'Doe' },
  { email: 'user2@example.com', firstName: 'Jane', lastName: 'Smith' },
]);
```

## Tags Management

### Add Tags to a Member

```javascript
const crypto = require('crypto');

async function addTagsToMember(listId, email, tags) {
  try {
    const subscriberHash = crypto
      .createHash('md5')
      .update(email.toLowerCase())
      .digest('hex');

    const response = await mailchimp.lists.updateListMemberTags(
      listId,
      subscriberHash,
      {
        tags: tags.map(tag => ({ name: tag, status: 'active' })),
      }
    );

    console.log(`Added tags to ${email}`);
    return response;
  } catch (error) {
    console.error('Error adding tags:', error);
  }
}

// Example usage
addTagsToMember('list123', 'user@example.com', ['VIP', 'Newsletter']);
```

### Remove Tags from a Member

```javascript
const crypto = require('crypto');

async function removeTagsFromMember(listId, email, tags) {
  try {
    const subscriberHash = crypto
      .createHash('md5')
      .update(email.toLowerCase())
      .digest('hex');

    const response = await mailchimp.lists.updateListMemberTags(
      listId,
      subscriberHash,
      {
        tags: tags.map(tag => ({ name: tag, status: 'inactive' })),
      }
    );

    console.log(`Removed tags from ${email}`);
    return response;
  } catch (error) {
    console.error('Error removing tags:', error);
  }
}
```

## Segments Management

### Get All Segments for a List

```javascript
async function getSegments(listId) {
  try {
    const response = await mailchimp.lists.listSegments(listId);

    console.log('Total segments:', response.total_items);

    response.segments.forEach(segment => {
      console.log(`Segment: ${segment.name} (ID: ${segment.id})`);
      console.log(`Type: ${segment.type}, Members: ${segment.member_count}`);
    });

    return response.segments;
  } catch (error) {
    console.error('Error fetching segments:', error);
  }
}
```

### Create a Segment

```javascript
async function createSegment(listId, segmentName, conditions) {
  try {
    const response = await mailchimp.lists.createSegment(listId, {
      name: segmentName,
      static_segment: [],
    });

    console.log('Segment created:', response.id);
    return response;
  } catch (error) {
    console.error('Error creating segment:', error);
  }
}
```

### Add Members to a Segment

```javascript
async function addMembersToSegment(listId, segmentId, emails) {
  try {
    const response = await mailchimp.lists.batchSegmentMembers(
      listId,
      segmentId,
      {
        members_to_add: emails,
      }
    );

    console.log('Members added to segment:', response.members_added.length);
    return response;
  } catch (error) {
    console.error('Error adding members to segment:', error);
  }
}
```

## Campaigns Management

### Get All Campaigns

```javascript
async function getAllCampaigns(count = 100) {
  try {
    const response = await mailchimp.campaigns.list({
      count: count,
      sort_field: 'create_time',
      sort_dir: 'DESC',
    });

    console.log('Total campaigns:', response.total_items);

    response.campaigns.forEach(campaign => {
      console.log(`Campaign: ${campaign.settings.title}`);
      console.log(`Status: ${campaign.status}, Type: ${campaign.type}`);
    });

    return response.campaigns;
  } catch (error) {
    console.error('Error fetching campaigns:', error);
  }
}
```

### Create a Campaign

```javascript
async function createCampaign(listId, subject, fromName, replyTo) {
  try {
    const response = await mailchimp.campaigns.create({
      type: 'regular',
      recipients: {
        list_id: listId,
      },
      settings: {
        subject_line: subject,
        title: subject,
        from_name: fromName,
        reply_to: replyTo,
      },
    });

    console.log('Campaign created:', response.id);
    return response;
  } catch (error) {
    console.error('Error creating campaign:', error);
  }
}
```

### Set Campaign Content

```javascript
async function setCampaignContent(campaignId, htmlContent) {
  try {
    const response = await mailchimp.campaigns.setContent(campaignId, {
      html: htmlContent,
    });

    console.log('Campaign content set');
    return response;
  } catch (error) {
    console.error('Error setting campaign content:', error);
  }
}
```

### Send a Campaign

```javascript
async function sendCampaign(campaignId) {
  try {
    const response = await mailchimp.campaigns.send(campaignId);
    console.log('Campaign sent successfully');
    return response;
  } catch (error) {
    console.error('Error sending campaign:', error);
  }
}
```

### Complete Campaign Workflow

```javascript
async function createAndSendCampaign(listId, subject, htmlContent) {
  try {
    // 1. Create campaign
    const campaign = await mailchimp.campaigns.create({
      type: 'regular',
      recipients: {
        list_id: listId,
      },
      settings: {
        subject_line: subject,
        title: subject,
        from_name: 'My Company',
        reply_to: 'hello@mycompany.com',
      },
    });

    console.log('Campaign created:', campaign.id);

    // 2. Set content
    await mailchimp.campaigns.setContent(campaign.id, {
      html: htmlContent,
    });

    console.log('Campaign content set');

    // 3. Send campaign
    await mailchimp.campaigns.send(campaign.id);

    console.log('Campaign sent successfully');

    return campaign;
  } catch (error) {
    console.error('Error in campaign workflow:', error);
  }
}
```

### Schedule a Campaign

```javascript
async function scheduleCampaign(campaignId, scheduleTime) {
  try {
    const response = await mailchimp.campaigns.schedule(campaignId, {
      schedule_time: scheduleTime, // ISO 8601 format: "2024-12-31T10:00:00Z"
    });

    console.log('Campaign scheduled for:', scheduleTime);
    return response;
  } catch (error) {
    console.error('Error scheduling campaign:', error);
  }
}
```

### Update Campaign Settings

```javascript
async function updateCampaign(campaignId, updates) {
  try {
    const response = await mailchimp.campaigns.update(campaignId, updates);
    console.log('Campaign updated');
    return response;
  } catch (error) {
    console.error('Error updating campaign:', error);
  }
}

// Example usage
updateCampaign('campaign123', {
  settings: {
    subject_line: 'Updated Subject Line',
    preview_text: 'Check out our latest updates!',
  },
});
```

### Delete a Campaign

```javascript
async function deleteCampaign(campaignId) {
  try {
    await mailchimp.campaigns.remove(campaignId);
    console.log('Campaign deleted');
  } catch (error) {
    console.error('Error deleting campaign:', error);
  }
}
```

## Templates Management

### Get All Templates

```javascript
async function getAllTemplates() {
  try {
    const response = await mailchimp.templates.list({
      count: 100,
    });

    console.log('Total templates:', response.total_items);

    response.templates.forEach(template => {
      console.log(`Template: ${template.name} (ID: ${template.id})`);
      console.log(`Type: ${template.type}, Category: ${template.category}`);
    });

    return response.templates;
  } catch (error) {
    console.error('Error fetching templates:', error);
  }
}
```

### Get Specific Template

```javascript
async function getTemplate(templateId) {
  try {
    const response = await mailchimp.templates.getTemplate(templateId);
    console.log('Template:', response.name);
    console.log('HTML:', response.html);
    return response;
  } catch (error) {
    console.error('Error fetching template:', error);
  }
}
```

### Create a Template

```javascript
async function createTemplate(name, htmlContent) {
  try {
    const response = await mailchimp.templates.create({
      name: name,
      html: htmlContent,
    });

    console.log('Template created:', response.id);
    return response;
  } catch (error) {
    console.error('Error creating template:', error);
  }
}
```

### Use Template in Campaign

```javascript
async function createCampaignFromTemplate(listId, templateId, subject) {
  try {
    const campaign = await mailchimp.campaigns.create({
      type: 'regular',
      recipients: {
        list_id: listId,
      },
      settings: {
        subject_line: subject,
        title: subject,
        from_name: 'My Company',
        reply_to: 'hello@mycompany.com',
        template_id: templateId,
      },
    });

    console.log('Campaign created from template:', campaign.id);
    return campaign;
  } catch (error) {
    console.error('Error creating campaign from template:', error);
  }
}
```

## Automation Workflows

### Get All Automations

```javascript
async function getAllAutomations() {
  try {
    const response = await mailchimp.automations.list({
      count: 100,
    });

    console.log('Total automations:', response.total_items);

    response.automations.forEach(automation => {
      console.log(`Automation: ${automation.settings.title}`);
      console.log(`Status: ${automation.status}, Recipients: ${automation.recipients.list_id}`);
    });

    return response.automations;
  } catch (error) {
    console.error('Error fetching automations:', error);
  }
}
```

### Get Specific Automation

```javascript
async function getAutomation(workflowId) {
  try {
    const response = await mailchimp.automations.get(workflowId);
    console.log('Automation:', response.settings.title);
    console.log('Status:', response.status);
    console.log('Emails:', response.emails.length);
    return response;
  } catch (error) {
    console.error('Error fetching automation:', error);
  }
}
```

### Pause an Automation

```javascript
async function pauseAutomation(workflowId) {
  try {
    await mailchimp.automations.pause(workflowId);
    console.log('Automation paused');
  } catch (error) {
    console.error('Error pausing automation:', error);
  }
}
```

### Start an Automation

```javascript
async function startAutomation(workflowId) {
  try {
    await mailchimp.automations.start(workflowId);
    console.log('Automation started');
  } catch (error) {
    console.error('Error starting automation:', error);
  }
}
```

### Add Subscriber to Automation Email Queue

```javascript
async function addSubscriberToAutomation(workflowId, workflowEmailId, email) {
  try {
    const response = await mailchimp.automations.addWorkflowEmailSubscriber(
      workflowId,
      workflowEmailId,
      {
        email_address: email,
      }
    );

    console.log(`Added ${email} to automation queue`);
    return response;
  } catch (error) {
    console.error('Error adding subscriber to automation:', error);
  }
}
```

## Reports and Analytics

### Get Campaign Reports

```javascript
async function getCampaignReport(campaignId) {
  try {
    const response = await mailchimp.reports.getCampaignReport(campaignId);

    console.log('Campaign:', response.campaign_title);
    console.log('Emails sent:', response.emails_sent);
    console.log('Opens:', response.opens.opens_total);
    console.log('Unique opens:', response.opens.unique_opens);
    console.log('Open rate:', response.opens.open_rate);
    console.log('Clicks:', response.clicks.clicks_total);
    console.log('Unique clicks:', response.clicks.unique_clicks);
    console.log('Click rate:', response.clicks.click_rate);
    console.log('Unsubscribes:', response.unsubscribed);

    return response;
  } catch (error) {
    console.error('Error fetching campaign report:', error);
  }
}
```

### Get All Campaign Reports

```javascript
async function getAllCampaignReports(count = 100) {
  try {
    const response = await mailchimp.reports.getAllCampaignReports({
      count: count,
    });

    console.log('Total reports:', response.total_items);

    response.reports.forEach(report => {
      console.log(`Campaign: ${report.campaign_title}`);
      console.log(`Open rate: ${report.opens.open_rate}%`);
      console.log(`Click rate: ${report.clicks.click_rate}%`);
    });

    return response.reports;
  } catch (error) {
    console.error('Error fetching reports:', error);
  }
}
```

### Get Email Activity for a Member

```javascript
const crypto = require('crypto');

async function getEmailActivity(campaignId, email) {
  try {
    const subscriberHash = crypto
      .createHash('md5')
      .update(email.toLowerCase())
      .digest('hex');

    const response = await mailchimp.reports.getEmailActivityForSubscriber(
      campaignId,
      subscriberHash
    );

    console.log(`Email activity for ${email}:`);
    response.activity.forEach(activity => {
      console.log(`${activity.action} at ${activity.timestamp}`);
    });

    return response;
  } catch (error) {
    console.error('Error fetching email activity:', error);
  }
}
```

### Get List Growth History

```javascript
async function getListGrowthHistory(listId) {
  try {
    const response = await mailchimp.lists.getListGrowthHistory(listId, {
      count: 100,
    });

    console.log('Growth history for list:');
    response.history.forEach(history => {
      console.log(`Month: ${history.month}`);
      console.log(`Subscribed: ${history.subscribed}, Unsubscribed: ${history.unsubscribed}`);
      console.log(`Existing: ${history.existing}`);
    });

    return response.history;
  } catch (error) {
    console.error('Error fetching growth history:', error);
  }
}
```

## E-commerce Integration

### Add a Store

```javascript
async function addStore(listId, storeId, storeName, currencyCode) {
  try {
    const response = await mailchimp.ecommerce.addStore({
      id: storeId,
      list_id: listId,
      name: storeName,
      currency_code: currencyCode,
    });

    console.log('Store created:', response.id);
    return response;
  } catch (error) {
    console.error('Error adding store:', error);
  }
}
```

### Get All Stores

```javascript
async function getAllStores() {
  try {
    const response = await mailchimp.ecommerce.getStores();

    console.log('Total stores:', response.total_items);

    response.stores.forEach(store => {
      console.log(`Store: ${store.name} (ID: ${store.id})`);
      console.log(`Currency: ${store.currency_code}`);
    });

    return response.stores;
  } catch (error) {
    console.error('Error fetching stores:', error);
  }
}
```

### Add a Customer

```javascript
async function addCustomer(storeId, customerId, email, firstName, lastName) {
  try {
    const response = await mailchimp.ecommerce.addStoreCustomer(storeId, {
      id: customerId,
      email_address: email,
      opt_in_status: true,
      first_name: firstName,
      last_name: lastName,
    });

    console.log('Customer added:', response.id);
    return response;
  } catch (error) {
    console.error('Error adding customer:', error);
  }
}
```

### Add a Product

```javascript
async function addProduct(storeId, productId, title, price) {
  try {
    const response = await mailchimp.ecommerce.addStoreProduct(storeId, {
      id: productId,
      title: title,
      variants: [
        {
          id: `${productId}-variant-1`,
          title: 'Default Variant',
          price: price,
        },
      ],
    });

    console.log('Product added:', response.id);
    return response;
  } catch (error) {
    console.error('Error adding product:', error);
  }
}
```

### Add an Order

```javascript
async function addOrder(storeId, orderId, customerId, lineItems, total) {
  try {
    const response = await mailchimp.ecommerce.addStoreOrder(storeId, {
      id: orderId,
      customer: {
        id: customerId,
      },
      lines: lineItems.map(item => ({
        id: item.id,
        product_id: item.productId,
        product_variant_id: item.variantId,
        quantity: item.quantity,
        price: item.price,
      })),
      currency_code: 'USD',
      order_total: total,
      processed_at_foreign: new Date().toISOString(),
    });

    console.log('Order added:', response.id);
    return response;
  } catch (error) {
    console.error('Error adding order:', error);
  }
}

// Example usage
addOrder('store123', 'order456', 'customer789', [
  {
    id: 'line1',
    productId: 'prod1',
    variantId: 'prod1-variant-1',
    quantity: 2,
    price: 29.99,
  },
], 59.98);
```

### Add a Cart

```javascript
async function addCart(storeId, cartId, customerId, lineItems) {
  try {
    const response = await mailchimp.ecommerce.addStoreCart(storeId, {
      id: cartId,
      customer: {
        id: customerId,
      },
      lines: lineItems.map(item => ({
        id: item.id,
        product_id: item.productId,
        product_variant_id: item.variantId,
        quantity: item.quantity,
        price: item.price,
      })),
      currency_code: 'USD',
    });

    console.log('Cart added:', response.id);
    return response;
  } catch (error) {
    console.error('Error adding cart:', error);
  }
}
```

## Error Handling

### Comprehensive Error Handling

```javascript
async function handleMailchimpRequest() {
  try {
    const response = await mailchimp.lists.getAllLists();
    return response;
  } catch (error) {
    console.error('Mailchimp API Error:', error);

    if (error.status) {
      console.error('Status Code:', error.status);
      console.error('Error Message:', error.response?.body?.title);
      console.error('Error Detail:', error.response?.body?.detail);

      switch (error.status) {
        case 401:
          console.error('Authentication failed. Check your API key.');
          break;
        case 403:
          console.error('Forbidden. Check your permissions.');
          break;
        case 404:
          console.error('Resource not found.');
          break;
        case 400:
          console.error('Bad request. Check your parameters.');
          if (error.response?.body?.errors) {
            console.error('Validation errors:', error.response.body.errors);
          }
          break;
        case 429:
          console.error('Rate limit exceeded. Wait before retrying.');
          break;
        case 500:
          console.error('Mailchimp server error. Try again later.');
          break;
        default:
          console.error('Unexpected error occurred.');
      }
    }

    throw error;
  }
}
```

### Retry Logic for Rate Limiting

```javascript
async function retryRequest(requestFn, maxRetries = 3, delay = 1000) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await requestFn();
    } catch (error) {
      if (error.status === 429 && i < maxRetries - 1) {
        console.log(`Rate limited. Retrying in ${delay}ms...`);
        await new Promise(resolve => setTimeout(resolve, delay));
        delay *= 2; // Exponential backoff
      } else {
        throw error;
      }
    }
  }
}

// Example usage
const result = await retryRequest(() => mailchimp.lists.getAllLists());
```

## Pagination

### Paginate Through Large Result Sets

```javascript
async function getAllListMembersPaginated(listId) {
  let offset = 0;
  const count = 100;
  const allMembers = [];

  while (true) {
    try {
      const response = await mailchimp.lists.getListMembersInfo(listId, {
        count: count,
        offset: offset,
      });

      allMembers.push(...response.members);

      console.log(`Fetched ${offset + response.members.length} of ${response.total_items} members`);

      if (response.members.length < count) {
        break; // No more results
      }

      offset += count;
    } catch (error) {
      console.error('Error during pagination:', error);
      break;
    }
  }

  return allMembers;
}
```

## TypeScript Support

The SDK includes TypeScript definitions:

```typescript
import mailchimp from '@mailchimp/mailchimp_marketing';

mailchimp.setConfig({
  apiKey: process.env.MAILCHIMP_API_KEY as string,
  server: process.env.MAILCHIMP_SERVER_PREFIX as string,
});

interface ListMember {
  email_address: string;
  status: 'subscribed' | 'unsubscribed' | 'cleaned' | 'pending';
  merge_fields: {
    FNAME: string;
    LNAME: string;
  };
}

async function addTypedMember(listId: string, member: ListMember) {
  const response = await mailchimp.lists.addListMember(listId, member);
  return response;
}
```

## Common Patterns

### Bulk Import Subscribers from CSV

```javascript
const fs = require('fs');
const csv = require('csv-parser');

async function importFromCSV(listId, csvFilePath) {
  const members = [];

  return new Promise((resolve, reject) => {
    fs.createReadStream(csvFilePath)
      .pipe(csv())
      .on('data', (row) => {
        members.push({
          email_address: row.email,
          status: 'subscribed',
          merge_fields: {
            FNAME: row.first_name,
            LNAME: row.last_name,
          },
        });
      })
      .on('end', async () => {
        try {
          // Batch in groups of 500 (Mailchimp limit)
          const batchSize = 500;
          for (let i = 0; i < members.length; i += batchSize) {
            const batch = members.slice(i, i + batchSize);

            const response = await mailchimp.lists.batchListMembers(listId, {
              members: batch,
              update_existing: true,
            });

            console.log(`Batch ${i / batchSize + 1}: ${response.new_members.length} added, ${response.updated_members.length} updated`);
          }

          resolve();
        } catch (error) {
          reject(error);
        }
      });
  });
}
```

### Send Welcome Email via Automation

```javascript
async function setupWelcomeAutomation(listId) {
  try {
    // Create a basic automation trigger when someone subscribes
    const automation = await mailchimp.automations.create({
      recipients: {
        list_id: listId,
      },
      trigger_settings: {
        workflow_type: 'welcomeSeries',
      },
      settings: {
        title: 'Welcome Email Series',
        from_name: 'My Company',
        reply_to: 'hello@mycompany.com',
      },
    });

    console.log('Welcome automation created:', automation.id);
    return automation;
  } catch (error) {
    console.error('Error setting up welcome automation:', error);
  }
}
```

### Segment Users by Engagement

```javascript
async function createEngagementSegment(listId, segmentName) {
  try {
    const response = await mailchimp.lists.createSegment(listId, {
      name: segmentName,
      options: {
        match: 'all',
        conditions: [
          {
            condition_type: 'EmailClient',
            field: 'email_client',
            op: 'is',
            value: 'Gmail',
          },
        ],
      },
    });

    console.log('Engagement segment created:', response.id);
    return response;
  } catch (error) {
    console.error('Error creating segment:', error);
  }
}
```

## Webhooks

### Process Webhook Events

```javascript
const express = require('express');
const crypto = require('crypto');

const app = express();
app.use(express.json());

// Verify webhook signature
function verifyWebhook(secret, body, signature) {
  const hash = crypto
    .createHmac('sha1', secret)
    .update(JSON.stringify(body))
    .digest('hex');

  return hash === signature;
}

app.post('/mailchimp-webhook', (req, res) => {
  const signature = req.headers['x-mailchimp-signature'];
  const webhookSecret = process.env.MAILCHIMP_WEBHOOK_SECRET;

  if (!verifyWebhook(webhookSecret, req.body, signature)) {
    return res.status(401).send('Invalid signature');
  }

  const { type, data } = req.body;

  switch (type) {
    case 'subscribe':
      console.log('New subscriber:', data.email);
      break;
    case 'unsubscribe':
      console.log('Unsubscribed:', data.email);
      break;
    case 'profile':
      console.log('Profile updated:', data.email);
      break;
    case 'campaign':
      console.log('Campaign event:', data);
      break;
    default:
      console.log('Unknown webhook type:', type);
  }

  res.status(200).send('OK');
});

app.listen(3000, () => console.log('Webhook server running on port 3000'));
```

## Notes

The Mailchimp Marketing Node.js SDK is auto-generated from the OpenAPI specification and provides comprehensive access to all Mailchimp Marketing API endpoints. Always use environment variables for API keys and server prefixes. The library supports both Basic Auth (API key) and OAuth2 authentication methods. Rate limits apply: 10 simultaneous connections per account and throttling based on your plan. Use batch operations when adding multiple members to optimize API usage.
