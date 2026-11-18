# Agral CAD: o que o AutoCAD e o Civil 3D têm e nós ainda não (18/11/2026)

Pedido do JP: "analise o CAD e veja o que tem que nós não temos, útil e indispensável". Feito pela documentação
pública e pelo comportamento conhecido dos programas, sem abrir nem descompilar nada. Comparado com
`docs/commands-reference.md` do Open CAD Studio (700 comandos, 247 atalhos) e com o civil da Agral.

## Falta e é indispensável

| # | Recurso | No Civil 3D | Para quê | Tamanho |
|---|---|---|---|---|
| 1 | Pontos COGO, grupos, importar PNEZD/CSV | CREATEPOINTS, IMPORTPOINTS | Levantamento RTK; o leitor já existe no motor (`pontos.rs`) | M |
| 2 | Superfície de pontos e linhas de quebra | Add Point Files, Breaklines | Crista, pé de talude, eixo de canal; o motor aceita, falta comando | P |
| 3 | Alinhamento com estaqueamento | CREATEALIGNMENTLAYOUT | Eixo da barragem, vertedouro, adutora, canal | G |
| 4 | Perfil e vista de perfil, greide | CREATEPROFILEFROMSURFACE | Perfil da adutora, do canal e do eixo do maciço | G |
| 5 | Seções e vistas de seção | SAMPLELINES, CREATESECTIONVIEW | Seções do maciço por estaca, projeto executivo | G |
| 6 | Volume entre superfícies | Volumes Dashboard | Corte e aterro do maciço, empréstimo | M |
| 7 | Cota, área e volume do reservatório | Stage Storage | Volume útil e morto, ligado ao Hydrus | M |
| 8 | Feature lines e Elevation Editor | CREATEFEATURELINES | Crista, bermas, plataformas | G |
| 9 | Grading | Grading Creation Tools | Taludes até o terreno, plataforma de pivô | G |
| 10 | Corredor, assembly, subassemblies | CREATECORRIDOR | Maciço, canais, estradas | G |
| 11 | Análise de declividade e elevação com legenda | Surface Analysis | Aptidão à irrigação, hipsometria | M |
| 12 | Rótulos de curva de nível | ADDCONTOURLABELS | Toda prancha topográfica | M |
| 13 | Rótulos de ponto e cota na superfície | Spot Elevation Labels | A cota hoje só aparece no mouse | M |
| 14 | Superfície LandXML (ler e gravar) | LANDXMLIN/OUT | Troca com Civil 3D e Hydrus; o motor já tem | P |
| 15 | Paste Surface | PASTESURFACE | Terreno mais maciço ("Junção") | M |

## Falta e é útil

Mass Haul (Brückner); Description Keys (camada e bloco pelo código de campo); fundo de satélite XYZ; mudança de
datum e reprojeção (SAD69 para SIRGAS 2000); importar SHP e KML/KMZ (CAR, APP, ANA); bacias e linhas de escoamento;
redes de tubos por pressão (adutora); lotes (áreas de pivô, APP); Data Shortcuts; Plan Production (pranchas de
planta e perfil); tabelas de coordenadas e de curvas; superfície de DEM, GeoTIFF e nuvem de pontos; mapa de corte e
aterro em cores; BREAKLINE e MPEDIT; rótulo de coordenada N e E.

## Existe em parte

- Propriedades da superfície: faltam área 2D e 3D, declividade média, Prospector completo.
- Edits: faltam Swap Edge, Smooth, Simplify, Minimize Flat Areas.
- Estilos: faltam elevação e declividade em cores e setas de escoamento.
- LANDXMLIMPORT da base: lê só pontos.
- GEOGRAPHICLOCATION: grava o sistema, não reprojeta.
- Nuvem de pontos (LAS/LAZ): falta gerar TIN e classificar solo.
- Sheet Set e PUBLISH: existem; testar pranchas A3/A1 em lote.
- DATALINK (Excel): existe; base para o vínculo nos dois sentidos que o JP pediu.

## A base já tem (o JP talvez não saiba)

IMAGEATTACH com GeoTIFF e world file; PDFIMPORT (PDF vetorial vira linhas); COMPARE; ETRANSMIT; Express Tools
(OVERKILL, TXTEXP, NCOPY, EXTRIM, FLATTEN, LAYWALK, LAYTRANS, TCOUNT, BURST, ATTOUT/ATTIN); DGN e DWF; QUICKCALC;
MVSETUP.

## Primeiros passos sugeridos (motor já pronto, P a M)

1. Importar pontos PNEZD como objeto ponto e na superfície.
2. Comando de linhas de quebra.
3. Ler e gravar superfície LandXML.
