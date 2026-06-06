# Hermes

OPC-UA bridge and MCP server for AI agent access to industrial plant-floor data.

Hermes has two components:

- **Rust CLI** — fast OPC-UA bridge with `poll`, `serve`, `log`, and `browse` commands
- **Python MCP server** — exposes OPC-UA nodes as tools so AI agents can read and write plant-floor data directly

---

## Rust CLI

### Install

```bash
cargo build --release
# binary at target/release/hermes
```

### Commands

**Poll** — live terminal display of node values

```bash
hermes poll --endpoint opc.tcp://localhost:4840 "ns=2;s=Temperature" "ns=2;s=Pressure"
```

**Serve** — REST API backed by live OPC-UA polling

```bash
hermes serve --endpoint opc.tcp://localhost:4840 --port 4000 "ns=2;s=Temperature"
# GET http://localhost:4000/nodes
# GET http://localhost:4000/nodes/ns=2;s=Temperature
```

**Log** — write node values to CSV at a fixed interval

```bash
hermes log --endpoint opc.tcp://localhost:4840 --interval 5 --output data.csv "ns=2;s=Temperature"
```

**Browse** — explore the OPC-UA namespace

```bash
hermes browse --endpoint opc.tcp://localhost:4840 --depth 3
hermes browse --endpoint opc.tcp://localhost:4840 --node "ns=2;s=MyFolder" --depth 2
```

---

## MCP Server

Exposes OPC-UA data as tools for AI agents.

### Install

```bash
uv sync
cp config.example.toml config.toml
```

### Configure

Set your endpoint via environment variable or `.env` file:

```env
HERMES_ENDPOINT=opc.tcp://localhost:4840
HERMES_USERNAME=admin
HERMES_PASSWORD=password
```

### Start

```bash
python -m mcp.server
```

### Tools

| Tool | Description |
|------|-------------|
| `read_node(node_id)` | Read current value of a single node |
| `read_nodes(node_ids)` | Read multiple nodes in one call |
| `write_node(node_id, value)` | Write a value to a node |
| `browse_nodes(parent_node_id, depth)` | Explore the OPC-UA namespace |
| `get_server_status()` | Server build info and state |
| `get_node_attributes(node_id)` | Display name, data type, access level |

### Node ID format

```
ns=2;s=MyStringTag      # namespace 2, string identifier
ns=2;i=1234             # namespace 2, integer identifier
i=2258                  # namespace 0 (standard OPC-UA nodes)
```

---

## REST API

When running `hermes serve`, the following endpoints are available:

| Endpoint | Description |
|----------|-------------|
| `GET /nodes` | All node values with last-updated timestamp |
| `GET /nodes/:node_id` | Single node value |
| `GET /health` | Server health check |

---

## Configuration

Copy `config.example.toml` to `config.toml` and adjust as needed.

```toml
[server]
endpoint = "opc.tcp://localhost:4840"
security_policy = "None"
security_mode = "None"

[auth]
method = "anonymous"  # anonymous | username | certificate

[polling]
interval_secs = 1.0
```

---

## Related Projects

- [ModBridge](https://github.com/AddisonTech/ModBridge) — Modbus TCP bridge (Rust)
- [ModBridge_py](https://github.com/AddisonTech/ModBridge_py) — Modbus TCP bridge (Python)
- [Argus](https://github.com/AddisonTech/Argus) — Modbus anomaly detection with LLM diagnosis
- [Smith_Agentic_MCP](https://github.com/AddisonTech/Smith_Agentic_MCP) — Multi-agent MCP server
