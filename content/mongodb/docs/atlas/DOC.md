---
name: atlas
description: "MongoDB Node.js driver for interacting with MongoDB Atlas databases using the official JavaScript/TypeScript SDK."
metadata:
  languages: "javascript"
  versions: "6.20.0"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "mongodb,atlas,database,nosql,driver"
---

# MongoDB Atlas Coding Guidelines (JavaScript/TypeScript)

You are a MongoDB Atlas coding expert. Help me with writing code using the MongoDB Node.js driver calling the official libraries and SDKs.

## Golden Rule: Use the Correct and Current SDK

Always use the official MongoDB Node.js driver for all MongoDB Atlas interactions.

- **Library Name:** MongoDB Node.js Driver
- **NPM Package:** `mongodb`
- **GitHub:** https://github.com/mongodb/node-mongodb-native

**Installation:**

```bash
npm install mongodb
```

**Import Patterns:**

```javascript
// ES6 import (recommended)
import { MongoClient } from 'mongodb';

// CommonJS require
const { MongoClient } = require('mongodb');

// Additional utilities
import { MongoClient, ObjectId, Timestamp } from 'mongodb';
```

**Do NOT use:**
- Deprecated MongoDB packages
- Third-party MongoDB wrappers (unless specifically requested)
- Old connection patterns from MongoDB driver v2 or v3

## Installation and Environment Setup

```bash
npm install mongodb
```

**Environment Variables Setup:**

Create a `.env` file in your project root:

```bash
MONGODB_URI=mongodb+srv://username:password@cluster.mongodb.net/database?retryWrites=true&w=majority
```

**Using dotenv for environment variables:**

```bash
npm install dotenv
```

```javascript
import 'dotenv/config';
import { MongoClient } from 'mongodb';

const uri = process.env.MONGODB_URI;
```

## Initialization and Connection

The MongoDB driver requires creating a `MongoClient` instance for all database operations.

**Basic Connection:**

```javascript
import { MongoClient } from 'mongodb';

const uri = process.env.MONGODB_URI;
const client = new MongoClient(uri);

async function main() {
  try {
    await client.connect();
    console.log('Connected to MongoDB Atlas');

    const database = client.db('myDatabase');
    const collection = database.collection('myCollection');

    // Perform operations...

  } finally {
    await client.close();
  }
}

main().catch(console.error);
```

**Connection with Options:**

```javascript
import { MongoClient, ServerApiVersion } from 'mongodb';

const uri = process.env.MONGODB_URI;

const client = new MongoClient(uri, {
  serverApi: {
    version: ServerApiVersion.v1,
    strict: true,
    deprecationErrors: true,
  }
});

async function run() {
  try {
    await client.connect();
    await client.db('admin').command({ ping: 1 });
    console.log('Pinged your deployment. Successfully connected to MongoDB!');
  } finally {
    await client.close();
  }
}

run().catch(console.error);
```

**Reusable Connection Pattern:**

```javascript
import { MongoClient } from 'mongodb';

let client;
let clientPromise;

const uri = process.env.MONGODB_URI;
const options = {};

if (process.env.NODE_ENV === 'development') {
  // In development mode, use a global variable to preserve connection
  if (!global._mongoClientPromise) {
    client = new MongoClient(uri, options);
    global._mongoClientPromise = client.connect();
  }
  clientPromise = global._mongoClientPromise;
} else {
  // In production mode, create a new client
  client = new MongoClient(uri, options);
  clientPromise = client.connect();
}

export default clientPromise;
```

## CRUD Operations

### Insert Documents

**Insert One Document:**

```javascript
import { MongoClient } from 'mongodb';

const client = new MongoClient(process.env.MONGODB_URI);

async function insertDocument() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const doc = {
      name: 'John Doe',
      email: 'john@example.com',
      age: 30,
      createdAt: new Date()
    };

    const result = await collection.insertOne(doc);
    console.log(`Document inserted with _id: ${result.insertedId}`);

  } finally {
    await client.close();
  }
}

insertDocument().catch(console.error);
```

**Insert Multiple Documents:**

```javascript
async function insertMultipleDocuments() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const docs = [
      { name: 'Alice', email: 'alice@example.com', age: 25 },
      { name: 'Bob', email: 'bob@example.com', age: 32 },
      { name: 'Charlie', email: 'charlie@example.com', age: 28 }
    ];

    const result = await collection.insertMany(docs);
    console.log(`${result.insertedCount} documents inserted`);
    console.log('Inserted IDs:', result.insertedIds);

  } finally {
    await client.close();
  }
}

insertMultipleDocuments().catch(console.error);
```

**Insert with Options:**

```javascript
async function insertWithOptions() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const doc = { name: 'David', email: 'david@example.com' };

    const result = await collection.insertOne(doc, {
      writeConcern: { w: 'majority', wtimeout: 5000 }
    });

    console.log(`Document inserted: ${result.insertedId}`);

  } finally {
    await client.close();
  }
}

insertWithOptions().catch(console.error);
```

### Find Documents

**Find All Documents:**

```javascript
async function findAllDocuments() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const cursor = collection.find({});
    const documents = await cursor.toArray();

    console.log('All documents:', documents);

  } finally {
    await client.close();
  }
}

findAllDocuments().catch(console.error);
```

**Find with Filter:**

```javascript
async function findWithFilter() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    // Find users older than 25
    const query = { age: { $gt: 25 } };
    const cursor = collection.find(query);
    const results = await cursor.toArray();

    console.log('Users older than 25:', results);

  } finally {
    await client.close();
  }
}

findWithFilter().catch(console.error);
```

**Find One Document:**

```javascript
async function findOneDocument() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const query = { email: 'john@example.com' };
    const user = await collection.findOne(query);

    if (user) {
      console.log('Found user:', user);
    } else {
      console.log('User not found');
    }

  } finally {
    await client.close();
  }
}

findOneDocument().catch(console.error);
```

**Find with Projection:**

```javascript
async function findWithProjection() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const query = { age: { $gte: 25 } };
    const options = {
      projection: { _id: 0, name: 1, email: 1 }
    };

    const cursor = collection.find(query, options);
    const results = await cursor.toArray();

    console.log('Users (name and email only):', results);

  } finally {
    await client.close();
  }
}

findWithProjection().catch(console.error);
```

**Find with Sort, Limit, and Skip:**

```javascript
async function findWithSortLimitSkip() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const cursor = collection.find({})
      .sort({ age: -1 })  // Sort by age descending
      .limit(10)          // Limit to 10 results
      .skip(5);           // Skip first 5 results

    const results = await cursor.toArray();

    console.log('Sorted and paginated results:', results);

  } finally {
    await client.close();
  }
}

findWithSortLimitSkip().catch(console.error);
```

**Pagination Pattern:**

```javascript
async function paginateDocuments(page = 1, pageSize = 10) {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const skip = (page - 1) * pageSize;

    const cursor = collection.find({})
      .sort({ createdAt: -1 })
      .skip(skip)
      .limit(pageSize);

    const documents = await cursor.toArray();
    const total = await collection.countDocuments({});

    return {
      documents,
      page,
      pageSize,
      totalPages: Math.ceil(total / pageSize),
      totalDocuments: total
    };

  } finally {
    await client.close();
  }
}

paginateDocuments(1, 20).then(console.log).catch(console.error);
```

**Cursor-Based Pagination (Better Performance):**

```javascript
async function cursorPagination(lastId = null, pageSize = 10) {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const query = lastId ? { _id: { $gt: lastId } } : {};

    const cursor = collection.find(query)
      .sort({ _id: 1 })
      .limit(pageSize);

    const documents = await cursor.toArray();

    return {
      documents,
      nextCursor: documents.length > 0
        ? documents[documents.length - 1]._id
        : null
    };

  } finally {
    await client.close();
  }
}

cursorPagination(null, 20).then(console.log).catch(console.error);
```

### Update Documents

**Update One Document:**

```javascript
async function updateOneDocument() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const filter = { email: 'john@example.com' };
    const update = {
      $set: { age: 31, updatedAt: new Date() }
    };

    const result = await collection.updateOne(filter, update);

    console.log(`${result.matchedCount} document(s) matched the filter`);
    console.log(`${result.modifiedCount} document(s) updated`);

  } finally {
    await client.close();
  }
}

updateOneDocument().catch(console.error);
```

**Update Multiple Documents:**

```javascript
async function updateManyDocuments() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const filter = { age: { $lt: 30 } };
    const update = {
      $set: { category: 'young', updatedAt: new Date() }
    };

    const result = await collection.updateMany(filter, update);

    console.log(`${result.matchedCount} document(s) matched`);
    console.log(`${result.modifiedCount} document(s) updated`);

  } finally {
    await client.close();
  }
}

updateManyDocuments().catch(console.error);
```

**Update with Upsert:**

```javascript
async function updateWithUpsert() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const filter = { email: 'newuser@example.com' };
    const update = {
      $set: {
        name: 'New User',
        email: 'newuser@example.com',
        age: 22,
        createdAt: new Date()
      }
    };

    const options = { upsert: true };
    const result = await collection.updateOne(filter, update, options);

    if (result.upsertedId) {
      console.log(`Document inserted with _id: ${result.upsertedId}`);
    } else {
      console.log(`${result.modifiedCount} document(s) updated`);
    }

  } finally {
    await client.close();
  }
}

updateWithUpsert().catch(console.error);
```

**Update Operators:**

```javascript
async function updateOperators() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const filter = { email: 'john@example.com' };
    const update = {
      $set: { status: 'active' },
      $inc: { loginCount: 1 },
      $push: { tags: 'premium' },
      $currentDate: { lastModified: true }
    };

    const result = await collection.updateOne(filter, update);
    console.log(`${result.modifiedCount} document(s) updated`);

  } finally {
    await client.close();
  }
}

updateOperators().catch(console.error);
```

**Replace Document:**

```javascript
async function replaceDocument() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const filter = { email: 'john@example.com' };
    const replacement = {
      name: 'John Doe Updated',
      email: 'john@example.com',
      age: 31,
      status: 'active',
      updatedAt: new Date()
    };

    const result = await collection.replaceOne(filter, replacement);
    console.log(`${result.modifiedCount} document(s) replaced`);

  } finally {
    await client.close();
  }
}

replaceDocument().catch(console.error);
```

**Find and Modify:**

```javascript
async function findAndModify() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const filter = { email: 'john@example.com' };
    const update = { $inc: { age: 1 } };
    const options = {
      returnDocument: 'after',  // Return the updated document
      upsert: false
    };

    const result = await collection.findOneAndUpdate(filter, update, options);

    if (result) {
      console.log('Updated document:', result);
    }

  } finally {
    await client.close();
  }
}

findAndModify().catch(console.error);
```

### Delete Documents

**Delete One Document:**

```javascript
async function deleteOneDocument() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const filter = { email: 'john@example.com' };
    const result = await collection.deleteOne(filter);

    console.log(`${result.deletedCount} document(s) deleted`);

  } finally {
    await client.close();
  }
}

deleteOneDocument().catch(console.error);
```

**Delete Multiple Documents:**

```javascript
async function deleteManyDocuments() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const filter = { age: { $lt: 18 } };
    const result = await collection.deleteMany(filter);

    console.log(`${result.deletedCount} document(s) deleted`);

  } finally {
    await client.close();
  }
}

deleteManyDocuments().catch(console.error);
```

**Find and Delete:**

```javascript
async function findAndDelete() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const filter = { email: 'john@example.com' };
    const options = {
      sort: { createdAt: -1 }
    };

    const result = await collection.findOneAndDelete(filter, options);

    if (result) {
      console.log('Deleted document:', result);
    } else {
      console.log('No document found to delete');
    }

  } finally {
    await client.close();
  }
}

findAndDelete().catch(console.error);
```

## Aggregation Pipeline

**Basic Aggregation:**

```javascript
async function basicAggregation() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const pipeline = [
      { $match: { age: { $gte: 25 } } },
      { $group: {
          _id: '$status',
          count: { $sum: 1 },
          avgAge: { $avg: '$age' }
        }
      },
      { $sort: { count: -1 } }
    ];

    const results = await collection.aggregate(pipeline).toArray();
    console.log('Aggregation results:', results);

  } finally {
    await client.close();
  }
}

basicAggregation().catch(console.error);
```

**Advanced Aggregation with Multiple Stages:**

```javascript
async function advancedAggregation() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('orders');

    const pipeline = [
      {
        $match: {
          status: 'completed',
          createdAt: { $gte: new Date('2024-01-01') }
        }
      },
      {
        $group: {
          _id: {
            year: { $year: '$createdAt' },
            month: { $month: '$createdAt' }
          },
          totalSales: { $sum: '$amount' },
          orderCount: { $sum: 1 },
          avgOrderValue: { $avg: '$amount' }
        }
      },
      {
        $sort: { '_id.year': 1, '_id.month': 1 }
      },
      {
        $project: {
          _id: 0,
          year: '$_id.year',
          month: '$_id.month',
          totalSales: 1,
          orderCount: 1,
          avgOrderValue: { $round: ['$avgOrderValue', 2] }
        }
      }
    ];

    const results = await collection.aggregate(pipeline).toArray();
    console.log('Sales report:', results);

  } finally {
    await client.close();
  }
}

advancedAggregation().catch(console.error);
```

**Aggregation with $lookup (Join):**

```javascript
async function aggregationWithLookup() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('orders');

    const pipeline = [
      {
        $lookup: {
          from: 'users',
          localField: 'userId',
          foreignField: '_id',
          as: 'userDetails'
        }
      },
      {
        $unwind: '$userDetails'
      },
      {
        $project: {
          orderNumber: 1,
          amount: 1,
          userName: '$userDetails.name',
          userEmail: '$userDetails.email'
        }
      }
    ];

    const results = await collection.aggregate(pipeline).toArray();
    console.log('Orders with user details:', results);

  } finally {
    await client.close();
  }
}

aggregationWithLookup().catch(console.error);
```

**Aggregation with $facet (Multiple Pipelines):**

```javascript
async function aggregationWithFacet() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('products');

    const pipeline = [
      {
        $facet: {
          categoryCounts: [
            { $group: { _id: '$category', count: { $sum: 1 } } },
            { $sort: { count: -1 } }
          ],
          priceStats: [
            {
              $group: {
                _id: null,
                avgPrice: { $avg: '$price' },
                minPrice: { $min: '$price' },
                maxPrice: { $max: '$price' }
              }
            }
          ],
          topProducts: [
            { $sort: { sales: -1 } },
            { $limit: 5 },
            { $project: { name: 1, sales: 1, price: 1 } }
          ]
        }
      }
    ];

    const results = await collection.aggregate(pipeline).toArray();
    console.log('Product analytics:', results[0]);

  } finally {
    await client.close();
  }
}

aggregationWithFacet().catch(console.error);
```

## Indexes

**Create Single Field Index:**

```javascript
async function createIndex() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const result = await collection.createIndex({ email: 1 });
    console.log(`Index created: ${result}`);

  } finally {
    await client.close();
  }
}

createIndex().catch(console.error);
```

**Create Compound Index:**

```javascript
async function createCompoundIndex() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const result = await collection.createIndex(
      { lastName: 1, firstName: 1 }
    );
    console.log(`Compound index created: ${result}`);

  } finally {
    await client.close();
  }
}

createCompoundIndex().catch(console.error);
```

**Create Unique Index:**

```javascript
async function createUniqueIndex() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const result = await collection.createIndex(
      { email: 1 },
      { unique: true }
    );
    console.log(`Unique index created: ${result}`);

  } finally {
    await client.close();
  }
}

createUniqueIndex().catch(console.error);
```

**Create Text Index:**

```javascript
async function createTextIndex() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('articles');

    const result = await collection.createIndex(
      { title: 'text', content: 'text' }
    );
    console.log(`Text index created: ${result}`);

  } finally {
    await client.close();
  }
}

createTextIndex().catch(console.error);
```

**Text Search Query:**

```javascript
async function textSearch() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('articles');

    const query = { $text: { $search: 'mongodb tutorial' } };
    const projection = { score: { $meta: 'textScore' } };

    const cursor = collection
      .find(query, { projection })
      .sort({ score: { $meta: 'textScore' } });

    const results = await cursor.toArray();
    console.log('Search results:', results);

  } finally {
    await client.close();
  }
}

textSearch().catch(console.error);
```

**Create 2dsphere Index (Geospatial):**

```javascript
async function createGeospatialIndex() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('locations');

    const result = await collection.createIndex(
      { location: '2dsphere' }
    );
    console.log(`Geospatial index created: ${result}`);

  } finally {
    await client.close();
  }
}

createGeospatialIndex().catch(console.error);
```

**Geospatial Query ($near):**

```javascript
async function geospatialNearQuery() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('locations');

    const query = {
      location: {
        $near: {
          $geometry: {
            type: 'Point',
            coordinates: [-73.9667, 40.78]  // [longitude, latitude]
          },
          $maxDistance: 5000  // 5km in meters
        }
      }
    };

    const results = await collection.find(query).limit(10).toArray();
    console.log('Nearby locations:', results);

  } finally {
    await client.close();
  }
}

geospatialNearQuery().catch(console.error);
```

**Geospatial Query ($geoWithin):**

```javascript
async function geospatialWithinQuery() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('locations');

    const query = {
      location: {
        $geoWithin: {
          $geometry: {
            type: 'Polygon',
            coordinates: [[
              [-74.0, 40.7],
              [-73.9, 40.7],
              [-73.9, 40.8],
              [-74.0, 40.8],
              [-74.0, 40.7]
            ]]
          }
        }
      }
    };

    const results = await collection.find(query).toArray();
    console.log('Locations within polygon:', results);

  } finally {
    await client.close();
  }
}

geospatialWithinQuery().catch(console.error);
```

**List Indexes:**

```javascript
async function listIndexes() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const indexes = await collection.listIndexes().toArray();
    console.log('Indexes:', indexes);

  } finally {
    await client.close();
  }
}

listIndexes().catch(console.error);
```

**Drop Index:**

```javascript
async function dropIndex() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const result = await collection.dropIndex('email_1');
    console.log('Index dropped:', result);

  } finally {
    await client.close();
  }
}

dropIndex().catch(console.error);
```

## Bulk Write Operations

**Bulk Write with Mixed Operations:**

```javascript
async function bulkWrite() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const operations = [
      {
        insertOne: {
          document: { name: 'User 1', email: 'user1@example.com' }
        }
      },
      {
        updateOne: {
          filter: { email: 'john@example.com' },
          update: { $set: { status: 'active' } }
        }
      },
      {
        updateMany: {
          filter: { age: { $lt: 25 } },
          update: { $set: { category: 'young' } }
        }
      },
      {
        deleteOne: {
          filter: { email: 'old@example.com' }
        }
      },
      {
        replaceOne: {
          filter: { email: 'replace@example.com' },
          replacement: { name: 'Replaced', email: 'replace@example.com', age: 40 }
        }
      }
    ];

    const result = await collection.bulkWrite(operations);

    console.log(`${result.insertedCount} documents inserted`);
    console.log(`${result.modifiedCount} documents modified`);
    console.log(`${result.deletedCount} documents deleted`);
    console.log(`${result.upsertedCount} documents upserted`);

  } finally {
    await client.close();
  }
}

bulkWrite().catch(console.error);
```

**Ordered vs Unordered Bulk Write:**

```javascript
async function orderedVsUnorderedBulk() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const operations = [
      { insertOne: { document: { name: 'User A' } } },
      { insertOne: { document: { name: 'User B' } } }
    ];

    // Ordered (default): stops on first error
    const orderedResult = await collection.bulkWrite(operations, { ordered: true });

    // Unordered: continues on errors, may execute in parallel
    const unorderedResult = await collection.bulkWrite(operations, { ordered: false });

    console.log('Ordered result:', orderedResult);
    console.log('Unordered result:', unorderedResult);

  } finally {
    await client.close();
  }
}

orderedVsUnorderedBulk().catch(console.error);
```

## Transactions

**Transaction with Session (Convenient API):**

```javascript
async function transactionExample() {
  const session = client.startSession();

  try {
    await client.connect();
    const database = client.db('sample_db');

    const result = await session.withTransaction(async () => {
      const accounts = database.collection('accounts');
      const transactions = database.collection('transactions');

      // Deduct from account A
      await accounts.updateOne(
        { accountId: 'A' },
        { $inc: { balance: -100 } },
        { session }
      );

      // Add to account B
      await accounts.updateOne(
        { accountId: 'B' },
        { $inc: { balance: 100 } },
        { session }
      );

      // Record transaction
      await transactions.insertOne(
        {
          from: 'A',
          to: 'B',
          amount: 100,
          timestamp: new Date()
        },
        { session }
      );

      return 'Transaction completed';
    });

    console.log(result);

  } finally {
    await session.endSession();
    await client.close();
  }
}

transactionExample().catch(console.error);
```

**Transaction with Core API (Manual Control):**

```javascript
async function manualTransaction() {
  const session = client.startSession();

  try {
    await client.connect();
    const database = client.db('sample_db');
    const accounts = database.collection('accounts');

    // Start transaction
    session.startTransaction({
      readConcern: { level: 'snapshot' },
      writeConcern: { w: 'majority' }
    });

    try {
      // Perform operations within transaction
      await accounts.updateOne(
        { accountId: 'A' },
        { $inc: { balance: -100 } },
        { session }
      );

      await accounts.updateOne(
        { accountId: 'B' },
        { $inc: { balance: 100 } },
        { session }
      );

      // Commit transaction
      await session.commitTransaction();
      console.log('Transaction committed');

    } catch (error) {
      // Abort transaction on error
      await session.abortTransaction();
      console.error('Transaction aborted:', error);
      throw error;
    }

  } finally {
    await session.endSession();
    await client.close();
  }
}

manualTransaction().catch(console.error);
```

**Transaction with Retry Logic:**

```javascript
async function transactionWithRetry() {
  const session = client.startSession();

  async function runTransactionWithRetry(txnFunc, session) {
    while (true) {
      try {
        return await txnFunc(session);
      } catch (error) {
        if (error.hasErrorLabel('TransientTransactionError')) {
          console.log('TransientTransactionError, retrying transaction...');
          continue;
        }
        throw error;
      }
    }
  }

  async function commitWithRetry(session) {
    while (true) {
      try {
        await session.commitTransaction();
        console.log('Transaction committed');
        break;
      } catch (error) {
        if (error.hasErrorLabel('UnknownTransactionCommitResult')) {
          console.log('UnknownTransactionCommitResult, retrying commit...');
          continue;
        }
        throw error;
      }
    }
  }

  try {
    await client.connect();
    const database = client.db('sample_db');

    await runTransactionWithRetry(async (session) => {
      session.startTransaction();

      const accounts = database.collection('accounts');

      await accounts.updateOne(
        { accountId: 'A' },
        { $inc: { balance: -50 } },
        { session }
      );

      await accounts.updateOne(
        { accountId: 'B' },
        { $inc: { balance: 50 } },
        { session }
      );

      await commitWithRetry(session);
    }, session);

  } finally {
    await session.endSession();
    await client.close();
  }
}

transactionWithRetry().catch(console.error);
```

## Change Streams

**Watch Collection for Changes:**

```javascript
async function watchCollection() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const changeStream = collection.watch();

    console.log('Watching for changes...');

    changeStream.on('change', (change) => {
      console.log('Change detected:', change);
    });

    changeStream.on('error', (error) => {
      console.error('Change stream error:', error);
    });

    // Keep the connection open
    // In production, you'd handle this differently

  } catch (error) {
    console.error(error);
  }
}

watchCollection().catch(console.error);
```

**Watch with Filter Pipeline:**

```javascript
async function watchWithFilter() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    const pipeline = [
      {
        $match: {
          'operationType': { $in: ['insert', 'update'] },
          'fullDocument.age': { $gte: 18 }
        }
      }
    ];

    const changeStream = collection.watch(pipeline);

    console.log('Watching for adult user changes...');

    for await (const change of changeStream) {
      console.log('Change:', change);

      if (change.operationType === 'insert') {
        console.log('New user inserted:', change.fullDocument);
      } else if (change.operationType === 'update') {
        console.log('User updated:', change.documentKey);
      }
    }

  } catch (error) {
    console.error(error);
  }
}

watchWithFilter().catch(console.error);
```

**Watch Database for Changes:**

```javascript
async function watchDatabase() {
  try {
    await client.connect();
    const database = client.db('sample_db');

    const changeStream = database.watch();

    console.log('Watching database for changes...');

    changeStream.on('change', (change) => {
      console.log(`Change in collection ${change.ns.coll}:`, change);
    });

  } catch (error) {
    console.error(error);
  }
}

watchDatabase().catch(console.error);
```

## ObjectId Utilities

**Working with ObjectId:**

```javascript
import { ObjectId } from 'mongodb';

async function objectIdExamples() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    // Generate new ObjectId
    const newId = new ObjectId();
    console.log('New ObjectId:', newId.toString());

    // Insert with custom ObjectId
    await collection.insertOne({
      _id: new ObjectId(),
      name: 'User with custom ID'
    });

    // Find by ObjectId string
    const userId = '507f1f77bcf86cd799439011';
    const user = await collection.findOne({
      _id: new ObjectId(userId)
    });

    // Get timestamp from ObjectId
    if (user) {
      const timestamp = user._id.getTimestamp();
      console.log('Document created at:', timestamp);
    }

    // Validate ObjectId
    const isValid = ObjectId.isValid('507f1f77bcf86cd799439011');
    console.log('Is valid ObjectId:', isValid);

  } finally {
    await client.close();
  }
}

objectIdExamples().catch(console.error);
```

## Error Handling

**Comprehensive Error Handling:**

```javascript
import { MongoClient, MongoServerError } from 'mongodb';

async function errorHandlingExample() {
  const client = new MongoClient(process.env.MONGODB_URI);

  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    // Attempt operation
    await collection.insertOne({
      email: 'duplicate@example.com'
    });

  } catch (error) {
    if (error instanceof MongoServerError) {
      switch (error.code) {
        case 11000:
          console.error('Duplicate key error:', error.message);
          break;
        case 121:
          console.error('Document validation failed:', error.message);
          break;
        default:
          console.error('MongoDB server error:', error.message);
      }
    } else if (error.name === 'MongoNetworkError') {
      console.error('Network error - cannot connect to MongoDB:', error.message);
    } else if (error.name === 'MongoParseError') {
      console.error('Invalid connection string:', error.message);
    } else {
      console.error('Unexpected error:', error);
    }
  } finally {
    await client.close();
  }
}

errorHandlingExample().catch(console.error);
```

**Retry Logic for Transient Errors:**

```javascript
async function retryOperation(operation, maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      if (attempt === maxRetries) {
        throw error;
      }

      const isRetryable =
        error.name === 'MongoNetworkError' ||
        error.code === 'ETIMEDOUT' ||
        error.code === 'ECONNRESET';

      if (!isRetryable) {
        throw error;
      }

      const delay = Math.pow(2, attempt) * 1000;
      console.log(`Attempt ${attempt} failed, retrying in ${delay}ms...`);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
}

async function useRetryLogic() {
  const client = new MongoClient(process.env.MONGODB_URI);

  try {
    await retryOperation(async () => {
      await client.connect();
      const database = client.db('sample_db');
      const collection = database.collection('users');

      return await collection.findOne({ email: 'test@example.com' });
    });

  } finally {
    await client.close();
  }
}

useRetryLogic().catch(console.error);
```

## Query Operators

**Comparison Operators:**

```javascript
async function comparisonOperators() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('products');

    // $eq (equal)
    const equal = await collection.find({ price: { $eq: 99 } }).toArray();

    // $ne (not equal)
    const notEqual = await collection.find({ status: { $ne: 'discontinued' } }).toArray();

    // $gt, $gte (greater than, greater than or equal)
    const greaterThan = await collection.find({ price: { $gt: 50 } }).toArray();
    const greaterOrEqual = await collection.find({ stock: { $gte: 100 } }).toArray();

    // $lt, $lte (less than, less than or equal)
    const lessThan = await collection.find({ price: { $lt: 100 } }).toArray();
    const lessOrEqual = await collection.find({ rating: { $lte: 3 } }).toArray();

    // $in (in array)
    const inArray = await collection.find({
      category: { $in: ['electronics', 'computers'] }
    }).toArray();

    // $nin (not in array)
    const notInArray = await collection.find({
      status: { $nin: ['discontinued', 'out-of-stock'] }
    }).toArray();

    console.log('Query results:', equal, notEqual, greaterThan);

  } finally {
    await client.close();
  }
}

comparisonOperators().catch(console.error);
```

**Logical Operators:**

```javascript
async function logicalOperators() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('products');

    // $and
    const andQuery = await collection.find({
      $and: [
        { price: { $lt: 100 } },
        { stock: { $gt: 0 } }
      ]
    }).toArray();

    // $or
    const orQuery = await collection.find({
      $or: [
        { category: 'electronics' },
        { featured: true }
      ]
    }).toArray();

    // $not
    const notQuery = await collection.find({
      price: { $not: { $gt: 100 } }
    }).toArray();

    // $nor
    const norQuery = await collection.find({
      $nor: [
        { status: 'discontinued' },
        { stock: 0 }
      ]
    }).toArray();

    console.log('Logical query results:', andQuery.length, orQuery.length);

  } finally {
    await client.close();
  }
}

logicalOperators().catch(console.error);
```

**Element Operators:**

```javascript
async function elementOperators() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    // $exists
    const hasPhone = await collection.find({
      phone: { $exists: true }
    }).toArray();

    // $type
    const stringEmails = await collection.find({
      email: { $type: 'string' }
    }).toArray();

    console.log('Element query results:', hasPhone.length, stringEmails.length);

  } finally {
    await client.close();
  }
}

elementOperators().catch(console.error);
```

**Array Operators:**

```javascript
async function arrayOperators() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    // $all
    const allTags = await collection.find({
      tags: { $all: ['premium', 'verified'] }
    }).toArray();

    // $elemMatch
    const elemMatch = await collection.find({
      scores: { $elemMatch: { $gte: 80, $lt: 90 } }
    }).toArray();

    // $size
    const exactSize = await collection.find({
      tags: { $size: 3 }
    }).toArray();

    console.log('Array query results:', allTags.length, elemMatch.length);

  } finally {
    await client.close();
  }
}

arrayOperators().catch(console.error);
```

## Advanced Patterns

**Connection Pooling:**

```javascript
import { MongoClient } from 'mongodb';

const uri = process.env.MONGODB_URI;

const client = new MongoClient(uri, {
  maxPoolSize: 50,
  minPoolSize: 10,
  maxIdleTimeMS: 30000,
  waitQueueTimeoutMS: 5000
});

let isConnected = false;

async function getDatabase() {
  if (!isConnected) {
    await client.connect();
    isConnected = true;
  }
  return client.db('sample_db');
}

export { getDatabase, client };
```

**Database and Collection Management:**

```javascript
async function databaseManagement() {
  try {
    await client.connect();

    // List databases
    const adminDb = client.db().admin();
    const dbList = await adminDb.listDatabases();
    console.log('Databases:', dbList.databases);

    // List collections
    const database = client.db('sample_db');
    const collections = await database.listCollections().toArray();
    console.log('Collections:', collections);

    // Create collection with options
    await database.createCollection('newCollection', {
      validator: {
        $jsonSchema: {
          bsonType: 'object',
          required: ['name', 'email'],
          properties: {
            name: {
              bsonType: 'string',
              description: 'must be a string and is required'
            },
            email: {
              bsonType: 'string',
              pattern: '^.+@.+$',
              description: 'must be a valid email'
            }
          }
        }
      }
    });

    // Drop collection
    // await database.collection('oldCollection').drop();

  } finally {
    await client.close();
  }
}

databaseManagement().catch(console.error);
```

**Schema Validation:**

```javascript
async function addSchemaValidation() {
  try {
    await client.connect();
    const database = client.db('sample_db');

    await database.command({
      collMod: 'users',
      validator: {
        $jsonSchema: {
          bsonType: 'object',
          required: ['name', 'email', 'age'],
          properties: {
            name: {
              bsonType: 'string',
              description: 'must be a string and is required'
            },
            email: {
              bsonType: 'string',
              pattern: '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$',
              description: 'must be a valid email and is required'
            },
            age: {
              bsonType: 'int',
              minimum: 0,
              maximum: 150,
              description: 'must be an integer between 0 and 150'
            },
            status: {
              enum: ['active', 'inactive', 'suspended'],
              description: 'can only be one of the enum values'
            }
          }
        }
      },
      validationLevel: 'moderate',
      validationAction: 'error'
    });

    console.log('Schema validation added');

  } finally {
    await client.close();
  }
}

addSchemaValidation().catch(console.error);
```

**Time Series Collections:**

```javascript
async function createTimeSeriesCollection() {
  try {
    await client.connect();
    const database = client.db('sample_db');

    await database.createCollection('sensor_data', {
      timeseries: {
        timeField: 'timestamp',
        metaField: 'sensorId',
        granularity: 'seconds'
      }
    });

    const collection = database.collection('sensor_data');

    // Insert time series data
    await collection.insertMany([
      {
        sensorId: 'sensor1',
        timestamp: new Date('2024-01-01T00:00:00Z'),
        temperature: 22.5,
        humidity: 60
      },
      {
        sensorId: 'sensor1',
        timestamp: new Date('2024-01-01T00:01:00Z'),
        temperature: 22.7,
        humidity: 59
      }
    ]);

    console.log('Time series data inserted');

  } finally {
    await client.close();
  }
}

createTimeSeriesCollection().catch(console.error);
```

**Read and Write Concerns:**

```javascript
async function readWriteConcerns() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    // Write concern
    const insertResult = await collection.insertOne(
      { name: 'John', email: 'john@example.com' },
      {
        writeConcern: {
          w: 'majority',
          j: true,
          wtimeout: 5000
        }
      }
    );

    // Read concern
    const findResult = await collection.findOne(
      { email: 'john@example.com' },
      {
        readConcern: { level: 'majority' }
      }
    );

    console.log('Insert result:', insertResult);
    console.log('Find result:', findResult);

  } finally {
    await client.close();
  }
}

readWriteConcerns().catch(console.error);
```

**Count Documents:**

```javascript
async function countExamples() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    // Count all documents
    const totalCount = await collection.countDocuments({});
    console.log('Total documents:', totalCount);

    // Count with filter
    const activeCount = await collection.countDocuments({ status: 'active' });
    console.log('Active users:', activeCount);

    // Estimated count (faster but less accurate)
    const estimatedCount = await collection.estimatedDocumentCount();
    console.log('Estimated count:', estimatedCount);

  } finally {
    await client.close();
  }
}

countExamples().catch(console.error);
```

**Distinct Values:**

```javascript
async function distinctValues() {
  try {
    await client.connect();
    const database = client.db('sample_db');
    const collection = database.collection('users');

    // Get distinct values
    const cities = await collection.distinct('city');
    console.log('Distinct cities:', cities);

    // Get distinct values with filter
    const activeCities = await collection.distinct('city', { status: 'active' });
    console.log('Cities with active users:', activeCities);

  } finally {
    await client.close();
  }
}

distinctValues().catch(console.error);
```
