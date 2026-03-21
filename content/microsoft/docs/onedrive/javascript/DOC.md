---
name: onedrive
description: "Microsoft OneDrive API coding guidelines for JavaScript/TypeScript using the official Microsoft Graph SDK"
metadata:
  languages: "javascript"
  versions: "3.0.7"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "microsoft,onedrive,storage,graph-api,files"
---

# Microsoft OneDrive API Coding Guidelines (JavaScript/TypeScript)

You are a **Microsoft OneDrive API coding expert**. Help me write correct, idiomatic JavaScript/TypeScript that accesses OneDrive files and folders using the official Microsoft Graph SDK.

Use **only official Microsoft sources** for behavior, fields, and constraints. This guide summarizes key patterns for both **Node.js** and **browser** applications.

> Ground truth: Microsoft Graph OneDrive API documentation on learn.microsoft.com.


## Golden Rule: Use the Official Microsoft Graph SDK

**ALWAYS use `@microsoft/microsoft-graph-client` version 3.0.7 or later** for OneDrive operations. This is the official Microsoft Graph JavaScript SDK that provides access to OneDrive, SharePoint, and all other Microsoft Graph APIs.

**DO NOT use**:
- Deprecated `onedrivesdk` package (obsolete)
- Direct REST calls without the SDK (unless absolutely necessary)
- Unofficial third-party OneDrive libraries

**Install (Node.js):**
```bash
npm install @microsoft/microsoft-graph-client
npm install @azure/identity
npm install @microsoft/microsoft-graph-client/authProviders/azureTokenCredentials
```

**For TypeScript projects, add type definitions:**
```bash
npm install --save-dev @microsoft/microsoft-graph-types
```


## Installation

### Complete Setup for Node.js Applications

```bash
# Core Microsoft Graph SDK
npm install @microsoft/microsoft-graph-client

# Azure authentication library
npm install @azure/identity

# Install isomorphic-fetch for Node.js environments
npm install isomorphic-fetch

# TypeScript types (optional but recommended)
npm install --save-dev @microsoft/microsoft-graph-types
```

### Browser Applications

For browser-based applications, you can load the SDK via CDN or bundle it with your application:

```html
<!-- Microsoft Graph Client -->
<script src="https://cdn.jsdelivr.net/npm/@microsoft/microsoft-graph-client/lib/graph-js-sdk.js"></script>

<!-- For authentication in browser, use MSAL -->
<script src="https://alcdn.msauth.net/browser/2.32.0/js/msal-browser.min.js"></script>
```


## Authentication

OneDrive access through Microsoft Graph requires OAuth 2.0 authentication. You need to register an application in Azure Active Directory (Azure AD) to obtain credentials.

### Azure AD App Registration

1. Go to [Azure Portal](https://portal.azure.com) → Azure Active Directory → App registrations
2. Create a new registration
3. Note your **Application (client) ID** and **Directory (tenant) ID**
4. Create a client secret under "Certificates & secrets"
5. Add API permissions: Microsoft Graph → **Files.Read**, **Files.ReadWrite**, **Files.Read.All**, **Files.ReadWrite.All**
6. Grant admin consent for the permissions

### Required Scopes

Common OneDrive permission scopes:

```javascript
// Read-only access to user's files
const SCOPES_READONLY = ['https://graph.microsoft.com/Files.Read'];

// Read/write access to user's files
const SCOPES_READWRITE = ['https://graph.microsoft.com/Files.ReadWrite'];

// Read all files user can access (including shared)
const SCOPES_READ_ALL = ['https://graph.microsoft.com/Files.Read.All'];

// Full access to all files user can access
const SCOPES_READWRITE_ALL = ['https://graph.microsoft.com/Files.ReadWrite.All'];

// Application permissions (no user context, requires admin consent)
const SCOPES_APP = ['https://graph.microsoft.com/.default'];
```


## Initialization

### Node.js with Client Credentials (Service/Daemon Apps)

For server-side applications using application permissions:

```typescript
import { Client } from '@microsoft/microsoft-graph-client';
import { TokenCredentialAuthenticationProvider } from '@microsoft/microsoft-graph-client/authProviders/azureTokenCredentials';
import { ClientSecretCredential } from '@azure/identity';
import 'isomorphic-fetch';

// Environment variables
const tenantId = process.env.AZURE_TENANT_ID;
const clientId = process.env.AZURE_CLIENT_ID;
const clientSecret = process.env.AZURE_CLIENT_SECRET;

// Create credential
const credential = new ClientSecretCredential(tenantId, clientId, clientSecret);

// Create authentication provider
const authProvider = new TokenCredentialAuthenticationProvider(credential, {
  scopes: ['https://graph.microsoft.com/.default']
});

// Initialize Microsoft Graph client
const client = Client.initWithMiddleware({ authProvider });

// Example: List files in user's OneDrive root
async function listFiles(userId) {
  const response = await client
    .api(`/users/${userId}/drive/root/children`)
    .get();

  return response.value;
}
```

### Node.js with Device Code Flow (Interactive)

For CLI applications that need user interaction:

```typescript
import { Client } from '@microsoft/microsoft-graph-client';
import { TokenCredentialAuthenticationProvider } from '@microsoft/microsoft-graph-client/authProviders/azureTokenCredentials';
import { DeviceCodeCredential } from '@azure/identity';
import 'isomorphic-fetch';

const credential = new DeviceCodeCredential({
  tenantId: process.env.AZURE_TENANT_ID,
  clientId: process.env.AZURE_CLIENT_ID,
  userPromptCallback: (info) => {
    console.log(info.message);
  }
});

const authProvider = new TokenCredentialAuthenticationProvider(credential, {
  scopes: ['Files.ReadWrite', 'Files.Read.All']
});

const client = Client.initWithMiddleware({ authProvider });
```

### Node.js with On-Behalf-Of Flow (Web APIs)

For web APIs that act on behalf of a signed-in user:

```typescript
import { Client } from '@microsoft/microsoft-graph-client';
import { TokenCredentialAuthenticationProvider } from '@microsoft/microsoft-graph-client/authProviders/azureTokenCredentials';
import { OnBehalfOfCredential } from '@azure/identity';

const credential = new OnBehalfOfCredential({
  tenantId: process.env.AZURE_TENANT_ID,
  clientId: process.env.AZURE_CLIENT_ID,
  clientSecret: process.env.AZURE_CLIENT_SECRET,
  userAssertionToken: userToken // Token from incoming request
});

const authProvider = new TokenCredentialAuthenticationProvider(credential, {
  scopes: ['https://graph.microsoft.com/Files.ReadWrite']
});

const client = Client.initWithMiddleware({ authProvider });
```

### Browser with MSAL (Interactive User Sign-In)

For single-page applications:

```typescript
import * as msal from '@azure/msal-browser';
import { Client } from '@microsoft/microsoft-graph-client';

// MSAL configuration
const msalConfig = {
  auth: {
    clientId: 'YOUR_CLIENT_ID',
    authority: 'https://login.microsoftonline.com/YOUR_TENANT_ID',
    redirectUri: 'http://localhost:3000'
  }
};

const msalInstance = new msal.PublicClientApplication(msalConfig);

// Custom authentication provider for MSAL
class MsalAuthenticationProvider {
  constructor(msalInstance, scopes) {
    this.msalInstance = msalInstance;
    this.scopes = scopes;
  }

  async getAccessToken() {
    const accounts = this.msalInstance.getAllAccounts();
    if (accounts.length === 0) {
      await this.msalInstance.loginPopup({ scopes: this.scopes });
    }

    const request = {
      scopes: this.scopes,
      account: this.msalInstance.getAllAccounts()[0]
    };

    try {
      const response = await this.msalInstance.acquireTokenSilent(request);
      return response.accessToken;
    } catch (error) {
      const response = await this.msalInstance.acquireTokenPopup(request);
      return response.accessToken;
    }
  }
}

// Initialize Graph client
const authProvider = new MsalAuthenticationProvider(
  msalInstance,
  ['Files.ReadWrite']
);

const client = Client.initWithMiddleware({
  authProvider: (done) => {
    authProvider.getAccessToken().then(token => {
      done(null, token);
    }).catch(err => done(err, null));
  }
});
```

### Environment Variables Setup

Create a `.env` file:

```bash
AZURE_TENANT_ID=your-tenant-id
AZURE_CLIENT_ID=your-client-id
AZURE_CLIENT_SECRET=your-client-secret
USER_ID=user@domain.com
```

Load environment variables:

```typescript
import dotenv from 'dotenv';
dotenv.config();
```


## Core API Surfaces


### 1. Listing Files and Folders

#### List Root Folder Contents

```typescript
// Current user's OneDrive root
const result = await client.api('/me/drive/root/children').get();

console.log(result.value); // Array of DriveItem objects
```

#### List Specific Folder Contents

```typescript
// By item ID
const result = await client
  .api(`/me/drive/items/{item-id}/children`)
  .get();

// By path
const result = await client
  .api('/me/drive/root:/Documents/Projects:/children')
  .get();

// With pagination
let items = [];
let nextLink = '/me/drive/root/children';

while (nextLink) {
  const response = await client.api(nextLink).get();
  items = items.concat(response.value);
  nextLink = response['@odata.nextLink'];
}
```

#### Advanced Listing with Query Parameters

```typescript
// Select specific fields
const result = await client
  .api('/me/drive/root/children')
  .select('id,name,size,createdDateTime,lastModifiedDateTime')
  .get();

// Filter files by type
const result = await client
  .api('/me/drive/root/children')
  .filter('file ne null')
  .get();

// Order by name
const result = await client
  .api('/me/drive/root/children')
  .orderby('name asc')
  .get();

// Limit results
const result = await client
  .api('/me/drive/root/children')
  .top(10)
  .get();

// Expand related properties
const result = await client
  .api('/me/drive/root/children')
  .expand('thumbnails')
  .get();

// Combine multiple query options
const result = await client
  .api('/me/drive/root/children')
  .select('id,name,size,file')
  .filter('file ne null')
  .orderby('lastModifiedDateTime desc')
  .top(20)
  .get();
```

#### List All Drives

```typescript
// List all drives accessible to user
const drives = await client.api('/me/drives').get();

// Get specific drive
const drive = await client.api('/drives/{drive-id}').get();

// Get default drive
const defaultDrive = await client.api('/me/drive').get();
```


### 2. Getting File/Folder Metadata

#### Get Item Metadata

```typescript
// By item ID
const item = await client
  .api(`/me/drive/items/{item-id}`)
  .get();

console.log(item.id);
console.log(item.name);
console.log(item.size);
console.log(item.createdDateTime);
console.log(item.lastModifiedDateTime);
console.log(item.webUrl);

// By path
const item = await client
  .api('/me/drive/root:/Documents/report.pdf')
  .get();
```

#### Get File with Specific Properties

```typescript
const item = await client
  .api('/me/drive/items/{item-id}')
  .select('id,name,size,file,@microsoft.graph.downloadUrl')
  .get();

// Access file-specific properties
if (item.file) {
  console.log('MIME type:', item.file.mimeType);
  console.log('Hashes:', item.file.hashes);
}

// Get download URL
console.log('Download URL:', item['@microsoft.graph.downloadUrl']);
```


### 3. Downloading Files

#### Simple Download (Small Files)

```typescript
import fs from 'fs';
import { Readable } from 'stream';

// Download file content
const fileStream = await client
  .api(`/me/drive/items/{item-id}/content`)
  .getStream();

// Save to disk
const writeStream = fs.createWriteStream('./downloaded-file.pdf');
fileStream.pipe(writeStream);

await new Promise((resolve, reject) => {
  writeStream.on('finish', resolve);
  writeStream.on('error', reject);
});
```

#### Download with Pre-Authenticated URL

```typescript
import fetch from 'node-fetch';
import fs from 'fs';

// Get item with download URL
const item = await client
  .api('/me/drive/items/{item-id}')
  .select('@microsoft.graph.downloadUrl')
  .get();

const downloadUrl = item['@microsoft.graph.downloadUrl'];

// Download using the URL (valid for a few minutes)
const response = await fetch(downloadUrl);
const buffer = await response.buffer();
fs.writeFileSync('./file.dat', buffer);
```

#### Download Specific Byte Range

```typescript
// Download partial file content
const stream = await client
  .api(`/me/drive/items/{item-id}/content`)
  .header('Range', 'bytes=0-1023')
  .getStream();
```


### 4. Uploading Files

#### Simple Upload (Files < 4MB)

```typescript
import fs from 'fs';

// Upload file from buffer or stream
const fileContent = fs.readFileSync('./local-file.pdf');

const uploadedFile = await client
  .api('/me/drive/root:/Documents/uploaded-file.pdf:/content')
  .putStream(fs.createReadStream('./local-file.pdf'));

console.log('Uploaded file ID:', uploadedFile.id);

// Or upload with buffer
const uploadedFile = await client
  .api('/me/drive/items/{parent-folder-id}:/filename.txt:/content')
  .put(fileContent);
```

#### Upload to Specific Folder

```typescript
// Upload to folder by ID
const file = await client
  .api(`/me/drive/items/{folder-id}:/newfile.txt:/content`)
  .put('File content here');

// Upload to folder by path
const file = await client
  .api('/me/drive/root:/Documents/Projects:/report.pdf:/content')
  .putStream(fs.createReadStream('./report.pdf'));
```

#### Large File Upload (Files > 4MB) - Resumable Upload Session

```typescript
import { LargeFileUploadTask, StreamUpload } from '@microsoft/microsoft-graph-client';
import fs from 'fs';

// Step 1: Create upload session
const uploadSession = await client
  .api('/me/drive/root:/large-file.zip:/createUploadSession')
  .post({
    item: {
      '@microsoft.graph.conflictBehavior': 'rename',
      name: 'large-file.zip'
    }
  });

// Step 2: Upload file in chunks
const fileObject = new StreamUpload(
  fs.createReadStream('./large-file.zip'),
  'large-file.zip',
  fs.statSync('./large-file.zip').size
);

const task = new LargeFileUploadTask(client, fileObject, uploadSession);

const uploadResult = await task.upload();

console.log('Upload complete:', uploadResult);
```

#### Advanced Large File Upload with Progress Tracking

```typescript
import { LargeFileUploadTask, StreamUpload, UploadEventHandlers } from '@microsoft/microsoft-graph-client';
import fs from 'fs';

const fileName = 'video.mp4';
const filePath = './video.mp4';
const stats = fs.statSync(filePath);
const fileSize = stats.size;

// Create upload session with conflict resolution
const uploadSession = await client
  .api('/me/drive/root:/Videos:/video.mp4:/createUploadSession')
  .post({
    item: {
      '@microsoft.graph.conflictBehavior': 'replace', // or 'fail', 'rename'
      name: fileName
    }
  });

// Configure upload
const fileStream = fs.createReadStream(filePath);
const fileObject = new StreamUpload(fileStream, fileName, fileSize);

// Upload with custom chunk size (must be multiple of 320 KB)
const options = {
  rangeSize: 1024 * 1024 * 10, // 10 MB chunks (recommended: 5-10 MB)
  uploadEventHandlers: {
    progress: (range, extraCallbackParam) => {
      const percentage = ((range?.minValue || 0) / fileSize) * 100;
      console.log(`Uploaded ${percentage.toFixed(2)}%`);
    },
    extraCallbackParam: null
  }
};

const task = new LargeFileUploadTask(client, fileObject, uploadSession);
const uploadResult = await task.upload();

console.log('File uploaded successfully:', uploadResult.id);
```

#### Resume Interrupted Upload

```typescript
// If upload fails, you can resume from where it stopped
const uploadSession = await client
  .api('/me/drive/root:/large-file.zip:/createUploadSession')
  .post({
    item: {
      '@microsoft.graph.conflictBehavior': 'replace',
      name: 'large-file.zip'
    }
  });

const fileObject = new StreamUpload(
  fs.createReadStream('./large-file.zip'),
  'large-file.zip',
  fs.statSync('./large-file.zip').size
);

const task = new LargeFileUploadTask(client, fileObject, uploadSession);

try {
  const uploadResult = await task.upload();
  console.log('Upload complete');
} catch (error) {
  console.error('Upload failed, attempting to resume...');

  // Resume upload
  const resumeResult = await task.resume();
  console.log('Resumed and completed:', resumeResult);
}
```


### 5. Creating Folders

```typescript
// Create folder in root
const folder = await client
  .api('/me/drive/root/children')
  .post({
    name: 'New Folder',
    folder: {},
    '@microsoft.graph.conflictBehavior': 'rename'
  });

// Create folder at specific path
const folder = await client
  .api('/me/drive/root:/Documents/Projects:/children')
  .post({
    name: 'Project Alpha',
    folder: {},
    '@microsoft.graph.conflictBehavior': 'fail' // or 'replace', 'rename'
  });

// Create nested folder structure
const parentFolder = await client
  .api('/me/drive/root/children')
  .post({
    name: 'Parent',
    folder: {}
  });

const childFolder = await client
  .api(`/me/drive/items/${parentFolder.id}/children`)
  .post({
    name: 'Child',
    folder: {}
  });
```


### 6. Searching Files

```typescript
// Search in entire drive
const results = await client
  .api('/me/drive/root/search(q=\'{search-query}\')')
  .get();

// Example: Search for PDFs
const pdfFiles = await client
  .api('/me/drive/root/search(q=\'.pdf\')')
  .get();

// Search in specific folder
const results = await client
  .api('/me/drive/items/{folder-id}/search(q=\'quarterly report\')')
  .get();

// Search with select and filter
const results = await client
  .api('/me/drive/root/search(q=\'presentation\')')
  .select('id,name,size,webUrl')
  .filter('file ne null')
  .top(10)
  .get();
```


### 7. Updating/Renaming Files and Folders

```typescript
// Rename file
const updated = await client
  .api('/me/drive/items/{item-id}')
  .patch({
    name: 'new-name.pdf'
  });

// Update file metadata
const updated = await client
  .api('/me/drive/items/{item-id}')
  .patch({
    description: 'Updated description',
    name: 'renamed-file.docx'
  });

// Move file to different folder
const moved = await client
  .api('/me/drive/items/{item-id}')
  .patch({
    parentReference: {
      id: '{new-parent-folder-id}'
    }
  });

// Move and rename simultaneously
const updated = await client
  .api('/me/drive/items/{item-id}')
  .patch({
    name: 'new-name.xlsx',
    parentReference: {
      id: '{new-parent-folder-id}'
    }
  });
```


### 8. Copying Files

```typescript
// Copy file to another location
const copyOperation = await client
  .api('/me/drive/items/{item-id}/copy')
  .post({
    parentReference: {
      id: '{destination-folder-id}'
    },
    name: 'copied-file.pdf'
  });

// Monitor copy operation status
const monitorUrl = copyOperation.headers.get('Location');

// Poll for completion
async function waitForCopy(monitorUrl) {
  while (true) {
    const response = await fetch(monitorUrl);
    const status = await response.json();

    if (status.status === 'completed') {
      return status.resourceId;
    } else if (status.status === 'failed') {
      throw new Error('Copy failed: ' + status.error);
    }

    await new Promise(resolve => setTimeout(resolve, 1000));
  }
}
```


### 9. Deleting Files and Folders

```typescript
// Delete file or folder
await client
  .api('/me/drive/items/{item-id}')
  .delete();

// Delete by path
await client
  .api('/me/drive/root:/Documents/old-file.pdf')
  .delete();
```


### 10. Sharing and Permissions

#### Create Sharing Link

```typescript
// Create anonymous view link
const link = await client
  .api('/me/drive/items/{item-id}/createLink')
  .post({
    type: 'view', // 'view', 'edit', 'embed'
    scope: 'anonymous' // 'anonymous', 'organization'
  });

console.log('Share link:', link.link.webUrl);

// Create organization-wide edit link
const link = await client
  .api('/me/drive/items/{item-id}/createLink')
  .post({
    type: 'edit',
    scope: 'organization'
  });

// Create link with expiration
const link = await client
  .api('/me/drive/items/{item-id}/createLink')
  .post({
    type: 'view',
    scope: 'anonymous',
    expirationDateTime: '2025-12-31T23:59:59Z',
    password: 'SecurePass123' // Optional password protection
  });
```

#### Grant Permissions to Specific Users

```typescript
// Invite user with edit permissions
const permission = await client
  .api('/me/drive/items/{item-id}/invite')
  .post({
    requireSignIn: true,
    sendInvitation: true,
    roles: ['write'], // 'read' or 'write'
    recipients: [
      { email: 'user@example.com' }
    ],
    message: 'Here is the file I mentioned'
  });

// Add multiple recipients
const permission = await client
  .api('/me/drive/items/{item-id}/invite')
  .post({
    requireSignIn: false,
    sendInvitation: true,
    roles: ['read'],
    recipients: [
      { email: 'user1@example.com' },
      { email: 'user2@example.com' }
    ]
  });
```

#### List Permissions

```typescript
// Get all permissions for an item
const permissions = await client
  .api('/me/drive/items/{item-id}/permissions')
  .get();

permissions.value.forEach(permission => {
  console.log('Permission ID:', permission.id);
  console.log('Roles:', permission.roles);
  if (permission.grantedTo) {
    console.log('Granted to:', permission.grantedTo.user.displayName);
  }
  if (permission.link) {
    console.log('Share link:', permission.link.webUrl);
  }
});
```

#### Remove Permissions

```typescript
// Delete specific permission
await client
  .api('/me/drive/items/{item-id}/permissions/{permission-id}')
  .delete();
```

#### List Files Shared With Me

```typescript
// Get files shared with the current user
const sharedItems = await client
  .api('/me/drive/sharedWithMe')
  .get();

sharedItems.value.forEach(item => {
  console.log('Shared file:', item.name);
  console.log('Owner:', item.remoteItem.createdBy.user.displayName);
  console.log('Parent path:', item.remoteItem.parentReference.path);
});
```


### 11. Thumbnails

```typescript
// Get thumbnails for an item
const thumbnails = await client
  .api('/me/drive/items/{item-id}/thumbnails')
  .get();

// Access different sizes
const thumbs = thumbnails.value[0];
console.log('Small:', thumbs.small.url);
console.log('Medium:', thumbs.medium.url);
console.log('Large:', thumbs.large.url);

// Get specific thumbnail size
const thumbnail = await client
  .api('/me/drive/items/{item-id}/thumbnails/0/medium')
  .get();

console.log('Thumbnail URL:', thumbnail.url);
```


### 12. Delta (Change Tracking)

```typescript
// Get initial delta token and items
let deltaUrl = '/me/drive/root/delta';
let allItems = [];

while (deltaUrl) {
  const response = await client.api(deltaUrl).get();

  allItems = allItems.concat(response.value);

  if (response['@odata.nextLink']) {
    deltaUrl = response['@odata.nextLink'];
  } else {
    // Store delta token for next sync
    const deltaToken = response['@odata.deltaLink'];
    console.log('Delta token:', deltaToken);
    break;
  }
}

// Later, use the delta token to get only changes
const changes = await client
  .api('/me/drive/root/delta?token={previous-delta-token}')
  .get();

changes.value.forEach(item => {
  if (item.deleted) {
    console.log('Deleted:', item.id);
  } else {
    console.log('Added/Modified:', item.name);
  }
});
```


### 13. Special Folders

```typescript
// Access special folders
const documents = await client
  .api('/me/drive/special/documents')
  .get();

const photos = await client
  .api('/me/drive/special/photos')
  .get();

const cameraRoll = await client
  .api('/me/drive/special/cameraroll')
  .get();

const appRoot = await client
  .api('/me/drive/special/approot')
  .get();

// List children of special folder
const files = await client
  .api('/me/drive/special/documents/children')
  .get();
```


### 14. Working with SharePoint Document Libraries

```typescript
// Access SharePoint site drive
const drive = await client
  .api('/sites/{site-id}/drive')
  .get();

// List document library contents
const items = await client
  .api('/sites/{site-id}/drive/root/children')
  .get();

// Upload to SharePoint
const file = await client
  .api('/sites/{site-id}/drive/root:/folder/file.pdf:/content')
  .put(fileContent);

// Get site by path
const site = await client
  .api('/sites/{hostname}:/{server-relative-path}')
  .get();

// Example: Get site by URL
const site = await client
  .api('/sites/contoso.sharepoint.com:/sites/marketing')
  .get();
```


### 15. Batch Requests

```typescript
// Batch multiple requests
const batch = {
  requests: [
    {
      id: '1',
      method: 'GET',
      url: '/me/drive/root/children'
    },
    {
      id: '2',
      method: 'GET',
      url: '/me/drive/special/documents'
    },
    {
      id: '3',
      method: 'GET',
      url: '/me/drive/root/search(q=\'report\')'
    }
  ]
};

const batchResponse = await client
  .api('/$batch')
  .post(batch);

batchResponse.responses.forEach(response => {
  console.log(`Request ${response.id}:`, response.status);
  console.log('Body:', response.body);
});

// Batch with dependencies
const batchWithDeps = {
  requests: [
    {
      id: '1',
      method: 'POST',
      url: '/me/drive/root/children',
      body: {
        name: 'NewFolder',
        folder: {}
      },
      headers: {
        'Content-Type': 'application/json'
      }
    },
    {
      id: '2',
      dependsOn: ['1'],
      method: 'PUT',
      url: '/me/drive/items/{$1.id}:/file.txt:/content',
      body: 'File content',
      headers: {
        'Content-Type': 'text/plain'
      }
    }
  ]
};
```


## Error Handling

```typescript
import { GraphError } from '@microsoft/microsoft-graph-client';

try {
  const item = await client
    .api('/me/drive/items/{item-id}')
    .get();
} catch (error) {
  if (error instanceof GraphError) {
    console.error('Graph error code:', error.code);
    console.error('Status code:', error.statusCode);
    console.error('Message:', error.message);

    // Handle specific errors
    if (error.statusCode === 404) {
      console.error('Item not found');
    } else if (error.statusCode === 401) {
      console.error('Unauthorized - check authentication');
    } else if (error.statusCode === 403) {
      console.error('Forbidden - check permissions');
    } else if (error.statusCode === 429) {
      console.error('Too many requests - rate limited');
      const retryAfter = error.headers?.get('Retry-After');
      console.log(`Retry after ${retryAfter} seconds`);
    }
  } else {
    console.error('Unexpected error:', error);
  }
}
```


## Complete Working Examples


### Example 1: File Upload and Share Workflow

```typescript
import { Client } from '@microsoft/microsoft-graph-client';
import { ClientSecretCredential } from '@azure/identity';
import { TokenCredentialAuthenticationProvider } from '@microsoft/microsoft-graph-client/authProviders/azureTokenCredentials';
import fs from 'fs';
import 'isomorphic-fetch';

async function uploadAndShare() {
  // Initialize client
  const credential = new ClientSecretCredential(
    process.env.AZURE_TENANT_ID!,
    process.env.AZURE_CLIENT_ID!,
    process.env.AZURE_CLIENT_SECRET!
  );

  const authProvider = new TokenCredentialAuthenticationProvider(credential, {
    scopes: ['https://graph.microsoft.com/.default']
  });

  const client = Client.initWithMiddleware({ authProvider });

  const userId = process.env.USER_ID!;

  // Create folder
  const folder = await client
    .api(`/users/${userId}/drive/root/children`)
    .post({
      name: 'Shared Documents',
      folder: {},
      '@microsoft.graph.conflictBehavior': 'rename'
    });

  console.log('Created folder:', folder.id);

  // Upload file
  const fileContent = fs.readFileSync('./document.pdf');
  const uploadedFile = await client
    .api(`/users/${userId}/drive/items/${folder.id}:/document.pdf:/content`)
    .put(fileContent);

  console.log('Uploaded file:', uploadedFile.id);

  // Create share link
  const shareLink = await client
    .api(`/users/${userId}/drive/items/${uploadedFile.id}/createLink`)
    .post({
      type: 'view',
      scope: 'organization',
      expirationDateTime: '2025-12-31T23:59:59Z'
    });

  console.log('Share URL:', shareLink.link.webUrl);

  return shareLink.link.webUrl;
}
```


### Example 2: Download All Files from Folder

```typescript
import { Client } from '@microsoft/microsoft-graph-client';
import fs from 'fs';
import path from 'path';

async function downloadFolder(client, folderId, localPath) {
  // Create local directory
  if (!fs.existsSync(localPath)) {
    fs.mkdirSync(localPath, { recursive: true });
  }

  // Get folder contents
  const items = await client
    .api(`/me/drive/items/${folderId}/children`)
    .get();

  for (const item of items.value) {
    if (item.folder) {
      // Recursively download subfolder
      const subPath = path.join(localPath, item.name);
      await downloadFolder(client, item.id, subPath);
    } else if (item.file) {
      // Download file
      console.log(`Downloading ${item.name}...`);

      const stream = await client
        .api(`/me/drive/items/${item.id}/content`)
        .getStream();

      const filePath = path.join(localPath, item.name);
      const writeStream = fs.createWriteStream(filePath);
      stream.pipe(writeStream);

      await new Promise((resolve, reject) => {
        writeStream.on('finish', resolve);
        writeStream.on('error', reject);
      });

      console.log(`Downloaded to ${filePath}`);
    }
  }
}
```


### Example 3: Sync Local Folder to OneDrive

```typescript
import { Client } from '@microsoft/microsoft-graph-client';
import fs from 'fs';
import path from 'path';
import { LargeFileUploadTask, StreamUpload } from '@microsoft/microsoft-graph-client';

async function syncFolderToOneDrive(client, localPath, oneDrivePath) {
  const files = fs.readdirSync(localPath);

  for (const file of files) {
    const filePath = path.join(localPath, file);
    const stats = fs.statSync(filePath);

    if (stats.isDirectory()) {
      // Create folder in OneDrive
      const folder = await client
        .api(`/me/drive/root:/${oneDrivePath}:/children`)
        .post({
          name: file,
          folder: {},
          '@microsoft.graph.conflictBehavior': 'rename'
        });

      // Recursively sync subfolder
      await syncFolderToOneDrive(
        client,
        filePath,
        `${oneDrivePath}/${file}`
      );
    } else {
      // Upload file
      console.log(`Uploading ${file}...`);

      if (stats.size < 4 * 1024 * 1024) {
        // Small file - simple upload
        const content = fs.readFileSync(filePath);
        await client
          .api(`/me/drive/root:/${oneDrivePath}/${file}:/content`)
          .put(content);
      } else {
        // Large file - resumable upload
        const uploadSession = await client
          .api(`/me/drive/root:/${oneDrivePath}/${file}:/createUploadSession`)
          .post({
            item: {
              '@microsoft.graph.conflictBehavior': 'replace'
            }
          });

        const fileObject = new StreamUpload(
          fs.createReadStream(filePath),
          file,
          stats.size
        );

        const task = new LargeFileUploadTask(client, fileObject, uploadSession);
        await task.upload();
      }

      console.log(`Uploaded ${file}`);
    }
  }
}
```


### Example 4: Search and Download Files by Type

```typescript
async function downloadFilesByType(client, fileExtension, downloadPath) {
  if (!fs.existsSync(downloadPath)) {
    fs.mkdirSync(downloadPath, { recursive: true });
  }

  // Search for files
  const results = await client
    .api(`/me/drive/root/search(q='.${fileExtension}')`)
    .select('id,name,size,@microsoft.graph.downloadUrl')
    .get();

  console.log(`Found ${results.value.length} ${fileExtension} files`);

  for (const file of results.value) {
    if (file.file) {
      console.log(`Downloading ${file.name}...`);

      const stream = await client
        .api(`/me/drive/items/${file.id}/content`)
        .getStream();

      const filePath = path.join(downloadPath, file.name);
      const writeStream = fs.createWriteStream(filePath);
      stream.pipe(writeStream);

      await new Promise((resolve, reject) => {
        writeStream.on('finish', resolve);
        writeStream.on('error', reject);
      });
    }
  }

  console.log('Download complete');
}

// Usage
await downloadFilesByType(client, 'pdf', './downloads/pdfs');
```


## Rate Limiting and Throttling

Microsoft Graph implements throttling to maintain service health:

```typescript
async function makeRequestWithRetry(client, apiPath, maxRetries = 3) {
  let retries = 0;

  while (retries < maxRetries) {
    try {
      return await client.api(apiPath).get();
    } catch (error) {
      if (error.statusCode === 429) {
        const retryAfter = parseInt(error.headers?.get('Retry-After') || '5');
        console.log(`Rate limited. Waiting ${retryAfter} seconds...`);

        await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
        retries++;
      } else {
        throw error;
      }
    }
  }

  throw new Error('Max retries exceeded');
}
```


## Advanced Configuration

### Custom Middleware

```typescript
import { Middleware } from '@microsoft/microsoft-graph-client';

// Custom logging middleware
class LoggingMiddleware implements Middleware {
  async execute(context) {
    console.log(`Request: ${context.request}`);

    await this.nextMiddleware.execute(context);

    console.log(`Response: ${context.response.status}`);
  }
}

// Register middleware
const client = Client.initWithMiddleware({
  authProvider,
  middleware: new LoggingMiddleware()
});
```

### Custom Headers

```typescript
// Add custom headers to requests
const result = await client
  .api('/me/drive/root/children')
  .header('Prefer', 'HonorNonIndexedQueriesWarningMayFailRandomly')
  .header('ConsistencyLevel', 'eventual')
  .get();
```


## Working with Different Drive Types

```typescript
// Personal OneDrive
const personalDrive = await client.api('/me/drive').get();

// User's OneDrive (requires admin permissions)
const userDrive = await client.api('/users/{user-id}/drive').get();

// Group drive
const groupDrive = await client.api('/groups/{group-id}/drive').get();

// SharePoint site drive
const siteDrive = await client.api('/sites/{site-id}/drive').get();

// Specific drive by ID
const drive = await client.api('/drives/{drive-id}').get();
```


## Webhooks and Change Notifications

```typescript
// Subscribe to changes in a drive
const subscription = await client
  .api('/subscriptions')
  .post({
    changeType: 'updated',
    notificationUrl: 'https://your-webhook-endpoint.com/notifications',
    resource: '/me/drive/root',
    expirationDateTime: '2025-12-31T18:23:45.9356913Z',
    clientState: 'secretClientState'
  });

console.log('Subscription ID:', subscription.id);

// Renew subscription
const renewed = await client
  .api(`/subscriptions/${subscription.id}`)
  .patch({
    expirationDateTime: '2026-01-31T18:23:45.9356913Z'
  });

// Delete subscription
await client
  .api(`/subscriptions/${subscription.id}`)
  .delete();
```
