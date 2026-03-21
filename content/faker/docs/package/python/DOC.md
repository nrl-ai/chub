---
name: package
description: "Faker Python package guide for generating fake test data, localized fixtures, and custom providers"
metadata:
  languages: "python"
  versions: "40.8.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "faker,python,test-data,fixtures,mocking,localization"
---

# Faker Python Package Guide

## Golden Rule

Use `Faker` for synthetic test data and fixtures, and seed it explicitly whenever output needs to be reproducible. For version `40.8.0`, trust PyPI for Python compatibility: as of March 12, 2026, the package requires `Python >=3.10` even though the docs landing page still says Python 3.8+.

## Install

Pin the version if your project needs deterministic provider behavior:

```bash
python -m pip install "Faker==40.8.0"
```

Common alternatives:

```bash
uv add "Faker==40.8.0"
poetry add "Faker==40.8.0"
```

`pip install faker` also works because PyPI names are case-insensitive, but the canonical project name is `Faker`.

PyPI exposes a `tzdata` extra. Most projects do not need to install it manually, but it can matter on platforms that do not ship a usable timezone database:

```bash
python -m pip install "Faker[tzdata]==40.8.0"
```

## Initialize And Generate Data

Create one generator and call provider methods on it:

```python
from faker import Faker

fake = Faker()

user = {
    "name": fake.name(),
    "email": fake.email(),
    "company": fake.company(),
    "address": fake.address(),
}
```

For repeatable output across runs, seed before generating values:

```python
from faker import Faker

Faker.seed(4321)
fake = Faker()

print(fake.name())
print(fake.name())
```

Important seeding rules for modern Faker:

- Use `Faker.seed(...)` for the shared class-level RNG.
- Use `fake.seed_instance(...)` for an isolated RNG on one generator instance.
- Use `fake.seed_locale("en_US", ...)` when you need deterministic output for one locale inside a multi-locale generator.
- Do not call `fake.seed(...)` on an instance. Current Faker documents that this raises `TypeError`.

## Locale And Multi-Locale Usage

Use a specific locale when your tests depend on localized names, addresses, phone numbers, or identifiers:

```python
from faker import Faker

fake = Faker("it_IT")
print(fake.name())
print(fake.address())
```

Faker also supports multiple locales in one generator:

```python
from collections import OrderedDict
from faker import Faker

fake = Faker(OrderedDict([
    ("en_US", 1),
    ("en_PH", 2),
    ("ja_JP", 3),
]))

print(fake.name())          # weighted locale selection
print(fake["ja_JP"].name()) # direct locale access
```

Practical guidance:

- Locale strings normalize hyphenated and underscored forms, but use one style consistently in your codebase.
- A provider may exist for one locale and not another. In multi-locale mode, method dispatch falls back to a locale that supports the provider; direct indexed access like `fake["en_PH"].zipcode()` can still raise `AttributeError`.
- If you need raw speed for bulk fixture generation, pass `use_weighting=False`. The default `True` tries to mimic real-world frequencies and is slower.

## Common Provider Patterns

Most day-to-day usage comes from bundled providers:

- person and identity: `name()`, `first_name()`, `ssn()`, `passport_number()`
- internet and auth-ish placeholders: `email()`, `ipv4_private()`, `user_name()`
- company and profile data: `company()`, `job()`, `profile()`
- text and content: `sentence()`, `paragraph()`, `text()`
- dates and numbers: `date_time()`, `uuid4()`, `random_int()`, `pydecimal()`

Example:

```python
from faker import Faker

fake = Faker()

record = {
    "id": fake.uuid4(),
    "full_name": fake.name(),
    "username": fake.user_name(),
    "email": fake.email(),
    "created_at": fake.date_time_this_year().isoformat(),
}
```

The standard providers index in the official docs is the fastest way to check whether a method belongs to `address`, `internet`, `person`, `phone_number`, `profile`, `python`, or another provider family.

## Unique Values

Use `.unique` only when collisions actually break your test:

```python
from faker import Faker

fake = Faker()
emails = [fake.unique.email() for _ in range(100)]
fake.unique.clear()
```

Pitfalls:

- Uniqueness is scoped to one `Faker` instance.
- Small output spaces will exhaust quickly and raise `UniquenessException`.
- Only hashable arguments and return values work with `.unique`.
- In multi-locale mode, you can scope uniqueness per locale with `fake.unique["en_US"].first_name()`.

## Custom And Dynamic Providers

Use a custom provider when project-specific fake values should look like first-class Faker methods:

```python
from faker import Faker
from faker.providers import BaseProvider

class PlanProvider(BaseProvider):
    def plan_code(self) -> str:
        return self.random_element(["free", "team", "enterprise"])

fake = Faker()
fake.add_provider(PlanProvider)

print(fake.plan_code())
```

For data loaded from a list or external source, use `DynamicProvider`:

```python
from faker import Faker
from faker.providers import DynamicProvider

department_provider = DynamicProvider(
    provider_name="department",
    elements=["sales", "support", "platform"],
)

fake = Faker()
fake.add_provider(department_provider)

print(fake.department())
```

## Pytest Integration

Faker ships a `pytest` plugin with a `faker` fixture. The important operational detail is that the fixture is seeded for every test:

```python
import pytest

@pytest.fixture(scope="module")
def faker_seed():
    return 12345

def test_profile_shape(faker):
    profile = faker.profile()
    assert "mail" in profile
```

Useful behavior from the official fixture docs:

- By default, the plugin reuses a session-scoped Faker instance for efficiency.
- Activating a `faker_locale` fixture causes tests to get a new Faker instance for that scope.
- Activating a `faker_seed` fixture changes the seed for that scope.
- If you need to reseed mid-test, call `faker.seed_instance(...)` on the fixture value.

## CLI Usage

Installing Faker also installs a `faker` CLI:

```bash
faker name
faker -l de_DE address
faker -r 3 -s ";" email
python -m faker profile ssn,birthdate
```

Use the CLI for quick fixture inspection or to snapshot provider output while designing tests.

## Configuration And Environment

Faker is a local data-generation library:

- no API key
- no remote service auth
- no network calls in normal use

Configuration is code-level:

- locale choice
- seeding strategy
- provider selection
- `use_weighting`
- custom providers

For test suites, make seed and locale explicit in fixtures instead of relying on global process state.

## Common Pitfalls

- `Faker` is the package name on PyPI, but imports are `from faker import Faker`.
- Do not assume docs examples are fully version-synchronized across all stable subpages. The stable root currently shows `40.8.0`, while the pytest fixtures page still renders a `40.4.0` header.
- Provider availability is locale-specific. Check the provider docs or test the exact method against the locale you plan to use.
- Reproducibility depends on Faker version as well as seed. Pin the package version when tests assert exact generated values.
- Shared class-level seeding can leak assumptions across tests. Prefer `seed_instance()` for isolated fixtures.
- `.unique` is not free. It adds bookkeeping overhead and can fail once the value space is exhausted.

## Version-Sensitive Notes

- PyPI shows `Faker 40.8.0` uploaded on March 4, 2026.
- PyPI metadata now requires `Python >=3.10`.
- The upstream changelog records support drops at `25.0.0` for Python 3.7, `36.0.0` for Python 3.8, and `38.0.0` for Python 3.9. If your runtime is older, you need an older Faker line.
- The changelog for `40.1.0` added selective uniqueness helpers, and `40.1.2` changed `tzdata` handling to be conditional by platform.

## Official Sources

- Docs root: `https://faker.readthedocs.io/en/stable/`
- Faker class details: `https://faker.readthedocs.io/en/stable/fakerclass.html`
- Pytest fixtures: `https://faker.readthedocs.io/en/stable/pytest-fixtures.html`
- Standard providers index: `https://faker.readthedocs.io/en/stable/providers.html`
- PyPI package page: `https://pypi.org/project/Faker/`
- Upstream changelog: `https://github.com/joke2k/faker/blob/master/CHANGELOG.md`
