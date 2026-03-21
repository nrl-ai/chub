---
name: package
description: "langserve package guide for Python FastAPI deployment, RemoteRunnable clients, auth boundaries, and 0.3.3 migration notes"
metadata:
  languages: "python"
  versions: "0.3.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "langserve,langchain,fastapi,runnable,api,python"
---

# langserve Python Package Guide

## When To Use LangServe

Use `langserve` when you need to expose a LangChain `Runnable` as HTTP endpoints on top of FastAPI and call it from Python or JavaScript clients.

It is most useful for:

- internal tools that already use LangChain `Runnable` graphs
- thin API wrappers around a prompt, retriever, or chain
- teams that want a built-in playground and schema endpoints for a runnable

For new greenfield deployments, the maintainers now recommend **LangGraph Platform** instead of LangServe. Keep using `langserve` only if you already depend on it or specifically want its FastAPI route model.

## Version-Sensitive Notes

- PyPI currently lists `0.3.3`, so the version used here matches current upstream as of 2026-03-12.
- The `langchain-ai/langserve` repository is archived. The archive banner on GitHub says it was archived on **March 10, 2026**.
- The maintainer migration guide says new users should use LangGraph Platform and migrate off LangServe over time.
- The docs URL now redirects into the general LangChain documentation overview instead of a maintained LangServe-specific doc tree. For practical usage, the archived repository README and migration guide are the canonical maintainer sources.
- `langserve>=0.3` supports Pydantic 2 fully.
- If you combine LangServe with older LangChain packages, watch the Pydantic boundary carefully:
  - with `langchain-core<0.3`, keep `pydantic==1.10.17`
  - do not mix Pydantic 1 and 2 namespaces inside the same FastAPI app unless you have verified the OpenAPI behavior you need
- The README notes that FastAPI cannot generate OpenAPI docs for LangServe invoke/batch/stream endpoints when LangServe uses Pydantic 2 but other FastAPI routes still use Pydantic 1 models. Upgrading to `langchain-core>=0.3` is the clean path.

## Install

Install the package plus the web server pieces explicitly:

```bash
python -m pip install "langserve==0.3.3" fastapi uvicorn
```

If you want the package's convenience extras from the maintainer README:

```bash
python -m pip install "langserve[all]==0.3.3"
```

What you still need to choose separately:

- your runnable implementation, often from `langchain-core`
- any provider packages such as model integrations
- deployment runtime such as `uvicorn` or another ASGI server

## Minimal Server Setup

The core server API is `add_routes(app, runnable, path=...)`.

```python
from fastapi import FastAPI
from langchain_core.runnables import RunnableLambda
from langserve import add_routes

def shout(text: str) -> str:
    return text.upper()

app = FastAPI(title="LangServe Example")
runnable = RunnableLambda(shout)

add_routes(app, runnable, path="/shout")
```

Run it:

```bash
uvicorn server:app --reload
```

That route mount creates these useful endpoints under `/shout`:

- `POST /invoke`
- `POST /batch`
- `POST /stream`
- `POST /stream_log`
- `POST /astream_events`
- `GET /input_schema`
- `GET /output_schema`
- `GET /config_schema`
- `GET /playground/`

## Remote Client Usage

Use `RemoteRunnable` when application code should call a LangServe deployment like a normal runnable.

```python
from langserve import RemoteRunnable

remote = RemoteRunnable("http://localhost:8000/shout/")

print(remote.invoke("hello"))
print(remote.batch(["one", "two"]))
```

Streaming works through the same client object:

```python
from langserve import RemoteRunnable

remote = RemoteRunnable("http://localhost:8000/shout/")

for chunk in remote.stream("stream me"):
    print(chunk)
```

Use the remote client instead of hand-building JSON requests unless you are integrating from a non-LangChain HTTP client.

## Serving Structured Inputs

In real apps, the runnable usually takes structured input rather than a single string. Keep the example schema explicit so the generated endpoint contracts stay predictable.

```python
from typing import TypedDict

from fastapi import FastAPI
from langchain_core.runnables import RunnableLambda
from langserve import add_routes

class JokeInput(TypedDict):
    topic: str

def make_joke(data: JokeInput) -> str:
    topic = data["topic"]
    return f"Why did the {topic} cross the road? To get to the runnable."

app = FastAPI(title="Joke Service")
runnable = RunnableLambda(make_joke)

add_routes(app, runnable, path="/joke", input_type=JokeInput)
```

Practical note:

- `langserve` only exposes the runnable boundary.
- Secrets, model credentials, and provider-specific setup belong to the underlying code inside the runnable, not to LangServe itself.

## Route Configuration

The most important `add_routes(...)` knobs for production work are:

- `path`: mount point for the runnable
- `input_type` and `output_type`: stabilize schemas when automatic inference is too loose
- `config_keys`: explicitly allow only the client-configurable runnable config keys you want exposed
- `enabled_endpoints`: reduce the surface area if you do not want every generated endpoint
- `disabled_endpoints`: selectively turn off generated endpoints
- `playground_type`: use `"chat"` for chat-style playground behavior when the input shape matches the README requirements

Use explicit schemas when the runnable input is complex:

```python
from typing import TypedDict

from fastapi import FastAPI
from langserve import add_routes

class JokeInput(TypedDict):
    topic: str

app = FastAPI()

add_routes(
    app,
    runnable,
    path="/joke",
    input_type=JokeInput,
)
```

## Browser Access And CORS

If the playground or a browser client runs on a different origin than the API, add FastAPI CORS middleware. The maintainer README explicitly calls this out.

```python
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
    expose_headers=["*"],
)
```

Without CORS, the playground or browser app may fail even though direct Python calls work.

## Auth And Per-Request Config

LangServe does not provide a package-level auth system. Treat auth and authorization as FastAPI concerns.

Use these boundaries:

- authenticate requests with FastAPI dependencies, middleware, or a gateway in front of the app
- keep secrets and tenant selection server-side
- only expose client-settable config through `config_keys`
- use `per_req_config_modifier` to derive runnable config from the authenticated request

The `add_routes(...)` source defines `per_req_config_modifier` as a callable that receives `(config, request)` and returns the config dict used for that request.

Pattern to follow:

```python
from fastapi import FastAPI, Request
from langserve import add_routes

def attach_request_config(config: dict, request: Request) -> dict:
    user_id = request.headers.get("x-user-id")
    if user_id:
        configurable = dict(config.get("configurable", {}))
        configurable["user_id"] = user_id
        config = dict(config)
        config["configurable"] = configurable
    return config

add_routes(
    app,
    runnable,
    path="/secure-runnable",
    config_keys=["configurable"],
    per_req_config_modifier=attach_request_config,
)
```

Do not let clients directly choose sensitive config such as provider API keys, internal datastore selectors, or unrestricted tracing flags.

## LangSmith Feedback And Trace Links

LangServe can expose feedback and public trace link endpoints for the playground, but the README treats these as optional LangSmith-integrated features.

Typical environment variables for that path are:

```bash
export LANGCHAIN_TRACING_V2=true
export LANGCHAIN_PROJECT="your-project"
export LANGCHAIN_API_KEY="ls__..."
```

Security note from the maintainer README:

- do not enable `enable_public_trace_link_endpoint=True` on a public internet-facing app
- public trace links can expose internal chain data and intermediate steps

## Common Pitfalls

- The old LangServe examples use older LangChain import paths in places. Validate the runnable's provider imports against the current packages in your own project.
- `RemoteRunnable` should point at the runnable base path, for example `http://localhost:8000/joke/`, not directly at `/invoke`.
- If browser calls fail but Python calls succeed, check CORS before debugging the runnable itself.
- If OpenAPI docs look broken in mixed FastAPI apps, audit your Pydantic 1 versus 2 usage first.
- If you want a smaller public surface, disable unused generated endpoints instead of exposing every route by default.
- The package is archived, so treat third-party blog posts with caution and prefer the archived maintainer README plus PyPI metadata.

## Canonical Sources

- PyPI package page: `https://pypi.org/project/langserve/`
- Archived maintainer repository and README: `https://github.com/langchain-ai/langserve`
- Migration guide: `https://github.com/langchain-ai/langserve/blob/main/MIGRATION.md`
- Docs URL now redirecting away from LangServe-specific docs: `https://python.langchain.com/docs/langserve/`
