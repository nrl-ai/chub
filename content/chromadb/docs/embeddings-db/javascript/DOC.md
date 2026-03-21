---
name: embeddings-db
description: "ChromaDB JavaScript/TypeScript SDK for vector embeddings and AI-powered search"
metadata:
  languages: "javascript"
  versions: "3.0.17"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "chromadb,embeddings,vector-db,ai,search"
---

# ChromaDB JavaScript/TypeScript SDK - v3.0.17

## Golden Rule

**ALWAYS use the official `chromadb` package (v3.0.17 or later) for JavaScript/TypeScript projects.**

```bash
npm install chromadb
```

**For default embeddings support, also install:**

```bash
npm install @chroma-core/default-embed
```

**DO NOT use:**
- Deprecated or unofficial ChromaDB packages
- Old client libraries like `chromadb-client`
- Community wrappers that may be outdated

ChromaDB is the official AI-native open-source vector database. It handles embeddings, indexing, and vector similarity search automatically.

---

## Installation

### Using npm

```bash
npm install chromadb @chroma-core/default-embed
```

### Using yarn

```bash
yarn add chromadb @chroma-core/default-embed
```

### Using pnpm

```bash
pnpm add chromadb @chroma-core/default-embed
```

### Using bun

```bash
bun add chromadb @chroma-core/default-embed
```

---

## Server Setup

ChromaDB requires a backend server to connect to. You have two options:

### Option 1: Run ChromaDB Server Locally

```bash
chroma run --path ./my-chroma-data
```

This starts a ChromaDB server on `http://localhost:8000`

### Option 2: Run ChromaDB with Docker

```bash
docker pull chromadb/chroma
docker run -p 8000:8000 chromadb/chroma
```

---

## Initialization

### Basic Client Connection

```typescript
import { ChromaClient } from "chromadb";

const client = new ChromaClient();
```

This connects to `http://localhost:8000` by default.

### Custom Host/Port Configuration

```typescript
import { ChromaClient } from "chromadb";

const client = new ChromaClient({
  path: "http://localhost:8000"
});
```

### With Authentication Headers

```typescript
import { ChromaClient } from "chromadb";

const client = new ChromaClient({
  path: "http://localhost:8000",
  auth: {
    provider: "token",
    credentials: "your-auth-token"
  }
});
```

### Remote Server Connection

```typescript
import { ChromaClient } from "chromadb";

const client = new ChromaClient({
  path: "https://your-chroma-server.com:8000"
});
```

---

## Collections

### Create a Collection

```typescript
const collection = await client.createCollection({
  name: "my_collection"
});
```

### Create Collection with Distance Metric

```typescript
const collection = await client.createCollection({
  name: "my_collection",
  metadata: {
    "hnsw:space": "cosine"  // Options: "cosine", "l2", "ip"
  }
});
```

**Distance Metrics:**
- `cosine`: Cosine similarity (best for text, normalized vectors)
- `l2`: Euclidean/L2 distance (default, sensitive to magnitude)
- `ip`: Inner product (for recommendation systems)

### Get an Existing Collection

```typescript
const collection = await client.getCollection({
  name: "my_collection"
});
```

### Get or Create Collection

```typescript
const collection = await client.getOrCreateCollection({
  name: "my_collection"
});
```

### List All Collections

```typescript
const collections = await client.listCollections();
console.log(collections);
```

### Delete a Collection

```typescript
await client.deleteCollection({
  name: "my_collection"
});
```

---

## Adding Data

### Add Documents (Auto-Embedding)

```typescript
await collection.add({
  ids: ["id1", "id2", "id3"],
  documents: [
    "This is a document about pineapples",
    "This is a document about oranges",
    "This is a document about apples"
  ]
});
```

ChromaDB will automatically embed the documents using the default embedding function.

### Add with Metadata

```typescript
await collection.add({
  ids: ["id1", "id2", "id3"],
  documents: [
    "This is a document about pineapples",
    "This is a document about oranges",
    "This is a document about apples"
  ],
  metadatas: [
    { category: "tropical", color: "yellow" },
    { category: "citrus", color: "orange" },
    { category: "temperate", color: "red" }
  ]
});
```

### Add with Custom Embeddings

```typescript
await collection.add({
  ids: ["id1", "id2"],
  embeddings: [
    [1.5, 2.9, 3.4, 1.2, 0.8],
    [9.8, 2.3, 2.9, 4.1, 3.3]
  ],
  documents: ["Document one", "Document two"],
  metadatas: [
    { source: "manual" },
    { source: "manual" }
  ]
});
```

### Batch Adding (Large Datasets)

```typescript
const batchSize = 5000;
for (let i = 0; i < totalDocuments.length; i += batchSize) {
  const batch = totalDocuments.slice(i, i + batchSize);
  await collection.add({
    ids: batch.map((_, idx) => `id${i + idx}`),
    documents: batch
  });
}
```

ChromaDB supports adding up to 100k+ documents at once.

---

## Querying Data

### Query with Text (Auto-Embedding)

```typescript
const results = await collection.query({
  queryTexts: ["What fruits are tropical?"],
  nResults: 2
});

console.log(results);
```

**Response Structure:**

```typescript
{
  ids: [["id1", "id2"]],
  distances: [[0.1234, 0.5678]],
  documents: [["This is a document about pineapples", "This is a document..."]],
  metadatas: [[{ category: "tropical", color: "yellow" }, { ... }]]
}
```

### Query with Multiple Texts

```typescript
const results = await collection.query({
  queryTexts: [
    "What fruits are tropical?",
    "What fruits are citrus?"
  ],
  nResults: 2
});
```

Returns `nResults` for each query text.

### Query with Custom Embeddings

```typescript
const results = await collection.query({
  queryEmbeddings: [[1.5, 2.9, 3.4, 1.2, 0.8]],
  nResults: 3
});
```

### Query with Metadata Filters

```typescript
const results = await collection.query({
  queryTexts: ["What fruits are available?"],
  nResults: 5,
  where: {
    category: "tropical"
  }
});
```

### Complex Metadata Filtering

```typescript
// Using $or operator
const results = await collection.query({
  queryTexts: ["Find fruits"],
  nResults: 5,
  where: {
    $or: [
      { category: "tropical" },
      { category: "citrus" }
    ]
  }
});

// Using $and operator
const results = await collection.query({
  queryTexts: ["Find fruits"],
  nResults: 5,
  where: {
    $and: [
      { category: "tropical" },
      { color: "yellow" }
    ]
  }
});

// Using comparison operators
const results = await collection.query({
  queryTexts: ["Find items"],
  nResults: 5,
  where: {
    price: { $gt: 10 }  // $gt, $gte, $lt, $lte, $ne, $eq
  }
});
```

### Query with Document Content Filters

```typescript
const results = await collection.query({
  queryTexts: ["Find documents"],
  nResults: 5,
  whereDocument: {
    $contains: "pineapple"
  }
});

// Using $not_contains
const results = await collection.query({
  queryTexts: ["Find documents"],
  nResults: 5,
  whereDocument: {
    $not_contains: "apple"
  }
});
```

### Query with Include Options

```typescript
const results = await collection.query({
  queryTexts: ["What fruits are tropical?"],
  nResults: 2,
  include: ["documents", "metadatas", "distances", "embeddings"]
});
```

**Include Options:**
- `documents`: The document text (included by default)
- `metadatas`: Metadata for each document (included by default)
- `distances`: Distance/similarity scores (included by default)
- `embeddings`: Vector embeddings (not included by default for performance)

---

## Getting Data

### Get Documents by IDs

```typescript
const results = await collection.get({
  ids: ["id1", "id2"]
});

console.log(results);
```

### Get All Documents

```typescript
const results = await collection.get();
```

Returns all documents in the collection.

### Get with Metadata Filter

```typescript
const results = await collection.get({
  where: {
    category: "tropical"
  }
});
```

### Get with Document Filter

```typescript
const results = await collection.get({
  whereDocument: {
    $contains: "pineapple"
  }
});
```

### Get with Limit and Offset

```typescript
const results = await collection.get({
  limit: 10,
  offset: 20
});
```

### Get with Include Options

```typescript
const results = await collection.get({
  ids: ["id1", "id2"],
  include: ["documents", "metadatas", "embeddings"]
});
```

---

## Updating Data

### Update Documents

```typescript
await collection.update({
  ids: ["id1", "id2"],
  documents: [
    "Updated document about pineapples",
    "Updated document about oranges"
  ],
  metadatas: [
    { category: "tropical", color: "yellow", updated: true },
    { category: "citrus", color: "orange", updated: true }
  ]
});
```

### Update with Custom Embeddings

```typescript
await collection.update({
  ids: ["id1"],
  embeddings: [[1.1, 2.2, 3.3, 4.4, 5.5]],
  documents: ["Updated document"],
  metadatas: [{ source: "updated" }]
});
```

---

## Upsert (Add or Update)

### Upsert Documents

```typescript
await collection.upsert({
  ids: ["id1", "id2", "id3"],
  documents: [
    "Document one - may be new or updated",
    "Document two - may be new or updated",
    "Document three - may be new or updated"
  ],
  metadatas: [
    { version: 2 },
    { version: 2 },
    { version: 1 }
  ]
});
```

If the ID exists, it updates the document. If not, it adds it as new.

---

## Deleting Data

### Delete by IDs

```typescript
await collection.delete({
  ids: ["id1", "id2"]
});
```

### Delete with Metadata Filter

```typescript
await collection.delete({
  where: {
    category: "tropical"
  }
});
```

### Delete with Document Filter

```typescript
await collection.delete({
  whereDocument: {
    $contains: "deprecated"
  }
});
```

### Delete All Documents (Keep Collection)

```typescript
await collection.delete();
```

---

## Collection Utilities

### Count Documents

```typescript
const count = await collection.count();
console.log(`Total documents: ${count}`);
```

### Peek at First Documents

```typescript
const firstDocs = await collection.peek({
  limit: 5
});

console.log(firstDocs);
```

Returns the first 5 documents in the collection.

### Modify Collection Metadata

```typescript
await collection.modify({
  metadata: {
    description: "Collection of fruit documents",
    version: "1.0"
  }
});
```

---

## Embedding Functions

### Using Default Embedding Function

```typescript
import { ChromaClient } from "chromadb";

const client = new ChromaClient();
const collection = await client.createCollection({
  name: "my_collection"
});
```

By default, ChromaDB uses the Sentence Transformers `all-MiniLM-L6-v2` model.

### Using OpenAI Embeddings

```typescript
import { ChromaClient } from "chromadb";
import { OpenAIEmbeddingFunction } from "chromadb";

const embedder = new OpenAIEmbeddingFunction({
  openai_api_key: "your-openai-api-key",
  model_name: "text-embedding-3-small"
});

const collection = await client.createCollection({
  name: "openai_collection",
  embeddingFunction: embedder
});
```

**Available OpenAI Models:**
- `text-embedding-3-small`
- `text-embedding-3-large`
- `text-embedding-ada-002`

### Using Cohere Embeddings

```typescript
import { ChromaClient } from "chromadb";
import { CohereEmbeddingFunction } from "chromadb";

const embedder = new CohereEmbeddingFunction({
  cohere_api_key: "your-cohere-api-key",
  model_name: "embed-english-v3.0"
});

const collection = await client.createCollection({
  name: "cohere_collection",
  embeddingFunction: embedder
});
```

### Using Hugging Face Embeddings

```typescript
import { ChromaClient } from "chromadb";
import { HuggingFaceEmbeddingFunction } from "chromadb";

const embedder = new HuggingFaceEmbeddingFunction({
  api_key: "your-hf-api-key",
  model_name: "sentence-transformers/all-MiniLM-L6-v2"
});

const collection = await client.createCollection({
  name: "hf_collection",
  embeddingFunction: embedder
});
```

### Custom Embedding Function

```typescript
import { IEmbeddingFunction } from "chromadb";

class CustomEmbeddingFunction implements IEmbeddingFunction {
  async generate(texts: string[]): Promise<number[][]> {
    // Your custom embedding logic here
    const embeddings = texts.map(text => {
      // Example: simple character code embedding (replace with real model)
      return Array.from(text).slice(0, 384).map(c => c.charCodeAt(0) / 255);
    });
    return embeddings;
  }
}

const embedder = new CustomEmbeddingFunction();
const collection = await client.createCollection({
  name: "custom_collection",
  embeddingFunction: embedder
});
```

---

## Advanced Client Configuration

### HttpClient with Full Options

```typescript
import { ChromaClient } from "chromadb";

const client = new ChromaClient({
  path: "https://my-chroma-server.com:8000",
  auth: {
    provider: "token",
    credentials: "my-auth-token"
  },
  tenant: "my-tenant",
  database: "my-database"
});
```

### Multi-Tenancy Setup

```typescript
import { ChromaClient } from "chromadb";

const client = new ChromaClient({
  path: "http://localhost:8000"
});

// Create a new tenant and database
await client.createTenant({ name: "acme-corp" });
await client.createDatabase({ name: "production", tenant: "acme-corp" });

// Connect to specific tenant/database
const tenantClient = new ChromaClient({
  path: "http://localhost:8000",
  tenant: "acme-corp",
  database: "production"
});
```

---

## Complete Example: Document Search System

```typescript
import { ChromaClient } from "chromadb";

async function main() {
  // Initialize client
  const client = new ChromaClient({
    path: "http://localhost:8000"
  });

  // Create or get collection
  const collection = await client.getOrCreateCollection({
    name: "knowledge_base",
    metadata: {
      "hnsw:space": "cosine"
    }
  });

  // Add documents
  await collection.add({
    ids: ["doc1", "doc2", "doc3", "doc4"],
    documents: [
      "The quick brown fox jumps over the lazy dog",
      "Machine learning is a subset of artificial intelligence",
      "Python is a popular programming language",
      "ChromaDB is a vector database for AI applications"
    ],
    metadatas: [
      { category: "phrases", language: "english" },
      { category: "ai", language: "english" },
      { category: "programming", language: "english" },
      { category: "database", language: "english" }
    ]
  });

  // Query the collection
  const results = await collection.query({
    queryTexts: ["What is AI?"],
    nResults: 2,
    where: {
      category: "ai"
    }
  });

  console.log("Search Results:");
  console.log(results.documents[0]);
  console.log(results.metadatas[0]);
  console.log(results.distances[0]);

  // Get document count
  const count = await collection.count();
  console.log(`Total documents: ${count}`);

  // Update a document
  await collection.update({
    ids: ["doc2"],
    documents: ["Machine learning is a powerful subset of artificial intelligence"],
    metadatas: [{ category: "ai", language: "english", updated: true }]
  });

  // Delete documents
  await collection.delete({
    ids: ["doc4"]
  });
}

main();
```

---

## Complete Example: Semantic Search with OpenAI

```typescript
import { ChromaClient, OpenAIEmbeddingFunction } from "chromadb";
import * as dotenv from "dotenv";

dotenv.config();

async function semanticSearch() {
  const client = new ChromaClient();

  const embedder = new OpenAIEmbeddingFunction({
    openai_api_key: process.env.OPENAI_API_KEY!,
    model_name: "text-embedding-3-small"
  });

  const collection = await client.getOrCreateCollection({
    name: "articles",
    embeddingFunction: embedder,
    metadata: {
      "hnsw:space": "cosine"
    }
  });

  // Add articles
  await collection.add({
    ids: ["art1", "art2", "art3"],
    documents: [
      "Climate change is affecting global weather patterns",
      "New breakthrough in quantum computing announced",
      "The future of renewable energy looks promising"
    ],
    metadatas: [
      { topic: "environment", date: "2024-01-15" },
      { topic: "technology", date: "2024-01-16" },
      { topic: "energy", date: "2024-01-17" }
    ]
  });

  // Search for relevant articles
  const results = await collection.query({
    queryTexts: ["Tell me about environmental issues"],
    nResults: 3
  });

  results.documents[0].forEach((doc, idx) => {
    console.log(`Result ${idx + 1}:`);
    console.log(`Document: ${doc}`);
    console.log(`Metadata: ${JSON.stringify(results.metadatas[0][idx])}`);
    console.log(`Distance: ${results.distances![0][idx]}`);
    console.log("---");
  });
}

semanticSearch();
```

---

## Complete Example: RAG (Retrieval-Augmented Generation)

```typescript
import { ChromaClient } from "chromadb";
import OpenAI from "openai";

async function ragExample() {
  // Initialize ChromaDB
  const chroma = new ChromaClient();
  const collection = await chroma.getOrCreateCollection({
    name: "company_docs",
    metadata: { "hnsw:space": "cosine" }
  });

  // Add company knowledge base
  await collection.add({
    ids: ["policy1", "policy2", "policy3"],
    documents: [
      "Our company offers 20 days of paid vacation per year",
      "Remote work is available 3 days per week",
      "Health insurance includes dental and vision coverage"
    ],
    metadatas: [
      { type: "policy", category: "time-off" },
      { type: "policy", category: "work-arrangement" },
      { type: "policy", category: "benefits" }
    ]
  });

  // User question
  const question = "How many vacation days do I get?";

  // Retrieve relevant context
  const searchResults = await collection.query({
    queryTexts: [question],
    nResults: 2
  });

  const context = searchResults.documents[0].join("\n");

  // Generate answer with OpenAI
  const openai = new OpenAI({
    apiKey: process.env.OPENAI_API_KEY
  });

  const completion = await openai.chat.completions.create({
    model: "gpt-4",
    messages: [
      {
        role: "system",
        content: "Answer the question based on the context provided."
      },
      {
        role: "user",
        content: `Context:\n${context}\n\nQuestion: ${question}`
      }
    ]
  });

  console.log("Answer:", completion.choices[0].message.content);
}

ragExample();
```

---

## Environment Variables

### .env File Setup

```bash
# ChromaDB Server
CHROMA_HOST=http://localhost:8000

# Authentication (if required)
CHROMA_AUTH_TOKEN=your-auth-token

# Embedding API Keys
OPENAI_API_KEY=sk-...
COHERE_API_KEY=...
HF_API_KEY=...
```

### Using Environment Variables

```typescript
import { ChromaClient } from "chromadb";
import * as dotenv from "dotenv";

dotenv.config();

const client = new ChromaClient({
  path: process.env.CHROMA_HOST || "http://localhost:8000",
  auth: process.env.CHROMA_AUTH_TOKEN ? {
    provider: "token",
    credentials: process.env.CHROMA_AUTH_TOKEN
  } : undefined
});
```

---

## TypeScript Types

### Query Results Type

```typescript
interface QueryResults {
  ids: string[][];
  embeddings?: number[][][];
  documents: (string | null)[][];
  metadatas: (Record<string, any> | null)[][];
  distances?: number[][];
}
```

### Get Results Type

```typescript
interface GetResults {
  ids: string[];
  embeddings?: number[][];
  documents: (string | null)[];
  metadatas: (Record<string, any> | null)[];
}
```

### Collection Metadata

```typescript
interface CollectionMetadata {
  [key: string]: string | number | boolean;
  "hnsw:space"?: "cosine" | "l2" | "ip";
}
```

---

## Error Handling

### Handle Collection Not Found

```typescript
try {
  const collection = await client.getCollection({
    name: "nonexistent_collection"
  });
} catch (error) {
  if (error instanceof Error && error.message.includes("does not exist")) {
    console.error("Collection not found");
    // Create the collection
    const collection = await client.createCollection({
      name: "nonexistent_collection"
    });
  }
}
```

### Handle Duplicate IDs

```typescript
try {
  await collection.add({
    ids: ["id1"],
    documents: ["Document"]
  });

  // This will fail - ID already exists
  await collection.add({
    ids: ["id1"],
    documents: ["Another document"]
  });
} catch (error) {
  console.error("ID already exists. Use update() or upsert() instead.");

  // Use upsert to add or update
  await collection.upsert({
    ids: ["id1"],
    documents: ["Another document"]
  });
}
```

### Handle Connection Errors

```typescript
try {
  const client = new ChromaClient({
    path: "http://localhost:8000"
  });

  const collections = await client.listCollections();
} catch (error) {
  if (error instanceof Error && error.message.includes("ECONNREFUSED")) {
    console.error("Cannot connect to ChromaDB server. Make sure it's running.");
    console.error("Start server with: chroma run --path ./data");
  } else {
    console.error("Error:", error);
  }
}
```

---

## Performance Optimization

### Batch Operations

```typescript
// Batch add for large datasets
const chunkSize = 5000;
for (let i = 0; i < documents.length; i += chunkSize) {
  await collection.add({
    ids: ids.slice(i, i + chunkSize),
    documents: documents.slice(i, i + chunkSize),
    metadatas: metadatas.slice(i, i + chunkSize)
  });
}
```

### Parallel Queries

```typescript
// Run multiple queries in parallel
const queries = [
  "Query about topic A",
  "Query about topic B",
  "Query about topic C"
];

const results = await Promise.all(
  queries.map(query =>
    collection.query({
      queryTexts: [query],
      nResults: 5
    })
  )
);
```

### Limit Included Fields

```typescript
// Exclude embeddings for better performance
const results = await collection.query({
  queryTexts: ["Search query"],
  nResults: 10,
  include: ["documents", "metadatas", "distances"]
  // Don't include embeddings unless needed
});
```

---

## Common Patterns

### Incremental Updates

```typescript
// Add new documents daily
async function addDailyDocuments(newDocs: string[]) {
  const count = await collection.count();
  const newIds = newDocs.map((_, idx) => `doc${count + idx}`);

  await collection.add({
    ids: newIds,
    documents: newDocs,
    metadatas: newDocs.map(() => ({
      added_date: new Date().toISOString()
    }))
  });
}
```

### Search with Fallback

```typescript
async function searchWithFallback(query: string) {
  // Try specific category first
  let results = await collection.query({
    queryTexts: [query],
    nResults: 5,
    where: { category: "premium" }
  });

  // If no results, search all documents
  if (results.ids[0].length === 0) {
    results = await collection.query({
      queryTexts: [query],
      nResults: 5
    });
  }

  return results;
}
```

### Deduplicate Documents

```typescript
async function addUnique(id: string, document: string, metadata: Record<string, any>) {
  try {
    const existing = await collection.get({ ids: [id] });
    if (existing.ids.length > 0) {
      console.log("Document already exists, updating...");
      await collection.update({ ids: [id], documents: [document], metadatas: [metadata] });
    } else {
      await collection.add({ ids: [id], documents: [document], metadatas: [metadata] });
    }
  } catch (error) {
    await collection.add({ ids: [id], documents: [document], metadatas: [metadata] });
  }
}
```

---

## Metadata Filter Operators

### Comparison Operators

```typescript
// Equal
where: { category: "tech" }
where: { category: { $eq: "tech" } }

// Not equal
where: { category: { $ne: "tech" } }

// Greater than
where: { price: { $gt: 100 } }

// Greater than or equal
where: { price: { $gte: 100 } }

// Less than
where: { price: { $lt: 100 } }

// Less than or equal
where: { price: { $lte: 100 } }
```

### Logical Operators

```typescript
// AND
where: {
  $and: [
    { category: "tech" },
    { price: { $lt: 1000 } }
  ]
}

// OR
where: {
  $or: [
    { category: "tech" },
    { category: "science" }
  ]
}

// NOT
where: {
  $not: { category: "archived" }
}
```

### Set Operators

```typescript
// In array
where: {
  category: { $in: ["tech", "science", "health"] }
}

// Not in array
where: {
  category: { $nin: ["archived", "deleted"] }
}
```

---

## Document Filter Operators

### Contains

```typescript
whereDocument: {
  $contains: "machine learning"
}
```

### Not Contains

```typescript
whereDocument: {
  $not_contains: "deprecated"
}
```

### Combined with Metadata

```typescript
const results = await collection.query({
  queryTexts: ["AI research"],
  where: { category: "research" },
  whereDocument: { $contains: "neural network" },
  nResults: 10
});
```
