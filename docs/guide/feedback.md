# Feedback & Quality Signals

Feedback helps doc maintainers understand what's working and what needs improvement. It's separate from [annotations](/guide/annotations) — annotations help your team, feedback helps the community.

## How it works

Rate any doc or skill with a thumbs up or down, optionally with structured labels and a comment:

```sh
chub feedback stripe/api up "Clear examples, well structured"
chub feedback openai/chat down --label outdated --label wrong-examples
```

Feedback is sent to the registry so maintainers can prioritize improvements.

## Labels

Labels pinpoint specific qualities. Use `--label` (repeatable) to attach one or more:

**Positive:**
- `accurate` — content is correct
- `well-structured` — easy to follow
- `helpful` — solved the problem
- `good-examples` — code examples work as shown

**Negative:**
- `outdated` — content references old API versions
- `inaccurate` — content is factually wrong
- `incomplete` — missing important information
- `wrong-examples` — code examples don't work
- `wrong-version` — version mismatch
- `poorly-structured` — hard to navigate

## CLI usage

```sh
# Simple up/down rating
chub feedback stripe/api up
chub feedback stripe/api down

# With labels
chub feedback openai/chat down --label outdated --label wrong-examples

# With comment
chub feedback stripe/api up "Webhook verification section is excellent"

# Target a specific file within a doc
chub feedback acme/widgets down --file references/advanced.md --label incomplete

# Include agent context (for tracking which agents find docs useful)
chub feedback stripe/api up --agent "claude-code" --model "claude-sonnet-4"

# Check feedback status
chub feedback --status
```

## MCP tool

Agents can submit feedback via the `chub_feedback` MCP tool:

```json
{ "id": "openai/chat", "rating": "down",
  "comment": "Missing streaming example for Python", "labels": ["missing-example"] }
```

## Disabling feedback

Feedback submission is opt-in behavior (only happens when you explicitly run `chub feedback`). To disable the command entirely:

```yaml
# ~/.chub/config.yaml
feedback: false
```

Or via environment variable: `CHUB_FEEDBACK=0`.

Check status with `chub feedback --status`.

## Feedback vs Annotations

| | Annotations | Feedback |
|---|---|---|
| **For whom** | Your team's agents | Doc authors and maintainers |
| **Where stored** | Locally, in git, or on server | In the registry |
| **Purpose** | Don't repeat mistakes | Improve the content for everyone |
| **Visible to** | You, your team, or your org | Maintainers |
| **Effect** | Shows on future fetches | Authors update the doc |

Both matter. Annotations help your agent today. Feedback helps everyone tomorrow.
