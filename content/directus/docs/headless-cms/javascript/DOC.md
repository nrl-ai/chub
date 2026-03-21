---
name: headless-cms
description: "Directus JavaScript/TypeScript SDK coding guidelines for interacting with Directus projects using the official SDK"
metadata:
  languages: "javascript"
  versions: "20.1.1"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "directus,headless-cms,api,content,backend"
---

# Directus JavaScript/TypeScript SDK Coding Guidelines

You are a Directus SDK coding expert. Help me with writing code using the Directus SDK calling the official libraries and SDKs.

You can find the official SDK documentation and code samples here:
https://docs.directus.io/guides/sdk/

## Golden Rule: Use the Correct and Current SDK

Always use the official Directus SDK (`@directus/sdk`) to interact with Directus projects. This is the standard library for all Directus API interactions. Do not use legacy libraries or unofficial SDKs.

- **Library Name:** Directus SDK
- **NPM Package:** `@directus/sdk`
- **Legacy Libraries:** `directus-sdk-js`, `@directus/sdk-js` are deprecated

**Installation:**

- **Correct:** `npm install @directus/sdk`
- **Incorrect:** `npm install directus-sdk-js`
- **Incorrect:** `npm install @directus/sdk-js`

**APIs and Usage:**

- **Correct:** `import { createDirectus, rest } from '@directus/sdk'`
- **Correct:** `const client = createDirectus('URL').with(rest())`
- **Correct:** `await client.request(readItems('collection'))`
- **Incorrect:** `DirectusClient` or `DirectusSDK`
- **Incorrect:** `client.items.read()`
- **Incorrect:** Legacy API patterns

## Installation

Install the Directus SDK using npm:

```bash
npm install @directus/sdk
```

## Initialization and Authentication

The `@directus/sdk` library requires creating a Directus client instance using `createDirectus` and composables like `rest()` for REST API functionality.

### Basic Client Setup

```javascript
import { createDirectus, rest } from '@directus/sdk';

// Create a client pointing to your Directus instance
const client = createDirectus('https://your-directus-instance.com').with(rest());
```

### Authentication with Static Token

Use a static access token for authentication:

```javascript
import { createDirectus, staticToken, rest } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(staticToken('YOUR_ACCESS_TOKEN'))
  .with(rest());
```

With environment variables:

```javascript
import { createDirectus, staticToken, rest } from '@directus/sdk';

const client = createDirectus(process.env.DIRECTUS_URL)
  .with(staticToken(process.env.DIRECTUS_TOKEN))
  .with(rest());
```

### Authentication with User Login

Use email and password authentication:

```javascript
import { createDirectus, authentication, rest } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(authentication())
  .with(rest());

// Login with credentials
const result = await client.login('user@example.com', 'password');

// The token is now stored in the client for subsequent requests
```

Login with mode specification:

```javascript
const result = await client.login('user@example.com', 'password', {
  mode: 'cookie' // or 'json'
});
```

### Logout

```javascript
await client.logout();
```

### Refresh Token

```javascript
await client.refresh();
```

### Current User

Get the currently authenticated user:

```javascript
import { createDirectus, rest, authentication, readMe } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(authentication())
  .with(rest());

await client.login('user@example.com', 'password');

const me = await client.request(readMe());
console.log(me);
```

## Reading Items from Collections

### Read All Items

```javascript
import { createDirectus, rest, readItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

// Read all items from a collection
const articles = await client.request(readItems('articles'));
```

### Read with Query Parameters

```javascript
import { createDirectus, rest, readItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

// Read with fields, filters, sorting, and limits
const articles = await client.request(
  readItems('articles', {
    fields: ['id', 'title', 'author.name', 'publish_date'],
    filter: {
      status: { _eq: 'published' },
      publish_date: { _gte: '2024-01-01' }
    },
    sort: ['-publish_date', 'title'],
    limit: 10,
    offset: 0
  })
);
```

### Read Single Item by ID

```javascript
import { createDirectus, rest, readItem } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const article = await client.request(
  readItem('articles', '1', {
    fields: ['*', 'author.*']
  })
);
```

### Search Items

```javascript
import { createDirectus, rest, readItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const results = await client.request(
  readItems('articles', {
    search: 'directus cms',
    fields: ['id', 'title', 'content']
  })
);
```

### Deep Query for Relations

```javascript
import { createDirectus, rest, readItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const articles = await client.request(
  readItems('articles', {
    fields: ['*'],
    deep: {
      author: {
        _filter: {
          status: { _eq: 'active' }
        }
      }
    }
  })
);
```

## Creating Items

### Create Single Item

```javascript
import { createDirectus, rest, createItem } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const newArticle = await client.request(
  createItem('articles', {
    title: 'New Article',
    content: 'Article content here',
    status: 'draft'
  })
);

console.log(newArticle.id);
```

### Create Multiple Items

```javascript
import { createDirectus, rest, createItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const newArticles = await client.request(
  createItems('articles', [
    { title: 'Article 1', status: 'draft' },
    { title: 'Article 2', status: 'draft' },
    { title: 'Article 3', status: 'published' }
  ])
);
```

## Updating Items

### Update Single Item

```javascript
import { createDirectus, rest, updateItem } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const updated = await client.request(
  updateItem('articles', '1', {
    status: 'published',
    publish_date: new Date().toISOString()
  })
);
```

### Update Multiple Items

```javascript
import { createDirectus, rest, updateItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

// Update items matching a filter
const updated = await client.request(
  updateItems('articles',
    { status: 'archived' },
    {
      filter: {
        publish_date: { _lt: '2020-01-01' }
      }
    }
  )
);
```

Update multiple items by IDs:

```javascript
import { createDirectus, rest, updateItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const updated = await client.request(
  updateItems('articles',
    ['1', '2', '3'],
    { status: 'published' }
  )
);
```

## Deleting Items

### Delete Single Item

```javascript
import { createDirectus, rest, deleteItem } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

await client.request(deleteItem('articles', '1'));
```

### Delete Multiple Items

```javascript
import { createDirectus, rest, deleteItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

// Delete by IDs
await client.request(deleteItems('articles', ['1', '2', '3']));
```

Delete items matching a filter:

```javascript
import { createDirectus, rest, deleteItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

await client.request(
  deleteItems('articles', {
    filter: {
      status: { _eq: 'draft' },
      date_created: { _lt: '2023-01-01' }
    }
  })
);
```

## Aggregation and Analytics

### Count Items

```javascript
import { createDirectus, rest, aggregate } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const result = await client.request(
  aggregate('articles', {
    aggregate: {
      count: ['id']
    }
  })
);

console.log(result[0].count.id); // Total count
```

### Count with Filter

```javascript
import { createDirectus, rest, aggregate } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const result = await client.request(
  aggregate('articles', {
    query: {
      filter: {
        status: { _eq: 'published' }
      }
    },
    aggregate: {
      count: ['id']
    }
  })
);
```

### Sum, Average, Min, Max

```javascript
import { createDirectus, rest, aggregate } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const stats = await client.request(
  aggregate('articles', {
    aggregate: {
      sum: ['views'],
      avg: ['views'],
      min: ['views'],
      max: ['views']
    }
  })
);

console.log(stats[0].sum.views);
console.log(stats[0].avg.views);
console.log(stats[0].min.views);
console.log(stats[0].max.views);
```

### Group By

```javascript
import { createDirectus, rest, aggregate } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const grouped = await client.request(
  aggregate('articles', {
    aggregate: {
      count: ['id'],
      avg: ['views']
    },
    groupBy: ['author', 'status']
  })
);

// Returns grouped statistics by author and status
```

Group by date functions:

```javascript
import { createDirectus, rest, aggregate } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const monthlyStats = await client.request(
  aggregate('articles', {
    aggregate: {
      count: ['id']
    },
    groupBy: ['year(publish_date)', 'month(publish_date)']
  })
);
```

## File Management

### Upload Files

```javascript
import { createDirectus, rest, uploadFiles } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const formData = new FormData();
formData.append('file', fileBlob);

const uploadedFile = await client.request(uploadFiles(formData));

console.log(uploadedFile.id);
```

Upload with metadata:

```javascript
import { createDirectus, rest, uploadFiles } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const formData = new FormData();
formData.append('title', 'My Image');
formData.append('folder', 'folder-uuid');
formData.append('file', fileBlob);

const uploadedFile = await client.request(uploadFiles(formData));
```

### Read Files

```javascript
import { createDirectus, rest, readFiles } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const files = await client.request(
  readFiles({
    filter: {
      type: { _starts_with: 'image/' }
    }
  })
);
```

### Read Single File

```javascript
import { createDirectus, rest, readFile } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const file = await client.request(readFile('file-uuid'));
```

### Read Asset (Raw File Content)

```javascript
import { createDirectus, rest, readAssetRaw } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const assetBlob = await client.request(readAssetRaw('file-uuid'));
```

With transformations (for images):

```javascript
import { createDirectus, rest, readAssetRaw } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const thumbnail = await client.request(
  readAssetRaw('file-uuid', {
    width: 300,
    height: 300,
    fit: 'cover',
    quality: 80,
    format: 'webp'
  })
);
```

Available transformation options:

```javascript
import { createDirectus, rest, readAssetRaw } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const transformed = await client.request(
  readAssetRaw('file-uuid', {
    width: 800,
    height: 600,
    fit: 'contain', // cover, contain, inside, outside
    quality: 85,
    format: 'jpg', // jpg, png, webp, tiff, avif
    transforms: [['flip']], // flip, flop, blur, sharpen, grayscale
    withoutEnlargement: true
  })
);
```

### Update File

```javascript
import { createDirectus, rest, updateFile } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const updated = await client.request(
  updateFile('file-uuid', {
    title: 'Updated Title',
    description: 'Updated description'
  })
);
```

### Delete File

```javascript
import { createDirectus, rest, deleteFile } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

await client.request(deleteFile('file-uuid'));
```

## User Management

### Read Users

```javascript
import { createDirectus, rest, readUsers } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const users = await client.request(
  readUsers({
    filter: {
      status: { _eq: 'active' }
    },
    fields: ['id', 'email', 'first_name', 'last_name']
  })
);
```

### Read Single User

```javascript
import { createDirectus, rest, readUser } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const user = await client.request(readUser('user-uuid'));
```

### Create User

```javascript
import { createDirectus, rest, createUser } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const newUser = await client.request(
  createUser({
    email: 'newuser@example.com',
    password: 'secure-password',
    role: 'role-uuid',
    first_name: 'John',
    last_name: 'Doe'
  })
);
```

### Update User

```javascript
import { createDirectus, rest, updateUser } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const updated = await client.request(
  updateUser('user-uuid', {
    first_name: 'Jane',
    status: 'active'
  })
);
```

### Delete User

```javascript
import { createDirectus, rest, deleteUser } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

await client.request(deleteUser('user-uuid'));
```

## Roles and Permissions

### Read Roles

```javascript
import { createDirectus, rest, readRoles } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const roles = await client.request(readRoles());
```

### Read Single Role

```javascript
import { createDirectus, rest, readRole } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const role = await client.request(readRole('role-uuid'));
```

### Create Role

```javascript
import { createDirectus, rest, createRole } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const newRole = await client.request(
  createRole({
    name: 'Content Editor',
    icon: 'edit',
    description: 'Can edit content'
  })
);
```

### Read Permissions

```javascript
import { createDirectus, rest, readPermissions } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const permissions = await client.request(
  readPermissions({
    filter: {
      role: { _eq: 'role-uuid' }
    }
  })
);
```

### Create Permission

```javascript
import { createDirectus, rest, createPermission } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const newPermission = await client.request(
  createPermission({
    role: 'role-uuid',
    collection: 'articles',
    action: 'read',
    fields: ['*']
  })
);
```

## Collections and Fields

### Read Collections

```javascript
import { createDirectus, rest, readCollections } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const collections = await client.request(readCollections());
```

### Read Single Collection

```javascript
import { createDirectus, rest, readCollection } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const collection = await client.request(readCollection('articles'));
```

### Read Fields

```javascript
import { createDirectus, rest, readFields } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

// Read all fields for a collection
const fields = await client.request(readFields('articles'));
```

### Read Single Field

```javascript
import { createDirectus, rest, readField } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const field = await client.request(readField('articles', 'title'));
```

## Real-time with WebSockets

### Setup Real-time Client

```javascript
import { createDirectus, realtime } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(realtime());

// Connect to WebSocket
await client.connect();
```

### Subscribe to Collection Updates

```javascript
import { createDirectus, realtime } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(realtime());

await client.connect();

const { subscription } = await client.subscribe('articles', {
  event: 'update',
  query: {
    fields: ['id', 'title', 'status']
  }
});

for await (const message of subscription) {
  console.log('Item updated:', message);
}
```

### Subscribe to Multiple Events

```javascript
import { createDirectus, realtime } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(realtime());

await client.connect();

const { subscription } = await client.subscribe('articles', {
  event: 'create'
});

for await (const message of subscription) {
  console.log('New item created:', message);
}
```

Available events: `create`, `update`, `delete`

### Unsubscribe

```javascript
await client.unsubscribe(subscription.uid);
```

### Close Connection

```javascript
await client.disconnect();
```

## GraphQL Support

### Setup GraphQL Client

```javascript
import { createDirectus, graphql } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(graphql());
```

### Execute GraphQL Query

```javascript
import { createDirectus, graphql } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(graphql());

const result = await client.query(`
  query {
    articles(filter: { status: { _eq: "published" } }) {
      id
      title
      author {
        first_name
        last_name
      }
    }
  }
`);

console.log(result.articles);
```

### Execute GraphQL Mutation

```javascript
import { createDirectus, graphql } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(graphql());

const result = await client.query(`
  mutation {
    create_articles_item(data: {
      title: "New Article"
      content: "Article content"
      status: "draft"
    }) {
      id
      title
    }
  }
`);
```

With variables:

```javascript
import { createDirectus, graphql } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(graphql());

const result = await client.query(
  `
  query GetArticle($id: ID!) {
    articles_by_id(id: $id) {
      id
      title
      content
    }
  }
  `,
  {
    id: '1'
  }
);
```

## TypeScript Support

### Type-Safe Collections

```typescript
import { createDirectus, rest, readItems } from '@directus/sdk';

// Define your schema types
interface Article {
  id: string;
  title: string;
  content: string;
  status: 'draft' | 'published' | 'archived';
  author: string | Author;
  publish_date: string;
}

interface Author {
  id: string;
  first_name: string;
  last_name: string;
  email: string;
}

interface Schema {
  articles: Article[];
  authors: Author[];
}

// Create typed client
const client = createDirectus<Schema>('https://your-directus-instance.com')
  .with(rest());

// Type-safe operations
const articles = await client.request(
  readItems('articles', {
    fields: ['id', 'title', 'author.first_name']
  })
);

// articles is properly typed as Article[]
```

### Type-Safe Item Creation

```typescript
import { createDirectus, rest, createItem } from '@directus/sdk';

const client = createDirectus<Schema>('https://your-directus-instance.com')
  .with(rest());

const newArticle = await client.request(
  createItem('articles', {
    title: 'New Article',
    content: 'Content here',
    status: 'draft' // TypeScript ensures this matches the union type
  })
);
```

## Advanced Query Filters

### Comparison Operators

```javascript
import { createDirectus, rest, readItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

const filtered = await client.request(
  readItems('articles', {
    filter: {
      views: { _gt: 1000 }, // Greater than
      publish_date: { _lte: '2024-12-31' }, // Less than or equal
      status: { _neq: 'draft' }, // Not equal
      title: { _contains: 'directus' }, // Contains substring
      author: { _null: false } // Not null
    }
  })
);
```

Available operators: `_eq`, `_neq`, `_lt`, `_lte`, `_gt`, `_gte`, `_in`, `_nin`, `_null`, `_nnull`, `_contains`, `_ncontains`, `_starts_with`, `_nstarts_with`, `_ends_with`, `_nends_with`, `_between`, `_nbetween`, `_empty`, `_nempty`

### Logical Operators

```javascript
import { createDirectus, rest, readItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

// AND (default behavior)
const andFilter = await client.request(
  readItems('articles', {
    filter: {
      status: { _eq: 'published' },
      views: { _gt: 100 }
    }
  })
);

// OR
const orFilter = await client.request(
  readItems('articles', {
    filter: {
      _or: [
        { status: { _eq: 'published' } },
        { status: { _eq: 'archived' } }
      ]
    }
  })
);

// Complex nested logic
const complexFilter = await client.request(
  readItems('articles', {
    filter: {
      _and: [
        {
          _or: [
            { status: { _eq: 'published' } },
            { status: { _eq: 'archived' } }
          ]
        },
        { views: { _gt: 100 } }
      ]
    }
  })
);
```

## Error Handling

```javascript
import { createDirectus, rest, readItems } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com').with(rest());

try {
  const articles = await client.request(readItems('articles'));
} catch (error) {
  if (error.errors) {
    // Directus API error
    console.error('API Error:', error.errors);
  } else {
    // Network or other error
    console.error('Error:', error.message);
  }
}
```

Handle authentication errors:

```javascript
import { createDirectus, authentication, rest } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com')
  .with(authentication())
  .with(rest());

try {
  await client.login('user@example.com', 'wrong-password');
} catch (error) {
  if (error.errors?.[0]?.extensions?.code === 'INVALID_CREDENTIALS') {
    console.error('Invalid email or password');
  } else {
    console.error('Login error:', error);
  }
}
```

## Request Configuration

### Custom Headers

```javascript
import { createDirectus, rest } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com', {
  globals: {
    headers: {
      'Custom-Header': 'value'
    }
  }
}).with(rest());
```

### Request Timeout

```javascript
import { createDirectus, rest } from '@directus/sdk';

const client = createDirectus('https://your-directus-instance.com', {
  globals: {
    fetch: {
      timeout: 10000 // 10 seconds
    }
  }
}).with(rest());
```

## Useful Links

- Documentation: https://docs.directus.io/
- SDK Reference: https://docs.directus.io/guides/sdk/
- REST API Reference: https://docs.directus.io/reference/introduction
- GraphQL Reference: https://docs.directus.io/guides/sdk/graphql
- Real-time Guide: https://docs.directus.io/guides/real-time/getting-started/websockets
