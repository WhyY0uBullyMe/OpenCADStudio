"""Cliente MCP mínimo para pilotar o Agral CAD aberto (roteiro de teste da tela)."""
import json, subprocess, sys, base64, time

class Mcp:
    def __init__(self, exe):
        self.p = subprocess.Popen([exe, "--mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
        self.n = 0
        self.call_raw("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "teste", "version": "1"}})
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")

    def call_raw(self, method, params):
        self.n += 1
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "id": self.n, "method": method, "params": params}) + "\n")
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise RuntimeError("MCP fechou")
            m = json.loads(line)
            if m.get("id") == self.n:
                return m

    def tool(self, name, args):
        r = self.call_raw("tools/call", {"name": name, "arguments": args})
        return r.get("result", r)

def texto(r):
    return " ".join(c.get("text", "") for c in r.get("content", []) if c.get("type") == "text")

def foto(m, sid, caminho):
    r = m.tool("ocs_capture", {"ocs_session_id": sid})
    for c in r.get("content", []):
        if c.get("type") == "image":
            open(caminho, "wb").write(base64.b64decode(c["data"]))
            return True
    print("sem imagem:", texto(r)[:300])
    return False
