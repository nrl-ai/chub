---
name: database
description: "Airtable JavaScript SDK (airtable.js) — use the official airtable npm package for Airtable API operations"
metadata:
  languages: "javascript"
  versions: "0.12.2"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "airtable,database,low-code,spreadsheet,api"
---

# Airtable JavaScript SDK (airtable.js) - Version 0.12.2

## Golden Rule

**ALWAYS use the official `airtable` npm package (version 0.12.2 or later)**

```bash
npm install airtable
```

**DO NOT use:**
- Deprecated or unofficial packages like `airtable-node`, `airtable-plus`, or other third-party wrappers
- The old API key authentication method (deprecated as of February 1, 2024)
- The `@airtable/blocks` package unless building Airtable extensions

**ALWAYS use Personal Access Tokens (PATs) for authentication**, not the deprecated API keys.

## Installation

```bash
npm install airtable
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
   - `schema.bases:read` - to read base structure (optional)
4. Select base access level (specific bases or all workspace bases)
5. Copy the token immediately (shown only once)

### Basic Configuration

**Option 1: Global Configuration**

```javascript
const Airtable = require('airtable');

Airtable.configure({
  apiKey: 'your_personal_access_token'
});

const base = Airtable.base('appYourBaseId');
```

**Option 2: Environment Variable**

```javascript
const Airtable = require('airtable');

// Automatically reads from AIRTABLE_API_KEY environment variable
const base = Airtable.base('appYourBaseId');
```

**Option 3: Per-Instance Configuration**

```javascript
const Airtable = require('airtable');

const airtable = new Airtable({
  apiKey: process.env.AIRTABLE_API_KEY
});

const base = airtable.base('appYourBaseId');
```

### Advanced Configuration Options

```javascript
const Airtable = require('airtable');

Airtable.configure({
  apiKey: process.env.AIRTABLE_API_KEY,
  endpointUrl: 'https://api.airtable.com', // Custom endpoint (optional)
  requestTimeout: 300000 // Request timeout in milliseconds (default: 300000)
});
```

### Finding Your Base and Table IDs

1. Go to https://airtable.com/api
2. Select your base
3. Base ID format: `appXXXXXXXXXXXXXX`
4. Table names are case-sensitive strings (e.g., 'Tasks', 'Contacts')

## Core API Surfaces

### Reading Records

#### Get Single Record by ID

```javascript
const table = base('Tasks');

// Callback style
table.find('recXXXXXXXXXXXXXX', function(err, record) {
  if (err) {
    console.error(err);
    return;
  }
  console.log('Retrieved', record.get('Name'));
  console.log('Record ID:', record.id);
  console.log('Created time:', record._rawJson.createdTime);
});

// Promise style
table.find('recXXXXXXXXXXXXXX')
  .then(record => {
    console.log('Retrieved', record.get('Name'));
  })
  .catch(err => {
    console.error(err);
  });

// Async/await style
async function getRecord() {
  try {
    const record = await table.find('recXXXXXXXXXXXXXX');
    console.log('Name:', record.get('Name'));
    console.log('Status:', record.get('Status'));

    // Access all fields
    const fields = record.fields;
    console.log('All fields:', fields);
  } catch (err) {
    console.error(err);
  }
}
```

#### Get All Records (≤100 records)

Use `firstPage()` when you know your table has 100 or fewer records:

```javascript
const table = base('Tasks');

// Callback style
table.select({
  view: 'Grid view'
}).firstPage(function(err, records) {
  if (err) {
    console.error(err);
    return;
  }

  records.forEach(function(record) {
    console.log('Retrieved', record.get('Name'));
  });
});

// Promise style
table.select().firstPage()
  .then(records => {
    records.forEach(record => {
      console.log('Name:', record.get('Name'));
      console.log('Status:', record.get('Status'));
    });
  })
  .catch(err => {
    console.error(err);
  });

// Async/await style
async function getAllRecords() {
  try {
    const records = await table.select().firstPage();

    records.forEach(record => {
      console.log('ID:', record.id);
      console.log('Name:', record.get('Name'));
    });
  } catch (err) {
    console.error(err);
  }
}
```

#### Paginated Reading (>100 records)

Use `eachPage()` for tables with more than 100 records:

```javascript
const table = base('Tasks');

let allRecords = [];

table.select({
  view: 'Grid view'
}).eachPage(
  function page(records, fetchNextPage) {
    // Called for each page of records
    records.forEach(function(record) {
      console.log('Retrieved', record.get('Name'));
      allRecords.push(record);
    });

    // Fetch the next page of records
    fetchNextPage();
  },
  function done(err) {
    // Called when all pages have been retrieved
    if (err) {
      console.error(err);
      return;
    }

    console.log(`Total records: ${allRecords.length}`);
  }
);
```

#### Using all() Method with Async/Await

The `all()` method retrieves all pages synchronously:

```javascript
const table = base('Tasks');

async function getAllRecordsSimple() {
  try {
    const records = await table.select({
      view: 'Grid view'
    }).all();

    console.log(`Total records: ${records.length}`);

    records.forEach(record => {
      console.log('Name:', record.get('Name'));
    });

    return records;
  } catch (err) {
    console.error(err);
  }
}
```

**Important:** Only use `all()` when you expect a manageable number of records. For very large tables, use `eachPage()` to avoid memory issues.

### Filtering Records

#### Using filterByFormula

```javascript
const table = base('Tasks');

// Simple filter
const records = await table.select({
  filterByFormula: "{Status} = 'Active'"
}).all();

// Multiple conditions with AND
const activeHighPriority = await table.select({
  filterByFormula: "AND({Status} = 'Active', {Priority} = 'High')"
}).all();

// Multiple conditions with OR
const urgentOrBlocked = await table.select({
  filterByFormula: "OR({Status} = 'Urgent', {Status} = 'Blocked')"
}).all();

// Not empty check
const withTitles = await table.select({
  filterByFormula: "NOT({Title} = '')"
}).all();

// Greater than comparison
const recentTasks = await table.select({
  filterByFormula: "{Created} > '2025-01-01'"
}).all();

// String matching with variable
const email = 'user@example.com';
const userRecords = await table.select({
  filterByFormula: `{Email} = "${email}"`
}).all();

// Complex formula
const complexFilter = await table.select({
  filterByFormula: `AND(
    {Status} = 'In Progress',
    {Priority} = 'High',
    {Assignee} != '',
    {DueDate} <= TODAY()
  )`
}).all();
```

### Sorting Records

```javascript
const table = base('Tasks');

// Sort by single field (ascending)
const sortedAsc = await table.select({
  sort: [{field: 'Name', direction: 'asc'}]
}).all();

// Sort by single field (descending)
const sortedDesc = await table.select({
  sort: [{field: 'Created', direction: 'desc'}]
}).all();

// Sort by multiple fields
const multiSort = await table.select({
  sort: [
    {field: 'Priority', direction: 'desc'},
    {field: 'DueDate', direction: 'asc'},
    {field: 'Name', direction: 'asc'}
  ]
}).all();
```

### Limiting and Pagination Parameters

```javascript
const table = base('Tasks');

// Limit total number of records returned
const limitedRecords = await table.select({
  maxRecords: 50
}).all();

// Set page size (max 100)
const customPageSize = await table.select({
  pageSize: 50
}).firstPage();

// Combine all parameters
const complexQuery = await table.select({
  view: 'Active Tasks',
  maxRecords: 100,
  pageSize: 50,
  filterByFormula: "NOT({Name} = '')",
  sort: [{field: 'Priority', direction: 'desc'}]
}).all();
```

### Selecting Specific Fields

```javascript
const table = base('Tasks');

// Only retrieve specific fields
const records = await table.select({
  fields: ['Name', 'Status', 'Assignee']
}).all();

records.forEach(record => {
  console.log('Name:', record.get('Name'));
  console.log('Status:', record.get('Status'));
  // Other fields will be undefined
});
```

### Using Views

```javascript
const table = base('Tasks');

// Select from a specific view
const viewRecords = await table.select({
  view: 'Active Tasks'
}).all();

// Combine view with other parameters
const filteredView = await table.select({
  view: 'Active Tasks',
  filterByFormula: "{Priority} = 'High'",
  sort: [{field: 'DueDate', direction: 'asc'}]
}).all();
```

### Creating Records

#### Create Single Record

```javascript
const table = base('Tasks');

// Callback style
table.create({
  'Name': 'New Task',
  'Status': 'To Do',
  'Priority': 'Medium'
}, function(err, record) {
  if (err) {
    console.error(err);
    return;
  }
  console.log('Created record:', record.id);
});

// Promise style
table.create({
  'Name': 'New Task',
  'Status': 'To Do'
}).then(record => {
  console.log('Created record:', record.id);
}).catch(err => {
  console.error(err);
});

// Async/await style
async function createTask() {
  try {
    const record = await table.create({
      'Name': 'New Task',
      'Status': 'To Do',
      'Priority': 'High',
      'Notes': 'This is a new task'
    });

    console.log('Created record:', record.id);
    console.log('Name:', record.get('Name'));

    return record;
  } catch (err) {
    console.error(err);
  }
}
```

#### Create with Typecast

```javascript
const table = base('Tasks');

// Typecast converts strings to appropriate types
const record = await table.create({
  'Name': 'Task with Date',
  'DueDate': '2025-12-31', // String will be converted to date
  'Count': '42' // String will be converted to number
}, {
  typecast: true
});
```

#### Batch Create (Multiple Records)

Airtable allows up to 10 records per batch operation:

```javascript
const table = base('Tasks');

// Callback style
table.create([
  {
    'Name': 'Task 1',
    'Status': 'To Do'
  },
  {
    'Name': 'Task 2',
    'Status': 'In Progress'
  },
  {
    'Name': 'Task 3',
    'Status': 'Done'
  }
], function(err, records) {
  if (err) {
    console.error(err);
    return;
  }

  records.forEach(function(record) {
    console.log('Created:', record.id);
  });
});

// Async/await style
async function batchCreate() {
  try {
    const records = await table.create([
      {'Name': 'Task 1', 'Status': 'To Do'},
      {'Name': 'Task 2', 'Status': 'In Progress'},
      {'Name': 'Task 3', 'Status': 'Done'},
      {'Name': 'Task 4', 'Status': 'To Do'},
      {'Name': 'Task 5', 'Status': 'Review'}
    ]);

    console.log(`Created ${records.length} records`);

    return records;
  } catch (err) {
    console.error(err);
  }
}

// With typecast
const recordsWithTypecast = await table.create([
  {'Name': 'Task 1', 'Count': '10'},
  {'Name': 'Task 2', 'Count': '20'}
], {
  typecast: true
});
```

### Updating Records

#### Update Single Record (Partial Update)

```javascript
const table = base('Tasks');

// Callback style
table.update('recXXXXXXXXXXXXXX', {
  'Status': 'In Progress',
  'Notes': 'Updated notes'
}, function(err, record) {
  if (err) {
    console.error(err);
    return;
  }
  console.log('Updated record:', record.id);
});

// Promise style
table.update('recXXXXXXXXXXXXXX', {
  'Status': 'Done'
}).then(record => {
  console.log('Updated:', record.get('Status'));
}).catch(err => {
  console.error(err);
});

// Async/await style
async function updateTask(recordId) {
  try {
    const record = await table.update(recordId, {
      'Status': 'In Progress',
      'Progress': 50
    });

    console.log('Updated record:', record.id);
    console.log('New status:', record.get('Status'));

    return record;
  } catch (err) {
    console.error(err);
  }
}
```

#### Replace Record (Full Update)

The `replace()` method replaces all fields (fields not included will be cleared):

```javascript
const table = base('Tasks');

// Replace clears fields not specified
const record = await table.replace('recXXXXXXXXXXXXXX', {
  'Name': 'Completely New Task',
  'Status': 'To Do'
  // All other fields will be cleared
});
```

#### Batch Update (Multiple Records)

```javascript
const table = base('Tasks');

// Update up to 10 records at once
const records = await table.update([
  {
    id: 'recXXXXXXXXXXXXXX',
    fields: {
      'Status': 'Done'
    }
  },
  {
    id: 'recYYYYYYYYYYYYYY',
    fields: {
      'Status': 'In Progress',
      'Progress': 75
    }
  }
]);

console.log(`Updated ${records.length} records`);

// With typecast
const updatedWithTypecast = await table.update([
  {
    id: 'recXXXXXXXXXXXXXX',
    fields: {'Count': '100'}
  }
], {
  typecast: true
});
```

#### Batch Replace (Multiple Records)

```javascript
const table = base('Tasks');

// Replace up to 10 records at once
const records = await table.replace([
  {
    id: 'recXXXXXXXXXXXXXX',
    fields: {
      'Name': 'New Name',
      'Status': 'To Do'
    }
  },
  {
    id: 'recYYYYYYYYYYYYYY',
    fields: {
      'Name': 'Another Task',
      'Status': 'Done'
    }
  }
]);

// With typecast
const replacedWithTypecast = await table.replace([
  {
    id: 'recXXXXXXXXXXXXXX',
    fields: {'Name': 'Task', 'Count': '50'}
  }
], {
  typecast: true
});
```

### Deleting Records

#### Delete Single Record

```javascript
const table = base('Tasks');

// Callback style
table.destroy('recXXXXXXXXXXXXXX', function(err, deletedRecord) {
  if (err) {
    console.error(err);
    return;
  }
  console.log('Deleted record:', deletedRecord.id);
});

// Promise style
table.destroy('recXXXXXXXXXXXXXX')
  .then(deletedRecord => {
    console.log('Deleted:', deletedRecord.id);
  })
  .catch(err => {
    console.error(err);
  });

// Async/await style
async function deleteTask(recordId) {
  try {
    const deletedRecord = await table.destroy(recordId);
    console.log('Deleted record:', deletedRecord.id);

    return deletedRecord;
  } catch (err) {
    console.error(err);
  }
}
```

#### Batch Delete (Multiple Records)

```javascript
const table = base('Tasks');

// Delete up to 10 records at once
// Callback style
table.destroy([
  'recXXXXXXXXXXXXXX',
  'recYYYYYYYYYYYYYY',
  'recZZZZZZZZZZZZZZ'
], function(err, deletedRecords) {
  if (err) {
    console.error(err);
    return;
  }
  console.log('Deleted', deletedRecords.length, 'records');
});

// Async/await style
async function batchDelete(recordIds) {
  try {
    const deletedRecords = await table.destroy(recordIds);
    console.log(`Deleted ${deletedRecords.length} records`);

    deletedRecords.forEach(record => {
      console.log('Deleted:', record.id);
    });

    return deletedRecords;
  } catch (err) {
    console.error(err);
  }
}

// Example usage
await batchDelete([
  'recXXXXXXXXXXXXXX',
  'recYYYYYYYYYYYYYY'
]);
```

### Working with Different Field Types

#### Text Fields

```javascript
const record = await table.create({
  'Single Line Text': 'Short text',
  'Long Text': 'This is a much longer text\nwith multiple lines',
  'Email': 'user@example.com',
  'URL': 'https://example.com',
  'Phone': '+1-555-0100'
});
```

#### Number Fields

```javascript
const record = await table.create({
  'Number': 42,
  'Currency': 99.99,
  'Percent': 0.75,
  'Rating': 5
});
```

#### Date and Time Fields

```javascript
const record = await table.create({
  'Date': '2025-10-25',
  'DateTime': '2025-10-25T14:30:00.000Z'
});

// Access date fields
const dateValue = record.get('Date');
console.log('Date:', dateValue);
```

#### Checkbox (Boolean) Fields

```javascript
const record = await table.create({
  'Completed': true,
  'Active': false
});

// Access checkbox
const isCompleted = record.get('Completed');
if (isCompleted) {
  console.log('Task is completed');
}
```

#### Single Select Fields

```javascript
const record = await table.create({
  'Status': 'In Progress', // Must match exact option name
  'Priority': 'High'
});
```

#### Multiple Select Fields

```javascript
const record = await table.create({
  'Tags': ['Important', 'Urgent', 'Client Work']
});

// Access multiple select
const tags = record.get('Tags');
console.log('Tags:', tags.join(', '));
```

#### Attachment Fields

```javascript
const record = await table.create({
  'Attachments': [
    {
      url: 'https://example.com/image.jpg'
    },
    {
      url: 'https://example.com/document.pdf'
    }
  ]
});

// Access attachments
const attachments = record.get('Attachments');
attachments.forEach(attachment => {
  console.log('File:', attachment.filename);
  console.log('URL:', attachment.url);
  console.log('Size:', attachment.size);
  console.log('Type:', attachment.type);
});
```

#### Linked Record Fields

```javascript
// Link to existing records by their IDs
const record = await table.create({
  'Linked Records': [
    'recXXXXXXXXXXXXXX',
    'recYYYYYYYYYYYYYY'
  ]
});

// Access linked records
const linkedRecords = record.get('Linked Records');
console.log('Linked record IDs:', linkedRecords);
```

#### Collaborator Fields

```javascript
const record = await table.create({
  'Assignee': {
    id: 'usrXXXXXXXXXXXXXX',
    email: 'user@example.com'
  },
  'Collaborators': [
    {id: 'usrXXXXXXXXXXXXXX'},
    {id: 'usrYYYYYYYYYYYYYY'}
  ]
});
```

## Complete Examples

### Example 1: Task Management System

```javascript
const Airtable = require('airtable');

Airtable.configure({
  apiKey: process.env.AIRTABLE_API_KEY
});

const base = Airtable.base('appTaskManager');
const tasksTable = base('Tasks');

// Create a new task
async function createTask(taskData) {
  try {
    const record = await tasksTable.create({
      'Name': taskData.name,
      'Description': taskData.description,
      'Status': 'To Do',
      'Priority': taskData.priority || 'Medium',
      'Assignee': taskData.assignee,
      'Due Date': taskData.dueDate
    });

    console.log('Created task:', record.id);
    return record;
  } catch (err) {
    console.error('Error creating task:', err);
    throw err;
  }
}

// Get all active tasks
async function getActiveTasks() {
  try {
    const records = await tasksTable.select({
      filterByFormula: "AND({Status} != 'Done', {Status} != 'Cancelled')",
      sort: [
        {field: 'Priority', direction: 'desc'},
        {field: 'Due Date', direction: 'asc'}
      ]
    }).all();

    return records.map(record => ({
      id: record.id,
      name: record.get('Name'),
      status: record.get('Status'),
      priority: record.get('Priority'),
      assignee: record.get('Assignee'),
      dueDate: record.get('Due Date')
    }));
  } catch (err) {
    console.error('Error fetching tasks:', err);
    throw err;
  }
}

// Update task status
async function updateTaskStatus(taskId, newStatus) {
  try {
    const record = await tasksTable.update(taskId, {
      'Status': newStatus,
      'Last Modified': new Date().toISOString()
    });

    console.log('Updated task:', record.id);
    return record;
  } catch (err) {
    console.error('Error updating task:', err);
    throw err;
  }
}

// Get overdue tasks
async function getOverdueTasks() {
  try {
    const today = new Date().toISOString().split('T')[0];

    const records = await tasksTable.select({
      filterByFormula: `AND(
        {Status} != 'Done',
        {Due Date} < '${today}'
      )`,
      sort: [{field: 'Due Date', direction: 'asc'}]
    }).all();

    return records;
  } catch (err) {
    console.error('Error fetching overdue tasks:', err);
    throw err;
  }
}

// Bulk update multiple tasks
async function bulkUpdateStatus(taskIds, newStatus) {
  try {
    const updates = taskIds.map(id => ({
      id: id,
      fields: {
        'Status': newStatus,
        'Last Modified': new Date().toISOString()
      }
    }));

    // Process in batches of 10
    const results = [];
    for (let i = 0; i < updates.length; i += 10) {
      const batch = updates.slice(i, i + 10);
      const updated = await tasksTable.update(batch);
      results.push(...updated);
    }

    console.log(`Updated ${results.length} tasks`);
    return results;
  } catch (err) {
    console.error('Error bulk updating:', err);
    throw err;
  }
}

// Delete completed tasks older than 30 days
async function deleteOldCompletedTasks() {
  try {
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);
    const cutoffDate = thirtyDaysAgo.toISOString().split('T')[0];

    const records = await tasksTable.select({
      filterByFormula: `AND(
        {Status} = 'Done',
        {Completed Date} < '${cutoffDate}'
      )`
    }).all();

    const recordIds = records.map(record => record.id);

    // Delete in batches of 10
    for (let i = 0; i < recordIds.length; i += 10) {
      const batch = recordIds.slice(i, i + 10);
      await tasksTable.destroy(batch);
    }

    console.log(`Deleted ${recordIds.length} old completed tasks`);
  } catch (err) {
    console.error('Error deleting old tasks:', err);
    throw err;
  }
}

// Export module
module.exports = {
  createTask,
  getActiveTasks,
  updateTaskStatus,
  getOverdueTasks,
  bulkUpdateStatus,
  deleteOldCompletedTasks
};
```

### Example 2: Contact Management with Error Handling

```javascript
const Airtable = require('airtable');

const airtable = new Airtable({
  apiKey: process.env.AIRTABLE_API_KEY
});

const base = airtable.base('appContactManager');
const contactsTable = base('Contacts');

// Find contact by email
async function findContactByEmail(email) {
  try {
    const records = await contactsTable.select({
      filterByFormula: `{Email} = "${email}"`,
      maxRecords: 1
    }).firstPage();

    return records.length > 0 ? records[0] : null;
  } catch (err) {
    console.error(`Error finding contact with email ${email}:`, err);
    throw err;
  }
}

// Create or update contact (upsert pattern)
async function upsertContact(contactData) {
  try {
    // First, try to find existing contact
    const existingContact = await findContactByEmail(contactData.email);

    if (existingContact) {
      // Update existing contact
      const updated = await contactsTable.update(existingContact.id, {
        'First Name': contactData.firstName,
        'Last Name': contactData.lastName,
        'Phone': contactData.phone,
        'Company': contactData.company,
        'Last Updated': new Date().toISOString()
      });

      console.log('Updated existing contact:', updated.id);
      return {record: updated, action: 'updated'};
    } else {
      // Create new contact
      const created = await contactsTable.create({
        'First Name': contactData.firstName,
        'Last Name': contactData.lastName,
        'Email': contactData.email,
        'Phone': contactData.phone,
        'Company': contactData.company,
        'Created': new Date().toISOString()
      });

      console.log('Created new contact:', created.id);
      return {record: created, action: 'created'};
    }
  } catch (err) {
    console.error('Error upserting contact:', err);
    throw err;
  }
}

// Get all contacts from a company
async function getContactsByCompany(companyName) {
  try {
    const records = await contactsTable.select({
      filterByFormula: `{Company} = "${companyName}"`,
      sort: [
        {field: 'Last Name', direction: 'asc'},
        {field: 'First Name', direction: 'asc'}
      ]
    }).all();

    return records.map(record => ({
      id: record.id,
      name: `${record.get('First Name')} ${record.get('Last Name')}`,
      email: record.get('Email'),
      phone: record.get('Phone')
    }));
  } catch (err) {
    console.error(`Error fetching contacts for ${companyName}:`, err);
    throw err;
  }
}

// Export contacts to array
async function exportAllContacts() {
  try {
    const allContacts = [];

    await contactsTable.select().eachPage(
      function page(records, fetchNextPage) {
        records.forEach(record => {
          allContacts.push({
            id: record.id,
            firstName: record.get('First Name'),
            lastName: record.get('Last Name'),
            email: record.get('Email'),
            phone: record.get('Phone'),
            company: record.get('Company'),
            created: record.get('Created')
          });
        });

        fetchNextPage();
      },
      function done(err) {
        if (err) {
          console.error('Error during pagination:', err);
          throw err;
        }
      }
    );

    return allContacts;
  } catch (err) {
    console.error('Error exporting contacts:', err);
    throw err;
  }
}

module.exports = {
  findContactByEmail,
  upsertContact,
  getContactsByCompany,
  exportAllContacts
};
```

### Example 3: E-commerce Inventory Management

```javascript
const Airtable = require('airtable');
const base = Airtable.base('appInventory');

const productsTable = base('Products');
const ordersTable = base('Orders');

// Check product availability
async function checkInventory(productId) {
  try {
    const product = await productsTable.find(productId);

    return {
      id: product.id,
      name: product.get('Name'),
      sku: product.get('SKU'),
      quantity: product.get('Quantity in Stock'),
      available: product.get('Quantity in Stock') > 0
    };
  } catch (err) {
    console.error(`Error checking inventory for ${productId}:`, err);
    throw err;
  }
}

// Update stock after purchase
async function updateStockQuantity(productId, quantityChange) {
  try {
    const product = await productsTable.find(productId);
    const currentStock = product.get('Quantity in Stock');
    const newStock = currentStock + quantityChange;

    if (newStock < 0) {
      throw new Error('Insufficient stock');
    }

    const updated = await productsTable.update(productId, {
      'Quantity in Stock': newStock,
      'Last Updated': new Date().toISOString()
    });

    console.log(`Updated stock for ${product.get('Name')}: ${currentStock} -> ${newStock}`);
    return updated;
  } catch (err) {
    console.error(`Error updating stock for ${productId}:`, err);
    throw err;
  }
}

// Get low stock products
async function getLowStockProducts(threshold = 10) {
  try {
    const records = await productsTable.select({
      filterByFormula: `{Quantity in Stock} < ${threshold}`,
      sort: [{field: 'Quantity in Stock', direction: 'asc'}]
    }).all();

    return records.map(record => ({
      id: record.id,
      name: record.get('Name'),
      sku: record.get('SKU'),
      quantity: record.get('Quantity in Stock'),
      reorderLevel: record.get('Reorder Level')
    }));
  } catch (err) {
    console.error('Error fetching low stock products:', err);
    throw err;
  }
}

// Create order with multiple line items
async function createOrder(orderData) {
  try {
    // Create order record
    const order = await ordersTable.create({
      'Order Number': orderData.orderNumber,
      'Customer Name': orderData.customerName,
      'Customer Email': orderData.customerEmail,
      'Status': 'Pending',
      'Total Amount': orderData.totalAmount,
      'Order Date': new Date().toISOString()
    });

    // Update inventory for each product
    for (const item of orderData.items) {
      await updateStockQuantity(item.productId, -item.quantity);
    }

    console.log('Created order:', order.id);
    return order;
  } catch (err) {
    console.error('Error creating order:', err);
    throw err;
  }
}

// Get orders by status
async function getOrdersByStatus(status) {
  try {
    const records = await ordersTable.select({
      filterByFormula: `{Status} = "${status}"`,
      sort: [{field: 'Order Date', direction: 'desc'}]
    }).all();

    return records.map(record => ({
      id: record.id,
      orderNumber: record.get('Order Number'),
      customer: record.get('Customer Name'),
      total: record.get('Total Amount'),
      date: record.get('Order Date')
    }));
  } catch (err) {
    console.error(`Error fetching ${status} orders:`, err);
    throw err;
  }
}

module.exports = {
  checkInventory,
  updateStockQuantity,
  getLowStockProducts,
  createOrder,
  getOrdersByStatus
};
```

## Rate Limits and Error Handling

Airtable enforces a rate limit of **5 requests per second per base**.

### Handling Rate Limits

```javascript
async function retryWithBackoff(fn, maxRetries = 3, initialDelay = 1000) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (err) {
      if (err.statusCode === 429 && i < maxRetries - 1) {
        const delay = initialDelay * Math.pow(2, i);
        console.log(`Rate limited. Retrying in ${delay}ms...`);
        await new Promise(resolve => setTimeout(resolve, delay));
      } else {
        throw err;
      }
    }
  }
}

// Usage
const record = await retryWithBackoff(() =>
  table.create({'Name': 'New Record'})
);
```

### Error Handling Patterns

```javascript
const table = base('Tasks');

// Basic error handling
try {
  const record = await table.find('recXXXXXXXXXXXXXX');
} catch (err) {
  if (err.statusCode === 404) {
    console.error('Record not found');
  } else if (err.statusCode === 401) {
    console.error('Authentication failed');
  } else if (err.statusCode === 429) {
    console.error('Rate limit exceeded');
  } else {
    console.error('Unexpected error:', err);
  }
}

// Comprehensive error handling
async function safeCreate(recordData) {
  try {
    const record = await table.create(recordData);
    return {success: true, record};
  } catch (err) {
    console.error('Error creating record:', err.message);

    return {
      success: false,
      error: {
        message: err.message,
        statusCode: err.statusCode,
        type: err.error
      }
    };
  }
}
```

## TypeScript Support

```typescript
import Airtable, { FieldSet, Records } from 'airtable';

interface Task extends FieldSet {
  'Name': string;
  'Status': 'To Do' | 'In Progress' | 'Done';
  'Priority': 'Low' | 'Medium' | 'High';
  'Due Date': string;
  'Assignee': string;
}

Airtable.configure({
  apiKey: process.env.AIRTABLE_API_KEY!
});

const base = Airtable.base('appYourBaseId');
const tasksTable = base<Task>('Tasks');

async function getHighPriorityTasks(): Promise<Records<Task>> {
  const records = await tasksTable.select({
    filterByFormula: "{Priority} = 'High'",
    sort: [{field: 'Due Date', direction: 'asc'}]
  }).all();

  return records;
}

async function createTask(taskData: Partial<Task>) {
  const record = await tasksTable.create(taskData);
  return record;
}
```

## Common Formulas Reference

```javascript
// Exact match
"{Status} = 'Active'"

// Not equal
"{Status} != 'Done'"

// Greater than / Less than
"{Price} > 100"
"{Stock} <= 10"

// String contains (case-sensitive)
"FIND('urgent', {Notes}) > 0"

// String contains (case-insensitive)
"SEARCH('urgent', LOWER({Notes})) > 0"

// Is empty
"{Email} = ''"
"OR({Email} = BLANK())"

// Is not empty
"NOT({Email} = '')"
"{Email} != ''"

// AND condition
"AND({Status} = 'Active', {Priority} = 'High')"

// OR condition
"OR({Status} = 'Urgent', {Priority} = 'High')"

// Date comparisons
"{Created} > '2025-01-01'"
"{Due Date} < TODAY()"
"{Modified} >= DATEADD(TODAY(), -7, 'days')"

// Multiple conditions
"AND({Status} = 'Active', OR({Priority} = 'High', {Due Date} < TODAY()))"

// Check if field is in a list
"OR({Status} = 'Active', {Status} = 'In Progress', {Status} = 'Review')"

// Numeric range
"AND({Price} >= 10, {Price} <= 100)"
```
