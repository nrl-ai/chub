---
name: package
description: "Gensim package guide for Python with corpus setup, embeddings, topic models, downloader usage, and 4.4.0 migration notes"
metadata:
  languages: "python"
  versions: "4.4.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "gensim,python,nlp,embeddings,word2vec,lda,topic-modeling"
---

# Gensim Python Package Guide

## What It Is

`gensim` is a Python library for topic modeling, document indexing, similarity retrieval, and vector-space NLP workflows. It is most commonly used for:

- streaming text corpora through memory-efficient training pipelines
- building dictionaries and bag-of-words corpora
- training or loading embedding models such as `Word2Vec`, `FastText`, and `Doc2Vec`
- topic modeling with `LdaModel`
- loading pre-trained datasets and vectors with `gensim.downloader`

Use it when a project needs classical NLP pipelines, embedding training, similarity search, or topic models without moving immediately to transformer tooling.

## Install

Pin the package version if you need behavior that matches this guide:

```bash
python -m pip install "gensim==4.4.0"
```

If you use `uv`:

```bash
uv add "gensim==4.4.0"
```

Runtime notes:

- `gensim` depends on `numpy`, `scipy`, and `smart_open`.
- Large-model performance depends heavily on the NumPy/BLAS stack available in your environment.
- `4.4.0` raises the floor to Python `3.9+` and adds compatibility updates for newer NumPy releases.

## Golden Rules

- Feed training code tokenized sentences or streamed corpora, not one giant in-memory string.
- Use `Dictionary` plus `doc2bow()` for bag-of-words / topic-model pipelines.
- Query word vectors through `model.wv`, not the full training model object.
- Save trained artifacts with `.save()` and reload them with the class-specific `.load()`.
- When porting old examples, expect `gensim 4.x` naming changes such as `vector_size`, `epochs`, `index_to_key`, and `key_to_index`.

## Core Workflow

The common `gensim` pipeline is:

1. tokenize documents into lists of strings
2. build a `Dictionary`
3. convert documents to bag-of-words with `doc2bow()`
4. train a model such as TF-IDF, LSI, or LDA
5. persist the model and dictionary for reuse

### Dictionary And Bag-Of-Words Setup

```python
from gensim.corpora import Dictionary

documents = [
    ["human", "interface", "computer"],
    ["survey", "user", "computer", "system", "response", "time"],
    ["graph", "minors", "trees"],
]

dictionary = Dictionary(documents)
dictionary.filter_extremes(no_below=1, no_above=0.8)

bow_corpus = [dictionary.doc2bow(doc) for doc in documents]

print(dictionary.token2id)
print(bow_corpus[0])
```

Use this representation when you want classic vector-space models, topic models, or similarity indexes.

### Topic Modeling With LDA

```python
from gensim.corpora import Dictionary
from gensim.models import LdaModel

texts = [
    ["bank", "loan", "credit", "money"],
    ["river", "bank", "water", "flood"],
    ["loan", "interest", "debt", "credit"],
]

dictionary = Dictionary(texts)
corpus = [dictionary.doc2bow(text) for text in texts]

lda = LdaModel(
    corpus=corpus,
    id2word=dictionary,
    num_topics=2,
    passes=20,
    random_state=42,
)

for topic_id, topic in lda.print_topics():
    print(topic_id, topic)

doc_topics = lda.get_document_topics(corpus[0])
print(doc_topics)
```

Practical notes:

- pass `id2word=dictionary` so topic output uses tokens instead of raw ids
- set `random_state` when you need repeatable output in tests or docs
- increase `passes` and train on a real corpus before evaluating topic quality

### Training Word2Vec

```python
from gensim.models import Word2Vec

sentences = [
    ["human", "interface", "computer"],
    ["survey", "user", "computer", "system"],
    ["graph", "trees", "minors"],
    ["human", "system", "response"],
]

model = Word2Vec(
    sentences=sentences,
    vector_size=100,
    window=5,
    min_count=1,
    workers=1,
    epochs=20,
)

print(model.wv.most_similar("computer", topn=3))
print(model.wv["human"][:5])
```

Important API shape:

- use `vector_size`, not the old `size`
- use `epochs`, not the old `iter`
- similarity lookup lives on `model.wv`

### Loading Pretrained Assets With `gensim.downloader`

```python
import gensim.downloader as api

available = api.info()
print("glove-wiki-gigaword-50" in available["models"])

vectors = api.load("glove-wiki-gigaword-50")
print(vectors.most_similar("cat", topn=5))
```

Use the downloader when you need a quick public baseline model or toy dataset. It is convenient for experiments, tests, and reproducing tutorial code.

## Similarity And Transform Pipelines

For document similarity, combine a corpus transform with a similarity index:

```python
from gensim.corpora import Dictionary
from gensim.models import TfidfModel
from gensim.similarities import MatrixSimilarity

texts = [
    ["apple", "banana", "fruit"],
    ["dog", "cat", "pet"],
    ["apple", "orange", "fruit"],
]

dictionary = Dictionary(texts)
corpus = [dictionary.doc2bow(text) for text in texts]

tfidf = TfidfModel(corpus)
index = MatrixSimilarity(tfidf[corpus], num_features=len(dictionary))

query_bow = dictionary.doc2bow(["apple", "fruit"])
scores = index[tfidf[query_bow]]
print(list(enumerate(scores)))
```

This pattern is useful when you need lightweight semantic retrieval without adding a vector database.

## Persistence

Persist both the model and the vocabulary objects you need for inference:

```python
from gensim.corpora import Dictionary
from gensim.models import Word2Vec

sentences = [["hello", "world"], ["hello", "gensim"]]

dictionary = Dictionary(sentences)
model = Word2Vec(sentences=sentences, min_count=1)

dictionary.save("dictionary.gensim")
model.save("word2vec.model")

loaded_dictionary = Dictionary.load("dictionary.gensim")
loaded_model = Word2Vec.load("word2vec.model")

print(loaded_dictionary.token2id)
print(loaded_model.wv.most_similar("hello"))
```

Large-array loading note:

- `SaveLoad.load(..., mmap="r")` can memory-map large arrays for sharing between processes.
- Memory mapping does not work for compressed files such as `.gz` or `.bz2`.

## Configuration And Environment

### Downloader Cache Location

`gensim.downloader` stores downloaded assets under `~/gensim-data` by default. Override that location with `GENSIM_DATA_DIR` before importing or downloading models:

```bash
export GENSIM_DATA_DIR=/srv/shared/gensim-data
```

You can inspect downloader metadata from the command line:

```bash
python -m gensim.downloader --info
python -m gensim.downloader --download glove-wiki-gigaword-50
```

### Authentication

`gensim` itself has no package-level authentication or API key flow.

Practical implication:

- public downloader assets require no auth
- local filesystem workflows require no auth
- if you open remote objects through `smart_open`-backed URLs such as S3, configure those storage credentials through the underlying provider tooling in your runtime environment

## Common Pitfalls

### Mixing Up Training Models And Keyed Vectors

Use the right object for the job:

- `Word2Vec`, `FastText`, and `Doc2Vec` are training-capable model classes
- `KeyedVectors` is the light-weight structure for querying vectors

If you load vectors from `load_word2vec_format()` or the downloader, you usually get queryable vectors, not a fully trainable model.

### Porting Pre-4.x Examples Without Translating Names

Old blog posts often use removed or renamed APIs. Common replacements in `gensim 4.x`:

- `size` -> `vector_size`
- `iter` -> `epochs`
- `model.wv.vocab` -> `model.wv.key_to_index` and vector attributes via `get_vecattr()`
- `model.wv.index2word` -> `model.wv.index_to_key`
- direct similarity methods on the training model -> `model.wv.<method>`

If copied code references `vocab`, `index2word`, or `size`, translate it before debugging anything else.

### Forgetting To Stream Large Corpora

Many `gensim` APIs accept iterables. Use that to your advantage for large datasets:

- prefer generators or corpus iterators over building giant intermediate lists
- save reusable corpora in `MmCorpus` or another serialized format when training repeatedly
- keep tokenization outside the training loop so retraining is deterministic

### Treating The Docs Site Version As The Exact Package Version

At the time of this guide:

- package metadata and release tags point to `4.4.0`
- the docs site header still shows `4.3.3`

Do not assume that page chrome is the authoritative package version. When behavior matters, cross-check PyPI metadata and the `4.4.0` release page.

## Version-Sensitive Notes For 4.4.0

- `4.4.0` is the package version on PyPI for this entry.
- Upstream `4.4.0` release notes call out support for NumPy `2.0`.
- PyPI metadata for `4.4.0` requires Python `>=3.9`.
- The upstream repository describes `gensim` as being in stable maintenance mode, so expect incremental fixes more often than major new surfaces.

## Official Sources Used

- Docs root: `https://radimrehurek.com/gensim/`
- API reference: `https://radimrehurek.com/gensim/apiref.html`
- Word2Vec tutorial: `https://radimrehurek.com/gensim/auto_examples/tutorials/run_word2vec.html`
- Topics and transformations tutorial: `https://radimrehurek.com/gensim/auto_examples/core/run_topics_and_transformations.html`
- Downloader how-to: `https://radimrehurek.com/gensim/auto_examples/howtos/run_downloader_api.html`
- `4.4.0` release page: `https://github.com/piskvorky/gensim/releases/tag/4.4.0`
- Package registry: `https://pypi.org/project/gensim/`
