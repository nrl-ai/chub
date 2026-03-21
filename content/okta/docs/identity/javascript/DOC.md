---
name: identity
description: "Okta Node.js SDK coding guidelines for the Okta Management API using official libraries"
metadata:
  languages: "javascript"
  versions: "7.3.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "okta,identity,sso,oauth,authentication"
---

# Okta Node.js SDK Coding Guidelines

You are an Okta API coding expert. Help me with writing code using the Okta Management API calling the official libraries and SDKs.

You can find the official SDK documentation and code samples here:
https://developer.okta.com/docs/reference/api/users/
https://github.com/okta/okta-sdk-nodejs

## Golden Rule: Use the Correct and Current SDK

Always use the official Okta Node.js SDK to interact with the Okta Management API. This is the standard library for all Okta Management API interactions. Do not use deprecated packages or the authentication SDK for management tasks.

- **Library Name:** Okta Node.js SDK
- **NPM Package:** `@okta/okta-sdk-nodejs`
- **Current Version:** 7.3.0
- **Authentication SDK (Different Use Case):** `@okta/okta-auth-js` - Only for end-user authentication flows, NOT for management API

**Installation:**

- **Correct:** `npm install @okta/okta-sdk-nodejs`

**APIs and Usage:**

- **Correct:** `const okta = require('@okta/okta-sdk-nodejs')`
- **Correct:** `const client = new okta.Client({ orgUrl, token })`
- **Correct:** `await client.userApi.createUser({ body: newUser })`
- **Correct:** `await client.groupApi.createGroup({ group: newGroup })`
- **Correct:** `await client.applicationApi.createApplication({ application })`
- **Incorrect:** Using `@okta/okta-auth-js` for management operations
- **Incorrect:** Direct HTTP calls without SDK

## System Requirements

The Okta Node.js SDK requires:
- Node.js v12.0.0 or higher
- An Okta organization URL
- An API token or OAuth 2.0 credentials

## Initialization and API Authentication

The `@okta/okta-sdk-nodejs` library requires creating a `Client` instance for all API calls.

### API Token Authentication (Simple)

```javascript
const okta = require('@okta/okta-sdk-nodejs');

const client = new okta.Client({
  orgUrl: 'https://dev-1234.okta.com',
  token: process.env.OKTA_API_TOKEN
});
```

### OAuth 2.0 Private Key Authentication (Recommended for Service Apps)

When using OAuth 2.0 with private key authentication, you don't need an API token. The SDK automatically requests access tokens.

```javascript
const okta = require('@okta/okta-sdk-nodejs');

const client = new okta.Client({
  orgUrl: 'https://dev-1234.okta.com',
  authorizationMode: 'PrivateKey',
  clientId: process.env.OKTA_CLIENT_ID,
  scopes: ['okta.users.manage', 'okta.groups.manage'],
  privateKey: process.env.OKTA_PRIVATE_KEY, // JWK JSON string or PEM format
  keyId: process.env.OKTA_KEY_ID // Optional if kid is in JWK
});
```

### Private Key Formats

The `privateKey` parameter accepts:

- **JWK (JSON Web Key) as string:**
```javascript
privateKey: '{"kty":"RSA","kid":"my-key-id","n":"...","e":"AQAB","d":"..."}'
```

- **JWK as object:**
```javascript
privateKey: {
  kty: 'RSA',
  kid: 'my-key-id',
  n: '...',
  e: 'AQAB',
  d: '...'
}
```

- **PEM format:**
```javascript
privateKey: `-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC...
-----END PRIVATE KEY-----`
```

## User Management

### Create a User

Create a new user with profile information and password:

```javascript
const okta = require('@okta/okta-sdk-nodejs');
const client = new okta.Client({ orgUrl, token });

async function createUser() {
  const newUser = {
    profile: {
      firstName: 'John',
      lastName: 'Doe',
      email: 'john.doe@example.com',
      login: 'john.doe@example.com'
    },
    credentials: {
      password: {
        value: 'SecurePassword123!'
      }
    }
  };

  const user = await client.userApi.createUser({ body: newUser });
  console.log('Created user:', user.id);
  return user;
}
```

### Create User with Activation

Create and automatically activate a user:

```javascript
async function createAndActivateUser() {
  const newUser = {
    profile: {
      firstName: 'Jane',
      lastName: 'Smith',
      email: 'jane.smith@example.com',
      login: 'jane.smith@example.com'
    },
    credentials: {
      password: {
        value: 'SecurePassword123!'
      }
    }
  };

  const user = await client.userApi.createUser({
    body: newUser,
    activate: true
  });

  console.log('Created and activated user:', user.id);
  return user;
}
```

### Get a User

Retrieve a user by ID or login:

```javascript
async function getUser() {
  // By user ID
  let user = await client.userApi.getUser({
    userId: 'ausmvdt5xg8wRVI1d0g3'
  });

  // By login email
  user = await client.userApi.getUser({
    userId: 'john.doe@example.com'
  });

  console.log('User:', user.profile.firstName, user.profile.lastName);
  return user;
}
```

### List All Users

Iterate through all users in your organization:

```javascript
async function listAllUsers() {
  const collection = await client.userApi.listUsers();

  // Using .each() method
  await collection.each(user => {
    console.log(`${user.profile.firstName} ${user.profile.lastName} (${user.profile.email})`);
  });
}
```

### List Users with Async Iteration

```javascript
async function listUsersAsyncIteration() {
  const collection = await client.userApi.listUsers();

  // Using for...await
  for await (let user of collection) {
    console.log(`User ID: ${user.id}, Name: ${user.profile.firstName} ${user.profile.lastName}`);
  }
}
```

### Search Users by Query

Search users using simple query string:

```javascript
async function searchUsersByName() {
  const collection = await client.userApi.listUsers({
    q: 'Robert'
  });

  for await (let user of collection) {
    console.log('Found user:', user.profile.email);
  }
}
```

### Search Users with SCIM Filter

Use SCIM expressions for precise filtering:

```javascript
async function searchUsersWithFilter() {
  // SCIM filter for exact match
  const collection = await client.userApi.listUsers({
    search: 'profile.nickName eq "bobby"'
  });

  for await (let user of collection) {
    console.log('Found user:', user.profile.email);
  }
}
```

### Filter Users by Time

Find users updated after a specific time:

```javascript
async function findRecentlyUpdatedUsers() {
  const collection = await client.userApi.listUsers({
    filter: 'lastUpdated gt "2025-01-01T00:00:00.000Z"'
  });

  for await (let user of collection) {
    console.log('Recently updated:', user.profile.email, 'at', user.lastUpdated);
  }
}
```

### Update a User

Modify user profile information:

```javascript
async function updateUser(userId) {
  const user = await client.userApi.getUser({ userId });

  // Update profile fields
  user.profile.nickName = 'Johnny';
  user.profile.mobilePhone = '+1-555-123-4567';

  await client.userApi.updateUser({
    userId: user.id,
    user: user
  });

  console.log('User updated successfully');
}
```

### Partial Update a User

Update specific fields without retrieving the full user object:

```javascript
async function partialUpdateUser(userId) {
  const updates = {
    profile: {
      nickName: 'JD',
      department: 'Engineering'
    }
  };

  await client.userApi.updateUser({
    userId: userId,
    user: updates
  });

  console.log('User partially updated');
}
```

### Deactivate a User

Deactivate a user account:

```javascript
async function deactivateUser(userId) {
  await client.userApi.deactivateUser({ userId });
  console.log('User deactivated');
}
```

### Delete a User

Permanently delete a user (must be deactivated first):

```javascript
async function deleteUser(userId) {
  // First deactivate
  await client.userApi.deactivateUser({ userId });

  // Then delete
  await client.userApi.deleteUser({ userId });

  console.log('User deleted permanently');
}
```

### Reactivate a User

Reactivate a previously deactivated user:

```javascript
async function reactivateUser(userId) {
  await client.userApi.activateUser({
    userId: userId,
    sendEmail: false
  });

  console.log('User reactivated');
}
```

### Suspend and Unsuspend User

Temporarily suspend a user:

```javascript
async function suspendUser(userId) {
  await client.userApi.suspendUser({ userId });
  console.log('User suspended');
}

async function unsuspendUser(userId) {
  await client.userApi.unsuspendUser({ userId });
  console.log('User unsuspended');
}
```

### Reset User Password

Send a password reset email:

```javascript
async function resetUserPassword(userId) {
  await client.userApi.resetPassword({
    userId: userId,
    sendEmail: true
  });

  console.log('Password reset email sent');
}
```

### Expire User Password

Force a user to change password on next login:

```javascript
async function expireUserPassword(userId) {
  const user = await client.userApi.expirePassword({
    userId: userId
  });

  console.log('Password expired for user:', user.id);
}
```

## Group Management

### Create a Group

Create a new group:

```javascript
async function createGroup() {
  const newGroup = {
    profile: {
      name: 'Engineering Team',
      description: 'All engineering department members'
    }
  };

  const group = await client.groupApi.createGroup({ group: newGroup });
  console.log('Created group:', group.id);
  return group;
}
```

### Get a Group

Retrieve a group by ID:

```javascript
async function getGroup(groupId) {
  const group = await client.groupApi.getGroup({ groupId });
  console.log('Group:', group.profile.name);
  return group;
}
```

### List All Groups

List all groups in the organization:

```javascript
async function listAllGroups() {
  const collection = await client.groupApi.listGroups();

  for await (let group of collection) {
    console.log(`Group: ${group.profile.name} (${group.id})`);
  }
}
```

### Search Groups by Name

Search for groups matching a query:

```javascript
async function searchGroups() {
  const collection = await client.groupApi.listGroups({
    q: 'Engineering'
  });

  for await (let group of collection) {
    console.log('Found group:', group.profile.name);
  }
}
```

### Update a Group

Update group profile information:

```javascript
async function updateGroup(groupId) {
  const group = await client.groupApi.getGroup({ groupId });

  group.profile.description = 'Updated description';

  await client.groupApi.updateGroup({
    groupId: group.id,
    group: group
  });

  console.log('Group updated');
}
```

### Delete a Group

Delete a group:

```javascript
async function deleteGroup(groupId) {
  await client.groupApi.deleteGroup({ groupId });
  console.log('Group deleted');
}
```

### Assign User to Group

Add a user to a group:

```javascript
async function assignUserToGroup(groupId, userId) {
  await client.groupApi.assignUserToGroup({
    groupId: groupId,
    userId: userId
  });

  console.log(`User ${userId} added to group ${groupId}`);
}
```

### Remove User from Group

Remove a user from a group:

```javascript
async function removeUserFromGroup(groupId, userId) {
  await client.groupApi.unassignUserFromGroup({
    groupId: groupId,
    userId: userId
  });

  console.log(`User ${userId} removed from group ${groupId}`);
}
```

### List Group Members

Get all users in a group:

```javascript
async function listGroupMembers(groupId) {
  const collection = await client.groupApi.listGroupUsers({ groupId });

  for await (let user of collection) {
    console.log(`Member: ${user.profile.firstName} ${user.profile.lastName}`);
  }
}
```

### List User's Groups

Get all groups a user belongs to:

```javascript
async function listUserGroups(userId) {
  const collection = await client.userApi.listUserGroups({ userId });

  for await (let group of collection) {
    console.log(`User is in group: ${group.profile.name}`);
  }
}
```

## Application Management

### Create a Basic Auth Application

```javascript
async function createBasicAuthApp() {
  const application = {
    name: 'template_basic_auth',
    label: 'Sample Basic Auth App',
    signOnMode: 'BASIC_AUTH',
    settings: {
      app: {
        url: 'https://example.com/auth.htm',
        authURL: 'https://example.com/login.html'
      }
    }
  };

  const app = await client.applicationApi.createApplication({
    application: application
  });

  console.log('Created application:', app.id);
  return app;
}
```

### Create a SAML 2.0 Application

```javascript
async function createSAMLApp() {
  const application = {
    name: 'template_saml_2_0',
    label: 'SAML 2.0 App',
    signOnMode: 'SAML_2_0',
    settings: {
      signOn: {
        defaultRelayState: '',
        ssoAcsUrl: 'https://example.com/sso/saml',
        recipient: 'https://example.com/sso/saml',
        destination: 'https://example.com/sso/saml',
        audience: 'https://example.com',
        idpIssuer: 'http://www.okta.com/${org.externalKey}',
        subjectNameIdTemplate: '${user.userName}',
        subjectNameIdFormat: 'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress',
        responseSigned: true,
        assertionSigned: true,
        signatureAlgorithm: 'RSA_SHA256',
        digestAlgorithm: 'SHA256',
        honorForceAuthn: true,
        authnContextClassRef: 'urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport'
      }
    }
  };

  const app = await client.applicationApi.createApplication({
    application: application
  });

  console.log('Created SAML application:', app.id);
  return app;
}
```

### Create an OAuth 2.0 Application

```javascript
async function createOAuthApp() {
  const application = {
    name: 'oidc_client',
    label: 'OAuth 2.0 App',
    signOnMode: 'OPENID_CONNECT',
    credentials: {
      oauthClient: {
        token_endpoint_auth_method: 'client_secret_post'
      }
    },
    settings: {
      oauthClient: {
        client_uri: 'https://example.com',
        logo_uri: 'https://example.com/logo.png',
        redirect_uris: ['https://example.com/oauth/callback'],
        response_types: ['code'],
        grant_types: ['authorization_code', 'refresh_token'],
        application_type: 'web'
      }
    }
  };

  const app = await client.applicationApi.createApplication({
    application: application
  });

  console.log('Created OAuth app:', app.id);
  console.log('Client ID:', app.credentials.oauthClient.client_id);
  console.log('Client Secret:', app.credentials.oauthClient.client_secret);
  return app;
}
```

### Get an Application

Retrieve an application by ID:

```javascript
async function getApplication(appId) {
  const app = await client.applicationApi.getApplication({
    appId: appId
  });

  console.log('Application:', app.label);
  return app;
}
```

### List All Applications

```javascript
async function listAllApplications() {
  const collection = await client.applicationApi.listApplications();

  for await (let app of collection) {
    console.log(`App: ${app.label} (${app.id})`);
  }
}
```

### Update an Application

```javascript
async function updateApplication(appId) {
  const app = await client.applicationApi.getApplication({ appId });

  app.label = 'Updated Application Name';

  await client.applicationApi.updateApplication({
    appId: app.id,
    application: app
  });

  console.log('Application updated');
}
```

### Delete an Application

```javascript
async function deleteApplication(appId) {
  await client.applicationApi.deactivateApplication({ appId });
  await client.applicationApi.deleteApplication({ appId });
  console.log('Application deleted');
}
```

### Assign User to Application

```javascript
async function assignUserToApplication(appId, userId) {
  const appUser = await client.applicationApi.assignUserToApplication({
    appId: appId,
    appUser: {
      id: userId
    }
  });

  console.log('User assigned to application:', appUser.id);
  return appUser;
}
```

### Assign User with Profile to Application

```javascript
async function assignUserWithProfile(appId, userId) {
  const appUser = await client.applicationApi.assignUserToApplication({
    appId: appId,
    appUser: {
      id: userId,
      credentials: {
        userName: 'user@example.com',
        password: { value: 'AppSpecificPassword123!' }
      },
      profile: {
        role: 'Admin',
        department: 'Engineering'
      }
    }
  });

  console.log('User assigned with profile');
  return appUser;
}
```

### Remove User from Application

```javascript
async function removeUserFromApplication(appId, userId) {
  await client.applicationApi.unassignUserFromApplication({
    appId: appId,
    userId: userId
  });

  console.log('User removed from application');
}
```

### Assign Group to Application

```javascript
async function assignGroupToApplication(appId, groupId) {
  const assignment = await client.applicationApi.assignGroupToApplication({
    appId: appId,
    groupId: groupId,
    applicationGroupAssignment: {}
  });

  console.log('Group assigned to application');
  return assignment;
}
```

### Remove Group from Application

```javascript
async function removeGroupFromApplication(appId, groupId) {
  await client.applicationApi.unassignApplicationFromGroup({
    appId: appId,
    groupId: groupId
  });

  console.log('Group removed from application');
}
```

### List Application Users

Get all users assigned to an application:

```javascript
async function listApplicationUsers(appId) {
  const collection = await client.applicationApi.listApplicationUsers({
    appId: appId
  });

  for await (let appUser of collection) {
    console.log(`User ${appUser.id} assigned to app`);
  }
}
```

### List Application Groups

Get all groups assigned to an application:

```javascript
async function listApplicationGroups(appId) {
  const collection = await client.applicationApi.listApplicationGroupAssignments({
    appId: appId
  });

  for await (let assignment of collection) {
    console.log(`Group ${assignment.id} assigned to app`);
  }
}
```

## Authentication and Sessions

### List Active User Sessions

```javascript
async function listUserSessions(userId) {
  const collection = await client.userApi.listUserSessions({ userId });

  for await (let session of collection) {
    console.log(`Session ID: ${session.id}, Created: ${session.createdAt}`);
  }
}
```

### Clear User Sessions

Clear all sessions for a user:

```javascript
async function clearUserSessions(userId) {
  await client.userApi.clearUserSessions({ userId });
  console.log('All user sessions cleared');
}
```

## Error Handling

Always wrap API calls in try-catch blocks:

```javascript
async function handleErrors() {
  try {
    const user = await client.userApi.getUser({
      userId: 'nonexistent@example.com'
    });
  } catch (error) {
    if (error.status === 404) {
      console.error('User not found');
    } else if (error.status === 401) {
      console.error('Authentication failed - check your API token');
    } else if (error.status === 403) {
      console.error('Forbidden - insufficient permissions');
    } else if (error.status === 429) {
      console.error('Rate limit exceeded');
    } else {
      console.error('API Error:', error.message);
    }
  }
}
```

### Comprehensive Error Handling Pattern

```javascript
async function createUserWithErrorHandling(userData) {
  try {
    const newUser = {
      profile: {
        firstName: userData.firstName,
        lastName: userData.lastName,
        email: userData.email,
        login: userData.email
      },
      credentials: {
        password: {
          value: userData.password
        }
      }
    };

    const user = await client.userApi.createUser({ body: newUser });
    console.log('User created successfully:', user.id);
    return { success: true, user };

  } catch (error) {
    console.error('Error creating user:', error.message);

    if (error.status === 400) {
      console.error('Bad request - check user data format');
      if (error.body && error.body.errorCauses) {
        error.body.errorCauses.forEach(cause => {
          console.error('Error cause:', cause.errorSummary);
        });
      }
    } else if (error.status === 401) {
      console.error('Authentication failed');
    } else if (error.status === 403) {
      console.error('Forbidden - insufficient permissions');
    } else if (error.status === 409) {
      console.error('Conflict - user already exists');
    } else if (error.status === 429) {
      console.error('Rate limit exceeded - retry after delay');
    } else if (error.status >= 500) {
      console.error('Server error - Okta service issue');
    }

    return { success: false, error: error.message };
  }
}
```

## Pagination

Collections automatically handle pagination. Use `limit` to control page size:

```javascript
async function paginateUsers() {
  const collection = await client.userApi.listUsers({
    limit: 20 // 20 users per page
  });

  for await (let user of collection) {
    // SDK automatically fetches next pages
    console.log(user.profile.email);
  }
}
```

### Manual Pagination with After Parameter

```javascript
async function manualPagination() {
  let after;
  let pageCount = 0;

  do {
    const response = await client.userApi.listUsers({
      limit: 10,
      after: after
    });

    const users = response.users || [];
    console.log(`Page ${++pageCount}: ${users.length} users`);

    users.forEach(user => {
      console.log('  -', user.profile.email);
    });

    // Check if there's a next page
    after = response.nextPage ? response.nextPage.after : null;

  } while (after);
}
```

## Rate Limiting

The SDK automatically handles rate limiting with retry logic. Be aware of Okta rate limits:

- Most endpoints: 600 requests per minute
- Authentication endpoints: Varies by org tier
- Concurrent requests: Limited per org

```javascript
async function bulkOperationWithRateLimit() {
  const users = []; // Large array of users to create

  for (const userData of users) {
    try {
      const user = await client.userApi.createUser({ body: userData });
      console.log('Created user:', user.id);

      // Optional: Add delay between requests
      await new Promise(resolve => setTimeout(resolve, 100));

    } catch (error) {
      if (error.status === 429) {
        console.log('Rate limit hit, waiting before retry...');
        await new Promise(resolve => setTimeout(resolve, 5000));
        // Retry logic here
      }
    }
  }
}
```

## Working with Custom User Attributes

### Create User with Custom Attributes

```javascript
async function createUserWithCustomAttributes() {
  const newUser = {
    profile: {
      firstName: 'John',
      lastName: 'Doe',
      email: 'john.doe@example.com',
      login: 'john.doe@example.com',
      // Custom attributes (must be defined in user schema first)
      employeeId: 'EMP-12345',
      department: 'Engineering',
      costCenter: 'CC-100'
    },
    credentials: {
      password: {
        value: 'SecurePassword123!'
      }
    }
  };

  const user = await client.userApi.createUser({ body: newUser });
  console.log('Created user with custom attributes:', user.id);
}
```

## Complete Example Application

```javascript
const okta = require('@okta/okta-sdk-nodejs');

// Initialize client
const client = new okta.Client({
  orgUrl: process.env.OKTA_ORG_URL,
  token: process.env.OKTA_API_TOKEN
});

async function main() {
  try {
    // Create a user
    const newUser = {
      profile: {
        firstName: 'Alice',
        lastName: 'Johnson',
        email: 'alice.johnson@example.com',
        login: 'alice.johnson@example.com'
      },
      credentials: {
        password: { value: 'SecurePassword123!' }
      }
    };

    const user = await client.userApi.createUser({
      body: newUser,
      activate: true
    });
    console.log('Created user:', user.id);

    // Create a group
    const newGroup = {
      profile: {
        name: 'Project Team Alpha',
        description: 'Members of Project Alpha'
      }
    };

    const group = await client.groupApi.createGroup({ group: newGroup });
    console.log('Created group:', group.id);

    // Add user to group
    await client.groupApi.assignUserToGroup({
      groupId: group.id,
      userId: user.id
    });
    console.log('User added to group');

    // Create an application
    const application = {
      name: 'template_basic_auth',
      label: 'Team Application',
      signOnMode: 'BASIC_AUTH',
      settings: {
        app: {
          url: 'https://example.com/app',
          authURL: 'https://example.com/login'
        }
      }
    };

    const app = await client.applicationApi.createApplication({
      application: application
    });
    console.log('Created application:', app.id);

    // Assign group to application
    await client.applicationApi.assignGroupToApplication({
      appId: app.id,
      groupId: group.id,
      applicationGroupAssignment: {}
    });
    console.log('Group assigned to application');

    // List all users in the group
    console.log('\nGroup members:');
    const members = await client.groupApi.listGroupUsers({ groupId: group.id });
    for await (let member of members) {
      console.log(`  - ${member.profile.firstName} ${member.profile.lastName}`);
    }

  } catch (error) {
    console.error('Error:', error.message);
    if (error.status) {
      console.error('Status:', error.status);
    }
  }
}

main();
```

## Environment Variables Setup

Create a `.env` file for configuration:

```bash
OKTA_ORG_URL=https://dev-1234567.okta.com
OKTA_API_TOKEN=your_api_token_here
```

Load environment variables:

```javascript
require('dotenv').config();

const okta = require('@okta/okta-sdk-nodejs');

const client = new okta.Client({
  orgUrl: process.env.OKTA_ORG_URL,
  token: process.env.OKTA_API_TOKEN
});
```

## OAuth 2.0 Private Key Configuration

For OAuth 2.0 authentication, use environment variables:

```bash
OKTA_ORG_URL=https://dev-1234567.okta.com
OKTA_CLIENT_ID=your_client_id
OKTA_PRIVATE_KEY='{"kty":"RSA","kid":"key-id","n":"...","e":"AQAB","d":"..."}'
OKTA_KEY_ID=key-id
OKTA_SCOPES=okta.users.manage,okta.groups.manage,okta.apps.manage
```

```javascript
require('dotenv').config();
const okta = require('@okta/okta-sdk-nodejs');

const client = new okta.Client({
  orgUrl: process.env.OKTA_ORG_URL,
  authorizationMode: 'PrivateKey',
  clientId: process.env.OKTA_CLIENT_ID,
  scopes: process.env.OKTA_SCOPES.split(','),
  privateKey: process.env.OKTA_PRIVATE_KEY,
  keyId: process.env.OKTA_KEY_ID
});
```

## Useful Links

- Official Documentation: https://developer.okta.com/
- API Reference: https://developer.okta.com/docs/reference/
- Users API: https://developer.okta.com/docs/reference/api/users/
- Groups API: https://developer.okta.com/docs/reference/api/groups/
- Applications API: https://developer.okta.com/docs/reference/api/apps/
- GitHub Repository: https://github.com/okta/okta-sdk-nodejs
- NPM Package: https://www.npmjs.com/package/@okta/okta-sdk-nodejs
