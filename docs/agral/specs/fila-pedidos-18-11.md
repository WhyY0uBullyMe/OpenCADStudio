# Agral CAD: pedidos do JP em 18/11/2026, depois do primeiro teste no Windows

"Bicho, ficou bom. Deu certo." O ícone ficou "perfeito". A parte das curvas não se cruzarem "está funcionando
perfeitamente". O Toolspace "até que não ficou ruim", mas precisa melhorar.

## Ordem combinada

| # | Pedido | Situação |
|---|---|---|
| 1 | Superfície como objeto do Civil 3D: selecionar seleciona a superfície, com a aba própria | **Feito** (bloco + aba TIN Surface) |
| 2 | Extract Objects da superfície | **Feito** (`EXTRACTSURFACEOBJECTS`) |
| 3 | Boundaries ("muito importante") | **Feito**: Outer e Hide; falta Show e Data Clip |
| 4 | Edits ("editar triângulos") | **Feito em parte**: Add Point, Delete Point, Delete Line, Raise/Lower; falta Swap Edge, Add Line, Move/Modify Point, Smooth, Paste Surface, Minimize Flat Areas |
| 5 | Editar o estilo da superfície selecionada | Pela Surface Properties (troca de estilo); falta o editor de estilos (spec 002-estilos) |
| 6 | Properties da superfície selecionada como no Civil 3D (hoje mostra "Block Reference") | Feito: "TIN Surface", estilo editável, estatística; duplo clique abre Surface Properties |
| 7 | Painéis: fixar cada um à parte (Properties sempre fixo, Toolspace aparece ao passar o mouse) | Em parte: Toolspace abre na borda oposta, escondendo sozinho; fixar por painel de verdade fica para a spec de interface |
| 8 | Aparência do programa mais bonita (linha do Hydrus); o logo dentro do menu do programa ficou feio | A fazer (spec de interface) |
| 9 | Lugar para o agente de IA dentro do programa | A fazer; o Open CAD Studio já tem um painel Assistant (`docs/assistant-panel.md`) |
| 10 | Prospector completo como o do Civil 3D (foto do JP): Points, Point Groups, Surfaces, Alignments, Feature Lines, Sites, Catchments, Channels, Stormwater Controls, Pipe Networks, Pressure Networks, Bridges, Turnouts and Crossovers, Corridors, Assemblies, Subassemblies, Intersections, Survey, View Frame Groups; Data Shortcuts; abas Prospector, Settings, Survey, Toolbox | A fazer: os ramos aparecem conforme cada objeto existir |
| 11 | Feature lines com o Elevation Editor (tabela para editar cotas de vários objetos) | Próxima spec de objetos lineares |
| 12 | Grading Creation Tools ("uso muito") | Depois das feature lines |
| 13 | Datum do lugar e imagem de satélite por trás, como o Geolocation; link XYZ "do Google, como no QGIS" | A fazer; **atenção à licença**: as imagens do Google por link XYZ ficam fora dos termos de uso do Google; alternativas: chave do Google Maps Tiles API, Esri World Imagery (termos próprios), Sentinel-2 cloudless da EOX (gratuito para uso não comercial). O programa aceita qualquer link XYZ; a escolha do serviço é do JP |

## Retorno do JP no mesmo dia (tarde)

- **Prioridades para usar como padrão:** corredor, assemblies, feature lines, Grading Creation Tools.
- Cota ao passar o mouse sobre a superfície. Feito (18/11).
- Imagem do Google por XYZ: aprovado pelo JP para uso interno.
- Toolspace mais bonito e arrumado.
- Repositório privado: o JP cria um repositório privado novo; o código vai para lá.
- Análise das bifurcações do Open CAD Studio: em andamento.
