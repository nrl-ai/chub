---
name: octokit
description: "Official GitHub SDK for JavaScript providing REST API, GraphQL API, authentication, and App support via Octokit packages."
metadata:
  languages: "javascript"
  versions: "5.0.5"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "github,octokit,rest,graphql,api"
---

# GitHub Octokit.js SDK Coding Guide

## 1. Golden Rule

**Always use the official Octokit packages from GitHub.** The main `octokit` package is recommended for most use cases as it includes REST API, GraphQL API, authentication, App support, and recommended plugins out of the box.

**Never use deprecated or unofficial GitHub API libraries.**

To view available Octokit packages and their details:
```bash
npm view octokit
npm view @octokit/core
npm view @octokit/rest
npm view @octokit/graphql
```

## 2. Installation

```bash
npm install octokit
# Or: yarn add octokit
# Or: pnpm add octokit
```

For specific components, use `@octokit/rest`, `@octokit/graphql`, or `@octokit/core`.

**Environment Variables:**
```bash
# Personal Access Token (classic or fine-grained)
GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx

# For GitHub Apps
GITHUB_APP_ID=123456
GITHUB_PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----\n..."
GITHUB_INSTALLATION_ID=789012

# For OAuth Apps
GITHUB_CLIENT_ID=Iv1.xxxxxxxxxxxx
GITHUB_CLIENT_SECRET=xxxxxxxxxxxxxxxxxxxx

# GitHub Enterprise Server (optional)
GITHUB_API_URL=https://github.mycompany.com/api/v3
```

## 3. Initialization

### Basic Authentication with Personal Access Token
```javascript
import { Octokit } from "octokit";

// Using environment variable
const octokit = new Octokit({
  auth: process.env.GITHUB_TOKEN
});

// Or explicit token
const octokit = new Octokit({
  auth: "ghp_xxxxxxxxxxxxxxxxxxxx"
});
```

### Unauthenticated Requests
```javascript
import { Octokit } from "octokit";

// Lower rate limits (60 requests/hour)
const octokit = new Octokit();
```

### GitHub App Authentication
```javascript
import { Octokit } from "octokit";
import { createAppAuth } from "@octokit/auth-app";

const octokit = new Octokit({
  authStrategy: createAppAuth,
  auth: {
    appId: process.env.GITHUB_APP_ID,
    privateKey: process.env.GITHUB_PRIVATE_KEY,
    installationId: process.env.GITHUB_INSTALLATION_ID
  }
});
```

### OAuth App Authentication
```javascript
import { Octokit } from "octokit";
import { createOAuthAppAuth } from "@octokit/auth-oauth-app";

const octokit = new Octokit({
  authStrategy: createOAuthAppAuth,
  auth: {
    clientId: process.env.GITHUB_CLIENT_ID,
    clientSecret: process.env.GITHUB_CLIENT_SECRET
  }
});
```

### GitHub Actions Authentication
```javascript
import { Octokit } from "octokit";

// In GitHub Actions, use the built-in token
const octokit = new Octokit({
  auth: process.env.GITHUB_TOKEN // Available in all workflows
});
```

### GitHub Enterprise Server
```javascript
import { Octokit } from "octokit";

const octokit = new Octokit({
  auth: process.env.GITHUB_TOKEN,
  baseUrl: "https://github.mycompany.com/api/v3"
});
```

## 4. Core API Surfaces

### Repositories

**Minimal Example - Get Repository:**
```javascript
const { data: repo } = await octokit.rest.repos.get({
  owner: "octokit",
  repo: "rest.js"
});
```

**Advanced Example - Create Repository:**
```javascript
const { data: newRepo } = await octokit.rest.repos.createForAuthenticatedUser({
  name: "my-new-repo",
  description: "Created via Octokit",
  private: false,
  auto_init: true,
  gitignore_template: "Node",
  license_template: "mit",
  homepage: "https://example.com",
  has_issues: true,
  has_projects: true,
  has_wiki: true
});
```

**List User Repositories:**

To view available methods and parameters:
```bash
npm view @octokit/plugin-rest-endpoint-methods
```

```javascript
const { data: repos } = await octokit.rest.repos.listForAuthenticatedUser({
  sort: "updated",
  direction: "desc",
  per_page: 100
});
```

**Update Repository:**
```javascript
const { data: updated } = await octokit.rest.repos.update({
  owner: "username",
  repo: "repo-name",
  description: "New description",
  homepage: "https://newsite.com",
  has_issues: false
});
```

### Issues

**Minimal Example - List Issues:**

For CLI alternative: `gh issue list --repo owner/repo`

```javascript
const { data: issues } = await octokit.rest.issues.listForRepo({
  owner: "facebook",
  repo: "react"
});
```

**Advanced Example - Create Issue with Labels:**
```javascript
const { data: issue } = await octokit.rest.issues.create({
  owner: "owner",
  repo: "repo",
  title: "Bug: Application crashes on startup",
  body: `## Description
Detailed description of the issue...

## Steps to Reproduce
1. Step 1
2. Step 2

## Expected Behavior
What should happen

## Actual Behavior
What actually happens`,
  labels: ["bug", "high-priority"],
  assignees: ["username1", "username2"],
  milestone: 1
});
```

**Update Issue:**
```javascript
const { data: updated } = await octokit.rest.issues.update({
  owner: "owner",
  repo: "repo",
  issue_number: 123,
  state: "closed",
  state_reason: "completed",
  labels: ["resolved"]
});
```

**Add Comment:**
```javascript
const { data: comment } = await octokit.rest.issues.createComment({
  owner: "owner",
  repo: "repo",
  issue_number: 123,
  body: "Thanks for reporting! This has been fixed in v2.0.0"
});
```

### Pull Requests

**Minimal Example - List Pull Requests:**

For CLI alternative: `gh pr list --repo owner/repo --state open`

```javascript
const { data: pulls } = await octokit.rest.pulls.list({
  owner: "microsoft",
  repo: "vscode",
  state: "open"
});
```

**Advanced Example - Create Pull Request:**
```javascript
const { data: pr } = await octokit.rest.pulls.create({
  owner: "owner",
  repo: "repo",
  title: "Add new feature",
  head: "feature-branch",
  base: "main",
  body: "Detailed description of changes in this pull request.",
  maintainer_can_modify: true,
  draft: false
});
```

**Get Pull Request Files:**
```javascript
const { data: files } = await octokit.rest.pulls.listFiles({
  owner: "owner",
  repo: "repo",
  pull_number: 123
});
```

**Merge Pull Request:**
```javascript
const { data: merge } = await octokit.rest.pulls.merge({
  owner: "owner",
  repo: "repo",
  pull_number: 123,
  commit_title: "Merge PR #123: Add new feature",
  commit_message: "Additional details about the merge",
  merge_method: "squash" // or "merge" or "rebase"
});
```

**Request Reviewers:**
```javascript
await octokit.rest.pulls.requestReviewers({
  owner: "owner",
  repo: "repo",
  pull_number: 123,
  reviewers: ["reviewer1", "reviewer2"],
  team_reviewers: ["team-slug"]
});
```

### Commits

**Minimal Example - Get Commit:**
```javascript
const { data: commit } = await octokit.rest.repos.getCommit({
  owner: "owner",
  repo: "repo",
  ref: "abc123"
});
```

**Advanced Example - List Commits with Filtering:**
```javascript
const { data: commits } = await octokit.rest.repos.listCommits({
  owner: "owner",
  repo: "repo",
  sha: "main",
  path: "src/index.js",
  author: "username",
  since: "2025-01-01T00:00:00Z",
  until: "2025-12-31T23:59:59Z",
  per_page: 100
});
```

**Compare Commits:**
```javascript
const { data: comparison } = await octokit.rest.repos.compareCommits({
  owner: "owner",
  repo: "repo",
  base: "main",
  head: "feature-branch"
});
```

### Branches

**List Branches:**

For CLI alternative: `gh api repos/owner/repo/branches` or `git branch -r`

```javascript
const { data: branches } = await octokit.rest.repos.listBranches({
  owner: "owner",
  repo: "repo"
});
```

**Get Branch:**
```javascript
const { data: branch } = await octokit.rest.repos.getBranch({
  owner: "owner",
  repo: "repo",
  branch: "main"
});
```

**Create Branch (via Git References):**
```javascript
// First, get the SHA of the source branch
const { data: refData } = await octokit.rest.git.getRef({
  owner: "owner",
  repo: "repo",
  ref: "heads/main"
});

// Create new branch from that SHA
await octokit.rest.git.createRef({
  owner: "owner",
  repo: "repo",
  ref: "refs/heads/new-feature",
  sha: refData.object.sha
});
```

### Files and Content

**Minimal Example - Get File Content:**
```javascript
const { data: file } = await octokit.rest.repos.getContent({
  owner: "owner",
  repo: "repo",
  path: "README.md"
});

// Decode base64 content
const content = Buffer.from(file.content, "base64").toString("utf8");
console.log(content);
```

**Advanced Example - Create or Update File:**
```javascript
// Get current file to retrieve SHA (required for updates)
let sha;
try {
  const { data: existing } = await octokit.rest.repos.getContent({
    owner: "owner",
    repo: "repo",
    path: "config.json"
  });
  sha = existing.sha;
} catch (error) {
  // File doesn't exist, will create new
}

const content = JSON.stringify({ version: "2.0.0" }, null, 2);
const { data: result } = await octokit.rest.repos.createOrUpdateFileContents({
  owner: "owner",
  repo: "repo",
  path: "config.json",
  message: "Update config version to 2.0.0",
  content: Buffer.from(content).toString("base64"),
  sha: sha, // Required for updates, omit for new files
  branch: "main",
  committer: {
    name: "Bot Name",
    email: "bot@example.com"
  },
  author: {
    name: "Author Name",
    email: "author@example.com"
  }
});

console.log(`File updated: ${result.content.html_url}`);
```

**Delete File:**
```javascript
// Get file SHA first
const { data: file } = await octokit.rest.repos.getContent({
  owner: "owner",
  repo: "repo",
  path: "file-to-delete.txt"
});

await octokit.rest.repos.deleteFile({
  owner: "owner",
  repo: "repo",
  path: "file-to-delete.txt",
  message: "Remove obsolete file",
  sha: file.sha,
  branch: "main"
});
```

### Releases

**Minimal Example - List Releases:**

For CLI alternative: `gh release list --repo owner/repo`

```javascript
const { data: releases } = await octokit.rest.repos.listReleases({
  owner: "owner",
  repo: "repo"
});
```

**Advanced Example - Create Release:**
```javascript
const { data: release } = await octokit.rest.repos.createRelease({
  owner: "owner",
  repo: "repo",
  tag_name: "v2.0.0",
  name: "Version 2.0.0",
  body: "Release notes and changelog content here.",
  draft: false,
  prerelease: false,
  generate_release_notes: false,
  target_commitish: "main"
});

```

**Get Latest Release:**

For CLI alternative: `gh release view --repo owner/repo`

```javascript
const { data: latest } = await octokit.rest.repos.getLatestRelease({
  owner: "owner",
  repo: "repo"
});
```

### Gists

**Minimal Example - Create Gist:**
```javascript
const { data: gist } = await octokit.rest.gists.create({
  files: {
    "hello.js": {
      content: "console.log('Hello World');"
    }
  },
  description: "Hello World example",
  public: true
});

```

**Advanced Example - Multi-file Gist:**
```javascript
const { data: gist } = await octokit.rest.gists.create({
  files: {
    "package.json": {
      content: JSON.stringify({
        name: "example",
        version: "1.0.0"
      }, null, 2)
    },
    "index.js": {
      content: "const express = require('express');\n// App code here"
    },
    "README.md": {
      content: "# Example Project\n\nDescription here"
    }
  },
  description: "Full project example",
  public: false
});
```

### Search

**Search Repositories:**
```javascript
const { data: results } = await octokit.rest.search.repos({
  q: "language:javascript stars:>1000 created:>2024-01-01",
  sort: "stars",
  order: "desc",
  per_page: 30
});
```

**Search Issues and Pull Requests:**
```javascript
const { data: results } = await octokit.rest.search.issuesAndPullRequests({
  q: "type:pr repo:facebook/react is:open label:bug",
  sort: "created",
  order: "desc"
});
```

**Search Code:**
```javascript
const { data: results } = await octokit.rest.search.code({
  q: "import Octokit from octokit language:javascript",
  per_page: 50
});
```

**Search Users:**
```javascript
const { data: results } = await octokit.rest.search.users({
  q: "followers:>1000 location:London",
  per_page: 20
});
```

### Users and Organizations

**Get Authenticated User:**

For CLI alternative: `gh api user`

```javascript
const { data: user } = await octokit.rest.users.getAuthenticated();
```

**Get User by Username:**

For CLI alternative: `gh api users/username`

```javascript
const { data: user } = await octokit.rest.users.getByUsername({
  username: "torvalds"
});
```

**List Organization Repositories:**

For CLI alternative: `gh repo list org-name`

```javascript
const { data: repos } = await octokit.rest.repos.listForOrg({
  org: "github",
  type: "public",
  sort: "updated",
  per_page: 100
});
```

**List Organization Members:**

For CLI alternative: `gh api orgs/org-name/members`

```javascript
const { data: members } = await octokit.rest.orgs.listMembers({
  org: "github",
  per_page: 100
});
```

### Webhooks

**List Repository Webhooks:**

For CLI alternative: `gh api repos/owner/repo/hooks`

```javascript
const { data: hooks } = await octokit.rest.repos.listWebhooks({
  owner: "owner",
  repo: "repo"
});
```

**Create Webhook:**
```javascript
const { data: hook } = await octokit.rest.repos.createWebhook({
  owner: "owner",
  repo: "repo",
  name: "web",
  active: true,
  events: ["push", "pull_request", "issues"],
  config: {
    url: "https://example.com/webhook",
    content_type: "json",
    secret: process.env.WEBHOOK_SECRET,
    insecure_ssl: "0"
  }
});
```

### GraphQL API

**Minimal Example - Simple Query:**
```javascript
const { repository } = await octokit.graphql(`
  query {
    repository(owner: "octokit", name: "graphql.js") {
      name
      description
      stargazerCount
    }
  }
`);
```

**Advanced Example - Query with Variables:**
```javascript
const query = `
  query($owner: String!, $repo: String!, $issueCount: Int!) {
    repository(owner: $owner, name: $repo) {
      name
      issues(last: $issueCount, states: OPEN) {
        edges {
          node {
            number
            title
            author {
              login
            }
            createdAt
            labels(first: 5) {
              nodes {
                name
              }
            }
          }
        }
      }
    }
  }
`;

const { repository } = await octokit.graphql(query, {
  owner: "facebook",
  repo: "react",
  issueCount: 10
});
```

**GraphQL Mutation Example:**
```javascript
const mutation = `
  mutation($repositoryId: ID!, $issueTitle: String!, $issueBody: String!) {
    createIssue(input: {
      repositoryId: $repositoryId,
      title: $issueTitle,
      body: $issueBody
    }) {
      issue {
        number
        url
      }
    }
  }
`;

// First get repository ID
const { repository } = await octokit.graphql(`
  query($owner: String!, $name: String!) {
    repository(owner: $owner, name: $name) {
      id
    }
  }
`, {
  owner: "owner",
  name: "repo"
});

// Create issue
const result = await octokit.graphql(mutation, {
  repositoryId: repository.id,
  issueTitle: "New issue via GraphQL",
  issueBody: "Issue body content"
});
```

## 5. Advanced Features

### Pagination

**Automatic Pagination - Get All Results:**
```javascript
// Get all issues (auto-handles pagination)
const allIssues = await octokit.paginate(
  octokit.rest.issues.listForRepo,
  {
    owner: "facebook",
    repo: "react",
    state: "all",
    per_page: 100
  }
);
```

**Iterator-based Pagination:**
```javascript
// Process results as they come
for await (const response of octokit.paginate.iterator(
  octokit.rest.repos.listForOrg,
  {
    org: "github",
    per_page: 100
  }
)) {
  // response.data contains up to 100 items
  // Process each batch here
}
```

**Custom Page Limit:**
```javascript
// Get only first 500 items across pages
const limitedResults = await octokit.paginate(
  octokit.rest.issues.listForRepo,
  {
    owner: "owner",
    repo: "repo",
    per_page: 100
  },
  (response, done) => {
    if (response.data.length >= 500) {
      done();
    }
    return response.data;
  }
);
```

### Error Handling

**Comprehensive Error Handling:**
```javascript
import { RequestError } from "@octokit/request-error";

try {
  const { data } = await octokit.rest.repos.get({
    owner: "owner",
    repo: "nonexistent"
  });
} catch (error) {
  if (error instanceof RequestError) {
    console.error(`Error ${error.status}: ${error.message}`);

    // Check specific error codes
    if (error.status === 404) {
      console.error("Repository not found");
    } else if (error.status === 403) {
      if (error.response.headers["x-ratelimit-remaining"] === "0") {
        console.error("Rate limit exceeded");
        const resetTime = new Date(
          error.response.headers["x-ratelimit-reset"] * 1000
        );
        console.error(`Rate limit resets at ${resetTime}`);
      } else {
        console.error("Forbidden - check permissions");
      }
    } else if (error.status === 401) {
      console.error("Unauthorized - check authentication token");
    } else if (error.status >= 500) {
      console.error("GitHub server error - retry later");
    }

    // Log additional error details
    console.error("Request ID:", error.response.headers["x-github-request-id"]);
  } else {
    console.error("Unexpected error:", error);
  }
}
```

### Rate Limiting with Throttle Plugin

**Setup:**
```bash
npm install @octokit/plugin-throttling
```

**Implementation:**
```javascript
import { Octokit } from "@octokit/core";
import { throttling } from "@octokit/plugin-throttling";

const MyOctokit = Octokit.plugin(throttling);

const octokit = new MyOctokit({
  auth: process.env.GITHUB_TOKEN,
  throttle: {
    onRateLimit: (retryAfter, options, octokit, retryCount) => {
      octokit.log.warn(
        `Request quota exhausted for request ${options.method} ${options.url}`
      );

      // Retry first 3 times
      if (retryCount < 3) {
        octokit.log.info(`Retrying after ${retryAfter} seconds!`);
        return true;
      }
    },
    onSecondaryRateLimit: (retryAfter, options, octokit, retryCount) => {
      octokit.log.warn(
        `SecondaryRateLimit detected for request ${options.method} ${options.url}`
      );

      // Always retry on secondary rate limit
      if (retryCount < 5) {
        octokit.log.info(`Retrying after ${retryAfter} seconds!`);
        return true;
      }
    }
  }
});
```

### Retry Plugin

**Setup:**
```bash
npm install @octokit/plugin-retry
```

**Implementation:**
```javascript
import { Octokit } from "@octokit/core";
import { retry } from "@octokit/plugin-retry";

const MyOctokit = Octokit.plugin(retry);

const octokit = new MyOctokit({
  auth: process.env.GITHUB_TOKEN
});

// Automatic retries on 500 errors (up to 3 times)
const { data } = await octokit.rest.repos.get({
  owner: "owner",
  repo: "repo"
});

// Manual retry configuration
const { data: issues } = await octokit.rest.issues.listForRepo({
  owner: "owner",
  repo: "repo",
  request: {
    retries: 5,
    retryAfter: 3 // seconds
  }
});
```

### Custom Request Options

**Timeouts:**
```javascript
const { data } = await octokit.rest.repos.get({
  owner: "owner",
  repo: "repo",
  request: {
    timeout: 10000 // 10 seconds
  }
});
```

**Custom Headers:**
```javascript
const { data } = await octokit.rest.repos.get({
  owner: "owner",
  repo: "repo",
  headers: {
    "X-GitHub-Api-Version": "2022-11-28"
  }
});
```

**Signal for Abort:**
```javascript
const controller = new AbortController();

setTimeout(() => controller.abort(), 5000); // Abort after 5s

try {
  const { data } = await octokit.rest.repos.get({
    owner: "owner",
    repo: "repo",
    request: {
      signal: controller.signal
    }
  });
} catch (error) {
  if (error.name === "AbortError") {
    console.log("Request was aborted");
  }
}
```

## 6. TypeScript Usage

### Basic TypeScript Setup

**tsconfig.json Configuration:**
```json
{
  "compilerOptions": {
    "moduleResolution": "node16",
    "module": "node16",
    "target": "ES2022",
    "lib": ["ES2022"]
  }
}
```

### Type-Safe API Calls

**Import Types:**
```typescript
import { Octokit } from "octokit";
import type { RestEndpointMethodTypes } from "@octokit/plugin-rest-endpoint-methods";

// Type for a specific endpoint
type GetRepoResponse = RestEndpointMethodTypes["repos"]["get"]["response"];
type GetRepoParams = RestEndpointMethodTypes["repos"]["get"]["parameters"];

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });

const params: GetRepoParams = {
  owner: "octokit",
  repo: "rest.js"
};

const response: GetRepoResponse = await octokit.rest.repos.get(params);
const repo = response.data;
```

**Generic Response Types:**
```typescript
import type { Endpoints } from "@octokit/types";

type IssuesListResponse = Endpoints["GET /repos/{owner}/{repo}/issues"]["response"];
type Issue = Endpoints["GET /repos/{owner}/{repo}/issues"]["response"]["data"][number];

const issues: IssuesListResponse = await octokit.rest.issues.listForRepo({
  owner: "facebook",
  repo: "react"
});

const firstIssue: Issue = issues.data[0];
```

**Custom Type-Safe Wrapper:**
```typescript
interface GitHubService {
  getRepository(owner: string, repo: string): Promise<Repository>;
  createIssue(params: CreateIssueParams): Promise<Issue>;
}

interface Repository {
  name: string;
  description: string;
  stars: number;
  url: string;
}

interface CreateIssueParams {
  owner: string;
  repo: string;
  title: string;
  body: string;
  labels?: string[];
}

interface Issue {
  number: number;
  title: string;
  url: string;
}

class GitHubClient implements GitHubService {
  constructor(private octokit: Octokit) {}

  async getRepository(owner: string, repo: string): Promise<Repository> {
    const { data } = await this.octokit.rest.repos.get({ owner, repo });

    return {
      name: data.name,
      description: data.description || "",
      stars: data.stargazers_count,
      url: data.html_url
    };
  }

  async createIssue(params: CreateIssueParams): Promise<Issue> {
    const { data } = await this.octokit.rest.issues.create({
      owner: params.owner,
      repo: params.repo,
      title: params.title,
      body: params.body,
      labels: params.labels
    });

    return {
      number: data.number,
      title: data.title,
      url: data.html_url
    };
  }
}

// Usage
const client = new GitHubClient(octokit);
const repo = await client.getRepository("facebook", "react");
```

### GraphQL TypeScript

```typescript
import { Octokit } from "octokit";

interface RepositoryQuery {
  repository: {
    name: string;
    stargazerCount: number;
    issues: {
      totalCount: number;
      nodes: Array<{
        number: number;
        title: string;
        author: {
          login: string;
        } | null;
      }>;
    };
  };
}

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });

const result: RepositoryQuery = await octokit.graphql(`
  query($owner: String!, $repo: String!) {
    repository(owner: $owner, name: $repo) {
      name
      stargazerCount
      issues(last: 5, states: OPEN) {
        totalCount
        nodes {
          number
          title
          author {
            login
          }
        }
      }
    }
  }
`, {
  owner: "facebook",
  repo: "react"
});
```

## 7. Best Practices

### Authentication and Security

**Never Hardcode Tokens:**
```javascript
// BAD - Never do this
const octokit = new Octokit({ auth: "ghp_actualtoken123" });

// GOOD - Use environment variables
const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });
```

**Use Fine-Grained Tokens:**
Fine-grained personal access tokens provide more granular permissions and are repository-scoped. Use them instead of classic tokens when possible.

**Rotate Tokens Regularly:**
Implement a token rotation strategy, especially for long-running applications.

**Use GitHub Apps for Production:**
GitHub Apps provide better security, higher rate limits, and better audit trails than personal access tokens.

### Rate Limiting Strategy

**Check Rate Limit Status:**
```javascript
const { data: rateLimit } = await octokit.rest.rateLimit.get();

console.log(`Remaining: ${rateLimit.rate.remaining}/${rateLimit.rate.limit}`);
console.log(`Resets at: ${new Date(rateLimit.rate.reset * 1000)}`);

// Check specific resource limits
console.log(`Search limit: ${rateLimit.resources.search.remaining}`);
console.log(`GraphQL limit: ${rateLimit.resources.graphql.remaining}`);
```

**Implement Backoff Strategy:**
```javascript
async function makeRequestWithBackoff(fn, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      if (error.status === 403 && error.response.headers["x-ratelimit-remaining"] === "0") {
        const resetTime = parseInt(error.response.headers["x-ratelimit-reset"]) * 1000;
        const waitTime = resetTime - Date.now();

        if (i < maxRetries - 1) {
          console.log(`Rate limited. Waiting ${waitTime}ms...`);
          await new Promise(resolve => setTimeout(resolve, waitTime));
          continue;
        }
      }
      throw error;
    }
  }
}
```

**Use Conditional Requests:**
```javascript
// First request
const { data, headers } = await octokit.rest.repos.get({
  owner: "owner",
  repo: "repo"
});

const etag = headers.etag;

// Later request - only downloads if changed
const { status, data: newData } = await octokit.rest.repos.get({
  owner: "owner",
  repo: "repo",
  headers: {
    "If-None-Match": etag
  }
});

if (status === 304) {
  console.log("No changes - use cached data");
} else {
  console.log("Data changed - use new data");
}
```

### Error Recovery

**Implement Retry Logic:**
```javascript
async function retryRequest(fn, maxRetries = 3, delay = 1000) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      const isLastRetry = i === maxRetries - 1;
      const shouldRetry = error.status >= 500 || error.status === 429;

      if (!shouldRetry || isLastRetry) {
        throw error;
      }

      const backoffDelay = delay * Math.pow(2, i);
      console.log(`Retry ${i + 1}/${maxRetries} after ${backoffDelay}ms`);
      await new Promise(resolve => setTimeout(resolve, backoffDelay));
    }
  }
}

// Usage
const data = await retryRequest(
  () => octokit.rest.repos.get({ owner: "owner", repo: "repo" })
);
```

### Performance Optimization

**Use GraphQL for Complex Queries:**
```javascript
// BAD - Multiple REST requests
const { data: repo } = await octokit.rest.repos.get({ owner, repo });
const { data: issues } = await octokit.rest.issues.listForRepo({ owner, repo });
const { data: pulls } = await octokit.rest.pulls.list({ owner, repo });

// GOOD - Single GraphQL request
const result = await octokit.graphql(`
  query($owner: String!, $repo: String!) {
    repository(owner: $owner, name: $repo) {
      name
      description
      issues(last: 10, states: OPEN) {
        totalCount
        nodes { number title }
      }
      pullRequests(last: 10, states: OPEN) {
        totalCount
        nodes { number title }
      }
    }
  }
`, { owner, repo });
```

**Batch Operations:**
```javascript
// Process items in batches to avoid overwhelming the API
async function processBatch(items, batchSize, handler) {
  for (let i = 0; i < items.length; i += batchSize) {
    const batch = items.slice(i, i + batchSize);
    await Promise.all(batch.map(handler));

    // Brief pause between batches
    if (i + batchSize < items.length) {
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }
}

// Usage
await processBatch(
  repositories,
  10,
  async (repo) => {
    const { data } = await octokit.rest.repos.get({
      owner: repo.owner,
      repo: repo.name
    });
    // Process repo data
  }
);
```

### Data Validation

**Validate Input:**
```javascript
function validateRepoParams(owner, repo) {
  if (!owner || typeof owner !== "string" || owner.trim() === "") {
    throw new Error("Invalid owner parameter");
  }
  if (!repo || typeof repo !== "string" || repo.trim() === "") {
    throw new Error("Invalid repo parameter");
  }
  if (!/^[a-zA-Z0-9-_.]+$/.test(owner)) {
    throw new Error("Owner contains invalid characters");
  }
  if (!/^[a-zA-Z0-9-_.]+$/.test(repo)) {
    throw new Error("Repo contains invalid characters");
  }
}

// Usage
validateRepoParams(userInput.owner, userInput.repo);
const { data } = await octokit.rest.repos.get({
  owner: userInput.owner,
  repo: userInput.repo
});
```

**Sanitize Output:**
```javascript
function sanitizeIssue(issue) {
  return {
    number: issue.number,
    title: issue.title.trim(),
    body: issue.body?.trim() || "",
    state: issue.state,
    createdAt: new Date(issue.created_at),
    author: issue.user?.login || "unknown"
  };
}
```

## 8. Production Checklist

### Version Pinning
```json
{
  "dependencies": {
    "octokit": "3.1.2"
  }
}
```

Pin exact versions (no `^` or `~`) to prevent unexpected breaking changes.

### Robust Error Handling
```javascript
// Production-ready request wrapper
async function safeRequest(requestFn) {
  try {
    const { data } = await requestFn();
    return { success: true, data, error: null };
  } catch (error) {
    console.error("GitHub API Error:", {
      status: error.status,
      message: error.message,
      requestId: error.response?.headers?.["x-github-request-id"],
      timestamp: new Date().toISOString()
    });

    return {
      success: false,
      data: null,
      error: {
        code: error.status,
        message: error.message,
        retryable: error.status >= 500 || error.status === 429
      }
    };
  }
}

// Usage
const result = await safeRequest(
  () => octokit.rest.repos.get({ owner: "owner", repo: "repo" })
);

if (result.success) {
  console.log(result.data);
} else {
  console.error("Request failed:", result.error);
  if (result.error.retryable) {
    // Implement retry logic
  }
}
```

### Environment Configuration
```javascript
// config.js
export const config = {
  github: {
    token: process.env.GITHUB_TOKEN,
    baseUrl: process.env.GITHUB_API_URL || "https://api.github.com",
    timeout: parseInt(process.env.GITHUB_TIMEOUT || "30000"),
    userAgent: `${process.env.APP_NAME}/${process.env.APP_VERSION}`
  }
};

// Validate configuration on startup
function validateConfig() {
  if (!config.github.token) {
    throw new Error("GITHUB_TOKEN environment variable is required");
  }
  if (config.github.timeout < 1000 || config.github.timeout > 60000) {
    throw new Error("GITHUB_TIMEOUT must be between 1000 and 60000");
  }
}

validateConfig();

export const octokit = new Octokit({
  auth: config.github.token,
  baseUrl: config.github.baseUrl,
  userAgent: config.github.userAgent,
  request: {
    timeout: config.github.timeout
  }
});
```

### Logging and Monitoring
```javascript
import { Octokit } from "@octokit/core";

class LoggingOctokit extends Octokit {
  constructor(options) {
    super(options);

    this.hook.before("request", async (options) => {
      console.log(`[GitHub API] ${options.method} ${options.url}`, {
        timestamp: new Date().toISOString()
      });
    });

    this.hook.after("request", async (response, options) => {
      console.log(`[GitHub API] ${options.method} ${options.url} - ${response.status}`, {
        rateLimit: response.headers["x-ratelimit-remaining"],
        timestamp: new Date().toISOString()
      });
    });

    this.hook.error("request", async (error, options) => {
      console.error(`[GitHub API] ${options.method} ${options.url} - ERROR`, {
        status: error.status,
        message: error.message,
        requestId: error.response?.headers?.["x-github-request-id"],
        timestamp: new Date().toISOString()
      });
      throw error;
    });
  }
}

const octokit = new LoggingOctokit({ auth: process.env.GITHUB_TOKEN });
```

### Validate Structured Output
```javascript
function validateRepository(data) {
  const required = ["id", "name", "full_name", "owner", "html_url"];
  for (const field of required) {
    if (!(field in data)) {
      throw new Error(`Missing required field: ${field}`);
    }
  }
  return data;
}

const { data } = await octokit.rest.repos.get({ owner, repo });
const validatedRepo = validateRepository(data);
```

### Avoid Preview/Unstable APIs
```javascript
// BAD - Using preview API
const { data } = await octokit.rest.repos.get({
  owner: "owner",
  repo: "repo",
  mediaType: {
    previews: ["mercy"] // Preview API
  }
});

// GOOD - Use stable APIs only
const { data } = await octokit.rest.repos.get({
  owner: "owner",
  repo: "repo"
});
```

### Health Checks
```javascript
async function healthCheck() {
  try {
    const { data: rateLimit } = await octokit.rest.rateLimit.get();
    const remaining = rateLimit.rate.remaining;

    return {
      healthy: remaining > 100,
      rateLimit: {
        remaining,
        limit: rateLimit.rate.limit,
        reset: new Date(rateLimit.rate.reset * 1000)
      }
    };
  } catch (error) {
    return {
      healthy: false,
      error: error.message
    };
  }
}

// Run periodically
setInterval(async () => {
  const health = await healthCheck();
  if (!health.healthy) {
    console.warn("GitHub API health check failed", health);
  }
}, 60000); // Every minute
```

### Testing Strategy
```javascript
// Mock Octokit for tests
import { jest } from "@jest/globals";

const mockOctokit = {
  rest: {
    repos: {
      get: jest.fn().mockResolvedValue({
        data: {
          id: 1,
          name: "test-repo",
          full_name: "owner/test-repo",
          owner: { login: "owner" },
          html_url: "https://github.com/owner/test-repo"
        }
      })
    }
  }
};

// Test
test("getRepository returns formatted data", async () => {
  const client = new GitHubClient(mockOctokit);
  const repo = await client.getRepository("owner", "test-repo");

  expect(repo.name).toBe("test-repo");
  expect(mockOctokit.rest.repos.get).toHaveBeenCalledWith({
    owner: "owner",
    repo: "test-repo"
  });
});
```

### Graceful Degradation
```javascript
async function getRepoWithFallback(owner, repo) {
  try {
    const { data } = await octokit.rest.repos.get({ owner, repo });
    return data;
  } catch (error) {
    if (error.status === 404) {
      console.warn(`Repository ${owner}/${repo} not found`);
      return null;
    }
    if (error.status === 403) {
      console.warn("Rate limited - using cached data");
      return getCachedRepo(owner, repo);
    }
    throw error; // Re-throw unexpected errors
  }
}
```
