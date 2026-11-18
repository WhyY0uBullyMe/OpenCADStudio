"""Gera um DXF de teste: morro com 40 curvas de nível fechadas (LWPOLYLINE com cota) na camada CURVAS."""
import math, sys
L = ["0", "SECTION", "2", "ENTITIES"]
for k in range(1, 41):
    r = 6.0 * k
    z = 150.0 - r / 6.0
    L += ["0", "LWPOLYLINE", "8", "CURVAS", "90", "90", "70", "1", "38", f"{z:.3f}"]
    for t in range(90):
        a = t / 90 * 2 * math.pi
        rr = r * (1 + 0.18 * math.sin(3 * a) + 0.05 * math.cos(5 * a))
        L += ["10", f"{500 + rr * math.cos(a):.3f}", "20", f"{500 + rr * math.sin(a) * 0.8:.3f}"]
L += ["0", "ENDSEC", "0", "EOF"]
open(sys.argv[1] if len(sys.argv) > 1 else "morro.dxf", "w").write("\n".join(L) + "\n")
