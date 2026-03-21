---
name: confluence
description: "Confluence Cloud API coding guidelines for JavaScript/TypeScript using the confluence.js library"
metadata:
  languages: "javascript"
  versions: "2.1.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "atlassian,confluence,wiki,documentation,collaboration"
---

# Confluence Cloud API Coding Guidelines (JavaScript/TypeScript)

You are a **Confluence Cloud API coding expert**. Help me write correct, idiomatic JavaScript/TypeScript that calls the Atlassian Confluence Cloud REST API using the confluence.js library.

Use **only official Atlassian sources** for API behavior, fields, and constraints. This guide summarizes key patterns for both **Node.js** and **browser** applications.

> Ground truth: Atlassian Confluence Cloud REST API documentation at developer.atlassian.com/cloud/confluence/

## Golden Rule: Use confluence.js Library

**CRITICAL:** Use the `confluence.js` npm package (version 2.1.0 or later) for interacting with Confluence Cloud REST API. This is the most comprehensive and actively maintained community library for both Cloud and Server APIs.

**DO NOT use:**
- `confluence-api` (outdated, less comprehensive)
- `atlassian-confluence` (deprecated)
- Direct REST API calls without a client library (error-prone)

**Install:**
```bash
npm install confluence.js
# or
yarn add confluence.js
# or
pnpm add confluence.js
```

The library supports Node.js v20.0.0+ and modern browsers with full TypeScript support.

## Authentication

Confluence Cloud supports multiple authentication methods. Choose based on your use case.

### API Token (Basic Auth) - Recommended for Scripts

Generate an API token from your Atlassian Account Settings (https://id.atlassian.com/manage-profile/security/api-tokens).

```ts
import { ConfluenceClient } from 'confluence.js';

const client = new ConfluenceClient({
  host: 'https://your-domain.atlassian.net/wiki',
  authentication: {
    basic: {
      email: 'your-email@example.com',
      apiToken: process.env.CONFLUENCE_API_TOKEN,
    },
  },
});
```

**Environment Variable Setup:**

```bash
# .env file
CONFLUENCE_API_TOKEN=your_api_token_here
CONFLUENCE_EMAIL=your-email@example.com
CONFLUENCE_HOST=https://your-domain.atlassian.net/wiki
```

**Loading in code:**

```ts
import { config } from 'dotenv';
import { ConfluenceClient } from 'confluence.js';

config(); // Load .env file

const client = new ConfluenceClient({
  host: process.env.CONFLUENCE_HOST!,
  authentication: {
    basic: {
      email: process.env.CONFLUENCE_EMAIL!,
      apiToken: process.env.CONFLUENCE_API_TOKEN!,
    },
  },
});
```

### OAuth 2.0 (3LO) - For Third-Party Apps

Use OAuth 2.0 three-legged authentication for apps that access Confluence on behalf of users.

```ts
import { ConfluenceClient } from 'confluence.js';

const client = new ConfluenceClient({
  host: 'https://your-domain.atlassian.net/wiki',
  authentication: {
    oauth2: {
      accessToken: process.env.OAUTH_ACCESS_TOKEN!,
    },
  },
});
```

### JWT (Server-to-Server) - For Atlassian Connect Apps

```ts
import { ConfluenceClient } from 'confluence.js';

const client = new ConfluenceClient({
  host: 'https://your-domain.atlassian.net/wiki',
  authentication: {
    jwt: {
      issuer: process.env.JWT_ISSUER!,
      secret: process.env.JWT_SECRET!,
      expiryTimeSeconds: 180, // Optional, defaults to 180
    },
  },
});
```

## Initialization Patterns

### Basic Initialization

```ts
import { ConfluenceClient } from 'confluence.js';

const client = new ConfluenceClient({
  host: 'https://your-domain.atlassian.net/wiki',
  authentication: {
    basic: {
      email: 'your-email@example.com',
      apiToken: 'your-api-token',
    },
  },
});
```

### With Custom API Prefix

If your Confluence instance uses a non-standard API path:

```ts
const client = new ConfluenceClient({
  host: 'https://your-domain.atlassian.net',
  apiPrefix: '/custom/api/path',
  authentication: {
    basic: {
      email: 'your-email@example.com',
      apiToken: 'your-api-token',
    },
  },
});
```

### Tree Shaking for Smaller Bundles

Import only the API modules you need:

```ts
import { ContentClient } from 'confluence.js/out/api/content';
import { SpaceClient } from 'confluence.js/out/api/space';

// Use individual clients
const contentClient = new ContentClient({
  host: 'https://your-domain.atlassian.net/wiki',
  authentication: { /* ... */ },
});
```

## Pages API

### Get Page by ID

```ts
const page = await client.content.getContent({
  id: '123456789',
  expand: ['body.storage', 'version', 'space'],
});

console.log(page.title);
console.log(page.body?.storage?.value);
console.log(page.version?.number);
```

### Get Page by Title and Space

```ts
const results = await client.content.getContent({
  spaceKey: 'DEMO',
  title: 'Project Overview',
  expand: ['body.storage', 'version'],
});

if (results.results && results.results.length > 0) {
  const page = results.results[0];
  console.log(page.id, page.title);
}
```

### Create a Page

```ts
const newPage = await client.content.createContent({
  type: 'page',
  title: 'Getting Started Guide',
  space: { key: 'DEMO' },
  body: {
    storage: {
      value: '<h1>Welcome</h1><p>This is the introduction to our project.</p>',
      representation: 'storage',
    },
  },
});

console.log(`Created page: ${newPage.id}`);
console.log(`URL: ${newPage._links?.webui}`);
```

### Create a Child Page

```ts
const childPage = await client.content.createContent({
  type: 'page',
  title: 'Installation Instructions',
  space: { key: 'DEMO' },
  ancestors: [{ id: '123456789' }], // Parent page ID
  body: {
    storage: {
      value: '<p>Follow these steps to install...</p>',
      representation: 'storage',
    },
  },
});
```

### Update a Page

**IMPORTANT:** Always increment the version number when updating.

```ts
// First, get the current page to retrieve version
const currentPage = await client.content.getContent({
  id: '123456789',
  expand: ['version'],
});

// Update with incremented version
const updatedPage = await client.content.updateContent({
  id: '123456789',
  type: 'page',
  title: 'Updated Title',
  version: {
    number: currentPage.version!.number! + 1,
  },
  body: {
    storage: {
      value: '<h1>Updated Content</h1><p>New information here.</p>',
      representation: 'storage',
    },
  },
});

console.log(`Updated to version ${updatedPage.version?.number}`);
```

### Update Only Page Content (Keep Title)

```ts
const currentPage = await client.content.getContent({
  id: '123456789',
  expand: ['version', 'space'],
});

const updatedPage = await client.content.updateContent({
  id: '123456789',
  type: 'page',
  title: currentPage.title, // Keep existing title
  space: { key: currentPage.space?.key },
  version: {
    number: currentPage.version!.number! + 1,
  },
  body: {
    storage: {
      value: '<p>Only the content changed.</p>',
      representation: 'storage',
    },
  },
});
```

### Delete a Page

```ts
await client.content.deleteContent({ id: '123456789' });
console.log('Page deleted successfully');
```

### Get Page Children

```ts
const children = await client.content.getContentChildren({
  id: '123456789',
  expand: ['page'],
});

if (children.page?.results) {
  for (const child of children.page.results) {
    console.log(`Child: ${child.title} (${child.id})`);
  }
}
```

### Get Page History

```ts
const history = await client.content.getHistory({ id: '123456789' });

console.log(`Created by: ${history.createdBy?.displayName}`);
console.log(`Created at: ${history.createdDate}`);
console.log(`Latest version: ${history.latest?.number}`);
```

### Get Page Versions

```ts
const versions = await client.content.getContentVersions({
  id: '123456789',
  limit: 10,
});

for (const version of versions.results || []) {
  console.log(`Version ${version.number} by ${version.by?.displayName}`);
  console.log(`  When: ${version.when}`);
  console.log(`  Message: ${version.message || 'No message'}`);
}
```

## Spaces API

### Get Space by Key

```ts
const space = await client.space.getSpace({
  spaceKey: 'DEMO',
  expand: ['description.plain', 'homepage'],
});

console.log(space.name);
console.log(space.description?.plain?.value);
console.log(space.homepage?.id);
```

### Get All Spaces

```ts
const spaces = await client.space.getSpaces({
  limit: 50,
  expand: ['description.plain'],
});

for (const space of spaces.results || []) {
  console.log(`${space.key}: ${space.name}`);
}
```

### Get Spaces with Pagination

```ts
let start = 0;
const limit = 25;
let hasMore = true;

while (hasMore) {
  const spaces = await client.space.getSpaces({
    start,
    limit,
  });

  for (const space of spaces.results || []) {
    console.log(`${space.key}: ${space.name}`);
  }

  start += limit;
  hasMore = (spaces.results?.length || 0) === limit;
}
```

### Create a Space

```ts
const newSpace = await client.space.createSpace({
  key: 'PROJ',
  name: 'Project Galaxy',
  description: {
    plain: {
      value: 'Documentation for Project Galaxy',
      representation: 'plain',
    },
  },
});

console.log(`Created space: ${newSpace.key}`);
console.log(`URL: ${newSpace._links?.webui}`);
```

### Create a Private Space

```ts
const privateSpace = await client.space.createPrivateSpace({
  key: 'PRIV',
  name: 'Private Team Space',
  description: {
    plain: {
      value: 'Internal team documentation',
      representation: 'plain',
    },
  },
});
```

### Update a Space

```ts
const updatedSpace = await client.space.updateSpace({
  spaceKey: 'DEMO',
  name: 'Demo Space - Updated',
  description: {
    plain: {
      value: 'Updated description',
      representation: 'plain',
    },
  },
});
```

### Delete a Space

```ts
const task = await client.space.deleteSpace({ spaceKey: 'OLDSPACE' });
console.log('Space deletion initiated');
```

### Get Space Content

```ts
const content = await client.space.getSpaceContent({
  spaceKey: 'DEMO',
  expand: ['body.storage'],
  limit: 50,
});

for (const item of content.page?.results || []) {
  console.log(`Page: ${item.title} (${item.id})`);
}
```

## Search API

### Search Content (CQL)

Use Confluence Query Language (CQL) for powerful searches:

```ts
const results = await client.search.search({
  cql: 'type=page AND space=DEMO AND title~"getting started"',
  limit: 20,
  expand: ['content.space', 'content.version'],
});

for (const result of results.results || []) {
  console.log(`${result.content?.title} - ${result.content?.id}`);
}
```

### Common CQL Patterns

```ts
// Find pages in a space
const pagesInSpace = await client.search.search({
  cql: 'type=page AND space=DEMO',
});

// Find pages by creator
const myPages = await client.search.search({
  cql: 'type=page AND creator=currentUser()',
});

// Find recently modified content
const recent = await client.search.search({
  cql: 'type=page AND lastModified > now("-7d") ORDER BY lastModified DESC',
  limit: 10,
});

// Find pages with specific label
const tagged = await client.search.search({
  cql: 'type=page AND label="documentation"',
});

// Complex search with AND/OR
const complex = await client.search.search({
  cql: 'type=page AND space IN (DEMO, PROJ) AND (title~"guide" OR text~"tutorial")',
});
```

### Search by Title

```ts
const results = await client.search.searchContent({
  title: 'Installation',
  limit: 10,
});
```

## Attachments API

### Get Attachments for Page

```ts
const attachments = await client.content.getAttachments({
  id: '123456789',
  limit: 50,
});

for (const attachment of attachments.results || []) {
  console.log(`${attachment.title} - ${attachment.extensions?.fileSize} bytes`);
  console.log(`Download: ${attachment._links?.download}`);
}
```

### Upload an Attachment

```ts
import fs from 'fs';
import FormData from 'form-data';

const form = new FormData();
form.append('file', fs.createReadStream('/path/to/file.pdf'), 'file.pdf');
form.append('comment', 'Uploaded via API');

const attachment = await client.content.createAttachment({
  id: '123456789',
  formData: form,
});

console.log(`Uploaded: ${attachment.results?.[0]?.title}`);
```

### Update an Existing Attachment

```ts
const form = new FormData();
form.append('file', fs.createReadStream('/path/to/updated-file.pdf'), 'file.pdf');
form.append('comment', 'Updated version');

const updated = await client.content.updateAttachment({
  id: '123456789',
  attachmentId: '987654321',
  formData: form,
});
```

### Download an Attachment

```ts
// Get attachment metadata first
const attachments = await client.content.getAttachments({
  id: '123456789',
});

const attachment = attachments.results?.[0];
if (attachment) {
  const downloadUrl = `https://your-domain.atlassian.net${attachment._links?.download}`;
  // Use fetch or axios to download
  console.log(`Download from: ${downloadUrl}`);
}
```

## Labels API

### Get Labels for Content

```ts
const labels = await client.content.getLabels({ id: '123456789' });

for (const label of labels.results || []) {
  console.log(`Label: ${label.name}`);
}
```

### Add Labels to Content

```ts
const newLabels = await client.content.addLabels({
  id: '123456789',
  labels: [
    { name: 'documentation' },
    { name: 'getting-started' },
    { name: 'tutorial' },
  ],
});
```

### Remove a Label

```ts
await client.content.removeLabel({
  id: '123456789',
  label: 'old-label',
});
```

## Comments API

### Get Comments for Page

```ts
const comments = await client.content.getContentComments({
  id: '123456789',
  expand: ['body.storage'],
  limit: 50,
});

for (const comment of comments.results || []) {
  console.log(`Comment by ${comment.history?.createdBy?.displayName}:`);
  console.log(comment.body?.storage?.value);
}
```

### Add a Comment

```ts
const comment = await client.content.createContent({
  type: 'comment',
  container: { id: '123456789', type: 'page' },
  body: {
    storage: {
      value: '<p>This is a helpful comment!</p>',
      representation: 'storage',
    },
  },
});
```

## User and Group APIs

### Get Current User

```ts
const user = await client.user.getCurrentUser();
console.log(`Logged in as: ${user.displayName} (${user.email})`);
```

### Get User by Account ID

```ts
const user = await client.user.getUser({
  accountId: '5a1234567890123456789012',
});
console.log(user.displayName);
```

### Get Group Members

```ts
const members = await client.group.getMembers({
  name: 'confluence-administrators',
  limit: 50,
});

for (const member of members.results || []) {
  console.log(member.displayName);
}
```

## Content Properties (Metadata)

### Get Content Property

```ts
const property = await client.content.getContentProperty({
  id: '123456789',
  key: 'custom-metadata',
});

console.log(property.value);
```

### Set Content Property

```ts
await client.content.createContentProperty({
  id: '123456789',
  key: 'custom-metadata',
  value: {
    lastReviewed: '2025-11-07',
    reviewer: 'john.doe@example.com',
    status: 'approved',
  },
});
```

### Update Content Property

```ts
const current = await client.content.getContentProperty({
  id: '123456789',
  key: 'custom-metadata',
});

await client.content.updateContentProperty({
  id: '123456789',
  key: 'custom-metadata',
  version: {
    number: current.version!.number! + 1,
  },
  value: {
    ...current.value,
    status: 'needs-review',
  },
});
```

### Delete Content Property

```ts
await client.content.deleteContentProperty({
  id: '123456789',
  key: 'custom-metadata',
});
```

## Advanced Content Operations

### Get Content Descendants

```ts
const descendants = await client.content.getContentDescendants({
  id: '123456789',
  expand: ['page'],
});

console.log('All descendant pages:');
for (const page of descendants.page?.results || []) {
  console.log(`  ${page.title} (${page.id})`);
}
```

### Copy a Page

```ts
const original = await client.content.getContent({
  id: '123456789',
  expand: ['body.storage', 'space'],
});

const copy = await client.content.createContent({
  type: 'page',
  title: `${original.title} (Copy)`,
  space: { key: original.space?.key },
  body: original.body,
});
```

### Move a Page to Different Parent

```ts
const page = await client.content.getContent({
  id: '123456789',
  expand: ['version', 'space', 'ancestors'],
});

await client.content.updateContent({
  id: '123456789',
  type: 'page',
  title: page.title,
  space: { key: page.space?.key },
  version: {
    number: page.version!.number! + 1,
  },
  ancestors: [{ id: 'new-parent-id' }],
  body: page.body,
});
```

### Archive a Page

```ts
await client.content.archivePage({ id: '123456789' });
console.log('Page archived');
```

### Restore from Trash

```ts
const restored = await client.content.restore({
  id: '123456789',
});
console.log('Page restored');
```

## Content Restrictions (Permissions)

### Get Content Restrictions

```ts
const restrictions = await client.content.getContentRestrictions({
  id: '123456789',
  expand: ['read.restrictions.user', 'update.restrictions.user'],
});

console.log('Read restrictions:', restrictions.read);
console.log('Update restrictions:', restrictions.update);
```

### Add Read Restriction

```ts
await client.content.addContentRestriction({
  id: '123456789',
  operation: 'read',
  restrictions: {
    user: [
      { accountId: '5a1234567890123456789012' },
    ],
    group: [
      { name: 'confluence-users' },
    ],
  },
});
```

### Remove All Restrictions

```ts
await client.content.deleteContentRestriction({
  id: '123456789',
  operation: 'read',
});

await client.content.deleteContentRestriction({
  id: '123456789',
  operation: 'update',
});
```

## Macros in Content

### Insert a Table of Contents Macro

```ts
const page = await client.content.createContent({
  type: 'page',
  title: 'Documentation Index',
  space: { key: 'DEMO' },
  body: {
    storage: {
      value: `
        <h1>Table of Contents</h1>
        <ac:structured-macro ac:name="toc" ac:schema-version="1">
          <ac:parameter ac:name="maxLevel">3</ac:parameter>
        </ac:structured-macro>
        <h2>Section 1</h2>
        <p>Content here...</p>
      `,
      representation: 'storage',
    },
  },
});
```

### Insert a Status Macro

```ts
const content = `
  <p>Project status:
    <ac:structured-macro ac:name="status" ac:schema-version="1">
      <ac:parameter ac:name="colour">Green</ac:parameter>
      <ac:parameter ac:name="title">Active</ac:parameter>
    </ac:structured-macro>
  </p>
`;
```

### Insert a Code Block Macro

```ts
const content = `
  <ac:structured-macro ac:name="code" ac:schema-version="1">
    <ac:parameter ac:name="language">javascript</ac:parameter>
    <ac:plain-text-body><![CDATA[
const greeting = 'Hello, World!';
console.log(greeting);
    ]]></ac:plain-text-body>
  </ac:structured-macro>
`;
```

### Insert an Info Panel Macro

```ts
const content = `
  <ac:structured-macro ac:name="info" ac:schema-version="1">
    <ac:rich-text-body>
      <p>This is important information for users to know.</p>
    </ac:rich-text-body>
  </ac:structured-macro>
`;
```

## Error Handling

### Basic Error Handling

```ts
try {
  const page = await client.content.getContent({ id: 'invalid-id' });
} catch (error) {
  if (error.response) {
    console.error('Status:', error.response.status);
    console.error('Message:', error.response.data?.message);
  } else {
    console.error('Error:', error.message);
  }
}
```

### Common HTTP Status Codes

- `400 Bad Request`: Invalid parameters or request body
- `401 Unauthorized`: Invalid or missing authentication
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Content or space does not exist
- `409 Conflict`: Version conflict (update with wrong version number)
- `429 Too Many Requests`: Rate limit exceeded

### Handling Version Conflicts

```ts
async function updatePageWithRetry(pageId: string, newContent: string, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const page = await client.content.getContent({
        id: pageId,
        expand: ['version', 'space'],
      });

      const updated = await client.content.updateContent({
        id: pageId,
        type: 'page',
        title: page.title,
        space: { key: page.space?.key },
        version: {
          number: page.version!.number! + 1,
        },
        body: {
          storage: {
            value: newContent,
            representation: 'storage',
          },
        },
      });

      return updated;
    } catch (error) {
      if (error.response?.status === 409 && i < maxRetries - 1) {
        console.log(`Conflict detected, retrying (${i + 1}/${maxRetries})...`);
        await new Promise(resolve => setTimeout(resolve, 1000));
        continue;
      }
      throw error;
    }
  }
}
```

### Rate Limit Handling

```ts
async function apiCallWithBackoff<T>(
  apiCall: () => Promise<T>,
  maxRetries = 5
): Promise<T> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await apiCall();
    } catch (error) {
      if (error.response?.status === 429 && i < maxRetries - 1) {
        const retryAfter = error.response.headers['retry-after'] || Math.pow(2, i);
        console.log(`Rate limited, waiting ${retryAfter}s before retry...`);
        await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
        continue;
      }
      throw error;
    }
  }
  throw new Error('Max retries exceeded');
}

// Usage
const page = await apiCallWithBackoff(() =>
  client.content.getContent({ id: '123456789' })
);
```

## Bulk Operations

### Get Multiple Pages by IDs

```ts
async function getPagesByIds(pageIds: string[]) {
  const pages = await Promise.all(
    pageIds.map(id =>
      client.content.getContent({
        id,
        expand: ['body.storage', 'version']
      })
    )
  );
  return pages;
}
```

### Batch Create Pages

```ts
async function createMultiplePages(
  spaceKey: string,
  pageData: Array<{ title: string; content: string }>
) {
  const created = [];

  for (const { title, content } of pageData) {
    const page = await client.content.createContent({
      type: 'page',
      title,
      space: { key: spaceKey },
      body: {
        storage: {
          value: content,
          representation: 'storage',
        },
      },
    });
    created.push(page);

    // Rate limit protection: wait between requests
    await new Promise(resolve => setTimeout(resolve, 200));
  }

  return created;
}
```

### Batch Update Pages

```ts
async function updateMultiplePages(
  updates: Array<{ id: string; title?: string; content: string }>
) {
  const results = [];

  for (const { id, title, content } of updates) {
    const current = await client.content.getContent({
      id,
      expand: ['version', 'space'],
    });

    const updated = await client.content.updateContent({
      id,
      type: 'page',
      title: title || current.title,
      space: { key: current.space?.key },
      version: {
        number: current.version!.number! + 1,
      },
      body: {
        storage: {
          value: content,
          representation: 'storage',
        },
      },
    });

    results.push(updated);
    await new Promise(resolve => setTimeout(resolve, 200));
  }

  return results;
}
```

## Performance and Optimization

### Use Expand Wisely

Only expand fields you need to reduce response size:

```ts
// Bad - expands everything
const page = await client.content.getContent({
  id: '123456789',
  expand: ['body.storage', 'body.view', 'body.editor', 'version', 'space', 'history', 'ancestors', 'descendants', 'container'],
});

// Good - only what you need
const page = await client.content.getContent({
  id: '123456789',
  expand: ['body.storage', 'version'],
});
```

### Pagination Best Practices

```ts
async function getAllPagesInSpace(spaceKey: string) {
  const allPages = [];
  let start = 0;
  const limit = 100; // Max recommended: 100

  while (true) {
    const response = await client.content.getContent({
      spaceKey,
      type: 'page',
      start,
      limit,
    });

    if (!response.results || response.results.length === 0) {
      break;
    }

    allPages.push(...response.results);

    if (response.results.length < limit) {
      break;
    }

    start += limit;
  }

  return allPages;
}
```

### Caching Strategies

```ts
class ConfluenceCache {
  private cache = new Map<string, { data: any; expires: number }>();

  constructor(private client: ConfluenceClient, private ttlMs = 60000) {}

  async getPage(id: string) {
    const cached = this.cache.get(id);

    if (cached && Date.now() < cached.expires) {
      return cached.data;
    }

    const page = await this.client.content.getContent({ id });

    this.cache.set(id, {
      data: page,
      expires: Date.now() + this.ttlMs,
    });

    return page;
  }

  invalidate(id: string) {
    this.cache.delete(id);
  }
}
```

## TypeScript Support

### Typed Responses

```ts
import { Content, Space, User } from 'confluence.js/out/api/models';

const page: Content = await client.content.getContent({ id: '123456789' });
const space: Space = await client.space.getSpace({ spaceKey: 'DEMO' });
const user: User = await client.user.getCurrentUser();
```

### Type-Safe Page Creation

```ts
import { ContentCreate } from 'confluence.js/out/api/models';

const pageData: ContentCreate = {
  type: 'page',
  title: 'Type-Safe Page',
  space: { key: 'DEMO' },
  body: {
    storage: {
      value: '<p>Content</p>',
      representation: 'storage',
    },
  },
};

const created = await client.content.createContent(pageData);
```

## Common Patterns

### Create or Update Page

```ts
async function createOrUpdatePage(
  spaceKey: string,
  title: string,
  content: string
) {
  // Try to find existing page
  const searchResult = await client.content.getContent({
    spaceKey,
    title,
    expand: ['version', 'space'],
  });

  if (searchResult.results && searchResult.results.length > 0) {
    // Update existing
    const existing = searchResult.results[0];
    return await client.content.updateContent({
      id: existing.id!,
      type: 'page',
      title,
      space: { key: spaceKey },
      version: {
        number: existing.version!.number! + 1,
      },
      body: {
        storage: {
          value: content,
          representation: 'storage',
        },
      },
    });
  } else {
    // Create new
    return await client.content.createContent({
      type: 'page',
      title,
      space: { key: spaceKey },
      body: {
        storage: {
          value: content,
          representation: 'storage',
        },
      },
    });
  }
}
```

### Build Table of Contents

```ts
async function buildTableOfContents(spaceKey: string) {
  const pages = await client.content.getContent({
    spaceKey,
    type: 'page',
    limit: 100,
  });

  let tocHtml = '<h2>Table of Contents</h2><ul>';

  for (const page of pages.results || []) {
    tocHtml += `<li><ac:link><ri:page ri:content-title="${page.title}"/></ac:link></li>`;
  }

  tocHtml += '</ul>';

  return tocHtml;
}
```

### Export Page to Different Format

```ts
async function exportPageAsPDF(pageId: string) {
  const page = await client.content.getContent({
    id: pageId,
    expand: ['body.export_view'],
  });

  // Export view provides HTML optimized for export
  return page.body?.export_view?.value;
}
```

## Content Body Formats

Confluence supports multiple body representations:

- `storage`: XHTML format used for storage (required for create/update)
- `view`: HTML for rendering in browser
- `export_view`: HTML optimized for export
- `editor`: Format used in the editor
- `anonymous_export_view`: HTML for anonymous viewing

```ts
const page = await client.content.getContent({
  id: '123456789',
  expand: ['body.storage', 'body.view', 'body.export_view'],
});

console.log('Storage format:', page.body?.storage?.value);
console.log('View format:', page.body?.view?.value);
console.log('Export format:', page.body?.export_view?.value);
```

## Common Mistakes to Avoid

- **Not incrementing version number** when updating content (causes 409 conflict)
- **Forgetting to expand fields** needed in response (results in undefined properties)
- **Using wrong representation** for body content (use 'storage' for create/update)
- **Not handling pagination** when fetching multiple items (misses items beyond first page)
- **Hardcoding credentials** instead of using environment variables (security risk)
- **Not handling rate limits** in bulk operations (causes 429 errors)
- **Creating pages without checking if they exist** (creates duplicates)
- **Not URL-encoding** special characters in space keys or titles
- **Assuming synchronous operations** (all API calls are async and return promises)
- **Not catching errors** properly (unhandled rejections crash Node.js apps)
- **Using outdated packages** like confluence-api instead of confluence.js
- **Forgetting authentication object** in ConfluenceClient initialization
- **Not escaping HTML** in content values (causes malformed XML in storage format)

## Reference Links

- **confluence.js Documentation**: https://mrrefactoring.github.io/confluence.js/
- **Confluence Cloud REST API v2**: https://developer.atlassian.com/cloud/confluence/rest/v2/
- **Confluence Cloud REST API v1**: https://developer.atlassian.com/cloud/confluence/rest/v1/
- **CQL (Confluence Query Language)**: https://developer.atlassian.com/cloud/confluence/advanced-searching-using-cql/
- **Storage Format Guide**: https://confluence.atlassian.com/doc/confluence-storage-format-790796544.html
- **Atlassian API Tokens**: https://id.atlassian.com/manage-profile/security/api-tokens
- **Rate Limits**: https://developer.atlassian.com/cloud/confluence/rate-limiting/
- **OAuth 2.0 (3LO)**: https://developer.atlassian.com/cloud/confluence/oauth-2-3lo-apps/
- **Atlassian Connect (JWT)**: https://developer.atlassian.com/cloud/confluence/authentication-for-apps/
