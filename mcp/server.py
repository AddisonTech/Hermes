"""
Hermes MCP Server — exposes OPC-UA plant-floor data as tools for AI agents.

Start:
    python -m mcp.server

Environment:
    HERMES_ENDPOINT   OPC-UA server URL (default: opc.tcp://localhost:4840)
    HERMES_USERNAME   Username for auth (optional)
    HERMES_PASSWORD   Password for auth (optional)
"""

import asyncio
import os
from contextlib import asynccontextmanager
from typing import Any

from asyncua import Client, ua
from fastmcp import FastMCP
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    endpoint: str = "opc.tcp://localhost:4840"
    username: str = ""
    password: str = ""

    model_config = {"env_prefix": "HERMES_"}


settings = Settings()
mcp = FastMCP("hermes")

_client: Client | None = None
_lock = asyncio.Lock()


async def get_client() -> Client:
    global _client
    async with _lock:
        if _client is None or not _client.uaclient.protocol:
            _client = Client(url=settings.endpoint)
            if settings.username:
                _client.set_user(settings.username)
                _client.set_password(settings.password)
            await _client.connect()
        return _client


@mcp.tool()
async def read_node(node_id: str) -> dict[str, Any]:
    """Read the current value of a single OPC-UA node.

    Args:
        node_id: OPC-UA NodeId string (e.g. 'ns=2;s=MyTag' or 'i=2258')
    """
    client = await get_client()
    node = client.get_node(node_id)
    try:
        dv = await node.read_data_value()
        return {
            "node_id": node_id,
            "value": _variant_to_python(dv.Value),
            "status": str(dv.StatusCode),
            "timestamp": str(dv.SourceTimestamp),
        }
    except Exception as e:
        return {"node_id": node_id, "error": str(e)}


@mcp.tool()
async def read_nodes(node_ids: list[str]) -> dict[str, Any]:
    """Read current values of multiple OPC-UA nodes in one call.

    Args:
        node_ids: List of OPC-UA NodeId strings
    """
    results = await asyncio.gather(
        *[read_node(nid) for nid in node_ids], return_exceptions=True
    )
    return {
        nid: (r if not isinstance(r, Exception) else {"error": str(r)})
        for nid, r in zip(node_ids, results)
    }


@mcp.tool()
async def write_node(node_id: str, value: float | int | str | bool) -> dict[str, Any]:
    """Write a value to an OPC-UA node.

    Args:
        node_id: OPC-UA NodeId string (e.g. 'ns=2;s=MyTag')
        value:   Value to write (float, int, str, or bool)
    """
    client = await get_client()
    node = client.get_node(node_id)
    try:
        await node.write_value(value)
        return {"node_id": node_id, "written": value, "status": "ok"}
    except Exception as e:
        return {"node_id": node_id, "error": str(e)}


@mcp.tool()
async def browse_nodes(
    parent_node_id: str = "i=85",
    depth: int = 2,
) -> dict[str, Any]:
    """Browse the OPC-UA server namespace to discover available nodes.

    Args:
        parent_node_id: Starting node (default 'i=85' = Objects folder)
        depth:          How many levels deep to browse (1-5, default 2)
    """
    depth = max(1, min(depth, 5))
    client = await get_client()
    node = client.get_node(parent_node_id)
    try:
        return await _browse_recursive(node, depth, 0)
    except Exception as e:
        return {"error": str(e)}


@mcp.tool()
async def get_server_status() -> dict[str, Any]:
    """Get OPC-UA server status, build info, and connection details."""
    client = await get_client()
    try:
        status_node = client.get_node("i=2256")  # ServerStatus
        status = await status_node.read_value()
        return {
            "endpoint": settings.endpoint,
            "state": str(status.State),
            "build_info": {
                "product_name": str(status.BuildInfo.ProductName),
                "manufacturer": str(status.BuildInfo.ManufacturerName),
                "software_version": str(status.BuildInfo.SoftwareVersion),
            },
            "start_time": str(status.StartTime),
            "current_time": str(status.CurrentTime),
        }
    except Exception as e:
        return {"endpoint": settings.endpoint, "error": str(e)}


@mcp.tool()
async def get_node_attributes(node_id: str) -> dict[str, Any]:
    """Get full metadata for a node: display name, description, data type, access level.

    Args:
        node_id: OPC-UA NodeId string
    """
    client = await get_client()
    node = client.get_node(node_id)
    try:
        attrs = await node.read_attributes([
            ua.AttributeIds.DisplayName,
            ua.AttributeIds.Description,
            ua.AttributeIds.DataType,
            ua.AttributeIds.AccessLevel,
            ua.AttributeIds.NodeClass,
        ])
        return {
            "node_id": node_id,
            "display_name": str(attrs[0].Value.Value),
            "description": str(attrs[1].Value.Value),
            "data_type": str(attrs[2].Value.Value),
            "access_level": str(attrs[3].Value.Value),
            "node_class": str(attrs[4].Value.Value),
        }
    except Exception as e:
        return {"node_id": node_id, "error": str(e)}


async def _browse_recursive(node, max_depth: int, current_depth: int) -> dict:
    result = {
        "node_id": str(node.nodeid),
    }
    try:
        result["display_name"] = str(await node.read_display_name())
    except Exception:
        pass

    if current_depth < max_depth:
        try:
            children = await node.get_children()
            result["children"] = [
                await _browse_recursive(child, max_depth, current_depth + 1)
                for child in children
            ]
        except Exception as e:
            result["browse_error"] = str(e)

    return result


def _variant_to_python(variant) -> Any:
    if variant is None:
        return None
    v = variant.Value if hasattr(variant, "Value") else variant
    if isinstance(v, (bool, int, float, str)):
        return v
    return str(v)


if __name__ == "__main__":
    mcp.run()
