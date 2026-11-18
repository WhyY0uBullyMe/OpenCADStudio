# Agral CAD: o que já existe nas bifurcações e plugins do Open CAD Studio (18/11/2026)

Pedido do JP: olhar as bifurcações (505) atrás de trabalho mais elaborado. Pesquisa só de leitura (buscas na API do
GitHub, `git ls-remote` e clones rasos; nada compilado nem executado). As ~505 bifurcações não foram varridas uma a
uma (limite de buscas): procurei por palavras-chave, PRs e plugins.

## Vale aproveitar (todos GPL 3)

| # | Projeto | O quê | Uso para nós |
|---|---|---|---|
| 1 | `BeeBeeSix/OCS_Plugin_RasterTools` | baixa tiles XYZ `{z}/{x}/{y}` num retângulo com EPSG, reprojeta, world file e GeoTIFF; Rust com testes | base da imagem de satélite por trás do desenho (falta ligar ao GEODATA) |
| 2 | `pabloxbezx/teren-ocs-plugin` (Terén, tcheco) | pontos, TIN com breaklines, limites, edição de triângulos, curvas rotuladas, estilos, Prospector; alinhamento, perfil com greide e curvas verticais (conceito) | referência para estilos de perfil e alinhamento e edição de TIN |
| 3 | `KevinGriffin-new/opencad-landsurvey-plugin` | PNEZD, LandXML, RW5/SDR, COGO, volumes entre superfícies, cota no ponto, alinhamento com clotoide e estaqueamento | volumes e alinhamento (código bem separado) |
| 4 | `OpenAEC-Foundation/Open-Infra-Studio` (19 commits novos) | comandos em inglês, GEODATA gravado com a localização, padrões de tiles | só a parte de localização; corredor e grading continuam iguais (holandês) |
| 5 | `saadmooh/OpenCADStudio` (ramo antigo) | lotes (parcels) dinâmicos | só se lotes interessarem; muito defasado para juntar |

## Já está na nossa base

- Leitura de objetos do Civil 3D (corredor, feature line, seção, TIN, grading): só mostra, não edita nem calcula.
- GEODATA e o comando GEOGRAPHICLOCATION.
- Fixar por painel e esconder sozinho por painel (`dock.auto_hides`): a conferir se a tela deixa usar como o JP quer
  (Properties fixo, Toolspace escondido).

## Não existe em lugar nenhum encontrado

Assemblies e subassemblies paramétricos com corredor gerado a partir deles; feature lines editáveis e grading no
estilo do Civil 3D; cota da superfície no mouse; imagem XYZ ligada ao GEODATA como fundo; Toolspace no estilo do
Civil 3D. Isso é trabalho nosso.
