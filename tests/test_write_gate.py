"""
Tests for the write-gating safety on the Hermes MCP server.

These exercise `_write_forbidden_reason` directly, so they need neither a live
OPC-UA server nor the MCP transport — the gate refuses before any connection.
"""
from hermes_mcp.server import _write_forbidden_reason, settings


def _reset(allow_writes: bool = False, write_token: str = "") -> None:
    settings.allow_writes = allow_writes
    settings.write_token = write_token


def test_writes_forbidden_by_default():
    _reset()
    assert _write_forbidden_reason("") is not None


def test_writes_allowed_when_enabled_without_token():
    _reset(allow_writes=True)
    assert _write_forbidden_reason("") is None


def test_token_required_when_configured():
    _reset(allow_writes=True, write_token="s3cret")
    assert _write_forbidden_reason("") is not None       # missing
    assert _write_forbidden_reason("wrong") is not None  # mismatch
    assert _write_forbidden_reason("s3cret") is None     # correct


def test_token_ignored_when_writes_disabled():
    _reset(allow_writes=False, write_token="s3cret")
    # Even a correct token cannot write while writes are globally disabled.
    assert _write_forbidden_reason("s3cret") is not None
