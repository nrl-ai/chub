---
name: support
description: "Zendesk API Python SDK (zenpy) for helpdesk, tickets, and customer service integration"
metadata:
  languages: "python"
  versions: "2.0.56"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "zendesk,support,helpdesk,tickets,customer-service"
---

# Zendesk API - Python SDK (zenpy)

## Golden Rule

**ALWAYS use the `zenpy` package (version 2.0.56 or later) for Zendesk API integration in Python projects.**

```bash
pip install zenpy
```

**DO NOT use:**
- `zend esk` (typo/incorrect package)
- `python-zendesk` (outdated)
- `@zendesk/client` (JavaScript package)
- Direct HTTP requests to Zendesk API endpoints (use the SDK instead)

The `zenpy` library is the officially recommended and most actively maintained Python client for the Zendesk API, providing a clean, Pythonic interface for interacting with Zendesk Support, Chat, and Help Center APIs.

---

## Installation

```bash
pip install zenpy
```

For development or testing:

```bash
pip install zenpy[dev]
```

With requirements.txt:

```txt
zenpy>=2.0.56
```

With Poetry:

```bash
poetry add zenpy
```

---

## Authentication & Initialization

### API Token Authentication (Recommended)

The most common authentication method uses email, API token, and subdomain:

```python
from zenpy import Zenpy

# Credentials dictionary
credentials = {
    'email': 'your_email@example.com',
    'token': 'your_api_token',
    'subdomain': 'your_subdomain'
}

# Create Zenpy client
zenpy_client = Zenpy(**credentials)
```

**Environment Variables Example:**

```python
import os
from zenpy import Zenpy
from dotenv import load_dotenv

load_dotenv()

credentials = {
    'email': os.getenv('ZENDESK_EMAIL'),
    'token': os.getenv('ZENDESK_API_TOKEN'),
    'subdomain': os.getenv('ZENDESK_SUBDOMAIN')
}

zenpy_client = Zenpy(**credentials)
```

**.env file:**

```
ZENDESK_EMAIL=your_email@example.com
ZENDESK_API_TOKEN=your_api_token_here
ZENDESK_SUBDOMAIN=your_company
```

### OAuth Token Authentication

For OAuth-based authentication:

```python
from zenpy import Zenpy

credentials = {
    'email': 'your_email@example.com',
    'oauth_token': 'your_oauth_access_token',
    'subdomain': 'your_subdomain'
}

zenpy_client = Zenpy(**credentials)
```

**With Environment Variables:**

```python
import os
from zenpy import Zenpy

credentials = {
    'email': os.getenv('ZENDESK_EMAIL'),
    'oauth_token': os.getenv('ZENDESK_OAUTH_TOKEN'),
    'subdomain': os.getenv('ZENDESK_SUBDOMAIN')
}

zenpy_client = Zenpy(**credentials)
```

### Password Authentication (Not Recommended)

```python
from zenpy import Zenpy

credentials = {
    'email': 'your_email@example.com',
    'password': 'your_password',
    'subdomain': 'your_subdomain'
}

zenpy_client = Zenpy(**credentials)
```

**Note:** Password authentication is deprecated and should be avoided. Use API token or OAuth authentication instead.

### Custom Configuration

```python
from zenpy import Zenpy

credentials = {
    'email': 'your_email@example.com',
    'token': 'your_api_token',
    'subdomain': 'your_subdomain',
    'timeout': 60,  # Request timeout in seconds
    'ratelimit': 700,  # API rate limit (requests per minute)
    'session': custom_session  # Optional: custom requests.Session object
}

zenpy_client = Zenpy(**credentials)
```

---

## Core API Surfaces

### Tickets API

#### List All Tickets

**Basic Example:**

```python
from zenpy import Zenpy

# Initialize client
zenpy_client = Zenpy(**credentials)

# List all tickets
for ticket in zenpy_client.tickets():
    print(f"#{ticket.id}: {ticket.subject}")
    print(f"Status: {ticket.status}, Priority: {ticket.priority}")
    print("---")
```

**Advanced Example with Filtering:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def list_open_tickets():
    """List all open tickets with details"""
    open_tickets = []

    for ticket in zenpy_client.tickets():
        if ticket.status == 'open':
            print(f"Ticket #{ticket.id}")
            print(f"Subject: {ticket.subject}")
            print(f"Status: {ticket.status}")
            print(f"Priority: {ticket.priority}")
            print(f"Requester: {ticket.requester.name}")
            print(f"Created: {ticket.created_at}")
            print("---")
            open_tickets.append(ticket)

    print(f"\nTotal open tickets: {len(open_tickets)}")
    return open_tickets

list_open_tickets()
```

#### Show Single Ticket

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get a single ticket by ID
ticket_id = 12345
ticket = zenpy_client.tickets(id=ticket_id)

print(f"Subject: {ticket.subject}")
print(f"Status: {ticket.status}")
print(f"Requester: {ticket.requester.name}")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def get_ticket_details(ticket_id):
    """Get comprehensive ticket details"""
    try:
        ticket = zenpy_client.tickets(id=ticket_id)

        print("=== Ticket Details ===")
        print(f"ID: {ticket.id}")
        print(f"Subject: {ticket.subject}")
        print(f"Description: {ticket.description}")
        print(f"Status: {ticket.status}")
        print(f"Priority: {ticket.priority}")
        print(f"Type: {ticket.type}")
        print(f"Requester: {ticket.requester.name} ({ticket.requester.email})")

        if ticket.assignee:
            print(f"Assignee: {ticket.assignee.name}")
        else:
            print("Assignee: Unassigned")

        if ticket.group:
            print(f"Group: {ticket.group.name}")

        print(f"Created: {ticket.created_at}")
        print(f"Updated: {ticket.updated_at}")
        print(f"Tags: {', '.join(ticket.tags)}")

        # Custom fields
        if ticket.custom_fields:
            print("\nCustom Fields:")
            for field in ticket.custom_fields:
                print(f"  {field.id}: {field.value}")

        return ticket

    except Exception as e:
        print(f"Error fetching ticket: {e}")
        return None

get_ticket_details(12345)
```

#### Create Ticket

**Basic Example:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import Ticket, Comment

zenpy_client = Zenpy(**credentials)

# Create a new ticket
ticket = Ticket(
    subject="Help with product installation",
    description="I need assistance installing the product on my system."
)

created_ticket = zenpy_client.tickets.create(ticket)

print("Ticket created successfully!")
print(f"Ticket ID: {created_ticket.id}")
print(f"Subject: {created_ticket.subject}")
```

**Advanced Example with Custom Fields:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import Ticket, User, Comment, CustomField

zenpy_client = Zenpy(**credentials)

def create_detailed_ticket():
    """Create a ticket with all details"""

    # Create ticket with requester
    ticket = Ticket(
        subject="Technical Support Request",
        description="Detailed description of the issue...\n\nSteps to reproduce:\n1. Step one\n2. Step two",
        requester=User(
            name="Jane Smith",
            email="jane.smith@example.com"
        ),
        priority="high",
        status="open",
        type="problem",
        tags=["technical", "urgent", "product-bug"],
        custom_fields=[
            CustomField(id=12345, value="Custom value 1"),
            CustomField(id=67890, value="Custom value 2")
        ]
    )

    # Create the ticket
    created_ticket = zenpy_client.tickets.create(ticket)

    print(f"Ticket created: {created_ticket.id}")
    print(f"Subject: {created_ticket.subject}")
    print(f"Requester: {created_ticket.requester.name}")

    return created_ticket

create_detailed_ticket()
```

**Create Ticket with Existing User:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import Ticket

zenpy_client = Zenpy(**credentials)

def create_ticket_for_user(user_id, subject, description):
    """Create ticket for an existing user"""

    ticket = Ticket(
        subject=subject,
        description=description,
        requester_id=user_id,
        priority="normal",
        status="new"
    )

    created_ticket = zenpy_client.tickets.create(ticket)
    print(f"Ticket {created_ticket.id} created for user {user_id}")

    return created_ticket

create_ticket_for_user(67890, "Account Issue", "Cannot access account")
```

#### Update Ticket

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get the ticket
ticket_id = 12345
ticket = zenpy_client.tickets(id=ticket_id)

# Update ticket status
ticket.status = "solved"
ticket.comment = "This issue has been resolved."

# Save changes
updated_ticket = zenpy_client.tickets.update(ticket)

print("Ticket updated successfully!")
print(f"New status: {updated_ticket.status}")
```

**Advanced Example:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import Comment

zenpy_client = Zenpy(**credentials)

def update_ticket_with_comment(ticket_id):
    """Update ticket with detailed changes"""

    # Get the ticket
    ticket = zenpy_client.tickets(id=ticket_id)

    # Update multiple fields
    ticket.status = "pending"
    ticket.priority = "high"
    ticket.tags.extend(["escalated", "requires-attention"])

    # Add internal comment
    ticket.comment = Comment(
        body="This ticket has been escalated to senior support.",
        public=False
    )

    # Update custom fields
    from zenpy.lib.api_objects import CustomField
    ticket.custom_fields = [
        CustomField(id=11111, value="Updated value")
    ]

    # Save changes
    updated_ticket = zenpy_client.tickets.update(ticket)

    print(f"Ticket updated: {updated_ticket.id}")
    print(f"New status: {updated_ticket.status}")
    print(f"New priority: {updated_ticket.priority}")

    return updated_ticket

update_ticket_with_comment(12345)
```

**Update Ticket Status:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def close_ticket(ticket_id):
    """Close a ticket with a resolution comment"""

    ticket = zenpy_client.tickets(id=ticket_id)
    ticket.status = "solved"
    ticket.comment = "This ticket has been resolved and closed."

    updated_ticket = zenpy_client.tickets.update(ticket)
    print(f"Ticket {ticket_id} closed successfully")

    return updated_ticket

close_ticket(12345)
```

#### Delete Ticket

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Delete a ticket
ticket_id = 12345
zenpy_client.tickets.delete(ticket_id)

print(f"Ticket {ticket_id} deleted successfully")
```

**Advanced Example with Confirmation:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def delete_ticket_safely(ticket_id):
    """Delete a ticket with verification"""

    try:
        # First, verify the ticket exists
        ticket = zenpy_client.tickets(id=ticket_id)
        print(f"About to delete ticket #{ticket.id}: {ticket.subject}")

        # Delete the ticket
        zenpy_client.tickets.delete(ticket_id)
        print("Ticket deleted successfully")

        return True

    except Exception as e:
        print(f"Error deleting ticket: {e}")
        return False

delete_ticket_safely(12345)
```

#### List Ticket Comments

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get ticket comments
ticket_id = 12345
comments = zenpy_client.tickets.comments(ticket_id)

for comment in comments:
    print(f"Author ID: {comment.author_id}")
    print(f"Body: {comment.body}")
    print(f"Public: {comment.public}")
    print("---")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def analyze_ticket_conversation(ticket_id):
    """Analyze ticket conversation history"""

    try:
        comments = list(zenpy_client.tickets.comments(ticket_id))

        public_comments = [c for c in comments if c.public]
        private_comments = [c for c in comments if not c.public]

        print("=== Ticket Conversation Analysis ===")
        print(f"Total comments: {len(comments)}")
        print(f"Public comments: {len(public_comments)}")
        print(f"Private/Internal notes: {len(private_comments)}")

        print("\n=== Comment History ===")
        for i, comment in enumerate(comments, 1):
            print(f"\n[Comment {i}]")
            print(f"ID: {comment.id}")
            print(f"Author ID: {comment.author_id}")
            print(f"Created: {comment.created_at}")
            print(f"Type: {'Public' if comment.public else 'Internal'}")
            print(f"Body: {comment.body[:100]}...")

            if hasattr(comment, 'attachments') and comment.attachments:
                print("Attachments:")
                for att in comment.attachments:
                    print(f"  - {att.file_name} ({att.size} bytes)")

        return comments

    except Exception as e:
        print(f"Error fetching comments: {e}")
        return None

analyze_ticket_conversation(12345)
```

---

### Users API

#### List Users

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# List all users
for user in zenpy_client.users():
    print(f"{user.name} ({user.email})")
```

**Advanced Example with Filtering:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def list_active_agents():
    """List all active agents"""

    agents = []

    for user in zenpy_client.users():
        if user.role == 'agent' and user.active:
            print(f"\nName: {user.name}")
            print(f"Email: {user.email}")
            print(f"Role: {user.role}")
            print(f"Timezone: {user.time_zone}")
            print(f"Locale: {user.locale}")

            if user.last_login_at:
                print(f"Last Login: {user.last_login_at}")

            agents.append(user)

    print(f"\nFound {len(agents)} active agents")
    return agents

list_active_agents()
```

#### Show User

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get a single user by ID
user_id = 67890
user = zenpy_client.users(id=user_id)

print(f"Name: {user.name}")
print(f"Email: {user.email}")
print(f"Role: {user.role}")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def get_user_profile(user_id):
    """Get comprehensive user profile"""

    try:
        user = zenpy_client.users(id=user_id)

        print("=== User Profile ===")
        print(f"ID: {user.id}")
        print(f"Name: {user.name}")
        print(f"Email: {user.email}")
        print(f"Phone: {user.phone or 'N/A'}")
        print(f"Role: {user.role}")
        print(f"Active: {user.active}")
        print(f"Verified: {user.verified}")
        print(f"Timezone: {user.time_zone}")
        print(f"Locale: {user.locale}")

        if user.organization_id:
            print(f"Organization ID: {user.organization_id}")

        print(f"Created: {user.created_at}")
        print(f"Updated: {user.updated_at}")

        if user.last_login_at:
            print(f"Last Login: {user.last_login_at}")

        if user.tags:
            print(f"Tags: {', '.join(user.tags)}")

        if hasattr(user, 'user_fields') and user.user_fields:
            print("\nCustom User Fields:")
            for key, value in user.user_fields.items():
                print(f"  {key}: {value}")

        return user

    except Exception as e:
        print(f"Error fetching user: {e}")
        return None

get_user_profile(67890)
```

#### Create User

**Basic Example:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import User

zenpy_client = Zenpy(**credentials)

# Create a new user
user = User(
    name="John Doe",
    email="john.doe@example.com",
    role="end-user"
)

created_user = zenpy_client.users.create(user)

print("User created successfully!")
print(f"User ID: {created_user.id}")
print(f"Name: {created_user.name}")
```

**Advanced Example:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import User

zenpy_client = Zenpy(**credentials)

def create_agent_user():
    """Create an agent user with full details"""

    user = User(
        name="Sarah Agent",
        email="sarah.agent@company.com",
        role="agent",
        phone="+1-555-123-4567",
        time_zone="America/New_York",
        locale="en-US",
        verified=True,
        tags=["support-team", "tier-2"],
        user_fields={
            "department": "Technical Support",
            "employee_id": "EMP-12345"
        },
        organization_id=98765
    )

    try:
        created_user = zenpy_client.users.create(user)
        print(f"Agent created: {created_user.id}")
        print(f"Name: {created_user.name}")
        print(f"Email: {created_user.email}")
        return created_user

    except Exception as e:
        print(f"Failed to create user: {e}")
        return None

create_agent_user()
```

#### Update User

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get the user
user_id = 67890
user = zenpy_client.users(id=user_id)

# Update user details
user.name = "Jane Smith"
user.phone = "+1-555-999-8888"

# Save changes
updated_user = zenpy_client.users.update(user)

print("User updated successfully!")
print(f"Name: {updated_user.name}")
print(f"Phone: {updated_user.phone}")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def update_user_profile(user_id):
    """Update user profile with multiple fields"""

    try:
        user = zenpy_client.users(id=user_id)

        # Update multiple fields
        user.name = "Jane Smith-Johnson"
        user.phone = "+1-555-987-6543"
        user.time_zone = "Pacific/Auckland"
        user.locale = "en-GB"
        user.tags = ["vip", "premium-support"]
        user.user_fields = {
            "department": "Engineering",
            "location": "Remote"
        }
        user.organization_id = 11111

        # Save changes
        updated_user = zenpy_client.users.update(user)

        print(f"User updated: {updated_user.id}")
        print(f"New name: {updated_user.name}")
        print(f"New timezone: {updated_user.time_zone}")

        return updated_user

    except Exception as e:
        print(f"Failed to update user: {e}")
        return None

update_user_profile(67890)
```

#### Delete User

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Delete a user
user_id = 67890
zenpy_client.users.delete(user_id)

print(f"User {user_id} deleted successfully")
```

#### Search Users

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Search for users by email
users = zenpy_client.search('john@example.com', type='user')

for user in users:
    print(f"{user.name} - {user.email}")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def search_users_by_name(search_term):
    """Search users by name"""

    try:
        users = list(zenpy_client.search(f'name:{search_term}', type='user'))

        print(f"Found {len(users)} users matching \"{search_term}\"")

        for user in users:
            print("\n---")
            print(f"ID: {user.id}")
            print(f"Name: {user.name}")
            print(f"Email: {user.email}")
            print(f"Role: {user.role}")
            print(f"Organization: {user.organization_id}")

        return users

    except Exception as e:
        print(f"Search failed: {e}")
        return []

search_users_by_name('John')
```

#### Get Current User

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get current authenticated user
current_user = zenpy_client.users.me()

print(f"Current user: {current_user.name}")
print(f"Email: {current_user.email}")
print(f"Role: {current_user.role}")
```

---

### Organizations API

#### List Organizations

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# List all organizations
for org in zenpy_client.organizations():
    print(f"{org.name} (ID: {org.id})")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def analyze_organizations():
    """Analyze all organizations"""

    organizations = list(zenpy_client.organizations())

    print(f"Total organizations: {len(organizations)}")

    for org in organizations:
        print("\n=== Organization ===")
        print(f"ID: {org.id}")
        print(f"Name: {org.name}")

        if hasattr(org, 'domain_names') and org.domain_names:
            print(f"Domain Names: {', '.join(org.domain_names)}")
        else:
            print("Domain Names: None")

        print(f"Details: {org.details or 'N/A'}")
        print(f"Notes: {org.notes or 'N/A'}")

        if org.tags:
            print(f"Tags: {', '.join(org.tags)}")

        print(f"Created: {org.created_at}")

        if hasattr(org, 'organization_fields') and org.organization_fields:
            print("Custom Fields:", org.organization_fields)

    return organizations

analyze_organizations()
```

#### Show Organization

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get a single organization by ID
org_id = 98765
org = zenpy_client.organizations(id=org_id)

print(f"Name: {org.name}")
print(f"Details: {org.details}")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def get_organization_details(org_id):
    """Get comprehensive organization details"""

    try:
        org = zenpy_client.organizations(id=org_id)

        print("=== Organization Details ===")
        print(f"ID: {org.id}")
        print(f"Name: {org.name}")

        if hasattr(org, 'domain_names') and org.domain_names:
            print(f"Domain Names: {', '.join(org.domain_names)}")
        else:
            print("Domain Names: None")

        print(f"Details: {org.details or 'N/A'}")
        print(f"Notes: {org.notes or 'N/A'}")

        if hasattr(org, 'group_id'):
            print(f"Group ID: {org.group_id or 'N/A'}")

        if org.tags:
            print(f"Tags: {', '.join(org.tags)}")

        print(f"Created: {org.created_at}")
        print(f"Updated: {org.updated_at}")

        if hasattr(org, 'organization_fields') and org.organization_fields:
            print("\nCustom Organization Fields:")
            for key, value in org.organization_fields.items():
                print(f"  {key}: {value}")

        return org

    except Exception as e:
        print(f"Error fetching organization: {e}")
        return None

get_organization_details(98765)
```

#### Create Organization

**Basic Example:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import Organization

zenpy_client = Zenpy(**credentials)

# Create a new organization
org = Organization(
    name="Acme Corporation",
    domain_names=["acme.com"]
)

created_org = zenpy_client.organizations.create(org)

print("Organization created!")
print(f"ID: {created_org.id}")
print(f"Name: {created_org.name}")
```

**Advanced Example:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import Organization

zenpy_client = Zenpy(**credentials)

def create_organization():
    """Create organization with full details"""

    organization = Organization(
        name="TechStart Solutions Inc.",
        domain_names=["techstart.com", "techstart.io"],
        details="Premium enterprise customer",
        notes="VIP support tier, 24/7 coverage",
        tags=["enterprise", "vip", "priority-support"],
        group_id=12345,
        organization_fields={
            "industry": "Technology",
            "account_tier": "Enterprise",
            "annual_revenue": "10M+"
        }
    )

    try:
        created_org = zenpy_client.organizations.create(organization)
        print(f"Organization created: {created_org.id}")
        print(f"Name: {created_org.name}")

        if created_org.domain_names:
            print(f"Domains: {', '.join(created_org.domain_names)}")

        return created_org

    except Exception as e:
        print(f"Failed to create organization: {e}")
        return None

create_organization()
```

#### Update Organization

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get the organization
org_id = 98765
org = zenpy_client.organizations(id=org_id)

# Update organization
org.name = "Acme Corporation Ltd."
org.details = "Updated company details"

# Save changes
updated_org = zenpy_client.organizations.update(org)

print("Organization updated!")
print(f"Name: {updated_org.name}")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def update_organization(org_id):
    """Update organization with multiple fields"""

    try:
        org = zenpy_client.organizations(id=org_id)

        # Update multiple fields
        org.name = "Acme Global Corporation"
        org.domain_names = ["acme.com", "acme.global", "acmeglobal.com"]
        org.details = "Global enterprise customer with multiple subsidiaries"
        org.notes = "Upgraded to platinum tier - assign dedicated account manager"
        org.tags = ["enterprise", "platinum", "global", "strategic-account"]
        org.organization_fields = {
            "account_tier": "Platinum",
            "contract_renewal": "2025-12-31",
            "dedicated_support": "yes"
        }

        # Save changes
        updated_org = zenpy_client.organizations.update(org)

        print(f"Organization updated: {updated_org.id}")
        print(f"New name: {updated_org.name}")

        if updated_org.tags:
            print(f"Tags: {', '.join(updated_org.tags)}")

        return updated_org

    except Exception as e:
        print(f"Failed to update organization: {e}")
        return None

update_organization(98765)
```

#### Delete Organization

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Delete an organization
org_id = 98765
zenpy_client.organizations.delete(org_id)

print(f"Organization {org_id} deleted successfully")
```

---

### Search API

#### Basic Search

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Search for open tickets
results = zenpy_client.search('status:open', type='ticket')

for ticket in results:
    print(f"#{ticket.id}: {ticket.subject}")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def search_open_tickets():
    """Search for high-priority open tickets"""

    try:
        results = list(zenpy_client.search('status:open priority:high', type='ticket'))

        print(f"Found {len(results)} high-priority open tickets")

        for ticket in results:
            print("\n---")
            print(f"Ticket #{ticket.id}")
            print(f"Subject: {ticket.subject}")
            print(f"Priority: {ticket.priority}")
            print(f"Status: {ticket.status}")
            print(f"Created: {ticket.created_at}")
            print(f"Assignee: {ticket.assignee_id or 'Unassigned'}")

        return results

    except Exception as e:
        print(f"Search failed: {e}")
        return []

search_open_tickets()
```

#### Search Tickets

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Search tickets by subject
tickets = zenpy_client.search('subject:login', type='ticket')

print(f"Tickets mentioning 'login': {len(list(tickets))}")
```

**Advanced Example with Multiple Filters:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def advanced_ticket_search():
    """Perform advanced ticket search with multiple filters"""

    # Build search query
    query_parts = [
        'type:ticket',
        'status:open',
        'priority:urgent',
        'tags:bug',
        'created>2025-01-01'
    ]
    query = ' '.join(query_parts)

    try:
        tickets = list(zenpy_client.search(query))

        print(f"Search Query: {query}")
        print(f"Results: {len(tickets)} tickets\n")

        for ticket in tickets:
            print(f"#{ticket.id}: {ticket.subject}")
            print(f"  Status: {ticket.status} | Priority: {ticket.priority}")

            if ticket.tags:
                print(f"  Tags: {', '.join(ticket.tags)}")

            print(f"  Created: {ticket.created_at}")

        return tickets

    except Exception as e:
        print(f"Search error: {e}")
        return []

advanced_ticket_search()
```

#### Search Users

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Search users by email domain
users = zenpy_client.search('email:*@example.com', type='user')

for user in users:
    print(f"{user.name} - {user.email}")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def find_agents_by_organization(org_id):
    """Find all agents in a specific organization"""

    query = f'type:user role:agent organization_id:{org_id}'

    try:
        agents = list(zenpy_client.search(query))

        print(f"Found {len(agents)} agents in organization {org_id}")

        for agent in agents:
            print("\n---")
            print(f"Name: {agent.name}")
            print(f"Email: {agent.email}")
            print(f"Role: {agent.role}")
            print(f"Active: {agent.active}")
            print(f"Last Login: {agent.last_login_at or 'Never'}")

        return agents

    except Exception as e:
        print(f"User search failed: {e}")
        return []

find_agents_by_organization(98765)
```

#### Search Organizations

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Search organizations by name
orgs = zenpy_client.search('name:Acme', type='organization')

for org in orgs:
    print(f"{org.name} (ID: {org.id})")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def search_organizations_by_domain(domain):
    """Search organizations by domain"""

    query = f'type:organization {domain}'

    try:
        organizations = list(zenpy_client.search(query))

        print(f"Found {len(organizations)} organizations with domain \"{domain}\"")

        for org in organizations:
            print("\n=== Organization ===")
            print(f"ID: {org.id}")
            print(f"Name: {org.name}")

            if hasattr(org, 'domain_names') and org.domain_names:
                print(f"Domains: {', '.join(org.domain_names)}")
            else:
                print("Domains: None")

            print(f"Details: {org.details or 'N/A'}")
            print(f"Created: {org.created_at}")

        return organizations

    except Exception as e:
        print(f"Organization search failed: {e}")
        return []

search_organizations_by_domain('acme.com')
```

#### Search Export (For Large Result Sets)

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def export_all_tickets():
    """Export all tickets using search_export for large datasets"""

    # search_export is designed for exporting large numbers of tickets
    # It bypasses the 1000-result limit of regular search

    count = 0

    for ticket in zenpy_client.search_export(type='ticket', status='open'):
        count += 1
        print(f"#{ticket.id}: {ticket.subject}")

        # Process tickets in batches
        if count % 100 == 0:
            print(f"Processed {count} tickets...")

    print(f"\nTotal tickets exported: {count}")
    return count

export_all_tickets()
```

---

### Groups API

#### List Groups

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# List all groups
for group in zenpy_client.groups():
    print(f"{group.name} (ID: {group.id})")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def list_all_groups():
    """List all groups with details"""

    groups = list(zenpy_client.groups())

    print(f"Found {len(groups)} groups\n")

    for group in groups:
        print("=== Group ===")
        print(f"ID: {group.id}")
        print(f"Name: {group.name}")
        print(f"Description: {group.description or 'N/A'}")
        print(f"Created: {group.created_at}")
        print(f"Updated: {group.updated_at}")
        print("---")

    return groups

list_all_groups()
```

#### Show Group

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get a single group by ID
group_id = 12345
group = zenpy_client.groups(id=group_id)

print(f"Group: {group.name}")
print(f"Description: {group.description}")
```

#### Create Group

**Basic Example:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import Group

zenpy_client = Zenpy(**credentials)

# Create a new group
group = Group(name="Technical Support Team")

created_group = zenpy_client.groups.create(group)

print("Group created!")
print(f"ID: {created_group.id}")
print(f"Name: {created_group.name}")
```

**Advanced Example:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import Group

zenpy_client = Zenpy(**credentials)

def create_support_group():
    """Create a support group with description"""

    group = Group(
        name="Enterprise Support Team",
        description="Dedicated support team for enterprise customers"
    )

    try:
        created_group = zenpy_client.groups.create(group)
        print(f"Group created: {created_group.id}")
        print(f"Name: {created_group.name}")
        print(f"Description: {created_group.description}")
        return created_group

    except Exception as e:
        print(f"Failed to create group: {e}")
        return None

create_support_group()
```

#### Update Group

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get the group
group_id = 12345
group = zenpy_client.groups(id=group_id)

# Update group
group.name = "Updated Group Name"
group.description = "Updated description"

# Save changes
updated_group = zenpy_client.groups.update(group)

print("Group updated!")
print(f"Name: {updated_group.name}")
```

#### Delete Group

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Delete a group
group_id = 12345
zenpy_client.groups.delete(group_id)

print(f"Group {group_id} deleted successfully")
```

---

### Attachments API

#### Upload Attachment

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Upload a file
file_path = './documents/file.pdf'
upload = zenpy_client.attachments.upload(file_path)

print("File uploaded successfully!")
print(f"Upload token: {upload.token}")
print(f"Attachment: {upload.attachment}")
```

**Advanced Example with Ticket Creation:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import Ticket, User, Comment

zenpy_client = Zenpy(**credentials)

def create_ticket_with_attachment():
    """Create a ticket with an attached file"""

    try:
        # Step 1: Upload the file
        file_path = './screenshots/bug-screenshot.png'
        upload = zenpy_client.attachments.upload(file_path)

        print(f"File uploaded. Token: {upload.token}")

        # Step 2: Create ticket with attachment
        ticket = Ticket(
            subject="Bug Report: UI Issue",
            comment=Comment(
                body="Please see the attached screenshot showing the UI bug.",
                uploads=[upload.token]
            ),
            requester=User(
                name="Bug Reporter",
                email="reporter@example.com"
            ),
            priority="high",
            tags=["bug", "ui"]
        )

        created_ticket = zenpy_client.tickets.create(ticket)

        print("Ticket created with attachment!")
        print(f"Ticket ID: {created_ticket.id}")
        print(f"Subject: {created_ticket.subject}")

        return created_ticket

    except Exception as e:
        print(f"Error: {e}")
        return None

create_ticket_with_attachment()
```

#### Upload Multiple Attachments

**Advanced Example:**

```python
from zenpy import Zenpy
from zenpy.lib.api_objects import Ticket, User, Comment

zenpy_client = Zenpy(**credentials)

def create_ticket_with_multiple_attachments():
    """Create a ticket with multiple attached files"""

    try:
        # Upload multiple files
        files = [
            './documents/report.pdf',
            './screenshots/screen1.png',
            './screenshots/screen2.png'
        ]

        upload_tokens = []
        for file_path in files:
            upload = zenpy_client.attachments.upload(file_path)
            upload_tokens.append(upload.token)
            print(f"Uploaded: {file_path}")

        print(f"Uploaded {len(upload_tokens)} files")

        # Create ticket with all attachments
        ticket = Ticket(
            subject="Detailed Bug Report",
            comment=Comment(
                body="Please find the attached report and screenshots.",
                uploads=upload_tokens
            ),
            requester=User(
                name="QA Tester",
                email="qa@example.com"
            ),
            priority="normal"
        )

        created_ticket = zenpy_client.tickets.create(ticket)

        print("Ticket created with multiple attachments!")
        print(f"Ticket ID: {created_ticket.id}")

        return created_ticket

    except Exception as e:
        print(f"Error: {e}")
        return None

create_ticket_with_multiple_attachments()
```

---

### Macros API

#### List Macros

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# List all macros
for macro in zenpy_client.macros():
    print(f"{macro.title} (ID: {macro.id})")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def list_active_macros():
    """List all active macros"""

    macros = list(zenpy_client.macros())
    active_macros = [m for m in macros if m.active]

    print(f"Found {len(active_macros)} active macros\n")

    for macro in active_macros:
        print("=== Macro ===")
        print(f"ID: {macro.id}")
        print(f"Title: {macro.title}")
        print(f"Description: {macro.description or 'N/A'}")
        print(f"Active: {macro.active}")
        print(f"Created: {macro.created_at}")
        print("---")

    return active_macros

list_active_macros()
```

#### Show Macro

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get a single macro by ID
macro_id = 54321
macro = zenpy_client.macros(id=macro_id)

print(f"Title: {macro.title}")
print(f"Description: {macro.description}")
print(f"Actions: {macro.actions}")
```

#### Apply Macro to Ticket

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def apply_macro_to_ticket(ticket_id, macro_id):
    """Apply a macro to a ticket"""

    try:
        # Get the ticket
        ticket = zenpy_client.tickets(id=ticket_id)

        # Show macro effect
        result = zenpy_client.tickets.show_macro_effect(ticket, macro_id)

        print("Macro applied successfully!")
        print(f"Result: {result}")

        return result

    except Exception as e:
        print(f"Error applying macro: {e}")
        return None

apply_macro_to_ticket(12345, 54321)
```

---

### Views API

#### List Views

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# List all views
for view in zenpy_client.views():
    print(f"{view.title} (ID: {view.id})")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def list_active_views():
    """List all active views"""

    views = list(zenpy_client.views())
    active_views = [v for v in views if v.active]

    print(f"Found {len(active_views)} active views\n")

    for view in active_views:
        print("=== View ===")
        print(f"ID: {view.id}")
        print(f"Title: {view.title}")
        print(f"Description: {view.description or 'N/A'}")
        print(f"Active: {view.active}")
        print(f"Position: {view.position}")
        print("---")

    return active_views

list_active_views()
```

#### Execute View

**Basic Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Execute a view
view_id = 11111
result = zenpy_client.views.execute(view_id)

print("View executed!")
for ticket in result:
    print(f"#{ticket.id}: {ticket.subject}")
```

**Advanced Example:**

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

def execute_view_and_analyze(view_id):
    """Execute a view and analyze results"""

    try:
        result = list(zenpy_client.views.execute(view_id))

        print("=== View Execution Results ===")
        print(f"Total tickets: {len(result)}")

        # Analyze ticket statuses
        status_counts = {}
        for ticket in result:
            status = ticket.status
            status_counts[status] = status_counts.get(status, 0) + 1

        print("\n=== Status Distribution ===")
        for status, count in status_counts.items():
            print(f"{status}: {count}")

        # Show first 10 tickets
        print("\n=== Tickets ===")
        for ticket in result[:10]:
            print(f"#{ticket.id}: {ticket.subject}")
            print(f"  Status: {ticket.status} | Priority: {ticket.priority}")

        return result

    except Exception as e:
        print(f"Error executing view: {e}")
        return []

execute_view_and_analyze(11111)
```

---

## Error Handling

### Basic Error Handling

```python
from zenpy import Zenpy
from zenpy.lib.exception import ZenpyException

zenpy_client = Zenpy(**credentials)

try:
    ticket = zenpy_client.tickets(id=99999)
    print(f"Ticket: {ticket.subject}")
except ZenpyException as e:
    print(f"Error occurred: {e}")
```

### Advanced Error Handling

```python
from zenpy import Zenpy
from zenpy.lib.exception import ZenpyException, RecordNotFoundException, ZenpyRateLimitExceeded
import time

zenpy_client = Zenpy(**credentials)

def handle_zendesk_errors():
    """Demonstrate comprehensive error handling"""

    try:
        ticket = zenpy_client.tickets(id=99999)
        return ticket

    except RecordNotFoundException:
        print("Ticket not found")

    except ZenpyRateLimitExceeded as e:
        print(f"Rate limit exceeded - retry after: {e.retry_after}")
        time.sleep(e.retry_after)

    except ZenpyException as e:
        print(f"Zenpy error: {e}")

    except Exception as e:
        print(f"Unknown error: {e}")

    return None
```

### Retry Logic

```python
from zenpy import Zenpy
from zenpy.lib.exception import ZenpyRateLimitExceeded
import time

zenpy_client = Zenpy(**credentials)

def fetch_ticket_with_retry(ticket_id, max_retries=3):
    """Fetch a ticket with automatic retry logic"""

    for attempt in range(1, max_retries + 1):
        try:
            ticket = zenpy_client.tickets(id=ticket_id)
            return ticket

        except ZenpyRateLimitExceeded as e:
            if attempt < max_retries:
                wait_time = e.retry_after if hasattr(e, 'retry_after') else 2 ** attempt
                print(f"Rate limited. Retrying in {wait_time}s...")
                time.sleep(wait_time)
            else:
                print(f"Failed after {max_retries} attempts")
                raise

        except Exception as e:
            if attempt < max_retries:
                print(f"Error. Attempt {attempt}/{max_retries}")
                time.sleep(1)
            else:
                raise

    return None

ticket = fetch_ticket_with_retry(12345)
```

---

## Complete Working Examples

### Basic Support Ticket System

```python
import os
from zenpy import Zenpy
from zenpy.lib.api_objects import Ticket, User
from dotenv import load_dotenv

load_dotenv()

credentials = {
    'email': os.getenv('ZENDESK_EMAIL'),
    'token': os.getenv('ZENDESK_API_TOKEN'),
    'subdomain': os.getenv('ZENDESK_SUBDOMAIN')
}

zenpy_client = Zenpy(**credentials)

def create_support_ticket(customer_email, subject, description):
    """Create a support ticket for a customer"""

    try:
        # Check if user exists
        users = list(zenpy_client.search(customer_email, type='user'))

        if users:
            user = users[0]
            print(f"Found existing user: {user.name}")
        else:
            # Create new user
            user = User(
                name=customer_email.split('@')[0],
                email=customer_email,
                role='end-user'
            )
            user = zenpy_client.users.create(user)
            print(f"Created new user: {user.name}")

        # Create ticket
        ticket = Ticket(
            subject=subject,
            description=description,
            requester_id=user.id,
            priority='normal',
            status='new'
        )

        created_ticket = zenpy_client.tickets.create(ticket)

        print("Ticket created successfully!")
        print(f"Ticket ID: {created_ticket.id}")
        print(f"Subject: {created_ticket.subject}")

        return created_ticket

    except Exception as e:
        print(f"Error creating support ticket: {e}")
        return None

# Usage
create_support_ticket(
    'customer@example.com',
    'Cannot access my account',
    'I am unable to log into my account. I keep getting an error message.'
)
```

### Ticket Management Dashboard

```python
import os
from zenpy import Zenpy
from dotenv import load_dotenv

load_dotenv()

credentials = {
    'email': os.getenv('ZENDESK_EMAIL'),
    'token': os.getenv('ZENDESK_API_TOKEN'),
    'subdomain': os.getenv('ZENDESK_SUBDOMAIN')
}

zenpy_client = Zenpy(**credentials)

def get_dashboard_stats():
    """Get support dashboard statistics"""

    try:
        # Search for different ticket categories
        open_tickets = list(zenpy_client.search('type:ticket status:open'))
        pending_tickets = list(zenpy_client.search('type:ticket status:pending'))
        solved_tickets = list(zenpy_client.search('type:ticket status:solved created>2025-01-01'))
        urgent_tickets = list(zenpy_client.search('type:ticket priority:urgent status<solved'))

        print("=== Support Dashboard ===")
        print(f"Open Tickets: {len(open_tickets)}")
        print(f"Pending Tickets: {len(pending_tickets)}")
        print(f"Solved This Year: {len(solved_tickets)}")
        print(f"Urgent Tickets: {len(urgent_tickets)}")

        # List urgent tickets
        if urgent_tickets:
            print("\n=== Urgent Tickets Requiring Attention ===")
            for ticket in urgent_tickets:
                print(f"#{ticket.id}: {ticket.subject}")
                print(f"  Status: {ticket.status} | Created: {ticket.created_at}")
                print(f"  Assignee: {ticket.assignee_id or 'Unassigned'}")

        return {
            'open': len(open_tickets),
            'pending': len(pending_tickets),
            'solved': len(solved_tickets),
            'urgent': len(urgent_tickets)
        }

    except Exception as e:
        print(f"Error fetching dashboard stats: {e}")
        return None

get_dashboard_stats()
```

### Bulk Ticket Operations

```python
import os
from zenpy import Zenpy
from zenpy.lib.api_objects import Comment
from dotenv import load_dotenv

load_dotenv()

credentials = {
    'email': os.getenv('ZENDESK_EMAIL'),
    'token': os.getenv('ZENDESK_API_TOKEN'),
    'subdomain': os.getenv('ZENDESK_SUBDOMAIN')
}

zenpy_client = Zenpy(**credentials)

def bulk_update_tickets(ticket_ids, status='solved', comment_text=None):
    """Update multiple tickets at once"""

    try:
        print(f"Updating {len(ticket_ids)} tickets...")

        results = []

        for ticket_id in ticket_ids:
            try:
                ticket = zenpy_client.tickets(id=ticket_id)
                ticket.status = status

                if comment_text:
                    ticket.comment = Comment(body=comment_text, public=False)

                updated_ticket = zenpy_client.tickets.update(ticket)
                print(f"✓ Updated ticket #{ticket_id}")
                results.append({'success': True, 'ticket_id': ticket_id, 'ticket': updated_ticket})

            except Exception as e:
                print(f"✗ Failed to update ticket #{ticket_id}: {e}")
                results.append({'success': False, 'ticket_id': ticket_id, 'error': str(e)})

        successful = len([r for r in results if r['success']])
        failed = len([r for r in results if not r['success']])

        print(f"\n=== Bulk Update Complete ===")
        print(f"Successful: {successful}")
        print(f"Failed: {failed}")

        return results

    except Exception as e:
        print(f"Bulk update error: {e}")
        return None

# Usage: Close multiple tickets at once
ticket_ids = [12345, 12346, 12347]
bulk_update_tickets(
    ticket_ids,
    status='solved',
    comment_text='This ticket has been resolved and closed.'
)
```

---

## Additional Features

### Object Conversion

Zenpy provides convenient methods for converting objects:

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get a ticket
ticket = zenpy_client.tickets(id=12345)

# Convert to dictionary
ticket_dict = ticket.to_dict()
print(ticket_dict)

# Convert to JSON
ticket_json = ticket.to_json()
print(ticket_json)
```

### Lazy Attribute Evaluation

Zenpy uses lazy evaluation to minimize API calls:

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# Get a ticket
ticket = zenpy_client.tickets(id=12345)

# Accessing requester automatically fetches the User object
print(f"Requester: {ticket.requester.name}")  # Only one API call

# Accessing organization
if ticket.organization:
    print(f"Organization: {ticket.organization.name}")
```

### Caching

Zenpy automatically caches objects to reduce API calls:

```python
from zenpy import Zenpy

zenpy_client = Zenpy(**credentials)

# First access - API call made
ticket1 = zenpy_client.tickets(id=12345)

# Second access - served from cache
ticket2 = zenpy_client.tickets(id=12345)

# Both references point to the same object
assert ticket1 is ticket2
```

---

## Best Practices

1. **Always use environment variables for credentials**
2. **Handle rate limiting with appropriate retry logic**
3. **Use search_export for large result sets (>1000 items)**
4. **Leverage lazy evaluation and caching**
5. **Catch specific Zenpy exceptions for better error handling**
6. **Use batch operations when updating multiple tickets**
7. **Validate user input before creating tickets**
8. **Use custom fields appropriately for your organization**
9. **Test with a sandbox account before production**
10. **Monitor API usage to avoid rate limits**

---

## Rate Limits

Zendesk enforces rate limits on API requests:

- **Standard**: 700 requests per minute
- **Enterprise**: 2500+ requests per minute (varies by plan)

Zenpy automatically handles rate limiting by:
- Tracking request counts
- Sleeping when limits are approached
- Raising `ZenpyRateLimitExceeded` when exceeded

```python
from zenpy import Zenpy

# Configure custom rate limit
credentials = {
    'email': 'your_email@example.com',
    'token': 'your_api_token',
    'subdomain': 'your_subdomain',
    'ratelimit': 700  # Requests per minute
}

zenpy_client = Zenpy(**credentials)
```

---

## Resources

- **GitHub Repository**: https://github.com/facetoe/zenpy
- **Documentation**: http://docs.facetoe.com.au/zenpy.html
- **PyPI Package**: https://pypi.org/project/zenpy/
- **Zendesk API Docs**: https://developer.zendesk.com/api-reference/
- **Issue Tracker**: https://github.com/facetoe/zenpy/issues
