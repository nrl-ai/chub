---
name: package
description: "SQLModel package guide for Python projects using the official SQLModel docs"
metadata:
  languages: "python"
  versions: "0.0.37"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "sqlmodel,python,sqlalchemy,pydantic,fastapi,orm,sqlite"
---

# SQLModel Python Package Guide

## Golden Rule

Use `sqlmodel` when you want one typed model layer that works as both a Pydantic data model and a SQLAlchemy ORM mapping. For `0.0.37`, the important compatibility floor is Python `>=3.10`, and support for Pydantic v1 is already gone.

## Install

Pin the package version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "sqlmodel==0.0.37"
```

Common alternatives:

```bash
uv add "sqlmodel==0.0.37"
poetry add "sqlmodel==0.0.37"
```

If you are not using SQLite, you also need a SQLAlchemy-compatible database driver for your backend.

## Initialize An Engine And Table Model

The official docs start with `Field`, `SQLModel`, `create_engine`, and `Session`. A minimal setup looks like this:

```python
import os

from sqlmodel import Field, Session, SQLModel, create_engine, select

DATABASE_URL = os.getenv("DATABASE_URL", "sqlite:///database.db")

connect_args = {"check_same_thread": False} if DATABASE_URL.startswith("sqlite") else {}
engine = create_engine(DATABASE_URL, echo=False, connect_args=connect_args)

class Hero(SQLModel, table=True):
    id: int | None = Field(default=None, primary_key=True)
    name: str = Field(index=True)
    secret_name: str
    age: int | None = Field(default=None, index=True)

def create_db_and_tables() -> None:
    SQLModel.metadata.create_all(engine)
```

Notes:

- `table=True` is what makes the model a mapped table instead of only a data model.
- `Field(default=None, primary_key=True)` is the standard pattern for auto-generated integer primary keys.
- `SQLModel.metadata.create_all(engine)` is fine for quickstarts and tests; it is not a replacement for migrations.

## Core CRUD Pattern

The normal synchronous workflow is "open session, add or query models, commit, refresh when needed":

```python
def create_hero() -> Hero:
    hero = Hero(name="Deadpond", secret_name="Dive Wilson", age=32)
    with Session(engine) as session:
        session.add(hero)
        session.commit()
        session.refresh(hero)
        return hero

def read_hero_by_name(name: str) -> Hero | None:
    with Session(engine) as session:
        statement = select(Hero).where(Hero.name == name)
        return session.exec(statement).first()

def list_heroes(offset: int = 0, limit: int = 100) -> list[Hero]:
    with Session(engine) as session:
        statement = select(Hero).offset(offset).limit(limit)
        return list(session.exec(statement))
```

Useful patterns from the official tutorial:

- Use `session.exec(select(...))` for queries instead of raw `session.execute(...)` in normal SQLModel code.
- Use `session.get(Model, primary_key)` when loading one row by primary key.
- Call `session.refresh(obj)` after `commit()` when you need generated fields like `id`.

## FastAPI Integration

SQLModel is designed to work directly with FastAPI. The official docs use separate models for request input, public output, and the table model:

```python
from fastapi import Depends, FastAPI, HTTPException, Query
from sqlmodel import Field, Session, SQLModel, create_engine, select

sqlite_url = "sqlite:///database.db"
connect_args = {"check_same_thread": False}
engine = create_engine(sqlite_url, echo=True, connect_args=connect_args)

class HeroBase(SQLModel):
    name: str = Field(index=True)
    secret_name: str
    age: int | None = Field(default=None, index=True)

class Hero(HeroBase, table=True):
    id: int | None = Field(default=None, primary_key=True)

class HeroCreate(HeroBase):
    pass

class HeroPublic(HeroBase):
    id: int

class HeroUpdate(SQLModel):
    name: str | None = None
    secret_name: str | None = None
    age: int | None = None

def get_session():
    with Session(engine) as session:
        yield session

app = FastAPI()

@app.on_event("startup")
def on_startup() -> None:
    SQLModel.metadata.create_all(engine)

@app.post("/heroes/", response_model=HeroPublic)
def create_hero(*, session: Session = Depends(get_session), hero: HeroCreate):
    db_hero = Hero.model_validate(hero)
    session.add(db_hero)
    session.commit()
    session.refresh(db_hero)
    return db_hero

@app.patch("/heroes/{hero_id}", response_model=HeroPublic)
def update_hero(*, session: Session = Depends(get_session), hero_id: int, hero: HeroUpdate):
    db_hero = session.get(Hero, hero_id)
    if not db_hero:
        raise HTTPException(status_code=404, detail="Hero not found")
    hero_data = hero.model_dump(exclude_unset=True)
    db_hero.sqlmodel_update(hero_data)
    session.add(db_hero)
    session.commit()
    session.refresh(db_hero)
    return db_hero

@app.get("/heroes/", response_model=list[HeroPublic])
def read_heroes(
    *, session: Session = Depends(get_session), offset: int = 0, limit: int = Query(default=100, le=100)
):
    heroes = session.exec(select(Hero).offset(offset).limit(limit)).all()
    return heroes
```

Agent guidance:

- Use `Hero.model_validate(hero_in)` to convert input models into table models.
- Use `hero.model_dump(exclude_unset=True)` plus `db_obj.sqlmodel_update(...)` for PATCH-style updates.
- Keep response models separate from table models when you need to hide internal columns.

## Relationships

Relationships use both a foreign-key column and `Relationship(...)` attributes:

```python
from sqlmodel import Field, Relationship, SQLModel

class Team(SQLModel, table=True):
    id: int | None = Field(default=None, primary_key=True)
    name: str = Field(index=True)
    headquarters: str

    heroes: list["Hero"] = Relationship(back_populates="team")

class Hero(SQLModel, table=True):
    id: int | None = Field(default=None, primary_key=True)
    name: str = Field(index=True)
    secret_name: str
    team_id: int | None = Field(default=None, foreign_key="team.id")

    team: Team | None = Relationship(back_populates="heroes")
```

Use this pattern when you want ORM-style navigation like `hero.team` or `team.heroes`, not just raw foreign-key ids.

## Configuration

SQLModel itself does not have an auth layer or package-specific credential system. The main configuration surfaces are:

- database URL
- SQLAlchemy engine arguments
- app-level model boundaries for create/update/read flows

Example environment setup:

```bash
export DATABASE_URL="sqlite:///./app.db"
```

Important engine notes:

- The FastAPI tutorial uses `connect_args = {"check_same_thread": False}` for SQLite.
- Only use that SQLite-specific argument when the URL is SQLite.
- `echo=True` is useful while debugging SQL locally, but usually too noisy for production.

## Common Pitfalls

- Do not forget `table=True`. Without it, you created a data model, not a mapped table.
- `SQLModel.metadata.create_all()` only creates missing tables. It does not generate or apply schema migrations.
- If you expose table models directly from an API, you can leak fields you did not intend to return. Prefer dedicated public models.
- Relationship attributes do not replace the foreign key column; you still need fields like `team_id` with `foreign_key="team.id"`.
- For SQLite in FastAPI examples, the docs use `check_same_thread=False`; copying that blindly to other databases is unnecessary.

## Version-Sensitive Notes

- The official SQLModel release notes include `0.0.37` as the current documented release in this line.
- `0.0.37` itself is an internal release; the main user-visible compatibility shifts happened earlier in the same line.
- `0.0.36` dropped support for `sqlmodel-slim`. Use `sqlmodel`.
- `0.0.35` dropped Python 3.9, which matches the current PyPI requirement of Python `>=3.10`.
- `0.0.31` dropped support for Pydantic v1. If your project still depends on Pydantic v1, do not upgrade to `0.0.37` without a broader migration plan.
- `0.0.32` fixed support for `Annotated` fields with Pydantic 2.12+, so older blog posts about `Annotated` breakage may be stale.
- The official docs examples use Python 3.10+ syntax such as `int | None` and `list[Hero]`.

## Practical Production Note

The official quickstarts create tables at app startup. That is reasonable for tutorials, local scripts, and tests. For deployed systems, treat schema changes as a separate migration workflow. The SQLModel advanced guide explicitly says migration guidance is still growing, so production migration setup still requires SQLAlchemy ecosystem tooling and project-specific decisions.

## Official Sources

- Docs root: `https://sqlmodel.tiangolo.com/`
- FastAPI session dependency tutorial: `https://sqlmodel.tiangolo.com/tutorial/fastapi/session-with-dependency/`
- Relationships tutorial: `https://sqlmodel.tiangolo.com/tutorial/relationship-attributes/define-relationships-attributes/`
- Advanced guide: `https://sqlmodel.tiangolo.com/advanced/`
- Release notes: `https://sqlmodel.tiangolo.com/release-notes/`
- PyPI: `https://pypi.org/project/sqlmodel/`
- Repository: `https://github.com/fastapi/sqlmodel`
