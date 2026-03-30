"""Chub: high-performance curated docs for AI coding agents."""

__version__ = "0.1.23"

import asyncio
import json
import os
import platform
import subprocess
import sys
from typing import Any, Dict, List, Optional


def _get_binary_path() -> str:
    """Resolve the path to the native chub binary bundled in this wheel."""
    pkg_dir = os.path.dirname(os.path.abspath(__file__))
    exe = "chub.exe" if platform.system() == "Windows" else "chub"
    return os.path.join(pkg_dir, "bin", exe)


def main() -> None:
    binary = _get_binary_path()
    if not os.path.isfile(binary):
        print(
            f"chub binary not found at {binary}\n"
            "This likely means the wheel was not built for your platform.\n"
            "Please install the correct platform wheel or build from source.",
            file=sys.stderr,
        )
        sys.exit(1)
    try:
        result = subprocess.run([binary, *sys.argv[1:]])
        sys.exit(result.returncode)
    except KeyboardInterrupt:
        sys.exit(130)


class ChubError(Exception):
    """Raised when the chub binary returns a non-zero exit code."""


class Client:
    """Synchronous chub SDK client.

    Example::

        import chub

        client = chub.Client()
        results = client.search("stripe payments")
        doc = client.get("stripe/package", lang="python")
        client.annotate("stripe/package", "Always use idempotency keys")
        stats = client.stats()
        matches = client.detect()
    """

    def __init__(self, binary: Optional[str] = None) -> None:
        self._binary = binary or _get_binary_path()

    def _run(self, *args: str) -> Any:
        cmd = [self._binary, "--json", *args]
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            raise ChubError(result.stderr.strip() or f"chub exited with code {result.returncode}")
        return json.loads(result.stdout)

    def search(self, query: str) -> List[Dict[str, Any]]:
        """Search docs by keyword. Returns a list of matching entries."""
        data = self._run("search", query)
        return data.get("results", [])

    def list(
        self,
        query: Optional[str] = None,
        *,
        tags: Optional[str] = None,
        lang: Optional[str] = None,
        type: Optional[str] = None,
        limit: Optional[int] = None,
    ) -> List[Dict[str, Any]]:
        """List available docs and skills with optional filters.

        Args:
            query: Optional search query to narrow results.
            tags: Comma-separated tag filter, e.g. ``"stripe,payments"``.
            lang: Language filter, e.g. ``"python"``.
            type: Entry type filter: ``"doc"`` or ``"skill"``.
            limit: Max number of results (default 20).
        """
        args = ["list"]
        if query:
            args.append(query)
        if tags:
            args += ["--tags", tags]
        if lang:
            args += ["--lang", lang]
        if type:
            args += ["--type", type]
        if limit is not None:
            args += ["--limit", str(limit)]
        data = self._run(*args)
        return data.get("results", [])

    def get(
        self,
        id: str,
        *,
        lang: Optional[str] = None,
        version: Optional[str] = None,
        file: Optional[str] = None,
    ) -> str:
        """Fetch a doc by ID. Returns the markdown content as a string.

        Args:
            id: Entry ID, e.g. ``"stripe/payments"``.
            lang: Language variant, e.g. ``"python"``.
            version: Pinned version string, e.g. ``"2024-06"``.
            file: Fetch a specific file path within the entry.
        """
        args = ["get", id]
        if lang:
            args += ["--lang", lang]
        if version:
            args += ["--version", version]
        if file:
            args += ["--file", file]
        data = self._run(*args)
        return data.get("content", "")

    def annotate(
        self,
        id: str,
        note: str,
        *,
        kind: Optional[str] = None,
        team: bool = False,
        author: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Attach a note to a doc entry.

        Args:
            id: Entry ID, e.g. ``"stripe/payments"``.
            note: Annotation text.
            kind: One of ``"note"`` (default), ``"issue"``, ``"fix"``, ``"practice"``.
            team: If ``True``, save as a git-tracked team annotation.
            author: Author name for team annotations.
        """
        args = ["annotate", id, note]
        if kind:
            args += ["--kind", kind]
        if team:
            args.append("--team")
        if author:
            args += ["--author", author]
        return self._run(*args)

    def pins(self) -> List[Dict[str, Any]]:
        """Return all currently pinned docs."""
        data = self._run("pin", "list")
        return data.get("pins", [])

    def stats(self, *, days: int = 30) -> Dict[str, Any]:
        """Return usage analytics.

        Args:
            days: Number of days to include (default 30).
        """
        return self._run("stats", "--days", str(days))

    def detect(self, *, pin: bool = False) -> Dict[str, Any]:
        """Detect project dependencies and match to available docs.

        Args:
            pin: If ``True``, auto-pin all detected docs.
        """
        args = ["detect"]
        if pin:
            args.append("--pin")
        return self._run(*args)

    def scan_secrets(
        self,
        path: str,
        *,
        staged: bool = False,
        config: Optional[str] = None,
        baseline: Optional[str] = None,
        redact: Optional[int] = None,
    ) -> Dict[str, Any]:
        """Scan a path for secrets.

        Scans a directory if ``path`` is a directory, or a git repo's history
        if ``path`` is a git repository root (pass ``staged=True`` for staged
        changes only).

        Returns a dict with a ``"findings"`` key (list of finding dicts).
        Never raises ``ChubError`` for found secrets — only for scan errors.

        Args:
            path: Directory or git repo root to scan.
            staged: Scan staged changes only (git repos).
            config: Path to ``.chub-scan.toml`` / ``.gitleaks.toml``.
            baseline: Path to a baseline report to suppress known findings.
            redact: Redact secrets at the given percentage (0–100).
        """
        is_git = os.path.isdir(os.path.join(path, ".git"))
        if is_git:
            args = ["scan", "secrets", "git", path]
            if staged:
                args.append("--staged")
        else:
            args = ["scan", "secrets", "dir", path]
        if config:
            args += ["--config", config]
        if baseline:
            args += ["--baseline-path", baseline]
        if redact is not None:
            args += ["--redact", str(redact)]
        # --exit-code 0 ensures non-zero only on real errors, not "found secrets"
        args += ["--exit-code", "0"]
        raw = self._run(*args)
        # The scanner outputs a JSON array of findings; normalise to a dict.
        if not isinstance(raw, dict):
            return {"findings": raw}
        return raw


class AsyncClient:
    """Async chub SDK client.

    Example::

        import chub

        async def main():
            client = chub.AsyncClient()
            results = await client.search("stripe payments")
            doc = await client.get("stripe/package", lang="python")
            stats = await client.stats()
    """

    def __init__(self, binary: Optional[str] = None) -> None:
        self._binary = binary or _get_binary_path()

    async def _run(self, *args: str) -> Any:
        cmd = [self._binary, "--json", *args]
        proc = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        stdout, stderr = await proc.communicate()
        if proc.returncode != 0:
            raise ChubError(stderr.decode().strip() or f"chub exited with code {proc.returncode}")
        return json.loads(stdout.decode())

    async def search(self, query: str) -> List[Dict[str, Any]]:
        """Search docs by keyword. Returns a list of matching entries."""
        data = await self._run("search", query)
        return data.get("results", [])

    async def list(
        self,
        query: Optional[str] = None,
        *,
        tags: Optional[str] = None,
        lang: Optional[str] = None,
        type: Optional[str] = None,
        limit: Optional[int] = None,
    ) -> List[Dict[str, Any]]:
        """List available docs and skills with optional filters."""
        args = ["list"]
        if query:
            args.append(query)
        if tags:
            args += ["--tags", tags]
        if lang:
            args += ["--lang", lang]
        if type:
            args += ["--type", type]
        if limit is not None:
            args += ["--limit", str(limit)]
        data = await self._run(*args)
        return data.get("results", [])

    async def get(
        self,
        id: str,
        *,
        lang: Optional[str] = None,
        version: Optional[str] = None,
        file: Optional[str] = None,
    ) -> str:
        """Fetch a doc by ID. Returns the markdown content as a string."""
        args = ["get", id]
        if lang:
            args += ["--lang", lang]
        if version:
            args += ["--version", version]
        if file:
            args += ["--file", file]
        data = await self._run(*args)
        return data.get("content", "")

    async def annotate(
        self,
        id: str,
        note: str,
        *,
        kind: Optional[str] = None,
        team: bool = False,
        author: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Attach a note to a doc entry."""
        args = ["annotate", id, note]
        if kind:
            args += ["--kind", kind]
        if team:
            args.append("--team")
        if author:
            args += ["--author", author]
        return await self._run(*args)

    async def pins(self) -> List[Dict[str, Any]]:
        """Return all currently pinned docs."""
        data = await self._run("pin", "list")
        return data.get("pins", [])

    async def stats(self, *, days: int = 30) -> Dict[str, Any]:
        """Return usage analytics."""
        return await self._run("stats", "--days", str(days))

    async def detect(self, *, pin: bool = False) -> Dict[str, Any]:
        """Detect project dependencies and match to available docs."""
        args = ["detect"]
        if pin:
            args.append("--pin")
        return await self._run(*args)

    async def scan_secrets(
        self,
        path: str,
        *,
        staged: bool = False,
        config: Optional[str] = None,
        baseline: Optional[str] = None,
        redact: Optional[int] = None,
    ) -> Dict[str, Any]:
        """Scan a path for secrets. Returns ``{"findings": [...]}``."""
        is_git = os.path.isdir(os.path.join(path, ".git"))
        if is_git:
            args = ["scan", "secrets", "git", path]
            if staged:
                args.append("--staged")
        else:
            args = ["scan", "secrets", "dir", path]
        if config:
            args += ["--config", config]
        if baseline:
            args += ["--baseline-path", baseline]
        if redact is not None:
            args += ["--redact", str(redact)]
        args += ["--exit-code", "0"]
        raw = await self._run(*args)
        if not isinstance(raw, dict):
            return {"findings": raw}
        return raw


# ---------------------------------------------------------------------------
# Module-level convenience API — mirrors Client methods with a shared instance
# ---------------------------------------------------------------------------

_default_client: Optional[Client] = None


def _client() -> Client:
    global _default_client
    if _default_client is None:
        _default_client = Client()
    return _default_client


def search(query: str) -> List[Dict[str, Any]]:
    """Search docs by keyword."""
    return _client().search(query)


def list(  # noqa: A001
    query: Optional[str] = None,
    *,
    tags: Optional[str] = None,
    lang: Optional[str] = None,
    type: Optional[str] = None,
    limit: Optional[int] = None,
) -> List[Dict[str, Any]]:
    """List available docs and skills."""
    return _client().list(query, tags=tags, lang=lang, type=type, limit=limit)


def get(
    id: str,
    *,
    lang: Optional[str] = None,
    version: Optional[str] = None,
    file: Optional[str] = None,
) -> str:
    """Fetch a doc by ID. Returns the markdown content."""
    return _client().get(id, lang=lang, version=version, file=file)


def annotate(
    id: str,
    note: str,
    *,
    kind: Optional[str] = None,
    team: bool = False,
    author: Optional[str] = None,
) -> Dict[str, Any]:
    """Attach a note to a doc entry."""
    return _client().annotate(id, note, kind=kind, team=team, author=author)


def pins() -> List[Dict[str, Any]]:
    """Return all currently pinned docs."""
    return _client().pins()


def stats(*, days: int = 30) -> Dict[str, Any]:
    """Return usage analytics."""
    return _client().stats(days=days)


def detect(*, pin: bool = False) -> Dict[str, Any]:
    """Detect project dependencies and match to available docs."""
    return _client().detect(pin=pin)


def scan_secrets(
    path: str,
    *,
    staged: bool = False,
    config: Optional[str] = None,
    baseline: Optional[str] = None,
    redact: Optional[int] = None,
) -> Dict[str, Any]:
    """Scan a path for secrets."""
    return _client().scan_secrets(path, staged=staged, config=config, baseline=baseline, redact=redact)
