# MCP Server

Chub includes a built-in MCP (Model Context Protocol) stdio server for AI agents.

## Starting the server

```sh
chub mcp
chub mcp --profile backend    # With a profile
```

## MCP Tools

### chub_search

Search for docs in the registry.

```json
{
  "name": "chub_search",
  "arguments": {
    "query": "stripe payments",
    "limit": 10
  }
}
```

### chub_get

Fetch a specific doc by ID.

```json
{
  "name": "chub_get",
  "arguments": {
    "id": "openai/chat",
    "lang": "python",
    "version": "4.0"
  }
}
```

When a doc is pinned, the pinned version/language is automatically applied.

### chub_list

List all available docs.

```json
{
  "name": "chub_list",
  "arguments": {}
}
```

### chub_annotate

Add an annotation to a doc.

```json
{
  "name": "chub_annotate",
  "arguments": {
    "id": "openai/chat",
    "note": "Use streaming API for chat completions"
  }
}
```

### chub_feedback

Submit feedback about a doc.

```json
{
  "name": "chub_feedback",
  "arguments": {
    "id": "openai/chat",
    "feedback": "Missing example for function calling"
  }
}
```

## MCP Resources

| URI | Description |
|---|---|
| `chub://registry` | Full merged registry |

## Agent Integration

### Claude Code

```json
{
  "mcpServers": {
    "chub": {
      "command": "chub",
      "args": ["mcp"]
    }
  }
}
```

### Cursor

Settings → MCP Servers → Add:
- Command: `chub mcp`
- Transport: stdio

## Team-aware behavior

When running as an MCP server, Chub automatically:

- Applies pinned versions and languages
- Serves project context docs (via `project/<name>`)
- Appends team annotations to doc content
- Appends pin notices to pinned docs
- Scopes results to the active profile (if set)
