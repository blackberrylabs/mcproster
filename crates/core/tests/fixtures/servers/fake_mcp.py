#!/usr/bin/env python3
"""A minimal MCP stdio server for tests: answers initialize and tools/list, nothing else.
With SLOW=1 in the environment it never answers, to test the probe's time limit."""
import json
import os
import sys
import time

if os.environ.get("SLOW") == "1":
    time.sleep(60)

for line in sys.stdin:
    try:
        msg = json.loads(line)
    except ValueError:
        continue
    method, rid = msg.get("method"), msg.get("id")
    if method == "initialize":
        out = {"jsonrpc": "2.0", "id": rid, "result": {
            "protocolVersion": msg["params"].get("protocolVersion"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "fake-mcp", "version": "9.9.9"}}}
    elif method == "tools/list":
        out = {"jsonrpc": "2.0", "id": rid, "result": {"tools": [
            {"name": "ping", "description": "Answers pong.\nSecond line ignored."},
            {"name": "echo", "description": "Returns its input. " + "x" * 200}]}}
    elif rid is None:
        continue  # notification
    else:
        out = {"jsonrpc": "2.0", "id": rid, "error": {"code": -32601, "message": "unknown method"}}
    sys.stdout.write(json.dumps(out) + "\n")
    sys.stdout.flush()
