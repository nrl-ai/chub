---
name: sdk
description: "Vector database for AI applications with semantic search, hybrid search, reranking, and integrated embeddings"
metadata:
  languages: "javascript"
  versions: "6.1.2"
  updated-on: "2025-10-26"
  source: maintainer
  tags: "pinecone,sdk,vector-db,ai,search"
---
# Pinecone JavaScript/TypeScript SDK Coding Guidelines

You are a Pinecone vector database coding expert. Help me with writing code using the Pinecone SDK for JavaScript/TypeScript.

You can find the official SDK documentation and code samples here:
https://docs.pinecone.io/

## Golden Rule: Use the Correct and Current SDK

Always use the official Pinecone TypeScript SDK for all Pinecone vector database interactions. Do not use legacy libraries or unofficial packages.

- **Library Name:** Pinecone TypeScript SDK
- **NPM Package:** `@pinecone-database/pinecone`
- **Legacy Libraries**: `pinecone-client` and other unofficial packages are not recommended

**Installation:**

- **Correct:** `npm install @pinecone-database/pinecone`

**APIs and Usage:**

- **Correct:** `import { Pinecone } from '@pinecone-database/pinecone'`
- **Correct:** `const pc = new Pinecone({})`
- **Correct:** `await pc.createIndex(...)`
- **Correct:** `pc.index('index-name')`
- **Incorrect:** `PineconeClient`
- **Incorrect:** Legacy initialization patterns

**Important Security Note:**

The Pinecone TypeScript SDK is intended for server-side use only. Using the SDK within a browser context can expose your API key(s). Always use server-side code or proxy requests through a backend.

## Installation

Install the Pinecone SDK using npm:

```bash
npm install @pinecone-database/pinecone
```

**Requirements:**
- TypeScript >=4.1
- Node.js >=18.x

## Initialization and API Key

The SDK requires creating a `Pinecone` instance for all API calls.

### Using Environment Variables

Set your API key as an environment variable:

```bash
export PINECONE_API_KEY="your_api_key_here"
```

Then initialize without arguments:

```typescript
import { Pinecone } from '@pinecone-database/pinecone';

const pc = new Pinecone();
```

### Using Configuration Object

Pass credentials directly:

```typescript
import { Pinecone } from '@pinecone-database/pinecone';

const pc = new Pinecone({
  apiKey: 'your_api_key_here',
});
```

### With Custom Retry Configuration

```typescript
import { Pinecone } from '@pinecone-database/pinecone';

const pc = new Pinecone({
  apiKey: process.env.PINECONE_API_KEY,
  maxRetries: 5,
});
```

The `maxRetries` parameter (defaults to 3) applies to operations like `upsert`, `update`, and `configureIndex`.

## Creating Indexes

### Serverless Index (Basic)

```typescript
import { Pinecone } from '@pinecone-database/pinecone';

const pc = new Pinecone();

await pc.createIndex({
  name: 'my-index',
  dimension: 1536,
  metric: 'cosine',
  spec: {
    serverless: {
      cloud: 'aws',
      region: 'us-west-2',
    },
  },
});
```

### Serverless Index with Wait

Use `waitUntilReady` to block until the index is operational:

```typescript
await pc.createIndex({
  name: 'serverless-index',
  dimension: 1536,
  metric: 'cosine',
  spec: {
    serverless: {
      cloud: 'aws',
      region: 'us-east-1',
    },
  },
  waitUntilReady: true,
});
```

### Pod-Based Index

```typescript
await pc.createIndex({
  name: 'pod-index',
  dimension: 1536,
  metric: 'dotproduct',
  spec: {
    pod: {
      environment: 'us-west-2',
      podType: 'p1.x1',
      pods: 1,
      replicas: 1,
    },
  },
});
```

### Hybrid Search Index (Sparse-Dense)

For hybrid search supporting both dense and sparse vectors, use `dotproduct` metric:

```typescript
await pc.createIndex({
  name: 'hybrid-index',
  dimension: 1024,
  metric: 'dotproduct',
  spec: {
    serverless: {
      cloud: 'aws',
      region: 'us-east-1',
    },
  },
});
```

**Note:** The `dotproduct` metric is the only metric that supports sparse-dense hybrid search.

## Index Management

### Describe Index

Get index details including status, dimension, and host:

```typescript
const indexStats = await pc.describeIndex('my-index');
console.log(indexStats);
```

### List All Indexes

```typescript
const indexList = await pc.listIndexes();
console.log(indexList.indexes);
```

### Delete Index

```typescript
await pc.deleteIndex('my-index');
```

### Configure Index (Scale)

For pod-based indexes, adjust replicas and pod type:

```typescript
await pc.configureIndex('pod-index', {
  spec: {
    pod: {
      replicas: 2,
      podType: 'p1.x2',
    },
  },
});
```

## Connecting to an Index

### Basic Connection

```typescript
import { Pinecone } from '@pinecone-database/pinecone';

const pc = new Pinecone();
const index = pc.index('my-index');
```

### Connection with Namespace

```typescript
const pc = new Pinecone();
const index = pc.index('my-index').namespace('my-namespace');
```

### Connection with Host (Faster)

If you know the index host, connect directly:

```typescript
const pc = new Pinecone();
const index = pc.index('my-index', 'https://my-index-abc123.svc.pinecone.io');
```

## Upserting Vectors

### Dense Vectors (Basic)

```typescript
const pc = new Pinecone();
const index = pc.index('my-index');

await index.upsert([
  {
    id: 'vec1',
    values: [0.1, 0.2, 0.3, 0.4, 0.5],
  },
  {
    id: 'vec2',
    values: [0.2, 0.3, 0.4, 0.5, 0.6],
  },
]);
```

### Dense Vectors with Metadata

```typescript
await index.upsert([
  {
    id: 'vec1',
    values: [0.1, 0.2, 0.3, 0.4, 0.5],
    metadata: {
      genre: 'comedy',
      year: 2020,
      title: 'Movie Title',
    },
  },
  {
    id: 'vec2',
    values: [0.2, 0.3, 0.4, 0.5, 0.6],
    metadata: {
      genre: 'action',
      year: 2021,
      title: 'Another Movie',
    },
  },
]);
```

### Namespaced Upsert

```typescript
const pc = new Pinecone();
const ns = pc.index('my-index').namespace('example-namespace');

await ns.upsert([
  {
    id: 'vec1',
    values: [0.1, 0.2, 0.3],
    metadata: { category: 'A' },
  },
]);
```

### Batch Upsert (Large Datasets)

For upserting many vectors, batch them in groups of up to 1,000 records (max 2 MB per batch):

```typescript
const batchSize = 1000;
const vectors = []; // Your vector array

for (let i = 0; i < vectors.length; i += batchSize) {
  const batch = vectors.slice(i, i + batchSize);
  await index.upsert(batch);
}
```

### Sparse-Dense Vectors (Hybrid Search)

```typescript
await index.upsert([
  {
    id: 'vec1',
    values: [0.1, 0.2, 0.3], // Dense vector
    sparseValues: {
      indices: [10, 45, 16],
      values: [0.5, 0.5, 0.2],
    },
    metadata: { text: 'original text content' },
  },
]);
```

## Querying Vectors

### Basic Query (Dense)

```typescript
const pc = new Pinecone();
const index = pc.index('my-index');

const queryResponse = await index.query({
  vector: [0.1, 0.2, 0.3, 0.4, 0.5],
  topK: 10,
  includeMetadata: true,
  includeValues: false,
});

console.log(queryResponse.matches);
```

### Query with Metadata Filter

```typescript
const queryResponse = await index.query({
  vector: [0.1, 0.2, 0.3, 0.4, 0.5],
  topK: 5,
  filter: {
    genre: { $eq: 'comedy' },
  },
  includeMetadata: true,
});
```

### Complex Metadata Filters

```typescript
// Multiple conditions with $and
const response = await index.query({
  vector: [0.1, 0.2, 0.3],
  topK: 10,
  filter: {
    $and: [
      { genre: { $eq: 'comedy' } },
      { year: { $gte: 2020 } },
    ],
  },
  includeMetadata: true,
});

// Using $or operator
const response2 = await index.query({
  vector: [0.1, 0.2, 0.3],
  topK: 10,
  filter: {
    $or: [
      { genre: { $eq: 'comedy' } },
      { genre: { $eq: 'drama' } },
    ],
  },
  includeMetadata: true,
});

// Using $in for multiple values
const response3 = await index.query({
  vector: [0.1, 0.2, 0.3],
  topK: 10,
  filter: {
    genre: { $in: ['comedy', 'action', 'drama'] },
  },
  includeMetadata: true,
});
```

### Query by ID

```typescript
const queryResponse = await index.query({
  id: 'vec1',
  topK: 10,
  includeMetadata: true,
});
```

### Namespaced Query

```typescript
const ns = pc.index('my-index').namespace('example-namespace');

const queryResponse = await ns.query({
  vector: [0.1, 0.2, 0.3],
  topK: 5,
  includeMetadata: true,
});
```

### Hybrid Query (Sparse-Dense)

```typescript
const queryResponse = await index.query({
  vector: [0.1, 0.2, 0.3], // Dense query vector
  sparseVector: {
    indices: [10, 45, 16],
    values: [0.5, 0.5, 0.2],
  },
  topK: 10,
  includeMetadata: true,
});
```

## Fetching Vectors

### Fetch by IDs

```typescript
const pc = new Pinecone();
const index = pc.index('my-index');

const fetchResponse = await index.fetch(['vec1', 'vec2', 'vec3']);
console.log(fetchResponse.records);
```

### Fetch from Namespace

```typescript
const ns = pc.index('my-index').namespace('example-namespace');
const fetchResponse = await ns.fetch(['vec1', 'vec2']);
console.log(fetchResponse.records);
```

## Updating Vectors

### Update Values

```typescript
const pc = new Pinecone();
const index = pc.index('my-index');

await index.update({
  id: 'vec1',
  values: [0.9, 0.8, 0.7, 0.6, 0.5],
});
```

### Update Metadata

```typescript
await index.update({
  id: 'vec1',
  metadata: {
    genre: 'documentary',
    year: 2023,
  },
});
```

### Update with Namespace

```typescript
const ns = pc.index('my-index').namespace('example-namespace');

await ns.update({
  id: 'vec1',
  values: [0.1, 0.2, 0.3],
  metadata: { updated: true },
});
```

## Deleting Vectors

### Delete by IDs

```typescript
const pc = new Pinecone();
const index = pc.index('my-index');

await index.deleteOne('vec1');

// Delete multiple
await index.deleteMany(['vec1', 'vec2', 'vec3']);
```

### Delete with Metadata Filter

```typescript
await index.deleteMany({
  filter: {
    genre: { $eq: 'comedy' },
  },
});
```

### Delete All in Namespace

```typescript
const ns = pc.index('my-index').namespace('example-namespace');
await ns.deleteAll();
```

**Warning:** Deleting all records from a namespace will also delete the namespace itself. This operation is irreversible.

### Delete from Specific Namespace

```typescript
const index = pc.index('my-index');

await index.namespace('example-namespace').deleteMany(['vec1', 'vec2']);
```

## Index Statistics

### Get Index Stats

```typescript
const pc = new Pinecone();
const index = pc.index('my-index');

const stats = await index.describeIndexStats();
console.log(stats);
```

Returns information including:
- Total vector count
- Dimension
- Index fullness
- Namespaces with vector counts

### Get Stats with Filter

```typescript
const stats = await index.describeIndexStats({
  filter: {
    genre: { $eq: 'comedy' },
  },
});
```

## Integrated Inference (Embeddings)

Pinecone provides hosted embedding models through the Inference API.

### Generate Embeddings

```typescript
import { Pinecone } from '@pinecone-database/pinecone';

const pc = new Pinecone();

// Embed documents
const embeddings = await pc.inference.embed({
  model: 'multilingual-e5-large',
  inputs: [
    { text: 'Turkey is a classic meat to eat at American Thanksgiving.' },
    { text: 'Many people enjoy the beautiful mosques in Turkey.' },
  ],
  parameters: {
    inputType: 'passage',
    truncate: 'END',
  },
});

console.log(embeddings);
```

### Embed Queries

```typescript
const pc = new Pinecone();

const queryEmbedding = await pc.inference.embed({
  model: 'multilingual-e5-large',
  inputs: [{ text: 'How should I prepare my turkey?' }],
  parameters: {
    inputType: 'query',
    truncate: 'END',
  },
});

// Use the embedding for querying
const index = pc.index('my-index');
const results = await index.query({
  vector: queryEmbedding[0].values,
  topK: 10,
  includeMetadata: true,
});
```

### Available Embedding Models

- `multilingual-e5-large`: Efficient dense embedding model trained on multilingual datasets
- `pinecone-sparse-english-v0`: Sparse embedding model for keyword or hybrid semantic/keyword search

## Integrated Records (Auto-Embedding)

For indexes with integrated embedding, use `upsertRecords` to automatically convert text to vectors.

### Upsert with Auto-Embedding

```typescript
const pc = new Pinecone();
const index = pc.index('integrated-index').namespace('my-namespace');

await index.upsertRecords([
  {
    id: 'rec1',
    chunk_text: "Apple's first product was the Apple I computer.",
    metadata: { category: 'tech' },
  },
  {
    id: 'rec2',
    chunk_text: 'The company was founded in 1976.',
    metadata: { category: 'history' },
  },
]);
```

### Query with Integrated Embedding

```typescript
const results = await index.queryRecords({
  query: 'Tell me about Apple computers',
  topK: 5,
  filter: { category: { $eq: 'tech' } },
  includeMetadata: true,
});

console.log(results.matches);
```

## Namespaces

Namespaces partition records within an index and enable multitenancy.

### Create Namespace (Implicit)

Namespaces are created automatically when you upsert to them:

```typescript
const ns1 = pc.index('my-index').namespace('customer-1');
await ns1.upsert([{ id: 'vec1', values: [0.1, 0.2, 0.3] }]);

const ns2 = pc.index('my-index').namespace('customer-2');
await ns2.upsert([{ id: 'vec1', values: [0.4, 0.5, 0.6] }]);
```

### List Namespaces

```typescript
const stats = await index.describeIndexStats();
console.log(Object.keys(stats.namespaces));
```

### Default Namespace

If no namespace is specified, vectors are stored in the `"__default__"` namespace:

```typescript
const index = pc.index('my-index');
// This uses the default namespace
await index.upsert([{ id: 'vec1', values: [0.1, 0.2, 0.3] }]);
```

## Reranking

Use Pinecone's reranking models to reorder results by relevance.

### Rerank Results

```typescript
import { Pinecone } from '@pinecone-database/pinecone';

const pc = new Pinecone();

const reranked = await pc.inference.rerank({
  model: 'bge-reranker-v2-m3',
  query: 'What is the capital of France?',
  documents: [
    { id: 'doc1', text: 'Paris is the capital of France.' },
    { id: 'doc2', text: 'London is the capital of England.' },
    { id: 'doc3', text: 'Berlin is the capital of Germany.' },
  ],
  topN: 2,
  returnDocuments: true,
});

console.log(reranked);
```

## Collections (Backups)

Collections are static snapshots of an index that can be used to create new indexes.

### Create Collection

```typescript
const pc = new Pinecone();

await pc.createCollection({
  name: 'my-collection',
  source: 'my-index',
});
```

### List Collections

```typescript
const collections = await pc.listCollections();
console.log(collections);
```

### Create Index from Collection

```typescript
await pc.createIndex({
  name: 'new-index-from-backup',
  dimension: 1536,
  metric: 'cosine',
  spec: {
    serverless: {
      cloud: 'aws',
      region: 'us-west-2',
    },
  },
  sourceCollection: 'my-collection',
});
```

### Delete Collection

```typescript
await pc.deleteCollection('my-collection');
```

## Error Handling

### Basic Error Handling

```typescript
import { Pinecone, PineconeConnectionError } from '@pinecone-database/pinecone';

const pc = new Pinecone();
const index = pc.index('my-index');

try {
  await index.query({
    vector: [0.1, 0.2, 0.3],
    topK: 10,
  });
} catch (error) {
  if (error instanceof PineconeConnectionError) {
    console.error('Connection error:', error.message);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

### Retry Configuration

```typescript
const pc = new Pinecone({
  apiKey: process.env.PINECONE_API_KEY,
  maxRetries: 5, // Increase retries for operations
});
```

## Performance Best Practices

### Batch Operations

Always batch upsert operations for better throughput:

```typescript
// Good: Batch upsert
const vectors = generateVectors(1000);
await index.upsert(vectors);

// Bad: Individual upserts
for (const vector of vectors) {
  await index.upsert([vector]); // Don't do this
}
```

### Parallel Requests

Use Promise.all for independent parallel operations:

```typescript
const [stats, listResult, queryResult] = await Promise.all([
  index.describeIndexStats(),
  pc.listIndexes(),
  index.query({ vector: [0.1, 0.2, 0.3], topK: 5 }),
]);
```

### Connection Reuse

Reuse the index connection instead of recreating it:

```typescript
// Good: Create once, reuse
const index = pc.index('my-index');
for (let i = 0; i < 100; i++) {
  await index.query({ vector: getVector(i), topK: 5 });
}

// Bad: Recreate each time
for (let i = 0; i < 100; i++) {
  const index = pc.index('my-index'); // Don't do this
  await index.query({ vector: getVector(i), topK: 5 });
}
```

## Metadata Filtering Operators

Pinecone supports MongoDB-style query operators for metadata filtering:

- `$eq`: Equal to
- `$ne`: Not equal to
- `$gt`: Greater than
- `$gte`: Greater than or equal to
- `$lt`: Less than
- `$lte`: Less than or equal to
- `$in`: In array
- `$nin`: Not in array
- `$exists`: Field exists
- `$and`: Logical AND
- `$or`: Logical OR

### Example: Complex Filter

```typescript
const results = await index.query({
  vector: [0.1, 0.2, 0.3],
  topK: 10,
  filter: {
    $and: [
      { year: { $gte: 2020, $lte: 2023 } },
      {
        $or: [
          { genre: { $eq: 'comedy' } },
          { genre: { $eq: 'drama' } },
        ],
      },
      { rating: { $exists: true } },
    ],
  },
  includeMetadata: true,
});
```

## Limits and Constraints

- **Max vectors per upsert**: 1,000 records (or 2 MB per batch)
- **Max vectors per upsert (with text)**: 96 records
- **Metadata size per vector**: 40 KB max
- **Record ID length**: 512 characters max
- **Dense vector dimensions**: Up to 20,000
- **Sparse vector non-zero values**: 2,048 max
- **Top-K query limit**: 10,000

## Common Patterns

### Pattern: Upsert and Query Flow

```typescript
import { Pinecone } from '@pinecone-database/pinecone';

const pc = new Pinecone();

// Create index
await pc.createIndex({
  name: 'example-index',
  dimension: 1536,
  metric: 'cosine',
  spec: {
    serverless: { cloud: 'aws', region: 'us-west-2' },
  },
  waitUntilReady: true,
});

// Connect and upsert
const index = pc.index('example-index');
await index.upsert([
  {
    id: 'doc1',
    values: [0.1, 0.2, 0.3], // ... 1536 dimensions
    metadata: { title: 'Document 1', category: 'A' },
  },
]);

// Query
const results = await index.query({
  vector: [0.1, 0.2, 0.3], // ... 1536 dimensions
  topK: 5,
  includeMetadata: true,
});

console.log(results.matches);
```

### Pattern: Multi-Tenant with Namespaces

```typescript
const pc = new Pinecone();
const index = pc.index('multi-tenant-index');

// Tenant 1
const tenant1 = index.namespace('tenant-1');
await tenant1.upsert([
  { id: 'vec1', values: [0.1, 0.2, 0.3], metadata: { user: 'user1' } },
]);

// Tenant 2
const tenant2 = index.namespace('tenant-2');
await tenant2.upsert([
  { id: 'vec1', values: [0.4, 0.5, 0.6], metadata: { user: 'user2' } },
]);

// Query tenant-specific data
const tenant1Results = await tenant1.query({
  vector: [0.1, 0.2, 0.3],
  topK: 5,
});
```

### Pattern: Hybrid Search with Weighting

```typescript
// Alpha controls dense vs sparse weighting
// alpha=1 is pure dense, alpha=0 is pure sparse
const alpha = 0.7;

const denseVector = [0.1, 0.2, 0.3];
const sparseVector = {
  indices: [10, 45, 16],
  values: [0.5, 0.5, 0.2],
};

// Apply weighting
const weightedDense = denseVector.map(v => v * alpha);
const weightedSparse = {
  indices: sparseVector.indices,
  values: sparseVector.values.map(v => v * (1 - alpha)),
};

const results = await index.query({
  vector: weightedDense,
  sparseVector: weightedSparse,
  topK: 10,
  includeMetadata: true,
});
```
