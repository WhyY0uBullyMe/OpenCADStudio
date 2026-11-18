# Agral CAD 001: arquitetura (18/11/2026)

Decisão do JP em 18/11: o Agral CAD é um **programa próprio**, com o civil **dentro do núcleo**, não como pacote.
Isso troca a proposta de 17/11 (pacote de fábrica `agral.civil`, removido).

## Onde fica cada parte (bifurcação `WhyY0uBullyMe/OpenCADStudio`, branch `agral-cad`)

| Parte | Caminho | O quê |
|---|---|---|
| Motor | `crates/agral_civil_motor` | TIN, curvas, diagnóstico, correção, emenda, LandXML; sem tela; 44 testes |
| Dados no DWG | `src/civil/definicao.rs` | XRECORD `AGRAL_CIVIL/SURFACES`, versão 3 (lê 1 e 2) |
| Estilos | `src/civil/estilos.rs` | os do JP: Curvas 0.1m, 0.5m, 1m (padrão), 2m, 5m, Apagado, Borda; mais Triângulos |
| Serviço | `src/app/civil.rs` | criar, acrescentar curvas, refazer, apagar, propriedades, ligar/desligar item |
| Comandos | `src/app/commands/surface.rs` | `CREATESURFACE`, `ADDSURFACECONTOURS`, `REBUILDSURFACE`, `SURFACEPROPERTIES`, `DELETESURFACE`, `ZOOMSURFACE`, `TOOLSPACE`, `SURFACEDEFINITION` |
| Janelas | `src/ui/window/civil_dialog.rs` | Create Surface, Add Contour Data, Surface Properties, Delete |
| Toolspace | `src/ui/window/toolspace.rs` | árvore Surfaces > superfície > Definition > tipos de dado, menu no botão direito |
| Aba | `src/modules/civil/` | Create Ground Data, Surface, Palettes |
| Marca | `assets/logo.svg`, título, tela inicial, Sobre, `build.rs` | logo AGRAL, "Agral CAD", `AgralCAD.exe`; sem doação, Reddit, patrocinadores, discussões e apoiadores do original; crédito ao Open CAD Studio no Sobre (GPL 3) |

## Fluxo (como no Civil 3D)

1. Aba Civil > Create Surface (ou `CREATESURFACE`, ou botão direito em Surfaces): janela com tipo TIN, nome,
   descrição e estilo.
2. No Toolspace, botão direito em Definition > Contours > Add... (ou `ADDSURFACECONTOURS`): janela Add Contour Data
   (superfície, descrição, corrigir cruzamentos, emendar até 5 m); OK; "Select contours:".
3. A superfície é montada e desenhada pelo estilo; cruzamentos e emendas ficam marcados em camadas próprias para
   conferência. As polilinhas do desenho não mudam.
4. Botão direito na superfície: Surface Properties, Rebuild, Zoom to, Delete; no item de curvas: Turn Off/On,
   Remove from Definition.

## Medições (container de 4 núcleos, compilação de desenvolvimento otimizada)

| Arquivo | Estilo | Tempo do comando | Observação |
|---|---|---|---|
| Cedro área 2 | Standard | 0,6 s | 186 mil triângulos |
| `CURVAS_V1` | Standard | 38,5 s | 31 s são a correção de 1.208 cruzamentos e as emendas; desenho 4,5 s |
| `CURVAS_V1` | Triangles, sem correção | 21,7 s | desenhar a malha ainda leva 17,8 s |

## Pendências

- Correção mais rápida no `CURVAS_V1` (rodadas repetem o diagnóstico das 400 curvas longas).
- Malha de triângulos lenta para desenhar (cálculo de arestas de vinco do programa).
- Enter nas janelas deve confirmar sem precisar clicar no campo.
- Limites (Boundaries) na árvore: necessários para emendar sem juntar pontas da borda.
- O `ALL` na seleção pega também a polilinha do limite; a camada certa resolve, mas vale um filtro por camada.

## Ligação futura com o Agral Tools (aviso da sessão mãe, 18/11)

O JP definiu que o Agral CAD também é o motor de CAD do Agral Tools: duas telas, o Tools manda o modelo (por
exemplo, a barragem) e o CAD gera e refaz maciço, seções, volumes e pranchas. Fonte: seção 8 de
`docs/specs/agral-tools-interface/estrutura-final.html` e `docs/ARQUITETURA_AGRAL_CAD_AI.md` (branch da sessão mãe).
Não é desta entrega, mas o desenho já atende o começo:

1. **Sem janela:** os comandos de superfície rodam em lote (`--batch`, `--each`, `--serve`) com argumentos.
2. **Objetos com id:** tudo o que a superfície desenha leva XDATA `AGRAL_CIVIL` (`SURFACE:`, `CONTOUR:`, `BORDER:`,
   `CROSSING:`, `JOIN:`) e a definição fica no XRECORD; dá para regenerar a partir do modelo.
3. **Edição à mão como proposta:** ainda não existe; entra quando houver a ligação.
