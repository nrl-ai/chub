---
name: package
description: "LlamaIndex package guide for Python RAG pipelines, ingestion, retrieval, and agent workflows"
metadata:
  languages: "python"
  versions: "0.14.16"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "llama-index,rag,agents,retrieval,vector-index,python"
---

# LlamaIndex Python Package Guide

## What This Package Is For

`llama-index` is the Python starter package for building LLM applications around your own data. Use it when you need:

- document loading and chunking
- embeddings and vector indexes
- query engines and chat engines
- agent and workflow building blocks
- pluggable model, embedding, reader, and vector-store integrations

For Python code, the main imports come from `llama_index.core` plus provider-specific integration packages such as `llama_index.llms.openai` or `llama_index.embeddings.huggingface`.

## Install

Install the starter bundle pinned to the package version you want:

```bash
pip install "llama-index==0.14.16"
```

Common alternatives:

```bash
uv add "llama-index==0.14.16"
poetry add "llama-index==0.14.16"
```

Verify what is installed:

```bash
python -c "import llama_index; print(llama_index.__version__)"
```

### Lean Install Pattern

The upstream installation guide recommends installing only the core package plus the integrations you actually need when you want a smaller dependency set:

```bash
pip install llama-index-core
pip install llama-index-llms-openai
pip install llama-index-embeddings-openai
```

For local models:

```bash
pip install llama-index-core
pip install llama-index-llms-ollama
pip install llama-index-embeddings-huggingface
```

Use the single `llama-index` starter package when you want the standard getting-started experience. Use `llama-index-core` plus explicit integrations when dependency size and provider control matter.

## Setup And Auth

LlamaIndex itself does not have one global platform API key. Authentication depends on the providers you plug in.

### OpenAI-backed setup

The official starter examples assume OpenAI for both the LLM and embeddings. Set provider configuration explicitly instead of relying on implicit defaults:

```bash
export OPENAI_API_KEY="..."
export OPENAI_MODEL="your-openai-chat-model"
export OPENAI_EMBED_MODEL="text-embedding-3-small"
```

### Local-model setup

For local development with Ollama plus Hugging Face embeddings:

```bash
ollama serve
ollama pull llama3.1
pip install llama-index-llms-ollama llama-index-embeddings-huggingface
```

No LlamaIndex-specific auth is required for that setup, but the Ollama server must be running and the model must already exist locally.

## Core Usage: Build A Queryable Index

The smallest useful flow is:

1. load documents
2. choose an LLM and embedding model
3. build an index
4. query it

```python
import os

from llama_index.core import Settings, SimpleDirectoryReader, VectorStoreIndex
from llama_index.embeddings.openai import OpenAIEmbedding
from llama_index.llms.openai import OpenAI

Settings.llm = OpenAI(model=os.environ["OPENAI_MODEL"])
Settings.embed_model = OpenAIEmbedding(
    model=os.getenv("OPENAI_EMBED_MODEL", "text-embedding-3-small")
)
Settings.chunk_size = 1024

documents = SimpleDirectoryReader("data").load_data()
index = VectorStoreIndex.from_documents(documents)

query_engine = index.as_query_engine(similarity_top_k=3)
response = query_engine.query("Summarize the main topics in these files.")

print(str(response))
```

Practical notes:

- `SimpleDirectoryReader` is the standard file-based loader for local directories.
- `VectorStoreIndex.from_documents(...)` handles parsing, chunking, embedding, and indexing in one path.
- `Settings` is global. Set it once at process startup, or override per component when different parts of the app use different models.

## Switch To Local Models

The same indexing flow works with local or self-hosted model providers.

```python
from llama_index.core import Settings, SimpleDirectoryReader, VectorStoreIndex
from llama_index.embeddings.huggingface import HuggingFaceEmbedding
from llama_index.llms.ollama import Ollama

Settings.llm = Ollama(model="llama3.1", request_timeout=120.0)
Settings.embed_model = HuggingFaceEmbedding(model_name="BAAI/bge-base-en-v1.5")
Settings.chunk_size = 1024

documents = SimpleDirectoryReader("data").load_data()
index = VectorStoreIndex.from_documents(documents)

response = index.as_query_engine(similarity_top_k=3).query(
    "What are the most important points?"
)
print(str(response))
```

Use this pattern when you need local development, lower cloud dependence, or a non-OpenAI stack.

## Control Chunking And Transformations

You can use global settings for defaults, but explicit transformations are safer when one index needs different chunking from the rest of the application.

```python
from llama_index.core import SimpleDirectoryReader, VectorStoreIndex
from llama_index.core.node_parser import SentenceSplitter

documents = SimpleDirectoryReader("data").load_data()

index = VectorStoreIndex.from_documents(
    documents,
    transformations=[
        SentenceSplitter(chunk_size=512, chunk_overlap=50),
    ],
)
```

Prefer explicit transformations when:

- different indexes in one process need different chunk sizes
- you are tuning retrieval quality
- you want indexing behavior to stay stable even if global `Settings` changes elsewhere

## Persist And Reload An Index

Persist the index when you do not want to recompute embeddings on every run.

```python
from llama_index.core import StorageContext, load_index_from_storage

index.storage_context.persist(persist_dir="storage")

storage_context = StorageContext.from_defaults(persist_dir="storage")
restored_index = load_index_from_storage(storage_context)

response = restored_index.as_query_engine().query("Ask the same corpus a new question.")
print(str(response))
```

Important behavior:

- Reload with the same embedding setup you used when building the index.
- Persisted storage saves index state, not your original source-of-truth documents.
- If you change chunking, embeddings, or vector-store backend, rebuild the index instead of assuming old persisted state is still valid.

## Use An External Vector Store

LlamaIndex can write to external vector stores instead of using the default in-memory path.

```bash
pip install chromadb llama-index-vector-stores-chroma
```

```python
import chromadb

from llama_index.core import SimpleDirectoryReader, StorageContext, VectorStoreIndex
from llama_index.vector_stores.chroma import ChromaVectorStore

documents = SimpleDirectoryReader("data").load_data()

client = chromadb.PersistentClient(path="chroma-db")
collection = client.get_or_create_collection("docs")
vector_store = ChromaVectorStore(chroma_collection=collection)
storage_context = StorageContext.from_defaults(vector_store=vector_store)

index = VectorStoreIndex.from_documents(
    documents,
    storage_context=storage_context,
)
```

The same pattern applies to Pinecone, Qdrant, Weaviate, Postgres, and other officially supported integrations: install the dedicated `llama-index-vector-stores-*` package and wire it through `StorageContext`.

## Common Pitfalls

- Package name and import name differ. Install `llama-index`, but import from `llama_index...`.
- `llama-index` is not every integration. Install the extra `llama-index-llms-*`, `llama-index-embeddings-*`, `llama-index-readers-*`, and `llama-index-vector-stores-*` packages you actually use.
- Prefer `Settings`, not `ServiceContext`. The upstream migration guide introduced `Settings` in `v0.10.0`; older blog posts still show deprecated patterns.
- Do not rely on implicit model defaults. Set `Settings.llm` and `Settings.embed_model` explicitly so upgrades do not silently change behavior.
- `SimpleDirectoryReader` is for files. Database, SaaS, or web-source ingestion usually needs a dedicated reader/integration package.
- Reusing persisted indexes across different embedding models produces bad retrieval results. Keep embeddings stable per index.
- Global `Settings` can leak across tests or mixed workloads. For libraries or complex apps, prefer local overrides or isolate setup code.

## Version-Sensitive Notes For `0.14.16`

- PyPI lists `0.14.16` as the current package version, uploaded on `2026-03-10`.
- The docs URL `https://docs.llamaindex.ai/en/stable/` is still valid, but it redirects to the newer `developers.llamaindex.ai` docs host.
- The current framework docs emphasize the package split: `llama-index-core` for the base framework and add-on packages for providers and vector stores.
- The durable API pattern for new code is `Settings` plus `llama_index.core`. Treat `ServiceContext` examples as legacy unless you are maintaining an older codebase.

## Official Sources

- Docs home: `https://developers.llamaindex.ai/python/framework/`
- Installation: `https://developers.llamaindex.ai/python/framework/getting_started/installation/`
- Starter tutorial: `https://developers.llamaindex.ai/python/framework/getting_started/starter_example/`
- Local models: `https://developers.llamaindex.ai/python/framework/getting_started/starter_example_local/`
- Using `Settings`: `https://developers.llamaindex.ai/python/framework/module_guides/supporting_modules/settings/`
- Persisting and loading: `https://developers.llamaindex.ai/python/framework/understanding/rag/storing/`
- ServiceContext migration: `https://developers.llamaindex.ai/python/framework/module_guides/supporting_modules/service_context_migration/`
- PyPI package: `https://pypi.org/project/llama-index/`
