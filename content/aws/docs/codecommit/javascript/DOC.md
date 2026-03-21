---
name: codecommit
description: "AWS SDK for JavaScript v3 client for Amazon CodeCommit repositories, branches, commits, pull requests, and approval workflows"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,codecommit,git,repositories,pull-requests,javascript"
---

# `@aws-sdk/client-codecommit`

Use this package for Amazon CodeCommit API operations from JavaScript or TypeScript. It is the AWS SDK v3 client for repository management, branches, commits, file reads and writes, pull requests, comments, triggers, and approval rules.

## Golden Rules

- Install `@aws-sdk/client-codecommit`, not the legacy `aws-sdk` v2 package.
- This doc covers package version `3.1006.0`.
- Prefer `CodeCommitClient` plus individual commands over the aggregated `CodeCommit` client.
- Use this client for CodeCommit API actions, not for `git clone`, `git fetch`, or `git push`.
- Import only from the package root. Do not deep-import from `dist-*` paths.

## Install

```bash
npm install @aws-sdk/client-codecommit
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## What This Client Is For

Use `@aws-sdk/client-codecommit` when you need to automate CodeCommit over the AWS API, including:

- creating or deleting repositories
- reading repository metadata and clone URLs
- listing branches or repositories
- creating commits and updating files
- reading files, blobs, folders, and differences
- creating or listing pull requests
- managing comments, approval rules, and repository triggers

For normal Git transport, use the repository's HTTPS or SSH clone URL with Git tooling. The SDK client signs AWS API requests; it is not a drop-in replacement for Git remote operations.

## Create a Client

```javascript
import { CodeCommitClient } from "@aws-sdk/client-codecommit";

const codecommit = new CodeCommitClient({
  region: "us-east-1",
});
```

## Credentials and Region

- In Node.js, the standard AWS SDK default credential chain applies.
- Set `region` in the client constructor, via `AWS_REGION`, or through shared AWS config.
- API calls use IAM credentials and SigV4 signing.
- Git-over-HTTPS or SSH access uses CodeCommit clone URLs and Git credentials or SSH keys, not this SDK client.
- Before building greenfield workflows on CodeCommit, verify current service and regional availability for your account.

## Common Operations

### Create a repository and read clone URLs

`CreateRepository` and `GetRepository` return `repositoryMetadata`, including the HTTPS and SSH clone URLs you use with Git.

```javascript
import {
  CodeCommitClient,
  CreateRepositoryCommand,
  GetRepositoryCommand,
} from "@aws-sdk/client-codecommit";

const codecommit = new CodeCommitClient({ region: "us-east-1" });

await codecommit.send(
  new CreateRepositoryCommand({
    repositoryName: "demo-repo",
    repositoryDescription: "Repository created from the AWS SDK for JavaScript v3",
  }),
);

const { repositoryMetadata } = await codecommit.send(
  new GetRepositoryCommand({
    repositoryName: "demo-repo",
  }),
);

console.log(repositoryMetadata?.cloneUrlHttp);
console.log(repositoryMetadata?.cloneUrlSsh);
```

### List repositories with `nextToken`

`ListRepositories` is paginated.

```javascript
import {
  CodeCommitClient,
  ListRepositoriesCommand,
} from "@aws-sdk/client-codecommit";

const codecommit = new CodeCommitClient({ region: "us-east-1" });

let nextToken;

do {
  const page = await codecommit.send(
    new ListRepositoriesCommand({
      nextToken,
      sortBy: "repositoryName",
      order: "ascending",
    }),
  );

  for (const repository of page.repositories ?? []) {
    console.log(repository.repositoryName, repository.repositoryId);
  }

  nextToken = page.nextToken;
} while (nextToken);
```

### Add or update one file on a branch

Use `PutFile` for a single-file change. For multi-file atomic changes, use `CreateCommit` instead.

```javascript
import {
  CodeCommitClient,
  PutFileCommand,
} from "@aws-sdk/client-codecommit";

const codecommit = new CodeCommitClient({ region: "us-east-1" });

const encoder = new TextEncoder();

await codecommit.send(
  new PutFileCommand({
    repositoryName: "demo-repo",
    branchName: "main",
    filePath: "/README.md",
    fileContent: encoder.encode("# Demo Repo\n\nCreated from the AWS SDK v3.\n"),
    commitMessage: "Add README",
    name: "Automation Bot",
    email: "bot@example.com",
    parentCommitId: "4c925148EXAMPLE",
  }),
);
```

Notes:

- `parentCommitId` helps prevent writing on top of a branch tip that has moved.
- For the first commit in an empty repository, the parent commit is not required.
- `filePath` is the full path in the repository.

### Read a file and decode its bytes

AWS CLI examples show `GetFile` content as base64 because the CLI serializes blob output. In the JavaScript SDK, `fileContent` is binary data, so decode it with `TextDecoder` when you expect text.

```javascript
import {
  CodeCommitClient,
  GetFileCommand,
} from "@aws-sdk/client-codecommit";

const codecommit = new CodeCommitClient({ region: "us-east-1" });

const response = await codecommit.send(
  new GetFileCommand({
    repositoryName: "demo-repo",
    commitSpecifier: "main",
    filePath: "README.md",
  }),
);

const text = new TextDecoder().decode(response.fileContent);

console.log(response.commitId);
console.log(text);
```

### Create and list pull requests

```javascript
import {
  CodeCommitClient,
  CreatePullRequestCommand,
  ListPullRequestsCommand,
} from "@aws-sdk/client-codecommit";

const codecommit = new CodeCommitClient({ region: "us-east-1" });

const { pullRequest } = await codecommit.send(
  new CreatePullRequestCommand({
    title: "Merge feature/login into main",
    description: "Please review the login flow changes",
    targets: [
      {
        repositoryName: "demo-repo",
        sourceReference: "feature/login",
        destinationReference: "main",
      },
    ],
  }),
);

console.log(pullRequest?.pullRequestId);

const page = await codecommit.send(
  new ListPullRequestsCommand({
    repositoryName: "demo-repo",
    pullRequestStatus: "OPEN",
  }),
);

console.log(page.pullRequestIds);
```

If you only pass a source branch in some workflows, CodeCommit can target the repository default branch. Passing both references explicitly is clearer in automation.

### Change the default branch

```javascript
import {
  CodeCommitClient,
  UpdateDefaultBranchCommand,
} from "@aws-sdk/client-codecommit";

const codecommit = new CodeCommitClient({ region: "us-east-1" });

await codecommit.send(
  new UpdateDefaultBranchCommand({
    repositoryName: "demo-repo",
    defaultBranchName: "main",
  }),
);
```

## Error Handling

CodeCommit commands throw service exceptions with stable names. Handle the cases your workflow expects and let unexpected failures bubble up.

```javascript
import {
  CodeCommitClient,
  GetRepositoryCommand,
} from "@aws-sdk/client-codecommit";

const codecommit = new CodeCommitClient({ region: "us-east-1" });

try {
  await codecommit.send(
    new GetRepositoryCommand({ repositoryName: "missing-repo" }),
  );
} catch (error) {
  if (error?.name === "RepositoryDoesNotExistException") {
    console.error("Repository not found");
  } else {
    throw error;
  }
}
```

Common cases to expect in automation:

- `RepositoryDoesNotExistException`
- `BranchDoesNotExistException`
- `FileDoesNotExistException`
- `ParentCommitIdOutdatedException`
- validation errors around bad branch names, file paths, or references

## CodeCommit-Specific Gotchas

- `PutFile` updates one file. Use `CreateCommit` when one logical change touches multiple files.
- `GetFile` and `GetBlob` return binary data in the SDK even when CLI examples show base64 text.
- Repository metadata includes clone URLs, but actual clone, fetch, merge, and push flows still belong to Git tooling.
- Branch and commit references matter for write safety. Supplying the current `parentCommitId` avoids racing another writer.
- Approval rules, comments, pull requests, and triggers are separate API surfaces. Grant IAM permissions for the exact actions your workflow needs.
- If the repository uses a customer-managed KMS key, your IAM principal also needs the right KMS permissions.
- Do not deep-import package internals from build directories.

## When To Reach For Other Packages Or Tools

- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, STS assume-role flows, and other credential helpers.
- Git CLI or Git libraries: working trees, clone/fetch/push, conflict resolution, and local branch management.

## High-Value APIs To Know

- Repository admin: `CreateRepositoryCommand`, `GetRepositoryCommand`, `DeleteRepositoryCommand`
- Branches: `GetBranchCommand`, `ListBranchesCommand`, `UpdateDefaultBranchCommand`
- Files and content: `GetFileCommand`, `PutFileCommand`, `DeleteFileCommand`, `GetFolderCommand`, `GetBlobCommand`
- Commits and diffs: `CreateCommitCommand`, `GetCommitCommand`, `GetDifferencesCommand`
- Pull requests and reviews: `CreatePullRequestCommand`, `ListPullRequestsCommand`, `GetPullRequestCommand`, `PostCommentForPullRequestCommand`
- Merge operations: `MergePullRequestByFastForwardCommand`, `MergePullRequestBySquashCommand`, `MergePullRequestByThreeWayCommand`
