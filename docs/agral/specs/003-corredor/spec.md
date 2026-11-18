# Agral CAD 003: pontos, linhas de quebra, LandXML, alinhamento, perfil, assembly e corredor

Situação: spec de 18/11/2026. Autorizado pelo JP ("faz o corredor... faz os dois, faz tudo"). Referência: o
comportamento e a ajuda pública do Civil 3D, sem abrir nem copiar o programa. Fluxo da barragem em
`../002-fluxo-barragem/brief.md`; o que falta no geral em `../analise-lacunas.md`.

## Ordem de entrega

1. Dados da superfície que o motor já tem (P): pontos de arquivo, linhas de quebra, LandXML.
2. Alinhamento (G), perfil (G), assembly da barragem (M), corredor e superfície do corredor (G).

Cada parte sai testada e com commit; a seguinte começa quando a anterior passa.

## Requisitos

### Parte 1: dados da superfície

- **R1. Linhas de quebra** (`ADDSURFACEBREAKLINES`): polilinhas 2D/3D e linhas do desenho entram na definição como
  item "Breaklines" (referência às entidades, como as curvas). Ao refazer, viram arestas obrigatórias do TIN. Aparecem
  no Toolspace em Breaklines.
- **R2. Arquivo de pontos** (`ADDSURFACEPOINTFILE`): PNEZD, PENZD, NEZ, ENZ (o leitor do motor). O item guarda o
  caminho do arquivo e o formato; ao refazer, lê de novo. Linhas que não leem aparecem no aviso, com número. Arquivo
  sumido avisa e não apaga a superfície.
- **R3. LandXML**: `IMPORTLANDXMLSURFACE` cria uma superfície por `<Surface>` do arquivo, com os mesmos triângulos (não
  triangula de novo). `EXPORTLANDXMLSURFACE` grava a superfície montada em LandXML 1.2, que o Civil 3D abre.

### Parte 2: corredor

- **R4. Alinhamento** (`CREATEALIGNMENT`): a partir de uma polilinha (retas e arcos), com nome, estaca inicial e
  rótulo de estaca a cada 20 m (padrão brasileiro) ou intervalo escolhido. Guardado no DWG (XRECORD em
  `AGRAL_CIVIL/ALIGNMENTS`) e desenhado como objeto único (bloco + inserção, como a superfície).
- **R5. Perfil do terreno** (`CREATEPROFILEFROMSURFACE`): amostra a superfície ao longo do alinhamento. **Vista de
  perfil** (`CREATEPROFILEVIEW`): grade com estacas e cotas, exagero vertical escolhido (padrão 10x).
- **R6. Greide** (`CREATEPROFILE`): pontos de inflexão (estaca, cota) digitados numa tabela, com curva vertical
  opcional. Para barragem, o greide é a cota da crista (horizontal) ao longo do eixo.
- **R7. Assembly da barragem** (`CREATEASSEMBLY`): crista (largura, declividade transversal com padrão 1% para
  montante), talude de montante e de jusante (H:V), bermas pela tabela de patamares (largura e altura entre
  bermas), até encontrar a superfície do terreno (saia). Parâmetros numa tabela editável.
- **R8. Corredor** (`CREATECORRIDOR`): alinhamento + greide + assembly + superfície alvo, com espaçamento de seções
  (padrão 5 m e nos pontos de mudança). Gera as linhas do corredor (crista, bordas, bermas, saia) como feature lines
  3D e a **superfície do corredor** com limite automático pela linha de saia (sem "dentes").
- **R9. Refazer em cadeia**: mudou o alinhamento, o greide, o assembly ou a superfície do terreno, o corredor fica
  marcado como desatualizado e se refaz pelo `REBUILDCORRIDOR` (e a superfície do corredor junto).
- **R10. Tudo em segundo plano** com a barra de progresso, sem travar a tela.

## Fora do escopo (agora)

Subassemblies genéricos de estrada; superelevação; interseções; seções transversais em prancha (vêm logo depois);
volume e Paste Surface (spec 002); Plan Production.

## Critérios de aceite

1. Linha de quebra muda o TIN: uma linha 3D atravessando um terreno plano aparece como aresta e a cota no mouse
   sobre ela é a da linha (teste automático).
2. Arquivo PNEZD de 1.000 pontos vira superfície; linha ruim aparece no aviso com o número (teste).
3. LandXML exportado e lido de volta dá o mesmo número de pontos e triângulos e a mesma cota em 10 pontos
   sorteados (teste).
4. Alinhamento de uma polilinha com reta e arco: comprimento igual ao da polilinha (1 mm) e estaca certa em 3 pontos
   (teste).
5. Perfil do terreno de um plano inclinado conhecido: cotas certas em todas as estacas (1 mm) (teste).
6. Corredor de barragem num terreno plano: largura da crista, declividade de 1%, taludes e bermas medidos nas seções
   batem com o assembly (1 cm); a saia encontra o terreno; volume do aterro igual ao calculado à mão para um prisma
   trapezoidal (0,5%) (teste).
7. Na tela (roteiro com Xvfb): criar alinhamento, perfil, assembly e corredor pelos comandos sem travar, com foto de
   cada passo.

## Tarefas e evidências

Registrar em `tarefas.md` desta pasta, com o commit de cada parte e o resultado dos testes.
