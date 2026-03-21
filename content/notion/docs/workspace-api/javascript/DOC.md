---
name: workspace-api
description: "Notion JavaScript/TypeScript SDK for interacting with Notion workspaces, pages, and databases via the official API."
metadata:
  languages: "javascript"
  versions: "5.3.0"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "notion,api,workspace,pages,databases"
---

# Notion API JavaScript/TypeScript SDK Coding Guidelines

You are a Notion API coding expert. Help me with writing code using the Notion API calling the official libraries and SDKs.

You can find the official SDK documentation and code samples here:
https://developers.notion.com/reference/

## Golden Rule: Use the Correct and Current SDK

Always use the Notion JavaScript SDK to interact with Notion workspaces, which is the standard library for all Notion API interactions. Do not use legacy libraries or unofficial SDKs.

- **Library Name:** Notion JavaScript SDK
- **NPM Package:** `@notionhq/client`
- **Legacy Libraries**: `notion-api-js`, `notion-client`, and other unofficial packages are not recommended

**Installation:**

- **Correct:** `npm install @notionhq/client`

**APIs and Usage:**

- **Correct:** `const { Client } = require("@notionhq/client")`
- **Correct:** `const notion = new Client({ auth: process.env.NOTION_TOKEN })`
- **Correct:** `await notion.pages.create(...)`
- **Correct:** `await notion.databases.query(...)`
- **Correct:** `await notion.dataSources.query(...)`
- **Incorrect:** `NotionClient` or `NotionAPI`
- **Incorrect:** Legacy notion-py style APIs

## Installation

Install the Notion JavaScript SDK using npm:

```bash
npm install @notionhq/client
```

Set your environment variable for the Notion API token:

```bash
export NOTION_TOKEN="your_integration_token_here"
```

You can obtain an integration token by creating a new integration at:
https://www.notion.com/my-integrations

## Initialization and API Key

The `@notionhq/client` library requires creating a `Client` instance for all API calls.

- Always use `const notion = new Client({})` to create an instance.
- Set the `NOTION_TOKEN` environment variable, which will be picked up automatically.

```javascript
const { Client } = require('@notionhq/client');

// Uses the NOTION_TOKEN environment variable if auth not specified
const notion = new Client({
  auth: process.env.NOTION_TOKEN,
});
```

### Client Configuration Options

```javascript
const { Client, LogLevel } = require('@notionhq/client');

const notion = new Client({
  auth: process.env.NOTION_TOKEN,
  logLevel: LogLevel.DEBUG, // Controls logging verbosity (default: LogLevel.WARN)
  timeoutMs: 60000, // Request timeout in milliseconds (default: 60000)
  baseUrl: 'https://api.notion.com', // API endpoint (default shown)
});
```

## Important API Version Changes (2025-09-03)

As of API version 2025-09-03, Notion separated databases and data sources:

- **Databases** parent one or more data sources
- **Data Sources** each parent zero or more pages
- Use `notion.dataSources.query()` instead of `notion.databases.query()` for newer implementations
- The SDK minimum version for this API is v5.0.0+

## Pages API

### Create a Page

Create a new page as a child of an existing page:

```javascript
const { Client } = require('@notionhq/client');
const notion = new Client({ auth: process.env.NOTION_TOKEN });

async function createPage() {
  const response = await notion.pages.create({
    parent: {
      page_id: '494c87d072c44cf6960f55f8427f7692',
    },
    properties: {
      title: {
        type: 'title',
        title: [
          {
            type: 'text',
            text: { content: 'A note from your pals at Notion' },
          },
        ],
      },
    },
  });

  console.log(response);
}

createPage();
```

### Create a Page in a Data Source (Database)

```javascript
async function createPageInDataSource() {
  const response = await notion.pages.create({
    parent: {
      data_source_id: '897e5a76-ae52-4b48-9fdf-e71f5945d1af',
    },
    properties: {
      // Properties must match your data source schema
      Name: {
        title: [
          {
            text: {
              content: 'Golden Gate Bridge',
            },
          },
        ],
      },
      Tags: {
        multi_select: [
          { name: 'Landmark' },
          { name: 'San Francisco' },
        ],
      },
      Status: {
        status: {
          name: 'Active',
        },
      },
    },
  });

  return response;
}
```

### Create a Page with Content Blocks

```javascript
async function createPageWithContent() {
  const response = await notion.pages.create({
    parent: {
      page_id: '494c87d072c44cf6960f55f8427f7692',
    },
    properties: {
      title: {
        title: [{ text: { content: 'My New Page' } }],
      },
    },
    children: [
      {
        object: 'block',
        type: 'heading_2',
        heading_2: {
          rich_text: [{ type: 'text', text: { content: 'Introduction' } }],
        },
      },
      {
        object: 'block',
        type: 'paragraph',
        paragraph: {
          rich_text: [
            {
              type: 'text',
              text: { content: 'This is a paragraph with ' },
            },
            {
              type: 'text',
              text: { content: 'bold text', link: null },
              annotations: { bold: true },
            },
          ],
        },
      },
      {
        object: 'block',
        type: 'bulleted_list_item',
        bulleted_list_item: {
          rich_text: [{ type: 'text', text: { content: 'First bullet point' } }],
        },
      },
    ],
  });

  return response;
}
```

### Retrieve a Page

```javascript
async function retrievePage(pageId) {
  const response = await notion.pages.retrieve({
    page_id: pageId,
  });

  console.log('Page title:', response.properties.title);
  console.log('Created at:', response.created_time);
  console.log('Last edited:', response.last_edited_time);

  return response;
}
```

### Update Page Properties

```javascript
async function updatePage(pageId) {
  const response = await notion.pages.update({
    page_id: pageId,
    properties: {
      Status: {
        status: {
          name: 'Completed',
        },
      },
      Tags: {
        multi_select: [
          { name: 'Updated' },
          { name: 'Important' },
        ],
      },
    },
  });

  return response;
}
```

### Archive (Trash) a Page

```javascript
async function archivePage(pageId) {
  const response = await notion.pages.update({
    page_id: pageId,
    archived: true,
  });

  return response;
}
```

## Databases API

### Query a Database (Legacy - Pre 2025-09-03)

```javascript
async function queryDatabase(databaseId) {
  const response = await notion.databases.query({
    database_id: databaseId,
  });

  console.log('Results:', response.results);
  return response;
}
```

### Query with Filters and Sorting

```javascript
async function queryDatabaseWithFilters(databaseId) {
  const response = await notion.databases.query({
    database_id: databaseId,
    filter: {
      and: [
        {
          property: 'Status',
          status: {
            equals: 'Active',
          },
        },
        {
          property: 'Priority',
          select: {
            equals: 'High',
          },
        },
      ],
    },
    sorts: [
      {
        property: 'Created',
        direction: 'descending',
      },
    ],
  });

  return response;
}
```

### Query Data Source (API Version 2025-09-03+)

```javascript
async function queryDataSource(dataSourceId) {
  const response = await notion.dataSources.query({
    data_source_id: dataSourceId,
    filter: {
      property: 'Landmark',
      rich_text: {
        contains: 'Bridge',
      },
    },
  });

  return response;
}
```

### Filter Examples

```javascript
// Text property contains value
const textFilter = {
  property: 'Description',
  rich_text: { contains: 'important' },
};

// Checkbox equals value
const checkboxFilter = {
  property: 'Done',
  checkbox: { equals: true },
};

// Date is after specific date
const dateFilter = {
  property: 'Due Date',
  date: { after: '2025-01-01' },
};

// Number greater than value
const numberFilter = {
  property: 'Score',
  number: { greater_than: 50 },
};

// Multi-select contains value
const multiSelectFilter = {
  property: 'Tags',
  multi_select: { contains: 'urgent' },
};

// Compound OR filter
const orFilter = {
  or: [
    { property: 'Tags', multi_select: { contains: 'A' } },
    { property: 'Tags', multi_select: { contains: 'B' } },
  ],
};

// Compound AND filter
const andFilter = {
  and: [
    { property: 'Done', checkbox: { equals: false } },
    { property: 'Priority', select: { equals: 'High' } },
  ],
};
```

### Retrieve a Database

```javascript
async function retrieveDatabase(databaseId) {
  const response = await notion.databases.retrieve({
    database_id: databaseId,
  });

  console.log('Database title:', response.title);
  console.log('Properties:', response.properties);

  return response;
}
```

### Create a Database

```javascript
async function createDatabase(parentPageId) {
  const response = await notion.databases.create({
    parent: {
      type: 'page_id',
      page_id: parentPageId,
    },
    title: [
      {
        type: 'text',
        text: {
          content: 'Task List',
        },
      },
    ],
    properties: {
      Name: {
        title: {},
      },
      Status: {
        status: {
          options: [
            { name: 'Not started', color: 'red' },
            { name: 'In progress', color: 'yellow' },
            { name: 'Completed', color: 'green' },
          ],
        },
      },
      Priority: {
        select: {
          options: [
            { name: 'High', color: 'red' },
            { name: 'Medium', color: 'yellow' },
            { name: 'Low', color: 'gray' },
          ],
        },
      },
      Tags: {
        multi_select: {
          options: [
            { name: 'Bug', color: 'red' },
            { name: 'Feature', color: 'blue' },
          ],
        },
      },
      'Due Date': {
        date: {},
      },
    },
  });

  return response;
}
```

### Update Database Properties

```javascript
async function updateDatabase(databaseId) {
  const response = await notion.databases.update({
    database_id: databaseId,
    title: [
      {
        text: {
          content: 'Updated Task List',
        },
      },
    ],
    properties: {
      Status: {
        status: {
          options: [
            { name: 'To Do', color: 'red' },
            { name: 'Doing', color: 'yellow' },
            { name: 'Done', color: 'green' },
          ],
        },
      },
    },
  });

  return response;
}
```

## Blocks API

### Retrieve Block Children

```javascript
async function getBlockChildren(blockId) {
  const response = await notion.blocks.children.list({
    block_id: blockId,
    page_size: 50,
  });

  console.log('Number of blocks:', response.results.length);
  return response.results;
}
```

### Append Block Children

```javascript
async function appendBlockChildren(blockId) {
  const response = await notion.blocks.children.append({
    block_id: blockId,
    children: [
      {
        object: 'block',
        type: 'heading_2',
        heading_2: {
          rich_text: [
            {
              type: 'text',
              text: { content: 'New Section' },
            },
          ],
        },
      },
      {
        object: 'block',
        type: 'paragraph',
        paragraph: {
          rich_text: [
            {
              type: 'text',
              text: { content: 'This is a new paragraph.' },
            },
          ],
        },
      },
      {
        object: 'block',
        type: 'to_do',
        to_do: {
          rich_text: [{ type: 'text', text: { content: 'Complete this task' } }],
          checked: false,
        },
      },
    ],
  });

  return response;
}
```

### Common Block Types

```javascript
// Heading blocks
const heading1 = {
  type: 'heading_1',
  heading_1: {
    rich_text: [{ text: { content: 'Heading 1' } }],
  },
};

const heading2 = {
  type: 'heading_2',
  heading_2: {
    rich_text: [{ text: { content: 'Heading 2' } }],
  },
};

const heading3 = {
  type: 'heading_3',
  heading_3: {
    rich_text: [{ text: { content: 'Heading 3' } }],
  },
};

// Paragraph with formatting
const paragraph = {
  type: 'paragraph',
  paragraph: {
    rich_text: [
      { text: { content: 'Normal text ' } },
      { text: { content: 'bold' }, annotations: { bold: true } },
      { text: { content: ' and ' } },
      { text: { content: 'italic' }, annotations: { italic: true } },
      { text: { content: ' and ' } },
      { text: { content: 'code' }, annotations: { code: true } },
    ],
  },
};

// Bulleted list item
const bulletedListItem = {
  type: 'bulleted_list_item',
  bulleted_list_item: {
    rich_text: [{ text: { content: 'Bullet point' } }],
  },
};

// Numbered list item
const numberedListItem = {
  type: 'numbered_list_item',
  numbered_list_item: {
    rich_text: [{ text: { content: 'Numbered item' } }],
  },
};

// To-do item
const toDo = {
  type: 'to_do',
  to_do: {
    rich_text: [{ text: { content: 'Task to complete' } }],
    checked: false,
  },
};

// Toggle block (collapsible)
const toggle = {
  type: 'toggle',
  toggle: {
    rich_text: [{ text: { content: 'Click to expand' } }],
  },
};

// Quote
const quote = {
  type: 'quote',
  quote: {
    rich_text: [{ text: { content: 'This is a quote' } }],
  },
};

// Callout
const callout = {
  type: 'callout',
  callout: {
    rich_text: [{ text: { content: 'Important note' } }],
    icon: { emoji: '💡' },
  },
};

// Code block
const codeBlock = {
  type: 'code',
  code: {
    rich_text: [{ text: { content: 'const x = 42;' } }],
    language: 'javascript',
  },
};

// Divider
const divider = {
  type: 'divider',
  divider: {},
};

// Image
const image = {
  type: 'image',
  image: {
    type: 'external',
    external: {
      url: 'https://example.com/image.png',
    },
  },
};

// Bookmark
const bookmark = {
  type: 'bookmark',
  bookmark: {
    url: 'https://notion.so',
  },
};
```

### Update a Block

```javascript
async function updateBlock(blockId) {
  const response = await notion.blocks.update({
    block_id: blockId,
    paragraph: {
      rich_text: [
        {
          text: {
            content: 'Updated paragraph content',
          },
        },
      ],
    },
  });

  return response;
}
```

### Delete a Block

```javascript
async function deleteBlock(blockId) {
  const response = await notion.blocks.delete({
    block_id: blockId,
  });

  return response;
}
```

### Retrieve a Block

```javascript
async function retrieveBlock(blockId) {
  const response = await notion.blocks.retrieve({
    block_id: blockId,
  });

  console.log('Block type:', response.type);
  return response;
}
```

## Users API

### List All Users

```javascript
async function listUsers() {
  const response = await notion.users.list({});

  console.log('Users:', response.results);
  return response.results;
}
```

### Retrieve a User

```javascript
async function retrieveUser(userId) {
  const response = await notion.users.retrieve({
    user_id: userId,
  });

  console.log('User name:', response.name);
  console.log('User type:', response.type);

  return response;
}
```

### Retrieve Bot User (Your Integration)

```javascript
async function retrieveBotUser() {
  const response = await notion.users.me();

  console.log('Bot name:', response.name);
  console.log('Bot ID:', response.id);

  return response;
}
```

## Search API

### Basic Search

```javascript
async function searchPages(query) {
  const response = await notion.search({
    query: query,
    filter: {
      value: 'page',
      property: 'object',
    },
    sort: {
      direction: 'descending',
      timestamp: 'last_edited_time',
    },
  });

  return response.results;
}
```

### Search for Databases

```javascript
async function searchDatabases(query) {
  const response = await notion.search({
    query: query,
    filter: {
      value: 'database',
      property: 'object',
    },
  });

  return response.results;
}
```

### Search All Content

```javascript
async function searchAll(query) {
  const response = await notion.search({
    query: query,
    page_size: 100,
  });

  console.log(`Found ${response.results.length} results`);
  return response.results;
}
```

## Comments API

### Create a Comment

```javascript
async function createComment(pageId) {
  const response = await notion.comments.create({
    parent: {
      page_id: pageId,
    },
    rich_text: [
      {
        text: {
          content: 'This is a comment on the page.',
        },
      },
    ],
  });

  return response;
}
```

### Create a Discussion Comment (Reply)

```javascript
async function createDiscussionComment(discussionId) {
  const response = await notion.comments.create({
    discussion_id: discussionId,
    rich_text: [
      {
        text: {
          content: 'This is a reply to an existing comment.',
        },
      },
    ],
  });

  return response;
}
```

### List Comments

```javascript
async function listComments(blockId) {
  const response = await notion.comments.list({
    block_id: blockId,
  });

  console.log('Comments:', response.results);
  return response.results;
}
```

## Pagination

### Manual Pagination

```javascript
async function getAllPages(databaseId) {
  let hasMore = true;
  let startCursor = undefined;
  let allResults = [];

  while (hasMore) {
    const response = await notion.databases.query({
      database_id: databaseId,
      start_cursor: startCursor,
      page_size: 100,
    });

    allResults = allResults.concat(response.results);
    hasMore = response.has_more;
    startCursor = response.next_cursor;
  }

  console.log(`Retrieved ${allResults.length} total results`);
  return allResults;
}
```

### Using Pagination Helpers

```javascript
const { iteratePaginatedAPI, collectPaginatedAPI } = require('@notionhq/client');

// Iterate through paginated results
async function iterateBlocks(blockId) {
  for await (const block of iteratePaginatedAPI(notion.blocks.children.list, {
    block_id: blockId,
  })) {
    console.log('Block type:', block.type);
  }
}

// Collect all paginated results at once
async function collectAllBlocks(blockId) {
  const blocks = await collectPaginatedAPI(notion.blocks.children.list, {
    block_id: blockId,
  });

  console.log(`Total blocks: ${blocks.length}`);
  return blocks;
}
```

## Error Handling

### Basic Error Handling

```javascript
const { Client, APIErrorCode } = require('@notionhq/client');

const notion = new Client({ auth: process.env.NOTION_TOKEN });

async function handleErrors() {
  try {
    const response = await notion.pages.retrieve({
      page_id: 'invalid-id',
    });
  } catch (error) {
    if (error.code === APIErrorCode.ObjectNotFound) {
      console.error('Page not found');
    } else if (error.code === APIErrorCode.Unauthorized) {
      console.error('Invalid API token');
    } else if (error.code === APIErrorCode.RestrictedResource) {
      console.error('Access denied - integration not shared with this resource');
    } else if (error.code === APIErrorCode.RateLimited) {
      console.error('Rate limit exceeded');
    } else {
      console.error('Error:', error.message);
    }
  }
}
```

### TypeScript Error Handling

```typescript
import { Client, isNotionClientError, ClientErrorCode, APIErrorCode } from '@notionhq/client';

const notion = new Client({ auth: process.env.NOTION_TOKEN });

async function handleErrorsTypescript() {
  try {
    const response = await notion.pages.retrieve({
      page_id: 'some-id',
    });
  } catch (error: unknown) {
    if (isNotionClientError(error)) {
      switch (error.code) {
        case ClientErrorCode.RequestTimeout:
          console.error('Request timed out');
          break;
        case APIErrorCode.ObjectNotFound:
          console.error('Object not found');
          break;
        case APIErrorCode.Unauthorized:
          console.error('Unauthorized');
          break;
        case APIErrorCode.RestrictedResource:
          console.error('Access denied');
          break;
        case APIErrorCode.RateLimited:
          console.error('Rate limited');
          break;
        default:
          console.error('Unknown error:', error);
      }
    }
  }
}
```

### Common Error Codes

| Error Code | Description |
|------------|-------------|
| `ObjectNotFound` | The requested resource doesn't exist |
| `Unauthorized` | Invalid or missing API token |
| `RestrictedResource` | Integration doesn't have access to the resource |
| `RateLimited` | Too many requests, retry after delay |
| `ValidationError` | Invalid request parameters |
| `ConflictError` | Resource conflict (e.g., duplicate) |
| `RequestTimeout` | Request took too long |

## Advanced Features

### Custom Request

```javascript
async function customRequest() {
  const response = await notion.request({
    path: 'comments',
    method: 'post',
    body: {
      parent: { page_id: '5c6a28216bb14a7eb6e1c50111515c3d' },
      rich_text: [{ text: { content: 'Custom request comment' } }],
    },
  });

  return response;
}
```

### TypeScript Type Guards

```typescript
import { isFullPageOrDataSource, isFullBlock } from '@notionhq/client';

async function useTypeGuards(dataSourceId: string) {
  const response = await notion.dataSources.query({
    data_source_id: dataSourceId,
  });

  for (const page of response.results) {
    if (isFullPageOrDataSource(page)) {
      // Now TypeScript knows page has full properties
      console.log('Created at:', page.created_time);
      console.log('Last edited:', page.last_edited_time);
    }
  }
}

async function checkBlockType(blockId: string) {
  const block = await notion.blocks.retrieve({ block_id: blockId });

  if (isFullBlock(block)) {
    console.log('Block type:', block.type);
  }
}
```

### Rich Text Formatting

```javascript
const richText = [
  {
    type: 'text',
    text: { content: 'Plain text ' },
  },
  {
    type: 'text',
    text: { content: 'bold text' },
    annotations: { bold: true },
  },
  {
    type: 'text',
    text: { content: ' ' },
  },
  {
    type: 'text',
    text: { content: 'italic text' },
    annotations: { italic: true },
  },
  {
    type: 'text',
    text: { content: ' ' },
  },
  {
    type: 'text',
    text: { content: 'strikethrough' },
    annotations: { strikethrough: true },
  },
  {
    type: 'text',
    text: { content: ' ' },
  },
  {
    type: 'text',
    text: { content: 'underline' },
    annotations: { underline: true },
  },
  {
    type: 'text',
    text: { content: ' ' },
  },
  {
    type: 'text',
    text: { content: 'code' },
    annotations: { code: true },
  },
  {
    type: 'text',
    text: { content: ' ' },
  },
  {
    type: 'text',
    text: { content: 'colored text' },
    annotations: { color: 'red' },
  },
  {
    type: 'text',
    text: { content: ' ' },
  },
  {
    type: 'text',
    text: { content: 'link text', link: { url: 'https://notion.so' } },
  },
];
```

### Property Types

```javascript
// Title property
const titleProperty = {
  title: [
    {
      text: { content: 'Page Title' },
    },
  ],
};

// Rich text property
const richTextProperty = {
  rich_text: [
    {
      text: { content: 'Some text content' },
    },
  ],
};

// Number property
const numberProperty = {
  number: 42,
};

// Select property
const selectProperty = {
  select: {
    name: 'Option 1',
  },
};

// Multi-select property
const multiSelectProperty = {
  multi_select: [
    { name: 'Tag 1' },
    { name: 'Tag 2' },
  ],
};

// Date property
const dateProperty = {
  date: {
    start: '2025-01-01',
    end: '2025-01-31',
  },
};

// Checkbox property
const checkboxProperty = {
  checkbox: true,
};

// URL property
const urlProperty = {
  url: 'https://notion.so',
};

// Email property
const emailProperty = {
  email: 'user@example.com',
};

// Phone number property
const phoneProperty = {
  phone_number: '+1-555-0100',
};

// Status property
const statusProperty = {
  status: {
    name: 'In Progress',
  },
};

// People property
const peopleProperty = {
  people: [
    {
      object: 'user',
      id: 'user-id-here',
    },
  ],
};

// Files property
const filesProperty = {
  files: [
    {
      name: 'Document.pdf',
      type: 'external',
      external: {
        url: 'https://example.com/document.pdf',
      },
    },
  ],
};

// Relation property
const relationProperty = {
  relation: [
    {
      id: 'related-page-id',
    },
  ],
};
```

## Complete Example: Task Management System

```javascript
const { Client } = require('@notionhq/client');

const notion = new Client({
  auth: process.env.NOTION_TOKEN,
});

async function createTaskManagementSystem() {
  // 1. Create a database for tasks
  const database = await notion.databases.create({
    parent: {
      type: 'page_id',
      page_id: process.env.PARENT_PAGE_ID,
    },
    title: [
      {
        type: 'text',
        text: { content: 'Task Manager' },
      },
    ],
    properties: {
      Name: {
        title: {},
      },
      Status: {
        status: {
          options: [
            { name: 'Not started', color: 'red' },
            { name: 'In progress', color: 'yellow' },
            { name: 'Completed', color: 'green' },
          ],
        },
      },
      Priority: {
        select: {
          options: [
            { name: 'High', color: 'red' },
            { name: 'Medium', color: 'yellow' },
            { name: 'Low', color: 'gray' },
          ],
        },
      },
      'Due Date': {
        date: {},
      },
      Tags: {
        multi_select: {
          options: [
            { name: 'Bug', color: 'red' },
            { name: 'Feature', color: 'blue' },
            { name: 'Documentation', color: 'purple' },
          ],
        },
      },
      Assignee: {
        people: {},
      },
    },
  });

  console.log('Database created:', database.id);

  // 2. Create a task
  const task = await notion.pages.create({
    parent: {
      database_id: database.id,
    },
    properties: {
      Name: {
        title: [
          {
            text: { content: 'Implement user authentication' },
          },
        ],
      },
      Status: {
        status: { name: 'In progress' },
      },
      Priority: {
        select: { name: 'High' },
      },
      'Due Date': {
        date: { start: '2025-02-01' },
      },
      Tags: {
        multi_select: [{ name: 'Feature' }],
      },
    },
  });

  console.log('Task created:', task.id);

  // 3. Add content to the task
  await notion.blocks.children.append({
    block_id: task.id,
    children: [
      {
        object: 'block',
        type: 'heading_2',
        heading_2: {
          rich_text: [{ text: { content: 'Task Details' } }],
        },
      },
      {
        object: 'block',
        type: 'paragraph',
        paragraph: {
          rich_text: [
            {
              text: {
                content: 'Implement OAuth 2.0 authentication for user login.',
              },
            },
          ],
        },
      },
      {
        object: 'block',
        type: 'heading_3',
        heading_3: {
          rich_text: [{ text: { content: 'Requirements' } }],
        },
      },
      {
        object: 'block',
        type: 'to_do',
        to_do: {
          rich_text: [{ text: { content: 'Set up OAuth provider' } }],
          checked: false,
        },
      },
      {
        object: 'block',
        type: 'to_do',
        to_do: {
          rich_text: [{ text: { content: 'Create login page' } }],
          checked: false,
        },
      },
      {
        object: 'block',
        type: 'to_do',
        to_do: {
          rich_text: [{ text: { content: 'Test authentication flow' } }],
          checked: false,
        },
      },
    ],
  });

  // 4. Add a comment
  await notion.comments.create({
    parent: { page_id: task.id },
    rich_text: [
      {
        text: { content: 'Starting work on this task today.' },
      },
    ],
  });

  // 5. Query all high priority tasks
  const highPriorityTasks = await notion.databases.query({
    database_id: database.id,
    filter: {
      property: 'Priority',
      select: {
        equals: 'High',
      },
    },
    sorts: [
      {
        property: 'Due Date',
        direction: 'ascending',
      },
    ],
  });

  console.log('High priority tasks:', highPriorityTasks.results.length);

  // 6. Update task status
  await notion.pages.update({
    page_id: task.id,
    properties: {
      Status: {
        status: { name: 'Completed' },
      },
    },
  });

  console.log('Task marked as completed');
}

// Run the example
createTaskManagementSystem().catch(console.error);
```

## Useful Links

- Documentation: https://developers.notion.com/
- API Reference: https://developers.notion.com/reference/
- SDK Repository: https://github.com/makenotion/notion-sdk-js
- NPM Package: https://www.npmjs.com/package/@notionhq/client
- Create Integration: https://www.notion.com/my-integrations
- Community: Notion Developers Slack
