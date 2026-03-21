---
name: package
description: "FastMCP Python package for building MCP servers, clients, and apps with typed tools, resources, prompts, transports, and auth"
metadata:
  languages: "python"
  versions: "3.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "fastmcp,mcp,python,agents,tools,client,server,oauth"
---

# FastMCP Python Package Guide

## Golden Rule

Use `fastmcp` for both MCP servers and MCP clients, import the main APIs from `fastmcp`, and pin the version you expect. The official docs site tracks FastMCP's `main` branch, not a frozen release branch, so check version badges before copying newer examples into a project pinned to `3.1.0`.

## Install

FastMCP officially recommends `uv`, but standard `pip` is also documented:

```bash
python -m pip install "fastmcp==3.1.0"
uv add "fastmcp==3.1.0"
```

Verify the install:

```bash
fastmcp version
```

Useful extras published on PyPI:

```bash
python -m pip install "fastmcp[tasks]==3.1.0"
python -m pip install "fastmcp[apps]==3.1.0"
python -m pip install "fastmcp[openai]==3.1.0"
```

Other extras listed on PyPI are `anthropic`, `azure`, `code-mode`, and `gemini`.

## Create A Server

FastMCP wraps ordinary Python callables as MCP tools, resources, and prompts. Type hints and docstrings become MCP schemas and descriptions automatically.

```python
from fastmcp import Context, FastMCP

mcp = FastMCP(
    name="SupportServer",
    instructions="Use the available tools and resources to answer support questions.",
)

@mcp.tool
async def get_ticket_status(ticket_id: str, ctx: Context) -> dict:
    """Look up a support ticket."""
    await ctx.info(f"Looking up {ticket_id}")
    return {"ticket_id": ticket_id, "status": "open"}

@mcp.resource("config://support")
def support_config() -> dict:
    """Expose read-only configuration to clients."""
    return {"queue": "tier-1", "region": "us-west-2"}

@mcp.prompt
def summarize_ticket(ticket_id: str) -> str:
    """Create a reusable prompt for ticket summaries."""
    return f"Summarize the latest activity for ticket {ticket_id}."

if __name__ == "__main__":
    mcp.run()
```

Practical notes:

- `@mcp.tool` is for executable actions.
- `@mcp.resource("scheme://...")` is for read-only content or templates addressed by URI.
- `@mcp.prompt` is for reusable prompt/message templates.
- Tool schemas come from function signatures. Avoid `*args` and `**kwargs` for tools.
- Use `Context` when a tool needs MCP features like logging, progress, resource reads, or client sampling.

## Run And Test The Server

Default transport is STDIO, which is what local desktop/editor MCP integrations commonly expect:

```bash
python my_server.py
fastmcp run my_server.py:mcp
```

For remote access, use HTTP transport:

```bash
fastmcp run my_server.py:mcp --transport http --port 8000
```

Equivalent in code:

```python
if __name__ == "__main__":
    mcp.run(transport="http", host="127.0.0.1", port=8000)
```

HTTP servers are exposed at `http://localhost:8000/mcp`. SSE is still supported for older clients, but the docs recommend HTTP/Streamable HTTP for new work.

FastMCP's CLI is useful for agent workflows and smoke tests:

```bash
fastmcp list my_server.py --resources --prompts
fastmcp call my_server.py get_ticket_status ticket_id=T-123 --json
```

`fastmcp list --json` and `fastmcp call --json` are explicitly documented for LLM agents that can execute shell commands but do not have native MCP support.

## Use The Programmatic Client

`Client` infers transport from what you pass in:

- `Client(FastMCP(...))`: in-memory transport, best for tests
- `Client("my_server.py")`: STDIO subprocess transport
- `Client("https://example.com/mcp")`: HTTP transport
- `Client(config_dict)`: multi-server config using MCP-style config dictionaries

Basic usage:

```python
import asyncio

from fastmcp import Client

async def main() -> None:
    async with Client("http://localhost:8000/mcp") as client:
        await client.ping()

        tools = await client.list_tools()
        print([tool.name for tool in tools])

        result = await client.call_tool("get_ticket_status", {"ticket_id": "T-123"})
        print(result.data)

        resources = await client.list_resources()
        print([resource.uri for resource in resources])

        prompt = await client.get_prompt("summarize_ticket", {"ticket_id": "T-123"})
        print(prompt.messages)

asyncio.run(main())
```

Important client behavior:

- Always use `async with client:` so the connection and MCP initialization handshake happen cleanly.
- For local script targets, pass required environment variables with `env={...}` because STDIO subprocesses do not inherit your application state automatically.
- Use in-memory `Client(server)` for unit tests when you do not want subprocess or network complexity.
- If you need full control over timing, set `auto_initialize=False` and call `await client.initialize()` yourself.

## Authentication And Configuration

Server-side authentication applies only to HTTP-based transports (`http` and `sse`), not STDIO. Configure auth providers in code and load secrets from environment variables.

Example pattern from the official auth docs:

```python
import os

from fastmcp import FastMCP
from fastmcp.server.auth.providers.github import GitHubProvider

auth = GitHubProvider(
    client_id=os.environ["GITHUB_CLIENT_ID"],
    client_secret=os.environ["GITHUB_CLIENT_SECRET"],
    base_url=os.environ.get("BASE_URL", "http://localhost:8000"),
)

mcp = FastMCP(name="SecureServer", auth=auth)
```

For service-to-service protection without an OAuth login flow, FastMCP also supports bearer-token verification, including `StaticTokenVerifier` for development and other token verifier backends for production.

Client auth options for HTTP targets:

- `auth="oauth"` for interactive OAuth with browser-based login
- `auth=OAuth(...)` when you need scopes, token storage, or pre-registered client credentials
- `auth="YOUR_TOKEN"` for bearer-token auth
- CLI: `fastmcp call http://localhost:8000/mcp my_tool --auth none` to skip OAuth against local dev servers

If you persist OAuth tokens, use encrypted storage in production.

## Common Pitfalls

- The docs site reflects `main`, not a release-specific snapshot. A page can describe features newer than your installed `3.1.0`.
- Some docs pages still show example output from `3.0.0` while PyPI currently publishes `3.1.0`; trust version badges and PyPI release metadata when they disagree.
- HTTP and STDIO are not interchangeable operationally. STDIO is local-process oriented; HTTP is for deployed services and remote clients.
- SSE is legacy. Use HTTP unless you specifically need compatibility with older clients.
- Tools need schema-friendly signatures. Do not expose `*args` or `**kwargs` as tools.
- Client code must use `async with`; calling operations on an unentered client is a common mistake.
- Local subprocess clients need explicit `env={...}` for secrets and config.
- Prompt arguments arrive over MCP as strings. FastMCP helps with typed conversion, but complex prompt inputs still need JSON-string-compatible formats at the protocol boundary.

## Version-Sensitive Notes For 3.1.0

- PyPI lists `fastmcp 3.1.0` as the latest release on March 12, 2026.
- The FastMCP docs are currently branded as `v3`, but the welcome page explicitly says the site tracks the `main` branch and may include unreleased features.
- The CLI surface for agent workflows, including `fastmcp list` and `fastmcp call`, is documented as new in `3.0.0`; those commands are stable enough to rely on in `3.1.0`.
- CIMD and pre-registered OAuth client flows are also documented as `3.0.0` features, so they are in scope for `3.1.0`.
- If you are migrating old FastMCP v2 code, use the official upgrade guides instead of mixing v2 docs paths with current v3 examples.

## Official Sources

- Docs root: https://gofastmcp.com/
- Welcome: https://gofastmcp.com/getting-started/welcome
- Installation: https://gofastmcp.com/getting-started/installation
- Quickstart: https://gofastmcp.com/getting-started/quickstart
- Server overview: https://gofastmcp.com/servers/server
- Tools: https://gofastmcp.com/servers/tools
- Resources: https://gofastmcp.com/servers/resources
- Client: https://gofastmcp.com/clients/client
- Client CLI: https://gofastmcp.com/clients/cli
- Server auth: https://gofastmcp.com/servers/auth/authentication
- Token verification: https://gofastmcp.com/servers/auth/token-verification
- Client OAuth auth: https://gofastmcp.com/clients/auth/oauth
- Client bearer auth: https://gofastmcp.com/clients/auth/bearer
- PyPI: https://pypi.org/project/fastmcp/
