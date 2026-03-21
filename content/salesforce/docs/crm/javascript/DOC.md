---
name: crm
description: "Salesforce JavaScript SDK (JSforce) coding guidelines for Salesforce API interactions"
metadata:
  languages: "javascript"
  versions: "3.10.8"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "salesforce,crm,soql,enterprise,api"
---

# Salesforce JavaScript SDK (JSforce) Coding Guidelines

You are a Salesforce API coding expert. Help me with writing code using the Salesforce API calling the official libraries and SDKs.

You can find the official SDK documentation and code samples here:
https://jsforce.github.io/

## Golden Rule: Use the Correct and Current SDK

Always use JSforce to interact with Salesforce APIs. JSforce is the standard JavaScript library for all Salesforce API interactions, supporting both Node.js and browser environments.

- **Library Name:** JSforce
- **NPM Package:** `jsforce`
- **Alternative Package:** `@jsforce/jsforce-node` (Node.js only, smaller bundle)
- **Legacy Libraries**: Do not use deprecated or unofficial Salesforce JavaScript packages

**Installation:**

- **Correct:** `npm install jsforce`
- **Node.js only:** `npm install @jsforce/jsforce-node`

**APIs and Usage:**

- **Correct:** `const jsforce = require('jsforce')`
- **Correct:** `const conn = new jsforce.Connection()`
- **Correct:** `await conn.query(...)`
- **Correct:** `await conn.sobject('Account').create(...)`
- **Incorrect:** Using unofficial Salesforce client libraries
- **Incorrect:** Direct REST API calls without SDK

## Installation

Install JSforce via npm:

```bash
npm install jsforce
```

For Node.js-only projects (smaller bundle):

```bash
npm install @jsforce/jsforce-node
```

For browser usage via CDN:

```html
<script src="https://cdn.jsdelivr.net/npm/jsforce@3.2.2/dist/jsforce.min.js"></script>
```

## Environment Variables

Set up your Salesforce credentials using environment variables:

```bash
export SALESFORCE_USERNAME="user@example.org"
export SALESFORCE_PASSWORD="your_password"
export SALESFORCE_SECURITY_TOKEN="your_security_token"
export SALESFORCE_LOGIN_URL="https://login.salesforce.com" # or https://test.salesforce.com for sandbox
```

Or create a `.env` file:

```
SALESFORCE_USERNAME=user@example.org
SALESFORCE_PASSWORD=your_password
SALESFORCE_SECURITY_TOKEN=your_security_token
SALESFORCE_LOGIN_URL=https://login.salesforce.com
```

## Initialization

JSforce requires creating a `Connection` instance for all API calls.

### Basic Connection

```javascript
const jsforce = require('jsforce');

// Create connection
const conn = new jsforce.Connection();
```

### Connection with Login URL

```javascript
const jsforce = require('jsforce');

// Connect to sandbox
const conn = new jsforce.Connection({
  loginUrl: 'https://test.salesforce.com'
});
```

### Connection with Session ID (OAuth)

```javascript
const jsforce = require('jsforce');

const conn = new jsforce.Connection({
  instanceUrl: 'https://na1.salesforce.com',
  accessToken: 'your_access_token'
});
```

### Connection with Session ID and Server URL

```javascript
const jsforce = require('jsforce');

const conn = new jsforce.Connection({
  serverUrl: 'https://na1.salesforce.com',
  sessionId: 'your_session_id'
});
```

## Authentication

### Username/Password Login

```javascript
const jsforce = require('jsforce');
const conn = new jsforce.Connection();

async function login() {
  try {
    const userInfo = await conn.login(
      process.env.SALESFORCE_USERNAME,
      process.env.SALESFORCE_PASSWORD + process.env.SALESFORCE_SECURITY_TOKEN
    );
    console.log('User ID:', userInfo.id);
    console.log('Org ID:', userInfo.organizationId);
    console.log('Access Token:', conn.accessToken);
    console.log('Instance URL:', conn.instanceUrl);
  } catch (err) {
    console.error('Login error:', err);
  }
}

login();
```

### OAuth 2.0 Username-Password Flow

```javascript
const jsforce = require('jsforce');

async function oauth2Login() {
  const conn = new jsforce.Connection({
    oauth2: {
      loginUrl: 'https://login.salesforce.com',
      clientId: 'your_client_id',
      clientSecret: 'your_client_secret',
      redirectUri: 'http://localhost:3000/oauth/callback'
    }
  });

  try {
    const userInfo = await conn.login(
      process.env.SALESFORCE_USERNAME,
      process.env.SALESFORCE_PASSWORD
    );
    console.log('Access Token:', conn.accessToken);
    console.log('Refresh Token:', conn.refreshToken);
  } catch (err) {
    console.error('OAuth login error:', err);
  }
}

oauth2Login();
```

### OAuth 2.0 Authorization Code Flow

```javascript
const jsforce = require('jsforce');
const express = require('express');

const oauth2 = new jsforce.OAuth2({
  loginUrl: 'https://login.salesforce.com',
  clientId: 'your_client_id',
  clientSecret: 'your_client_secret',
  redirectUri: 'http://localhost:3000/oauth/callback'
});

const app = express();

app.get('/oauth', (req, res) => {
  res.redirect(oauth2.getAuthorizationUrl({ scope: 'api id web' }));
});

app.get('/oauth/callback', async (req, res) => {
  const conn = new jsforce.Connection({ oauth2 });
  const code = req.query.code;

  try {
    const userInfo = await conn.authorize(code);
    console.log('Access Token:', conn.accessToken);
    console.log('Refresh Token:', conn.refreshToken);
    res.send('OAuth authentication successful!');
  } catch (err) {
    console.error('OAuth error:', err);
    res.status(500).send('OAuth error');
  }
});

app.listen(3000);
```

### OAuth 2.0 Client Credentials Flow

```javascript
const jsforce = require('jsforce');

async function clientCredentialsFlow() {
  const conn = new jsforce.Connection({
    oauth2: {
      loginUrl: 'https://login.salesforce.com',
      clientId: 'your_client_id',
      clientSecret: 'your_client_secret'
    }
  });

  try {
    await conn.oauth2.authenticate({
      grant_type: 'client_credentials'
    });
    console.log('Access Token:', conn.accessToken);
  } catch (err) {
    console.error('Client credentials error:', err);
  }
}

clientCredentialsFlow();
```

### JWT Bearer Flow

```javascript
const jsforce = require('jsforce');
const fs = require('fs');

const privateKey = fs.readFileSync('private.key', 'utf8');

async function jwtBearerFlow() {
  const conn = new jsforce.Connection({
    oauth2: {
      loginUrl: 'https://login.salesforce.com',
      clientId: 'your_client_id',
      privateKey: privateKey
    }
  });

  try {
    await conn.oauth2.authenticate({
      grant_type: 'urn:ietf:params:oauth:grant-type:jwt-bearer',
      assertion: conn.oauth2.createJwtBearerToken({
        iss: 'your_client_id',
        sub: 'user@example.org',
        aud: 'https://login.salesforce.com'
      })
    });
    console.log('Access Token:', conn.accessToken);
  } catch (err) {
    console.error('JWT Bearer flow error:', err);
  }
}

jwtBearerFlow();
```

### Browser OAuth Initialization

```javascript
jsforce.browser.init({
  clientId: 'your_oauth2_client_id',
  redirectUri: 'https://yourapp.example.com/oauth/callback'
});

jsforce.browser.login();

jsforce.browser.on('connect', function(conn) {
  console.log('Connected to Salesforce');
  console.log('Access Token:', conn.accessToken);
});
```

## SOQL Queries

### Basic Query

```javascript
const jsforce = require('jsforce');
const conn = new jsforce.Connection();

async function basicQuery() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.query('SELECT Id, Name FROM Account LIMIT 10');

  console.log('Total records:', result.totalSize);
  console.log('Records fetched:', result.records.length);

  result.records.forEach(record => {
    console.log('Account:', record.Name);
  });
}

basicQuery();
```

### Query with WHERE Clause

```javascript
async function queryWithFilter() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.query(
    "SELECT Id, Name, Industry FROM Account WHERE Industry = 'Technology'"
  );

  console.log('Found', result.totalSize, 'Technology accounts');
}
```

### Query with Relationships

```javascript
async function queryRelationships() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  // Parent to child relationship
  const result = await conn.query(
    'SELECT Id, Name, (SELECT Id, Name FROM Contacts) FROM Account LIMIT 5'
  );

  result.records.forEach(account => {
    console.log('Account:', account.Name);
    if (account.Contacts) {
      account.Contacts.records.forEach(contact => {
        console.log('  Contact:', contact.Name);
      });
    }
  });
}
```

### Query All Records (Including Deleted)

```javascript
async function queryAll() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.queryAll(
    'SELECT Id, Name, IsDeleted FROM Account WHERE IsDeleted = true'
  );

  console.log('Deleted accounts:', result.totalSize);
}
```

### Query with Pagination

```javascript
async function queryWithPagination() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  let result = await conn.query('SELECT Id, Name FROM Account');
  let records = result.records;

  while (!result.done) {
    result = await conn.queryMore(result.nextRecordsUrl);
    records = records.concat(result.records);
  }

  console.log('Total records retrieved:', records.length);
}
```

### Query with Method Chaining

```javascript
async function methodChainQuery() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const records = await conn.sobject('Account')
    .select('Id, Name, Industry')
    .where({ Industry: 'Technology' })
    .limit(10)
    .execute();

  console.log('Records:', records);
}
```

### Event-Driven Query

```javascript
async function eventDrivenQuery() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  conn.query('SELECT Id, Name FROM Account')
    .on('record', record => {
      console.log('Account:', record.Name);
    })
    .on('end', () => {
      console.log('Query completed');
    })
    .on('error', err => {
      console.error('Query error:', err);
    })
    .run();
}
```

## SOSL Search

### Basic Search

```javascript
async function basicSearch() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.search(
    'FIND {United*} IN NAME FIELDS RETURNING Account(Id, Name), Contact(Id, Name)'
  );

  console.log('Search results:', result.searchRecords);
}
```

## CRUD Operations

### Create Single Record

```javascript
async function createRecord() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.sobject('Account').create({
    Name: 'New Account',
    Industry: 'Technology',
    BillingCity: 'San Francisco'
  });

  console.log('Created account ID:', result.id);
  console.log('Success:', result.success);
}
```

### Create Multiple Records

```javascript
async function createMultipleRecords() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const results = await conn.sobject('Account').create([
    { Name: 'Account 1', Industry: 'Technology' },
    { Name: 'Account 2', Industry: 'Finance' },
    { Name: 'Account 3', Industry: 'Healthcare' }
  ]);

  results.forEach(result => {
    if (result.success) {
      console.log('Created account ID:', result.id);
    } else {
      console.error('Error:', result.errors);
    }
  });
}
```

### Retrieve Single Record

```javascript
async function retrieveRecord() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const account = await conn.sobject('Account').retrieve('001XXXXXXXXXXXXXXX');

  console.log('Account Name:', account.Name);
  console.log('Industry:', account.Industry);
}
```

### Retrieve Multiple Records

```javascript
async function retrieveMultipleRecords() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const accounts = await conn.sobject('Account').retrieve([
    '001XXXXXXXXXXXXXXX',
    '001YYYYYYYYYYYYYYY'
  ]);

  accounts.forEach(account => {
    console.log('Account:', account.Name);
  });
}
```

### Update Single Record

```javascript
async function updateRecord() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.sobject('Account').update({
    Id: '001XXXXXXXXXXXXXXX',
    Name: 'Updated Account Name',
    Industry: 'Media'
  });

  console.log('Updated:', result.success);
}
```

### Update Multiple Records

```javascript
async function updateMultipleRecords() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const results = await conn.sobject('Account').update([
    { Id: '001XXXXXXXXXXXXXXX', Name: 'Updated Name 1' },
    { Id: '001YYYYYYYYYYYYYYY', Name: 'Updated Name 2' }
  ]);

  results.forEach(result => {
    if (result.success) {
      console.log('Updated account ID:', result.id);
    } else {
      console.error('Error:', result.errors);
    }
  });
}
```

### Upsert Records

```javascript
async function upsertRecords() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const results = await conn.sobject('Account').upsert(
    [
      { ExternalId__c: 'EXT001', Name: 'Account 1', Industry: 'Technology' },
      { ExternalId__c: 'EXT002', Name: 'Account 2', Industry: 'Finance' }
    ],
    'ExternalId__c'
  );

  results.forEach(result => {
    if (result.success) {
      console.log('Upserted account ID:', result.id);
      console.log('Created:', result.created);
    }
  });
}
```

### Delete Single Record

```javascript
async function deleteRecord() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.sobject('Account').destroy('001XXXXXXXXXXXXXXX');

  console.log('Deleted:', result.success);
}
```

### Delete Multiple Records

```javascript
async function deleteMultipleRecords() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const results = await conn.sobject('Account').destroy([
    '001XXXXXXXXXXXXXXX',
    '001YYYYYYYYYYYYYYY'
  ]);

  results.forEach(result => {
    if (result.success) {
      console.log('Deleted account ID:', result.id);
    } else {
      console.error('Error:', result.errors);
    }
  });
}
```

## Bulk API

### Bulk Query

```javascript
async function bulkQuery() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const records = [];

  conn.bulk.query('SELECT Id, Name FROM Account')
    .on('record', record => {
      records.push(record);
    })
    .on('end', () => {
      console.log('Total records:', records.length);
    })
    .on('error', err => {
      console.error('Bulk query error:', err);
    });
}
```

### Bulk Insert

```javascript
async function bulkInsert() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const accounts = [];
  for (let i = 0; i < 1000; i++) {
    accounts.push({
      Name: `Bulk Account ${i}`,
      Industry: 'Technology'
    });
  }

  conn.bulk.load('Account', 'insert', accounts, (err, results) => {
    if (err) {
      console.error('Bulk insert error:', err);
      return;
    }

    results.forEach(result => {
      if (result.success) {
        console.log('Created ID:', result.id);
      } else {
        console.error('Error:', result.errors);
      }
    });
  });
}
```

### Bulk Update

```javascript
async function bulkUpdate() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const updates = [
    { Id: '001XXXXXXXXXXXXXXX', Industry: 'Finance' },
    { Id: '001YYYYYYYYYYYYYYY', Industry: 'Healthcare' }
  ];

  conn.bulk.load('Account', 'update', updates, (err, results) => {
    if (err) {
      console.error('Bulk update error:', err);
      return;
    }

    console.log('Updated records:', results.length);
  });
}
```

### Bulk Delete

```javascript
async function bulkDelete() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const idsToDelete = [
    { Id: '001XXXXXXXXXXXXXXX' },
    { Id: '001YYYYYYYYYYYYYYY' }
  ];

  conn.bulk.load('Account', 'delete', idsToDelete, (err, results) => {
    if (err) {
      console.error('Bulk delete error:', err);
      return;
    }

    console.log('Deleted records:', results.length);
  });
}
```

### Bulk 2.0 Query

```javascript
async function bulk2Query() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const recordStream = await conn.bulk2.query('SELECT Id, Name FROM Account');

  recordStream.on('record', record => {
    console.log('Record:', record);
  });

  recordStream.on('end', () => {
    console.log('Bulk 2.0 query completed');
  });

  recordStream.on('error', err => {
    console.error('Bulk 2.0 query error:', err);
  });
}
```

## SObject Metadata

### Describe SObject

```javascript
async function describeSObject() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const metadata = await conn.sobject('Account').describe();

  console.log('Label:', metadata.label);
  console.log('Updateable:', metadata.updateable);
  console.log('Fields:', metadata.fields.length);

  metadata.fields.forEach(field => {
    console.log(`  ${field.name} (${field.type})`);
  });
}
```

### Describe Global

```javascript
async function describeGlobal() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.describeGlobal();

  console.log('Available SObjects:');
  result.sobjects.forEach(sobject => {
    console.log(`  ${sobject.name} - ${sobject.label}`);
  });
}
```

### Describe Multiple SObjects

```javascript
async function describeMultiple() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.describe(['Account', 'Contact', 'Opportunity']);

  result.sobjects.forEach(sobject => {
    console.log('SObject:', sobject.name);
    console.log('Fields:', sobject.fields.length);
  });
}
```

## Metadata API

### List Metadata Types

```javascript
async function listMetadataTypes() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.metadata.describe();

  console.log('API Version:', result.organizationNamespace);
  result.metadataObjects.forEach(obj => {
    console.log('Metadata Type:', obj.xmlName);
  });
}
```

### List Metadata

```javascript
async function listMetadata() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.metadata.list([
    { type: 'CustomObject' },
    { type: 'ApexClass' }
  ]);

  result.forEach(item => {
    console.log('Full Name:', item.fullName);
    console.log('Type:', item.type);
  });
}
```

### Read Metadata

```javascript
async function readMetadata() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.metadata.read('CustomObject', ['Account', 'Contact']);

  result.forEach(metadata => {
    console.log('Object:', metadata.fullName);
    console.log('Label:', metadata.label);
  });
}
```

### Create Metadata

```javascript
async function createMetadata() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const customField = {
    fullName: 'Account.CustomField__c',
    label: 'Custom Field',
    type: 'Text',
    length: 100
  };

  const result = await conn.metadata.create('CustomField', [customField]);

  console.log('Created:', result[0].success);
}
```

### Update Metadata

```javascript
async function updateMetadata() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const customField = {
    fullName: 'Account.CustomField__c',
    label: 'Updated Custom Field',
    type: 'Text',
    length: 255
  };

  const result = await conn.metadata.update('CustomField', [customField]);

  console.log('Updated:', result[0].success);
}
```

### Delete Metadata

```javascript
async function deleteMetadata() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.metadata.delete('CustomField', [
    'Account.CustomField__c'
  ]);

  console.log('Deleted:', result[0].success);
}
```

## Tooling API

### Query with Tooling API

```javascript
async function toolingQuery() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.tooling.query(
    'SELECT Id, Name FROM ApexClass WHERE Name LIKE \'Test%\''
  );

  result.records.forEach(record => {
    console.log('Apex Class:', record.Name);
  });
}
```

### Execute Anonymous Apex

```javascript
async function executeAnonymous() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const apexCode = `
    System.debug('Hello from anonymous Apex!');
    Account acc = new Account(Name='Test Account');
    insert acc;
  `;

  const result = await conn.tooling.executeAnonymous(apexCode);

  console.log('Success:', result.success);
  console.log('Compiled:', result.compiled);
  console.log('Logs:', result.logs);
}
```

### Create Apex Class

```javascript
async function createApexClass() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.tooling.sobject('ApexClass').create({
    Name: 'MyApexClass',
    Body: 'public class MyApexClass { public static void hello() { System.debug(\'Hello\'); } }'
  });

  console.log('Created Apex Class ID:', result.id);
}
```

## Analytics API

### List Reports

```javascript
async function listReports() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const reports = await conn.analytics.reports();

  reports.forEach(report => {
    console.log('Report:', report.name);
    console.log('ID:', report.id);
  });
}
```

### Describe Report

```javascript
async function describeReport() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const reportId = '00O1234567890ABCD';
  const metadata = await conn.analytics.report(reportId).describe();

  console.log('Report Metadata:', metadata.reportMetadata);
  console.log('Report Type:', metadata.reportTypeMetadata);
}
```

### Execute Report

```javascript
async function executeReport() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const reportId = '00O1234567890ABCD';
  const result = await conn.analytics.report(reportId).execute();

  console.log('Report Results:', result.factMap);
}
```

### List Dashboards

```javascript
async function listDashboards() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const dashboards = await conn.analytics.dashboards();

  dashboards.forEach(dashboard => {
    console.log('Dashboard:', dashboard.name);
    console.log('ID:', dashboard.id);
  });
}
```

## Chatter API

### Get User Info

```javascript
async function getUserInfo() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const user = await conn.chatter.resource('/users/me').retrieve();

  console.log('Username:', user.username);
  console.log('Email:', user.email);
  console.log('Display Name:', user.displayName);
}
```

### Post to Feed

```javascript
async function postToFeed() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.chatter.resource('/feed-elements').create({
    body: {
      messageSegments: [
        { type: 'Text', text: 'Hello from JSforce!' }
      ]
    },
    feedElementType: 'FeedItem',
    subjectId: 'me'
  });

  console.log('Posted feed item ID:', result.id);
}
```

### Get Feed Items

```javascript
async function getFeedItems() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.chatter.resource('/feeds/news/me/feed-elements').retrieve();

  result.elements.forEach(element => {
    console.log('Feed Item:', element.body.text);
  });
}
```

## Streaming API

### Subscribe to PushTopic

```javascript
async function subscribeToPushTopic() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  conn.streaming.topic('AllAccounts').subscribe((message) => {
    console.log('Event received:', message);
    console.log('Account ID:', message.sobject.Id);
    console.log('Account Name:', message.sobject.Name);
  });
}
```

### Create PushTopic

```javascript
async function createPushTopic() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.sobject('PushTopic').create({
    Name: 'AllAccounts',
    Query: 'SELECT Id, Name FROM Account',
    ApiVersion: 58.0,
    NotifyForOperationCreate: true,
    NotifyForOperationUpdate: true,
    NotifyForOperationDelete: true,
    NotifyForFields: 'All'
  });

  console.log('PushTopic created:', result.id);
}
```

### Platform Event Subscription

```javascript
async function subscribeToEvent() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  conn.streaming.channel('/event/Custom_Event__e').subscribe((message) => {
    console.log('Platform Event received:', message);
  });
}
```

### Change Data Capture

```javascript
async function subscribeToChangeDataCapture() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  conn.streaming.channel('/data/AccountChangeEvent').subscribe((message) => {
    console.log('Change Event:', message);
    console.log('Change Type:', message.payload.ChangeEventHeader.changeType);
  });
}
```

## Apex REST

### Call Custom Apex REST Endpoint

```javascript
async function callApexRest() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.apex.get('/MyApexRestService');

  console.log('Response:', result);
}
```

### POST to Apex REST

```javascript
async function postToApexRest() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.apex.post('/MyApexRestService', {
    name: 'Test',
    value: 123
  });

  console.log('Response:', result);
}
```

### PUT to Apex REST

```javascript
async function putToApexRest() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.apex.put('/MyApexRestService/001XXXXXXXXXXXXXXX', {
    name: 'Updated Name'
  });

  console.log('Response:', result);
}
```

### DELETE to Apex REST

```javascript
async function deleteToApexRest() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const result = await conn.apex.del('/MyApexRestService/001XXXXXXXXXXXXXXX');

  console.log('Response:', result);
}
```

## Error Handling

### Basic Error Handling

```javascript
const jsforce = require('jsforce');
const conn = new jsforce.Connection();

async function handleErrors() {
  try {
    await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

    const result = await conn.sobject('Account').create({
      Name: 'Test Account'
    });

    console.log('Success:', result.success);
  } catch (err) {
    console.error('Error Name:', err.name);
    console.error('Error Message:', err.message);
    console.error('Error Code:', err.errorCode);
  }
}
```

### CRUD Error Handling

```javascript
async function handleCrudErrors() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const results = await conn.sobject('Account').create([
    { Name: 'Account 1' },
    { Name: '' } // This will fail validation
  ]);

  results.forEach((result, index) => {
    if (result.success) {
      console.log(`Record ${index} created:`, result.id);
    } else {
      console.error(`Record ${index} failed:`, result.errors);
    }
  });
}
```

## Limits and Identity

### Get API Limits

```javascript
async function getApiLimits() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const limits = await conn.limits();

  console.log('Daily API Requests:', limits.DailyApiRequests);
  console.log('Max:', limits.DailyApiRequests.Max);
  console.log('Remaining:', limits.DailyApiRequests.Remaining);
}
```

### Get Identity Information

```javascript
async function getIdentity() {
  await conn.login(process.env.SALESFORCE_USERNAME, process.env.SALESFORCE_PASSWORD);

  const identity = await conn.identity();

  console.log('User ID:', identity.user_id);
  console.log('Organization ID:', identity.organization_id);
  console.log('Username:', identity.username);
  console.log('Display Name:', identity.display_name);
}
```

## Useful Links

- Documentation: https://jsforce.github.io/
- Getting Started: https://jsforce.github.io/start/
- API Reference: https://jsforce.github.io/document/
- GitHub: https://github.com/jsforce/jsforce
- Salesforce Developer Docs: https://developer.salesforce.com/
- Salesforce REST API Guide: https://developer.salesforce.com/docs/atlas.en-us.api_rest.meta/api_rest/
