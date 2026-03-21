---
name: vector-db
description: "Weaviate TypeScript/JavaScript SDK (v3) for vector database operations and semantic search"
metadata:
  languages: "javascript"
  versions: "3.9.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "weaviate,vector-db,search,embeddings,ai"
---

# Weaviate TypeScript/JavaScript SDK (v3)

## Golden Rule

**ALWAYS use the official `weaviate-client` package (v3.x).**

```bash
npm install weaviate-client
```

**NEVER use:**
- `weaviate-ts-client` (deprecated)
- The v2 client (deprecated as of 2024)
- Unofficial or outdated packages

The v3 client uses gRPC for 60-80% faster performance, first-class TypeScript support, and modern ES Modules.

---

## Installation

### Install the Client

```bash
npm install weaviate-client
```

### Environment Variables

Create a `.env` file:

```env
WEAVIATE_URL=https://your-instance.weaviate.network
WEAVIATE_API_KEY=your-api-key-here
OPENAI_API_KEY=your-openai-key-here
```

---

## Initialization

### Connect to Weaviate Cloud

```typescript
import weaviate, { WeaviateClient } from 'weaviate-client';

const client: WeaviateClient = await weaviate.connectToWeaviateCloud(
  process.env.WEAVIATE_URL!,
  {
    authCredentials: new weaviate.ApiKey(process.env.WEAVIATE_API_KEY!),
    headers: {
      'X-OpenAI-Api-Key': process.env.OPENAI_API_KEY!,
    }
  }
);

console.log('Connected to Weaviate');
```

### Connect to Local Instance

```typescript
import weaviate from 'weaviate-client';

const client = await weaviate.connectToLocal({
  host: 'localhost',
  port: 8080,
  grpcPort: 50051,
  headers: {
    'X-OpenAI-Api-Key': process.env.OPENAI_API_KEY!,
  }
});
```

### Custom Connection

```typescript
import weaviate from 'weaviate-client';

const client = await weaviate.connectToCustom({
  httpHost: 'weaviate.example.com',
  httpPort: 443,
  httpSecure: true,
  grpcHost: 'grpc.weaviate.example.com',
  grpcPort: 443,
  grpcSecure: true,
  authCredentials: new weaviate.ApiKey(process.env.WEAVIATE_API_KEY!),
  headers: {
    'X-OpenAI-Api-Key': process.env.OPENAI_API_KEY!,
  }
});
```

### Close Connection

```typescript
await client.close();
```

---

## Collections

### Create a Collection

#### Basic Collection

```typescript
import { dataType } from 'weaviate-client';

await client.collections.create({
  name: 'Article',
  properties: [
    { name: 'title', dataType: dataType.TEXT },
    { name: 'body', dataType: dataType.TEXT },
    { name: 'author', dataType: dataType.TEXT },
    { name: 'publishDate', dataType: dataType.DATE },
    { name: 'viewCount', dataType: dataType.INT },
  ],
});
```

#### Collection with Vectorizer

```typescript
import { configure, dataType } from 'weaviate-client';

await client.collections.create({
  name: 'Article',
  vectorizers: configure.vectorizer.text2VecOpenAI({
    model: 'text-embedding-3-small',
  }),
  properties: [
    { name: 'title', dataType: dataType.TEXT },
    { name: 'body', dataType: dataType.TEXT },
    { name: 'author', dataType: dataType.TEXT },
  ],
});
```

#### Collection with Multiple Named Vectors

```typescript
import { configure, dataType } from 'weaviate-client';

await client.collections.create({
  name: 'MultiModalArticle',
  properties: [
    { name: 'title', dataType: dataType.TEXT },
    { name: 'body', dataType: dataType.TEXT },
    { name: 'image', dataType: dataType.BLOB },
  ],
  vectorizers: [
    configure.namedVectors.text2VecOpenAI('text_vector', {
      sourceProperties: ['title', 'body'],
    }),
    configure.namedVectors.multi2VecClip('image_vector', {
      imageFields: ['image'],
    }),
  ],
});
```

#### Collection with Generative Module

```typescript
import { configure, dataType } from 'weaviate-client';

await client.collections.create({
  name: 'Article',
  vectorizers: configure.vectorizer.text2VecOpenAI(),
  generative: configure.generative.openAI({
    model: 'gpt-4',
  }),
  properties: [
    { name: 'title', dataType: dataType.TEXT },
    { name: 'body', dataType: dataType.TEXT },
  ],
});
```

### List All Collections

```typescript
const collections = await client.collections.listAll();
console.log(collections);
```

### Get a Collection Reference

```typescript
const articles = client.collections.get('Article');
```

### Delete a Collection

```typescript
await client.collections.delete('Article');
```

### Check if Collection Exists

```typescript
const exists = await client.collections.exists('Article');
console.log(`Collection exists: ${exists}`);
```

---

## Data Types

### Available Data Types

```typescript
import { dataType } from 'weaviate-client';

// Text types
dataType.TEXT          // Single text value
dataType.TEXT_ARRAY    // Array of text values

// Numeric types
dataType.NUMBER        // Float/double
dataType.INT           // Integer
dataType.INT_ARRAY     // Array of integers
dataType.NUMBER_ARRAY  // Array of numbers

// Boolean
dataType.BOOLEAN       // True/false
dataType.BOOLEAN_ARRAY // Array of booleans

// Date and UUID
dataType.DATE          // ISO 8601 date-time
dataType.DATE_ARRAY    // Array of dates
dataType.UUID          // UUID
dataType.UUID_ARRAY    // Array of UUIDs

// Binary data
dataType.BLOB          // Base64 encoded binary data
dataType.BLOB_ARRAY    // Array of blobs

// Geolocation
dataType.GEO_COORDINATES // { latitude: number, longitude: number }

// Object reference
dataType.OBJECT        // Nested object
dataType.OBJECT_ARRAY  // Array of nested objects
```

---

## Insert Data

### Insert Single Object

```typescript
const articles = client.collections.get('Article');

const uuid = await articles.data.insert({
  title: 'Weaviate is Amazing',
  body: 'A comprehensive guide to vector databases',
  author: 'John Doe',
  publishDate: new Date('2024-01-15').toISOString(),
  viewCount: 1250,
});

console.log(`Inserted object with UUID: ${uuid}`);
```

### Insert with Custom UUID

```typescript
import { generateUuid5 } from 'weaviate-client';

const myUuid = generateUuid5('my-unique-id');

await articles.data.insert({
  title: 'Custom UUID Article',
  body: 'Article with custom UUID',
}, { uuid: myUuid });
```

### Insert with Vector

```typescript
const uuid = await articles.data.insert({
  title: 'Manual Vector Article',
  body: 'Article with manually provided vector',
}, {
  vector: [0.1, 0.2, 0.3, ...], // Your embedding vector
});
```

### Batch Insert (insertMany)

```typescript
const articles = client.collections.get('Article');

const dataObjects = [
  {
    title: 'First Article',
    body: 'Content of first article',
    author: 'Jane Smith',
  },
  {
    title: 'Second Article',
    body: 'Content of second article',
    author: 'Bob Johnson',
  },
  {
    title: 'Third Article',
    body: 'Content of third article',
    author: 'Alice Williams',
  },
];

const response = await articles.data.insertMany(dataObjects);

console.log(`Inserted ${response.uuids.length} objects`);
console.log('UUIDs:', response.uuids);

// Check for errors
if (response.hasErrors) {
  console.error('Errors occurred:', response.errors);
}
```

### Batch Insert with Vectors

```typescript
const dataWithVectors = [
  {
    title: 'Article 1',
    body: 'Content 1',
    vector: [0.1, 0.2, 0.3],
  },
  {
    title: 'Article 2',
    body: 'Content 2',
    vector: [0.4, 0.5, 0.6],
  },
];

await articles.data.insertMany(dataWithVectors);
```

---

## Query Data

### Fetch Objects (No Search)

```typescript
const articles = client.collections.get('Article');

const result = await articles.query.fetchObjects({
  limit: 10,
  offset: 0,
});

for (const object of result.objects) {
  console.log(object.properties);
  console.log(`UUID: ${object.uuid}`);
}
```

### Fetch by ID

```typescript
const object = await articles.query.fetchObjectById(uuid);
console.log(object?.properties);
```

### Fetch with Filters

```typescript
const result = await articles.query.fetchObjects({
  filters: articles.filter.byProperty('viewCount').greaterThan(1000),
  limit: 5,
});

for (const object of result.objects) {
  console.log(object.properties);
}
```

---

## Vector Search

### nearText Search

```typescript
const articles = client.collections.get('Article');

const result = await articles.query.nearText('vector databases', {
  limit: 5,
  returnMetadata: ['distance', 'score'],
});

for (const object of result.objects) {
  console.log(object.properties);
  console.log(`Distance: ${object.metadata?.distance}`);
  console.log(`Score: ${object.metadata?.score}`);
}
```

### nearText with Filters

```typescript
const result = await articles.query.nearText('machine learning', {
  limit: 3,
  filters: articles.filter.byProperty('author').equal('John Doe'),
  returnMetadata: ['distance'],
});

for (const object of result.objects) {
  console.log(object.properties);
}
```

### nearVector Search

```typescript
const queryVector = [0.1, 0.2, 0.3, ...]; // Your query vector

const result = await articles.query.nearVector(queryVector, {
  limit: 5,
  returnMetadata: ['distance'],
});

for (const object of result.objects) {
  console.log(object.properties);
}
```

### nearObject Search

```typescript
const result = await articles.query.nearObject(existingUuid, {
  limit: 5,
  returnMetadata: ['distance'],
});

for (const object of result.objects) {
  console.log(object.properties);
}
```

---

## Hybrid Search

Hybrid search combines keyword (BM25) and vector search.

### Basic Hybrid Search

```typescript
const articles = client.collections.get('Article');

const result = await articles.query.hybrid('artificial intelligence', {
  limit: 10,
  alpha: 0.5, // 0 = pure keyword, 1 = pure vector, 0.5 = balanced
});

for (const object of result.objects) {
  console.log(object.properties);
}
```

### Hybrid Search with Filters

```typescript
const result = await articles.query.hybrid('deep learning', {
  limit: 5,
  alpha: 0.75,
  filters: articles.filter.byProperty('publishDate').greaterThan(
    new Date('2024-01-01').toISOString()
  ),
  returnMetadata: ['score'],
});

for (const object of result.objects) {
  console.log(object.properties);
  console.log(`Score: ${object.metadata?.score}`);
}
```

### Hybrid Search with Vector

```typescript
const result = await articles.query.hybrid('machine learning', {
  limit: 5,
  alpha: 0.5,
  vector: [0.1, 0.2, 0.3, ...], // Optional manual vector
});
```

---

## Filters

### Basic Filters

```typescript
const articles = client.collections.get('Article');

// Equal
articles.filter.byProperty('author').equal('John Doe')

// Not Equal
articles.filter.byProperty('author').notEqual('Jane Smith')

// Greater Than
articles.filter.byProperty('viewCount').greaterThan(1000)

// Greater Than or Equal
articles.filter.byProperty('viewCount').greaterOrEqual(500)

// Less Than
articles.filter.byProperty('viewCount').lessThan(2000)

// Less Than or Equal
articles.filter.byProperty('viewCount').lessOrEqual(1500)

// Like (text contains)
articles.filter.byProperty('title').like('*vector*')

// Contains Any
articles.filter.byProperty('author').containsAny(['John Doe', 'Jane Smith'])

// Is Null
articles.filter.byProperty('author').isNull(true)
```

### Combining Filters (AND)

```typescript
const result = await articles.query.fetchObjects({
  filters: articles.filter
    .byProperty('viewCount').greaterThan(1000)
    .and()
    .byProperty('author').equal('John Doe'),
  limit: 10,
});
```

### Combining Filters (OR)

```typescript
const result = await articles.query.fetchObjects({
  filters: articles.filter
    .byProperty('author').equal('John Doe')
    .or()
    .byProperty('author').equal('Jane Smith'),
  limit: 10,
});
```

### Complex Filter Combinations

```typescript
const result = await articles.query.fetchObjects({
  filters: articles.filter
    .byProperty('viewCount').greaterThan(500)
    .and()
    .byProperty('publishDate').greaterThan(new Date('2024-01-01').toISOString())
    .or()
    .byProperty('author').equal('John Doe'),
  limit: 10,
});
```

---

## Generative Search (RAG)

Generative search pipes search results through an LLM.

### Single Result Generation

```typescript
const articles = client.collections.get('Article');

const result = await articles.generate.nearText('AI trends', {
  singlePrompt: 'Summarize this article in one sentence: {title} - {body}',
  limit: 5,
});

for (const object of result.objects) {
  console.log('Article:', object.properties);
  console.log('Generated:', object.generated);
}
```

### Grouped Task Generation

```typescript
const result = await articles.generate.nearText('machine learning', {
  groupedTask: 'Write a brief overview summarizing these articles about machine learning',
  limit: 5,
});

console.log('Generated summary:', result.generated);

for (const object of result.objects) {
  console.log('Source article:', object.properties.title);
}
```

### Both Single and Grouped

```typescript
const result = await articles.generate.nearText('neural networks', {
  singlePrompt: 'Explain this in simple terms: {body}',
  groupedTask: 'Create a comprehensive guide based on all these articles',
  limit: 5,
});

for (const object of result.objects) {
  console.log('Article:', object.properties.title);
  console.log('Individual generation:', object.generated);
}

console.log('Grouped generation:', result.generated);
```

### Generative Search with Filters

```typescript
const result = await articles.generate.nearText('databases', {
  singlePrompt: 'Summarize: {title}',
  limit: 3,
  filters: articles.filter.byProperty('author').equal('John Doe'),
});
```

### Generate with nearVector

```typescript
const queryVector = [0.1, 0.2, 0.3, ...];

const result = await articles.generate.nearVector(queryVector, {
  groupedTask: 'Create a summary of these related articles',
  limit: 5,
});

console.log(result.generated);
```

---

## Update Data

### Update Object

```typescript
const articles = client.collections.get('Article');

await articles.data.update({
  id: uuid,
  properties: {
    viewCount: 2000,
    title: 'Updated Title',
  },
});
```

### Replace Object

Replace replaces the entire object (not a merge).

```typescript
await articles.data.replace({
  id: uuid,
  properties: {
    title: 'Completely New Title',
    body: 'Completely new body',
    author: 'New Author',
    publishDate: new Date().toISOString(),
    viewCount: 0,
  },
});
```

---

## Delete Data

### Delete by ID

```typescript
const articles = client.collections.get('Article');

await articles.data.deleteById(uuid);
console.log('Object deleted');
```

### Delete Many by Filter

```typescript
const response = await articles.data.deleteMany(
  articles.filter.byProperty('viewCount').lessThan(100)
);

console.log(`Deleted ${response.successful} objects`);
console.log(`Failed: ${response.failed}`);
```

### Delete All Objects in Collection

```typescript
await articles.data.deleteMany(
  articles.filter.byProperty('title').like('*')
);
```

---

## Aggregations

### Count Objects

```typescript
const articles = client.collections.get('Article');

const result = await articles.aggregate.overAll();
console.log(`Total objects: ${result.totalCount}`);
```

### Count with Filter

```typescript
const result = await articles.aggregate.overAll({
  filters: articles.filter.byProperty('author').equal('John Doe'),
});

console.log(`Objects by John Doe: ${result.totalCount}`);
```

### Aggregate Numeric Properties

```typescript
const result = await articles.aggregate.overAll({
  returnMetrics: articles.metrics.number('viewCount').maximum(),
});

console.log(`Max views: ${result.properties.viewCount?.maximum}`);
```

### Multiple Aggregations

```typescript
const result = await articles.aggregate.overAll({
  returnMetrics: [
    articles.metrics.number('viewCount').sum(),
    articles.metrics.number('viewCount').average(),
    articles.metrics.number('viewCount').minimum(),
    articles.metrics.number('viewCount').maximum(),
  ],
});

console.log('View count stats:', result.properties.viewCount);
```

---

## Multi-Tenancy

### Create Collection with Multi-Tenancy

```typescript
import { configure, dataType } from 'weaviate-client';

await client.collections.create({
  name: 'Article',
  multiTenancyConfig: configure.multiTenancy({ enabled: true }),
  properties: [
    { name: 'title', dataType: dataType.TEXT },
    { name: 'body', dataType: dataType.TEXT },
  ],
});
```

### Add Tenants

```typescript
const articles = client.collections.get('Article');

await articles.tenants.create([
  { name: 'tenant1' },
  { name: 'tenant2' },
  { name: 'tenant3' },
]);
```

### Get Tenant Reference

```typescript
const tenant1Articles = articles.withTenant('tenant1');
```

### Insert Data for Tenant

```typescript
await tenant1Articles.data.insert({
  title: 'Tenant 1 Article',
  body: 'This belongs to tenant 1',
});
```

### Query Tenant Data

```typescript
const result = await tenant1Articles.query.fetchObjects({ limit: 10 });

for (const object of result.objects) {
  console.log(object.properties);
}
```

### List Tenants

```typescript
const tenants = await articles.tenants.get();
console.log(tenants);
```

### Remove Tenant

```typescript
await articles.tenants.remove(['tenant3']);
```

---

## Batch Operations

### Batch with Error Handling

```typescript
const articles = client.collections.get('Article');

const data = [
  { title: 'Article 1', body: 'Content 1', author: 'Author 1' },
  { title: 'Article 2', body: 'Content 2', author: 'Author 2' },
  { title: 'Article 3', body: 'Content 3', author: 'Author 3' },
];

const response = await articles.data.insertMany(data);

if (response.hasErrors) {
  console.error('Batch insert had errors:');
  for (let i = 0; i < response.errors.length; i++) {
    if (response.errors[i]) {
      console.error(`Object ${i}: ${response.errors[i]?.message}`);
    }
  }
}

console.log(`Successfully inserted: ${response.uuids.length} objects`);
```

---

## Cross-References

### Define Collection with Reference

```typescript
import { dataType } from 'weaviate-client';

// Create Author collection
await client.collections.create({
  name: 'Author',
  properties: [
    { name: 'name', dataType: dataType.TEXT },
    { name: 'bio', dataType: dataType.TEXT },
  ],
});

// Create Article collection with reference to Author
await client.collections.create({
  name: 'Article',
  properties: [
    { name: 'title', dataType: dataType.TEXT },
    { name: 'body', dataType: dataType.TEXT },
  ],
  references: [
    {
      name: 'hasAuthor',
      targetCollection: 'Author',
    },
  ],
});
```

### Insert with Reference

```typescript
const authors = client.collections.get('Author');
const articles = client.collections.get('Article');

// Insert author
const authorUuid = await authors.data.insert({
  name: 'John Doe',
  bio: 'Expert in AI',
});

// Insert article with reference
await articles.data.insert({
  title: 'AI Article',
  body: 'Content about AI',
}, {
  references: {
    hasAuthor: authorUuid,
  },
});
```

### Query with References

```typescript
const result = await articles.query.fetchObjects({
  limit: 10,
  returnReferences: [{
    linkOn: 'hasAuthor',
    returnProperties: ['name', 'bio'],
  }],
});

for (const object of result.objects) {
  console.log('Article:', object.properties);
  console.log('Author:', object.references?.hasAuthor);
}
```

---

## Advanced Configurations

### Collection with Custom Vectorizer Settings

```typescript
import { configure, dataType, vectorDistances } from 'weaviate-client';

await client.collections.create({
  name: 'Article',
  vectorizers: configure.vectorizer.text2VecOpenAI({
    model: 'text-embedding-3-large',
    dimensions: 1536,
    vectorizeCollectionName: false,
  }),
  vectorIndexConfig: configure.vectorIndex.hnsw({
    distanceMetric: vectorDistances.COSINE,
    efConstruction: 128,
    maxConnections: 64,
  }),
  properties: [
    {
      name: 'title',
      dataType: dataType.TEXT,
      vectorizePropertyName: false,
      skipVectorization: false,
    },
    {
      name: 'body',
      dataType: dataType.TEXT,
    },
    {
      name: 'metadata',
      dataType: dataType.TEXT,
      skipVectorization: true, // Don't vectorize this property
    },
  ],
});
```

### Collection with Inverted Index Config

```typescript
import { configure, dataType, tokenization } from 'weaviate-client';

await client.collections.create({
  name: 'Article',
  properties: [
    {
      name: 'title',
      dataType: dataType.TEXT,
      tokenization: tokenization.WORD,
      indexFilterable: true,
      indexSearchable: true,
    },
    {
      name: 'sku',
      dataType: dataType.TEXT,
      tokenization: tokenization.FIELD, // Exact match only
      indexFilterable: true,
      indexSearchable: false,
    },
  ],
  invertedIndexConfig: configure.invertedIndex({
    indexTimestamps: true,
    indexNullState: true,
    indexPropertyLength: true,
  }),
});
```

---

## Error Handling

### Try-Catch Pattern

```typescript
import weaviate from 'weaviate-client';

try {
  const client = await weaviate.connectToWeaviateCloud(
    process.env.WEAVIATE_URL!,
    {
      authCredentials: new weaviate.ApiKey(process.env.WEAVIATE_API_KEY!),
    }
  );

  const articles = client.collections.get('Article');

  const result = await articles.query.nearText('AI', { limit: 5 });

  for (const object of result.objects) {
    console.log(object.properties);
  }

  await client.close();
} catch (error) {
  console.error('Error:', error);
}
```

### Batch Insert Error Handling

```typescript
const response = await articles.data.insertMany(dataObjects);

if (response.hasErrors) {
  response.errors.forEach((error, index) => {
    if (error) {
      console.error(`Error at index ${index}:`, error.message);
    }
  });
}

console.log('Successful UUIDs:', response.uuids.filter(uuid => uuid !== null));
```

---

## Complete Example

```typescript
import weaviate, { WeaviateClient, dataType, configure } from 'weaviate-client';

async function main() {
  // Connect
  const client: WeaviateClient = await weaviate.connectToWeaviateCloud(
    process.env.WEAVIATE_URL!,
    {
      authCredentials: new weaviate.ApiKey(process.env.WEAVIATE_API_KEY!),
      headers: {
        'X-OpenAI-Api-Key': process.env.OPENAI_API_KEY!,
      }
    }
  );

  console.log('Connected to Weaviate');

  // Create collection
  await client.collections.create({
    name: 'Article',
    vectorizers: configure.vectorizer.text2VecOpenAI({
      model: 'text-embedding-3-small',
    }),
    generative: configure.generative.openAI({
      model: 'gpt-4',
    }),
    properties: [
      { name: 'title', dataType: dataType.TEXT },
      { name: 'body', dataType: dataType.TEXT },
      { name: 'author', dataType: dataType.TEXT },
      { name: 'publishDate', dataType: dataType.DATE },
      { name: 'viewCount', dataType: dataType.INT },
    ],
  });

  console.log('Collection created');

  // Get collection reference
  const articles = client.collections.get('Article');

  // Insert data
  const dataObjects = [
    {
      title: 'Vector Databases Explained',
      body: 'A comprehensive guide to understanding vector databases and their applications in modern AI systems.',
      author: 'John Doe',
      publishDate: new Date('2024-01-15').toISOString(),
      viewCount: 1250,
    },
    {
      title: 'Machine Learning in Production',
      body: 'Best practices for deploying machine learning models in production environments.',
      author: 'Jane Smith',
      publishDate: new Date('2024-02-20').toISOString(),
      viewCount: 2100,
    },
    {
      title: 'Introduction to RAG',
      body: 'Retrieval Augmented Generation combines search with language models for better responses.',
      author: 'John Doe',
      publishDate: new Date('2024-03-10').toISOString(),
      viewCount: 890,
    },
  ];

  const insertResponse = await articles.data.insertMany(dataObjects);
  console.log(`Inserted ${insertResponse.uuids.length} articles`);

  // Vector search
  console.log('\n--- Vector Search ---');
  const searchResult = await articles.query.nearText('AI and databases', {
    limit: 2,
    returnMetadata: ['distance', 'score'],
  });

  for (const object of searchResult.objects) {
    console.log(`\nTitle: ${object.properties.title}`);
    console.log(`Author: ${object.properties.author}`);
    console.log(`Score: ${object.metadata?.score}`);
  }

  // Filtered search
  console.log('\n--- Filtered Search ---');
  const filteredResult = await articles.query.nearText('machine learning', {
    limit: 3,
    filters: articles.filter.byProperty('viewCount').greaterThan(1000),
  });

  for (const object of filteredResult.objects) {
    console.log(`\nTitle: ${object.properties.title}`);
    console.log(`Views: ${object.properties.viewCount}`);
  }

  // Hybrid search
  console.log('\n--- Hybrid Search ---');
  const hybridResult = await articles.query.hybrid('production deployment', {
    limit: 2,
    alpha: 0.5,
  });

  for (const object of hybridResult.objects) {
    console.log(`\nTitle: ${object.properties.title}`);
  }

  // Generative search (RAG)
  console.log('\n--- Generative Search ---');
  const ragResult = await articles.generate.nearText('vector databases', {
    singlePrompt: 'Summarize this article in one sentence: {title}',
    groupedTask: 'Write a brief paragraph about these articles',
    limit: 2,
  });

  console.log('\nGrouped generation:');
  console.log(ragResult.generated);

  for (const object of ragResult.objects) {
    console.log(`\n${object.properties.title}`);
    console.log(`Summary: ${object.generated}`);
  }

  // Aggregate
  console.log('\n--- Aggregations ---');
  const aggResult = await articles.aggregate.overAll({
    returnMetrics: [
      articles.metrics.number('viewCount').sum(),
      articles.metrics.number('viewCount').average(),
      articles.metrics.number('viewCount').maximum(),
    ],
  });

  console.log(`Total articles: ${aggResult.totalCount}`);
  console.log(`Total views: ${aggResult.properties.viewCount?.sum}`);
  console.log(`Average views: ${aggResult.properties.viewCount?.average}`);
  console.log(`Max views: ${aggResult.properties.viewCount?.maximum}`);

  // Update
  const firstUuid = insertResponse.uuids[0];
  await articles.data.update({
    id: firstUuid,
    properties: {
      viewCount: 1500,
    },
  });
  console.log('\nUpdated first article view count');

  // Delete by filter
  const deleteResult = await articles.data.deleteMany(
    articles.filter.byProperty('viewCount').lessThan(900)
  );
  console.log(`\nDeleted ${deleteResult.successful} low-view articles`);

  // Cleanup
  await client.collections.delete('Article');
  console.log('\nCollection deleted');

  await client.close();
  console.log('Connection closed');
}

main().catch(console.error);
```

---

## TypeScript Types

```typescript
import type {
  WeaviateClient,
  Collection,
  QueryReturn,
  GenerativeReturn,
} from 'weaviate-client';

// Collection reference type
const articles: Collection<{
  title: string;
  body: string;
  author: string;
  publishDate: string;
  viewCount: number;
}> = client.collections.get('Article');

// Query result type
const result: QueryReturn<{
  title: string;
  body: string;
  author: string;
}> = await articles.query.nearText('AI', { limit: 5 });

// Generative result type
const genResult: GenerativeReturn<{
  title: string;
  body: string;
}> = await articles.generate.nearText('AI', {
  singlePrompt: 'Summarize: {title}',
});
```
