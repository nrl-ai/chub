---
name: issues
description: "Jira Python SDK Coding Guidelines for writing code using the Jira API with official libraries and SDKs"
metadata:
  languages: "python"
  versions: "3.10.5"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "jira,issues,atlassian,project-management,agile"
---

# Jira Python SDK Coding Guidelines

You are a Jira API coding expert. Help me with writing code using the Jira API calling the official libraries and SDKs.

You can find the official Jira API documentation and code samples here:
https://jira.readthedocs.io/
https://developer.atlassian.com/cloud/jira/platform/rest/v3/

## Golden Rule: Use the Correct and Current SDK

Always use the jira Python library to interact with the Jira Cloud, Jira Server, and Jira Data Center REST APIs. This is the most actively maintained and comprehensive Python wrapper for Jira APIs.

- **Library Name:** jira
- **PyPI Package:** `jira`
- **Minimum Python Version:** 3.10 or newer
- **Legacy Libraries**: `jira-python`, `atlassian-python-api` (different library), and other unofficial packages may have different APIs

**Installation:**

- **Correct:** `pip install jira`
- **Correct:** `pip install jira[opt,cli,test]` (with optional dependencies)

**APIs and Usage:**

- **Correct:** `from jira import JIRA`
- **Correct:** `jira = JIRA('https://your-domain.atlassian.net', basic_auth=('email', 'token'))`
- **Correct:** `jira.search_issues('project = PROJ')`
- **Incorrect:** Using REST API endpoints directly without the SDK
- **Incorrect:** Using deprecated cookie-based authentication
- **Incorrect:** Using username/password for Jira Cloud (use API tokens instead)

## Authentication

The jira library supports multiple authentication methods. Choose the appropriate method based on your Jira instance type.

### API Token Authentication (Recommended for Jira Cloud)

Generate an API token at: https://id.atlassian.com/manage-profile/security/api-tokens

```python
from jira import JIRA

# Using basic_auth with email and API token
jira = JIRA(
    server='https://your-domain.atlassian.net',
    basic_auth=('your.email@example.com', 'your_api_token')
)
```

### Personal Access Token (PAT) Authentication (Self-Hosted Jira)

For Jira Server/Data Center instances:

```python
from jira import JIRA

# Using token_auth parameter
jira = JIRA(
    server='https://jira.yourcompany.com',
    token_auth='your_personal_access_token'
)

# Alternative: Using headers
headers = JIRA.DEFAULT_OPTIONS["headers"].copy()
headers["Authorization"] = f"Bearer {personal_access_token}"

jira = JIRA(
    server='https://jira.yourcompany.com',
    options={"headers": headers}
)
```

### OAuth Authentication

For OAuth integrations:

```python
from jira import JIRA

oauth_dict = {
    'access_token': 'your_access_token',
    'access_token_secret': 'your_access_token_secret',
    'consumer_key': 'your_consumer_key',
    'key_cert': key_cert_data  # RSA private key data
}

jira = JIRA(
    server='https://your-domain.atlassian.net',
    oauth=oauth_dict
)
```

### Kerberos Authentication

For enterprise environments with Kerberos:

```python
from jira import JIRA

# Basic Kerberos
jira = JIRA(server='https://jira.yourcompany.com', kerberos=True)

# With options
jira = JIRA(
    server='https://jira.yourcompany.com',
    kerberos=True,
    kerberos_options={'mutual_authentication': 'DISABLED'}
)
```

### Environment Variables Setup

Create a `.env` file or set environment variables:

```bash
JIRA_SERVER=https://your-domain.atlassian.net
JIRA_EMAIL=your.email@example.com
JIRA_API_TOKEN=your_api_token_here
```

Load environment variables in your code:

```python
import os
from jira import JIRA

jira = JIRA(
    server=os.environ['JIRA_SERVER'],
    basic_auth=(os.environ['JIRA_EMAIL'], os.environ['JIRA_API_TOKEN'])
)
```

## Initialization

Create a JIRA client instance for all API interactions:

```python
from jira import JIRA

# Basic initialization
jira = JIRA(
    server='https://your-domain.atlassian.net',
    basic_auth=('your.email@example.com', 'your_api_token')
)

# With custom options
options = {
    'server': 'https://your-domain.atlassian.net',
    'verify': True,  # SSL certificate verification
    'max_retries': 3,
    'timeout': 30,
}

jira = JIRA(
    options=options,
    basic_auth=('your.email@example.com', 'your_api_token')
)
```

## Error Handling

Handle exceptions for robust applications:

```python
from jira import JIRA, JIRAError

jira = JIRA(
    server='https://your-domain.atlassian.net',
    basic_auth=('your.email@example.com', 'your_api_token')
)

try:
    issue = jira.issue('PROJ-123')
    print(f'Issue: {issue.key} - {issue.fields.summary}')
except JIRAError as e:
    if e.status_code == 404:
        print('Issue not found')
    elif e.status_code == 401:
        print('Authentication failed')
    elif e.status_code == 403:
        print('Permission denied')
    else:
        print(f'Error: {e.status_code} - {e.text}')
except Exception as e:
    print(f'Unexpected error: {e}')
```

## Core API Surfaces

### Projects

#### Get Project

```python
# Minimal example
project = jira.project('PROJ')

print(project.key)
print(project.name)
print(project.lead.displayName)
```

#### Advanced: Get Project with All Details

```python
# Get project with all properties
project = jira.project('PROJ')

print('Project Key:', project.key)
print('Project Name:', project.name)
print('Project Lead:', project.lead.displayName)
print('Description:', project.raw.get('description', 'No description'))
print('Project Type:', project.projectTypeKey)
print('URL:', project.raw.get('url'))
```

#### Get All Projects

```python
# Get all projects
projects = jira.projects()

for project in projects:
    print(f'{project.key}: {project.name}')
```

#### Get Project Components

```python
# Get components for a project
components = jira.project_components('PROJ')

for component in components:
    print(f'{component.name}: {component.description}')
    if component.lead:
        print(f'  Lead: {component.lead.displayName}')
```

#### Get Project Versions

```python
# Get versions/releases for a project
versions = jira.project_versions('PROJ')

for version in versions:
    print(f'{version.name} - Released: {version.released}')
    if hasattr(version, 'releaseDate'):
        print(f'  Release Date: {version.releaseDate}')
```

### Issues

#### Get Issue

```python
# Minimal example
issue = jira.issue('PROJ-123')

print(issue.key)
print(issue.fields.summary)
print(issue.fields.status.name)
```

#### Advanced: Get Issue with Specific Fields

```python
# Get issue with expanded fields
issue = jira.issue(
    'PROJ-123',
    fields='summary,status,assignee,reporter,created,updated,priority,labels',
    expand='changelog,renderedFields'
)

print('Key:', issue.key)
print('Summary:', issue.fields.summary)
print('Status:', issue.fields.status.name)
print('Priority:', issue.fields.priority.name)
print('Assignee:', issue.fields.assignee.displayName if issue.fields.assignee else 'Unassigned')
print('Reporter:', issue.fields.reporter.displayName)
print('Created:', issue.fields.created)
print('Updated:', issue.fields.updated)
print('Labels:', issue.fields.labels)

# Access changelog
if hasattr(issue, 'changelog'):
    print('\nChange History:')
    for history in issue.changelog.histories:
        print(f'  {history.created} by {history.author.displayName}')
```

#### Create Issue

```python
# Minimal example
new_issue = jira.create_issue(
    project='PROJ',
    summary='New issue from Python',
    description='This is a test issue created via the API',
    issuetype={'name': 'Task'}
)

print(f'Created issue: {new_issue.key}')
```

#### Advanced: Create Issue with All Fields

```python
# Advanced example with multiple fields
issue_dict = {
    'project': {'key': 'PROJ'},
    'summary': 'Implement user authentication',
    'description': 'Need to implement OAuth 2.0 authentication for the API.\n\nAcceptance Criteria:\n- Support OAuth 2.0\n- Token refresh\n- Secure storage',
    'issuetype': {'name': 'Story'},
    'priority': {'name': 'High'},
    'labels': ['security', 'authentication', 'api'],
    'components': [{'name': 'Backend'}],
    'assignee': {'accountId': '5b10a2844c20165700ede21g'},
    'duedate': '2025-12-31',
    'customfield_10001': 'Custom field value',
}

new_issue = jira.create_issue(fields=issue_dict)

print(f'Created issue: {new_issue.key} - {new_issue.fields.summary}')
```

#### Bulk Create Issues

```python
# Create multiple issues at once
issue_list = [
    {
        'project': {'key': 'PROJ'},
        'summary': 'First task',
        'description': 'Description for first task',
        'issuetype': {'name': 'Task'},
    },
    {
        'project': {'key': 'PROJ'},
        'summary': 'Second task',
        'description': 'Description for second task',
        'issuetype': {'name': 'Task'},
        'priority': {'name': 'High'},
    },
]

issues = jira.create_issues(field_list=issue_list)

for issue in issues:
    print(f'Created: {issue["issue"].key}')
```

#### Update Issue

```python
# Update issue fields
issue = jira.issue('PROJ-123')

# Method 1: Using update method
issue.update(
    summary='Updated summary',
    description='Updated description',
    priority={'name': 'Critical'}
)

# Method 2: Using fields parameter
issue.update(fields={
    'labels': ['updated', 'priority'],
    'customfield_10001': 'New value'
})

# Method 3: Silent update (no notifications)
issue.update(
    notify=False,
    description='Silent update - no emails sent'
)

print('Issue updated successfully')
```

#### Assign Issue

```python
# Assign issue to a user
jira.assign_issue('PROJ-123', 'username')

# Or use accountId
jira.assign_issue('PROJ-123', '5b10a2844c20165700ede21g')

# Unassign issue
jira.assign_issue('PROJ-123', None)

print('Issue assigned successfully')
```

#### Delete Issue

```python
# Delete an issue
issue = jira.issue('PROJ-123')
issue.delete()

# Delete with subtasks
issue.delete(deleteSubtasks=True)

print('Issue deleted successfully')
```

### Issue Search

#### Basic JQL Search

```python
# Search for issues in a project
issues = jira.search_issues('project = PROJ')

for issue in issues:
    print(f'{issue.key}: {issue.fields.summary}')
```

#### Advanced: Search with Pagination and Options

```python
# Advanced search with pagination and field selection
issues = jira.search_issues(
    jql_str='project = PROJ AND status = "In Progress" ORDER BY created DESC',
    startAt=0,
    maxResults=50,
    fields='summary,status,assignee,priority,created',
    expand='changelog'
)

print(f'Total issues: {issues.total}')
print(f'Returned: {len(issues)}')

for issue in issues:
    print(f'{issue.key}: {issue.fields.summary}')
    print(f'  Status: {issue.fields.status.name}')
    print(f'  Priority: {issue.fields.priority.name}')
    print(f'  Created: {issue.fields.created}')
```

#### Pagination Example

```python
# Fetch all issues with pagination
def get_all_issues(jira, jql):
    """Fetch all issues matching JQL query with pagination"""
    all_issues = []
    start_at = 0
    max_results = 100

    while True:
        issues = jira.search_issues(
            jql_str=jql,
            startAt=start_at,
            maxResults=max_results
        )

        all_issues.extend(issues)

        print(f'Fetched {len(all_issues)} of {issues.total} issues')

        if len(issues) < max_results:
            break

        start_at += max_results

    return all_issues

# Usage
all_project_issues = get_all_issues(jira, 'project = PROJ')
print(f'Total issues fetched: {len(all_project_issues)}')
```

#### Complex JQL Queries

```python
# Complex JQL with multiple conditions
jql = """
    project = PROJ AND
    status IN ("To Do", "In Progress") AND
    priority IN (High, Critical) AND
    assignee = currentUser() AND
    created >= -30d
    ORDER BY priority DESC, created ASC
"""

issues = jira.search_issues(jql.strip(), maxResults=100)

print(f'High priority issues: {len(issues)}')
for issue in issues:
    print(f'{issue.key}: {issue.fields.summary} ({issue.fields.priority.name})')
```

#### Common JQL Patterns

```python
# Issues assigned to current user
issues = jira.search_issues('assignee = currentUser()')

# Issues created in last 7 days
issues = jira.search_issues('created >= -7d')

# High priority unresolved bugs
issues = jira.search_issues('type = Bug AND priority = High AND status != Resolved')

# Issues in specific sprint
issues = jira.search_issues('sprint = 123')

# Issues with specific labels
issues = jira.search_issues('labels IN (urgent, critical)')

# Issues due this week
issues = jira.search_issues('duedate >= startOfWeek() AND duedate <= endOfWeek()')

# Combining multiple conditions
issues = jira.search_issues(
    'project = PROJ AND issuetype in (Story, Task) AND status = Open',
    maxResults=50
)
```

### Comments

#### Get Comments

```python
# Get all comments for an issue
issue = jira.issue('PROJ-123')
comments = issue.fields.comment.comments

for comment in comments:
    print(f'{comment.author.displayName} at {comment.created}:')
    print(f'  {comment.body}')
```

#### Get Specific Comment

```python
# Get a specific comment by ID
comment = jira.comment('PROJ-123', '10234')

print(f'Author: {comment.author.displayName}')
print(f'Created: {comment.created}')
print(f'Body: {comment.body}')
```

#### Add Comment

```python
# Add a simple comment
comment = jira.add_comment('PROJ-123', 'This is a new comment from the API')

print(f'Comment added: {comment.id}')
```

#### Advanced: Add Comment with Visibility

```python
# Add comment visible only to specific role
comment = jira.add_comment(
    'PROJ-123',
    'This is visible only to administrators',
    visibility={'type': 'role', 'value': 'Administrators'}
)

# Add comment visible to specific group
comment = jira.add_comment(
    'PROJ-123',
    'This is visible only to developers',
    visibility={'type': 'group', 'value': 'jira-developers'}
)

print(f'Restricted comment added: {comment.id}')
```

#### Add Comment with Rich Text (ADF Format)

```python
# Add comment using Atlassian Document Format (ADF)
comment_adf = {
    "type": "doc",
    "version": 1,
    "content": [
        {
            "type": "paragraph",
            "content": [
                {
                    "type": "text",
                    "text": "This is a comment with "
                },
                {
                    "type": "text",
                    "text": "bold text",
                    "marks": [{"type": "strong"}]
                }
            ]
        },
        {
            "type": "codeBlock",
            "content": [
                {
                    "type": "text",
                    "text": "print('Hello, World!')"
                }
            ]
        }
    ]
}

comment = jira.add_comment('PROJ-123', comment_adf)
print(f'Rich text comment added: {comment.id}')
```

#### Update Comment

```python
# Update an existing comment
comment = jira.comment('PROJ-123', '10234')

comment.update(body='Updated comment text')

# Silent update (no notifications)
comment.update(body='Quiet update', notify=False)

print('Comment updated successfully')
```

#### Delete Comment

```python
# Delete a comment
comment = jira.comment('PROJ-123', '10234')
comment.delete()

print('Comment deleted successfully')
```

### Transitions

#### Get Available Transitions

```python
# Get all available transitions for an issue
transitions = jira.transitions('PROJ-123')

print('Available transitions:')
for transition in transitions:
    print(f"  {transition['id']}: {transition['name']} -> {transition['to']['name']}")
```

#### Transition Issue

```python
# Transition issue to a new status
jira.transition_issue('PROJ-123', '21')  # Use transition ID

print('Issue transitioned successfully')
```

#### Advanced: Transition with Fields

```python
# Transition issue and set fields
jira.transition_issue(
    'PROJ-123',
    '5',  # Transition ID for "Resolve Issue"
    assignee={'name': 'pm_user'},
    resolution={'id': '3'},  # Fixed
    comment='Issue resolved and tested'
)

# Alternative using fields parameter
jira.transition_issue(
    'PROJ-123',
    '5',
    fields={
        'assignee': {'name': 'qa_user'},
        'resolution': {'name': 'Done'},
        'customfield_10001': 'custom value'
    }
)

print('Issue transitioned with fields updated')
```

### Attachments

#### Add Attachment

```python
# Add attachment from file path
jira.add_attachment(
    issue='PROJ-123',
    attachment='/path/to/file.pdf'
)

print('Attachment added successfully')
```

#### Advanced: Add Multiple Attachments

```python
# Add multiple attachments
jira.add_attachment(
    issue='PROJ-123',
    attachment='/path/to/document.pdf',
    filename='report.pdf'
)

# Add attachment from file object
with open('/path/to/file.txt', 'rb') as f:
    jira.add_attachment(issue='PROJ-123', attachment=f, filename='data.txt')

# Add multiple files
attachments = ['/path/to/file1.txt', '/path/to/file2.pdf']
for attachment_path in attachments:
    jira.add_attachment(issue='PROJ-123', attachment=attachment_path)

print('Multiple attachments added successfully')
```

#### Get Attachments

```python
# Get all attachments for an issue
issue = jira.issue('PROJ-123')

for attachment in issue.fields.attachment:
    print(f'Filename: {attachment.filename}')
    print(f'Size: {attachment.size} bytes')
    print(f'MIME Type: {attachment.mimeType}')
    print(f'Created: {attachment.created}')
    print(f'Download URL: {attachment.content}')
    print()
```

#### Download Attachment

```python
# Download attachment content
issue = jira.issue('PROJ-123')

for attachment in issue.fields.attachment:
    # Get attachment content
    content = attachment.get()

    # Save to file
    with open(attachment.filename, 'wb') as f:
        f.write(content)

    print(f'Downloaded: {attachment.filename}')
```

#### Delete Attachment

```python
# Delete an attachment by ID
jira.delete_attachment('10001')

print('Attachment deleted successfully')
```

### Agile - Boards

#### Get All Boards

```python
# Get all boards
boards = jira.boards()

for board in boards:
    print(f'{board.id}: {board.name} ({board.type})')
```

#### Advanced: Get Boards with Filters

```python
# Get boards with specific parameters
boards = jira.boards(
    startAt=0,
    maxResults=50,
    type='scrum',  # or 'kanban'
    name='Development',
    projectKeyOrID='PROJ'
)

for board in boards:
    print(f'{board.id}: {board.name}')
    print(f'  Type: {board.type}')
    if hasattr(board, 'location'):
        print(f'  Location: {board.location.name}')
```

#### Get Specific Board

```python
# Get a specific board by ID
board = jira.board(1)

print(f'Board: {board.name}')
print(f'Type: {board.type}')
print(f'Location: {board.location.name}')
```

#### Get Issues on Board

```python
# Get all issues on a board
board_id = 1
issues = jira.search_issues(f'board = {board_id}')

for issue in issues:
    print(f'{issue.key}: {issue.fields.summary}')
```

### Agile - Sprints

#### Get Sprints for Board

```python
# Get all sprints for a board
board_id = 1
sprints = jira.sprints(board_id)

for sprint in sprints:
    print(f'{sprint.id}: {sprint.name}')
    print(f'  State: {sprint.state}')
    if hasattr(sprint, 'startDate'):
        print(f'  Start: {sprint.startDate}')
    if hasattr(sprint, 'endDate'):
        print(f'  End: {sprint.endDate}')
```

#### Advanced: Get Sprints with Filters

```python
# Get sprints with specific state
board_id = 1
sprints = jira.sprints(
    board_id,
    startAt=0,
    maxResults=50,
    state='active'  # 'active', 'closed', 'future'
)

for sprint in sprints:
    print(f'{sprint.name} - {sprint.state}')
```

#### Get Sprint

```python
# Get a specific sprint by ID
sprint = jira.sprint(123)

print(f'Sprint: {sprint.name}')
print(f'State: {sprint.state}')
print(f'Start Date: {sprint.startDate}')
print(f'End Date: {sprint.endDate}')
print(f'Goal: {sprint.goal}')
```

#### Get Issues in Sprint

```python
# Get all issues in a sprint
sprint_id = 123
issues = jira.search_issues(f'sprint = {sprint_id}')

print(f'Issues in sprint: {len(issues)}')
for issue in issues:
    print(f'{issue.key}: {issue.fields.summary}')
    print(f'  Status: {issue.fields.status.name}')
```

#### Advanced: Get Sprint Issues with JQL

```python
# Get incomplete issues in active sprint
sprint_id = 123
jql = f'sprint = {sprint_id} AND status != Done'

issues = jira.search_issues(jql, maxResults=100)

print(f'Incomplete issues: {len(issues)}')
for issue in issues:
    print(f'{issue.key}: {issue.fields.summary} ({issue.fields.status.name})')
```

### Users

#### Get Current User

```python
# Get currently authenticated user
current_user = jira.current_user()

print(f'Username: {current_user}')
```

#### Search for Users

```python
# Search for users by username or email
users = jira.search_users(user='john')

for user in users:
    print(f'{user.displayName} ({user.emailAddress})')
    print(f'  Account ID: {user.accountId}')
    print(f'  Active: {user.active}')
```

#### Advanced: Search Users with Filters

```python
# Search for users in a project
users = jira.search_users(
    user='john',
    startAt=0,
    maxResults=50,
    includeActive=True,
    includeInactive=False
)

for user in users:
    print(f'{user.displayName} - {user.emailAddress}')
```

#### Search Assignable Users

```python
# Search for users who can be assigned to issues in a project
users = jira.search_assignable_users_for_projects(
    username='',  # Empty string to get all
    projectKeys='PROJ',
    startAt=0,
    maxResults=50
)

for user in users:
    print(f'{user.displayName} can be assigned to PROJ')
```

### Watchers

#### Get Watchers

```python
# Get watchers for an issue
issue = jira.issue('PROJ-123')
watchers = jira.watchers(issue)

print(f'Watch Count: {watchers.watchCount}')
print('Watchers:')
for watcher in watchers.watchers:
    print(f'  {watcher.displayName}')
```

#### Add Watcher

```python
# Add a watcher to an issue
jira.add_watcher('PROJ-123', 'username')

# Or use accountId
jira.add_watcher('PROJ-123', '5b10a2844c20165700ede21g')

print('Watcher added successfully')
```

#### Remove Watcher

```python
# Remove a watcher from an issue
jira.remove_watcher('PROJ-123', 'username')

print('Watcher removed successfully')
```

### Labels

#### Get Issue Labels

```python
# Get labels for an issue
issue = jira.issue('PROJ-123')
labels = issue.fields.labels

print('Labels:', labels)
```

#### Update Labels

```python
# Add labels to an issue
issue = jira.issue('PROJ-123')
issue.update(fields={'labels': ['backend', 'api', 'urgent']})

# Append labels (preserve existing)
existing_labels = issue.fields.labels
new_labels = existing_labels + ['new-label']
issue.update(fields={'labels': new_labels})

print('Labels updated successfully')
```

### Filters

#### Get Favorite Filters

```python
# Get all favorite filters for current user
filters = jira.favourite_filters()

for f in filters:
    print(f'{f.id}: {f.name}')
    print(f'  JQL: {f.jql}')
    print(f'  Owner: {f.owner.displayName}')
```

#### Get Filter

```python
# Get a specific filter by ID
filter_obj = jira.filter('10000')

print(f'Filter: {filter_obj.name}')
print(f'JQL: {filter_obj.jql}')
print(f'Owner: {filter_obj.owner.displayName}')
```

#### Search Using Filter

```python
# Execute a saved filter
filter_id = 10000
jql = f'filter = {filter_id}'
issues = jira.search_issues(jql, maxResults=100)

print(f'Filter results: {len(issues)} issues')
```

### Versions (Releases)

#### Create Version

```python
# Create a new version/release
version = jira.create_version(
    name='v1.2.0',
    project='PROJ',
    description='Q4 2025 Release',
    releaseDate='2025-12-15',
    archived=False,
    released=False
)

print(f'Created version: {version.name} (ID: {version.id})')
```

#### Get Version

```python
# Get a specific version
version = jira.version('10001')

print(f'Version: {version.name}')
print(f'Released: {version.released}')
if hasattr(version, 'releaseDate'):
    print(f'Release Date: {version.releaseDate}')
```

#### Update Version

```python
# Update a version
version = jira.version('10001')
version.update(
    name='v1.2.1',
    description='Updated release',
    released=True,
    releaseDate='2025-12-20'
)

print('Version updated successfully')
```

#### Delete Version

```python
# Delete a version
version = jira.version('10001')
version.delete()

print('Version deleted successfully')
```

### Components

#### Create Component

```python
# Create a new component
component = jira.create_component(
    name='Frontend',
    project='PROJ',
    description='Frontend components and pages',
    leadAccountId='5b10a2844c20165700ede21g'
)

print(f'Created component: {component.name} (ID: {component.id})')
```

#### Get Component

```python
# Get a specific component
component = jira.component('10000')

print(f'Component: {component.name}')
print(f'Description: {component.description}')
if component.lead:
    print(f'Lead: {component.lead.displayName}')
```

#### Update Component

```python
# Update a component
component = jira.component('10000')
component.update(
    name='Updated Frontend',
    description='Updated description'
)

print('Component updated successfully')
```

### Workflows

#### Get Workflows

```python
# Get all workflows (requires project admin permissions)
workflows = jira.workflows()

for workflow in workflows:
    print(f'{workflow.name}')
```

### Custom Fields

#### Get All Fields

```python
# Get all fields in Jira
fields = jira.fields()

print('Custom Fields:')
for field in fields:
    if field['custom']:
        print(f"{field['id']}: {field['name']}")
```

#### Get Custom Field Value

```python
# Get custom field value from an issue
issue = jira.issue('PROJ-123')
custom_field_id = 'customfield_10001'

if hasattr(issue.fields, custom_field_id):
    value = getattr(issue.fields, custom_field_id)
    print(f'Custom field value: {value}')
```

#### Set Custom Field Value

```python
# Set custom field value
issue = jira.issue('PROJ-123')
issue.update(fields={'customfield_10001': 'New custom value'})

print('Custom field updated successfully')
```

### Permissions

#### Get My Permissions

```python
# Get permissions for current user
permissions = jira.my_permissions(projectKey='PROJ')

print('Permissions for project PROJ:')
for key, permission in permissions['permissions'].items():
    if permission['havePermission']:
        print(f'  {key}: Granted')
```

#### Check Specific Permission

```python
# Check if user has specific permission
permissions = jira.my_permissions(
    projectKey='PROJ',
    permissions='CREATE_ISSUES,EDIT_ISSUES'
)

can_create = permissions['permissions']['CREATE_ISSUES']['havePermission']
can_edit = permissions['permissions']['EDIT_ISSUES']['havePermission']

print(f'Can create issues: {can_create}')
print(f'Can edit issues: {can_edit}')
```

## Common Patterns

### Batch Operations

```python
# Create multiple issues efficiently
def create_multiple_issues(jira, issues_data):
    """Create multiple issues using bulk operation"""
    return jira.create_issues(field_list=issues_data)

# Usage
issues_data = [
    {
        'project': {'key': 'PROJ'},
        'summary': f'Task {i}',
        'issuetype': {'name': 'Task'},
    }
    for i in range(1, 11)
]

created_issues = create_multiple_issues(jira, issues_data)

for result in created_issues:
    if 'issue' in result:
        print(f"Created: {result['issue'].key}")
    else:
        print(f"Error: {result['error']}")
```

### Rate Limiting

```python
import time

def rate_limited_operation(items, operation, rate_limit=10, delay=1):
    """Execute operations with rate limiting"""
    results = []

    for i, item in enumerate(items):
        result = operation(item)
        results.append(result)

        # Add delay every rate_limit operations
        if (i + 1) % rate_limit == 0 and i < len(items) - 1:
            print(f'Processed {i + 1} items, waiting {delay}s...')
            time.sleep(delay)

    return results

# Usage
def update_issue(issue_key):
    issue = jira.issue(issue_key)
    issue.update(fields={'labels': ['batch-updated']})
    return issue_key

issue_keys = ['PROJ-1', 'PROJ-2', 'PROJ-3', 'PROJ-4', 'PROJ-5']
results = rate_limited_operation(issue_keys, update_issue, rate_limit=2, delay=1)
```

### Retry Logic

```python
import time
from jira import JIRAError

def with_retry(operation, max_retries=3, delay=1):
    """Execute operation with retry logic"""
    for attempt in range(1, max_retries + 1):
        try:
            return operation()
        except JIRAError as e:
            if attempt == max_retries:
                raise

            print(f'Attempt {attempt} failed: {e.text}')
            print(f'Retrying in {delay}s...')
            time.sleep(delay)
            delay *= 2  # Exponential backoff

# Usage
issue = with_retry(lambda: jira.issue('PROJ-123'))
print(f'Retrieved: {issue.key}')
```

### Context Manager for Connection

```python
from contextlib import contextmanager

@contextmanager
def jira_connection(server, email, api_token):
    """Context manager for Jira connection"""
    jira = JIRA(
        server=server,
        basic_auth=(email, api_token)
    )
    try:
        yield jira
    finally:
        jira.close()

# Usage
with jira_connection(
    'https://your-domain.atlassian.net',
    'your@email.com',
    'your_token'
) as jira:
    issues = jira.search_issues('project = PROJ', maxResults=10)
    for issue in issues:
        print(f'{issue.key}: {issue.fields.summary}')
```

## Complete Application Example

```python
import os
from jira import JIRA, JIRAError

def main():
    # Initialize Jira client
    jira = JIRA(
        server=os.environ.get('JIRA_SERVER', 'https://your-domain.atlassian.net'),
        basic_auth=(
            os.environ.get('JIRA_EMAIL'),
            os.environ.get('JIRA_API_TOKEN')
        )
    )

    try:
        # Get current user
        current_user = jira.current_user()
        print(f'Logged in as: {current_user}')

        # Create a new issue
        new_issue = jira.create_issue(
            project='PROJ',
            summary='Implement new feature',
            description='This is a new feature request from the API',
            issuetype={'name': 'Story'},
            priority={'name': 'High'}
        )

        print(f'Created issue: {new_issue.key}')

        # Add a comment
        comment = jira.add_comment(
            new_issue.key,
            'Starting work on this issue'
        )

        print(f'Comment added: {comment.id}')

        # Search for issues
        jql = 'project = PROJ AND status = "To Do" ORDER BY created DESC'
        issues = jira.search_issues(jql, maxResults=10)

        print(f'\nFound {issues.total} issues:')
        for issue in issues:
            print(f'  {issue.key}: {issue.fields.summary}')
            print(f'    Status: {issue.fields.status.name}')
            print(f'    Priority: {issue.fields.priority.name}')

        # Get available transitions
        transitions = jira.transitions(new_issue.key)
        print(f'\nAvailable transitions for {new_issue.key}:')
        for t in transitions:
            print(f"  {t['id']}: {t['name']}")

    except JIRAError as e:
        print(f'JIRA Error: {e.status_code} - {e.text}')
    except Exception as e:
        print(f'Error: {e}')
    finally:
        jira.close()

if __name__ == '__main__':
    main()
```

## Advanced Features

### Custom JQL Functions

```python
# Use JQL functions for dynamic queries
recent_issues = jira.search_issues('created >= startOfWeek()')
my_overdue = jira.search_issues('assignee = currentUser() AND duedate < now()')
unresolved_bugs = jira.search_issues('resolution = Unresolved AND type = Bug')
```

### Issue Links

```python
# Create issue link
jira.create_issue_link(
    type='Blocks',
    inwardIssue='PROJ-123',
    outwardIssue='PROJ-124'
)

# Get issue links
issue = jira.issue('PROJ-123')
for link in issue.fields.issuelinks:
    if hasattr(link, 'outwardIssue'):
        print(f'Blocks: {link.outwardIssue.key}')
    if hasattr(link, 'inwardIssue'):
        print(f'Blocked by: {link.inwardIssue.key}')
```

### Work Logs

```python
# Add work log to issue
worklog = jira.add_worklog(
    issue='PROJ-123',
    timeSpent='2h 30m',
    comment='Worked on implementation'
)

# Get work logs
issue = jira.issue('PROJ-123')
worklogs = jira.worklogs(issue)

for wl in worklogs:
    print(f'{wl.author.displayName}: {wl.timeSpent}')
```

## Response Format Examples

### Issue Object

```python
{
    'key': 'PROJ-123',
    'id': '10002',
    'fields': {
        'summary': 'Issue summary',
        'description': 'Issue description',
        'status': {'name': 'In Progress'},
        'priority': {'name': 'High'},
        'assignee': {
            'displayName': 'John Doe',
            'emailAddress': 'john@example.com',
            'accountId': '5b10a2844c20165700ede21g'
        },
        'reporter': {'displayName': 'Jane Smith'},
        'created': '2025-11-01T10:00:00.000+0000',
        'updated': '2025-11-07T15:30:00.000+0000',
        'labels': ['backend', 'api']
    }
}
```

### Search Results

```python
{
    'startAt': 0,
    'maxResults': 50,
    'total': 127,
    'issues': [<Issue object>, <Issue object>, ...]
}
```
