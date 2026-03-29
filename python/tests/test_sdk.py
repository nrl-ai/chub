"""Tests for the chub Python SDK.

Runs against the dev binary at ../../target/debug/chub[.exe].
Set CHUB_TEST_BINARY to override the binary path.
"""

import asyncio
import os
import platform
import sys
import tempfile

import pytest

# Make sure we import from the local source tree, not any installed copy.
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
import chub

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
_EXE = "chub.exe" if platform.system() == "Windows" else "chub"
_DEV_BINARY = os.environ.get(
    "CHUB_TEST_BINARY",
    os.path.join(_REPO_ROOT, "target", "debug", _EXE),
)


def _client() -> chub.Client:
    return chub.Client(binary=_DEV_BINARY)


def _async_client() -> chub.AsyncClient:
    return chub.AsyncClient(binary=_DEV_BINARY)


def run(coro):
    return asyncio.get_event_loop().run_until_complete(coro)


# ---------------------------------------------------------------------------
# search
# ---------------------------------------------------------------------------


class TestSearch:
    def test_returns_list(self):
        results = _client().search("stripe")
        assert isinstance(results, list)
        assert len(results) > 0

    def test_result_shape(self):
        results = _client().search("openai")
        for r in results:
            assert "id" in r
            assert "description" in r
            assert "type" in r

    def test_no_results_returns_empty_list(self):
        results = _client().search("xyzzy_nonexistent_query_12345")
        assert results == []

    def test_async(self):
        results = run(_async_client().search("stripe"))
        assert isinstance(results, list)
        assert len(results) > 0


# ---------------------------------------------------------------------------
# list
# ---------------------------------------------------------------------------


class TestList:
    def test_returns_list(self):
        results = _client().list()
        assert isinstance(results, list)
        assert len(results) > 0

    def test_result_shape(self):
        results = _client().list(limit=5)
        assert len(results) <= 5
        for r in results:
            assert "id" in r
            assert "type" in r

    def test_filter_by_lang(self):
        results = _client().list(lang="python", limit=20)
        for r in results:
            assert "python" in r.get("languages", [])

    def test_filter_by_type(self):
        results = _client().list(type="doc", limit=10)
        for r in results:
            assert r["type"] == "doc"

    def test_query_narrows_results(self):
        all_results = _client().list(limit=100)
        filtered = _client().list("stripe", limit=100)
        assert len(filtered) <= len(all_results)
        ids = [r["id"] for r in filtered]
        assert any("stripe" in id for id in ids)

    def test_async(self):
        results = run(_async_client().list(limit=5))
        assert isinstance(results, list)


# ---------------------------------------------------------------------------
# get
# ---------------------------------------------------------------------------


class TestGet:
    def test_returns_markdown_string(self):
        content = _client().get("stripe/package", lang="python")
        assert isinstance(content, str)
        assert len(content) > 100
        assert "stripe" in content.lower()

    def test_lang_param(self):
        content = _client().get("stripe/package", lang="python")
        assert "python" in content.lower()

    def test_missing_id_raises(self):
        with pytest.raises(chub.ChubError):
            _client().get("nonexistent/entry_xyz")

    def test_async(self):
        content = run(_async_client().get("stripe/package", lang="python"))
        assert isinstance(content, str)
        assert len(content) > 100


# ---------------------------------------------------------------------------
# pins
# ---------------------------------------------------------------------------


class TestPins:
    def test_returns_list(self):
        pins = _client().pins()
        assert isinstance(pins, list)

    def test_pin_shape(self):
        pins = _client().pins()
        for p in pins:
            assert "id" in p

    def test_async(self):
        pins = run(_async_client().pins())
        assert isinstance(pins, list)


# ---------------------------------------------------------------------------
# stats
# ---------------------------------------------------------------------------


class TestStats:
    def test_returns_dict(self):
        data = _client().stats()
        assert isinstance(data, dict)

    def test_expected_keys(self):
        data = _client().stats()
        assert "period_days" in data
        assert "total_fetches" in data
        assert "total_searches" in data

    def test_days_param(self):
        data = _client().stats(days=7)
        assert data["period_days"] == 7

    def test_async(self):
        data = run(_async_client().stats(days=7))
        assert data["period_days"] == 7


# ---------------------------------------------------------------------------
# detect
# ---------------------------------------------------------------------------


class TestDetect:
    def test_returns_dict(self):
        data = _client().detect()
        assert isinstance(data, dict)

    def test_expected_keys(self):
        data = _client().detect()
        assert "matches" in data
        assert isinstance(data["matches"], list)

    def test_match_shape(self):
        data = _client().detect()
        for m in data["matches"]:
            assert "dependency" in m
            assert "doc_id" in m
            assert "confidence" in m

    def test_async(self):
        data = run(_async_client().detect())
        assert "matches" in data


# ---------------------------------------------------------------------------
# scan_secrets
# ---------------------------------------------------------------------------


class TestScanSecrets:
    def test_clean_dir_returns_no_findings(self):
        with tempfile.TemporaryDirectory() as tmp:
            with open(os.path.join(tmp, "clean.py"), "w") as f:
                f.write("x = 1\nprint(x)\n")
            data = _client().scan_secrets(tmp)
            assert isinstance(data, dict)
            assert "findings" in data
            assert data["findings"] == []

    def test_dir_with_secret_detected(self):
        with tempfile.TemporaryDirectory() as tmp:
            with open(os.path.join(tmp, "config.py"), "w") as f:
                # Fake AWS key pattern — triggers scanner.
                f.write('AWS_SECRET = "AKIAIOSFODNN7EXAMPLE/wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"\n')
            data = _client().scan_secrets(tmp)
            assert "findings" in data
            assert len(data["findings"]) > 0

    def test_finding_shape(self):
        with tempfile.TemporaryDirectory() as tmp:
            with open(os.path.join(tmp, "config.py"), "w") as f:
                f.write('AWS_SECRET = "AKIAIOSFODNN7EXAMPLE/wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"\n')
            data = _client().scan_secrets(tmp)
            for finding in data["findings"]:
                assert "RuleID" in finding or "rule_id" in finding or "Description" in finding

    def test_git_repo_scan(self):
        # Scan the repo we're in — should return a dict regardless of findings.
        data = _client().scan_secrets(_REPO_ROOT)
        assert isinstance(data, dict)
        assert "findings" in data

    def test_async_dir_scan(self):
        with tempfile.TemporaryDirectory() as tmp:
            with open(os.path.join(tmp, "clean.txt"), "w") as f:
                f.write("nothing secret here\n")
            data = run(_async_client().scan_secrets(tmp))
            assert isinstance(data, dict)
            assert "findings" in data


# ---------------------------------------------------------------------------
# annotate (light — avoids mutating shared state)
# ---------------------------------------------------------------------------


class TestAnnotate:
    def test_returns_dict(self):
        # Personal annotation — safe to call in tests, scoped to local ~/.chub.
        data = _client().annotate("stripe/package", "SDK test annotation")
        assert isinstance(data, dict)

    def test_kind_param(self):
        data = _client().annotate("stripe/package", "test practice", kind="practice")
        assert isinstance(data, dict)

    def test_async(self):
        data = run(_async_client().annotate("stripe/package", "async test annotation"))
        assert isinstance(data, dict)


# ---------------------------------------------------------------------------
# Module-level convenience API
# ---------------------------------------------------------------------------


class TestModuleLevel:
    """The module-level functions should delegate to a shared Client."""

    def setup_method(self):
        # Point the shared client at our dev binary.
        chub._default_client = chub.Client(binary=_DEV_BINARY)

    def test_search(self):
        results = chub.search("stripe")
        assert len(results) > 0

    def test_list(self):
        results = chub.list(limit=5)
        assert len(results) <= 5

    def test_get(self):
        content = chub.get("stripe/package", lang="python")
        assert len(content) > 100

    def test_pins(self):
        assert isinstance(chub.pins(), list)

    def test_stats(self):
        assert "total_fetches" in chub.stats()

    def test_detect(self):
        assert "matches" in chub.detect()

    def test_scan_secrets(self):
        with tempfile.TemporaryDirectory() as tmp:
            open(os.path.join(tmp, "f.txt"), "w").close()
            data = chub.scan_secrets(tmp)
            assert isinstance(data, dict)
            assert "findings" in data


# ---------------------------------------------------------------------------
# Error handling
# ---------------------------------------------------------------------------


class TestErrors:
    def test_chub_error_on_bad_id(self):
        with pytest.raises(chub.ChubError):
            _client().get("totally/invalid/id/xyz")

    def test_async_chub_error(self):
        with pytest.raises(chub.ChubError):
            run(_async_client().get("totally/invalid/id/xyz"))
