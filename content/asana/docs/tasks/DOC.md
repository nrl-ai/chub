---
name: tasks
description: "Asana API coding guide for tasks, project management, and workflow"
metadata:
  languages: "javascript"
  versions: "3.1.2"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "asana,tasks,project-management,workflow,api"
---

# Asana API Coding Guide

## 1. Golden Rule

**Always use the official Asana Node.js SDK package:**
- Package name: `asana`
- Official library maintained by Asana for Node.js and browser JavaScript

**Never use deprecated or unofficial libraries.** The `asana` package is the only supported library maintained by Asana, Inc.

**Current SDK Version:** v3.1.2 (Node.js library)

**API Version:** Asana API 1.0

## 2. Installation

### Node.js Installation

```bash
npm install asana
```

```bash
yarn add asana
```

```bash
pnpm add asana
```

**Requirements:** Node.js 12+ (recommended Node.js 18+ for production)

### Environment Variables

```bash
# Required - Personal Access Token
ASANA_ACCESS_TOKEN=your_personal_access_token_here

# Optional - OAuth credentials
ASANA_CLIENT_ID=your_client_id
ASANA_CLIENT_SECRET=your_client_secret
ASANA_REDIRECT_URI=http://localhost:3000/auth/callback

# Optional - Workspace/Organization IDs
ASANA_WORKSPACE_ID=your_workspace_gid
ASANA_PROJECT_ID=your_project_gid
```

**CRITICAL:** Never commit access tokens to version control. Use environment variables or secure secret management systems.

## 3. Initialization

### Basic Initialization with Personal Access Token

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;
```

**With ES6 Modules:**

```javascript
import Asana from 'asana';

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;
```

### Advanced Initialization with OAuth

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const oauth = client.authentications['oauth2'];
oauth.accessToken = 'YOUR_OAUTH_ACCESS_TOKEN';
```

### Client Configuration Options

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
client.defaultHeaders = {
  'asana-enable': 'new_user_task_lists,new_project_templates'
};
client.timeout = 60000; // 60 seconds

const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;
```

## 4. Core API Surfaces

### Tasks API

Tasks are the basic unit of action in Asana. They can be assigned, have due dates, contain notes, and be organized into projects.

#### Creating Tasks

**Minimal Example:**

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const tasksApiInstance = new Asana.TasksApi();

const body = {
  data: {
    name: 'Buy milk',
    workspace: '1234567890123456'
  }
};

const opts = {
  opt_fields: 'name,completed,assignee,due_on'
};

tasksApiInstance.createTask(body, opts).then((result) => {
  console.log('Task created:', result.data);
}).catch((error) => {
  console.error('Error creating task:', error.response.body);
});
```

**Advanced Example with All Options:**

```javascript
const body = {
  data: {
    name: 'Design new feature mockups',
    notes: 'Create high-fidelity mockups for the new dashboard feature',
    assignee: '9876543210987654',
    workspace: '1234567890123456',
    projects: ['1111111111111111'],
    due_on: '2025-12-31',
    due_at: '2025-12-31T17:00:00.000Z',
    start_on: '2025-01-15',
    followers: ['user_gid_1', 'user_gid_2'],
    tags: ['tag_gid_1'],
    custom_fields: {
      '5678901234567890': 'High',
      '9012345678901234': '42'
    },
    resource_subtype: 'default_task',
    completed: false,
    liked: false,
    html_notes: '<body>Create <strong>high-fidelity</strong> mockups</body>',
    external: {
      gid: 'my_external_id_123',
      data: 'Custom external data'
    }
  }
};

const opts = {
  opt_fields: 'name,assignee,assignee.name,due_on,completed,projects,projects.name,tags,tags.name,custom_fields,custom_fields.name,followers,followers.name'
};

tasksApiInstance.createTask(body, opts).then((result) => {
  console.log('Task created:', JSON.stringify(result.data, null, 2));
}).catch((error) => {
  console.error('Error:', error.response.body);
});
```

#### Getting a Task

```javascript
const taskGid = '1234567890123456';

const opts = {
  opt_fields: 'name,notes,assignee,assignee.name,assignee.email,completed,due_on,due_at,projects,projects.name,tags,tags.name,custom_fields,custom_fields.name,custom_fields.display_value,followers,followers.name,created_at,modified_at,completed_at,memberships,memberships.project.name,memberships.section.name'
};

tasksApiInstance.getTask(taskGid, opts).then((result) => {
  console.log('Task details:', JSON.stringify(result.data, null, 2));
}).catch((error) => {
  console.error('Error getting task:', error.response.body);
});
```

#### Updating Tasks

```javascript
const taskGid = '1234567890123456';

const body = {
  data: {
    name: 'Updated task name',
    notes: 'Updated task description',
    completed: true,
    assignee: 'another_user_gid',
    due_on: '2025-12-31'
  }
};

const opts = {
  opt_fields: 'name,completed,assignee.name,due_on'
};

tasksApiInstance.updateTask(body, taskGid, opts).then((result) => {
  console.log('Task updated:', result.data);
}).catch((error) => {
  console.error('Error updating task:', error.response.body);
});
```

#### Deleting Tasks

```javascript
const taskGid = '1234567890123456';

tasksApiInstance.deleteTask(taskGid).then((result) => {
  console.log('Task deleted successfully');
}).catch((error) => {
  console.error('Error deleting task:', error.response.body);
});
```

#### Searching Tasks in a Workspace

```javascript
const workspaceGid = '1234567890123456';

const opts = {
  assignee: 'me',
  completed: false,
  opt_fields: 'name,assignee.name,due_on,projects.name,completed',
  sort_by: 'due_date',
  sort_ascending: true
};

tasksApiInstance.searchTasksForWorkspace(workspaceGid, opts).then((result) => {
  console.log('Tasks found:', result.data);
}).catch((error) => {
  console.error('Error searching tasks:', error.response.body);
});
```

### Projects API

Projects represent a prioritized list of tasks or a board with columns of tasks.

#### Creating Projects

**Minimal Example:**

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const projectsApiInstance = new Asana.ProjectsApi();

const body = {
  data: {
    name: 'Marketing Campaign Q4',
    workspace: '1234567890123456'
  }
};

const opts = {
  opt_fields: 'name,owner,due_date,created_at'
};

projectsApiInstance.createProject(body, opts).then((result) => {
  console.log('Project created:', result.data);
}).catch((error) => {
  console.error('Error creating project:', error.response.body);
});
```

**Advanced Example:**

```javascript
const body = {
  data: {
    name: 'Website Redesign 2025',
    notes: 'Complete redesign of company website with new branding',
    workspace: '1234567890123456',
    team: '9876543210987654',
    owner: 'user_gid',
    due_date: '2025-12-31',
    start_on: '2025-01-01',
    color: 'light-green',
    archived: false,
    public: true,
    default_view: 'board',
    custom_fields: {
      '5678901234567890': 'Active'
    },
    followers: ['user_gid_1', 'user_gid_2']
  }
};

const opts = {
  opt_fields: 'name,owner.name,due_date,team.name,custom_fields,members,members.name,archived,color,created_at,current_status,default_view'
};

projectsApiInstance.createProject(body, opts).then((result) => {
  console.log('Project created:', JSON.stringify(result.data, null, 2));
}).catch((error) => {
  console.error('Error:', error.response.body);
});
```

#### Getting Projects

```javascript
const projectGid = '1234567890123456';

const opts = {
  opt_fields: 'name,owner.name,notes,due_date,start_on,archived,color,created_at,modified_at,team.name,workspace.name,members,members.name,followers,followers.name,custom_fields,custom_fields.name,custom_fields.display_value'
};

projectsApiInstance.getProject(projectGid, opts).then((result) => {
  console.log('Project details:', JSON.stringify(result.data, null, 2));
}).catch((error) => {
  console.error('Error getting project:', error.response.body);
});
```

#### Updating Projects

```javascript
const projectGid = '1234567890123456';

const body = {
  data: {
    name: 'Updated Project Name',
    notes: 'Updated project description',
    color: 'dark-blue',
    archived: false,
    public: false
  }
};

projectsApiInstance.updateProject(body, projectGid).then((result) => {
  console.log('Project updated:', result.data);
}).catch((error) => {
  console.error('Error updating project:', error.response.body);
});
```

#### Getting Tasks in a Project

```javascript
const projectGid = '1234567890123456';

const opts = {
  opt_fields: 'name,assignee.name,completed,due_on,tags.name'
};

tasksApiInstance.getTasksForProject(projectGid, opts).then((result) => {
  console.log('Project tasks:', result.data);
}).catch((error) => {
  console.error('Error getting project tasks:', error.response.body);
});
```

#### Adding a Task to a Project

```javascript
const taskGid = '1234567890123456';

const body = {
  data: {
    project: '9876543210987654'
  }
};

tasksApiInstance.addProjectForTask(body, taskGid).then((result) => {
  console.log('Task added to project');
}).catch((error) => {
  console.error('Error adding task to project:', error.response.body);
});
```

### Sections API

Sections divide tasks within a project into categories, workflow stages, or priorities.

#### Creating Sections

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const sectionsApiInstance = new Asana.SectionsApi();
const projectGid = '1234567890123456';

const body = {
  data: {
    name: 'In Progress'
  }
};

const opts = {
  opt_fields: 'name,created_at,project.name'
};

sectionsApiInstance.createSectionForProject(body, projectGid, opts).then((result) => {
  console.log('Section created:', result.data);
}).catch((error) => {
  console.error('Error creating section:', error.response.body);
});
```

#### Getting Sections in a Project

```javascript
const projectGid = '1234567890123456';

const opts = {
  opt_fields: 'name,created_at,project.name'
};

sectionsApiInstance.getSectionsForProject(projectGid, opts).then((result) => {
  console.log('Sections:', result.data);
}).catch((error) => {
  console.error('Error getting sections:', error.response.body);
});
```

#### Adding a Task to a Section

```javascript
const sectionGid = '1234567890123456';

const body = {
  data: {
    task: '9876543210987654'
  }
};

sectionsApiInstance.addTaskForSection(body, sectionGid).then((result) => {
  console.log('Task added to section');
}).catch((error) => {
  console.error('Error adding task to section:', error.response.body);
});
```

### Workspaces API

Workspaces are the highest-level organizational unit in Asana.

#### Getting Workspaces

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const workspacesApiInstance = new Asana.WorkspacesApi();

const opts = {
  opt_fields: 'name,is_organization,email_domains'
};

workspacesApiInstance.getWorkspaces(opts).then((result) => {
  console.log('Workspaces:', result.data);
}).catch((error) => {
  console.error('Error getting workspaces:', error.response.body);
});
```

#### Getting a Workspace

```javascript
const workspaceGid = '1234567890123456';

const opts = {
  opt_fields: 'name,is_organization,email_domains'
};

workspacesApiInstance.getWorkspace(workspaceGid, opts).then((result) => {
  console.log('Workspace:', result.data);
}).catch((error) => {
  console.error('Error getting workspace:', error.response.body);
});
```

#### Getting Projects in a Workspace

```javascript
const workspaceGid = '1234567890123456';

const projectsApiInstance = new Asana.ProjectsApi();

const opts = {
  archived: false,
  opt_fields: 'name,owner.name,due_date,created_at'
};

projectsApiInstance.getProjectsForWorkspace(workspaceGid, opts).then((result) => {
  console.log('Projects:', result.data);
}).catch((error) => {
  console.error('Error getting projects:', error.response.body);
});
```

### Users API

Users represent individuals in Asana.

#### Getting the Current User

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const usersApiInstance = new Asana.UsersApi();
const userGid = 'me';

const opts = {
  opt_fields: 'name,email,photo,workspaces,workspaces.name'
};

usersApiInstance.getUser(userGid, opts).then((result) => {
  console.log('Current user:', result.data);
}).catch((error) => {
  console.error('Error getting user:', error.response.body);
});
```

#### Getting Users in a Workspace

```javascript
const workspaceGid = '1234567890123456';

const opts = {
  opt_fields: 'name,email,photo'
};

usersApiInstance.getUsersForWorkspace(workspaceGid, opts).then((result) => {
  console.log('Users:', result.data);
}).catch((error) => {
  console.error('Error getting users:', error.response.body);
});
```

### Teams API

Teams organize people and projects within a workspace.

#### Getting Teams

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const teamsApiInstance = new Asana.TeamsApi();
const workspaceGid = '1234567890123456';

const opts = {
  opt_fields: 'name,description,organization.name'
};

teamsApiInstance.getTeamsForWorkspace(workspaceGid, opts).then((result) => {
  console.log('Teams:', result.data);
}).catch((error) => {
  console.error('Error getting teams:', error.response.body);
});
```

#### Getting a Team

```javascript
const teamGid = '1234567890123456';

const opts = {
  opt_fields: 'name,description,organization.name,html_description'
};

teamsApiInstance.getTeam(teamGid, opts).then((result) => {
  console.log('Team:', result.data);
}).catch((error) => {
  console.error('Error getting team:', error.response.body);
});
```

### Custom Fields API

Custom fields allow you to add structured metadata to tasks and projects.

#### Getting Custom Fields in a Workspace

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const customFieldsApiInstance = new Asana.CustomFieldsApi();
const workspaceGid = '1234567890123456';

const opts = {
  opt_fields: 'name,resource_subtype,type,enum_options,enum_options.name,precision'
};

customFieldsApiInstance.getCustomFieldsForWorkspace(workspaceGid, opts).then((result) => {
  console.log('Custom fields:', result.data);
}).catch((error) => {
  console.error('Error getting custom fields:', error.response.body);
});
```

#### Creating a Custom Field

```javascript
const workspaceGid = '1234567890123456';

const body = {
  data: {
    name: 'Priority',
    resource_subtype: 'enum',
    type: 'enum',
    workspace: workspaceGid,
    enum_options: [
      { name: 'Low', enabled: true, color: 'blue' },
      { name: 'Medium', enabled: true, color: 'yellow' },
      { name: 'High', enabled: true, color: 'red' }
    ]
  }
};

customFieldsApiInstance.createCustomField(body).then((result) => {
  console.log('Custom field created:', result.data);
}).catch((error) => {
  console.error('Error creating custom field:', error.response.body);
});
```

#### Updating Custom Field Value on a Task

```javascript
const taskGid = '1234567890123456';
const customFieldGid = '9876543210987654';

const body = {
  data: {
    custom_fields: {
      [customFieldGid]: 'High'
    }
  }
};

tasksApiInstance.updateTask(body, taskGid).then((result) => {
  console.log('Custom field updated');
}).catch((error) => {
  console.error('Error updating custom field:', error.response.body);
});
```

### Tags API

Tags are labels that can be attached to tasks.

#### Creating Tags

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const tagsApiInstance = new Asana.TagsApi();

const body = {
  data: {
    name: 'Bug',
    workspace: '1234567890123456',
    color: 'red'
  }
};

const opts = {
  opt_fields: 'name,color,created_at'
};

tagsApiInstance.createTag(body, opts).then((result) => {
  console.log('Tag created:', result.data);
}).catch((error) => {
  console.error('Error creating tag:', error.response.body);
});
```

#### Getting Tags in a Workspace

```javascript
const workspaceGid = '1234567890123456';

const opts = {
  opt_fields: 'name,color,created_at'
};

tagsApiInstance.getTagsForWorkspace(workspaceGid, opts).then((result) => {
  console.log('Tags:', result.data);
}).catch((error) => {
  console.error('Error getting tags:', error.response.body);
});
```

### Attachments API

Attachments are files or URLs associated with tasks.

#### Uploading an Attachment to a Task

```javascript
const Asana = require('asana');
const fs = require('fs');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const attachmentsApiInstance = new Asana.AttachmentsApi();
const taskGid = '1234567890123456';

const body = {
  file: fs.createReadStream('/path/to/file.pdf'),
  parent: taskGid
};

const opts = {
  opt_fields: 'name,download_url,size,host'
};

attachmentsApiInstance.createAttachmentForObject(body, opts).then((result) => {
  console.log('Attachment uploaded:', result.data);
}).catch((error) => {
  console.error('Error uploading attachment:', error.response.body);
});
```

#### Getting Attachments for a Task

```javascript
const taskGid = '1234567890123456';

const opts = {
  opt_fields: 'name,download_url,size,host,created_at'
};

attachmentsApiInstance.getAttachmentsForObject(taskGid, opts).then((result) => {
  console.log('Attachments:', result.data);
}).catch((error) => {
  console.error('Error getting attachments:', error.response.body);
});
```

### Webhooks API

Webhooks allow applications to be notified of changes in Asana.

#### Creating a Webhook

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const webhooksApiInstance = new Asana.WebhooksApi();

const body = {
  data: {
    resource: '1234567890123456', // project GID
    target: 'https://example.com/webhooks/asana'
  }
};

const opts = {
  opt_fields: 'resource,target,active,last_success_at,last_failure_at'
};

webhooksApiInstance.createWebhook(body, opts).then((result) => {
  console.log('Webhook created:', result.data);
}).catch((error) => {
  console.error('Error creating webhook:', error.response.body);
});
```

#### Getting Webhooks

```javascript
const workspaceGid = '1234567890123456';

const opts = {
  resource: workspaceGid,
  opt_fields: 'resource,target,active,created_at,last_success_at,last_failure_at'
};

webhooksApiInstance.getWebhooks(opts).then((result) => {
  console.log('Webhooks:', result.data);
}).catch((error) => {
  console.error('Error getting webhooks:', error.response.body);
});
```

#### Deleting a Webhook

```javascript
const webhookGid = '1234567890123456';

webhooksApiInstance.deleteWebhook(webhookGid).then((result) => {
  console.log('Webhook deleted');
}).catch((error) => {
  console.error('Error deleting webhook:', error.response.body);
});
```

#### Handling Webhook Events

```javascript
const express = require('express');
const crypto = require('crypto');
const app = express();

app.use(express.json());

app.post('/webhooks/asana', (req, res) => {
  // Verify webhook signature
  const signature = req.headers['x-hook-signature'];
  const secret = process.env.ASANA_WEBHOOK_SECRET;

  const hash = crypto
    .createHmac('sha256', secret)
    .update(JSON.stringify(req.body))
    .digest('hex');

  if (signature !== hash) {
    return res.status(401).send('Invalid signature');
  }

  // Handle handshake
  if (req.headers['x-hook-secret']) {
    res.setHeader('X-Hook-Secret', req.headers['x-hook-secret']);
    return res.status(200).send();
  }

  // Process webhook events
  const events = req.body.events || [];

  events.forEach((event) => {
    console.log('Event:', event.action, 'Resource:', event.resource);

    if (event.action === 'added') {
      console.log('Task added:', event.resource.gid);
    } else if (event.action === 'changed') {
      console.log('Task changed:', event.resource.gid);
    } else if (event.action === 'removed') {
      console.log('Task removed:', event.resource.gid);
    }
  });

  res.status(200).send();
});

app.listen(3000, () => {
  console.log('Webhook server listening on port 3000');
});
```

### Stories API (Comments and Activity)

Stories represent the activity feed on tasks and projects.

#### Creating a Comment on a Task

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const storiesApiInstance = new Asana.StoriesApi();
const taskGid = '1234567890123456';

const body = {
  data: {
    text: 'This is a comment on the task',
    is_pinned: false
  }
};

const opts = {
  opt_fields: 'text,created_at,created_by.name,is_pinned'
};

storiesApiInstance.createStoryForTask(body, taskGid, opts).then((result) => {
  console.log('Comment created:', result.data);
}).catch((error) => {
  console.error('Error creating comment:', error.response.body);
});
```

#### Getting Comments for a Task

```javascript
const taskGid = '1234567890123456';

const opts = {
  opt_fields: 'text,created_at,created_by.name,resource_subtype,type'
};

storiesApiInstance.getStoriesForTask(taskGid, opts).then((result) => {
  console.log('Stories:', result.data);
}).catch((error) => {
  console.error('Error getting stories:', error.response.body);
});
```

### Portfolios API

Portfolios are collections of projects.

#### Creating a Portfolio

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const portfoliosApiInstance = new Asana.PortfoliosApi();

const body = {
  data: {
    name: 'Product Initiatives',
    workspace: '1234567890123456',
    color: 'light-pink',
    public: false
  }
};

const opts = {
  opt_fields: 'name,color,created_at,owner.name'
};

portfoliosApiInstance.createPortfolio(body, opts).then((result) => {
  console.log('Portfolio created:', result.data);
}).catch((error) => {
  console.error('Error creating portfolio:', error.response.body);
});
```

#### Adding a Project to a Portfolio

```javascript
const portfolioGid = '1234567890123456';

const body = {
  data: {
    project: '9876543210987654'
  }
};

portfoliosApiInstance.addItemForPortfolio(body, portfolioGid).then((result) => {
  console.log('Project added to portfolio');
}).catch((error) => {
  console.error('Error adding project to portfolio:', error.response.body);
});
```

## Error Handling

### Basic Error Handling

```javascript
const Asana = require('asana');

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const tasksApiInstance = new Asana.TasksApi();

async function getTask(taskGid) {
  try {
    const result = await tasksApiInstance.getTask(taskGid);
    console.log('Task:', result.data);
    return result.data;
  } catch (error) {
    if (error.response) {
      console.error('API Error:', error.response.body);
      console.error('Status:', error.response.status);

      if (error.response.status === 401) {
        console.error('Authentication failed. Check your access token.');
      } else if (error.response.status === 403) {
        console.error('Permission denied. You do not have access to this resource.');
      } else if (error.response.status === 404) {
        console.error('Resource not found.');
      } else if (error.response.status === 429) {
        console.error('Rate limit exceeded. Please retry after some time.');
      }
    } else {
      console.error('Network error:', error.message);
    }
    throw error;
  }
}
```

### Retry Pattern for Rate Limiting

```javascript
async function makeRequestWithRetry(apiCall, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await apiCall();
    } catch (error) {
      if (error.response?.status === 429) {
        const retryAfter = error.response.headers['retry-after'] || Math.pow(2, i);
        console.log(`Rate limited. Retrying after ${retryAfter} seconds...`);
        await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
      } else {
        throw error;
      }
    }
  }
  throw new Error('Max retries exceeded');
}

// Usage
makeRequestWithRetry(() => tasksApiInstance.getTask(taskGid))
  .then(result => console.log('Task:', result.data))
  .catch(error => console.error('Failed after retries:', error));
```

## Pagination

### Handling Paginated Results

```javascript
async function getAllTasks(projectGid) {
  const allTasks = [];
  let offset = undefined;

  do {
    const opts = {
      limit: 100,
      offset: offset,
      opt_fields: 'name,completed,assignee.name'
    };

    const result = await tasksApiInstance.getTasksForProject(projectGid, opts);
    allTasks.push(...result.data);

    offset = result.next_page?.offset;
  } while (offset);

  return allTasks;
}

// Usage
getAllTasks('1234567890123456').then(tasks => {
  console.log(`Total tasks: ${tasks.length}`);
  tasks.forEach(task => console.log(task.name));
});
```

## OAuth Authentication Flow

### Setting Up OAuth

```javascript
const express = require('express');
const Asana = require('asana');

const app = express();

const ASANA_CLIENT_ID = process.env.ASANA_CLIENT_ID;
const ASANA_CLIENT_SECRET = process.env.ASANA_CLIENT_SECRET;
const REDIRECT_URI = process.env.ASANA_REDIRECT_URI;

// Generate authorization URL
app.get('/auth', (req, res) => {
  const authUrl = `https://app.asana.com/-/oauth_authorize?client_id=${ASANA_CLIENT_ID}&redirect_uri=${encodeURIComponent(REDIRECT_URI)}&response_type=code&state=random_state_string`;
  res.redirect(authUrl);
});

// Handle OAuth callback
app.get('/auth/callback', async (req, res) => {
  const code = req.query.code;

  try {
    const response = await fetch('https://app.asana.com/-/oauth_token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        client_id: ASANA_CLIENT_ID,
        client_secret: ASANA_CLIENT_SECRET,
        redirect_uri: REDIRECT_URI,
        code: code
      })
    });

    const data = await response.json();

    if (data.access_token) {
      // Store access_token and refresh_token securely
      console.log('Access token:', data.access_token);
      console.log('Refresh token:', data.refresh_token);

      // Initialize Asana client with OAuth token
      const client = Asana.ApiClient.instance;
      const oauth = client.authentications['oauth2'];
      oauth.accessToken = data.access_token;

      res.send('Authentication successful!');
    } else {
      res.status(400).send('Authentication failed');
    }
  } catch (error) {
    console.error('OAuth error:', error);
    res.status(500).send('Error during authentication');
  }
});

app.listen(3000, () => {
  console.log('Server listening on http://localhost:3000');
});
```

### Refreshing OAuth Tokens

```javascript
async function refreshAccessToken(refreshToken) {
  try {
    const response = await fetch('https://app.asana.com/-/oauth_token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        client_id: process.env.ASANA_CLIENT_ID,
        client_secret: process.env.ASANA_CLIENT_SECRET,
        refresh_token: refreshToken
      })
    });

    const data = await response.json();

    if (data.access_token) {
      console.log('New access token:', data.access_token);
      return data.access_token;
    }
  } catch (error) {
    console.error('Token refresh error:', error);
    throw error;
  }
}
```

## TypeScript Support

### Using Asana SDK with TypeScript

```typescript
import Asana from 'asana';

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN!;

const tasksApiInstance = new Asana.TasksApi();

interface TaskData {
  name: string;
  workspace: string;
  assignee?: string;
  due_on?: string;
}

async function createTask(taskData: TaskData): Promise<any> {
  const body = {
    data: taskData
  };

  const opts = {
    opt_fields: 'name,assignee.name,due_on,completed'
  };

  try {
    const result = await tasksApiInstance.createTask(body, opts);
    return result.data;
  } catch (error) {
    console.error('Error creating task:', error);
    throw error;
  }
}

// Usage
createTask({
  name: 'TypeScript task',
  workspace: '1234567890123456',
  due_on: '2025-12-31'
}).then(task => {
  console.log('Task created:', task);
});
```

## Batch Operations

### Creating Multiple Tasks

```javascript
async function createMultipleTasks(taskDataArray) {
  const promises = taskDataArray.map(taskData => {
    const body = { data: taskData };
    return tasksApiInstance.createTask(body);
  });

  try {
    const results = await Promise.all(promises);
    console.log(`Created ${results.length} tasks`);
    return results.map(r => r.data);
  } catch (error) {
    console.error('Error creating tasks:', error);
    throw error;
  }
}

// Usage
const tasksToCreate = [
  { name: 'Task 1', workspace: '1234567890123456' },
  { name: 'Task 2', workspace: '1234567890123456' },
  { name: 'Task 3', workspace: '1234567890123456' }
];

createMultipleTasks(tasksToCreate).then(tasks => {
  console.log('Tasks created:', tasks);
});
```

## Environment Variable Validation

```javascript
const Asana = require('asana');
require('dotenv').config();

function validateEnvironment() {
  if (!process.env.ASANA_ACCESS_TOKEN) {
    console.error('Error: ASANA_ACCESS_TOKEN is required in .env file');
    process.exit(1);
  }

  console.log('Environment validated successfully');
}

validateEnvironment();

const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;
```

## Complete Example Application

```javascript
const Asana = require('asana');
require('dotenv').config();

// Validate environment
if (!process.env.ASANA_ACCESS_TOKEN || !process.env.ASANA_WORKSPACE_ID) {
  console.error('Missing required environment variables');
  process.exit(1);
}

// Initialize client
const client = Asana.ApiClient.instance;
const token = client.authentications['token'];
token.accessToken = process.env.ASANA_ACCESS_TOKEN;

const tasksApiInstance = new Asana.TasksApi();
const projectsApiInstance = new Asana.ProjectsApi();
const usersApiInstance = new Asana.UsersApi();

async function main() {
  try {
    // Get current user
    console.log('Getting current user...');
    const userResult = await usersApiInstance.getUser('me', {
      opt_fields: 'name,email,workspaces.name'
    });
    console.log('Logged in as:', userResult.data.name);

    // Create a project
    console.log('\nCreating project...');
    const projectBody = {
      data: {
        name: 'API Demo Project',
        workspace: process.env.ASANA_WORKSPACE_ID,
        notes: 'Project created via Asana API'
      }
    };
    const projectResult = await projectsApiInstance.createProject(projectBody, {
      opt_fields: 'name,gid'
    });
    console.log('Project created:', projectResult.data.name);
    const projectGid = projectResult.data.gid;

    // Create tasks
    console.log('\nCreating tasks...');
    const taskNames = ['Design mockups', 'Implement feature', 'Write tests', 'Deploy'];

    for (const taskName of taskNames) {
      const taskBody = {
        data: {
          name: taskName,
          workspace: process.env.ASANA_WORKSPACE_ID,
          projects: [projectGid]
        }
      };
      const taskResult = await tasksApiInstance.createTask(taskBody);
      console.log('Created task:', taskResult.data.name);
    }

    // Get all tasks in project
    console.log('\nFetching project tasks...');
    const tasksResult = await tasksApiInstance.getTasksForProject(projectGid, {
      opt_fields: 'name,completed'
    });
    console.log(`Project has ${tasksResult.data.length} tasks`);

    // Mark first task as complete
    if (tasksResult.data.length > 0) {
      const firstTaskGid = tasksResult.data[0].gid;
      console.log('\nMarking first task as complete...');
      await tasksApiInstance.updateTask(
        { data: { completed: true } },
        firstTaskGid
      );
      console.log('Task marked as complete');
    }

    console.log('\nDemo completed successfully!');

  } catch (error) {
    console.error('Error:', error.response?.body || error.message);
    process.exit(1);
  }
}

main();
```

## Notes

The Asana Node.js SDK is auto-generated from the OpenAPI specification, ensuring it stays current with the latest API features. The SDK supports both Personal Access Tokens for simple authentication and OAuth 2.0 for multi-user applications. All API methods return Promises and can be used with async/await syntax. The SDK automatically handles request formatting and response parsing. Rate limits apply: 1500 requests per minute for most operations, with some endpoints having lower limits. Use the `opt_fields` parameter to optimize API responses by requesting only the fields you need.
