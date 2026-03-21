---
name: search
description: "Meilisearch JavaScript SDK coding guidelines for full-text search and indexing"
metadata:
  languages: "javascript"
  versions: "0.53.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "meilisearch,search,full-text,indexing,instant"
---

# Meilisearch JavaScript SDK Coding Guidelines

You are a Meilisearch API coding expert. Help me with writing code using the Meilisearch API calling the official libraries and SDKs.

You can find the official SDK documentation and code samples here:
https://meilisearch.github.io/meilisearch-js/

## Golden Rule: Use the Correct and Current SDK

Always use the Meilisearch JavaScript SDK to call Meilisearch, which is the standard library for all Meilisearch API interactions. Do not use legacy libraries or unofficial SDKs.

- **Library Name:** Meilisearch JavaScript SDK
- **NPM Package:** `meilisearch`
- **Legacy Libraries**: Other unofficial packages are not recommended

**Installation:**

- **Correct:** `npm install meilisearch`

**APIs and Usage:**

- **Correct:** `import { MeiliSearch } from 'meilisearch'`
- **Correct:** `const client = new MeiliSearch({ host, apiKey })`
- **Correct:** `await client.index('indexName').search('query')`
- **Correct:** `await client.index('indexName').addDocuments(documents)`
- **Incorrect:** `MeilisearchClient` or `MeiliSearchAPI`
- **Incorrect:** Legacy v0.x packages

## Installation

```bash
npm install meilisearch
```

For browser environments, you can use a CDN:

```html
<script src="https://cdn.jsdelivr.net/npm/meilisearch@latest/dist/bundles/meilisearch.umd.js"></script>
```

## Initialization

The `meilisearch` library requires creating a `MeiliSearch` instance for all API calls.

### Basic Initialization

```javascript
import { MeiliSearch } from 'meilisearch';

const client = new MeiliSearch({
  host: 'http://127.0.0.1:7700',
  apiKey: 'masterKey',
});
```

### Using Environment Variables

For production applications, always use environment variables to store API keys:

```javascript
import { MeiliSearch } from 'meilisearch';

const client = new MeiliSearch({
  host: process.env.MEILISEARCH_HOST || 'http://127.0.0.1:7700',
  apiKey: process.env.MEILISEARCH_API_KEY,
});
```

### CommonJS Import

```javascript
const { MeiliSearch } = require('meilisearch');

const client = new MeiliSearch({
  host: 'http://127.0.0.1:7700',
  apiKey: 'masterKey',
});
```

### Browser Initialization

```javascript
const client = new window.MeiliSearch({
  host: 'http://127.0.0.1:7700',
  apiKey: 'searchOnlyApiKey', // Use search-only key for frontend
});
```

## API Keys and Security

Meilisearch uses API keys for authentication. There are different types of API keys for different use cases.

### Master Key

The master key has full access to all endpoints and should only be used server-side:

```javascript
const adminClient = new MeiliSearch({
  host: 'http://127.0.0.1:7700',
  apiKey: process.env.MEILISEARCH_MASTER_KEY,
});
```

### Default Search API Key

Use the search-only key for frontend applications to limit access:

```javascript
const searchClient = new MeiliSearch({
  host: 'http://127.0.0.1:7700',
  apiKey: process.env.MEILISEARCH_SEARCH_KEY,
});
```

### Creating Custom API Keys

```javascript
const key = await client.createKey({
  description: 'Search key for products index',
  actions: ['search'],
  indexes: ['products'],
  expiresAt: new Date('2025-12-31'),
});

console.log(key.key); // Use this key in your application
```

### Getting All API Keys

```javascript
const keys = await client.getKeys();
console.log(keys.results);
```

### Deleting an API Key

```javascript
await client.deleteKey('your-key-uid');
```

## Indexes

Indexes are where your documents are stored and searched.

### Creating an Index

```javascript
const index = await client.createIndex('movies', { primaryKey: 'id' });
console.log(index.uid); // 'movies'
```

### Getting an Index

```javascript
const index = client.index('movies');
```

### Listing All Indexes

```javascript
const indexes = await client.getIndexes();
console.log(indexes.results);
```

### Updating an Index

```javascript
await client.index('movies').update({ primaryKey: 'movie_id' });
```

### Deleting an Index

```javascript
await client.deleteIndex('movies');
```

### Getting Index Stats

```javascript
const stats = await client.index('movies').getStats();
console.log(stats.numberOfDocuments);
console.log(stats.isIndexing);
```

## Documents

Documents are the individual records in your index.

### Adding Documents

```javascript
const documents = [
  { id: 1, title: 'Inception', genre: 'Sci-Fi', year: 2010 },
  { id: 2, title: 'The Dark Knight', genre: 'Action', year: 2008 },
  { id: 3, title: 'Interstellar', genre: 'Sci-Fi', year: 2014 },
];

const response = await client.index('movies').addDocuments(documents);
console.log(response.taskUid); // Use this to check task status
```

### Adding Documents with Auto-ID Generation

If documents don't have an ID, Meilisearch can generate them:

```javascript
const documents = [
  { title: 'Movie without ID', genre: 'Drama' },
];

await client.index('movies').addDocuments(documents);
```

### Adding Documents with Primary Key

Specify a custom primary key field:

```javascript
const documents = [
  { movie_id: 'mv001', title: 'Inception' },
  { movie_id: 'mv002', title: 'The Matrix' },
];

await client.index('movies').addDocuments(documents, { primaryKey: 'movie_id' });
```

### Updating Documents

Update existing documents (replaces entire document):

```javascript
const documents = [
  { id: 1, title: 'Inception', genre: 'Sci-Fi', year: 2010, rating: 8.8 },
];

await client.index('movies').updateDocuments(documents);
```

### Getting a Document

```javascript
const document = await client.index('movies').getDocument(1);
console.log(document.title);
```

### Getting Documents with Pagination

```javascript
const documents = await client.index('movies').getDocuments({
  offset: 0,
  limit: 20,
});

console.log(documents.results);
```

### Getting Documents with Field Selection

```javascript
const documents = await client.index('movies').getDocuments({
  fields: ['id', 'title', 'year'],
  limit: 10,
});
```

### Deleting a Single Document

```javascript
await client.index('movies').deleteDocument(1);
```

### Deleting Multiple Documents

```javascript
await client.index('movies').deleteDocuments([1, 2, 3]);
```

### Deleting Documents by Filter

```javascript
await client.index('movies').deleteDocuments({
  filter: 'year < 2000',
});
```

### Deleting All Documents

```javascript
await client.index('movies').deleteAllDocuments();
```

## Search

The core functionality of Meilisearch is search.

### Basic Search

```javascript
const results = await client.index('movies').search('inception');

console.log(results.hits); // Array of matching documents
console.log(results.query); // The search query
console.log(results.processingTimeMs); // Search duration
console.log(results.estimatedTotalHits); // Number of matches
```

### Search with Options

```javascript
const results = await client.index('movies').search('batman', {
  limit: 10,
  offset: 0,
  attributesToRetrieve: ['id', 'title', 'year'],
  attributesToHighlight: ['title'],
  attributesToCrop: ['overview'],
  cropLength: 20,
  filter: 'year > 2000',
  sort: ['year:desc'],
  matchingStrategy: 'last',
});

console.log(results.hits);
```

### Filtering

Configure filterable attributes first:

```javascript
await client.index('movies').updateFilterableAttributes([
  'genre',
  'year',
  'rating',
]);
```

Then use filters in search:

```javascript
// Exact match
const sciFi = await client.index('movies').search('space', {
  filter: 'genre = "Sci-Fi"',
});

// Numeric comparison
const recent = await client.index('movies').search('', {
  filter: 'year > 2015',
});

// Range filter
const highRated = await client.index('movies').search('', {
  filter: 'rating 8 TO 10',
});

// Multiple filters with AND
const filtered = await client.index('movies').search('', {
  filter: 'genre = "Action" AND year > 2010',
});

// Multiple filters with OR
const multiGenre = await client.index('movies').search('', {
  filter: 'genre = "Action" OR genre = "Sci-Fi"',
});

// Array syntax for AND
const arrayFilter = await client.index('movies').search('', {
  filter: [
    'genre = "Action"',
    'year > 2010',
  ],
});

// Nested array syntax for OR within AND
const complexFilter = await client.index('movies').search('', {
  filter: [
    ['genre = "Action"', 'genre = "Sci-Fi"'],
    'year > 2010',
  ],
});
```

### Sorting

Configure sortable attributes first:

```javascript
await client.index('movies').updateSortableAttributes([
  'year',
  'rating',
  'title',
]);
```

Then use sort in search:

```javascript
// Sort by single attribute
const byYear = await client.index('movies').search('', {
  sort: ['year:desc'],
});

// Sort by multiple attributes
const multiSort = await client.index('movies').search('', {
  sort: ['rating:desc', 'year:desc'],
});
```

### Pagination

```javascript
const page1 = await client.index('movies').search('action', {
  limit: 20,
  offset: 0,
});

const page2 = await client.index('movies').search('action', {
  limit: 20,
  offset: 20,
});

// Using hitsPerPage and page (alternative pagination)
const results = await client.index('movies').search('action', {
  hitsPerPage: 20,
  page: 1,
});
```

### Faceted Search

Configure facets:

```javascript
await client.index('movies').updateFilterableAttributes([
  'genre',
  'year',
  'director',
]);
```

Search with facets:

```javascript
const results = await client.index('movies').search('classic', {
  facets: ['genre', 'year', 'director'],
});

console.log(results.facetDistribution);
// {
//   genre: { 'Sci-Fi': 42, 'Action': 38, 'Drama': 25 },
//   year: { '2020': 10, '2019': 15, '2018': 12 },
//   director: { 'Nolan': 5, 'Spielberg': 8 }
// }

console.log(results.facetStats);
// { year: { min: 1980, max: 2024 } }
```

### Facet Search

Search within facet values:

```javascript
const results = await client.index('movies').searchForFacetValues({
  facetName: 'genre',
  facetQuery: 'sci',
  filter: 'year > 2010',
});

console.log(results.facetHits);
// [{ value: 'Sci-Fi', count: 42 }]
```

### Highlighting and Cropping

```javascript
const results = await client.index('movies').search('space exploration', {
  attributesToHighlight: ['title', 'overview'],
  attributesToCrop: ['overview'],
  cropLength: 30,
  cropMarker: '...',
  highlightPreTag: '<mark>',
  highlightPostTag: '</mark>',
});

console.log(results.hits[0]._formatted.title);
// "The <mark>Space</mark> Between Us"
console.log(results.hits[0]._formatted.overview);
// "...astronaut on a mission of <mark>space</mark> <mark>exploration</mark>..."
```

### Geosearch

Add documents with `_geo` field:

```javascript
const restaurants = [
  {
    id: 1,
    name: "Joe's Pizza",
    _geo: { lat: 40.7484, lng: -73.9857 },
  },
  {
    id: 2,
    name: "Pasta Palace",
    _geo: { lat: 40.7589, lng: -73.9851 },
  },
];

await client.index('restaurants').addDocuments(restaurants);
```

Configure `_geo` as filterable:

```javascript
await client.index('restaurants').updateFilterableAttributes(['_geo']);
```

Search by geo radius:

```javascript
const results = await client.index('restaurants').search('pizza', {
  filter: '_geoRadius(40.7484, -73.9857, 5000)', // 5km radius
});
```

Search by geo bounding box:

```javascript
const results = await client.index('restaurants').search('', {
  filter: '_geoBoundingBox([40.7590, -73.9850], [40.7480, -73.9860])',
});
```

Sort by distance from a point:

```javascript
await client.index('restaurants').updateSortableAttributes(['_geo']);

const results = await client.index('restaurants').search('', {
  sort: ['_geoPoint(40.7484, -73.9857):asc'],
});

console.log(results.hits[0]._geoDistance); // Distance in meters
```

## Multi-Search

Perform multiple searches in a single request:

```javascript
const results = await client.multiSearch({
  queries: [
    {
      indexUid: 'movies',
      q: 'batman',
      filter: 'year > 2000',
    },
    {
      indexUid: 'movies',
      q: 'superman',
      limit: 5,
    },
    {
      indexUid: 'books',
      q: 'javascript',
    },
  ],
});

console.log(results.results[0].hits); // Batman search results
console.log(results.results[1].hits); // Superman search results
console.log(results.results[2].hits); // JavaScript books results
```

### Federated Search

Merge results from multiple queries into a single list:

```javascript
const results = await client.multiSearch({
  federation: {
    limit: 20,
    offset: 0,
  },
  queries: [
    {
      indexUid: 'movies',
      q: 'batman',
    },
    {
      indexUid: 'comics',
      q: 'batman',
    },
    {
      indexUid: 'books',
      q: 'batman',
    },
  ],
});

console.log(results.hits); // Merged results from all three indexes
console.log(results.facetsByIndex); // Facets grouped by index
```

## Index Settings

Settings control how Meilisearch processes and ranks search results.

### Get All Settings

```javascript
const settings = await client.index('movies').getSettings();
console.log(settings);
```

### Update All Settings

```javascript
await client.index('movies').updateSettings({
  searchableAttributes: ['title', 'overview', 'genre'],
  filterableAttributes: ['genre', 'year', 'rating'],
  sortableAttributes: ['year', 'rating'],
  rankingRules: [
    'words',
    'typo',
    'proximity',
    'attribute',
    'sort',
    'exactness',
  ],
  stopWords: ['the', 'a', 'an'],
  synonyms: {
    'wolverine': ['logan', 'weapon x'],
    'batman': ['dark knight', 'caped crusader'],
  },
  distinctAttribute: 'movie_id',
  typoTolerance: {
    enabled: true,
    minWordSizeForTypos: {
      oneTypo: 5,
      twoTypos: 9,
    },
  },
  pagination: {
    maxTotalHits: 1000,
  },
});
```

### Reset All Settings

```javascript
await client.index('movies').resetSettings();
```

### Searchable Attributes

Defines which fields are searchable and their ranking order:

```javascript
await client.index('movies').updateSearchableAttributes([
  'title',
  'overview',
  'genre',
]);
```

Get searchable attributes:

```javascript
const attrs = await client.index('movies').getSearchableAttributes();
```

Reset to default (all attributes):

```javascript
await client.index('movies').resetSearchableAttributes();
```

### Filterable Attributes

Defines which fields can be used in filters and facets:

```javascript
await client.index('movies').updateFilterableAttributes([
  'genre',
  'year',
  'rating',
  'director',
]);
```

### Sortable Attributes

Defines which fields can be used for sorting:

```javascript
await client.index('movies').updateSortableAttributes([
  'year',
  'rating',
  'title',
]);
```

### Ranking Rules

Controls how results are ranked:

```javascript
await client.index('movies').updateRankingRules([
  'words',
  'typo',
  'proximity',
  'attribute',
  'sort',
  'exactness',
  'rating:desc', // Custom ranking rule
]);
```

Get ranking rules:

```javascript
const rules = await client.index('movies').getRankingRules();
```

Reset to default:

```javascript
await client.index('movies').resetRankingRules();
```

### Stop Words

Words ignored during search:

```javascript
await client.index('movies').updateStopWords([
  'the',
  'a',
  'an',
  'and',
  'or',
  'but',
]);
```

### Synonyms

Configure word equivalences:

```javascript
await client.index('movies').updateSynonyms({
  'wolverine': ['logan', 'weapon x'],
  'batman': ['dark knight', 'caped crusader'],
  'sw': ['star wars'],
});
```

### Distinct Attribute

Return only one document per distinct value:

```javascript
await client.index('movies').updateDistinctAttribute('movie_id');
```

### Typo Tolerance

Configure typo tolerance settings:

```javascript
await client.index('movies').updateTypoTolerance({
  enabled: true,
  minWordSizeForTypos: {
    oneTypo: 5,
    twoTypos: 9,
  },
  disableOnWords: ['spiderman', 'batman'],
  disableOnAttributes: ['title'],
});
```

Get typo tolerance settings:

```javascript
const settings = await client.index('movies').getTypoTolerance();
```

### Faceting Settings

Configure faceting behavior:

```javascript
await client.index('movies').updateFaceting({
  maxValuesPerFacet: 100,
  sortFacetValuesBy: {
    '*': 'alpha',
    'genre': 'count',
  },
});
```

### Pagination Settings

Configure maximum search results:

```javascript
await client.index('movies').updatePagination({
  maxTotalHits: 1000,
});
```

### Displayed Attributes

Control which fields are returned in search results:

```javascript
await client.index('movies').updateDisplayedAttributes([
  'id',
  'title',
  'year',
  'genre',
  'rating',
]);
```

## Tasks

All operations in Meilisearch are asynchronous and return a task.

### Get Task by UID

```javascript
const task = await client.getTask(12);

console.log(task.status); // 'enqueued', 'processing', 'succeeded', 'failed', 'canceled'
console.log(task.type); // 'documentAdditionOrUpdate', 'settingsUpdate', etc.
console.log(task.details);
console.log(task.duration);
```

### Get Tasks

```javascript
const tasks = await client.getTasks({
  limit: 20,
  from: 0,
});

console.log(tasks.results);
```

### Get Tasks with Filters

```javascript
const tasks = await client.getTasks({
  indexUids: ['movies'],
  statuses: ['succeeded', 'failed'],
  types: ['documentAdditionOrUpdate'],
  limit: 50,
});
```

### Wait for Task

```javascript
const response = await client.index('movies').addDocuments(documents);

// Wait for the task to complete
const task = await client.waitForTask(response.taskUid);

if (task.status === 'succeeded') {
  console.log('Documents added successfully');
} else if (task.status === 'failed') {
  console.error('Task failed:', task.error);
}
```

### Wait for Multiple Tasks

```javascript
await client.waitForTasks([taskUid1, taskUid2, taskUid3]);
```

### Cancel Tasks

```javascript
await client.cancelTasks({
  uids: [12, 13, 14],
});

// Cancel by filter
await client.cancelTasks({
  statuses: ['enqueued'],
  indexUids: ['movies'],
});
```

### Delete Tasks

```javascript
await client.deleteTasks({
  uids: [12, 13, 14],
});

// Delete by filter
await client.deleteTasks({
  statuses: ['succeeded', 'failed'],
  beforeEnqueuedAt: new Date('2024-01-01'),
});
```

## Tenant Tokens

Tenant tokens provide secure, scoped access for multi-tenant applications.

### Generating Tenant Tokens

```javascript
import { MeiliSearch } from 'meilisearch';

// Server-side code
const client = new MeiliSearch({
  host: 'http://127.0.0.1:7700',
  apiKey: 'YOUR_MASTER_KEY',
});

const apiKey = await client.getKey('YOUR_API_KEY_UID');

const searchRules = {
  'patient_medical_records': {
    filter: 'user_id = 42',
  },
};

const token = await client.generateTenantToken(
  apiKey.uid,
  searchRules,
  {
    apiKey: apiKey.key,
    expiresAt: new Date('2025-12-31'),
  }
);

// Send this token to the frontend
```

### Using Tenant Tokens

```javascript
// Frontend code
const frontendClient = new MeiliSearch({
  host: 'http://127.0.0.1:7700',
  apiKey: tenantToken, // Token from backend
});

const results = await frontendClient.index('patient_medical_records').search('blood test');
// Only returns documents where user_id = 42
```

### Multi-Index Tenant Tokens

```javascript
const searchRules = {
  'products': {
    filter: 'company_id = 123',
  },
  'orders': {
    filter: 'company_id = 123',
  },
  'invoices': {
    filter: 'company_id = 123',
  },
};

const token = await client.generateTenantToken(
  apiKeyUid,
  searchRules,
  { apiKey: apiKey.key }
);
```

### Wildcard Tenant Tokens

```javascript
const searchRules = {
  '*': {
    filter: 'tenant_id = 456',
  },
};

const token = await client.generateTenantToken(
  apiKeyUid,
  searchRules,
  { apiKey: apiKey.key }
);
```

## Health and Stats

### Check Health

```javascript
const health = await client.health();
console.log(health.status); // 'available'
```

### Get Version

```javascript
const version = await client.getVersion();
console.log(version.pkgVersion); // '1.12.0'
```

### Get Database Stats

```javascript
const stats = await client.getStats();
console.log(stats.databaseSize);
console.log(stats.lastUpdate);
console.log(stats.indexes);
```

## Error Handling

```javascript
try {
  const results = await client.index('movies').search('batman');
  console.log(results.hits);
} catch (error) {
  if (error.message.includes('index_not_found')) {
    console.error('Index does not exist');
  } else if (error.message.includes('invalid_api_key')) {
    console.error('Invalid API key');
  } else {
    console.error('Search error:', error.message);
  }
}
```

### Handling Task Failures

```javascript
const response = await client.index('movies').addDocuments(documents);
const task = await client.waitForTask(response.taskUid);

if (task.status === 'failed') {
  console.error('Error code:', task.error.code);
  console.error('Error type:', task.error.type);
  console.error('Error message:', task.error.message);
  console.error('Error link:', task.error.link);
}
```

## Advanced Examples

### Full-Text Search with Filters and Sorting

```javascript
await client.index('movies').updateSettings({
  searchableAttributes: ['title', 'overview', 'genre'],
  filterableAttributes: ['genre', 'year', 'rating', 'director'],
  sortableAttributes: ['year', 'rating'],
  rankingRules: ['words', 'typo', 'proximity', 'attribute', 'sort', 'exactness'],
});

const results = await client.index('movies').search('space adventure', {
  filter: [
    ['genre = "Sci-Fi"', 'genre = "Adventure"'],
    'year > 2010',
    'rating >= 7.0',
  ],
  sort: ['rating:desc', 'year:desc'],
  attributesToRetrieve: ['title', 'year', 'rating', 'overview'],
  attributesToHighlight: ['title', 'overview'],
  attributesToCrop: ['overview'],
  cropLength: 50,
  limit: 20,
});
```

### E-commerce Product Search

```javascript
const products = [
  {
    id: 1,
    name: 'Laptop Pro 15',
    category: 'Electronics',
    brand: 'TechCorp',
    price: 1299.99,
    rating: 4.5,
    inStock: true,
  },
  {
    id: 2,
    name: 'Wireless Mouse',
    category: 'Electronics',
    brand: 'TechCorp',
    price: 29.99,
    rating: 4.8,
    inStock: true,
  },
];

await client.index('products').addDocuments(products);

await client.index('products').updateSettings({
  filterableAttributes: ['category', 'brand', 'price', 'rating', 'inStock'],
  sortableAttributes: ['price', 'rating'],
  rankingRules: ['words', 'typo', 'proximity', 'attribute', 'sort', 'exactness'],
});

const results = await client.index('products').search('laptop', {
  filter: [
    'category = "Electronics"',
    'inStock = true',
    'price 0 TO 1500',
  ],
  sort: ['rating:desc'],
  facets: ['brand', 'category'],
  limit: 20,
});
```

### Location-Based Restaurant Search

```javascript
const restaurants = [
  {
    id: 1,
    name: 'Italian Bistro',
    cuisine: 'Italian',
    rating: 4.5,
    priceRange: '$$',
    _geo: { lat: 40.7589, lng: -73.9851 },
  },
  {
    id: 2,
    name: 'Sushi Palace',
    cuisine: 'Japanese',
    rating: 4.8,
    priceRange: '$$$',
    _geo: { lat: 40.7484, lng: -73.9857 },
  },
];

await client.index('restaurants').addDocuments(restaurants);

await client.index('restaurants').updateSettings({
  filterableAttributes: ['cuisine', 'rating', 'priceRange', '_geo'],
  sortableAttributes: ['rating', '_geo'],
});

const results = await client.index('restaurants').search('', {
  filter: [
    '_geoRadius(40.7580, -73.9855, 2000)', // 2km radius
    'rating >= 4.0',
  ],
  sort: ['_geoPoint(40.7580, -73.9855):asc'],
  limit: 10,
});

results.hits.forEach(hit => {
  console.log(`${hit.name} - ${hit._geoDistance}m away`);
});
```

### Real-Time Search with Autocomplete

```javascript
let timeoutId;

function handleSearchInput(query) {
  clearTimeout(timeoutId);

  timeoutId = setTimeout(async () => {
    const results = await client.index('movies').search(query, {
      limit: 5,
      attributesToRetrieve: ['id', 'title', 'year'],
      attributesToHighlight: ['title'],
    });

    displayResults(results.hits);
  }, 300); // Debounce for 300ms
}
```

### Bulk Document Operations

```javascript
// Process large datasets in batches
const batchSize = 1000;
const allDocuments = [...]; // Your large dataset

for (let i = 0; i < allDocuments.length; i += batchSize) {
  const batch = allDocuments.slice(i, i + batchSize);
  const response = await client.index('movies').addDocuments(batch);
  await client.waitForTask(response.taskUid);
  console.log(`Processed ${i + batch.length} documents`);
}
```

### Search with Custom Ranking

```javascript
await client.index('movies').updateSettings({
  rankingRules: [
    'words',
    'typo',
    'proximity',
    'attribute',
    'sort',
    'exactness',
    'rating:desc', // Boost higher-rated content
    'year:desc',   // Prefer newer content
  ],
});

const results = await client.index('movies').search('action');
```

## Useful Links

- Official Documentation: https://www.meilisearch.com/docs
- JavaScript SDK Documentation: https://meilisearch.github.io/meilisearch-js/
- API Reference: https://www.meilisearch.com/docs/reference/api/overview
- GitHub Repository: https://github.com/meilisearch/meilisearch-js
- Cloud Hosting: https://www.meilisearch.com/cloud
