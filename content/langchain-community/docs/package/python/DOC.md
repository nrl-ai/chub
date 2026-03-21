---
name: package
description: "langchain-community package guide for Python covering community integrations, loaders, vector stores, tools, and practical 0.4.1 usage"
metadata:
  languages: "python"
  versions: "0.4.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "langchain-community,langchain,python,integrations,document-loaders,vectorstores,tools"
---

# langchain-community Python Package Guide

## What It Is

`langchain-community` is the Python package for community and third-party LangChain integrations that do not live in `langchain-core` and are not published as dedicated `langchain-*` provider packages.

Use it for integration surfaces such as:

- document loaders
- vector stores
- tools
- retrievers
- utilities and wrappers

Do not treat it as the default package for every provider. LangChain's current integrations docs center dedicated packages such as `langchain-openai` and other provider-specific packages when they exist.

## Version Notes

- This guide is pinned to `langchain-community==0.4.1`.
- LangChain's Python release policy explicitly calls out that integration packages such as `langchain-community` do not follow the same guarantees as `langchain` and `langchain-core`.
- Pin exact versions if you need reproducible behavior across environments.
- The current LangChain publishing guidance prefers standalone `langchain-*` packages for new integrations, so some older community integrations are transitional or effectively legacy.
- The PyPI project page still contains older prose that refers to the `0.0.x` line; prefer the live package version and current LangChain reference docs over that stale summary text.

## Install

Install the package itself:

```bash
python -m pip install "langchain-community==0.4.1"
```

Common alternatives:

```bash
uv add "langchain-community==0.4.1"
poetry add "langchain-community==0.4.1"
```

In practice you usually install it with the dependency required by the integration you actually use:

```bash
python -m pip install "langchain-community==0.4.1" "langchain-openai"
python -m pip install "langchain-community==0.4.1" "faiss-cpu"
python -m pip install "langchain-community==0.4.1" "duckduckgo-search"
```

Typical package boundaries:

- install `langchain` for higher-level agent and application APIs
- install `langchain-core` for lower-level prompts, runnables, and shared abstractions
- install a dedicated provider package when one exists
- install the backend library required by the community integration you chose

## Setup And Configuration

`langchain-community` does not have one package-wide authentication model. Setup depends on the integration.

Practical rules:

1. Install the backend dependency required by the integration.
2. Set provider-specific environment variables for the provider package or vendor SDK you use.
3. Keep LangSmith tracing separate from model or service credentials.

Example environment setup for a local FAISS workflow that uses OpenAI embeddings:

```bash
export OPENAI_API_KEY="sk-..."
export LANGSMITH_TRACING="true"
export LANGSMITH_API_KEY="lsv2_..."
```

Important boundary:

- credentials usually belong to the provider package or vendor SDK, not to `langchain-community` itself
- many community integrations only work after you install a second package such as `faiss-cpu`, `duckduckgo-search`, `pypdf`, or `unstructured`

## Common Workflows

### Load Local Files With `TextLoader`

Use `TextLoader` to turn a local text file into LangChain `Document` objects:

```python
from langchain_community.document_loaders import TextLoader

loader = TextLoader("notes.txt")
docs = loader.load()

for doc in docs:
    print(doc.page_content[:80])
    print(doc.metadata)
```

If the loader supports it and your input is large, prefer `lazy_load()` over eager `.load()`.

### Build A FAISS Vector Store

The `FAISS` integration in `langchain-community` uses the separate `faiss-cpu` package for the index itself.

```python
from langchain_community.vectorstores import FAISS
from langchain_openai import OpenAIEmbeddings

texts = [
    "LangChain Community contains third-party integrations.",
    "FAISS is useful for local similarity search.",
]

embeddings = OpenAIEmbeddings(model="text-embedding-3-small")
vector_store = FAISS.from_texts(texts, embeddings)

results = vector_store.similarity_search(
    "Which package contains third-party integrations?",
    k=1,
)

for doc in results:
    print(doc.page_content)
```

Useful follow-on calls:

- `similarity_search_with_score(...)` when you need scores
- `save_local(...)` and `load_local(...)` for persistence
- `as_retriever(...)` when you want retriever behavior

### Turn The Vector Store Into A Retriever

`FAISS.as_retriever(...)` is the direct bridge from a local index to retrieval-oriented LangChain components:

```python
retriever = vector_store.as_retriever(
    search_type="mmr",
    search_kwargs={"k": 2, "fetch_k": 4, "lambda_mult": 0.5},
)

docs = retriever.invoke("third-party LangChain integrations")

for doc in docs:
    print(doc.page_content)
```

### Use A Community Tool Wrapper

`DuckDuckGoSearchRun` is a simple example of a tool wrapper from this package:

```python
from langchain_community.tools import DuckDuckGoSearchRun

tool = DuckDuckGoSearchRun()
result = tool.invoke("latest LangChain release policy")
print(result)
```

Install `duckduckgo-search` first, and check the integrations index before adopting a community tool wrapper for a provider that may already have a dedicated package.

## Common Pitfalls

- `langchain-community` is not the first package to assume for every provider; check the integrations overview for a dedicated package first.
- Installing only `langchain-community` is often not enough; many integrations need an extra backend library or vendor SDK.
- The package has no universal API key or shared client object.
- Older LangChain examples on the web often use pre-`1.0` import paths or community wrappers that have since moved into dedicated packages.
- Do not assume `langchain-community` version numbers should match `langchain` or `langchain-core`; these packages are versioned separately.
- Minor-version changes in `0.x` integration packages can change behavior, so pin exact versions when you need stable builds.
- `.load()` is eager; use streaming-friendly loader methods when the loader supports them.

## What To Reach For First

- Use `langchain-community` for loaders, vector stores, tools, utilities, and older shared integrations that still live here.
- Use `langchain-core` for prompts, runnables, tools, and shared interfaces.
- Use `langchain` for higher-level agent and application APIs.
- Use dedicated provider packages for models, embeddings, and provider-owned integrations whenever possible.

## Official Sources Used For This Entry

- PyPI package page: `https://pypi.org/project/langchain-community/`
- PyPI release history: `https://pypi.org/project/langchain-community/#history`
- Maintainer docs URL: `https://python.langchain.com/api_reference/community/`
- Canonical API reference root: `https://reference.langchain.com/python/langchain-community/`
- LangChain integrations overview: `https://docs.langchain.com/oss/python/integrations/providers/overview`
- LangChain install guide: `https://docs.langchain.com/oss/python/langchain/install`
- LangChain Python release policy: `https://docs.langchain.com/oss/python/release-policy`
- LangChain integration publishing guidance: `https://docs.langchain.com/oss/python/contributing/publish-langchain`
