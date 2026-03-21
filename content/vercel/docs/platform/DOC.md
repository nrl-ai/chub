---
name: platform
description: "Vercel SDK for deploying, managing, and interacting with the Vercel platform via its official JavaScript/TypeScript API."
metadata:
  languages: "javascript"
  versions: "1.16.0"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "vercel,deployment,platform,serverless,edge"
---

# Vercel SDK JavaScript/TypeScript Coding Guidelines

You are a Vercel API coding expert. Help me with writing code using the Vercel API calling the official libraries and SDKs.

You can find the official SDK documentation and code samples here:
https://vercel.com/docs/rest-api/reference/sdk

## Golden Rule: Use the Correct and Current SDK

Always use the Vercel SDK to interact with the Vercel platform, which is the official library for all Vercel API interactions. Do not use legacy libraries or unofficial SDKs.

- **Library Name:** Vercel SDK
- **NPM Package:** `@vercel/sdk`
- **Legacy Libraries**: Other unofficial packages are not recommended

**Installation:**

- **Correct:** `npm install @vercel/sdk`
- **Correct:** `pnpm add @vercel/sdk`
- **Correct:** `yarn add @vercel/sdk`

**APIs and Usage:**

- **Correct:** `import { Vercel } from '@vercel/sdk'`
- **Correct:** `const vercel = new Vercel({ bearerToken: '...' })`
- **Correct:** `await vercel.deployments.getDeployments(...)`
- **Correct:** `await vercel.projects.updateProject(...)`
- **Incorrect:** `VercelClient` or `VercelAPI`
- **Incorrect:** Using unofficial REST API wrappers

**Important Notes:**

- This SDK is in beta, and there may be breaking changes between versions without a major version update
- Recommend pinning usage to a specific package version
- This is an ES Module (ESM) only package
- CommonJS users should use `await import("@vercel/sdk")`

## Initialization and API Key

The `@vercel/sdk` library requires creating a `Vercel` instance for all API calls.

- Always use `const vercel = new Vercel({ bearerToken: '...' })` to create an instance
- Set the `VERCEL_TOKEN` or `VERCEL_BEARER_TOKEN` environment variable, which will be picked up automatically
- Access tokens must be created in the Vercel dashboard with appropriate scopes

```javascript
import { Vercel } from '@vercel/sdk';

// Uses environment variable if bearerToken not specified
const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

// Or pass the bearer token directly
// const vercel = new Vercel({ bearerToken: 'your_token_here' });
```

### Creating Access Tokens

1. Navigate to your Vercel account settings
2. Go to the Tokens section
3. Create a new token with the required scopes
4. Optionally scope the token to specific teams
5. Store the token securely as `VERCEL_TOKEN` environment variable

```javascript
// Example .env file
VERCEL_TOKEN=your_access_token_here
```

### Common Authentication Errors

Permission errors (403) may occur due to:

- **Expired tokens** - Verify expiration dates in your dashboard
- **Insufficient scope access** - Ensure the token has appropriate team or account-level permissions
- **Feature unavailability** - Some features like AccessGroups require Enterprise plans

## Working with Teams

Many API operations require specifying a team. You can provide team information using either `teamId` or `slug`:

```javascript
// Using team ID
const result = await vercel.projects.getProjects({
  teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
});

// Using team slug
const result = await vercel.projects.getProjects({
  slug: 'my-team-url-slug',
});
```

To find your team ID:

```javascript
const teams = await vercel.teams.listTeams();
console.log(teams.teams);
```

## Deployments

The Vercel SDK provides comprehensive deployment management capabilities.

### Listing Deployments

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function listDeployments() {
  const result = await vercel.deployments.getDeployments({
    limit: 10,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.deployments);
}

listDeployments();
```

### Filter Deployments by Target

```javascript
const result = await vercel.deployments.getDeployments({
  limit: 20,
  target: 'production',
  teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
});

console.log(result.deployments);
```

### Filter Deployments by Project

```javascript
const result = await vercel.deployments.getDeployments({
  projectId: 'prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB',
  teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
});

console.log(result.deployments);
```

### Get a Single Deployment

```javascript
async function getDeployment(deploymentId) {
  const result = await vercel.deployments.getDeployment({
    idOrUrl: deploymentId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result);
}

getDeployment('dpl_abc123xyz');
```

### Cancel a Deployment

```javascript
async function cancelDeployment(deploymentId) {
  const result = await vercel.deployments.cancelDeployment({
    id: deploymentId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Deployment cancelled:', result);
}

cancelDeployment('dpl_abc123xyz');
```

### Delete a Deployment

```javascript
async function deleteDeployment(deploymentId) {
  const result = await vercel.deployments.deleteDeployment({
    id: deploymentId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Deployment deleted:', result);
}

deleteDeployment('dpl_abc123xyz');
```

### Upload Files for Deployment

Before creating a deployment, you need to upload the required files:

```javascript
import { Vercel } from '@vercel/sdk';
import fs from 'fs';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function uploadFile() {
  const fileContent = fs.readFileSync('./index.html', 'utf-8');

  const result = await vercel.deployments.uploadFile({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      file: fileContent,
    },
  });

  console.log('File hash:', result);
  return result;
}

uploadFile();
```

### Create a Deployment

```javascript
async function createDeployment() {
  // First upload files (as shown above), then create deployment
  const deployment = await vercel.deployments.createDeployment({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'my-project',
      files: [
        {
          file: 'index.html',
          sha: 'file_sha_from_upload',
          size: 1234,
        },
      ],
      target: 'production',
    },
  });

  console.log('Deployment created:', deployment);
}

createDeployment();
```

### Get Deployment Files

```javascript
async function getDeploymentFiles(deploymentId) {
  const result = await vercel.deployments.getDeploymentFiles({
    idOrUrl: deploymentId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.files);
}

getDeploymentFiles('dpl_abc123xyz');
```

### Get Deployment File Contents

```javascript
async function getFileContents(deploymentId, fileId) {
  const result = await vercel.deployments.getDeploymentFileContents({
    idOrUrl: deploymentId,
    fileId: fileId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result);
}

getFileContents('dpl_abc123xyz', 'file_abc123');
```

### Get Deployment Events

```javascript
async function getDeploymentEvents(deploymentId) {
  const result = await vercel.deployments.getDeploymentEvents({
    idOrUrl: deploymentId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result);
}

getDeploymentEvents('dpl_abc123xyz');
```

## Projects

The SDK provides comprehensive project management capabilities.

### List Projects

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function listProjects() {
  const result = await vercel.projects.getProjects({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.projects);
}

listProjects();
```

### Get a Single Project

```javascript
async function getProject(projectName) {
  const result = await vercel.projects.getProject({
    idOrName: projectName,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result);
}

getProject('my-project');
```

### Create a Project

```javascript
async function createProject() {
  const result = await vercel.projects.createProject({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'my-new-project',
      framework: 'nextjs',
      buildCommand: 'npm run build',
      outputDirectory: '.next',
      installCommand: 'npm install',
      devCommand: 'npm run dev',
    },
  });

  console.log('Project created:', result);
}

createProject();
```

### Update a Project

```javascript
async function updateProject(projectId) {
  const result = await vercel.projects.updateProject({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'updated-project-name',
      framework: 'nextjs',
      buildCommand: 'pnpm build',
    },
  });

  console.log('Project updated:', result);
}

updateProject('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Delete a Project

```javascript
async function deleteProject(projectId) {
  const result = await vercel.projects.deleteProject({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Project deleted:', result);
}

deleteProject('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Link Project to Git Repository

```javascript
async function linkGitRepository(projectId) {
  const result = await vercel.projects.updateProject({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      link: {
        type: 'github',
        repo: 'username/repository-name',
        repoId: 123456789,
        gitBranch: 'main',
      },
    },
  });

  console.log('Git repository linked:', result);
}

linkGitRepository('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

## Domains

Manage custom domains for your projects.

### Add a Domain to a Project

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function addDomain(projectId) {
  const result = await vercel.projects.addProjectDomain({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'www.example.com',
    },
  });

  console.log('Domain added:', result);
}

addDomain('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Add Domain with Redirect

```javascript
async function addDomainWithRedirect(projectId) {
  const result = await vercel.projects.addProjectDomain({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'www.example.com',
      redirect: 'example.com',
      redirectStatusCode: 308,
    },
  });

  console.log('Domain added with redirect:', result);
}

addDomainWithRedirect('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Add Domain for Specific Git Branch

```javascript
async function addBranchDomain(projectId) {
  const result = await vercel.projects.addProjectDomain({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'staging.example.com',
      gitBranch: 'staging',
    },
  });

  console.log('Branch domain added:', result);
}

addBranchDomain('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Get Project Domains

```javascript
async function getProjectDomains(projectId) {
  const result = await vercel.projects.getProjectDomains({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.domains);
}

getProjectDomains('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Update a Project Domain

```javascript
async function updateProjectDomain(projectId, domainName) {
  const result = await vercel.projects.updateProjectDomain({
    idOrName: projectId,
    domain: domainName,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      redirect: 'new-redirect.com',
      redirectStatusCode: 307,
    },
  });

  console.log('Domain updated:', result);
}

updateProjectDomain('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB', 'www.example.com');
```

### Remove a Domain from a Project

```javascript
async function removeDomain(projectId, domainName) {
  const result = await vercel.projects.removeProjectDomain({
    idOrName: projectId,
    domain: domainName,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Domain removed:', result);
}

removeDomain('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB', 'www.example.com');
```

### Verify a Project Domain

```javascript
async function verifyDomain(projectId, domainName) {
  const result = await vercel.projects.verifyProjectDomain({
    idOrName: projectId,
    domain: domainName,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Domain verification result:', result);
}

verifyDomain('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB', 'www.example.com');
```

### List All Domains

```javascript
async function listDomains() {
  const result = await vercel.domains.listDomains({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.domains);
}

listDomains();
```

### Get Domain Information

```javascript
async function getDomain(domainName) {
  const result = await vercel.domains.getDomain({
    domain: domainName,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result);
}

getDomain('example.com');
```

### Buy a Domain

```javascript
async function buyDomain() {
  const result = await vercel.domains.createOrTransferDomain({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'example.com',
    },
  });

  console.log('Domain purchased:', result);
}

buyDomain();
```

### Delete a Domain

```javascript
async function deleteDomain(domainName) {
  const result = await vercel.domains.deleteDomain({
    domain: domainName,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Domain deleted:', result);
}

deleteDomain('example.com');
```

## DNS Records

Manage DNS records for your domains.

### Get DNS Records

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function getDnsRecords(domainName) {
  const result = await vercel.dns.getDnsRecords({
    domain: domainName,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.records);
}

getDnsRecords('example.com');
```

### Create a DNS Record

```javascript
async function createDnsRecord(domainName) {
  const result = await vercel.dns.createDnsRecord({
    domain: domainName,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'subdomain',
      type: 'A',
      value: '192.0.2.1',
      ttl: 60,
    },
  });

  console.log('DNS record created:', result);
}

createDnsRecord('example.com');
```

### Create CNAME Record

```javascript
async function createCnameRecord(domainName) {
  const result = await vercel.dns.createDnsRecord({
    domain: domainName,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'www',
      type: 'CNAME',
      value: 'example.com',
      ttl: 60,
    },
  });

  console.log('CNAME record created:', result);
}

createCnameRecord('example.com');
```

### Create MX Record

```javascript
async function createMxRecord(domainName) {
  const result = await vercel.dns.createDnsRecord({
    domain: domainName,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: '@',
      type: 'MX',
      value: 'mail.example.com',
      mxPriority: 10,
      ttl: 60,
    },
  });

  console.log('MX record created:', result);
}

createMxRecord('example.com');
```

### Update a DNS Record

```javascript
async function updateDnsRecord(domainName, recordId) {
  const result = await vercel.dns.patchDnsRecord({
    domain: domainName,
    recordId: recordId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      value: '192.0.2.2',
      ttl: 120,
    },
  });

  console.log('DNS record updated:', result);
}

updateDnsRecord('example.com', 'rec_abc123');
```

### Delete a DNS Record

```javascript
async function deleteDnsRecord(domainName, recordId) {
  const result = await vercel.dns.deleteDnsRecord({
    domain: domainName,
    recordId: recordId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('DNS record deleted:', result);
}

deleteDnsRecord('example.com', 'rec_abc123');
```

## Environment Variables

Manage environment variables for your projects.

### Get Project Environment Variables

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function getEnvVars(projectId) {
  const result = await vercel.projects.getProjectEnvs({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.envs);
}

getEnvVars('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Create Environment Variables

```javascript
async function createEnvVars(projectId) {
  const result = await vercel.projects.createProjectEnv({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    upsert: 'true',
    requestBody: [
      {
        key: 'API_KEY',
        value: 'secret_value',
        target: ['production', 'preview'],
        type: 'encrypted',
      },
      {
        key: 'DEBUG',
        value: 'true',
        target: ['development'],
        type: 'plain',
      },
    ],
  });

  console.log('Environment variables created:', result);
}

createEnvVars('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Create Encrypted Environment Variable

```javascript
async function createSecretEnvVar(projectId) {
  const result = await vercel.projects.createProjectEnv({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: [
      {
        key: 'DATABASE_URL',
        value: 'postgresql://user:pass@host:5432/db',
        target: ['production'],
        type: 'encrypted',
      },
    ],
  });

  console.log('Secret environment variable created:', result);
}

createSecretEnvVar('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Create Multi-Environment Variable

```javascript
async function createMultiEnvVar(projectId) {
  const result = await vercel.projects.createProjectEnv({
    idOrName: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: [
      {
        key: 'API_URL',
        value: 'https://api.example.com',
        target: ['production', 'preview', 'development'],
        type: 'plain',
      },
    ],
  });

  console.log('Multi-environment variable created:', result);
}

createMultiEnvVar('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Get a Single Environment Variable

```javascript
async function getEnvVar(projectId, envId) {
  const result = await vercel.projects.getProjectEnv({
    idOrName: projectId,
    id: envId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result);
}

getEnvVar('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB', 'env_abc123');
```

### Edit an Environment Variable

```javascript
async function editEnvVar(projectId, envId) {
  const result = await vercel.projects.editProjectEnv({
    idOrName: projectId,
    id: envId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      value: 'new_value',
      target: ['production', 'preview'],
    },
  });

  console.log('Environment variable updated:', result);
}

editEnvVar('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB', 'env_abc123');
```

### Delete an Environment Variable

```javascript
async function deleteEnvVar(projectId, envId) {
  const result = await vercel.projects.deleteProjectEnv({
    idOrName: projectId,
    id: envId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Environment variable deleted:', result);
}

deleteEnvVar('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB', 'env_abc123');
```

## Teams

Manage teams and team members.

### List Teams

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function listTeams() {
  const result = await vercel.teams.listTeams();
  console.log(result.teams);
}

listTeams();
```

### Get Team Information

```javascript
async function getTeam(teamId) {
  const result = await vercel.teams.getTeam({
    teamId: teamId,
  });

  console.log(result);
}

getTeam('team_1a2b3c4d5e6f7g8h9i0j1k2l');
```

### Create a Team

```javascript
async function createTeam() {
  const result = await vercel.teams.createTeam({
    requestBody: {
      name: 'My New Team',
      slug: 'my-new-team',
    },
  });

  console.log('Team created:', result);
}

createTeam();
```

### Update Team Settings

```javascript
async function updateTeam(teamId) {
  const result = await vercel.teams.updateTeam({
    teamId: teamId,
    requestBody: {
      name: 'Updated Team Name',
      description: 'This is my team description',
    },
  });

  console.log('Team updated:', result);
}

updateTeam('team_1a2b3c4d5e6f7g8h9i0j1k2l');
```

### Delete a Team

```javascript
async function deleteTeam(teamId) {
  const result = await vercel.teams.deleteTeam({
    teamId: teamId,
  });

  console.log('Team deleted:', result);
}

deleteTeam('team_1a2b3c4d5e6f7g8h9i0j1k2l');
```

### List Team Members

```javascript
async function listTeamMembers(teamId) {
  const result = await vercel.teams.getTeamMembers({
    teamId: teamId,
  });

  console.log(result.members);
}

listTeamMembers('team_1a2b3c4d5e6f7g8h9i0j1k2l');
```

### Invite Member to Team

```javascript
async function inviteTeamMember(teamId) {
  const result = await vercel.teams.inviteUserToTeam({
    teamId: teamId,
    requestBody: {
      email: 'user@example.com',
      role: 'MEMBER',
    },
  });

  console.log('Team member invited:', result);
}

inviteTeamMember('team_1a2b3c4d5e6f7g8h9i0j1k2l');
```

### Update Team Member Role

```javascript
async function updateMemberRole(teamId, memberId) {
  const result = await vercel.teams.updateTeamMember({
    teamId: teamId,
    memberId: memberId,
    requestBody: {
      role: 'OWNER',
    },
  });

  console.log('Member role updated:', result);
}

updateMemberRole('team_1a2b3c4d5e6f7g8h9i0j1k2l', 'member_abc123');
```

### Remove Team Member

```javascript
async function removeTeamMember(teamId, memberId) {
  const result = await vercel.teams.deleteTeamMember({
    teamId: teamId,
    memberId: memberId,
  });

  console.log('Member removed:', result);
}

removeTeamMember('team_1a2b3c4d5e6f7g8h9i0j1k2l', 'member_abc123');
```

## Access Groups (Enterprise)

Access Groups is an Enterprise-only feature for advanced team management.

### List Access Groups

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function listAccessGroups() {
  const result = await vercel.accessGroups.listAccessGroups({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.accessGroups);
}

listAccessGroups();
```

### Filter Access Groups by Project

```javascript
async function getProjectAccessGroups(projectId) {
  const result = await vercel.accessGroups.listAccessGroups({
    projectId: projectId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.accessGroups);
}

getProjectAccessGroups('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Search Access Groups

```javascript
async function searchAccessGroups(searchTerm) {
  const result = await vercel.accessGroups.listAccessGroups({
    search: searchTerm,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.accessGroups);
}

searchAccessGroups('engineering');
```

### Create Access Group

```javascript
async function createAccessGroup() {
  const result = await vercel.accessGroups.createAccessGroup({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'Engineering Team',
      projects: ['prj_abc123', 'prj_def456'],
      members: ['member_123', 'member_456'],
    },
  });

  console.log('Access group created:', result);
}

createAccessGroup();
```

### Update Access Group

```javascript
async function updateAccessGroup(groupId) {
  const result = await vercel.accessGroups.updateAccessGroup({
    idOrName: groupId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'Updated Engineering Team',
      projects: ['prj_abc123', 'prj_def456', 'prj_ghi789'],
    },
  });

  console.log('Access group updated:', result);
}

updateAccessGroup('ag_abc123');
```

### Delete Access Group

```javascript
async function deleteAccessGroup(groupId) {
  const result = await vercel.accessGroups.deleteAccessGroup({
    idOrName: groupId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Access group deleted:', result);
}

deleteAccessGroup('ag_abc123');
```

## Webhooks

Set up webhooks to receive notifications about events in your Vercel projects.

### List Webhooks

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function listWebhooks() {
  const result = await vercel.webhooks.listWebhooks({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.webhooks);
}

listWebhooks();
```

### Create a Webhook

```javascript
async function createWebhook() {
  const result = await vercel.webhooks.createWebhook({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      url: 'https://example.com/webhook',
      events: ['deployment.created', 'deployment.succeeded'],
    },
  });

  console.log('Webhook created:', result);
}

createWebhook();
```

### Create Project-Specific Webhook

```javascript
async function createProjectWebhook(projectId) {
  const result = await vercel.webhooks.createWebhook({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      url: 'https://example.com/webhook',
      events: ['deployment.created', 'deployment.succeeded', 'deployment.failed'],
      projectIds: [projectId],
    },
  });

  console.log('Project webhook created:', result);
}

createProjectWebhook('prj_12HKQaOmR5t5Uy6vdcQsNIiZgHGB');
```

### Get Webhook Details

```javascript
async function getWebhook(webhookId) {
  const result = await vercel.webhooks.getWebhook({
    id: webhookId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result);
}

getWebhook('hook_abc123');
```

### Delete a Webhook

```javascript
async function deleteWebhook(webhookId) {
  const result = await vercel.webhooks.deleteWebhook({
    id: webhookId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Webhook deleted:', result);
}

deleteWebhook('hook_abc123');
```

## Artifacts (Remote Caching)

Manage artifact caching for build optimization.

### Record Artifact Cache Events

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function recordCacheEvents() {
  const result = await vercel.artifacts.recordEvents({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: [
      {
        sessionId: 'session_abc123',
        source: 'REMOTE',
        event: 'HIT',
        hash: 'hash_def456',
        duration: 123,
      },
      {
        sessionId: 'session_abc123',
        source: 'LOCAL',
        event: 'MISS',
        hash: 'hash_ghi789',
        duration: 456,
      },
    ],
  });

  console.log('Cache events recorded:', result);
}

recordCacheEvents();
```

### Get Artifact Status

```javascript
async function getArtifactStatus(hash) {
  const result = await vercel.artifacts.artifactExists({
    hash: hash,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Artifact exists:', result);
}

getArtifactStatus('hash_abc123');
```

### Upload Artifact

```javascript
import fs from 'fs';

async function uploadArtifact() {
  const fileContent = fs.readFileSync('./artifact.tar.gz');

  const result = await vercel.artifacts.uploadArtifact({
    hash: 'hash_abc123',
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: fileContent,
  });

  console.log('Artifact uploaded:', result);
}

uploadArtifact();
```

### Download Artifact

```javascript
async function downloadArtifact(hash) {
  const result = await vercel.artifacts.downloadArtifact({
    hash: hash,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Artifact downloaded:', result);
}

downloadArtifact('hash_abc123');
```

## Checks

Manage deployment checks and integration actions.

### Get Deployment Checks

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function getDeploymentChecks(deploymentId) {
  const result = await vercel.checks.listChecks({
    deploymentId: deploymentId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.checks);
}

getDeploymentChecks('dpl_abc123xyz');
```

### Create a Check

```javascript
async function createCheck(deploymentId) {
  const result = await vercel.checks.createCheck({
    deploymentId: deploymentId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'Security Scan',
      path: '/',
      status: 'running',
      blocking: true,
    },
  });

  console.log('Check created:', result);
}

createCheck('dpl_abc123xyz');
```

### Update a Check

```javascript
async function updateCheck(deploymentId, checkId) {
  const result = await vercel.checks.updateCheck({
    deploymentId: deploymentId,
    checkId: checkId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      status: 'completed',
      conclusion: 'succeeded',
      output: {
        summary: 'Security scan passed',
      },
    },
  });

  console.log('Check updated:', result);
}

updateCheck('dpl_abc123xyz', 'check_abc123');
```

### Rerequest a Check

```javascript
async function rerequestCheck(deploymentId, checkId) {
  const result = await vercel.checks.rerequestCheck({
    deploymentId: deploymentId,
    checkId: checkId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Check rerequested:', result);
}

rerequestCheck('dpl_abc123xyz', 'check_abc123');
```

## Logs

Access deployment and build logs.

### Get Deployment Logs

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function getDeploymentLogs(deploymentId) {
  const result = await vercel.logs.getDeploymentLogs({
    id: deploymentId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result);
}

getDeploymentLogs('dpl_abc123xyz');
```

### Get Build Logs

```javascript
async function getBuildLogs(deploymentId) {
  const result = await vercel.logs.getDeploymentLogs({
    id: deploymentId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    direction: 'forward',
    limit: 100,
  });

  console.log(result);
}

getBuildLogs('dpl_abc123xyz');
```

### Filter Logs by Time

```javascript
async function getLogsInTimeRange(deploymentId, since, until) {
  const result = await vercel.logs.getDeploymentLogs({
    id: deploymentId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    since: since,
    until: until,
  });

  console.log(result);
}

getLogsInTimeRange('dpl_abc123xyz', 1609459200000, 1609545600000);
```

## Aliases

Manage deployment aliases.

### List Aliases

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function listAliases() {
  const result = await vercel.aliases.listAliases({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.aliases);
}

listAliases();
```

### Assign Alias to Deployment

```javascript
async function assignAlias(deploymentId, alias) {
  const result = await vercel.aliases.assignAlias({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      alias: alias,
      deploymentId: deploymentId,
    },
  });

  console.log('Alias assigned:', result);
}

assignAlias('dpl_abc123xyz', 'my-app.example.com');
```

### Get Alias Information

```javascript
async function getAlias(aliasId) {
  const result = await vercel.aliases.getAlias({
    id: aliasId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result);
}

getAlias('alias_abc123');
```

### Delete an Alias

```javascript
async function deleteAlias(aliasId) {
  const result = await vercel.aliases.deleteAlias({
    id: aliasId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Alias deleted:', result);
}

deleteAlias('alias_abc123');
```

## Authentication

Manage authentication tokens and settings.

### Get Current User

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function getCurrentUser() {
  const result = await vercel.user.getAuthUser();
  console.log(result);
}

getCurrentUser();
```

### List Access Tokens

```javascript
async function listTokens() {
  const result = await vercel.authentication.listTokens({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.tokens);
}

listTokens();
```

### Delete an Access Token

```javascript
async function deleteToken(tokenId) {
  const result = await vercel.authentication.deleteToken({
    id: tokenId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Token deleted:', result);
}

deleteToken('token_abc123');
```

## Edge Config

Manage Edge Config stores for ultra-low latency global data.

### List Edge Configs

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function listEdgeConfigs() {
  const result = await vercel.edgeConfig.getEdgeConfigs({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result.edgeConfigs);
}

listEdgeConfigs();
```

### Create Edge Config

```javascript
async function createEdgeConfig() {
  const result = await vercel.edgeConfig.createEdgeConfig({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      name: 'my-edge-config',
    },
  });

  console.log('Edge Config created:', result);
}

createEdgeConfig();
```

### Get Edge Config

```javascript
async function getEdgeConfig(edgeConfigId) {
  const result = await vercel.edgeConfig.getEdgeConfig({
    edgeConfigId: edgeConfigId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log(result);
}

getEdgeConfig('ecfg_abc123');
```

### Update Edge Config Items

```javascript
async function updateEdgeConfigItems(edgeConfigId) {
  const result = await vercel.edgeConfig.updateEdgeConfigItems({
    edgeConfigId: edgeConfigId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    requestBody: {
      items: [
        {
          operation: 'upsert',
          key: 'feature_flag_new_ui',
          value: true,
        },
        {
          operation: 'upsert',
          key: 'welcome_message',
          value: 'Hello from Edge Config!',
        },
      ],
    },
  });

  console.log('Edge Config items updated:', result);
}

updateEdgeConfigItems('ecfg_abc123');
```

### Delete Edge Config

```javascript
async function deleteEdgeConfig(edgeConfigId) {
  const result = await vercel.edgeConfig.deleteEdgeConfig({
    edgeConfigId: edgeConfigId,
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });

  console.log('Edge Config deleted:', result);
}

deleteEdgeConfig('ecfg_abc123');
```

## Error Handling

The SDK provides comprehensive error handling capabilities.

### Basic Error Handling

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

try {
  const result = await vercel.projects.getProject({
    idOrName: 'my-project',
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  });
  console.log(result);
} catch (error) {
  if (error.statusCode === 404) {
    console.error('Project not found');
  } else if (error.statusCode === 403) {
    console.error('Permission denied - check token scopes');
  } else if (error.statusCode === 401) {
    console.error('Authentication failed - check your token');
  } else {
    console.error('Error:', error.message);
  }
}
```

### Advanced Error Handling

```javascript
async function safeApiCall() {
  try {
    const result = await vercel.deployments.getDeployments({
      teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    });
    return result;
  } catch (error) {
    if (error.statusCode >= 500) {
      console.error('Server error - retry later');
    } else if (error.statusCode === 429) {
      console.error('Rate limit exceeded - wait before retrying');
    } else if (error.statusCode >= 400 && error.statusCode < 500) {
      console.error('Client error:', error.message);
    }
    throw error;
  }
}
```

## Advanced Configuration

### Custom HTTP Client Options

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
  serverURL: 'https://api.vercel.com',
  retryConfig: {
    strategy: 'backoff',
    backoff: {
      initialInterval: 500,
      maxInterval: 60000,
      exponent: 1.5,
      maxElapsedTime: 3600000,
    },
    retryConnectionErrors: true,
  },
});
```

### Setting Timeouts

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
  timeoutMs: 30000, // 30 seconds
});
```

### Debug Mode

```javascript
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
  debugLogger: {
    log: (message) => console.log('[DEBUG]', message),
  },
});
```

## Pagination

Many API endpoints support pagination for handling large result sets.

### Manual Pagination

```javascript
async function getAllProjects() {
  let allProjects = [];
  let limit = 20;
  let until = undefined;

  while (true) {
    const result = await vercel.projects.getProjects({
      teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
      limit: limit,
      until: until,
    });

    allProjects.push(...result.projects);

    if (!result.pagination || !result.pagination.next) {
      break;
    }

    until = result.pagination.next;
  }

  return allProjects;
}
```

### Pagination with Limit

```javascript
async function getRecentDeployments(maxResults = 100) {
  const result = await vercel.deployments.getDeployments({
    teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
    limit: Math.min(maxResults, 100),
  });

  return result.deployments.slice(0, maxResults);
}
```

## Complete Example: Deploy a Project

Here's a complete example showing how to create a project, upload files, and create a deployment:

```javascript
import { Vercel } from '@vercel/sdk';
import fs from 'fs';
import path from 'path';

const vercel = new Vercel({
  bearerToken: process.env.VERCEL_TOKEN,
});

async function deployProject() {
  try {
    // 1. Create a project
    const project = await vercel.projects.createProject({
      teamId: process.env.VERCEL_TEAM_ID,
      requestBody: {
        name: 'my-app',
        framework: 'nextjs',
      },
    });
    console.log('Project created:', project.id);

    // 2. Upload files
    const files = [
      { path: 'index.html', content: '<html><body>Hello World</body></html>' },
      { path: 'package.json', content: '{"name":"my-app","version":"1.0.0"}' },
    ];

    const uploadedFiles = [];
    for (const file of files) {
      const hash = await vercel.deployments.uploadFile({
        teamId: process.env.VERCEL_TEAM_ID,
        requestBody: {
          file: file.content,
        },
      });
      uploadedFiles.push({
        file: file.path,
        sha: hash,
        size: file.content.length,
      });
    }
    console.log('Files uploaded');

    // 3. Create deployment
    const deployment = await vercel.deployments.createDeployment({
      teamId: process.env.VERCEL_TEAM_ID,
      requestBody: {
        name: 'my-app',
        files: uploadedFiles,
        projectSettings: {
          framework: 'nextjs',
        },
        target: 'production',
      },
    });
    console.log('Deployment created:', deployment.url);

    // 4. Wait for deployment to complete
    let status = 'BUILDING';
    while (status === 'BUILDING' || status === 'QUEUED') {
      await new Promise((resolve) => setTimeout(resolve, 5000));
      const deploymentStatus = await vercel.deployments.getDeployment({
        idOrUrl: deployment.id,
        teamId: process.env.VERCEL_TEAM_ID,
      });
      status = deploymentStatus.readyState;
      console.log('Deployment status:', status);
    }

    if (status === 'READY') {
      console.log('Deployment successful:', `https://${deployment.url}`);
    } else {
      console.error('Deployment failed:', status);
    }
  } catch (error) {
    console.error('Error:', error.message);
  }
}

deployProject();
```

## Useful Links

- Documentation: https://vercel.com/docs
- API Reference: https://vercel.com/docs/rest-api/reference
- SDK Documentation: https://vercel.com/docs/rest-api/reference/sdk
- GitHub Repository: https://github.com/vercel/sdk
- Support: https://vercel.com/support
