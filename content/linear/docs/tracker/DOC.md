---
name: tracker
description: "Linear SDK for JavaScript/TypeScript for issue tracking and project management via GraphQL"
metadata:
  languages: "javascript"
  versions: "62.0.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "linear,tracker,issues,project-management,graphql"
---

# Linear SDK for JavaScript/TypeScript

## Golden Rule

**ALWAYS use `@linear/sdk` version 62.0.0 or later.**

Install with:
```bash
npm install @linear/sdk
```

**DO NOT use:**
- Unofficial Linear packages
- Direct GraphQL requests without the SDK (unless specifically required)
- Deprecated authentication methods
- Outdated Linear client libraries

The `@linear/sdk` is the official Linear TypeScript SDK that provides strongly-typed access to Linear's GraphQL API.

---

## Installation

### Install the SDK

```bash
npm install @linear/sdk
```

### Environment Setup

Create a `.env` file:

```bash
LINEAR_API_KEY=lin_api_your_personal_api_key_here
# OR for OAuth
LINEAR_ACCESS_TOKEN=your_oauth_access_token_here
```

### TypeScript Configuration (Optional)

The SDK ships with TypeScript types. No additional `@types` package needed.

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "esModuleInterop": true,
    "strict": true
  }
}
```

---

## Initialization

### Import the Client

```typescript
import { LinearClient } from '@linear/sdk'
```

```javascript
const { LinearClient } = require('@linear/sdk')
```

### Authentication with API Key

```typescript
import { LinearClient } from '@linear/sdk'

const client = new LinearClient({
  apiKey: process.env.LINEAR_API_KEY
})
```

```javascript
const { LinearClient } = require('@linear/sdk')
require('dotenv').config()

const client = new LinearClient({
  apiKey: process.env.LINEAR_API_KEY
})
```

### Authentication with OAuth 2.0

```typescript
const client = new LinearClient({
  accessToken: process.env.LINEAR_ACCESS_TOKEN
})
```

### Get Current User

```typescript
async function getCurrentUser() {
  const me = await client.viewer
  console.log(`Logged in as: ${me.name} (${me.email})`)
  return me
}
```

---

## Core API Surfaces

### Issues

#### Fetch All Issues

**Minimal:**

```typescript
async function getAllIssues() {
  const issues = await client.issues()

  issues.nodes.forEach(issue => {
    console.log(`${issue.identifier}: ${issue.title}`)
  })

  return issues
}
```

**Advanced with Pagination:**

```typescript
async function getIssuesPaginated() {
  const issues = await client.issues({
    first: 50,
    orderBy: 'createdAt'
  })

  for (const issue of issues.nodes) {
    console.log(`[${issue.identifier}] ${issue.title}`)
    console.log(`  Status: ${(await issue.state)?.name}`)
    console.log(`  Assignee: ${(await issue.assignee)?.displayName || 'Unassigned'}`)
    console.log(`  Priority: ${issue.priority}`)
  }

  // Handle pagination
  if (issues.pageInfo.hasNextPage) {
    const nextPage = await client.issues({
      first: 50,
      after: issues.pageInfo.endCursor
    })
    // Process next page
  }

  return issues
}
```

#### Get User's Assigned Issues

```typescript
async function getMyIssues() {
  const me = await client.viewer
  const myIssues = await me.assignedIssues()

  if (myIssues.nodes.length) {
    myIssues.nodes.forEach(issue => {
      console.log(`${me.displayName} has issue: ${issue.title}`)
    })
  }

  return myIssues
}
```

#### Query Single Issue by ID

```typescript
async function getIssueById(issueId: string) {
  const issue = await client.issue(issueId)

  console.log(`Title: ${issue.title}`)
  console.log(`Description: ${issue.description}`)
  console.log(`Created: ${issue.createdAt}`)

  const state = await issue.state
  console.log(`State: ${state?.name}`)

  const assignee = await issue.assignee
  console.log(`Assignee: ${assignee?.displayName}`)

  return issue
}

// Usage: Can use UUID or identifier like "ENG-123"
getIssueById("ENG-123")
```

#### Create Issue

**Minimal:**

```typescript
async function createIssue(teamId: string, title: string) {
  const issuePayload = await client.issueCreate({
    teamId: teamId,
    title: title
  })

  if (issuePayload.success) {
    console.log(`Created issue: ${issuePayload.issue?.identifier}`)
    return issuePayload.issue
  } else {
    throw new Error('Failed to create issue')
  }
}
```

**Advanced with All Fields:**

```typescript
async function createDetailedIssue() {
  // First get team ID
  const teams = await client.teams()
  const team = teams.nodes[0]

  // Get workflow state
  const states = await client.workflowStates({
    filter: { team: { id: { eq: team.id } } }
  })
  const todoState = states.nodes.find(s => s.name === 'Todo')

  // Get labels
  const labels = await client.issueLabels()
  const bugLabel = labels.nodes.find(l => l.name === 'Bug')

  const issuePayload = await client.issueCreate({
    teamId: team.id,
    title: 'Fix authentication error',
    description: '## Problem\n\nUsers cannot log in\n\n## Steps to reproduce\n\n1. Go to login\n2. Enter credentials',
    priority: 1, // 0=No priority, 1=Urgent, 2=High, 3=Medium, 4=Low
    estimate: 3, // Story points
    stateId: todoState?.id,
    labelIds: bugLabel ? [bugLabel.id] : [],
    assigneeId: (await client.viewer).id,
    dueDate: new Date('2025-12-31')
  })

  if (issuePayload.success) {
    return issuePayload.issue
  }

  throw new Error('Failed to create issue')
}
```

#### Update Issue

**Minimal:**

```typescript
async function updateIssue(issueId: string, title: string) {
  const updatePayload = await client.issueUpdate(issueId, {
    title: title
  })

  if (updatePayload.success) {
    console.log('Issue updated')
    return updatePayload.issue
  }
}
```

**Advanced:**

```typescript
async function updateIssueComplete(issueId: string) {
  // Get "Done" state
  const issue = await client.issue(issueId)
  const team = await issue.team
  const states = await client.workflowStates({
    filter: { team: { id: { eq: team.id } } }
  })
  const doneState = states.nodes.find(s => s.name === 'Done')

  const updatePayload = await client.issueUpdate(issueId, {
    stateId: doneState?.id,
    title: 'Updated Issue Title',
    description: 'New description',
    priority: 3,
    estimate: 5
  })

  if (updatePayload.success) {
    console.log(`Updated: ${updatePayload.issue?.title}`)
    return updatePayload.issue
  }
}
```

#### Filter Issues

```typescript
async function filterIssues() {
  const issues = await client.issues({
    filter: {
      assignee: {
        email: { eq: 'user@example.com' }
      },
      state: {
        name: { in: ['Todo', 'In Progress'] }
      },
      priority: {
        gte: 2 // High priority and above
      },
      createdAt: {
        gte: new Date('2025-01-01')
      }
    },
    orderBy: 'priority',
    first: 25
  })

  return issues.nodes
}
```

#### Complex Filtering with AND/OR

```typescript
async function complexFilterIssues() {
  const issues = await client.issues({
    filter: {
      or: [
        { priority: { eq: 1 } }, // Urgent
        {
          and: [
            { priority: { eq: 2 } }, // High
            { dueDate: { lte: new Date() } } // Overdue
          ]
        }
      ]
    }
  })

  return issues.nodes
}
```

#### Archive Issue

```typescript
async function archiveIssue(issueId: string) {
  const payload = await client.issueArchive(issueId)

  if (payload.success) {
    console.log('Issue archived')
  }

  return payload
}
```

### Comments

#### Get Issue Comments

```typescript
async function getIssueComments(issueId: string) {
  const issue = await client.issue(issueId)
  const comments = await issue.comments()

  for (const comment of comments.nodes) {
    const user = await comment.user
    console.log(`${user?.displayName}: ${comment.body}`)
    console.log(`Posted: ${comment.createdAt}`)
  }

  return comments
}
```

#### Create Comment

**Minimal:**

```typescript
async function addComment(issueId: string, body: string) {
  const commentPayload = await client.commentCreate({
    issueId: issueId,
    body: body
  })

  if (commentPayload.success) {
    console.log('Comment added')
    return commentPayload.comment
  }

  throw new Error('Failed to create comment')
}
```

**Advanced with Markdown:**

```typescript
async function addDetailedComment(issueId: string) {
  const body = `## Update

I've investigated this issue and found:

- The authentication token expires too quickly
- Need to implement refresh token logic

**Next steps:**
1. Update token service
2. Add refresh endpoint
3. Test token renewal

cc @teammate`

  const commentPayload = await client.commentCreate({
    issueId: issueId,
    body: body
  })

  return commentPayload.comment
}
```

#### Update Comment

```typescript
async function updateComment(commentId: string, newBody: string) {
  const payload = await client.commentUpdate(commentId, {
    body: newBody
  })

  if (payload.success) {
    return payload.comment
  }
}
```

#### Delete Comment

```typescript
async function deleteComment(commentId: string) {
  const payload = await client.commentDelete(commentId)

  if (payload.success) {
    console.log('Comment deleted')
  }

  return payload
}
```

### Teams

#### Get All Teams

```typescript
async function getTeams() {
  const teams = await client.teams()

  teams.nodes.forEach(team => {
    console.log(`${team.name} (${team.key})`)
  })

  return teams
}
```

#### Get Team by ID

```typescript
async function getTeam(teamId: string) {
  const team = await client.team(teamId)

  console.log(`Team: ${team.name}`)
  console.log(`Key: ${team.key}`)
  console.log(`Description: ${team.description}`)

  return team
}
```

#### Get Team Issues

```typescript
async function getTeamIssues(teamId: string) {
  const team = await client.team(teamId)
  const issues = await team.issues({
    first: 50,
    orderBy: 'updatedAt'
  })

  console.log(`${team.name} has ${issues.nodes.length} issues`)

  return issues
}
```

#### Get Team Members

```typescript
async function getTeamMembers(teamId: string) {
  const team = await client.team(teamId)
  const members = await team.members()

  for (const member of members.nodes) {
    console.log(`${member.displayName} - ${member.email}`)
  }

  return members
}
```

### Projects

#### Get All Projects

```typescript
async function getProjects() {
  const projects = await client.projects()

  for (const project of projects.nodes) {
    console.log(`${project.name}`)
    console.log(`  State: ${project.state}`)
    console.log(`  Progress: ${project.progress}%`)
  }

  return projects
}
```

#### Get Project by ID

```typescript
async function getProject(projectId: string) {
  const project = await client.project(projectId)

  console.log(`Project: ${project.name}`)
  console.log(`Description: ${project.description}`)
  console.log(`Start: ${project.startDate}`)
  console.log(`Target: ${project.targetDate}`)

  const lead = await project.lead
  console.log(`Lead: ${lead?.displayName}`)

  return project
}
```

#### Get Project Issues

```typescript
async function getProjectIssues(projectId: string) {
  const project = await client.project(projectId)
  const issues = await project.issues()

  console.log(`${project.name} issues:`)
  issues.nodes.forEach(issue => {
    console.log(`  - ${issue.identifier}: ${issue.title}`)
  })

  return issues
}
```

#### Create Project

```typescript
async function createProject(teamId: string) {
  const payload = await client.projectCreate({
    teamIds: [teamId],
    name: 'Q4 Authentication Improvements',
    description: 'Improve authentication flow and security',
    state: 'started',
    targetDate: new Date('2025-12-31')
  })

  if (payload.success) {
    return payload.project
  }
}
```

#### Update Project

```typescript
async function updateProject(projectId: string) {
  const payload = await client.projectUpdate(projectId, {
    state: 'completed',
    progress: 100
  })

  if (payload.success) {
    console.log('Project completed')
    return payload.project
  }
}
```

### Labels

#### Get All Labels

```typescript
async function getLabels() {
  const labels = await client.issueLabels()

  labels.nodes.forEach(label => {
    console.log(`${label.name} - ${label.color}`)
  })

  return labels
}
```

#### Get Label by ID

```typescript
async function getLabel(labelId: string) {
  const label = await client.issueLabel(labelId)

  console.log(`Label: ${label.name}`)
  console.log(`Description: ${label.description}`)

  return label
}
```

#### Create Label

```typescript
async function createLabel(teamId: string) {
  const payload = await client.issueLabelCreate({
    teamId: teamId,
    name: 'security',
    description: 'Security related issues',
    color: '#ff0000'
  })

  if (payload.success) {
    return payload.issueLabel
  }
}
```

#### Filter Issues by Label

```typescript
async function getIssuesByLabel(labelName: string) {
  const issues = await client.issues({
    filter: {
      labels: {
        name: { eq: labelName }
      }
    }
  })

  return issues.nodes
}
```

### Workflow States

#### Get All Workflow States

```typescript
async function getWorkflowStates() {
  const states = await client.workflowStates()

  states.nodes.forEach(state => {
    console.log(`${state.name} (${state.type})`)
  })

  return states
}
```

#### Get Workflow States for Team

```typescript
async function getTeamWorkflowStates(teamId: string) {
  const states = await client.workflowStates({
    filter: {
      team: { id: { eq: teamId } }
    }
  })

  console.log('Available states:')
  states.nodes.forEach(state => {
    console.log(`  - ${state.name}`)
  })

  return states
}
```

#### Get Issues in Specific State

```typescript
async function getIssuesInState(stateId: string) {
  const state = await client.workflowState(stateId)
  const issues = await state.issues()

  console.log(`Issues in "${state.name}":`)
  issues.nodes.forEach(issue => {
    console.log(`  ${issue.identifier}: ${issue.title}`)
  })

  return issues
}
```

### Users

#### Get Current User

```typescript
async function getCurrentUser() {
  const me = await client.viewer

  console.log(`Name: ${me.displayName}`)
  console.log(`Email: ${me.email}`)
  console.log(`Admin: ${me.admin}`)

  return me
}
```

#### Get User by ID

```typescript
async function getUser(userId: string) {
  const user = await client.user(userId)

  console.log(`${user.displayName}`)
  console.log(`Email: ${user.email}`)
  console.log(`Active: ${user.active}`)

  return user
}
```

#### Get All Users

```typescript
async function getUsers() {
  const users = await client.users()

  users.nodes.forEach(user => {
    console.log(`${user.displayName} - ${user.email}`)
  })

  return users
}
```

#### Get User's Teams

```typescript
async function getUserTeams(userId: string) {
  const user = await client.user(userId)
  const teams = await user.teams()

  console.log(`${user.displayName}'s teams:`)
  teams.nodes.forEach(team => {
    console.log(`  - ${team.name}`)
  })

  return teams
}
```

### Cycles

#### Get Active Cycles

```typescript
async function getActiveCycles() {
  const cycles = await client.cycles({
    filter: {
      isActive: { eq: true }
    }
  })

  for (const cycle of cycles.nodes) {
    console.log(`${cycle.name}`)
    console.log(`  Start: ${cycle.startsAt}`)
    console.log(`  End: ${cycle.endsAt}`)
    console.log(`  Progress: ${cycle.progress}%`)
  }

  return cycles
}
```

#### Get Cycle Issues

```typescript
async function getCycleIssues(cycleId: string) {
  const cycle = await client.cycle(cycleId)
  const issues = await cycle.issues()

  console.log(`${cycle.name} issues:`)
  issues.nodes.forEach(issue => {
    console.log(`  ${issue.identifier}: ${issue.title}`)
  })

  return issues
}
```

### Attachments

#### Create Attachment

```typescript
async function createAttachment(issueId: string) {
  const payload = await client.attachmentCreate({
    issueId: issueId,
    title: 'Design Mockup',
    url: 'https://example.com/mockup.png',
    subtitle: 'Login page redesign'
  })

  if (payload.success) {
    return payload.attachment
  }
}
```

#### Get Issue Attachments

```typescript
async function getIssueAttachments(issueId: string) {
  const issue = await client.issue(issueId)
  const attachments = await issue.attachments()

  attachments.nodes.forEach(attachment => {
    console.log(`${attachment.title}: ${attachment.url}`)
  })

  return attachments
}
```

### Webhooks

#### Create Webhook

```typescript
async function createWebhook() {
  const payload = await client.webhookCreate({
    url: 'https://example.com/webhook',
    label: 'Issue Updates',
    resourceTypes: ['Issue', 'Comment'],
    enabled: true
  })

  if (payload.success) {
    console.log(`Webhook created: ${payload.webhook?.id}`)
    return payload.webhook
  }
}
```

#### Get Webhooks

```typescript
async function getWebhooks() {
  const webhooks = await client.webhooks()

  webhooks.nodes.forEach(webhook => {
    console.log(`${webhook.label}`)
    console.log(`  URL: ${webhook.url}`)
    console.log(`  Enabled: ${webhook.enabled}`)
    console.log(`  Resources: ${webhook.resourceTypes.join(', ')}`)
  })

  return webhooks
}
```

#### Update Webhook

```typescript
async function updateWebhook(webhookId: string) {
  const payload = await client.webhookUpdate(webhookId, {
    enabled: false
  })

  if (payload.success) {
    return payload.webhook
  }
}
```

#### Delete Webhook

```typescript
async function deleteWebhook(webhookId: string) {
  const payload = await client.webhookDelete(webhookId)

  if (payload.success) {
    console.log('Webhook deleted')
  }

  return payload
}
```

### Search

#### Search Issues

```typescript
async function searchIssues(query: string) {
  const results = await client.searchIssues(query)

  results.nodes.forEach(issue => {
    console.log(`${issue.identifier}: ${issue.title}`)
  })

  return results
}

// Usage
searchIssues('authentication bug')
```

#### Search Projects

```typescript
async function searchProjects(query: string) {
  const results = await client.searchProjects(query)

  results.nodes.forEach(project => {
    console.log(`${project.name}`)
  })

  return results
}
```

---

## Pagination

### Basic Pagination

```typescript
async function paginateIssues() {
  let allIssues = []
  let hasNextPage = true
  let cursor = null

  while (hasNextPage) {
    const issues = await client.issues({
      first: 50,
      after: cursor
    })

    allIssues.push(...issues.nodes)

    hasNextPage = issues.pageInfo.hasNextPage
    cursor = issues.pageInfo.endCursor
  }

  console.log(`Total issues: ${allIssues.length}`)
  return allIssues
}
```

### Reverse Pagination

```typescript
async function paginateReverse() {
  const issues = await client.issues({
    last: 25,
    before: someCursor
  })

  return issues.nodes
}
```

---

## Error Handling

### Try-Catch Pattern

```typescript
async function safeCreateIssue(teamId: string, title: string) {
  try {
    const payload = await client.issueCreate({
      teamId: teamId,
      title: title
    })

    if (!payload.success) {
      console.error('Issue creation failed')
      return null
    }

    return payload.issue
  } catch (error) {
    console.error('Error creating issue:', error.message)
    throw error
  }
}
```

### Check Success Field

```typescript
async function updateWithCheck(issueId: string) {
  const payload = await client.issueUpdate(issueId, {
    title: 'Updated Title'
  })

  if (payload.success) {
    console.log('Update successful')
    return payload.issue
  } else {
    console.error('Update failed')
    return null
  }
}
```

---

## Advanced Operations

### Batch Operations

```typescript
async function batchCreateIssues(teamId: string, titles: string[]) {
  const promises = titles.map(title =>
    client.issueCreate({
      teamId: teamId,
      title: title
    })
  )

  const results = await Promise.all(promises)

  const created = results.filter(r => r.success)
  console.log(`Created ${created.length}/${titles.length} issues`)

  return created.map(r => r.issue)
}
```

### Complex Relationships

```typescript
async function getCompleteIssueData(issueId: string) {
  const issue = await client.issue(issueId)

  // Fetch all related data
  const [state, assignee, team, comments, attachments, labels, project, cycle] = await Promise.all([
    issue.state,
    issue.assignee,
    issue.team,
    issue.comments(),
    issue.attachments(),
    issue.labels(),
    issue.project,
    issue.cycle
  ])

  return {
    issue,
    state,
    assignee,
    team,
    comments: comments.nodes,
    attachments: attachments.nodes,
    labels: labels.nodes,
    project,
    cycle
  }
}
```

### Conditional Updates

```typescript
async function updateIfAssigned(issueId: string) {
  const issue = await client.issue(issueId)
  const assignee = await issue.assignee

  if (assignee) {
    await client.issueUpdate(issueId, {
      priority: 2 // Increase priority if assigned
    })
  }
}
```

### Archive Completed Issues

```typescript
async function archiveCompletedIssues(teamId: string) {
  const team = await client.team(teamId)
  const states = await client.workflowStates({
    filter: { team: { id: { eq: teamId } } }
  })

  const doneState = states.nodes.find(s => s.name === 'Done')

  if (!doneState) return

  const issues = await client.issues({
    filter: {
      state: { id: { eq: doneState.id } },
      completedAt: { lte: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000) } // 30 days ago
    }
  })

  for (const issue of issues.nodes) {
    await client.issueArchive(issue.id)
    console.log(`Archived: ${issue.identifier}`)
  }
}
```

---

## Real-World Examples

### Daily Standup Report

```typescript
async function generateStandupReport() {
  const me = await client.viewer
  const yesterday = new Date(Date.now() - 24 * 60 * 60 * 1000)

  // Issues completed yesterday
  const completed = await client.issues({
    filter: {
      assignee: { id: { eq: me.id } },
      completedAt: { gte: yesterday }
    }
  })

  // Issues in progress
  const inProgress = await client.issues({
    filter: {
      assignee: { id: { eq: me.id } },
      state: { name: { eq: 'In Progress' } }
    }
  })

  console.log('## Completed Yesterday')
  completed.nodes.forEach(i => console.log(`- ${i.identifier}: ${i.title}`))

  console.log('\n## In Progress')
  inProgress.nodes.forEach(i => console.log(`- ${i.identifier}: ${i.title}`))

  return { completed: completed.nodes, inProgress: inProgress.nodes }
}
```

### Team Velocity Report

```typescript
async function calculateTeamVelocity(teamId: string, days: number = 7) {
  const since = new Date(Date.now() - days * 24 * 60 * 60 * 1000)

  const completed = await client.issues({
    filter: {
      team: { id: { eq: teamId } },
      completedAt: { gte: since }
    }
  })

  let totalEstimate = 0
  for (const issue of completed.nodes) {
    totalEstimate += issue.estimate || 0
  }

  console.log(`Team completed ${completed.nodes.length} issues`)
  console.log(`Total points: ${totalEstimate}`)
  console.log(`Velocity: ${(totalEstimate / days * 7).toFixed(1)} points/week`)

  return {
    issueCount: completed.nodes.length,
    totalPoints: totalEstimate,
    velocityPerWeek: totalEstimate / days * 7
  }
}
```

### Bug Triage

```typescript
async function triageBugs(teamId: string) {
  const labels = await client.issueLabels()
  const bugLabel = labels.nodes.find(l => l.name.toLowerCase() === 'bug')

  if (!bugLabel) {
    console.log('No bug label found')
    return
  }

  const bugs = await client.issues({
    filter: {
      team: { id: { eq: teamId } },
      labels: { id: { eq: bugLabel.id } },
      priority: { eq: 0 } // No priority set
    },
    orderBy: 'createdAt'
  })

  console.log(`Found ${bugs.nodes.length} untriaged bugs`)

  for (const bug of bugs.nodes) {
    console.log(`\n${bug.identifier}: ${bug.title}`)
    console.log(`Created: ${bug.createdAt}`)

    // Auto-prioritize based on keywords
    const desc = (bug.description || '').toLowerCase()
    let priority = 4 // Low

    if (desc.includes('crash') || desc.includes('error')) {
      priority = 2 // High
    } else if (desc.includes('login') || desc.includes('payment')) {
      priority = 1 // Urgent
    }

    await client.issueUpdate(bug.id, { priority })
    console.log(`Set priority to ${priority}`)
  }
}
```

### Sync GitHub Issues to Linear

```typescript
async function syncGitHubIssue(teamId: string, githubIssue: any) {
  const existingIssues = await client.issues({
    filter: {
      team: { id: { eq: teamId } },
      title: { contains: githubIssue.title }
    }
  })

  if (existingIssues.nodes.length > 0) {
    console.log('Issue already exists')
    return existingIssues.nodes[0]
  }

  const labels = await client.issueLabels()
  const githubLabel = labels.nodes.find(l => l.name === 'github')

  const payload = await client.issueCreate({
    teamId: teamId,
    title: githubIssue.title,
    description: `${githubIssue.body}\n\n---\nSource: ${githubIssue.html_url}`,
    labelIds: githubLabel ? [githubLabel.id] : []
  })

  if (payload.success) {
    console.log(`Created Linear issue: ${payload.issue?.identifier}`)
    return payload.issue
  }
}
```

### Automated Sprint Planning

```typescript
async function planSprint(teamId: string, sprintCapacity: number) {
  const states = await client.workflowStates({
    filter: { team: { id: { eq: teamId } } }
  })
  const backlogState = states.nodes.find(s => s.name === 'Backlog')
  const todoState = states.nodes.find(s => s.name === 'Todo')

  if (!backlogState || !todoState) return

  const backlogIssues = await client.issues({
    filter: {
      team: { id: { eq: teamId } },
      state: { id: { eq: backlogState.id } }
    },
    orderBy: 'priority'
  })

  let totalEstimate = 0
  const sprintIssues = []

  for (const issue of backlogIssues.nodes) {
    const estimate = issue.estimate || 1

    if (totalEstimate + estimate <= sprintCapacity) {
      await client.issueUpdate(issue.id, {
        stateId: todoState.id
      })

      sprintIssues.push(issue)
      totalEstimate += estimate

      console.log(`Added to sprint: ${issue.identifier} (${estimate} points)`)
    }

    if (totalEstimate >= sprintCapacity) break
  }

  console.log(`\nSprint planned with ${totalEstimate}/${sprintCapacity} points`)
  return sprintIssues
}
```

---

## Environment Variable Configuration

### Complete Setup Example

```typescript
import { LinearClient } from '@linear/sdk'
import * as dotenv from 'dotenv'

dotenv.config()

interface LinearConfig {
  apiKey?: string
  accessToken?: string
}

function createLinearClient(): LinearClient {
  const config: LinearConfig = {}

  if (process.env.LINEAR_API_KEY) {
    config.apiKey = process.env.LINEAR_API_KEY
  } else if (process.env.LINEAR_ACCESS_TOKEN) {
    config.accessToken = process.env.LINEAR_ACCESS_TOKEN
  } else {
    throw new Error('LINEAR_API_KEY or LINEAR_ACCESS_TOKEN must be set')
  }

  return new LinearClient(config)
}

export const client = createLinearClient()
```

---

## Type Definitions

### Important Types

```typescript
import {
  LinearClient,
  LinearFetch,
  User,
  Issue,
  IssueConnection,
  Team,
  Project,
  Comment,
  WorkflowState,
  IssueLabel
} from '@linear/sdk'

// LinearFetch is a Promise-like type
const user: LinearFetch<User> = client.viewer

// Connections have nodes and pageInfo
const issues: LinearFetch<IssueConnection> = client.issues()

// Access the actual data
const issuesData = await issues
const firstIssue: Issue = issuesData.nodes[0]
```

### Custom Types

```typescript
interface IssueWithDetails {
  issue: Issue
  state: WorkflowState | null
  assignee: User | null
  team: Team
  labels: IssueLabel[]
}

async function getIssueWithDetails(id: string): Promise<IssueWithDetails> {
  const issue = await client.issue(id)

  return {
    issue,
    state: await issue.state,
    assignee: await issue.assignee,
    team: await issue.team,
    labels: (await issue.labels()).nodes
  }
}
```

---

## Priority Values

```typescript
// Priority mapping
const PRIORITY = {
  NONE: 0,
  URGENT: 1,
  HIGH: 2,
  MEDIUM: 3,
  LOW: 4
}

// Usage
await client.issueCreate({
  teamId: teamId,
  title: 'Critical bug',
  priority: PRIORITY.URGENT
})
```

---

## GraphQL Direct Queries (Advanced)

### Raw GraphQL Query

```typescript
async function rawGraphQL() {
  const query = `
    query {
      viewer {
        id
        name
        assignedIssues(first: 10) {
          nodes {
            id
            title
            state {
              name
            }
          }
        }
      }
    }
  `

  // The SDK wraps the GraphQL API, but you can access raw client if needed
  // For most use cases, use the typed SDK methods instead
}
```

---

## Complete Application Example

```typescript
import { LinearClient } from '@linear/sdk'
import * as dotenv from 'dotenv'

dotenv.config()

const client = new LinearClient({
  apiKey: process.env.LINEAR_API_KEY
})

async function main() {
  try {
    // Get current user
    const me = await client.viewer
    console.log(`Logged in as: ${me.displayName}`)

    // Get teams
    const teams = await client.teams()
    const myTeam = teams.nodes[0]
    console.log(`Working with team: ${myTeam.name}`)

    // Create an issue
    const issuePayload = await client.issueCreate({
      teamId: myTeam.id,
      title: 'Test issue from SDK',
      description: 'This is a test issue created via the Linear SDK',
      priority: 3
    })

    if (issuePayload.success && issuePayload.issue) {
      const issue = issuePayload.issue
      console.log(`Created issue: ${issue.identifier}`)

      // Add a comment
      const commentPayload = await client.commentCreate({
        issueId: issue.id,
        body: 'First comment on this issue!'
      })

      if (commentPayload.success) {
        console.log('Comment added')
      }

      // Get workflow states
      const states = await client.workflowStates({
        filter: { team: { id: { eq: myTeam.id } } }
      })

      const inProgressState = states.nodes.find(s => s.name === 'In Progress')

      if (inProgressState) {
        // Update issue state
        const updatePayload = await client.issueUpdate(issue.id, {
          stateId: inProgressState.id
        })

        if (updatePayload.success) {
          console.log('Issue moved to In Progress')
        }
      }

      // Get updated issue data
      const updatedIssue = await client.issue(issue.id)
      const currentState = await updatedIssue.state
      console.log(`Current state: ${currentState?.name}`)

      // Get all comments
      const comments = await updatedIssue.comments()
      console.log(`Issue has ${comments.nodes.length} comment(s)`)
    }

  } catch (error) {
    console.error('Error:', error.message)
  }
}

main()
```
