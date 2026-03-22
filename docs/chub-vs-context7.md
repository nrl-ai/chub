# Chub vs Context7

## TL;DR

They solve different problems. Run both.

| | Context7 | Chub |
|---|---|---|
| Public library docs | Auto-crawled, always fresh | 1,560+ curated entries |
| Private/internal docs | No | Yes |
| Team annotations | No | Yes (git-tracked) |
| Version pinning | No | Yes (with reasons) |
| Project context | No | Yes (`.chub/context/`) |
| AGENTS.md generation | No | Yes |
| Self-hosted | No (SaaS only) | Yes |
| Offline | No | Yes |
| Air-gapped/compliance | No | Yes |

---

## What Context7 does well

Context7 is a SaaS documentation platform backed by Upstash. It crawls public library repos and package registries continuously, so docs are always current. Its MCP server exposes two tools — `resolve-library-id` and `query-docs` — and its backend uses LLM-powered ranking to return the most relevant snippets.

**It wins at**: "What does the Stripe API do?" — any public library, any version, always up to date.

**It cannot**: serve private docs, store team knowledge, run offline, or be self-hosted.

## What Chub does

Chub is the team context layer for AI agents. Its core thesis is that the most valuable context for a coding agent is not what the Stripe docs say — it's what *your team* knows about Stripe:

- The webhook endpoint requires raw body parsing — don't use `express.json()` before it.
- You're locked to Next.js 15.0.0 until the app-router migration is complete.
- The internal auth service expects JWTs signed with RS256, not HS256.

None of that is on the public internet. Context7 cannot provide it. Chub can.

## The right mental model

```
Context7  →  "What does the Stripe API do?"     (public, crawled, always fresh)
Chub      →  "How does our team use Stripe?"     (private, annotated, version-pinned)
```

They are complementary. Register both MCP servers and the agent uses whichever fits the query.

## Running both

In your MCP config (e.g. `.claude/settings.json`):

```json
{
  "mcpServers": {
    "chub": { "command": "chub", "args": ["mcp"] },
    "context7": { "command": "npx", "args": ["-y", "@upstash/context7-mcp"] }
  }
}
```

The agent calls `chub_get` for your internal API docs, pinned versions, and team annotations. It calls `resolve-library-id` / `query-docs` for up-to-date public library reference.

## What Chub deliberately does not try to do

Chub does not crawl upstream library repos. It cannot compete with Context7 on public library freshness, and it does not try to. The curated registry (1,560+ entries) covers the most common libraries at known-good versions — sufficient for most needs, and supplemented by Context7 when currency matters.

If you need always-fresh public docs: use Context7.
If you need your team's knowledge in the agent's context: use Chub.
