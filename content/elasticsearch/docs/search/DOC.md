---
name: search
description: "Official Elasticsearch JavaScript client for full-text search, indexing, and vector search operations."
metadata:
  languages: "javascript"
  versions: "9.2.0"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "elasticsearch,search,full-text,indexing,vector"
---

# Elasticsearch JavaScript Client Coding Guidelines

You are an Elasticsearch API coding expert. Help me with writing code using the Elasticsearch JavaScript client, calling the official libraries and SDKs.

## Golden Rule: Use the Correct and Current SDK

Always use the official Elasticsearch JavaScript client for all Elasticsearch interactions.

- **Library Name:** Elasticsearch JavaScript Client
- **NPM Package:** `@elastic/elasticsearch`
- **Current Version:** 9.2.0

**Installation:**

```bash
npm install @elastic/elasticsearch
```

**Import Patterns:**

```javascript
// ES6 import
import { Client } from '@elastic/elasticsearch';

// CommonJS require
const { Client } = require('@elastic/elasticsearch');
```

Do not use legacy or unofficial packages. The `@elastic/elasticsearch` package is the only official client.

## Initialization and Authentication

The Elasticsearch client requires creating a `Client` instance for all API calls.

### Basic Initialization with API Key

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({
  node: 'https://localhost:9200',
  auth: {
    apiKey: 'base64EncodedApiKey'
  }
});
```

### API Key with ID and Secret

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({
  node: 'https://localhost:9200',
  auth: {
    apiKey: {
      id: 'your-api-key-id',
      api_key: 'your-api-key-secret'
    }
  }
});
```

### Basic Authentication

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({
  node: 'https://localhost:9200',
  auth: {
    username: 'elastic',
    password: 'changeme'
  }
});
```

### Bearer Token Authentication

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({
  node: 'https://localhost:9200',
  auth: {
    bearer: 'your-bearer-token'
  }
});
```

### Elastic Cloud Authentication

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({
  cloud: {
    id: 'your-cloud-id'
  },
  auth: {
    username: 'elastic',
    password: 'your-password'
  }
});
```

### SSL/TLS Configuration

```javascript
import { Client } from '@elastic/elasticsearch';
import fs from 'fs';

const client = new Client({
  node: 'https://localhost:9200',
  auth: {
    username: 'elastic',
    password: 'changeme'
  },
  tls: {
    ca: fs.readFileSync('./http_ca.crt'),
    rejectUnauthorized: true
  }
});
```

### CA Certificate Fingerprint

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({
  node: 'https://localhost:9200',
  auth: {
    username: 'elastic',
    password: 'changeme'
  },
  caFingerprint: '20:0D:CA:FA:76:...'
});
```

### Multiple Certificates

```javascript
import { Client } from '@elastic/elasticsearch';
import fs from 'fs';

const client = new Client({
  node: 'https://localhost:9200',
  tls: {
    ca: [
      fs.readFileSync('./ca.pem'),
      fs.readFileSync('./intermediateRoot.pem')
    ],
    rejectUnauthorized: true
  }
});
```

## Document Operations

### Index a Document (Create/Update)

```javascript
// Index with auto-generated ID
const result = await client.index({
  index: 'products',
  document: {
    name: 'Laptop',
    price: 999.99,
    category: 'electronics'
  }
});

console.log(result._id); // Auto-generated ID
```

### Index with Specific ID

```javascript
const result = await client.index({
  index: 'products',
  id: 'product-123',
  document: {
    name: 'Laptop',
    price: 999.99,
    category: 'electronics'
  }
});

console.log(result.result); // 'created' or 'updated'
```

### Index with Refresh

```javascript
const result = await client.index({
  index: 'products',
  id: 'product-123',
  refresh: 'wait_for', // Options: true, false, 'wait_for'
  document: {
    name: 'Laptop',
    price: 999.99,
    category: 'electronics'
  }
});
```

### Get a Document

```javascript
const result = await client.get({
  index: 'products',
  id: 'product-123'
});

console.log(result._source);
```

### Update a Document

```javascript
const result = await client.update({
  index: 'products',
  id: 'product-123',
  doc: {
    price: 899.99,
    on_sale: true
  }
});

console.log(result.result); // 'updated'
```

### Update with Script

```javascript
const result = await client.update({
  index: 'products',
  id: 'product-123',
  script: {
    source: 'ctx._source.price -= params.discount',
    lang: 'painless',
    params: {
      discount: 100
    }
  }
});
```

### Delete a Document

```javascript
const result = await client.delete({
  index: 'products',
  id: 'product-123'
});

console.log(result.result); // 'deleted'
```

### Check if Document Exists

```javascript
const exists = await client.exists({
  index: 'products',
  id: 'product-123'
});

console.log(exists); // true or false
```

## Bulk Operations

### Bulk Index

```javascript
const operations = [];

const documents = [
  { name: 'Product 1', price: 10 },
  { name: 'Product 2', price: 20 },
  { name: 'Product 3', price: 30 }
];

documents.forEach(doc => {
  operations.push({ index: { _index: 'products' } });
  operations.push(doc);
});

const result = await client.bulk({
  operations: operations
});

// Check for errors
if (result.errors) {
  const erroredDocuments = [];
  result.items.forEach((item, i) => {
    if (item.index?.error) {
      erroredDocuments.push({
        status: item.index.status,
        error: item.index.error,
        document: documents[i]
      });
    }
  });
  console.log('Errors:', erroredDocuments);
}
```

### Bulk with Multiple Operations

```javascript
const operations = [
  // Index operation
  { index: { _index: 'products', _id: '1' } },
  { name: 'Product 1', price: 10 },

  // Update operation
  { update: { _index: 'products', _id: '2' } },
  { doc: { price: 25 } },

  // Delete operation
  { delete: { _index: 'products', _id: '3' } }
];

const result = await client.bulk({ operations });
```

### Bulk Helper

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({ node: 'https://localhost:9200' });

const documents = [
  { name: 'Product 1', price: 10 },
  { name: 'Product 2', price: 20 },
  { name: 'Product 3', price: 30 }
];

const result = await client.helpers.bulk({
  datasource: documents,
  onDocument(doc) {
    return {
      index: { _index: 'products' }
    };
  }
});

console.log(result);
```

### Bulk Helper with Generator

```javascript
async function* generator() {
  for (let i = 0; i < 1000; i++) {
    yield { name: `Product ${i}`, price: i * 10 };
  }
}

const result = await client.helpers.bulk({
  datasource: generator(),
  onDocument(doc) {
    return {
      index: { _index: 'products' }
    };
  }
});
```

## Search Operations

### Basic Search

```javascript
const result = await client.search({
  index: 'products',
  query: {
    match_all: {}
  }
});

result.hits.hits.forEach(hit => {
  console.log(hit._source);
});
```

### Match Query

```javascript
const result = await client.search({
  index: 'products',
  query: {
    match: {
      name: 'laptop'
    }
  }
});
```

### Match with Options

```javascript
const result = await client.search({
  index: 'products',
  query: {
    match: {
      description: {
        query: 'gaming laptop',
        operator: 'and',
        fuzziness: 'AUTO'
      }
    }
  }
});
```

### Match Phrase Query

```javascript
const result = await client.search({
  index: 'products',
  query: {
    match_phrase: {
      description: 'high performance laptop'
    }
  }
});
```

### Term Query (Exact Match)

```javascript
const result = await client.search({
  index: 'products',
  query: {
    term: {
      'category.keyword': 'electronics'
    }
  }
});
```

### Terms Query (Multiple Values)

```javascript
const result = await client.search({
  index: 'products',
  query: {
    terms: {
      'category.keyword': ['electronics', 'computers', 'accessories']
    }
  }
});
```

### Range Query

```javascript
const result = await client.search({
  index: 'products',
  query: {
    range: {
      price: {
        gte: 100,
        lte: 1000
      }
    }
  }
});
```

### Range Query with Dates

```javascript
const result = await client.search({
  index: 'products',
  query: {
    range: {
      created_at: {
        gte: '2024-01-01',
        lte: 'now'
      }
    }
  }
});
```

### Boolean Query

```javascript
const result = await client.search({
  index: 'products',
  query: {
    bool: {
      must: [
        { match: { name: 'laptop' } }
      ],
      filter: [
        { term: { 'category.keyword': 'electronics' } },
        { range: { price: { lte: 1000 } } }
      ],
      should: [
        { term: { on_sale: true } }
      ],
      must_not: [
        { term: { discontinued: true } }
      ],
      minimum_should_match: 0
    }
  }
});
```

### Search with Pagination

```javascript
const result = await client.search({
  index: 'products',
  from: 0,
  size: 20,
  query: {
    match_all: {}
  }
});
```

### Search with Sorting

```javascript
const result = await client.search({
  index: 'products',
  query: {
    match_all: {}
  },
  sort: [
    { price: { order: 'desc' } },
    { created_at: { order: 'desc' } }
  ]
});
```

### Search with Source Filtering

```javascript
const result = await client.search({
  index: 'products',
  query: {
    match_all: {}
  },
  _source: ['name', 'price', 'category']
});
```

### Multi-Match Query

```javascript
const result = await client.search({
  index: 'products',
  query: {
    multi_match: {
      query: 'laptop',
      fields: ['name^2', 'description', 'category']
    }
  }
});
```

### Wildcard Query

```javascript
const result = await client.search({
  index: 'products',
  query: {
    wildcard: {
      'name.keyword': 'lap*'
    }
  }
});
```

### Prefix Query

```javascript
const result = await client.search({
  index: 'products',
  query: {
    prefix: {
      'name.keyword': 'lap'
    }
  }
});
```

### Exists Query

```javascript
const result = await client.search({
  index: 'products',
  query: {
    exists: {
      field: 'discount'
    }
  }
});
```

## Aggregations

### Terms Aggregation

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  query: {
    match_all: {}
  },
  aggs: {
    categories: {
      terms: {
        field: 'category.keyword',
        size: 10
      }
    }
  }
});

result.aggregations.categories.buckets.forEach(bucket => {
  console.log(`${bucket.key}: ${bucket.doc_count}`);
});
```

### Terms Aggregation with Size

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  aggs: {
    top_categories: {
      terms: {
        field: 'category.keyword',
        size: 20,
        order: { _count: 'desc' }
      }
    }
  }
});
```

### Metric Aggregations

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  aggs: {
    avg_price: {
      avg: { field: 'price' }
    },
    min_price: {
      min: { field: 'price' }
    },
    max_price: {
      max: { field: 'price' }
    },
    sum_price: {
      sum: { field: 'price' }
    }
  }
});

console.log('Average:', result.aggregations.avg_price.value);
console.log('Min:', result.aggregations.min_price.value);
console.log('Max:', result.aggregations.max_price.value);
console.log('Sum:', result.aggregations.sum_price.value);
```

### Stats Aggregation

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  aggs: {
    price_stats: {
      stats: { field: 'price' }
    }
  }
});

console.log(result.aggregations.price_stats);
// Returns: count, min, max, avg, sum
```

### Nested Aggregations

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  aggs: {
    categories: {
      terms: {
        field: 'category.keyword'
      },
      aggs: {
        avg_price: {
          avg: { field: 'price' }
        }
      }
    }
  }
});

result.aggregations.categories.buckets.forEach(bucket => {
  console.log(`${bucket.key}: ${bucket.avg_price.value}`);
});
```

### Date Histogram Aggregation

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  aggs: {
    sales_over_time: {
      date_histogram: {
        field: 'created_at',
        calendar_interval: 'month'
      }
    }
  }
});
```

### Date Histogram with Metrics

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  aggs: {
    sales_per_month: {
      date_histogram: {
        field: 'created_at',
        calendar_interval: 'month'
      },
      aggs: {
        total_sales: {
          sum: { field: 'price' }
        },
        avg_price: {
          avg: { field: 'price' }
        }
      }
    }
  }
});
```

### Range Aggregation

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  aggs: {
    price_ranges: {
      range: {
        field: 'price',
        ranges: [
          { to: 50 },
          { from: 50, to: 100 },
          { from: 100, to: 500 },
          { from: 500 }
        ]
      }
    }
  }
});
```

### Histogram Aggregation

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  aggs: {
    price_histogram: {
      histogram: {
        field: 'price',
        interval: 100
      }
    }
  }
});
```

### Filter Aggregation

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  aggs: {
    electronics: {
      filter: {
        term: { 'category.keyword': 'electronics' }
      },
      aggs: {
        avg_price: {
          avg: { field: 'price' }
        }
      }
    }
  }
});
```

### Cardinality Aggregation

```javascript
const result = await client.search({
  index: 'products',
  size: 0,
  aggs: {
    unique_categories: {
      cardinality: {
        field: 'category.keyword'
      }
    }
  }
});

console.log('Unique categories:', result.aggregations.unique_categories.value);
```

## Advanced Search Operations

### Multi-Search (msearch)

```javascript
const searches = [
  { index: 'products' },
  { query: { match: { category: 'electronics' } } },

  { index: 'products' },
  { query: { match: { category: 'books' } } }
];

const result = await client.msearch({
  searches: searches
});

result.responses.forEach((response, i) => {
  console.log(`Search ${i}:`, response.hits.total.value);
});
```

### Scroll API for Large Result Sets

```javascript
// Initial search with scroll
let result = await client.search({
  index: 'products',
  scroll: '1m',
  size: 100,
  query: {
    match_all: {}
  }
});

let scrollId = result._scroll_id;
let hits = result.hits.hits;

console.log(`Retrieved ${hits.length} documents`);

// Continue scrolling
while (hits.length > 0) {
  result = await client.scroll({
    scroll_id: scrollId,
    scroll: '1m'
  });

  scrollId = result._scroll_id;
  hits = result.hits.hits;

  console.log(`Retrieved ${hits.length} more documents`);
}

// Clear scroll
await client.clearScroll({
  scroll_id: scrollId
});
```

### Count API

```javascript
const result = await client.count({
  index: 'products',
  query: {
    match: {
      category: 'electronics'
    }
  }
});

console.log('Total documents:', result.count);
```

### Update By Query

```javascript
const result = await client.updateByQuery({
  index: 'products',
  query: {
    match: {
      category: 'electronics'
    }
  },
  script: {
    source: 'ctx._source.price = ctx._source.price * 0.9',
    lang: 'painless'
  }
});

console.log('Updated:', result.updated);
```

### Update By Query with Parameters

```javascript
const result = await client.updateByQuery({
  index: 'products',
  query: {
    term: { on_sale: false }
  },
  script: {
    source: 'ctx._source.price -= params.discount',
    lang: 'painless',
    params: {
      discount: 50
    }
  }
});
```

### Delete By Query

```javascript
const result = await client.deleteByQuery({
  index: 'products',
  query: {
    term: {
      discontinued: true
    }
  }
});

console.log('Deleted:', result.deleted);
```

### Delete By Query with Conflicts

```javascript
const result = await client.deleteByQuery({
  index: 'products',
  conflicts: 'proceed',
  query: {
    range: {
      created_at: {
        lt: 'now-1y'
      }
    }
  }
});
```

### Reindex

```javascript
const result = await client.reindex({
  source: {
    index: 'products'
  },
  dest: {
    index: 'products_v2'
  }
});

console.log('Reindexed:', result.total);
```

### Reindex with Query

```javascript
const result = await client.reindex({
  source: {
    index: 'products',
    query: {
      term: { category: 'electronics' }
    }
  },
  dest: {
    index: 'electronics_products'
  }
});
```

## Index Management

### Create Index

```javascript
const result = await client.indices.create({
  index: 'products'
});
```

### Create Index with Settings

```javascript
const result = await client.indices.create({
  index: 'products',
  settings: {
    number_of_shards: 3,
    number_of_replicas: 2
  }
});
```

### Create Index with Mappings

```javascript
const result = await client.indices.create({
  index: 'products',
  mappings: {
    properties: {
      name: { type: 'text' },
      description: { type: 'text' },
      price: { type: 'float' },
      category: {
        type: 'text',
        fields: {
          keyword: { type: 'keyword' }
        }
      },
      created_at: { type: 'date' },
      tags: { type: 'keyword' },
      in_stock: { type: 'boolean' }
    }
  }
});
```

### Create Index with Settings and Mappings

```javascript
const result = await client.indices.create({
  index: 'products',
  settings: {
    number_of_shards: 3,
    number_of_replicas: 2,
    analysis: {
      analyzer: {
        custom_analyzer: {
          type: 'custom',
          tokenizer: 'standard',
          filter: ['lowercase', 'asciifolding']
        }
      }
    }
  },
  mappings: {
    properties: {
      name: {
        type: 'text',
        analyzer: 'custom_analyzer'
      },
      price: { type: 'float' },
      category: { type: 'keyword' }
    }
  }
});
```

### Delete Index

```javascript
const result = await client.indices.delete({
  index: 'products'
});
```

### Check if Index Exists

```javascript
const exists = await client.indices.exists({
  index: 'products'
});

console.log(exists); // true or false
```

### Get Index

```javascript
const result = await client.indices.get({
  index: 'products'
});

console.log(result.products);
```

### Get Index Mapping

```javascript
const result = await client.indices.getMapping({
  index: 'products'
});

console.log(result.products.mappings);
```

### Update Index Mapping

```javascript
const result = await client.indices.putMapping({
  index: 'products',
  properties: {
    new_field: { type: 'text' }
  }
});
```

### Get Index Settings

```javascript
const result = await client.indices.getSettings({
  index: 'products'
});

console.log(result.products.settings);
```

### Update Index Settings

```javascript
// Close index first
await client.indices.close({ index: 'products' });

// Update settings
await client.indices.putSettings({
  index: 'products',
  settings: {
    number_of_replicas: 3
  }
});

// Reopen index
await client.indices.open({ index: 'products' });
```

### Refresh Index

```javascript
const result = await client.indices.refresh({
  index: 'products'
});
```

### Flush Index

```javascript
const result = await client.indices.flush({
  index: 'products'
});
```

### Index Aliases

```javascript
// Add alias
await client.indices.putAlias({
  index: 'products_v1',
  name: 'products'
});

// Get aliases
const aliases = await client.indices.getAlias({
  index: 'products_v1'
});

// Delete alias
await client.indices.deleteAlias({
  index: 'products_v1',
  name: 'products'
});
```

### Update Aliases (Atomic)

```javascript
const result = await client.indices.updateAliases({
  actions: [
    {
      remove: { index: 'products_v1', alias: 'products' }
    },
    {
      add: { index: 'products_v2', alias: 'products' }
    }
  ]
});
```

### Index Statistics

```javascript
const result = await client.indices.stats({
  index: 'products'
});

console.log(result._all.total);
```

## Error Handling

### Basic Error Handling

```javascript
try {
  const result = await client.search({
    index: 'products',
    query: {
      match: { name: 'laptop' }
    }
  });
  console.log(result.hits.hits);
} catch (error) {
  console.error('Error:', error.message);
  console.error('Status:', error.meta?.statusCode);
}
```

### Handling Specific Errors

```javascript
import { errors } from '@elastic/elasticsearch';

try {
  const result = await client.get({
    index: 'products',
    id: 'missing-id'
  });
} catch (error) {
  if (error instanceof errors.ResponseError) {
    if (error.meta.statusCode === 404) {
      console.log('Document not found');
    } else {
      console.error('Response error:', error.message);
    }
  } else if (error instanceof errors.TimeoutError) {
    console.error('Request timeout');
  } else if (error instanceof errors.ConnectionError) {
    console.error('Connection error');
  } else {
    console.error('Unknown error:', error);
  }
}
```

### Bulk Error Handling

```javascript
const operations = [
  { index: { _index: 'products', _id: '1' } },
  { name: 'Product 1', price: 10 },
  { index: { _index: 'products', _id: '2' } },
  { name: 'Product 2', price: 20 }
];

const result = await client.bulk({ operations });

if (result.errors) {
  const erroredDocuments = [];

  result.items.forEach((item, i) => {
    const operation = Object.keys(item)[0];
    if (item[operation].error) {
      erroredDocuments.push({
        status: item[operation].status,
        error: item[operation].error,
        operation: operations[i * 2],
        document: operations[i * 2 + 1]
      });
    }
  });

  console.error('Failed documents:', erroredDocuments);
} else {
  console.log('All documents indexed successfully');
}
```

## Client Configuration

### Connection Pool Configuration

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({
  node: 'https://localhost:9200',
  maxRetries: 5,
  requestTimeout: 60000,
  sniffOnStart: true
});
```

### Multiple Nodes

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({
  nodes: [
    'https://node1.example.com:9200',
    'https://node2.example.com:9200',
    'https://node3.example.com:9200'
  ]
});
```

### Custom Headers

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({
  node: 'https://localhost:9200',
  headers: {
    'X-Custom-Header': 'custom-value'
  }
});
```

### Proxy Configuration

```javascript
import { Client } from '@elastic/elasticsearch';
import { HttpProxyAgent } from 'http-proxy-agent';

const client = new Client({
  node: 'https://localhost:9200',
  agent: new HttpProxyAgent('http://proxy.example.com:8080')
});
```

### Request and Response Serialization

```javascript
import { Client } from '@elastic/elasticsearch';

const client = new Client({
  node: 'https://localhost:9200',
  compression: 'gzip',
  enableMetaHeader: true
});
```

### Child Client

```javascript
const childClient = client.child({
  headers: {
    'X-Custom-Header': 'child-value'
  }
});

// Child client inherits parent configuration but can override
const result = await childClient.search({
  index: 'products',
  query: { match_all: {} }
});
```

## Advanced Features

### Point in Time (PIT)

```javascript
// Open PIT
const pitResult = await client.openPointInTime({
  index: 'products',
  keep_alive: '1m'
});

const pitId = pitResult.id;

// Search with PIT
const searchResult = await client.search({
  size: 100,
  query: { match_all: {} },
  pit: {
    id: pitId,
    keep_alive: '1m'
  },
  sort: [{ _shard_doc: 'asc' }]
});

// Close PIT
await client.closePointInTime({
  id: pitId
});
```

### Search After for Deep Pagination

```javascript
// Initial search
let result = await client.search({
  index: 'products',
  size: 100,
  query: { match_all: {} },
  sort: [
    { price: 'asc' },
    { _id: 'asc' }
  ]
});

let hits = result.hits.hits;

// Get next page
if (hits.length > 0) {
  const lastHit = hits[hits.length - 1];

  result = await client.search({
    index: 'products',
    size: 100,
    query: { match_all: {} },
    sort: [
      { price: 'asc' },
      { _id: 'asc' }
    ],
    search_after: lastHit.sort
  });
}
```

### Explain API

```javascript
const result = await client.explain({
  index: 'products',
  id: 'product-123',
  query: {
    match: {
      name: 'laptop'
    }
  }
});

console.log(result.explanation);
```

### Validate Query

```javascript
const result = await client.indices.validateQuery({
  index: 'products',
  query: {
    match: {
      name: 'laptop'
    }
  }
});

console.log('Valid:', result.valid);
```

### Analyze API

```javascript
const result = await client.indices.analyze({
  analyzer: 'standard',
  text: 'Quick brown fox'
});

result.tokens.forEach(token => {
  console.log(token.token);
});
```

### Index Template

```javascript
await client.indices.putIndexTemplate({
  name: 'products_template',
  index_patterns: ['products-*'],
  template: {
    settings: {
      number_of_shards: 2,
      number_of_replicas: 1
    },
    mappings: {
      properties: {
        name: { type: 'text' },
        price: { type: 'float' }
      }
    }
  }
});
```

### Component Template

```javascript
await client.cluster.putComponentTemplate({
  name: 'products_settings',
  template: {
    settings: {
      number_of_shards: 2
    }
  }
});

await client.cluster.putComponentTemplate({
  name: 'products_mappings',
  template: {
    mappings: {
      properties: {
        name: { type: 'text' },
        price: { type: 'float' }
      }
    }
  }
});
```

### Cluster Health

```javascript
const result = await client.cluster.health();

console.log('Status:', result.status);
console.log('Nodes:', result.number_of_nodes);
console.log('Active shards:', result.active_shards);
```

### Cat APIs

```javascript
// Cat indices
const indices = await client.cat.indices({ format: 'json' });
console.log(indices);

// Cat nodes
const nodes = await client.cat.nodes({ format: 'json' });
console.log(nodes);

// Cat health
const health = await client.cat.health({ format: 'json' });
console.log(health);

// Cat count
const count = await client.cat.count({ format: 'json' });
console.log(count);
```

### Ingest Pipelines

```javascript
// Create pipeline
await client.ingest.putPipeline({
  id: 'lowercase_processor',
  processors: [
    {
      lowercase: {
        field: 'name'
      }
    }
  ]
});

// Use pipeline when indexing
await client.index({
  index: 'products',
  pipeline: 'lowercase_processor',
  document: {
    name: 'LAPTOP',
    price: 999
  }
});

// Get pipeline
const pipeline = await client.ingest.getPipeline({
  id: 'lowercase_processor'
});
```

### Simulate Pipeline

```javascript
const result = await client.ingest.simulate({
  id: 'lowercase_processor',
  docs: [
    {
      _source: {
        name: 'LAPTOP'
      }
    }
  ]
});

console.log(result.docs);
```
