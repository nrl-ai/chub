---
name: identity
description: "Okta Python SDK coding guidelines for the Okta Management API using official libraries"
metadata:
  languages: "python"
  versions: "2.9.13"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "okta,identity,sso,oauth,authentication"
---

# Okta Python SDK Coding Guidelines

You are an Okta API coding expert. Help me with writing code using the Okta Management API calling the official libraries and SDKs.

You can find the official SDK documentation and code samples here:
https://developer.okta.com/docs/reference/api/users/
https://github.com/okta/okta-sdk-python

## Golden Rule: Use the Correct and Current SDK

Always use the official Okta Python SDK to interact with the Okta Management API. This is the standard library for all Okta Management API interactions. Do not construct manual HTTP requests or use unofficial libraries.

- **Library Name:** Okta Python SDK
- **PyPI Package:** `okta`
- **Current Version:** 2.9.13
- **Important:** This SDK is for the Management API only. For Authentication API, construct your own HTTP requests.

**Installation:**

- **Correct:** `pip install okta`

**APIs and Usage:**

- **Correct:** `from okta.client import Client as OktaClient`
- **Correct:** `client = OktaClient(config)`
- **Correct:** `await client.create_user(user_request)`
- **Correct:** `users, resp, err = await client.list_users()`
- **Incorrect:** Manual HTTP requests to Management API endpoints
- **Incorrect:** Using unofficial Okta Python packages

## System Requirements

The Okta Python SDK requires:
- Python 3.9 or higher
- An Okta organization URL
- An API token or OAuth 2.0 credentials

## Initialization and API Authentication

The Okta Python SDK uses asynchronous operations with `async`/`await` syntax throughout.

### API Token Authentication (Simple)

```python
from okta.client import Client as OktaClient
import asyncio
import os

config = {
    'orgUrl': os.getenv('OKTA_ORG_URL'),
    'token': os.getenv('OKTA_API_TOKEN')
}

client = OktaClient(config)
```

### OAuth 2.0 Private Key Authentication (Recommended for Service Apps)

When using OAuth 2.0 with private key authentication, you don't need an API token. The SDK automatically requests access tokens.

```python
from okta.client import Client as OktaClient
import os

config = {
    'orgUrl': os.getenv('OKTA_ORG_URL'),
    'authorizationMode': 'PrivateKey',
    'clientId': os.getenv('OKTA_CLIENT_ID'),
    'scopes': ['okta.users.manage', 'okta.groups.manage'],
    'privateKey': os.getenv('OKTA_PRIVATE_KEY'),  # JWK JSON string or PEM format
    'kid': os.getenv('OKTA_KEY_ID')  # Optional if kid is in JWK
}

client = OktaClient(config)
```

### Async Event Loop Pattern

All SDK operations are asynchronous and must be executed within an event loop:

```python
import asyncio
from okta.client import Client as OktaClient

async def main():
    config = {
        'orgUrl': 'https://dev-1234567.okta.com',
        'token': 'your_api_token'
    }
    client = OktaClient(config)

    users, resp, err = await client.list_users()
    print(f"Total users: {len(users)}")

# Run the async function
loop = asyncio.get_event_loop()
loop.run_until_complete(main())
```

### Response Pattern

All SDK methods return a tuple of `(result, response, error)`:

```python
users, resp, err = await client.list_users()

if err:
    print(f"Error occurred: {err}")
else:
    print(f"Retrieved {len(users)} users")
```

## User Management

### Create a User

Create a new user with profile information and password:

```python
from okta.client import Client as OktaClient
from okta.models import UserProfile, PasswordCredential, UserCredentials, CreateUserRequest
import asyncio

async def create_user():
    config = {
        'orgUrl': 'https://dev-1234567.okta.com',
        'token': 'your_api_token'
    }
    client = OktaClient(config)

    # Create user profile
    user_profile = UserProfile({
        'firstName': 'John',
        'lastName': 'Doe',
        'email': 'john.doe@example.com',
        'login': 'john.doe@example.com'
    })

    # Create password credential
    password_credential = PasswordCredential({
        'value': 'SecurePassword123!'
    })

    user_credentials = UserCredentials({
        'password': password_credential
    })

    # Create user request
    create_user_request = CreateUserRequest({
        'profile': user_profile,
        'credentials': user_credentials
    })

    # Create the user
    user, resp, err = await client.create_user(create_user_request)

    if err:
        print(f"Error creating user: {err}")
    else:
        print(f"Created user: {user.id}")
        return user

asyncio.run(create_user())
```

### Create User with Activation

Create and automatically activate a user:

```python
async def create_and_activate_user():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    user_profile = UserProfile({
        'firstName': 'Jane',
        'lastName': 'Smith',
        'email': 'jane.smith@example.com',
        'login': 'jane.smith@example.com',
        'mobilePhone': '555-123-4567'
    })

    password_credential = PasswordCredential({'value': 'SecurePassword123!'})
    user_credentials = UserCredentials({'password': password_credential})

    create_user_request = CreateUserRequest({
        'profile': user_profile,
        'credentials': user_credentials
    })

    # Activate on creation
    user, resp, err = await client.create_user(
        create_user_request,
        activate=True,
        provider=False
    )

    if err:
        print(f"Error: {err}")
    else:
        print(f"Created and activated user: {user.id}")
        return user

asyncio.run(create_and_activate_user())
```

### Get a User

Retrieve a user by ID or login:

```python
async def get_user(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # By user ID or login email
    user, resp, err = await client.get_user(user_id)

    if err:
        print(f"Error: {err}")
    else:
        print(f"User: {user.profile.first_name} {user.profile.last_name}")
        print(f"Email: {user.profile.email}")
        print(f"Status: {user.status}")
        return user

# Can use ID or email
asyncio.run(get_user('00u1234567890abcdef'))
asyncio.run(get_user('john.doe@example.com'))
```

### List All Users

Retrieve all users in your organization:

```python
async def list_all_users():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    users, resp, err = await client.list_users()

    if err:
        print(f"Error: {err}")
    else:
        print(f"Total users: {len(users)}")
        for user in users:
            print(f"  - {user.profile.first_name} {user.profile.last_name} ({user.profile.email})")
        return users

asyncio.run(list_all_users())
```

### List Users with Query Parameters

Filter users with query parameters:

```python
async def list_users_with_filters():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # Search by query string
    query_params = {'q': 'Robert'}
    users, resp, err = await client.list_users(query_params)

    if err:
        print(f"Error: {err}")
    else:
        print(f"Found {len(users)} users matching 'Robert'")
        for user in users:
            print(f"  - {user.profile.email}")

asyncio.run(list_users_with_filters())
```

### Search Users with SCIM Filter

Use SCIM expressions for precise filtering:

```python
async def search_users_scim():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # SCIM filter
    query_params = {
        'search': 'profile.nickName eq "bobby"'
    }
    users, resp, err = await client.list_users(query_params)

    if err:
        print(f"Error: {err}")
    else:
        for user in users:
            print(f"Found: {user.profile.email}")

asyncio.run(search_users_scim())
```

### Filter Users by Time

Find users updated after a specific time:

```python
async def find_recently_updated_users():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    query_params = {
        'filter': 'lastUpdated gt "2025-01-01T00:00:00.000Z"'
    }
    users, resp, err = await client.list_users(query_params)

    if err:
        print(f"Error: {err}")
    else:
        print(f"Found {len(users)} recently updated users")
        for user in users:
            print(f"  - {user.profile.email} (updated: {user.last_updated})")

asyncio.run(find_recently_updated_users())
```

### Update a User

Modify user profile information:

```python
async def update_user(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # Get the user first
    user, resp, err = await client.get_user(user_id)

    if err:
        print(f"Error getting user: {err}")
        return

    # Update profile fields
    user.profile.nick_name = 'Johnny'
    user.profile.mobile_phone = '+1-555-123-4567'
    user.profile.department = 'Engineering'

    # Save changes
    updated_user, resp, err = await client.update_user(user_id, user)

    if err:
        print(f"Error updating user: {err}")
    else:
        print(f"User updated successfully: {updated_user.id}")

asyncio.run(update_user('00u1234567890abcdef'))
```

### Partial Update a User

Update specific fields without retrieving the full user object:

```python
from okta.models import User

async def partial_update_user(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # Create a user object with only the fields to update
    user = User({
        'profile': {
            'nickName': 'JD',
            'department': 'Engineering'
        }
    })

    updated_user, resp, err = await client.update_user(user_id, user)

    if err:
        print(f"Error: {err}")
    else:
        print(f"User partially updated: {updated_user.id}")

asyncio.run(partial_update_user('00u1234567890abcdef'))
```

### Deactivate a User

Deactivate a user account:

```python
async def deactivate_user(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    user, resp, err = await client.deactivate_user(user_id)

    if err:
        print(f"Error deactivating user: {err}")
    else:
        print(f"User deactivated: {user.id}")

asyncio.run(deactivate_user('00u1234567890abcdef'))
```

### Delete a User

Permanently delete a user (must be deactivated first):

```python
async def delete_user(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # First deactivate
    _, resp, err = await client.deactivate_user(user_id)
    if err:
        print(f"Error deactivating: {err}")
        return

    # Then delete
    resp, err = await client.delete_user(user_id)

    if err:
        print(f"Error deleting user: {err}")
    else:
        print(f"User deleted permanently")

asyncio.run(delete_user('00u1234567890abcdef'))
```

### Reactivate a User

Reactivate a previously deactivated user:

```python
async def reactivate_user(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    query_params = {'sendEmail': False}
    user, resp, err = await client.activate_user(user_id, query_params)

    if err:
        print(f"Error reactivating user: {err}")
    else:
        print(f"User reactivated: {user.id}")

asyncio.run(reactivate_user('00u1234567890abcdef'))
```

### Suspend and Unsuspend User

Temporarily suspend a user:

```python
async def suspend_user(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    user, resp, err = await client.suspend_user(user_id)

    if err:
        print(f"Error suspending user: {err}")
    else:
        print(f"User suspended: {user.id}")

async def unsuspend_user(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    user, resp, err = await client.unsuspend_user(user_id)

    if err:
        print(f"Error unsuspending user: {err}")
    else:
        print(f"User unsuspended: {user.id}")

asyncio.run(suspend_user('00u1234567890abcdef'))
asyncio.run(unsuspend_user('00u1234567890abcdef'))
```

### Reset User Password

Send a password reset email:

```python
async def reset_user_password(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    query_params = {'sendEmail': True}
    reset_token, resp, err = await client.reset_password(user_id, query_params)

    if err:
        print(f"Error resetting password: {err}")
    else:
        print(f"Password reset email sent")

asyncio.run(reset_user_password('00u1234567890abcdef'))
```

### Expire User Password

Force a user to change password on next login:

```python
async def expire_user_password(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    user, resp, err = await client.expire_password(user_id)

    if err:
        print(f"Error expiring password: {err}")
    else:
        print(f"Password expired for user: {user.id}")

asyncio.run(expire_user_password('00u1234567890abcdef'))
```

## Group Management

### Create a Group

Create a new group:

```python
from okta.models import Group, GroupProfile

async def create_group():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    group_profile = GroupProfile({
        'name': 'Engineering Team',
        'description': 'All engineering department members'
    })

    new_group = Group({'profile': group_profile})

    group, resp, err = await client.create_group(new_group)

    if err:
        print(f"Error creating group: {err}")
    else:
        print(f"Created group: {group.id}")
        return group

asyncio.run(create_group())
```

### Get a Group

Retrieve a group by ID:

```python
async def get_group(group_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    group, resp, err = await client.get_group(group_id)

    if err:
        print(f"Error getting group: {err}")
    else:
        print(f"Group: {group.profile.name}")
        print(f"Description: {group.profile.description}")
        return group

asyncio.run(get_group('00g1234567890abcdef'))
```

### List All Groups

List all groups in the organization:

```python
async def list_all_groups():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    groups, resp, err = await client.list_groups()

    if err:
        print(f"Error listing groups: {err}")
    else:
        print(f"Total groups: {len(groups)}")
        for group in groups:
            print(f"  - {group.profile.name} ({group.id})")
        return groups

asyncio.run(list_all_groups())
```

### Search Groups by Name

Search for groups matching a query:

```python
async def search_groups(query):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    query_params = {'q': query}
    groups, resp, err = await client.list_groups(query_params)

    if err:
        print(f"Error searching groups: {err}")
    else:
        print(f"Found {len(groups)} groups matching '{query}'")
        for group in groups:
            print(f"  - {group.profile.name}")

asyncio.run(search_groups('Engineering'))
```

### Update a Group

Update group profile information:

```python
async def update_group(group_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # Get the group first
    group, resp, err = await client.get_group(group_id)

    if err:
        print(f"Error getting group: {err}")
        return

    # Update profile
    group.profile.description = 'Updated description for the team'

    # Save changes
    updated_group, resp, err = await client.update_group(group_id, group)

    if err:
        print(f"Error updating group: {err}")
    else:
        print(f"Group updated: {updated_group.id}")

asyncio.run(update_group('00g1234567890abcdef'))
```

### Delete a Group

Delete a group:

```python
async def delete_group(group_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    resp, err = await client.delete_group(group_id)

    if err:
        print(f"Error deleting group: {err}")
    else:
        print(f"Group deleted successfully")

asyncio.run(delete_group('00g1234567890abcdef'))
```

### Assign User to Group

Add a user to a group:

```python
async def assign_user_to_group(group_id, user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    resp, err = await client.add_user_to_group(group_id, user_id)

    if err:
        print(f"Error adding user to group: {err}")
    else:
        print(f"User {user_id} added to group {group_id}")

asyncio.run(assign_user_to_group('00g1234567890abcdef', '00u1234567890abcdef'))
```

### Remove User from Group

Remove a user from a group:

```python
async def remove_user_from_group(group_id, user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    resp, err = await client.remove_user_from_group(group_id, user_id)

    if err:
        print(f"Error removing user from group: {err}")
    else:
        print(f"User {user_id} removed from group {group_id}")

asyncio.run(remove_user_from_group('00g1234567890abcdef', '00u1234567890abcdef'))
```

### List Group Members

Get all users in a group:

```python
async def list_group_members(group_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    users, resp, err = await client.list_group_users(group_id)

    if err:
        print(f"Error listing group members: {err}")
    else:
        print(f"Group has {len(users)} members:")
        for user in users:
            print(f"  - {user.profile.first_name} {user.profile.last_name}")

asyncio.run(list_group_members('00g1234567890abcdef'))
```

### List User's Groups

Get all groups a user belongs to:

```python
async def list_user_groups(user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    groups, resp, err = await client.list_user_groups(user_id)

    if err:
        print(f"Error listing user groups: {err}")
    else:
        print(f"User belongs to {len(groups)} groups:")
        for group in groups:
            print(f"  - {group.profile.name}")

asyncio.run(list_user_groups('00u1234567890abcdef'))
```

## Application Management

### Create an Application

Create a basic authentication application:

```python
from okta.models import Application

async def create_application():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    application = Application({
        'name': 'template_basic_auth',
        'label': 'Sample Basic Auth App',
        'signOnMode': 'BASIC_AUTH',
        'settings': {
            'app': {
                'url': 'https://example.com/auth.htm',
                'authURL': 'https://example.com/login.html'
            }
        }
    })

    app, resp, err = await client.create_application(application)

    if err:
        print(f"Error creating application: {err}")
    else:
        print(f"Created application: {app.id}")
        return app

asyncio.run(create_application())
```

### Create an OAuth 2.0 Application

```python
async def create_oauth_application():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    application = Application({
        'name': 'oidc_client',
        'label': 'OAuth 2.0 App',
        'signOnMode': 'OPENID_CONNECT',
        'credentials': {
            'oauthClient': {
                'token_endpoint_auth_method': 'client_secret_post'
            }
        },
        'settings': {
            'oauthClient': {
                'client_uri': 'https://example.com',
                'logo_uri': 'https://example.com/logo.png',
                'redirect_uris': ['https://example.com/oauth/callback'],
                'response_types': ['code'],
                'grant_types': ['authorization_code', 'refresh_token'],
                'application_type': 'web'
            }
        }
    })

    app, resp, err = await client.create_application(application)

    if err:
        print(f"Error creating OAuth app: {err}")
    else:
        print(f"Created OAuth app: {app.id}")
        if hasattr(app.credentials, 'oauthClient'):
            print(f"Client ID: {app.credentials.oauthClient.client_id}")
            print(f"Client Secret: {app.credentials.oauthClient.client_secret}")
        return app

asyncio.run(create_oauth_application())
```

### Get an Application

Retrieve an application by ID:

```python
async def get_application(app_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    app, resp, err = await client.get_application(app_id)

    if err:
        print(f"Error getting application: {err}")
    else:
        print(f"Application: {app.label}")
        print(f"Sign-on mode: {app.sign_on_mode}")
        return app

asyncio.run(get_application('0oa1234567890abcdef'))
```

### List All Applications

```python
async def list_all_applications():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    apps, resp, err = await client.list_applications()

    if err:
        print(f"Error listing applications: {err}")
    else:
        print(f"Total applications: {len(apps)}")
        for app in apps:
            print(f"  - {app.label} ({app.id})")
        return apps

asyncio.run(list_all_applications())
```

### Update an Application

```python
async def update_application(app_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # Get the application first
    app, resp, err = await client.get_application(app_id)

    if err:
        print(f"Error getting application: {err}")
        return

    # Update label
    app.label = 'Updated Application Name'

    # Save changes
    updated_app, resp, err = await client.update_application(app_id, app)

    if err:
        print(f"Error updating application: {err}")
    else:
        print(f"Application updated: {updated_app.id}")

asyncio.run(update_application('0oa1234567890abcdef'))
```

### Delete an Application

```python
async def delete_application(app_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # First deactivate
    resp, err = await client.deactivate_application(app_id)
    if err:
        print(f"Error deactivating application: {err}")
        return

    # Then delete
    resp, err = await client.delete_application(app_id)

    if err:
        print(f"Error deleting application: {err}")
    else:
        print(f"Application deleted successfully")

asyncio.run(delete_application('0oa1234567890abcdef'))
```

### Assign User to Application

```python
from okta.models import AppUser

async def assign_user_to_application(app_id, user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    app_user = AppUser({'id': user_id})

    assigned_user, resp, err = await client.assign_user_to_application(app_id, app_user)

    if err:
        print(f"Error assigning user to application: {err}")
    else:
        print(f"User {user_id} assigned to application {app_id}")
        return assigned_user

asyncio.run(assign_user_to_application('0oa1234567890abcdef', '00u1234567890abcdef'))
```

### Assign User with Credentials to Application

```python
async def assign_user_with_credentials(app_id, user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    app_user = AppUser({
        'id': user_id,
        'credentials': {
            'userName': 'user@example.com',
            'password': {'value': 'AppSpecificPassword123!'}
        },
        'profile': {
            'role': 'Admin',
            'department': 'Engineering'
        }
    })

    assigned_user, resp, err = await client.assign_user_to_application(app_id, app_user)

    if err:
        print(f"Error: {err}")
    else:
        print(f"User assigned with credentials")

asyncio.run(assign_user_with_credentials('0oa1234567890abcdef', '00u1234567890abcdef'))
```

### Remove User from Application

```python
async def remove_user_from_application(app_id, user_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    resp, err = await client.delete_application_user(app_id, user_id)

    if err:
        print(f"Error removing user from application: {err}")
    else:
        print(f"User {user_id} removed from application {app_id}")

asyncio.run(remove_user_from_application('0oa1234567890abcdef', '00u1234567890abcdef'))
```

### Assign Group to Application

```python
from okta.models import ApplicationGroupAssignment

async def assign_group_to_application(app_id, group_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    assignment = ApplicationGroupAssignment({'priority': 0})

    result, resp, err = await client.create_application_group_assignment(
        app_id,
        group_id,
        assignment
    )

    if err:
        print(f"Error assigning group to application: {err}")
    else:
        print(f"Group {group_id} assigned to application {app_id}")
        return result

asyncio.run(assign_group_to_application('0oa1234567890abcdef', '00g1234567890abcdef'))
```

### Remove Group from Application

```python
async def remove_group_from_application(app_id, group_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    resp, err = await client.delete_application_group_assignment(app_id, group_id)

    if err:
        print(f"Error removing group from application: {err}")
    else:
        print(f"Group {group_id} removed from application {app_id}")

asyncio.run(remove_group_from_application('0oa1234567890abcdef', '00g1234567890abcdef'))
```

### List Application Users

Get all users assigned to an application:

```python
async def list_application_users(app_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    app_users, resp, err = await client.list_application_users(app_id)

    if err:
        print(f"Error listing application users: {err}")
    else:
        print(f"Application has {len(app_users)} users:")
        for app_user in app_users:
            print(f"  - User ID: {app_user.id}")

asyncio.run(list_application_users('0oa1234567890abcdef'))
```

### List Application Groups

Get all groups assigned to an application:

```python
async def list_application_groups(app_id):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    groups, resp, err = await client.list_application_group_assignments(app_id)

    if err:
        print(f"Error listing application groups: {err}")
    else:
        print(f"Application has {len(groups)} groups assigned:")
        for group in groups:
            print(f"  - Group ID: {group.id}")

asyncio.run(list_application_groups('0oa1234567890abcdef'))
```

## Custom Headers Management

Set custom headers sent with every request (requires SDK v1.3.0+):

```python
async def use_custom_headers():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # Set custom headers
    client.set_custom_headers({'Custom-Header': 'custom value', 'X-Request-ID': '12345'})

    # Make API calls with custom headers
    users, resp, err = await client.list_users()

    # Get current custom headers
    headers = client.get_custom_headers()
    print(f"Custom headers: {headers}")

    # Clear custom headers
    client.clear_custom_headers()
    print(f"Headers after clear: {client.get_custom_headers()}")

asyncio.run(use_custom_headers())
```

## Error Handling

Always check the error return value:

```python
async def handle_errors():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    user, resp, err = await client.get_user('nonexistent@example.com')

    if err:
        print(f"Error occurred: {err}")
        if hasattr(err, 'status'):
            if err.status == 404:
                print("User not found")
            elif err.status == 401:
                print("Authentication failed - check your API token")
            elif err.status == 403:
                print("Forbidden - insufficient permissions")
            elif err.status == 429:
                print("Rate limit exceeded")
    else:
        print(f"User found: {user.profile.email}")

asyncio.run(handle_errors())
```

### Comprehensive Error Handling Pattern

```python
async def create_user_with_error_handling(user_data):
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    try:
        user_profile = UserProfile(user_data['profile'])
        password_credential = PasswordCredential({'value': user_data['password']})
        user_credentials = UserCredentials({'password': password_credential})

        create_user_request = CreateUserRequest({
            'profile': user_profile,
            'credentials': user_credentials
        })

        user, resp, err = await client.create_user(create_user_request)

        if err:
            print(f"Error creating user: {err}")

            if hasattr(err, 'status'):
                if err.status == 400:
                    print("Bad request - check user data format")
                elif err.status == 401:
                    print("Authentication failed")
                elif err.status == 403:
                    print("Forbidden - insufficient permissions")
                elif err.status == 409:
                    print("Conflict - user already exists")
                elif err.status == 429:
                    print("Rate limit exceeded - retry after delay")
                elif err.status >= 500:
                    print("Server error - Okta service issue")

            return {'success': False, 'error': str(err)}
        else:
            print(f"User created successfully: {user.id}")
            return {'success': True, 'user': user}

    except Exception as e:
        print(f"Unexpected error: {e}")
        return {'success': False, 'error': str(e)}

user_data = {
    'profile': {
        'firstName': 'Alice',
        'lastName': 'Johnson',
        'email': 'alice.johnson@example.com',
        'login': 'alice.johnson@example.com'
    },
    'password': 'SecurePassword123!'
}

asyncio.run(create_user_with_error_handling(user_data))
```

## Pagination

Handle pagination for large result sets:

```python
async def paginate_users():
    config = {'orgUrl': 'https://dev-1234567.okta.com', 'token': 'your_api_token'}
    client = OktaClient(config)

    # Limit results per page
    query_params = {'limit': 20}

    users, resp, err = await client.list_users(query_params)

    if err:
        print(f"Error: {err}")
    else:
        print(f"Retrieved {len(users)} users")

        # Check if there are more pages
        if hasattr(resp, 'has_next') and resp.has_next():
            print("More pages available")

asyncio.run(paginate_users())
```

## Complete Example Application

```python
import asyncio
import os
from okta.client import Client as OktaClient
from okta.models import (
    UserProfile, PasswordCredential, UserCredentials, CreateUserRequest,
    Group, GroupProfile, Application
)

async def main():
    # Initialize client
    config = {
        'orgUrl': os.getenv('OKTA_ORG_URL'),
        'token': os.getenv('OKTA_API_TOKEN')
    }
    client = OktaClient(config)

    try:
        # Create a user
        print("Creating user...")
        user_profile = UserProfile({
            'firstName': 'Alice',
            'lastName': 'Johnson',
            'email': 'alice.johnson@example.com',
            'login': 'alice.johnson@example.com'
        })

        password_credential = PasswordCredential({'value': 'SecurePassword123!'})
        user_credentials = UserCredentials({'password': password_credential})

        create_user_request = CreateUserRequest({
            'profile': user_profile,
            'credentials': user_credentials
        })

        user, resp, err = await client.create_user(create_user_request, activate=True)
        if err:
            print(f"Error creating user: {err}")
            return
        print(f"Created user: {user.id}")

        # Create a group
        print("\nCreating group...")
        group_profile = GroupProfile({
            'name': 'Project Team Alpha',
            'description': 'Members of Project Alpha'
        })
        new_group = Group({'profile': group_profile})

        group, resp, err = await client.create_group(new_group)
        if err:
            print(f"Error creating group: {err}")
            return
        print(f"Created group: {group.id}")

        # Add user to group
        print("\nAdding user to group...")
        resp, err = await client.add_user_to_group(group.id, user.id)
        if err:
            print(f"Error adding user to group: {err}")
        else:
            print("User added to group successfully")

        # Create an application
        print("\nCreating application...")
        application = Application({
            'name': 'template_basic_auth',
            'label': 'Team Application',
            'signOnMode': 'BASIC_AUTH',
            'settings': {
                'app': {
                    'url': 'https://example.com/app',
                    'authURL': 'https://example.com/login'
                }
            }
        })

        app, resp, err = await client.create_application(application)
        if err:
            print(f"Error creating application: {err}")
            return
        print(f"Created application: {app.id}")

        # Assign group to application
        print("\nAssigning group to application...")
        from okta.models import ApplicationGroupAssignment
        assignment = ApplicationGroupAssignment({'priority': 0})

        result, resp, err = await client.create_application_group_assignment(
            app.id,
            group.id,
            assignment
        )
        if err:
            print(f"Error assigning group: {err}")
        else:
            print("Group assigned to application successfully")

        # List all users in the group
        print("\nGroup members:")
        members, resp, err = await client.list_group_users(group.id)
        if err:
            print(f"Error listing group members: {err}")
        else:
            for member in members:
                print(f"  - {member.profile.first_name} {member.profile.last_name}")

        print("\n✅ All operations completed successfully!")

    except Exception as e:
        print(f"Unexpected error: {e}")

if __name__ == '__main__':
    asyncio.run(main())
```

## Environment Variables Setup

Create a `.env` file for configuration:

```bash
OKTA_ORG_URL=https://dev-1234567.okta.com
OKTA_API_TOKEN=your_api_token_here
```

Load environment variables:

```python
import os
from dotenv import load_dotenv
from okta.client import Client as OktaClient

load_dotenv()

config = {
    'orgUrl': os.getenv('OKTA_ORG_URL'),
    'token': os.getenv('OKTA_API_TOKEN')
}

client = OktaClient(config)
```

## OAuth 2.0 Private Key Configuration

For OAuth 2.0 authentication, use environment variables:

```bash
OKTA_ORG_URL=https://dev-1234567.okta.com
OKTA_CLIENT_ID=your_client_id
OKTA_PRIVATE_KEY={"kty":"RSA","kid":"key-id","n":"...","e":"AQAB","d":"..."}
OKTA_KEY_ID=key-id
```

```python
import os
from dotenv import load_dotenv
from okta.client import Client as OktaClient

load_dotenv()

config = {
    'orgUrl': os.getenv('OKTA_ORG_URL'),
    'authorizationMode': 'PrivateKey',
    'clientId': os.getenv('OKTA_CLIENT_ID'),
    'scopes': ['okta.users.manage', 'okta.groups.manage', 'okta.apps.manage'],
    'privateKey': os.getenv('OKTA_PRIVATE_KEY'),
    'kid': os.getenv('OKTA_KEY_ID')
}

client = OktaClient(config)
```

## Useful Links

- Official Documentation: https://developer.okta.com/
- API Reference: https://developer.okta.com/docs/reference/
- Users API: https://developer.okta.com/docs/reference/api/users/
- Groups API: https://developer.okta.com/docs/reference/api/groups/
- Applications API: https://developer.okta.com/docs/reference/api/apps/
- GitHub Repository: https://github.com/okta/okta-sdk-python
- PyPI Package: https://pypi.org/project/okta/
