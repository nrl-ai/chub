---
name: issues
description: "Jira JavaScript/TypeScript SDK Coding Guidelines for writing code using the Jira API with official libraries and SDKs"
metadata:
  languages: "javascript"
  versions: "5.2.2"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "jira,issues,atlassian,project-management,agile"
---

# Jira JavaScript/TypeScript SDK Coding Guidelines

You are a Jira API coding expert. Help me with writing code using the Jira API calling the official libraries and SDKs.

You can find the official Jira API documentation and code samples here:
https://developer.atlassian.com/cloud/jira/platform/rest/v3/
https://mrrefactoring.github.io/jira.js/

## Golden Rule: Use the Correct and Current SDK

Always use the jira.js library to interact with the Jira Cloud, Jira Agile, and Jira Service Desk REST APIs. This is the most comprehensive and actively maintained JavaScript/TypeScript wrapper for Jira APIs.

- **Library Name:** jira.js
- **NPM Package:** `jira.js`
- **Minimum Node.js Version:** 20.0.0 or newer
- **Legacy Libraries**: `jira-connector`, `jira-client`, and other unofficial packages are not recommended

**Installation:**

- **Correct:** `npm install jira.js`
- **Correct:** `yarn add jira.js`
- **Correct:** `pnpm add jira.js`

**APIs and Usage:**

- **Correct:** `import { Version3Client } from 'jira.js'`
- **Correct:** `const client = new Version3Client({ host, authentication })`
- **Correct:** `await client.issues.createIssue(...)`
- **Correct:** `await client.issueSearch.searchForIssuesUsingJql(...)`
- **Incorrect:** `JiraClient`, `JiraApi`, or `JiraConnector`
- **Incorrect:** Using REST API endpoints directly without the SDK
- **Incorrect:** Using deprecated v1 or v2 API versions

## Authentication

The jira.js library supports multiple authentication methods. Always obtain API credentials from your Jira Cloud instance.

### Basic Authentication (Recommended for Jira Cloud)

Generate an API token at: https://id.atlassian.com/manage-profile/security/api-tokens

```javascript
import { Version3Client } from 'jira.js';

const client = new Version3Client({
  host: 'https://your-domain.atlassian.net',
  authentication: {
    basic: {
      email: 'your@email.com',
      apiToken: process.env.JIRA_API_TOKEN,
    },
  },
});
```

### OAuth 2.0 Authentication

OAuth 2.0 uses an access token obtained through the OAuth flow:

```javascript
import { Version3Client } from 'jira.js';

const client = new Version3Client({
  host: 'https://your-domain.atlassian.net',
  authentication: {
    oauth2: {
      accessToken: process.env.JIRA_ACCESS_TOKEN,
    },
  },
});
```

### OAuth 1.0a Authentication (Legacy)

For Jira Server instances or legacy integrations:

```javascript
import { Version3Client } from 'jira.js';

const client = new Version3Client({
  host: 'https://your-domain.atlassian.net',
  authentication: {
    oauth: {
      consumerKey: process.env.JIRA_CONSUMER_KEY,
      consumerSecret: process.env.JIRA_CONSUMER_SECRET,
      accessToken: process.env.JIRA_ACCESS_TOKEN,
      tokenSecret: process.env.JIRA_TOKEN_SECRET,
    },
  },
});
```

### Environment Variables Setup

Create a `.env` file in your project root:

```bash
JIRA_HOST=https://your-domain.atlassian.net
JIRA_EMAIL=your@email.com
JIRA_API_TOKEN=your_api_token_here
```

Load environment variables in your code:

```javascript
import 'dotenv/config';
import { Version3Client } from 'jira.js';

const client = new Version3Client({
  host: process.env.JIRA_HOST,
  authentication: {
    basic: {
      email: process.env.JIRA_EMAIL,
      apiToken: process.env.JIRA_API_TOKEN,
    },
  },
});
```

## Initialization

Create a client instance for all API interactions:

```javascript
import { Version3Client } from 'jira.js';

// Basic initialization
const client = new Version3Client({
  host: 'https://your-domain.atlassian.net',
  authentication: {
    basic: {
      email: 'your@email.com',
      apiToken: 'your_api_token',
    },
  },
});

// With custom timeout and other options
const clientWithOptions = new Version3Client({
  host: 'https://your-domain.atlassian.net',
  authentication: {
    basic: {
      email: 'your@email.com',
      apiToken: 'your_api_token',
    },
  },
  timeout: 30000, // 30 seconds
});
```

## Error Handling

The library provides two error types for comprehensive error handling:

```javascript
import { Version3Client } from 'jira.js';
import { AxiosError } from 'axios';

const client = new Version3Client({
  host: 'https://your-domain.atlassian.net',
  authentication: {
    basic: {
      email: process.env.JIRA_EMAIL,
      apiToken: process.env.JIRA_API_TOKEN,
    },
  },
});

try {
  const issue = await client.issues.getIssue({ issueIdOrKey: 'PROJECT-123' });
  console.log(issue);
} catch (error) {
  if (error.name === 'HttpException') {
    // Server errors with parsed details
    console.error('HTTP Error:', error.statusCode, error.message);
    console.error('Error details:', error.data);
  } else if (error instanceof AxiosError) {
    // Network or configuration errors
    console.error('Network Error:', error.code, error.message);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

## Core API Surfaces

### Projects

#### Get Project

```javascript
// Minimal example
const project = await client.projects.getProject({
  projectIdOrKey: 'PROJ',
});

console.log(project.name);
console.log(project.key);
console.log(project.id);
```

#### Advanced: Get Project with Expanded Properties

```javascript
// Advanced example with expand options
const project = await client.projects.getProject({
  projectIdOrKey: 'PROJ',
  expand: 'description,lead,issueTypes,url,projectKeys',
  properties: '*all',
});

console.log('Project Name:', project.name);
console.log('Project Lead:', project.lead.displayName);
console.log('Description:', project.description);
console.log('Issue Types:', project.issueTypes);
```

#### Search Projects

```javascript
// Search for projects
const projects = await client.projects.searchProjects({
  startAt: 0,
  maxResults: 50,
  orderBy: 'name',
  query: 'development',
});

console.log('Total projects:', projects.total);
projects.values.forEach(project => {
  console.log(`${project.key}: ${project.name}`);
});
```

#### Create Project

```javascript
// Create a new project
const newProject = await client.projects.createProject({
  key: 'NEWP',
  name: 'New Project',
  projectTypeKey: 'software',
  projectTemplateKey: 'com.atlassian.jira-core-project-templates:jira-core-simplified-project-management',
  description: 'This is a new project',
  leadAccountId: '5b10a2844c20165700ede21g',
});

console.log('Created project:', newProject.key);
```

### Issues

#### Create Issue

```javascript
// Minimal example
const issue = await client.issues.createIssue({
  fields: {
    project: {
      key: 'PROJ',
    },
    summary: 'New issue from API',
    issuetype: {
      name: 'Task',
    },
  },
});

console.log('Created issue:', issue.key);
```

#### Advanced: Create Issue with All Fields

```javascript
// Advanced example with multiple fields
const issue = await client.issues.createIssue({
  fields: {
    project: {
      key: 'PROJ',
    },
    summary: 'Implement user authentication',
    description: {
      type: 'doc',
      version: 1,
      content: [
        {
          type: 'paragraph',
          content: [
            {
              type: 'text',
              text: 'Need to implement OAuth 2.0 authentication for the API.',
            },
          ],
        },
      ],
    },
    issuetype: {
      name: 'Story',
    },
    priority: {
      name: 'High',
    },
    labels: ['security', 'authentication'],
    assignee: {
      accountId: '5b10a2844c20165700ede21g',
    },
    duedate: '2025-12-31',
    customfield_10001: 'Custom field value',
  },
});

console.log('Created issue:', issue.key, issue.id);
```

#### Get Issue

```javascript
// Get an issue with all fields
const issue = await client.issues.getIssue({
  issueIdOrKey: 'PROJ-123',
});

console.log('Summary:', issue.fields.summary);
console.log('Status:', issue.fields.status.name);
console.log('Assignee:', issue.fields.assignee?.displayName);
```

#### Advanced: Get Issue with Specific Fields

```javascript
// Get specific fields and expand certain properties
const issue = await client.issues.getIssue({
  issueIdOrKey: 'PROJ-123',
  fields: ['summary', 'status', 'assignee', 'priority', 'created', 'updated'],
  expand: 'changelog,renderedFields',
});

console.log('Issue:', issue.key);
console.log('Summary:', issue.fields.summary);
console.log('Created:', issue.fields.created);
console.log('Change History:', issue.changelog);
```

#### Update Issue

```javascript
// Update issue fields
await client.issues.editIssue({
  issueIdOrKey: 'PROJ-123',
  fields: {
    summary: 'Updated summary',
    description: {
      type: 'doc',
      version: 1,
      content: [
        {
          type: 'paragraph',
          content: [
            {
              type: 'text',
              text: 'Updated description',
            },
          ],
        },
      ],
    },
    priority: {
      name: 'Critical',
    },
  },
});

console.log('Issue updated successfully');
```

#### Assign Issue

```javascript
// Assign issue to a user
await client.issues.assignIssue({
  issueIdOrKey: 'PROJ-123',
  accountId: '5b10a2844c20165700ede21g',
});
```

#### Add Comment

```javascript
// Add a comment to an issue
const comment = await client.issueComments.addComment({
  issueIdOrKey: 'PROJ-123',
  body: {
    type: 'doc',
    version: 1,
    content: [
      {
        type: 'paragraph',
        content: [
          {
            type: 'text',
            text: 'This is a comment from the API',
          },
        ],
      },
    ],
  },
});

console.log('Comment added:', comment.id);
```

#### Transition Issue

```javascript
// Get available transitions
const transitions = await client.issues.getTransitions({
  issueIdOrKey: 'PROJ-123',
});

console.log('Available transitions:');
transitions.transitions.forEach(t => {
  console.log(`${t.id}: ${t.name}`);
});

// Transition issue to a new status
await client.issues.doTransition({
  issueIdOrKey: 'PROJ-123',
  transition: {
    id: '21', // ID from getTransitions
  },
});

console.log('Issue transitioned successfully');
```

#### Delete Issue

```javascript
// Delete an issue
await client.issues.deleteIssue({
  issueIdOrKey: 'PROJ-123',
  deleteSubtasks: 'true',
});

console.log('Issue deleted successfully');
```

### Issue Search

#### Search with JQL (Jira Query Language)

```javascript
// Basic JQL search
const searchResults = await client.issueSearch.searchForIssuesUsingJql({
  jql: 'project = PROJ AND status = "In Progress"',
});

console.log('Total issues:', searchResults.total);
searchResults.issues.forEach(issue => {
  console.log(`${issue.key}: ${issue.fields.summary}`);
});
```

#### Advanced: Search with Pagination and Sorting

```javascript
// Advanced search with pagination, sorting, and field selection
const searchResults = await client.issueSearch.searchForIssuesUsingJql({
  jql: 'project = PROJ AND assignee = currentUser() ORDER BY created DESC',
  startAt: 0,
  maxResults: 50,
  fields: ['summary', 'status', 'assignee', 'priority', 'created'],
  expand: ['changelog'],
});

console.log(`Showing ${searchResults.issues.length} of ${searchResults.total} issues`);

searchResults.issues.forEach(issue => {
  console.log(`${issue.key}: ${issue.fields.summary}`);
  console.log(`  Status: ${issue.fields.status.name}`);
  console.log(`  Priority: ${issue.fields.priority.name}`);
  console.log(`  Created: ${issue.fields.created}`);
});
```

#### Pagination Example

```javascript
// Fetch all issues with pagination
async function getAllIssues(jql) {
  const allIssues = [];
  let startAt = 0;
  const maxResults = 100;
  let total = 0;

  do {
    const searchResults = await client.issueSearch.searchForIssuesUsingJql({
      jql,
      startAt,
      maxResults,
    });

    allIssues.push(...searchResults.issues);
    total = searchResults.total;
    startAt += maxResults;

    console.log(`Fetched ${allIssues.length} of ${total} issues`);
  } while (allIssues.length < total);

  return allIssues;
}

// Usage
const allProjectIssues = await getAllIssues('project = PROJ');
console.log('Total issues fetched:', allProjectIssues.length);
```

#### Complex JQL Queries

```javascript
// Complex JQL with multiple conditions
const jql = `
  project = PROJ AND
  status IN ("To Do", "In Progress") AND
  priority IN (High, Critical) AND
  assignee = currentUser() AND
  created >= -30d
  ORDER BY priority DESC, created ASC
`;

const issues = await client.issueSearch.searchForIssuesUsingJql({
  jql: jql.trim(),
  maxResults: 100,
});

console.log('High priority issues:', issues.total);
```

### Agile - Boards

#### Get All Boards

```javascript
// Minimal example
import { AgileClient } from 'jira.js';

const agileClient = new AgileClient({
  host: 'https://your-domain.atlassian.net',
  authentication: {
    basic: {
      email: process.env.JIRA_EMAIL,
      apiToken: process.env.JIRA_API_TOKEN,
    },
  },
});

const boards = await agileClient.board.getAllBoards();

boards.values.forEach(board => {
  console.log(`${board.id}: ${board.name} (${board.type})`);
});
```

#### Advanced: Get Boards with Filters

```javascript
// Filter boards by project and type
const scrumBoards = await agileClient.board.getAllBoards({
  projectKeyOrId: 'PROJ',
  type: 'scrum',
  startAt: 0,
  maxResults: 50,
});

console.log('Scrum boards:');
scrumBoards.values.forEach(board => {
  console.log(`${board.id}: ${board.name}`);
});
```

#### Get Board

```javascript
// Get a specific board
const board = await agileClient.board.getBoard({
  boardId: 1,
});

console.log('Board name:', board.name);
console.log('Board type:', board.type);
console.log('Board location:', board.location);
```

#### Get Issues for Board

```javascript
// Get all issues on a board
const boardIssues = await agileClient.board.getIssuesForBoard({
  boardId: 1,
  startAt: 0,
  maxResults: 50,
  jql: 'status != Done',
});

console.log('Issues on board:', boardIssues.total);
boardIssues.issues.forEach(issue => {
  console.log(`${issue.key}: ${issue.fields.summary}`);
});
```

### Agile - Sprints

#### Create Sprint

```javascript
// Create a new sprint
const sprint = await agileClient.sprint.createSprint({
  name: 'Sprint 23',
  startDate: '2025-11-15T10:00:00.000Z',
  endDate: '2025-11-29T18:00:00.000Z',
  originBoardId: 1,
  goal: 'Complete user authentication feature',
});

console.log('Created sprint:', sprint.id, sprint.name);
```

#### Advanced: Create Sprint with Minimal Fields

```javascript
// Create a future sprint (only name and board required)
const futureSprint = await agileClient.sprint.createSprint({
  name: 'Q1 Sprint 1',
  originBoardId: 1,
});

console.log('Created future sprint:', futureSprint.id);
```

#### Get Sprint

```javascript
// Get sprint details
const sprint = await agileClient.sprint.getSprint({
  sprintId: 123,
});

console.log('Sprint:', sprint.name);
console.log('State:', sprint.state);
console.log('Start Date:', sprint.startDate);
console.log('End Date:', sprint.endDate);
console.log('Goal:', sprint.goal);
```

#### Update Sprint

```javascript
// Update sprint details
await agileClient.sprint.updateSprint({
  sprintId: 123,
  name: 'Updated Sprint Name',
  goal: 'Updated sprint goal',
  startDate: '2025-11-15T10:00:00.000Z',
  endDate: '2025-11-29T18:00:00.000Z',
  state: 'active',
});

console.log('Sprint updated successfully');
```

#### Get Issues for Sprint

```javascript
// Get all issues in a sprint
const sprintIssues = await agileClient.sprint.getIssuesForSprint({
  sprintId: 123,
  startAt: 0,
  maxResults: 50,
  jql: 'status != Done',
});

console.log('Issues in sprint:', sprintIssues.total);
sprintIssues.issues.forEach(issue => {
  console.log(`${issue.key}: ${issue.fields.summary}`);
});
```

#### Move Issues to Sprint

```javascript
// Move issues to a sprint (max 50 issues per operation)
await agileClient.sprint.moveIssuesToSprintAndRank({
  sprintId: 123,
  issues: ['PROJ-123', 'PROJ-124', 'PROJ-125'],
});

console.log('Issues moved to sprint successfully');
```

#### Delete Sprint

```javascript
// Delete a sprint
await agileClient.sprint.deleteSprint({
  sprintId: 123,
});

console.log('Sprint deleted successfully');
```

### Users

#### Get Current User

```javascript
// Get currently authenticated user
const currentUser = await client.myself.getCurrentUser();

console.log('User:', currentUser.displayName);
console.log('Email:', currentUser.emailAddress);
console.log('Account ID:', currentUser.accountId);
```

#### Find Users

```javascript
// Search for users by query
const users = await client.userSearch.findUsers({
  query: 'john',
  maxResults: 50,
});

users.forEach(user => {
  console.log(`${user.displayName} (${user.emailAddress})`);
});
```

#### Get User

```javascript
// Get a specific user by account ID
const user = await client.users.getUser({
  accountId: '5b10a2844c20165700ede21g',
});

console.log('User:', user.displayName);
console.log('Active:', user.active);
```

### Filters

#### Get Filters

```javascript
// Get all filters for the current user
const filters = await client.filters.getFavouriteFilters();

filters.forEach(filter => {
  console.log(`${filter.id}: ${filter.name}`);
  console.log(`  JQL: ${filter.jql}`);
});
```

#### Create Filter

```javascript
// Create a new filter
const filter = await client.filters.createFilter({
  name: 'My High Priority Issues',
  description: 'All high priority issues assigned to me',
  jql: 'assignee = currentUser() AND priority = High',
  favourite: true,
});

console.log('Created filter:', filter.id, filter.name);
```

#### Get Filter

```javascript
// Get a specific filter
const filter = await client.filters.getFilter({
  id: 10000,
  expand: 'subscriptions,owner',
});

console.log('Filter:', filter.name);
console.log('JQL:', filter.jql);
console.log('Owner:', filter.owner.displayName);
```

#### Search with Filter

```javascript
// Execute a saved filter
const filterResults = await client.issueSearch.searchForIssuesUsingJql({
  jql: `filter = ${filterId}`,
  maxResults: 100,
});

console.log('Filter results:', filterResults.total);
```

### Workflows

#### Get All Workflows

```javascript
// Get all workflows
const workflows = await client.workflows.getAllWorkflows({
  workflowName: undefined,
});

workflows.forEach(workflow => {
  console.log(`${workflow.id}: ${workflow.name}`);
});
```

#### Get Issue Transitions

```javascript
// Get available transitions for an issue
const transitions = await client.issues.getTransitions({
  issueIdOrKey: 'PROJ-123',
  expand: 'transitions.fields',
});

console.log('Available transitions:');
transitions.transitions.forEach(transition => {
  console.log(`${transition.id}: ${transition.name}`);
  console.log(`  To status: ${transition.to.name}`);
});
```

### Custom Fields

#### Get All Fields

```javascript
// Get all fields in Jira
const fields = await client.issueFields.getFields();

fields.forEach(field => {
  if (field.custom) {
    console.log(`Custom Field: ${field.id} - ${field.name}`);
  }
});
```

#### Get Custom Field Options

```javascript
// Get options for a custom field
const customFieldId = 'customfield_10001';
const issue = await client.issues.getIssue({
  issueIdOrKey: 'PROJ-123',
  fields: [customFieldId],
});

console.log('Custom field value:', issue.fields[customFieldId]);
```

### Versions (Releases)

#### Create Version

```javascript
// Create a new version/release
const version = await client.projectVersions.createVersion({
  name: 'v1.2.0',
  description: 'Q4 2025 Release',
  projectId: 10000,
  releaseDate: '2025-12-15',
  archived: false,
  released: false,
});

console.log('Created version:', version.id, version.name);
```

#### Get Version

```javascript
// Get a specific version
const version = await client.projectVersions.getVersion({
  id: '10001',
});

console.log('Version:', version.name);
console.log('Released:', version.released);
console.log('Release Date:', version.releaseDate);
```

#### Update Version

```javascript
// Update a version
await client.projectVersions.updateVersion({
  id: '10001',
  name: 'v1.2.1',
  description: 'Updated release',
  released: true,
  releaseDate: '2025-12-20',
});
```

### Labels

#### Add Labels to Issue

```javascript
// Add labels to an issue
await client.issues.editIssue({
  issueIdOrKey: 'PROJ-123',
  fields: {
    labels: ['backend', 'api', 'urgent'],
  },
});

console.log('Labels added successfully');
```

#### Get All Labels

```javascript
// Search for labels
const labels = await client.labels.getAllLabels({
  startAt: 0,
  maxResults: 1000,
});

console.log('Total labels:', labels.total);
labels.values.forEach(label => {
  console.log(label);
});
```

### Components

#### Get Project Components

```javascript
// Get all components for a project
const components = await client.projectComponents.getProjectComponents({
  projectIdOrKey: 'PROJ',
});

components.forEach(component => {
  console.log(`${component.id}: ${component.name}`);
  console.log(`  Description: ${component.description}`);
  console.log(`  Lead: ${component.lead?.displayName}`);
});
```

#### Create Component

```javascript
// Create a new component
const component = await client.projectComponents.createComponent({
  name: 'Frontend',
  description: 'Frontend components and pages',
  project: 'PROJ',
  leadAccountId: '5b10a2844c20165700ede21g',
});

console.log('Created component:', component.id, component.name);
```

### Attachments

#### Add Attachment

```javascript
import fs from 'fs';
import FormData from 'form-data';

// Add attachment to an issue
const formData = new FormData();
formData.append('file', fs.createReadStream('/path/to/file.pdf'));

const attachments = await client.issueAttachments.addAttachment({
  issueIdOrKey: 'PROJ-123',
  attachment: formData,
});

console.log('Attachment added:', attachments[0].filename);
```

#### Get Attachment

```javascript
// Get attachment metadata
const issue = await client.issues.getIssue({
  issueIdOrKey: 'PROJ-123',
  fields: ['attachment'],
});

issue.fields.attachment.forEach(attachment => {
  console.log(`${attachment.filename} - ${attachment.size} bytes`);
  console.log(`  Download: ${attachment.content}`);
});
```

### Webhooks

#### Create Webhook

```javascript
// Create a webhook
const webhook = await client.webhooks.registerDynamicWebhooks({
  url: 'https://your-domain.com/webhook',
  webhooks: [
    {
      events: ['jira:issue_created', 'jira:issue_updated'],
      jqlFilter: 'project = PROJ',
    },
  ],
});

console.log('Webhook created:', webhook);
```

#### Get Webhooks

```javascript
// Get all registered webhooks
const webhooks = await client.webhooks.getDynamicWebhooksForApp();

webhooks.forEach(webhook => {
  console.log(`Webhook ${webhook.id}:`);
  console.log(`  URL: ${webhook.url}`);
  console.log(`  Events: ${webhook.events.join(', ')}`);
});
```

### Permissions

#### Get My Permissions

```javascript
// Get current user permissions
const permissions = await client.permissions.getMyPermissions({
  projectKey: 'PROJ',
});

console.log('Permissions for project PROJ:');
Object.entries(permissions.permissions).forEach(([key, permission]) => {
  console.log(`${key}: ${permission.havePermission}`);
});
```

#### Get Bulk Permissions

```javascript
// Get permissions for multiple projects
const bulkPermissions = await client.permissions.getBulkPermissions({
  projectKey: ['PROJ1', 'PROJ2', 'PROJ3'],
  permissions: ['BROWSE_PROJECTS', 'CREATE_ISSUES', 'EDIT_ISSUES'],
});

console.log('Bulk permissions:', bulkPermissions);
```

## API Client Optimization

### Using Version 2 Client

For legacy Jira instances or when you need v2 endpoints:

```javascript
import { Version2Client } from 'jira.js';

const v2Client = new Version2Client({
  host: 'https://your-domain.atlassian.net',
  authentication: {
    basic: {
      email: process.env.JIRA_EMAIL,
      apiToken: process.env.JIRA_API_TOKEN,
    },
  },
});
```

### Using Agile Client

For dedicated Agile/Scrum operations:

```javascript
import { AgileClient } from 'jira.js';

const agileClient = new AgileClient({
  host: 'https://your-domain.atlassian.net',
  authentication: {
    basic: {
      email: process.env.JIRA_EMAIL,
      apiToken: process.env.JIRA_API_TOKEN,
    },
  },
});

// Agile client provides specialized methods for boards, sprints, and backlogs
const boards = await agileClient.board.getAllBoards();
const sprints = await agileClient.sprint.getAllSprints({ boardId: 1 });
```

### Bundle Size Optimization

Import only what you need to reduce bundle size:

```javascript
// Instead of importing the full client
import { Board } from 'jira.js/agile';
import { Issues } from 'jira.js/version3';

// Use specific modules
const board = new Board({
  host: 'https://your-domain.atlassian.net',
  authentication: { /* ... */ },
});
```

## Common Patterns

### Batch Operations

```javascript
// Create multiple issues in batch
async function createMultipleIssues(issuesData) {
  const promises = issuesData.map(issueData =>
    client.issues.createIssue({
      fields: issueData,
    })
  );

  const results = await Promise.all(promises);
  return results;
}

// Usage
const issues = await createMultipleIssues([
  {
    project: { key: 'PROJ' },
    summary: 'Task 1',
    issuetype: { name: 'Task' },
  },
  {
    project: { key: 'PROJ' },
    summary: 'Task 2',
    issuetype: { name: 'Task' },
  },
]);

console.log('Created issues:', issues.map(i => i.key));
```

### Rate Limiting

```javascript
// Implement rate limiting for bulk operations
async function rateLimitedOperation(items, operation, rateLimit = 10) {
  const results = [];

  for (let i = 0; i < items.length; i += rateLimit) {
    const batch = items.slice(i, i + rateLimit);
    const batchResults = await Promise.all(
      batch.map(item => operation(item))
    );
    results.push(...batchResults);

    // Wait before next batch (optional)
    if (i + rateLimit < items.length) {
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }

  return results;
}
```

### Retry Logic

```javascript
// Implement retry logic for failed requests
async function withRetry(operation, maxRetries = 3, delay = 1000) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      if (attempt === maxRetries) throw error;

      console.log(`Attempt ${attempt} failed, retrying in ${delay}ms...`);
      await new Promise(resolve => setTimeout(resolve, delay));
      delay *= 2; // Exponential backoff
    }
  }
}

// Usage
const issue = await withRetry(() =>
  client.issues.getIssue({ issueIdOrKey: 'PROJ-123' })
);
```

## Complete Application Example

```javascript
import 'dotenv/config';
import { Version3Client } from 'jira.js';

// Initialize client
const client = new Version3Client({
  host: process.env.JIRA_HOST,
  authentication: {
    basic: {
      email: process.env.JIRA_EMAIL,
      apiToken: process.env.JIRA_API_TOKEN,
    },
  },
});

async function main() {
  try {
    // Get current user
    const user = await client.myself.getCurrentUser();
    console.log(`Logged in as: ${user.displayName}`);

    // Create a new issue
    const newIssue = await client.issues.createIssue({
      fields: {
        project: { key: 'PROJ' },
        summary: 'Implement new feature',
        description: {
          type: 'doc',
          version: 1,
          content: [
            {
              type: 'paragraph',
              content: [
                {
                  type: 'text',
                  text: 'This is a new feature request from the API',
                },
              ],
            },
          ],
        },
        issuetype: { name: 'Story' },
        priority: { name: 'High' },
      },
    });

    console.log(`Created issue: ${newIssue.key}`);

    // Add a comment
    await client.issueComments.addComment({
      issueIdOrKey: newIssue.key,
      body: {
        type: 'doc',
        version: 1,
        content: [
          {
            type: 'paragraph',
            content: [
              {
                type: 'text',
                text: 'Starting work on this issue',
              },
            ],
          },
        ],
      },
    });

    console.log('Comment added');

    // Search for issues
    const searchResults = await client.issueSearch.searchForIssuesUsingJql({
      jql: 'project = PROJ AND status = "To Do" ORDER BY created DESC',
      maxResults: 10,
    });

    console.log(`Found ${searchResults.total} issues`);
    searchResults.issues.forEach(issue => {
      console.log(`  ${issue.key}: ${issue.fields.summary}`);
    });

  } catch (error) {
    console.error('Error:', error.message);
    if (error.data) {
      console.error('Error details:', error.data);
    }
  }
}

main();
```

## TypeScript Support

jira.js is written in TypeScript and provides full type definitions:

```typescript
import { Version3Client } from 'jira.js';
import type { Issue, IssueBean, SearchResults } from 'jira.js/out/version3/models';

const client = new Version3Client({
  host: process.env.JIRA_HOST!,
  authentication: {
    basic: {
      email: process.env.JIRA_EMAIL!,
      apiToken: process.env.JIRA_API_TOKEN!,
    },
  },
});

async function getIssues(projectKey: string): Promise<Issue[]> {
  const results: SearchResults = await client.issueSearch.searchForIssuesUsingJql({
    jql: `project = ${projectKey}`,
    maxResults: 100,
  });

  return results.issues || [];
}

async function createIssue(summary: string, projectKey: string): Promise<IssueBean> {
  return await client.issues.createIssue({
    fields: {
      project: { key: projectKey },
      summary,
      issuetype: { name: 'Task' },
    },
  });
}
```

## Common JQL Patterns

```javascript
// Issues assigned to current user
const jql1 = 'assignee = currentUser()';

// Issues created in the last 7 days
const jql2 = 'created >= -7d';

// High priority bugs not resolved
const jql3 = 'type = Bug AND priority = High AND status != Resolved';

// Issues in specific sprint
const jql4 = 'sprint = 123';

// Issues with specific labels
const jql5 = 'labels IN (urgent, critical)';

// Issues due this week
const jql6 = 'duedate >= startOfWeek() AND duedate <= endOfWeek()';

// Combining multiple conditions
const jql7 = `
  project = PROJ AND
  status IN ("To Do", "In Progress") AND
  assignee IN (currentUser(), "john.doe") AND
  created >= -30d
  ORDER BY priority DESC, created ASC
`;
```

## Response Format Examples

### Issue Response

```javascript
{
  "id": "10002",
  "key": "PROJ-123",
  "self": "https://your-domain.atlassian.net/rest/api/3/issue/10002",
  "fields": {
    "summary": "Issue summary",
    "description": { /* ADF format */ },
    "status": {
      "name": "In Progress",
      "statusCategory": { "key": "indeterminate" }
    },
    "priority": { "name": "High" },
    "assignee": {
      "accountId": "5b10a2844c20165700ede21g",
      "displayName": "John Doe",
      "emailAddress": "john@example.com"
    },
    "created": "2025-11-01T10:00:00.000+0000",
    "updated": "2025-11-07T15:30:00.000+0000",
    "labels": ["backend", "api"]
  }
}
```

### Search Results Response

```javascript
{
  "startAt": 0,
  "maxResults": 50,
  "total": 127,
  "issues": [
    { /* issue object */ },
    { /* issue object */ }
  ]
}
```
