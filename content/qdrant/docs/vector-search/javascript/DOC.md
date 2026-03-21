---
name: vector-search
description: "Qdrant JavaScript SDK — use @qdrant/js-client-rest for vector database operations"
metadata:
  languages: "javascript"
  versions: "1.15.1"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "qdrant,vector-search,embeddings,similarity,ai"
---

# Qdrant JavaScript SDK v1.15.1

## Golden Rule

**Always use `@qdrant/js-client-rest` for Qdrant vector database operations.**

Install with:
```bash
npm install @qdrant/js-client-rest
```

**Do NOT use:**
- `node-qdrant` (deprecated, unmaintained)
- `@qdrant/qdrant-js` (main package, but REST client is preferred for most use cases)
- `@qdrant/js-client-grpc` (unless you specifically need gRPC)

The REST client is recommended for most applications. It's easier to debug and provides all necessary functionality. Use gRPC only when you need maximum performance.

## Installation

```bash
npm install @qdrant/js-client-rest
```

For TypeScript projects, types are included automatically.

## Initialization

### Basic Client Setup

```javascript
import { QdrantClient } from '@qdrant/js-client-rest';

// Local Qdrant instance
const client = new QdrantClient({
  url: 'http://127.0.0.1:6333'
});
```

### Alternative Initialization

```javascript
import { QdrantClient } from '@qdrant/js-client-rest';

// Using host and port
const client = new QdrantClient({
  host: 'localhost',
  port: 6333
});
```

### Cloud Configuration

```javascript
import { QdrantClient } from '@qdrant/js-client-rest';

// Qdrant Cloud with API key
const client = new QdrantClient({
  url: 'https://xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.us-east-0-1.aws.cloud.qdrant.io',
  apiKey: process.env.QDRANT_API_KEY,
});
```

### With Environment Variables

```javascript
import { QdrantClient } from '@qdrant/js-client-rest';

const client = new QdrantClient({
  url: process.env.QDRANT_URL || 'http://127.0.0.1:6333',
  apiKey: process.env.QDRANT_API_KEY,
});
```

## Collections

### List Collections

```javascript
const result = await client.getCollections();
console.log('Collections:', result.collections);
```

### Create Collection - Single Vector

```javascript
await client.createCollection('my_collection', {
  vectors: {
    size: 384,
    distance: 'Cosine'
  }
});
```

### Create Collection - Named Vectors

```javascript
await client.createCollection('multi_vector_collection', {
  vectors: {
    image: {
      size: 512,
      distance: 'Dot'
    },
    text: {
      size: 384,
      distance: 'Cosine'
    },
  },
});
```

### Create Collection - Multivector Configuration

```javascript
await client.createCollection('multivec_collection', {
  vectors: {
    size: 128,
    distance: 'Cosine',
    multivector_config: {
      comparator: 'max_sim'
    }
  }
});
```

### Distance Metrics

Available distance metrics:
- `"Cosine"` - Cosine similarity (normalized dot product)
- `"Euclid"` - Euclidean distance (L2)
- `"Dot"` - Dot product
- `"Manhattan"` - Manhattan distance (L1)

### Create Collection - Advanced Options

```javascript
await client.createCollection('advanced_collection', {
  vectors: {
    size: 768,
    distance: 'Cosine',
    on_disk: false,
  },
  shard_number: 2,
  replication_factor: 1,
  write_consistency_factor: 1,
  optimizers_config: {
    default_segment_number: 2,
    indexing_threshold: 20000,
  },
  hnsw_config: {
    m: 16,
    ef_construct: 100,
    full_scan_threshold: 10000,
  }
});
```

### Get Collection Info

```javascript
const info = await client.getCollection('my_collection');
console.log('Collection info:', info);
```

### Delete Collection

```javascript
await client.deleteCollection('my_collection');
```

### Update Collection

```javascript
await client.updateCollection('my_collection', {
  optimizers_config: {
    indexing_threshold: 30000
  }
});
```

## Points (Vectors)

### Upsert Points - Basic

```javascript
await client.upsert('my_collection', {
  points: [
    {
      id: 1,
      vector: [0.05, 0.61, 0.76, 0.74],
      payload: {
        city: 'Berlin',
        price: 1.99,
      }
    },
    {
      id: 2,
      vector: [0.19, 0.81, 0.75, 0.11],
      payload: {
        city: 'London',
        price: 2.49,
      }
    }
  ]
});
```

### Upsert Points - Named Vectors

```javascript
await client.upsert('multi_vector_collection', {
  points: [
    {
      id: 1,
      vector: {
        image: [0.1, 0.2, 0.3, 0.4],
        text: [0.5, 0.6, 0.7, 0.8, 0.9, 0.1, 0.2, 0.3]
      },
      payload: {
        title: 'Product A',
        category: 'electronics'
      }
    }
  ]
});
```

### Upsert Points - Sparse Vectors

```javascript
await client.upsert('sparse_collection', {
  points: [
    {
      id: 1,
      vector: {
        text: {
          indices: [1, 3, 5, 7],
          values: [0.1, 0.2, 0.3, 0.4]
        }
      },
      payload: {
        document: 'Sample text'
      }
    }
  ]
});
```

### Upsert Points - Batch Insert

```javascript
const points = [];
for (let i = 0; i < 1000; i++) {
  points.push({
    id: i,
    vector: Array(384).fill(0).map(() => Math.random()),
    payload: {
      index: i,
      category: `cat_${i % 10}`
    }
  });
}

await client.upsert('my_collection', {
  points: points,
  wait: true
});
```

### Retrieve Points by ID

```javascript
const points = await client.retrieve('my_collection', {
  ids: [1, 2, 3],
  with_payload: true,
  with_vector: false
});
console.log('Retrieved points:', points);
```

### Retrieve Points - With Vectors

```javascript
const points = await client.retrieve('my_collection', {
  ids: [1, 2, 3],
  with_payload: true,
  with_vector: true
});
```

### Delete Points by ID

```javascript
await client.delete('my_collection', {
  points: [1, 2, 3]
});
```

### Delete Points by Filter

```javascript
await client.delete('my_collection', {
  filter: {
    must: [
      {
        key: 'city',
        match: {
          value: 'Berlin'
        }
      }
    ]
  }
});
```

### Count Points

```javascript
const count = await client.count('my_collection', {
  exact: true
});
console.log('Total points:', count.count);
```

### Count Points with Filter

```javascript
const count = await client.count('my_collection', {
  filter: {
    must: [
      {
        key: 'price',
        range: {
          gte: 2.0
        }
      }
    ]
  },
  exact: true
});
```

## Scroll (Pagination)

### Basic Scroll

```javascript
const result = await client.scroll('my_collection', {
  limit: 10,
  with_payload: true,
  with_vector: false
});

console.log('Points:', result.points);
console.log('Next offset:', result.next_page_offset);
```

### Scroll with Filter

```javascript
const result = await client.scroll('my_collection', {
  filter: {
    must: [
      {
        key: 'city',
        match: {
          value: 'Berlin'
        }
      }
    ]
  },
  limit: 20,
  with_payload: true
});
```

### Scroll Pagination

```javascript
let offset = null;
const allPoints = [];

while (true) {
  const result = await client.scroll('my_collection', {
    limit: 100,
    offset: offset,
    with_payload: true,
    with_vector: false
  });

  allPoints.push(...result.points);

  if (!result.next_page_offset) {
    break;
  }

  offset = result.next_page_offset;
}

console.log('Total points retrieved:', allPoints.length);
```

### Scroll with Order

```javascript
const result = await client.scroll('my_collection', {
  limit: 10,
  order_by: 'price',
  with_payload: true
});
```

## Search

### Basic Vector Search

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  limit: 5
});

results.forEach(result => {
  console.log(`ID: ${result.id}, Score: ${result.score}`);
  console.log('Payload:', result.payload);
});
```

### Search with Payload Only

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  limit: 5,
  with_payload: true,
  with_vector: false
});
```

### Search with Score Threshold

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  limit: 10,
  score_threshold: 0.8
});
```

### Search Named Vectors

```javascript
const results = await client.search('multi_vector_collection', {
  vector: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
  using: 'text',
  limit: 5
});
```

## Filtering

### Match Filter

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  filter: {
    must: [
      {
        key: 'city',
        match: {
          value: 'Berlin'
        }
      }
    ]
  },
  limit: 5
});
```

### Range Filter

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  filter: {
    must: [
      {
        key: 'price',
        range: {
          gte: 1.0,
          lt: 3.0
        }
      }
    ]
  },
  limit: 5
});
```

### Multiple Conditions (AND)

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  filter: {
    must: [
      {
        key: 'city',
        match: {
          value: 'Berlin'
        }
      },
      {
        key: 'price',
        range: {
          gte: 2.0
        }
      }
    ]
  },
  limit: 5
});
```

### OR Conditions

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  filter: {
    should: [
      {
        key: 'city',
        match: {
          value: 'Berlin'
        }
      },
      {
        key: 'city',
        match: {
          value: 'London'
        }
      }
    ]
  },
  limit: 5
});
```

### NOT Conditions

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  filter: {
    must_not: [
      {
        key: 'city',
        match: {
          value: 'Berlin'
        }
      }
    ]
  },
  limit: 5
});
```

### Complex Filter

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  filter: {
    must: [
      {
        key: 'category',
        match: {
          value: 'electronics'
        }
      }
    ],
    should: [
      {
        key: 'price',
        range: {
          lt: 100
        }
      },
      {
        key: 'discount',
        match: {
          value: true
        }
      }
    ],
    must_not: [
      {
        key: 'out_of_stock',
        match: {
          value: true
        }
      }
    ]
  },
  limit: 10
});
```

### Match Any (Array)

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  filter: {
    must: [
      {
        key: 'tags',
        match: {
          any: ['electronics', 'gadgets', 'tech']
        }
      }
    ]
  },
  limit: 5
});
```

### Geo Radius Filter

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  filter: {
    must: [
      {
        key: 'location',
        geo_radius: {
          center: {
            lon: 13.404954,
            lat: 52.520008
          },
          radius: 5000.0
        }
      }
    ]
  },
  limit: 5
});
```

### Is Empty Filter

```javascript
const results = await client.search('my_collection', {
  vector: [0.18, 0.81, 0.75, 0.12],
  filter: {
    must: [
      {
        is_empty: {
          key: 'tags'
        }
      }
    ]
  },
  limit: 5
});
```

## Query API (Universal)

### Basic Query

```javascript
const results = await client.query('my_collection', {
  query: [0.1, 0.2, 0.3, 0.4],
  limit: 10
});
```

### Query with Filter

```javascript
const results = await client.query('my_collection', {
  query: [0.1, 0.1, 0.9],
  filter: {
    must: [
      {
        key: 'group_id',
        match: {
          value: 'user_1'
        }
      }
    ]
  },
  limit: 10
});
```

### Recommend Query

```javascript
const results = await client.query('my_collection', {
  query: {
    recommend: {
      positive: [100, 231],
      negative: [718]
    }
  },
  limit: 10
});
```

### Recommend with Vector Examples

```javascript
const results = await client.query('my_collection', {
  query: {
    recommend: {
      positive: [100, 231],
      negative: [[0.2, 0.3, 0.4, 0.5]]
    }
  },
  limit: 10
});
```

### Discover Query

```javascript
const results = await client.query('my_collection', {
  query: {
    discover: {
      target: 100,
      context: [
        {
          positive: 200,
          negative: 300
        },
        {
          positive: 150,
          negative: 250
        }
      ]
    }
  },
  limit: 10
});
```

### Context Query

```javascript
const results = await client.query('my_collection', {
  query: {
    context: [
      {
        positive: 100,
        negative: 718
      },
      {
        positive: 200,
        negative: 300
      }
    ]
  },
  limit: 10
});
```

## Batch Operations

### Batch Search

```javascript
const requests = [
  {
    vector: [0.1, 0.2, 0.3, 0.4],
    limit: 5,
    filter: {
      must: [{ key: 'category', match: { value: 'electronics' } }]
    }
  },
  {
    vector: [0.5, 0.6, 0.7, 0.8],
    limit: 5,
    filter: {
      must: [{ key: 'category', match: { value: 'books' } }]
    }
  }
];

const results = await client.searchBatch('my_collection', {
  searches: requests
});

results.forEach((result, index) => {
  console.log(`Results for request ${index}:`, result);
});
```

### Batch Query

```javascript
const requests = [
  {
    query: [0.1, 0.2, 0.3, 0.4],
    limit: 5
  },
  {
    query: {
      recommend: {
        positive: [100, 200],
        negative: [300]
      }
    },
    limit: 5
  }
];

const results = await client.queryBatch('my_collection', {
  requests: requests
});
```

### Batch Recommend

```javascript
const requests = [
  {
    positive: [100, 231],
    negative: [718],
    limit: 10
  },
  {
    positive: [500, 600],
    negative: [700, 800],
    limit: 10,
    filter: {
      must: [{ key: 'active', match: { value: true } }]
    }
  }
];

const results = await client.recommendBatch('my_collection', {
  searches: requests
});
```

### Batch Upsert Operations

```javascript
const operations = [
  {
    upsert: {
      points: [
        {
          id: 1,
          vector: [0.1, 0.2, 0.3, 0.4],
          payload: { title: 'First' }
        }
      ]
    }
  },
  {
    upsert: {
      points: [
        {
          id: 2,
          vector: [0.5, 0.6, 0.7, 0.8],
          payload: { title: 'Second' }
        }
      ]
    }
  }
];

await client.batch('my_collection', {
  operations: operations
});
```

### Mixed Batch Operations

```javascript
const operations = [
  {
    upsert: {
      points: [
        { id: 1, vector: [0.1, 0.2, 0.3, 0.4], payload: { title: 'Item 1' } }
      ]
    }
  },
  {
    delete_points: {
      points: [5, 6, 7]
    }
  },
  {
    set_payload: {
      payload: { updated: true },
      points: [1, 2, 3]
    }
  }
];

await client.batch('my_collection', {
  operations: operations,
  wait: true
});
```

## Payload Operations

### Set Payload

```javascript
await client.setPayload('my_collection', {
  payload: {
    updated_at: new Date().toISOString(),
    verified: true
  },
  points: [1, 2, 3]
});
```

### Set Payload with Filter

```javascript
await client.setPayload('my_collection', {
  payload: {
    promoted: true
  },
  filter: {
    must: [
      {
        key: 'price',
        range: {
          lt: 50
        }
      }
    ]
  }
});
```

### Overwrite Payload

```javascript
await client.overwritePayload('my_collection', {
  payload: {
    title: 'New Title',
    price: 99.99
  },
  points: [1, 2]
});
```

### Delete Payload Keys

```javascript
await client.deletePayload('my_collection', {
  keys: ['old_field', 'deprecated_field'],
  points: [1, 2, 3]
});
```

### Clear Payload

```javascript
await client.clearPayload('my_collection', {
  points: [1, 2, 3]
});
```

## Snapshots

### Create Collection Snapshot

```javascript
const snapshot = await client.createSnapshot('my_collection');
console.log('Snapshot name:', snapshot.name);
```

### List Collection Snapshots

```javascript
const snapshots = await client.listSnapshots('my_collection');
console.log('Snapshots:', snapshots);
```

### Delete Snapshot

```javascript
await client.deleteSnapshot('my_collection', 'snapshot_name');
```

### Create Full Storage Snapshot

```javascript
const snapshot = await client.createFullSnapshot();
console.log('Full snapshot:', snapshot.name);
```

## Cluster Operations

### Get Cluster Info

```javascript
const clusterInfo = await client.clusterStatus();
console.log('Cluster status:', clusterInfo);
```

## Payload Indexing

### Create Payload Index

```javascript
await client.createPayloadIndex('my_collection', {
  field_name: 'city',
  field_schema: 'keyword'
});
```

### Create Payload Index - Integer

```javascript
await client.createPayloadIndex('my_collection', {
  field_name: 'price',
  field_schema: 'integer'
});
```

### Create Payload Index - Float

```javascript
await client.createPayloadIndex('my_collection', {
  field_name: 'rating',
  field_schema: 'float'
});
```

### Create Payload Index - Geo

```javascript
await client.createPayloadIndex('my_collection', {
  field_name: 'location',
  field_schema: 'geo'
});
```

### Delete Payload Index

```javascript
await client.deletePayloadIndex('my_collection', 'field_name');
```

## Error Handling

### Basic Error Handling

```javascript
try {
  const results = await client.search('my_collection', {
    vector: [0.1, 0.2, 0.3, 0.4],
    limit: 10
  });
  console.log('Results:', results);
} catch (error) {
  console.error('Search failed:', error.message);
}
```

### Check Collection Exists

```javascript
try {
  const info = await client.getCollection('my_collection');
  console.log('Collection exists');
} catch (error) {
  if (error.status === 404) {
    console.log('Collection does not exist');
    // Create collection
    await client.createCollection('my_collection', {
      vectors: { size: 384, distance: 'Cosine' }
    });
  } else {
    throw error;
  }
}
```

### Retry Logic

```javascript
async function searchWithRetry(collection, params, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await client.search(collection, params);
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, 1000 * (i + 1)));
    }
  }
}

const results = await searchWithRetry('my_collection', {
  vector: [0.1, 0.2, 0.3, 0.4],
  limit: 10
});
```

## Complete Example

```javascript
import { QdrantClient } from '@qdrant/js-client-rest';

const client = new QdrantClient({
  url: process.env.QDRANT_URL || 'http://127.0.0.1:6333',
  apiKey: process.env.QDRANT_API_KEY
});

async function main() {
  const collectionName = 'products';

  // Create collection
  await client.createCollection(collectionName, {
    vectors: { size: 4, distance: 'Cosine' }
  });

  // Upsert points
  await client.upsert(collectionName, {
    points: [
      {
        id: 1,
        vector: [0.05, 0.61, 0.76, 0.74],
        payload: {
          name: 'Product A',
          category: 'electronics',
          price: 299.99
        }
      },
      {
        id: 2,
        vector: [0.19, 0.81, 0.75, 0.11],
        payload: {
          name: 'Product B',
          category: 'electronics',
          price: 199.99
        }
      },
      {
        id: 3,
        vector: [0.36, 0.55, 0.47, 0.94],
        payload: {
          name: 'Product C',
          category: 'books',
          price: 29.99
        }
      }
    ]
  });

  // Search
  const results = await client.search(collectionName, {
    vector: [0.2, 0.7, 0.8, 0.1],
    limit: 2,
    with_payload: true
  });

  console.log('Search results:');
  results.forEach(result => {
    console.log(`  ${result.payload.name} - Score: ${result.score}`);
  });

  // Filtered search
  const filteredResults = await client.search(collectionName, {
    vector: [0.2, 0.7, 0.8, 0.1],
    filter: {
      must: [
        {
          key: 'category',
          match: { value: 'electronics' }
        },
        {
          key: 'price',
          range: { lt: 250 }
        }
      ]
    },
    limit: 5
  });

  console.log('\nFiltered results:');
  filteredResults.forEach(result => {
    console.log(`  ${result.payload.name} - $${result.payload.price}`);
  });

  // Count points
  const count = await client.count(collectionName, { exact: true });
  console.log(`\nTotal points: ${count.count}`);

  // Delete collection
  await client.deleteCollection(collectionName);
  console.log('Collection deleted');
}

main().catch(console.error);
```
