# Private Content Repo

Teams have internal documentation that agents need: private API references, deployment playbooks, coding conventions, auth patterns. This content doesn't belong in a public registry, but agents should be able to search and fetch it just as easily as public docs.

## How It Works

Build your private docs with `chub build`, then add the output as a local source or serve it via `chub serve`. See the [BYOD Guide](byod-guide.md) for the full walkthrough.

```sh
chub build my-content/ -o .chub-local/       # build local registry
chub serve my-content/ --port 4242           # or serve over HTTP
```

Add to `~/.chub/config.yaml` (or `.chub/config.yaml` for team-wide):

```yaml
sources:
  - name: community
    url: https://cdn.aichub.org/v1
  - name: internal
    path: /path/to/.chub-local          # local path
    # or: url: http://localhost:4242    # HTTP server
```

Now `chub search` and `chub get` cover both public and private content seamlessly.

## Team Distribution

Put your content directory in a shared git repo or internal CDN. Every team member points their config at the same source. Private docs and skills are available to every agent without publishing anything publicly.

If a private id collides with a public one, use the `source:` prefix to disambiguate:

```sh
chub get internal:openai/chat           # your internal version
chub get community:openai/chat         # the public version
```

## Enterprise

For enterprise use cases, reach out at info@tbd-domain.com.
