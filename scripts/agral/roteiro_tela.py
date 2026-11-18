"""Roteiro de teste do Agral CAD pela tela: abre um morro de teste, monta a superfície pelos comandos de linha de
comando, confere a cota no mouse, a seleção (Properties TIN Surface) e o duplo clique, tirando fotos.

Uso (Linux, sem placa de vídeo):
  Xvfb :77 -screen 0 1600x1000x24 &
  DISPLAY=:77 VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json target/debug/OpenCADStudio &
  python3 -I scripts/agral/gerar_morro.py /tmp/morro.dxf
  AGRAL_EXE=target/debug/OpenCADStudio MORRO=/tmp/morro.dxf python3 -I scripts/agral/roteiro_tela.py 2
Passo 0: só superfície e fotos pelo MCP. Passo 2: com mouse (xdotool) e fotos da janela.
"""
import os
MORRO = os.environ.get("MORRO", "/tmp/morro.dxf")
import sys, json, math, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mcp_cliente import Mcp, texto, foto
EXE = os.environ.get("AGRAL_EXE", "target/debug/OpenCADStudio")
m = Mcp(EXE)
sid = json.loads(texto(m.tool("ocs_sessions", {"launch_if_none": False})))[0]["session_id"]
N = [0]
def ex(req):
    N[0] += 1
    req = dict(req, request_id=f"t{time.time_ns()}-{N[0]}")
    r = texto(m.tool("ocs_execute", {"ocs_session_id": sid, "request": req}))
    print("EX", json.dumps(req)[:90], "->", r[:300].replace("\n", " "))
    return r
def run(cmd):
    return ex({"op": "run", "cmd": cmd})
def cmd(nome, *resp):
    run(nome)
    for r in resp:
        ex({"op": "input", "kind": "text", "text": r}) if r else ex({"op": "input", "kind": "enter"})
passo = int(sys.argv[1]) if len(sys.argv) > 1 else 0
if passo == 0:
    ex({"op": "open", "path": MORRO})
    time.sleep(3)
    run("ZOOM E")
    foto(m, sid, "01_curvas.png")
    cmd("-CREATESURFACE", "TN")
    cmd("-ADDSURFACECONTOURS", "TN", "CURVAS", "FIX=1", "JOIN=0")
    time.sleep(0.5)
    foto(m, sid, "02_montando.png")
    time.sleep(10)
    foto(m, sid, "03_superficie.png")
    print(texto(m.tool("ocs_read", {"ocs_session_id": sid, "op": "history"}))[-1500:])
elif passo == 2:
    import subprocess as sp, os
    D = {**os.environ, "DISPLAY": ":77"}
    def xd(*a):
        return sp.run(["xdotool", *a], env=D, capture_output=True, text=True).stdout.strip()
    def tela(n):
        sp.run(f"xwd -root -silent | convert xwd:- -crop 1600x1000+0+0 -resize 1300x {n}", shell=True, env=D)
    w = xd("search", "--name", "Agral CAD").split()[0]
    xd("windowsize", w, "1600", "1000"); xd("windowmove", w, "0", "0")
    time.sleep(1)
    # Aviso de placa de vídeo desta máquina de teste (sem GPU): "Don't show again".
    st = json.loads(texto(m.tool("ocs_read", {"ocs_session_id": sid, "op": "state"})))
    if st.get("modal"):
        xd("mousemove", "740", "565"); xd("click", "1"); time.sleep(1)
    ex({"op": "open", "path": MORRO})
    time.sleep(2)
    cmd("-CREATESURFACE", "TN")
    cmd("-ADDSURFACECONTOURS", "TN", "CURVAS", "FIX=1", "JOIN=0")
    time.sleep(6)
    xd("mousemove", "740", "564"); xd("click", "1"); time.sleep(1)
    for _ in range(3):
        if '"ok":true' in run("ZOOM E"):
            break
        time.sleep(1)
    time.sleep(1)
    tela("10_janela.png")
    xd("mousemove", "700", "500"); time.sleep(0.3); xd("mousemove", "705", "505")
    time.sleep(0.8); tela("11_mouse_antes.png")
    time.sleep(2.5); tela("12_mouse_parado.png")
    xd("click", "1"); time.sleep(1.5); tela("13_selecionada.png")
    st = json.loads(texto(m.tool("ocs_read", {"ocs_session_id": sid, "op": "state"})))
    print("selecao", st.get("selection"), "modal", st.get("modal"))
    xd("key", "Escape"); time.sleep(0.5)
    xd("mousemove", "705", "505"); xd("click", "--repeat", "2", "--delay", "80", "1"); time.sleep(1.5); tela("14_duplo_clique.png")
    st = json.loads(texto(m.tool("ocs_read", {"ocs_session_id": sid, "op": "state"})))
    print("modal depois do duplo clique", st.get("modal"))
