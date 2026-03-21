---
name: workspace-api
description: "Notion Python SDK for interacting with Notion workspaces, pages, and databases via the official API."
metadata:
  languages: "python"
  versions: "2.6.0"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "notion,api,workspace,pages,databases"
---

# Notion API Python SDK Coding Guidelines

You are a Notion API coding expert. Help me with writing code using the Notion API calling the official libraries and SDKs.

You can find the official SDK documentation and code samples here:
https://developers.notion.com/reference/
https://ramnes.github.io/notion-sdk-py/

## Golden Rule: Use the Correct and Current SDK

Always use the Notion Python SDK to interact with Notion workspaces, which is the standard library for all Notion API interactions. Do not use legacy libraries or unofficial SDKs.

- **Library Name:** Notion Python SDK
- **PyPI Package:** `notion-client`
- **Legacy Libraries**: `notion-py`, `notion`, `notion-sdk`, and other unofficial packages are not recommended

**Installation:**

- **Correct:** `pip install notion-client`

**APIs and Usage:**

- **Correct:** `from notion_client import Client`
- **Correct:** `notion = Client(auth=os.environ["NOTION_TOKEN"])`
- **Correct:** `notion.pages.create(...)`
- **Correct:** `notion.databases.query(...)`
- **Correct:** `notion.data_sources.query(...)`
- **Incorrect:** `NotionClient` or `NotionAPI` from other packages
- **Incorrect:** Legacy notion-py APIs

## Installation

Install the Notion Python SDK using pip:

```bash
pip install notion-client
```

Set your environment variable for the Notion API token:

```bash
export NOTION_TOKEN="your_integration_token_here"
```

You can obtain an integration token by creating a new integration at:
https://www.notion.com/my-integrations

## Initialization and API Key

The `notion-client` library requires creating a `Client` instance for all API calls.

- Always use `notion = Client(auth=token)` to create an instance.
- Set the `NOTION_TOKEN` environment variable for security.

```python
import os
from notion_client import Client

# Uses the NOTION_TOKEN environment variable
notion = Client(auth=os.environ["NOTION_TOKEN"])
```

### Client Configuration Options

```python
from notion_client import Client

notion = Client(
    auth=os.environ["NOTION_TOKEN"],
    log_level="DEBUG",  # Logging verbosity: DEBUG, INFO, WARNING, ERROR
    timeout_ms=60000,   # Request timeout in milliseconds (default: 60000)
)
```

### Async Client

The SDK supports async/await operations:

```python
import os
from notion_client import AsyncClient

async_notion = AsyncClient(auth=os.environ["NOTION_TOKEN"])

# Use with async/await
async def get_users():
    response = await async_notion.users.list()
    return response
```

## Important API Version Changes (2025-09-03)

As of API version 2025-09-03, Notion separated databases and data sources:

- **Databases** parent one or more data sources
- **Data Sources** each parent zero or more pages
- Use `notion.data_sources.query()` instead of `notion.databases.query()` for newer implementations

## Pages API

### Create a Page

Create a new page as a child of an existing page:

```python
from notion_client import Client
import os

notion = Client(auth=os.environ["NOTION_TOKEN"])

def create_page():
    response = notion.pages.create(
        parent={
            "page_id": "494c87d072c44cf6960f55f8427f7692"
        },
        properties={
            "title": {
                "type": "title",
                "title": [
                    {
                        "type": "text",
                        "text": {"content": "A note from your pals at Notion"}
                    }
                ]
            }
        }
    )

    return response

page = create_page()
print(page)
```

### Create a Page in a Data Source (Database)

```python
def create_page_in_data_source():
    response = notion.pages.create(
        parent={
            "data_source_id": "897e5a76-ae52-4b48-9fdf-e71f5945d1af"
        },
        properties={
            "Name": {
                "title": [
                    {
                        "text": {
                            "content": "Golden Gate Bridge"
                        }
                    }
                ]
            },
            "Tags": {
                "multi_select": [
                    {"name": "Landmark"},
                    {"name": "San Francisco"}
                ]
            },
            "Status": {
                "status": {
                    "name": "Active"
                }
            }
        }
    )

    return response
```

### Create a Page with Content Blocks

```python
def create_page_with_content():
    response = notion.pages.create(
        parent={
            "page_id": "494c87d072c44cf6960f55f8427f7692"
        },
        properties={
            "title": {
                "title": [{"text": {"content": "My New Page"}}]
            }
        },
        children=[
            {
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [{"type": "text", "text": {"content": "Introduction"}}]
                }
            },
            {
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {"content": "This is a paragraph with "}
                        },
                        {
                            "type": "text",
                            "text": {"content": "bold text"},
                            "annotations": {"bold": True}
                        }
                    ]
                }
            },
            {
                "object": "block",
                "type": "bulleted_list_item",
                "bulleted_list_item": {
                    "rich_text": [{"type": "text", "text": {"content": "First bullet point"}}]
                }
            }
        ]
    )

    return response
```

### Retrieve a Page

```python
def retrieve_page(page_id):
    response = notion.pages.retrieve(page_id=page_id)

    print("Page title:", response["properties"]["title"])
    print("Created at:", response["created_time"])
    print("Last edited:", response["last_edited_time"])

    return response
```

### Update Page Properties

```python
def update_page(page_id):
    response = notion.pages.update(
        page_id=page_id,
        properties={
            "Status": {
                "status": {
                    "name": "Completed"
                }
            },
            "Tags": {
                "multi_select": [
                    {"name": "Updated"},
                    {"name": "Important"}
                ]
            }
        }
    )

    return response
```

### Archive (Trash) a Page

```python
def archive_page(page_id):
    response = notion.pages.update(
        page_id=page_id,
        archived=True
    )

    return response
```

## Databases API

### Query a Database

```python
def query_database(database_id):
    response = notion.databases.query(database_id=database_id)

    print("Results:", response["results"])
    return response
```

### Query with Filters and Sorting

```python
def query_database_with_filters(database_id):
    response = notion.databases.query(
        database_id=database_id,
        filter={
            "and": [
                {
                    "property": "Status",
                    "status": {
                        "equals": "Active"
                    }
                },
                {
                    "property": "Priority",
                    "select": {
                        "equals": "High"
                    }
                }
            ]
        },
        sorts=[
            {
                "property": "Created",
                "direction": "descending"
            }
        ]
    )

    return response
```

### Query Data Source (API Version 2025-09-03+)

```python
def query_data_source(data_source_id):
    response = notion.data_sources.query(
        data_source_id=data_source_id,
        filter={
            "property": "Landmark",
            "rich_text": {
                "contains": "Bridge"
            }
        }
    )

    return response
```

### Filter Examples

```python
# Text property contains value
text_filter = {
    "property": "Description",
    "rich_text": {"contains": "important"}
}

# Checkbox equals value
checkbox_filter = {
    "property": "Done",
    "checkbox": {"equals": True}
}

# Date is after specific date
date_filter = {
    "property": "Due Date",
    "date": {"after": "2025-01-01"}
}

# Number greater than value
number_filter = {
    "property": "Score",
    "number": {"greater_than": 50}
}

# Multi-select contains value
multi_select_filter = {
    "property": "Tags",
    "multi_select": {"contains": "urgent"}
}

# Compound OR filter
or_filter = {
    "or": [
        {"property": "Tags", "multi_select": {"contains": "A"}},
        {"property": "Tags", "multi_select": {"contains": "B"}}
    ]
}

# Compound AND filter
and_filter = {
    "and": [
        {"property": "Done", "checkbox": {"equals": False}},
        {"property": "Priority", "select": {"equals": "High"}}
    ]
}
```

### Retrieve a Database

```python
def retrieve_database(database_id):
    response = notion.databases.retrieve(database_id=database_id)

    print("Database title:", response["title"])
    print("Properties:", response["properties"])

    return response
```

### Create a Database

```python
def create_database(parent_page_id):
    response = notion.databases.create(
        parent={
            "type": "page_id",
            "page_id": parent_page_id
        },
        title=[
            {
                "type": "text",
                "text": {
                    "content": "Task List"
                }
            }
        ],
        properties={
            "Name": {
                "title": {}
            },
            "Status": {
                "status": {
                    "options": [
                        {"name": "Not started", "color": "red"},
                        {"name": "In progress", "color": "yellow"},
                        {"name": "Completed", "color": "green"}
                    ]
                }
            },
            "Priority": {
                "select": {
                    "options": [
                        {"name": "High", "color": "red"},
                        {"name": "Medium", "color": "yellow"},
                        {"name": "Low", "color": "gray"}
                    ]
                }
            },
            "Tags": {
                "multi_select": {
                    "options": [
                        {"name": "Bug", "color": "red"},
                        {"name": "Feature", "color": "blue"}
                    ]
                }
            },
            "Due Date": {
                "date": {}
            }
        }
    )

    return response
```

### Update Database Properties

```python
def update_database(database_id):
    response = notion.databases.update(
        database_id=database_id,
        title=[
            {
                "text": {
                    "content": "Updated Task List"
                }
            }
        ],
        properties={
            "Status": {
                "status": {
                    "options": [
                        {"name": "To Do", "color": "red"},
                        {"name": "Doing", "color": "yellow"},
                        {"name": "Done", "color": "green"}
                    ]
                }
            }
        }
    )

    return response
```

## Blocks API

### Retrieve Block Children

```python
def get_block_children(block_id):
    response = notion.blocks.children.list(
        block_id=block_id,
        page_size=50
    )

    print("Number of blocks:", len(response["results"]))
    return response["results"]
```

### Append Block Children

```python
def append_block_children(block_id):
    response = notion.blocks.children.append(
        block_id=block_id,
        children=[
            {
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {"content": "New Section"}
                        }
                    ]
                }
            },
            {
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {"content": "This is a new paragraph."}
                        }
                    ]
                }
            },
            {
                "object": "block",
                "type": "to_do",
                "to_do": {
                    "rich_text": [{"type": "text", "text": {"content": "Complete this task"}}],
                    "checked": False
                }
            }
        ]
    )

    return response
```

### Common Block Types

```python
# Heading blocks
heading_1 = {
    "type": "heading_1",
    "heading_1": {
        "rich_text": [{"text": {"content": "Heading 1"}}]
    }
}

heading_2 = {
    "type": "heading_2",
    "heading_2": {
        "rich_text": [{"text": {"content": "Heading 2"}}]
    }
}

heading_3 = {
    "type": "heading_3",
    "heading_3": {
        "rich_text": [{"text": {"content": "Heading 3"}}]
    }
}

# Paragraph with formatting
paragraph = {
    "type": "paragraph",
    "paragraph": {
        "rich_text": [
            {"text": {"content": "Normal text "}},
            {"text": {"content": "bold"}, "annotations": {"bold": True}},
            {"text": {"content": " and "}},
            {"text": {"content": "italic"}, "annotations": {"italic": True}},
            {"text": {"content": " and "}},
            {"text": {"content": "code"}, "annotations": {"code": True}}
        ]
    }
}

# Bulleted list item
bulleted_list_item = {
    "type": "bulleted_list_item",
    "bulleted_list_item": {
        "rich_text": [{"text": {"content": "Bullet point"}}]
    }
}

# Numbered list item
numbered_list_item = {
    "type": "numbered_list_item",
    "numbered_list_item": {
        "rich_text": [{"text": {"content": "Numbered item"}}]
    }
}

# To-do item
to_do = {
    "type": "to_do",
    "to_do": {
        "rich_text": [{"text": {"content": "Task to complete"}}],
        "checked": False
    }
}

# Toggle block (collapsible)
toggle = {
    "type": "toggle",
    "toggle": {
        "rich_text": [{"text": {"content": "Click to expand"}}]
    }
}

# Quote
quote = {
    "type": "quote",
    "quote": {
        "rich_text": [{"text": {"content": "This is a quote"}}]
    }
}

# Callout
callout = {
    "type": "callout",
    "callout": {
        "rich_text": [{"text": {"content": "Important note"}}],
        "icon": {"emoji": "💡"}
    }
}

# Code block
code_block = {
    "type": "code",
    "code": {
        "rich_text": [{"text": {"content": "x = 42"}}],
        "language": "python"
    }
}

# Divider
divider = {
    "type": "divider",
    "divider": {}
}

# Image
image = {
    "type": "image",
    "image": {
        "type": "external",
        "external": {
            "url": "https://example.com/image.png"
        }
    }
}

# Bookmark
bookmark = {
    "type": "bookmark",
    "bookmark": {
        "url": "https://notion.so"
    }
}
```

### Update a Block

```python
def update_block(block_id):
    response = notion.blocks.update(
        block_id=block_id,
        paragraph={
            "rich_text": [
                {
                    "text": {
                        "content": "Updated paragraph content"
                    }
                }
            ]
        }
    )

    return response
```

### Delete a Block

```python
def delete_block(block_id):
    response = notion.blocks.delete(block_id=block_id)
    return response
```

### Retrieve a Block

```python
def retrieve_block(block_id):
    response = notion.blocks.retrieve(block_id=block_id)

    print("Block type:", response["type"])
    return response
```

## Users API

### List All Users

```python
def list_users():
    response = notion.users.list()

    print("Users:", response["results"])
    return response["results"]
```

### Retrieve a User

```python
def retrieve_user(user_id):
    response = notion.users.retrieve(user_id=user_id)

    print("User name:", response["name"])
    print("User type:", response["type"])

    return response
```

### Retrieve Bot User (Your Integration)

```python
def retrieve_bot_user():
    response = notion.users.me()

    print("Bot name:", response["name"])
    print("Bot ID:", response["id"])

    return response
```

## Search API

### Basic Search

```python
def search_pages(query):
    response = notion.search(
        query=query,
        filter={
            "value": "page",
            "property": "object"
        },
        sort={
            "direction": "descending",
            "timestamp": "last_edited_time"
        }
    )

    return response["results"]
```

### Search for Databases

```python
def search_databases(query):
    response = notion.search(
        query=query,
        filter={
            "value": "database",
            "property": "object"
        }
    )

    return response["results"]
```

### Search All Content

```python
def search_all(query):
    response = notion.search(
        query=query,
        page_size=100
    )

    print(f"Found {len(response['results'])} results")
    return response["results"]
```

## Comments API

### Create a Comment

```python
def create_comment(page_id):
    response = notion.comments.create(
        parent={
            "page_id": page_id
        },
        rich_text=[
            {
                "text": {
                    "content": "This is a comment on the page."
                }
            }
        ]
    )

    return response
```

### Create a Discussion Comment (Reply)

```python
def create_discussion_comment(discussion_id):
    response = notion.comments.create(
        discussion_id=discussion_id,
        rich_text=[
            {
                "text": {
                    "content": "This is a reply to an existing comment."
                }
            }
        ]
    )

    return response
```

### List Comments

```python
def list_comments(block_id):
    response = notion.comments.list(block_id=block_id)

    print("Comments:", response["results"])
    return response["results"]
```

## Pagination

### Manual Pagination

```python
def get_all_pages(database_id):
    has_more = True
    start_cursor = None
    all_results = []

    while has_more:
        response = notion.databases.query(
            database_id=database_id,
            start_cursor=start_cursor,
            page_size=100
        )

        all_results.extend(response["results"])
        has_more = response["has_more"]
        start_cursor = response.get("next_cursor")

    print(f"Retrieved {len(all_results)} total results")
    return all_results
```

### Using Pagination Helpers

```python
from notion_client import iterate_paginated_api, collect_paginated_api

# Iterate through paginated results
def iterate_blocks(block_id):
    for block in iterate_paginated_api(
        notion.blocks.children.list,
        block_id=block_id
    ):
        print("Block type:", block["type"])

# Collect all paginated results at once
def collect_all_blocks(block_id):
    blocks = collect_paginated_api(
        notion.blocks.children.list,
        block_id=block_id
    )

    print(f"Total blocks: {len(blocks)}")
    return blocks
```

### Async Pagination Helpers

```python
from notion_client import AsyncClient, async_iterate_paginated_api, async_collect_paginated_api

async_notion = AsyncClient(auth=os.environ["NOTION_TOKEN"])

# Async iteration
async def async_iterate_blocks(block_id):
    async for block in async_iterate_paginated_api(
        async_notion.blocks.children.list,
        block_id=block_id
    ):
        print("Block type:", block["type"])

# Async collection
async def async_collect_blocks(block_id):
    blocks = await async_collect_paginated_api(
        async_notion.blocks.children.list,
        block_id=block_id
    )
    return blocks
```

## Error Handling

### Basic Error Handling

```python
from notion_client import Client, APIResponseError

notion = Client(auth=os.environ["NOTION_TOKEN"])

def handle_errors():
    try:
        response = notion.pages.retrieve(page_id="invalid-id")
    except APIResponseError as error:
        if error.code == "object_not_found":
            print("Page not found")
        elif error.code == "unauthorized":
            print("Invalid API token")
        elif error.code == "restricted_resource":
            print("Access denied - integration not shared with this resource")
        elif error.code == "rate_limited":
            print("Rate limit exceeded")
        else:
            print(f"Error: {error.message}")
```

### Advanced Error Handling

```python
from notion_client import Client, APIResponseError, RequestTimeoutError

def advanced_error_handling():
    try:
        response = notion.pages.retrieve(page_id="some-id")
    except RequestTimeoutError:
        print("Request timed out")
    except APIResponseError as error:
        print(f"API Error: {error.code}")
        print(f"Status: {error.status}")
        print(f"Message: {error.message}")

        # Access full response
        if hasattr(error, 'response'):
            print(f"Full response: {error.response}")
    except Exception as error:
        print(f"Unexpected error: {error}")
```

### Common Error Codes

| Error Code | Description |
|------------|-------------|
| `object_not_found` | The requested resource doesn't exist |
| `unauthorized` | Invalid or missing API token |
| `restricted_resource` | Integration doesn't have access to the resource |
| `rate_limited` | Too many requests, retry after delay |
| `validation_error` | Invalid request parameters |
| `conflict_error` | Resource conflict (e.g., duplicate) |

## Advanced Features

### Rich Text Formatting

```python
rich_text = [
    {
        "type": "text",
        "text": {"content": "Plain text "}
    },
    {
        "type": "text",
        "text": {"content": "bold text"},
        "annotations": {"bold": True}
    },
    {
        "type": "text",
        "text": {"content": " "}
    },
    {
        "type": "text",
        "text": {"content": "italic text"},
        "annotations": {"italic": True}
    },
    {
        "type": "text",
        "text": {"content": " "}
    },
    {
        "type": "text",
        "text": {"content": "strikethrough"},
        "annotations": {"strikethrough": True}
    },
    {
        "type": "text",
        "text": {"content": " "}
    },
    {
        "type": "text",
        "text": {"content": "underline"},
        "annotations": {"underline": True}
    },
    {
        "type": "text",
        "text": {"content": " "}
    },
    {
        "type": "text",
        "text": {"content": "code"},
        "annotations": {"code": True}
    },
    {
        "type": "text",
        "text": {"content": " "}
    },
    {
        "type": "text",
        "text": {"content": "colored text"},
        "annotations": {"color": "red"}
    },
    {
        "type": "text",
        "text": {"content": " "}
    },
    {
        "type": "text",
        "text": {"content": "link text", "link": {"url": "https://notion.so"}}
    }
]
```

### Property Types

```python
# Title property
title_property = {
    "title": [
        {
            "text": {"content": "Page Title"}
        }
    ]
}

# Rich text property
rich_text_property = {
    "rich_text": [
        {
            "text": {"content": "Some text content"}
        }
    ]
}

# Number property
number_property = {
    "number": 42
}

# Select property
select_property = {
    "select": {
        "name": "Option 1"
    }
}

# Multi-select property
multi_select_property = {
    "multi_select": [
        {"name": "Tag 1"},
        {"name": "Tag 2"}
    ]
}

# Date property
date_property = {
    "date": {
        "start": "2025-01-01",
        "end": "2025-01-31"
    }
}

# Checkbox property
checkbox_property = {
    "checkbox": True
}

# URL property
url_property = {
    "url": "https://notion.so"
}

# Email property
email_property = {
    "email": "user@example.com"
}

# Phone number property
phone_property = {
    "phone_number": "+1-555-0100"
}

# Status property
status_property = {
    "status": {
        "name": "In Progress"
    }
}

# People property
people_property = {
    "people": [
        {
            "object": "user",
            "id": "user-id-here"
        }
    ]
}

# Files property
files_property = {
    "files": [
        {
            "name": "Document.pdf",
            "type": "external",
            "external": {
                "url": "https://example.com/document.pdf"
            }
        }
    ]
}

# Relation property
relation_property = {
    "relation": [
        {
            "id": "related-page-id"
        }
    ]
}
```

## Complete Example: Task Management System

```python
import os
from notion_client import Client

notion = Client(auth=os.environ["NOTION_TOKEN"])

def create_task_management_system():
    # 1. Create a database for tasks
    database = notion.databases.create(
        parent={
            "type": "page_id",
            "page_id": os.environ["PARENT_PAGE_ID"]
        },
        title=[
            {
                "type": "text",
                "text": {"content": "Task Manager"}
            }
        ],
        properties={
            "Name": {
                "title": {}
            },
            "Status": {
                "status": {
                    "options": [
                        {"name": "Not started", "color": "red"},
                        {"name": "In progress", "color": "yellow"},
                        {"name": "Completed", "color": "green"}
                    ]
                }
            },
            "Priority": {
                "select": {
                    "options": [
                        {"name": "High", "color": "red"},
                        {"name": "Medium", "color": "yellow"},
                        {"name": "Low", "color": "gray"}
                    ]
                }
            },
            "Due Date": {
                "date": {}
            },
            "Tags": {
                "multi_select": {
                    "options": [
                        {"name": "Bug", "color": "red"},
                        {"name": "Feature", "color": "blue"},
                        {"name": "Documentation", "color": "purple"}
                    ]
                }
            },
            "Assignee": {
                "people": {}
            }
        }
    )

    print(f"Database created: {database['id']}")

    # 2. Create a task
    task = notion.pages.create(
        parent={
            "database_id": database["id"]
        },
        properties={
            "Name": {
                "title": [
                    {
                        "text": {"content": "Implement user authentication"}
                    }
                ]
            },
            "Status": {
                "status": {"name": "In progress"}
            },
            "Priority": {
                "select": {"name": "High"}
            },
            "Due Date": {
                "date": {"start": "2025-02-01"}
            },
            "Tags": {
                "multi_select": [{"name": "Feature"}]
            }
        }
    )

    print(f"Task created: {task['id']}")

    # 3. Add content to the task
    notion.blocks.children.append(
        block_id=task["id"],
        children=[
            {
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [{"text": {"content": "Task Details"}}]
                }
            },
            {
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [
                        {
                            "text": {
                                "content": "Implement OAuth 2.0 authentication for user login."
                            }
                        }
                    ]
                }
            },
            {
                "object": "block",
                "type": "heading_3",
                "heading_3": {
                    "rich_text": [{"text": {"content": "Requirements"}}]
                }
            },
            {
                "object": "block",
                "type": "to_do",
                "to_do": {
                    "rich_text": [{"text": {"content": "Set up OAuth provider"}}],
                    "checked": False
                }
            },
            {
                "object": "block",
                "type": "to_do",
                "to_do": {
                    "rich_text": [{"text": {"content": "Create login page"}}],
                    "checked": False
                }
            },
            {
                "object": "block",
                "type": "to_do",
                "to_do": {
                    "rich_text": [{"text": {"content": "Test authentication flow"}}],
                    "checked": False
                }
            }
        ]
    )

    # 4. Add a comment
    notion.comments.create(
        parent={"page_id": task["id"]},
        rich_text=[
            {
                "text": {"content": "Starting work on this task today."}
            }
        ]
    )

    # 5. Query all high priority tasks
    high_priority_tasks = notion.databases.query(
        database_id=database["id"],
        filter={
            "property": "Priority",
            "select": {
                "equals": "High"
            }
        },
        sorts=[
            {
                "property": "Due Date",
                "direction": "ascending"
            }
        ]
    )

    print(f"High priority tasks: {len(high_priority_tasks['results'])}")

    # 6. Update task status
    notion.pages.update(
        page_id=task["id"],
        properties={
            "Status": {
                "status": {"name": "Completed"}
            }
        }
    )

    print("Task marked as completed")

# Run the example
if __name__ == "__main__":
    create_task_management_system()
```

## Async Example

```python
import os
import asyncio
from notion_client import AsyncClient

async_notion = AsyncClient(auth=os.environ["NOTION_TOKEN"])

async def async_example():
    # List users
    users = await async_notion.users.list()
    print(f"Found {len(users['results'])} users")

    # Create a page
    page = await async_notion.pages.create(
        parent={"page_id": "parent-page-id"},
        properties={
            "title": {
                "title": [{"text": {"content": "Async Page"}}]
            }
        }
    )

    # Query a database
    results = await async_notion.databases.query(
        database_id="database-id"
    )

    return results

# Run async code
if __name__ == "__main__":
    asyncio.run(async_example())
```

## Useful Links

- Documentation: https://developers.notion.com/
- API Reference: https://developers.notion.com/reference/
- SDK Repository: https://github.com/ramnes/notion-sdk-py
- SDK Documentation: https://ramnes.github.io/notion-sdk-py/
- PyPI Package: https://pypi.org/project/notion-client/
- Create Integration: https://www.notion.com/my-integrations
- Community: Notion Developers Slack
