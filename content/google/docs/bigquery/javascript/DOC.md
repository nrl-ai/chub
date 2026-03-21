---
name: bigquery
description: "BigQuery API JavaScript/TypeScript coding guidelines using the official Node.js client library"
metadata:
  languages: "javascript"
  versions: "8.1.1"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "google,bigquery,data-warehouse,sql,analytics"
---

# BigQuery API Coding Guidelines (JavaScript/TypeScript)

You are a BigQuery API coding expert. Help me with writing code using the BigQuery API and the official Node.js client library.

Please follow the following guidelines when generating code.

You can find the official SDK documentation and code samples here:
https://cloud.google.com/nodejs/docs/reference/bigquery/latest

## Golden Rule: Use the Correct and Current SDK

Always use the official Google Cloud BigQuery Node.js client library for all BigQuery API interactions. Do not use unofficial or deprecated libraries.

- **Library Name:** Google Cloud BigQuery Node.js Client
- **NPM Package:** `@google-cloud/bigquery`
- **Current Version:** 8.1.1
- **Do NOT use:** `bigquery` (deprecated standalone package)
- **Do NOT use:** Any unofficial BigQuery wrappers

**Installation:**

```bash
npm install @google-cloud/bigquery
```

**Correct Import:**

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
```

or with ES6 modules:

```javascript
import {BigQuery} from '@google-cloud/bigquery';
```

**Incorrect patterns to avoid:**
- `const bigquery = require('bigquery')` - Wrong package
- `import BigQuery from '@google-cloud/bigquery'` - Wrong import syntax (missing curly braces)
- Using any package other than `@google-cloud/bigquery`

## Installation and Setup

### Install the package

```bash
npm install @google-cloud/bigquery
```

### Authentication

BigQuery requires authentication. The client library uses Application Default Credentials (ADC).

**Set up authentication using one of these methods:**

**Option 1: Service Account Key File (Recommended for local development)**

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/your/service-account-key.json"
```

**Option 2: Application Default Credentials (Recommended for production)**

When running on Google Cloud (Cloud Functions, Cloud Run, GKE, etc.), authentication happens automatically.

**Option 3: Explicit credentials in code**

```javascript
const {BigQuery} = require('@google-cloud/bigquery');

const bigquery = new BigQuery({
  projectId: 'your-project-id',
  keyFilename: '/path/to/service-account-key.json'
});
```

### Environment Variables

```javascript
// Set these environment variables
process.env.GOOGLE_APPLICATION_CREDENTIALS = '/path/to/service-account-key.json';
process.env.GOOGLE_CLOUD_PROJECT = 'your-project-id';
```

## Initialization

### Basic Client Initialization

```javascript
const {BigQuery} = require('@google-cloud/bigquery');

// Uses GOOGLE_APPLICATION_CREDENTIALS and GOOGLE_CLOUD_PROJECT env vars
const bigquery = new BigQuery();
```

### Client with Explicit Project ID

```javascript
const {BigQuery} = require('@google-cloud/bigquery');

const bigquery = new BigQuery({
  projectId: 'your-project-id'
});
```

### Client with Explicit Credentials

```javascript
const {BigQuery} = require('@google-cloud/bigquery');

const bigquery = new BigQuery({
  projectId: 'your-project-id',
  keyFilename: '/path/to/service-account-key.json'
});
```

### Client with Location

```javascript
const {BigQuery} = require('@google-cloud/bigquery');

const bigquery = new BigQuery({
  projectId: 'your-project-id',
  location: 'US' // or 'EU', 'us-central1', etc.
});
```

## Working with Datasets

### Create a Dataset

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function createDataset() {
  const datasetId = 'my_new_dataset';

  const [dataset] = await bigquery.createDataset(datasetId);
  console.log(`Dataset ${dataset.id} created.`);
}

createDataset();
```

### Create Dataset with Location

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function createDatasetWithLocation() {
  const datasetId = 'my_new_dataset';

  const options = {
    location: 'US'
  };

  const [dataset] = await bigquery.createDataset(datasetId, options);
  console.log(`Dataset ${dataset.id} created in ${dataset.location}.`);
}

createDatasetWithLocation();
```

### List Datasets

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function listDatasets() {
  const [datasets] = await bigquery.getDatasets();

  console.log('Datasets:');
  datasets.forEach(dataset => console.log(dataset.id));
}

listDatasets();
```

### Get Dataset Reference

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

const dataset = bigquery.dataset('my_dataset');
```

### Check if Dataset Exists

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function datasetExists() {
  const dataset = bigquery.dataset('my_dataset');
  const [exists] = await dataset.exists();
  console.log(`Dataset exists: ${exists}`);
}

datasetExists();
```

### Delete a Dataset

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function deleteDataset() {
  const datasetId = 'my_dataset';

  await bigquery.dataset(datasetId).delete({force: true}); // force deletes all tables
  console.log(`Dataset ${datasetId} deleted.`);
}

deleteDataset();
```

## Working with Tables

### Create a Table

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function createTable() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const schema = [
    {name: 'name', type: 'STRING', mode: 'REQUIRED'},
    {name: 'age', type: 'INTEGER', mode: 'NULLABLE'},
    {name: 'email', type: 'STRING', mode: 'NULLABLE'}
  ];

  const [table] = await bigquery
    .dataset(datasetId)
    .createTable(tableId, {schema: schema});

  console.log(`Table ${table.id} created.`);
}

createTable();
```

### Create Table with Advanced Schema

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function createTableWithAdvancedSchema() {
  const datasetId = 'my_dataset';
  const tableId = 'my_advanced_table';

  const schema = [
    {name: 'id', type: 'INTEGER', mode: 'REQUIRED'},
    {name: 'name', type: 'STRING', mode: 'REQUIRED'},
    {name: 'timestamp', type: 'TIMESTAMP', mode: 'REQUIRED'},
    {name: 'scores', type: 'FLOAT', mode: 'REPEATED'}, // Array field
    {
      name: 'address',
      type: 'RECORD', // Nested/struct field
      mode: 'NULLABLE',
      fields: [
        {name: 'street', type: 'STRING'},
        {name: 'city', type: 'STRING'},
        {name: 'zipcode', type: 'STRING'}
      ]
    }
  ];

  const options = {
    schema: schema,
    location: 'US',
    timePartitioning: {
      type: 'DAY',
      field: 'timestamp'
    }
  };

  const [table] = await bigquery
    .dataset(datasetId)
    .createTable(tableId, options);

  console.log(`Table ${table.id} created with partitioning.`);
}

createTableWithAdvancedSchema();
```

### List Tables

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function listTables() {
  const datasetId = 'my_dataset';

  const [tables] = await bigquery.dataset(datasetId).getTables();

  console.log('Tables:');
  tables.forEach(table => console.log(table.id));
}

listTables();
```

### Get Table Metadata

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function getTableMetadata() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const table = bigquery.dataset(datasetId).table(tableId);
  const [metadata] = await table.getMetadata();

  console.log('Table metadata:');
  console.log(`Schema: ${JSON.stringify(metadata.schema.fields)}`);
  console.log(`Num rows: ${metadata.numRows}`);
  console.log(`Num bytes: ${metadata.numBytes}`);
  console.log(`Created: ${metadata.creationTime}`);
}

getTableMetadata();
```

### Get Table Reference

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

const table = bigquery.dataset('my_dataset').table('my_table');
```

### Check if Table Exists

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function tableExists() {
  const table = bigquery.dataset('my_dataset').table('my_table');
  const [exists] = await table.exists();
  console.log(`Table exists: ${exists}`);
}

tableExists();
```

### Delete a Table

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function deleteTable() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  await bigquery.dataset(datasetId).table(tableId).delete();
  console.log(`Table ${tableId} deleted.`);
}

deleteTable();
```

### Copy a Table

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function copyTable() {
  const srcDatasetId = 'my_dataset';
  const srcTableId = 'my_source_table';
  const destDatasetId = 'my_dataset';
  const destTableId = 'my_destination_table';

  const srcTable = bigquery.dataset(srcDatasetId).table(srcTableId);
  const destTable = bigquery.dataset(destDatasetId).table(destTableId);

  const [job] = await srcTable.copy(destTable);

  console.log(`Job ${job.id} completed. Table copied.`);
}

copyTable();
```

## Querying Data

### Simple Query

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function queryData() {
  const query = `
    SELECT name, age
    FROM \`my-project.my_dataset.my_table\`
    WHERE age > 25
    ORDER BY age DESC
    LIMIT 10
  `;

  const [rows] = await bigquery.query(query);

  console.log('Rows:');
  rows.forEach(row => console.log(`${row.name}: ${row.age}`));
}

queryData();
```

### Query with Location

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function queryWithLocation() {
  const query = 'SELECT name FROM `my-project.my_dataset.my_table` LIMIT 10';

  const options = {
    query: query,
    location: 'US'
  };

  const [rows] = await bigquery.query(options);

  rows.forEach(row => console.log(row.name));
}

queryWithLocation();
```

### Query Public Dataset

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function queryPublicDataset() {
  const query = `
    SELECT name, SUM(number) as total
    FROM \`bigquery-public-data.usa_names.usa_1910_2013\`
    WHERE state = 'TX'
    GROUP BY name, state
    ORDER BY total DESC
    LIMIT 10
  `;

  const [rows] = await bigquery.query(query);

  console.log('Top 10 names in Texas:');
  rows.forEach(row => console.log(`${row.name}: ${row.total}`));
}

queryPublicDataset();
```

### Query with Parameters (Parameterized Query)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function queryWithParameters() {
  const query = `
    SELECT name, age
    FROM \`my-project.my_dataset.my_table\`
    WHERE age > @min_age
      AND name LIKE @name_pattern
    LIMIT @limit
  `;

  const options = {
    query: query,
    params: {
      min_age: 25,
      name_pattern: 'John%',
      limit: 10
    }
  };

  const [rows] = await bigquery.query(options);

  rows.forEach(row => console.log(`${row.name}: ${row.age}`));
}

queryWithParameters();
```

### Query with Typed Parameters

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function queryWithTypedParameters() {
  const query = `
    SELECT name, timestamp
    FROM \`my-project.my_dataset.my_table\`
    WHERE timestamp > @start_date
      AND status IN UNNEST(@statuses)
  `;

  const options = {
    query: query,
    params: {
      start_date: '2024-01-01',
      statuses: ['active', 'pending']
    },
    types: {
      start_date: 'DATE',
      statuses: ['STRING']
    }
  };

  const [rows] = await bigquery.query(options);

  rows.forEach(row => console.log(row));
}

queryWithTypedParameters();
```

### Query with Job Configuration

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function queryWithJobConfig() {
  const query = 'SELECT * FROM `my-project.my_dataset.my_table` LIMIT 100';

  const options = {
    query: query,
    location: 'US',
    useLegacySql: false, // Use Standard SQL (default)
    useQueryCache: true,
    maximumBytesBilled: '1000000' // Set query cost limit
  };

  const [job] = await bigquery.createQueryJob(options);
  console.log(`Job ${job.id} started.`);

  const [rows] = await job.getQueryResults();
  console.log(`Rows: ${rows.length}`);
}

queryWithJobConfig();
```

### Dry Run Query (Estimate Query Cost)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function dryRunQuery() {
  const query = 'SELECT * FROM `my-project.my_dataset.my_table`';

  const options = {
    query: query,
    dryRun: true
  };

  const [job] = await bigquery.createQueryJob(options);

  console.log('Query would process:');
  console.log(`${job.metadata.statistics.totalBytesProcessed} bytes`);
}

dryRunQuery();
```

### Check Query Job Status

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function checkJobStatus() {
  const query = 'SELECT * FROM `my-project.my_dataset.large_table`';

  const [job] = await bigquery.createQueryJob({query: query});

  console.log(`Job ${job.id} started.`);

  // Wait for job to complete
  const [rows] = await job.getQueryResults();

  // Get job metadata
  const [metadata] = await job.getMetadata();

  console.log('Job statistics:');
  console.log(`State: ${metadata.status.state}`);
  console.log(`Bytes processed: ${metadata.statistics.totalBytesProcessed}`);
  console.log(`Rows: ${rows.length}`);
}

checkJobStatus();
```

## Inserting Data

### Streaming Insert (Single Row)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function insertSingleRow() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const row = {
    name: 'John Doe',
    age: 30,
    email: 'john@example.com'
  };

  await bigquery
    .dataset(datasetId)
    .table(tableId)
    .insert(row);

  console.log('Row inserted successfully.');
}

insertSingleRow();
```

### Streaming Insert (Multiple Rows)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function insertMultipleRows() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const rows = [
    {name: 'Tom Smith', age: 28, email: 'tom@example.com'},
    {name: 'Jane Doe', age: 32, email: 'jane@example.com'},
    {name: 'Bob Johnson', age: 45, email: 'bob@example.com'}
  ];

  await bigquery
    .dataset(datasetId)
    .table(tableId)
    .insert(rows);

  console.log(`Inserted ${rows.length} rows successfully.`);
}

insertMultipleRows();
```

### Streaming Insert with Options

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function insertWithOptions() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const rows = [
    {name: 'Alice', age: 25, email: 'alice@example.com'},
    {name: 'Charlie', age: 35, email: 'charlie@example.com'}
  ];

  const options = {
    skipInvalidRows: true, // Skip rows with errors
    ignoreUnknownValues: true, // Ignore fields not in schema
    raw: false // Use field names, not raw API format
  };

  await bigquery
    .dataset(datasetId)
    .table(tableId)
    .insert(rows, options);

  console.log('Rows inserted with options.');
}

insertWithOptions();
```

### Streaming Insert with Error Handling

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function insertWithErrorHandling() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const rows = [
    {name: 'Valid User', age: 30, email: 'valid@example.com'},
    {name: 'Invalid User', age: 'not a number', email: 'invalid@example.com'}
  ];

  try {
    await bigquery
      .dataset(datasetId)
      .table(tableId)
      .insert(rows);

    console.log('All rows inserted successfully.');
  } catch (error) {
    if (error.name === 'PartialFailureError') {
      console.error('Some rows failed to insert:');
      error.errors.forEach((err, index) => {
        console.error(`Row ${index}:`, err.errors);
      });
    } else {
      console.error('Insert error:', error);
    }
  }
}

insertWithErrorHandling();
```

## Loading Data from Files

### Load Data from Local CSV File

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function loadCSVFromFile() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';
  const filename = '/path/to/file.csv';

  const metadata = {
    sourceFormat: 'CSV',
    skipLeadingRows: 1, // Skip header row
    autodetect: true, // Auto-detect schema
    location: 'US'
  };

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .load(filename, metadata);

  console.log(`Job ${job.id} completed. Data loaded.`);
}

loadCSVFromFile();
```

### Load Data from Cloud Storage (CSV)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function loadCSVFromGCS() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const metadata = {
    sourceFormat: 'CSV',
    skipLeadingRows: 1,
    schema: {
      fields: [
        {name: 'name', type: 'STRING'},
        {name: 'age', type: 'INTEGER'},
        {name: 'email', type: 'STRING'}
      ]
    },
    location: 'US',
    writeDisposition: 'WRITE_TRUNCATE' // Overwrite table
  };

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .load('gs://my-bucket/data.csv', metadata);

  console.log(`Job ${job.id} completed.`);

  const errors = job.status.errors;
  if (errors && errors.length > 0) {
    throw errors;
  }
}

loadCSVFromGCS();
```

### Load Data from Cloud Storage (JSON)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function loadJSONFromGCS() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const metadata = {
    sourceFormat: 'NEWLINE_DELIMITED_JSON',
    autodetect: true,
    location: 'US',
    writeDisposition: 'WRITE_APPEND' // Append to table
  };

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .load('gs://my-bucket/data.json', metadata);

  console.log(`Job ${job.id} completed. JSON data loaded.`);
}

loadJSONFromGCS();
```

### Load Data from Multiple Files

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function loadFromMultipleFiles() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const metadata = {
    sourceFormat: 'CSV',
    skipLeadingRows: 1,
    autodetect: true,
    location: 'US'
  };

  const sourceUris = [
    'gs://my-bucket/data1.csv',
    'gs://my-bucket/data2.csv',
    'gs://my-bucket/data3.csv'
  ];

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .load(sourceUris, metadata);

  console.log(`Job ${job.id} completed. Multiple files loaded.`);
}

loadFromMultipleFiles();
```

### Load Data from Cloud Storage (Parquet)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function loadParquetFromGCS() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const metadata = {
    sourceFormat: 'PARQUET',
    location: 'US',
    writeDisposition: 'WRITE_TRUNCATE'
  };

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .load('gs://my-bucket/data.parquet', metadata);

  console.log('Parquet data loaded successfully.');
}

loadParquetFromGCS();
```

### Load with createLoadJob

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function createLoadJobExample() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const metadata = {
    sourceFormat: 'CSV',
    skipLeadingRows: 1,
    schema: {
      fields: [
        {name: 'id', type: 'INTEGER'},
        {name: 'name', type: 'STRING'},
        {name: 'value', type: 'FLOAT'}
      ]
    },
    location: 'US'
  };

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .createLoadJob('gs://my-bucket/file.csv', metadata);

  console.log(`Load job ${job.id} started.`);

  // Wait for job to complete
  const [completedJob] = await job.promise();

  console.log(`Job ${completedJob.id} completed.`);
}

createLoadJobExample();
```

## Exporting Data

### Export Table to Cloud Storage (CSV)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const {Storage} = require('@google-cloud/storage');

const bigquery = new BigQuery();
const storage = new Storage();

async function exportTableToGCS() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';
  const bucketName = 'my-bucket';
  const filename = 'export.csv';

  const destination = storage.bucket(bucketName).file(filename);

  const options = {
    format: 'CSV',
    location: 'US'
  };

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .extract(destination, options);

  console.log(`Job ${job.id} completed. Table exported to ${filename}.`);
}

exportTableToGCS();
```

### Export Table to Cloud Storage (JSON)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const {Storage} = require('@google-cloud/storage');

const bigquery = new BigQuery();
const storage = new Storage();

async function exportTableToJSON() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';
  const bucketName = 'my-bucket';
  const filename = 'export.json';

  const destination = storage.bucket(bucketName).file(filename);

  const options = {
    format: 'JSON',
    location: 'US'
  };

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .extract(destination, options);

  console.log('Table exported to JSON.');
}

exportTableToJSON();
```

### Export Table with Compression

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const {Storage} = require('@google-cloud/storage');

const bigquery = new BigQuery();
const storage = new Storage();

async function exportTableCompressed() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';
  const bucketName = 'my-bucket';
  const filename = 'export.csv.gz';

  const destination = storage.bucket(bucketName).file(filename);

  const options = {
    format: 'CSV',
    gzip: true,
    location: 'US'
  };

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .extract(destination, options);

  console.log('Table exported with gzip compression.');
}

exportTableCompressed();
```

### Export to Multiple Files

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const {Storage} = require('@google-cloud/storage');

const bigquery = new BigQuery();
const storage = new Storage();

async function exportToMultipleFiles() {
  const datasetId = 'my_dataset';
  const tableId = 'my_large_table';
  const bucketName = 'my-bucket';

  // Use wildcard to split into multiple files
  const destination = storage.bucket(bucketName).file('export-*.csv');

  const options = {
    format: 'CSV',
    location: 'US'
  };

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .extract(destination, options);

  console.log('Table exported to multiple files.');
}

exportToMultipleFiles();
```

### Export with createExtractJob

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const {Storage} = require('@google-cloud/storage');

const bigquery = new BigQuery();
const storage = new Storage();

async function createExtractJobExample() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';
  const bucketName = 'my-bucket';
  const filename = 'extract.csv';

  const destination = storage.bucket(bucketName).file(filename);

  const options = {
    format: 'CSV',
    printHeader: true,
    location: 'US'
  };

  const [job] = await bigquery
    .dataset(datasetId)
    .table(tableId)
    .createExtractJob(destination, options);

  console.log(`Extract job ${job.id} started.`);

  const [completedJob] = await job.promise();
  console.log(`Job ${completedJob.id} completed.`);
}

createExtractJobExample();
```

## DML Operations (INSERT, UPDATE, DELETE)

### INSERT with DML

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function insertWithDML() {
  const query = `
    INSERT INTO \`my-project.my_dataset.my_table\` (name, age, email)
    VALUES
      ('Alice', 25, 'alice@example.com'),
      ('Bob', 30, 'bob@example.com'),
      ('Charlie', 35, 'charlie@example.com')
  `;

  const [job] = await bigquery.createQueryJob({
    query: query,
    location: 'US'
  });

  await job.getQueryResults();
  console.log('Rows inserted via DML.');
}

insertWithDML();
```

### UPDATE with DML

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function updateWithDML() {
  const query = `
    UPDATE \`my-project.my_dataset.my_table\`
    SET age = age + 1
    WHERE name = 'Alice'
  `;

  const [job] = await bigquery.createQueryJob({
    query: query,
    location: 'US'
  });

  await job.getQueryResults();
  console.log('Rows updated via DML.');
}

updateWithDML();
```

### DELETE with DML

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function deleteWithDML() {
  const query = `
    DELETE FROM \`my-project.my_dataset.my_table\`
    WHERE age < 18
  `;

  const [job] = await bigquery.createQueryJob({
    query: query,
    location: 'US'
  });

  await job.getQueryResults();
  console.log('Rows deleted via DML.');
}

deleteWithDML();
```

### MERGE (Upsert) with DML

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function mergeWithDML() {
  const query = `
    MERGE \`my-project.my_dataset.target_table\` T
    USING \`my-project.my_dataset.source_table\` S
    ON T.id = S.id
    WHEN MATCHED THEN
      UPDATE SET T.name = S.name, T.age = S.age
    WHEN NOT MATCHED THEN
      INSERT (id, name, age) VALUES (S.id, S.name, S.age)
  `;

  const [job] = await bigquery.createQueryJob({
    query: query,
    location: 'US'
  });

  await job.getQueryResults();
  console.log('Merge operation completed.');
}

mergeWithDML();
```

### TRUNCATE Table

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function truncateTable() {
  const query = `TRUNCATE TABLE \`my-project.my_dataset.my_table\``;

  const [job] = await bigquery.createQueryJob({
    query: query,
    location: 'US'
  });

  await job.getQueryResults();
  console.log('Table truncated.');
}

truncateTable();
```

## Advanced Query Patterns

### Create Table As Select (CTAS)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function createTableAsSelect() {
  const query = `
    CREATE OR REPLACE TABLE \`my-project.my_dataset.new_table\` AS
    SELECT name, age
    FROM \`my-project.my_dataset.source_table\`
    WHERE age > 25
  `;

  const [job] = await bigquery.createQueryJob({
    query: query,
    location: 'US'
  });

  await job.getQueryResults();
  console.log('Table created from query.');
}

createTableAsSelect();
```

### Query with Common Table Expressions (CTEs)

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function queryWithCTE() {
  const query = `
    WITH filtered_data AS (
      SELECT name, age, email
      FROM \`my-project.my_dataset.my_table\`
      WHERE age > 25
    ),
    aggregated_data AS (
      SELECT COUNT(*) as total
      FROM filtered_data
    )
    SELECT * FROM aggregated_data
  `;

  const [rows] = await bigquery.query(query);

  rows.forEach(row => console.log(row));
}

queryWithCTE();
```

### Query with Window Functions

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function queryWithWindowFunctions() {
  const query = `
    SELECT
      name,
      age,
      ROW_NUMBER() OVER (ORDER BY age DESC) as rank,
      AVG(age) OVER () as avg_age
    FROM \`my-project.my_dataset.my_table\`
    ORDER BY rank
    LIMIT 10
  `;

  const [rows] = await bigquery.query(query);

  rows.forEach(row => {
    console.log(`${row.rank}. ${row.name} (${row.age}) - Avg: ${row.avg_age}`);
  });
}

queryWithWindowFunctions();
```

### Query with Arrays and Structs

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function queryArraysAndStructs() {
  const query = `
    SELECT
      name,
      scores,
      ARRAY_LENGTH(scores) as num_scores,
      address.city,
      address.zipcode
    FROM \`my-project.my_dataset.my_table\`
    WHERE 90 IN UNNEST(scores)
  `;

  const [rows] = await bigquery.query(query);

  rows.forEach(row => console.log(row));
}

queryArraysAndStructs();
```

## Pagination and Iterating Results

### Paginate Query Results

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function paginateResults() {
  const query = 'SELECT * FROM `my-project.my_dataset.large_table`';

  const options = {
    query: query,
    maxResults: 100 // Page size
  };

  const [job] = await bigquery.createQueryJob(options);

  let pageToken = null;
  let pageCount = 0;

  do {
    const [rows, nextQuery, apiResponse] = await job.getQueryResults({
      pageToken: pageToken,
      maxResults: 100
    });

    pageCount++;
    console.log(`Page ${pageCount}: ${rows.length} rows`);

    rows.forEach(row => {
      // Process each row
      console.log(row);
    });

    pageToken = apiResponse.pageToken;
  } while (pageToken);

  console.log(`Total pages: ${pageCount}`);
}

paginateResults();
```

### Stream Query Results

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function streamQueryResults() {
  const query = 'SELECT * FROM `my-project.my_dataset.large_table`';

  const [job] = await bigquery.createQueryJob({query: query});

  return new Promise((resolve, reject) => {
    let rowCount = 0;

    job.getQueryResultsStream()
      .on('data', row => {
        rowCount++;
        console.log(row);
      })
      .on('error', reject)
      .on('end', () => {
        console.log(`Processed ${rowCount} rows`);
        resolve();
      });
  });
}

streamQueryResults();
```

## Working with Views

### Create a View

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function createView() {
  const datasetId = 'my_dataset';
  const tableId = 'my_view';

  const query = `
    SELECT name, age
    FROM \`my-project.my_dataset.my_table\`
    WHERE age > 25
  `;

  const options = {
    view: query,
    location: 'US'
  };

  const [view] = await bigquery
    .dataset(datasetId)
    .createTable(tableId, options);

  console.log(`View ${view.id} created.`);
}

createView();
```

### Update a View

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function updateView() {
  const datasetId = 'my_dataset';
  const viewId = 'my_view';

  const newQuery = `
    SELECT name, age, email
    FROM \`my-project.my_dataset.my_table\`
    WHERE age > 30
  `;

  const table = bigquery.dataset(datasetId).table(viewId);

  await table.setMetadata({
    view: newQuery
  });

  console.log('View updated.');
}

updateView();
```

## Error Handling

### Complete Error Handling Example

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function queryWithErrorHandling() {
  const query = 'SELECT * FROM `my-project.my_dataset.my_table` LIMIT 10';

  try {
    const [rows] = await bigquery.query({
      query: query,
      location: 'US'
    });

    console.log(`Query returned ${rows.length} rows`);
    rows.forEach(row => console.log(row));

  } catch (error) {
    console.error('Error occurred:');
    console.error(`Message: ${error.message}`);

    if (error.code) {
      console.error(`Error code: ${error.code}`);
    }

    if (error.errors) {
      console.error('Detailed errors:');
      error.errors.forEach(err => console.error(err));
    }
  }
}

queryWithErrorHandling();
```

### Handle Partial Insert Errors

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function handlePartialInsertErrors() {
  const datasetId = 'my_dataset';
  const tableId = 'my_table';

  const rows = [
    {name: 'Valid', age: 25},
    {name: 'Invalid', age: 'not a number'},
    {name: 'Another Valid', age: 30}
  ];

  try {
    await bigquery
      .dataset(datasetId)
      .table(tableId)
      .insert(rows);

    console.log('All rows inserted successfully.');
  } catch (error) {
    if (error.name === 'PartialFailureError') {
      console.log('Some rows were inserted, but some failed:');

      error.errors.forEach((rowError, index) => {
        console.log(`Row ${index} errors:`);
        rowError.errors.forEach(err => {
          console.log(`  - ${err.reason}: ${err.message}`);
        });
      });
    } else {
      console.error('Complete failure:', error.message);
    }
  }
}

handlePartialInsertErrors();
```

## Working with Jobs

### List Recent Jobs

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function listJobs() {
  const [jobs] = await bigquery.getJobs({
    maxResults: 10,
    allUsers: false
  });

  console.log('Recent jobs:');
  jobs.forEach(job => {
    console.log(`${job.id} - ${job.metadata.status.state}`);
  });
}

listJobs();
```

### Get Job Details

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function getJobDetails(jobId) {
  const job = bigquery.job(jobId);
  const [metadata] = await job.getMetadata();

  console.log('Job details:');
  console.log(`ID: ${metadata.id}`);
  console.log(`State: ${metadata.status.state}`);
  console.log(`Created: ${metadata.statistics.creationTime}`);
  console.log(`Started: ${metadata.statistics.startTime}`);
  console.log(`Ended: ${metadata.statistics.endTime}`);

  if (metadata.statistics.query) {
    console.log(`Bytes processed: ${metadata.statistics.query.totalBytesProcessed}`);
  }
}

getJobDetails('your-job-id');
```

### Cancel a Job

```javascript
const {BigQuery} = require('@google-cloud/bigquery');
const bigquery = new BigQuery();

async function cancelJob(jobId) {
  const job = bigquery.job(jobId);

  await job.cancel();
  console.log(`Job ${jobId} cancelled.`);
}

cancelJob('your-job-id');
```

## Important Notes

### Streaming Insert Limitations

- Streaming inserts are available immediately for querying but may take up to 90 minutes to become available for copy, export, or DML operations
- Tables with data in the streaming buffer cannot be updated or deleted using UPDATE/DELETE DML statements
- Use batch loading for large data volumes to reduce costs

### DML Quotas

- Maximum 1,000 DML statements per table per day
- Use batch operations when possible to stay within quota limits

### Query Costs

- Queries are billed based on bytes processed
- Use `dryRun: true` to estimate query costs before running
- Use partitioned tables and clustering to reduce query costs
- Set `maximumBytesBilled` to prevent accidentally expensive queries

### Data Types

Available BigQuery data types:
- **Numeric**: INTEGER (INT64), NUMERIC, BIGNUMERIC, FLOAT (FLOAT64)
- **String**: STRING, BYTES
- **Date/Time**: DATE, DATETIME, TIME, TIMESTAMP
- **Boolean**: BOOLEAN (BOOL)
- **Complex**: ARRAY, STRUCT (RECORD), GEOGRAPHY, JSON

### Best Practices

- Always specify location for datasets and queries to avoid cross-region data transfer costs
- Use parameterized queries to improve query caching and prevent SQL injection
- Use batch loading instead of streaming for large data volumes
- Enable query cache when possible to reduce costs
- Use partitioned and clustered tables for better query performance
- Set timeouts and cost limits for queries in production
- Handle errors gracefully, especially for streaming inserts

## Useful Links

- Official Documentation: https://cloud.google.com/bigquery/docs
- Node.js Client Reference: https://cloud.google.com/nodejs/docs/reference/bigquery/latest
- BigQuery Pricing: https://cloud.google.com/bigquery/pricing
- SQL Reference: https://cloud.google.com/bigquery/docs/reference/standard-sql/query-syntax
- Quotas and Limits: https://cloud.google.com/bigquery/quotas
