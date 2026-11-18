"""Gera um DXF de teste: vale em V com um córrego correndo para leste (curvas de nível a cada 1 m, camada CURVAS).
z = 100 + 0,15·|y - 500| + 0,01·x. Serve para testar a barragem atravessando o vale (eixo em x = 500)."""
import sys
L = ["0", "SECTION", "2", "ENTITIES"]
for z in range(96, 135):
    for lado in (-1, 1):
        pts = []
        for x in range(0, 1001, 20):
            d = (z - 100 - 0.01 * x) / 0.15
            if d >= 0:
                pts.append((x, 500 + lado * d))
        if len(pts) >= 2:
            L += ["0", "LWPOLYLINE", "8", "CURVAS", "90", str(len(pts)), "70", "0", "38", f"{z:.3f}"]
            for x, y in pts:
                L += ["10", f"{x:.3f}", "20", f"{y:.3f}"]
L += ["0", "ENDSEC", "0", "EOF"]
open(sys.argv[1] if len(sys.argv) > 1 else "vale.dxf", "w").write("\n".join(L) + "\n")
