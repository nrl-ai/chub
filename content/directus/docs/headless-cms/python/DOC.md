---
name: headless-cms
description: "Directus Python SDK coding guidelines for interacting with Directus projects using the py-directus library"
metadata:
  languages: "python"
  versions: "0.0.30"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "directus,headless-cms,api,content,backend"
---

# Directus Python SDK Coding Guidelines

You are a Directus Python SDK coding expert. Help me with writing code using the Directus Python SDK calling the official libraries.

You can find the official SDK documentation and code samples here:
https://panos-stavrianos.github.io/py-directus/

## Golden Rule: Use the Correct and Current SDK

Always use the py-directus library to interact with Directus projects in Python. This is a community-maintained async Python wrapper for the Directus API. Do not use outdated or unmaintained libraries.

- **Library Name:** py-directus
- **PyPI Package:** `py-directus`
- **Alternative Libraries:** `directus-sdk-py`, `pydirectus-sdk` (less maintained)

**Installation:**

- **Correct:** `pip install py-directus`
- **Incorrect:** `pip install directus`
- **Incorrect:** `pip install directus-python`

**APIs and Usage:**

- **Correct:** `from py_directus import Directus`
- **Correct:** `directus = await Directus(url, email, password)`
- **Correct:** `await directus.collection("articles").read()`
- **Incorrect:** `DirectusClient` or `DirectusAPI`
- **Incorrect:** Synchronous API calls without `await`

## Installation

Install py-directus using pip:

```bash
pip install py-directus
```

## Initialization and Authentication

The `py-directus` library is fully asynchronous and requires using `async`/`await` syntax. The library requires creating a Directus instance with authentication.

### Authentication with Email and Password

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        email="user@example.com",
        password="your-password"
    )

    # Use directus client
    # ...

# Run the async function
import asyncio
asyncio.run(main())
```

### Authentication with Static Token

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-static-token"
    )

    # Use directus client

import asyncio
asyncio.run(main())
```

### Using Context Manager

```python
from py_directus import Directus

async def main():
    async with await Directus(
        url="https://your-directus-instance.com",
        email="user@example.com",
        password="your-password"
    ) as directus:
        # Use directus client
        users = await directus.collection("directus_users").read()

import asyncio
asyncio.run(main())
```

### With Environment Variables

```python
import os
from py_directus import Directus

async def main():
    directus = await Directus(
        url=os.getenv("DIRECTUS_URL"),
        token=os.getenv("DIRECTUS_TOKEN")
    )

    # Use directus client

import asyncio
asyncio.run(main())
```

## Reading Items from Collections

### Read All Items

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Read all items from a collection
    response = await directus.collection("articles").read()
    articles = response.items

    for article in articles:
        print(article)

import asyncio
asyncio.run(main())
```

### Read Single Item

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Read single item by ID
    response = await directus.collection("articles").read(id="1")
    article = response.item

    print(article)

import asyncio
asyncio.run(main())
```

### Read with Field Selection

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Select specific fields
    response = await directus.collection("articles").fields("id", "title", "author.name").read()
    articles = response.items

import asyncio
asyncio.run(main())
```

### Read with Limit and Offset

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Pagination
    response = await directus.collection("articles").limit(10).offset(20).read()
    articles = response.items

import asyncio
asyncio.run(main())
```

## Filtering Items

### Simple Filter

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Filter by field value
    response = await directus.collection("articles").filter(status="published").read()
    articles = response.items

import asyncio
asyncio.run(main())
```

### Multiple Filters (AND)

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Multiple filters are combined with AND
    response = await (
        directus.collection("articles")
        .filter(status="published")
        .filter(views__gte=1000)
        .read()
    )
    articles = response.items

import asyncio
asyncio.run(main())
```

### Complex Filters with F Object

```python
from py_directus import Directus, F

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # OR conditions
    response = await (
        directus.collection("users")
        .filter(F(first_name="John") | F(first_name="Jane"))
        .read()
    )

    # Complex nested conditions
    response = await (
        directus.collection("articles")
        .filter(
            (F(status="published") | F(status="archived"))
            & F(views__gte=100)
        )
        .read()
    )
    articles = response.items

import asyncio
asyncio.run(main())
```

### Filter Operators

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Greater than
    response = await directus.collection("articles").filter(views__gt=1000).read()

    # Greater than or equal
    response = await directus.collection("articles").filter(views__gte=1000).read()

    # Less than
    response = await directus.collection("articles").filter(views__lt=1000).read()

    # Less than or equal
    response = await directus.collection("articles").filter(views__lte=1000).read()

    # Not equal
    response = await directus.collection("articles").filter(status__neq="draft").read()

    # Contains
    response = await directus.collection("articles").filter(title__contains="directus").read()

    # In list
    response = await directus.collection("articles").filter(status__in=["published", "archived"]).read()

    # Not in list
    response = await directus.collection("articles").filter(status__nin=["draft", "deleted"]).read()

    # Is null
    response = await directus.collection("articles").filter(author__null=True).read()

    # Is not null
    response = await directus.collection("articles").filter(author__nnull=True).read()

import asyncio
asyncio.run(main())
```

## Sorting Items

### Single Sort

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Sort ascending
    response = await directus.collection("articles").sort("title").read()

    # Sort descending
    response = await directus.collection("articles").sort("-publish_date").read()

import asyncio
asyncio.run(main())
```

### Multiple Sorts

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Multiple sort fields
    response = await directus.collection("articles").sort("-publish_date", "title").read()

import asyncio
asyncio.run(main())
```

## Searching Items

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Full-text search
    response = await directus.collection("articles").search("directus cms").read()
    articles = response.items

import asyncio
asyncio.run(main())
```

## Creating Items

### Create Single Item

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Create a new item
    new_article = {
        "title": "New Article",
        "content": "Article content here",
        "status": "draft"
    }

    response = await directus.collection("articles").create(new_article)
    created_article = response.item

    print(f"Created article with ID: {created_article['id']}")

import asyncio
asyncio.run(main())
```

### Create Multiple Items

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Create multiple items
    new_articles = [
        {"title": "Article 1", "status": "draft"},
        {"title": "Article 2", "status": "draft"},
        {"title": "Article 3", "status": "published"}
    ]

    response = await directus.collection("articles").create(new_articles)
    created_articles = response.items

import asyncio
asyncio.run(main())
```

## Updating Items

### Update Single Item

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Update an item by ID
    updated_data = {
        "status": "published",
        "publish_date": "2024-01-15"
    }

    response = await directus.collection("articles").update("1", updated_data)
    updated_article = response.item

import asyncio
asyncio.run(main())
```

### Update Multiple Items

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Update multiple items by IDs
    item_ids = ["1", "2", "3"]
    updated_data = {"status": "published"}

    response = await directus.collection("articles").update(item_ids, updated_data)
    updated_articles = response.items

import asyncio
asyncio.run(main())
```

## Deleting Items

### Delete Single Item

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Delete an item by ID
    await directus.collection("articles").delete("1")

import asyncio
asyncio.run(main())
```

### Delete Multiple Items

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Delete multiple items by IDs
    await directus.collection("articles").delete(["1", "2", "3"])

import asyncio
asyncio.run(main())
```

## Aggregation

### Count Items

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Count all items
    response = await directus.collection("articles").aggregate(count="id").read()
    total_count = response.items[0]["count"]["id"]

    print(f"Total articles: {total_count}")

import asyncio
asyncio.run(main())
```

### Count Distinct

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Count distinct values
    response = await directus.collection("articles").aggregate(countDistinct="author").read()

import asyncio
asyncio.run(main())
```

### Sum, Average, Min, Max

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Sum
    response = await directus.collection("articles").aggregate(sum="views").read()
    total_views = response.items[0]["sum"]["views"]

    # Average
    response = await directus.collection("articles").aggregate(avg="views").read()
    avg_views = response.items[0]["avg"]["views"]

    # Min
    response = await directus.collection("articles").aggregate(min="views").read()
    min_views = response.items[0]["min"]["views"]

    # Max
    response = await directus.collection("articles").aggregate(max="views").read()
    max_views = response.items[0]["max"]["views"]

import asyncio
asyncio.run(main())
```

### Multiple Aggregations

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Chain multiple aggregations
    response = await (
        directus.collection("articles")
        .aggregate(count="id")
        .aggregate(sum="views")
        .aggregate(avg="views")
        .read()
    )

    result = response.items[0]
    print(f"Count: {result['count']['id']}")
    print(f"Total views: {result['sum']['views']}")
    print(f"Average views: {result['avg']['views']}")

import asyncio
asyncio.run(main())
```

### Aggregation with Agg Class

```python
from py_directus import Directus, Agg, AggregationOperators

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Using Agg class for aggregations
    sum_agg = Agg(operator=AggregationOperators.Sum, fields="amount")
    avg_agg = Agg(operator=AggregationOperators.Avg, fields="amount")

    response = await (
        directus.collection("transactions")
        .aggregate(sum_agg)
        .aggregate(avg_agg)
        .read()
    )

import asyncio
asyncio.run(main())
```

### Group By

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Group by single field
    response = await (
        directus.collection("articles")
        .aggregate(count="id")
        .groupby("status")
        .read()
    )

    # Group by multiple fields
    response = await (
        directus.collection("articles")
        .aggregate(count="id")
        .aggregate(avg="views")
        .groupby("status", "author")
        .read()
    )

import asyncio
asyncio.run(main())
```

## Working with Pydantic Models

### Using Built-in Models

```python
from py_directus import Directus, DirectusUser

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Using DirectusUser model
    response = await (
        directus.collection(DirectusUser)
        .filter(first_name="John")
        .read()
    )

    # Response items are typed as DirectusUser objects
    user = response.item
    print(user.email)
    print(user.first_name)
    print(user.last_name)

import asyncio
asyncio.run(main())
```

### Creating Custom Pydantic Models

```python
from py_directus import Directus
from pydantic import BaseModel
from typing import Optional

class Article(BaseModel):
    id: Optional[str] = None
    title: str
    content: str
    status: str
    author: Optional[str] = None
    views: int = 0

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Using custom model - collection name inferred from class name
    response = await directus.collection(Article).read()
    articles = response.items

    # Type-safe access
    for article in articles:
        print(article.title)
        print(article.status)

import asyncio
asyncio.run(main())
```

## File Management

### Upload File

```python
from py_directus import Directus
import aiofiles

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Upload a file
    async with aiofiles.open("image.jpg", "rb") as f:
        file_data = await f.read()

    response = await directus.upload_file(
        data=file_data,
        filename="image.jpg",
        title="My Image",
        folder="folder-uuid"
    )

    uploaded_file = response.item
    print(f"Uploaded file ID: {uploaded_file['id']}")

import asyncio
asyncio.run(main())
```

### Read Files

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Read all files
    response = await directus.collection("directus_files").read()
    files = response.items

    # Filter image files
    response = await (
        directus.collection("directus_files")
        .filter(type__starts_with="image/")
        .read()
    )

import asyncio
asyncio.run(main())
```

### Get File URL

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    file_id = "file-uuid"

    # Construct file URL
    file_url = f"{directus.url}/assets/{file_id}"

    # With transformations (for images)
    thumbnail_url = f"{directus.url}/assets/{file_id}?width=300&height=300&fit=cover"

import asyncio
asyncio.run(main())
```

## User Management

### Read Users

```python
from py_directus import Directus, DirectusUser

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Read all users
    response = await directus.collection(DirectusUser).read()
    users = response.items

    # Filter active users
    response = await (
        directus.collection(DirectusUser)
        .filter(status="active")
        .read()
    )

import asyncio
asyncio.run(main())
```

### Get Current User

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        email="user@example.com",
        password="password"
    )

    # Get current authenticated user
    response = await directus.collection("directus_users").read(id="me")
    current_user = response.item

    print(f"Logged in as: {current_user['email']}")

import asyncio
asyncio.run(main())
```

### Create User

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Create a new user
    new_user = {
        "email": "newuser@example.com",
        "password": "secure-password",
        "role": "role-uuid",
        "first_name": "John",
        "last_name": "Doe",
        "status": "active"
    }

    response = await directus.collection("directus_users").create(new_user)
    created_user = response.item

import asyncio
asyncio.run(main())
```

### Update User

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Update a user
    updated_data = {
        "first_name": "Jane",
        "status": "active"
    }

    response = await directus.collection("directus_users").update("user-uuid", updated_data)

import asyncio
asyncio.run(main())
```

## Roles and Permissions

### Read Roles

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Read all roles
    response = await directus.collection("directus_roles").read()
    roles = response.items

import asyncio
asyncio.run(main())
```

### Read Permissions

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Read permissions for a specific role
    response = await (
        directus.collection("directus_permissions")
        .filter(role="role-uuid")
        .read()
    )
    permissions = response.items

import asyncio
asyncio.run(main())
```

## Collections and Fields

### Read Collections

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Get all collections
    response = await directus.collection("directus_collections").read()
    collections = response.items

import asyncio
asyncio.run(main())
```

### Read Fields

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Get all fields for a collection
    response = await (
        directus.collection("directus_fields")
        .filter(collection="articles")
        .read()
    )
    fields = response.items

import asyncio
asyncio.run(main())
```

## Real-time with WebSockets

### Subscribe to Collection

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Subscribe to realtime updates
    async for message in directus.subscribe("articles"):
        print(f"Received update: {message}")

import asyncio
asyncio.run(main())
```

### Subscribe with Authentication

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        email="user@example.com",
        password="password"
    )

    # Subscribe to updates for authenticated user
    async for message in directus.subscribe("articles", uid=True):
        event_type = message.get("event")
        data = message.get("data")

        if event_type == "create":
            print(f"New article created: {data}")
        elif event_type == "update":
            print(f"Article updated: {data}")
        elif event_type == "delete":
            print(f"Article deleted: {data}")

import asyncio
asyncio.run(main())
```

## Advanced Query Building

### Chaining Methods

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Chain multiple query methods
    response = await (
        directus.collection("articles")
        .fields("id", "title", "author.name", "publish_date")
        .filter(status="published")
        .filter(views__gte=1000)
        .sort("-publish_date")
        .limit(10)
        .offset(0)
        .search("directus")
        .read()
    )

    articles = response.items

import asyncio
asyncio.run(main())
```

### Deep Filtering on Relations

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    # Filter based on related collection
    response = await (
        directus.collection("articles")
        .filter(author__status="active")
        .filter(author__verified=True)
        .read()
    )

import asyncio
asyncio.run(main())
```

## Error Handling

```python
from py_directus import Directus
import httpx

async def main():
    try:
        directus = await Directus(
            url="https://your-directus-instance.com",
            token="your-token"
        )

        response = await directus.collection("articles").read()
        articles = response.items

    except httpx.HTTPError as e:
        print(f"HTTP error occurred: {e}")
    except Exception as e:
        print(f"An error occurred: {e}")

import asyncio
asyncio.run(main())
```

Handle authentication errors:

```python
from py_directus import Directus

async def main():
    try:
        directus = await Directus(
            url="https://your-directus-instance.com",
            email="user@example.com",
            password="wrong-password"
        )
    except Exception as e:
        print(f"Authentication failed: {e}")

import asyncio
asyncio.run(main())
```

## Request Metadata

### Access Response Metadata

```python
from py_directus import Directus

async def main():
    directus = await Directus(
        url="https://your-directus-instance.com",
        token="your-token"
    )

    response = await directus.collection("articles").read()

    # Access items
    articles = response.items

    # Access single item (when reading by ID)
    # article = response.item

    # Check if request was successful
    print(f"Total items: {len(articles)}")

import asyncio
asyncio.run(main())
```

## Complete Example

```python
import os
import asyncio
from py_directus import Directus, F

async def main():
    # Initialize client
    directus = await Directus(
        url=os.getenv("DIRECTUS_URL"),
        token=os.getenv("DIRECTUS_TOKEN")
    )

    # Create a new article
    new_article = {
        "title": "Getting Started with Directus",
        "content": "Directus is a powerful headless CMS...",
        "status": "draft",
        "author": "author-uuid"
    }

    create_response = await directus.collection("articles").create(new_article)
    article_id = create_response.item["id"]
    print(f"Created article: {article_id}")

    # Read articles with filters
    read_response = await (
        directus.collection("articles")
        .fields("id", "title", "status", "author.name")
        .filter(status="published")
        .filter(views__gte=100)
        .sort("-publish_date")
        .limit(10)
        .read()
    )

    articles = read_response.items
    print(f"Found {len(articles)} published articles")

    # Update the article
    update_data = {"status": "published"}
    await directus.collection("articles").update(article_id, update_data)
    print(f"Published article: {article_id}")

    # Get statistics
    stats_response = await (
        directus.collection("articles")
        .aggregate(count="id")
        .aggregate(sum="views")
        .aggregate(avg="views")
        .groupby("status")
        .read()
    )

    for stat in stats_response.items:
        print(f"Status: {stat['status']}")
        print(f"Count: {stat['count']['id']}")
        print(f"Total views: {stat['sum']['views']}")
        print(f"Avg views: {stat['avg']['views']}")

    # Clean up - delete the article
    await directus.collection("articles").delete(article_id)
    print(f"Deleted article: {article_id}")

if __name__ == "__main__":
    asyncio.run(main())
```

## Useful Links

- Documentation: https://panos-stavrianos.github.io/py-directus/
- GitHub Repository: https://github.com/panos-stavrianos/py-directus
- PyPI Package: https://pypi.org/project/py-directus/
- Directus Official Docs: https://docs.directus.io/
- Directus REST API Reference: https://docs.directus.io/reference/introduction
