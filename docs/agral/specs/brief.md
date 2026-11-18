# Agral CAD (civil): brief

Situação: **análise para o JP decidir** (17/11/2026). Não autoriza código.

## Pedido do JP (17/11/2026)

- "Estou precisando correr com a versão do CAD nossa, porque estou parado." O CAD passa a ser prioridade.
- Base sugerida por ele: LibreCAD, transformado numa versão nossa do Civil 3D, mais leve.
- Funções pedidas:
  - geração de alinhamento;
  - geração de corredor;
  - administração de superfícies;
  - geração de sample lines.
- Tudo bem parecido com o Civil 3D, para ninguém ter de reaprender a ferramenta.

## O que conferi nas fontes (17/11/2026)

| | LibreCAD | CADcraft (o que o JP usa hoje) |
|---|---|---|
| Licença | GPL versão 2: programa derivado distribuído tem de abrir o código | MIT ou Apache 2.0: livre para adaptar, sem obrigação de abrir |
| Linguagem | C++ com Qt 6 | Rust, com as camadas separadas (geometria, desenho, comandos, tela) |
| Escopo | Só 2D; estável (2.2.x) | 2D; o 3D está em 2% do previsto; projeto em "pre-alpha" |
| DWG | Não confirmei | Lê R13 a 2018 e grava R2000; o próprio projeto avisa que o DWG gravado "nunca foi conferido no AutoCAD" e perde o que ele não modela |
| DXF | Sim | Lê R12 a 2018 e grava R2000 |
| IA operando | Não tem | Servidor MCP próprio e canal de controle em JSON |
| Rotinas | Plugins em C++ | Scripts `.scr` e rotinas LISP, em evolução |
| Funções de civil | Nenhuma | Nenhuma |

Outras opções vistas:
- **FreeCAD com a bancada Road:** tem alinhamento, perfil, seção e importa LandXML. O volume e a exportação de
  LandXML aparecem como não feitos. A interface é a do FreeCAD, bem diferente do Civil 3D.
- **ZCAD:** código aberto (GPL 3 ou MPL 2), em Pascal, considerado imaturo.

Daqui dá para compilar o CADcraft: há Rust 1.97 (ele pede 1.95) e o repositório é público. **Não compilei nem testei.**

## O ponto principal

Nenhuma das bases tem civil. Superfície, alinhamento, perfil, corredor, sample lines, seções e volume têm de ser
feitos por nós, qualquer que seja a base. A escolha da base decide só onde o desenho aparece e como ele é editado.

O Hydrus já tem parte desse motor: superfície por LandXML e por pontos (TIN), costura com o MDT público, curvas de
nível, perfil da adutora e volume do lago pelo terreno.

## Caminhos

| | Como é | A favor | Contra |
|---|---|---|---|
| **A. Motor civil no Tools, desenho no CADcraft (recomendado)** | O motor civil (superfície, alinhamento, perfil, corredor, seções, volume) fica no Tools. A tela de civil imita o Civil 3D: árvore com Superfícies, Alinhamentos, Corredores e Grupos de seções, e os mesmos nomes de comando. O resultado vai para o CADcraft por DXF e pelo canal MCP, para acabamento e prancha | Destrava mais rápido; reaproveita o que o Hydrus já calcula; o corredor da barragem é o "gerador da barragem" que já estava no plano; não depende da maturidade do CADcraft | Alinhamento e corredor não são objetos vivos dentro do CAD: mudou no Tools, redesenha |
| B. Bifurcar o CADcraft e pôr o civil dentro | Objetos de civil nativos no desenho, como no Civil 3D | A experiência mais parecida com o Civil 3D | Base em "pre-alpha"; DWG gravado sem conferência; muito mais trabalho antes do primeiro resultado |
| C. Bifurcar o LibreCAD | Como o JP sugeriu | Base estável | Só 2D (superfície e corredor precisam de cota); GPL; C++ com Qt, longe do que já temos |

## Primeiro pedaço proposto (caminho A)

1. **Superfícies:** importar LandXML, pontos, curvas e nuvem já classificada; listar, renomear, ver as curvas; colar
   uma na outra.
2. **Alinhamento:** desenhar pelos PIs, com curvas; estaqueamento; tabela.
3. **Perfil:** terreno ao longo do alinhamento e o greide.
4. **Corredor:** seção-tipo ao longo do alinhamento (a barragem primeiro: crista, taludes, bermas); a superfície do
   corredor.
5. **Sample lines e seções:** a cada tantos metros e em estacas escolhidas; seção do terreno e do projeto.
6. **Volume:** corte e aterro pelas seções e entre superfícies.
7. **Saída:** DXF e LandXML para o CADcraft e o Civil 3D; pranchas de perfil e de seções.

## Perguntas para o JP

1. O que está parado, e por quê? Qual entrega depende do CAD agora (por exemplo, a barragem do Murilo: eixo, perfil,
   seções e volume do maciço), e o que impede de fazer no Civil 3D ou no CADcraft hoje?
2. O corredor que você precisa primeiro é o da barragem, ou também estrada, canal e platô?

## Pesquisa no GitHub (17/11/2026, a pedido do JP: "alguém já deve estar fazendo esse estudo em cima do Civil 3D")

Tudo abaixo vem das páginas dos projetos. **Nada foi instalado nem executado.**

| Projeto | O que é | Licença | Tamanho e atividade | Civil que declara ter |
|---|---|---|---|---|
| **Open CAD Studio** (`HakanSeven12/OpenCADStudio`) | CAD 2D e 3D em Rust, para Windows, Linux, macOS e navegador. Lê e grava DWG e DXF (R14 a 2018), tem layouts, estilos de plotagem, sólidos 3D, plugins e ponte para IA. Instalador MSI assinado para Windows | GPL 3 | 4,1 mil estrelas, 8.684 commits | Vem pelo plugin abaixo |
| **Open Infra Studio** (`OpenAEC-Foundation/Open-Infra-Studio`) | Plugin de infraestrutura para o Open CAD Studio, em Rust, com cerca de 29 mil linhas e 120 testes | GPL 3 | 27 estrelas, 590 commits | Pontos de levantamento, LandXML, TIN, curvas de nível, alinhamento com estaqueamento, perfil do terreno e de projeto, seções transversais, corredor com saia até o terreno, terraplenagem e balanço de massas |
| **CorridorRoad** (`ganadara135/CorridorRoad`) | Bancada do FreeCAD, em Python | LGPL 2.1 | 491 commits, versão 1.1.2 de 15/11/2026 | TIN, alinhamento, estacas, perfil, editor de seção-tipo (valeta, talude, berma), corredor, seções, relatório de terraplenagem, LandXML |
| **Road** (`hakanseven12/Road`) | Bancada do FreeCAD, do mesmo autor do Open CAD Studio | LGPL 2.1 | | Alinhamento, perfil, seção; volume e exportação de LandXML não feitos |
| **Civil-Design** (`sajadsarlak1361/Civil-Design`) | Aplicativo próprio (Tauri) | "Todos os direitos reservados": **não dá para reaproveitar o código** | 29 commits | Lista muita coisa, mas é novo e pequeno |
| **spade** (biblioteca Rust) | Triangulação de Delaunay com arestas forçadas (linhas de quebra) | MIT ou Apache | | Só a triangulação |

**Cuidado:** os repositórios `onyxthrasherribbon/Autodesk-Civil-3D` e `OpenRoads-Designer/.github` não são código
aberto. São páginas de download com o nome de programas comerciais. Não baixar.

### O que isso muda

O Open CAD Studio com o Open Infra Studio já declara ter quase tudo o que o JP pediu, numa base bem mais madura que
o CADcraft (que está em "pre-alpha", sem 3D e sem civil). Antes de bifurcar o CADcraft, vale testar esse par num
trabalho real.

| | Bifurcar o CADcraft | Partir do Open CAD Studio com o Open Infra |
|---|---|---|
| Civil | Tudo por fazer | Já existe, a conferir |
| 3D | 2% do previsto | Sólidos e câmera em perspectiva |
| DWG | Grava R2000, sem conferência | Lê e grava R14 a 2018 |
| Licença | MIT ou Apache: livre | GPL 3: uso interno é livre; se um dia distribuir a nossa versão, o código tem de ir junto |
| Nosso trabalho | Escrever o civil inteiro | Estilos da Agral, o que faltar (sample lines como no Civil 3D, seção-tipo de barragem, pranchas) e correções |

**Recomendação:** o JP instala o Open CAD Studio e o Open Infra Studio e testa com um trabalho real. Se servir, a
nossa versão parte dele, e não do CADcraft. Se não servir, voltamos ao plano da bifurcação do CADcraft, aproveitando
a `spade` para a superfície.

## Teste do JP no Open CAD Studio (17/11/2026): o que apareceu

Lido no código do plugin (versão 2026.47.0, commit 6795a90), sem executar.

- **Limite de 120 mil triângulos por superfície.** É um valor fixo em `src/civiel/terrein.rs` (`MAX_TRIANGLES`).
  O levantamento do JP deu 234.095 triângulos e foi recusado. Para levantamento de fazenda, esse teto é baixo:
  é o primeiro ponto a mudar numa versão nossa.
- **Superfície a partir de curvas: aceita.** O `CIV_TIN` usa o que estiver selecionado, ou uma camada inteira
  (`LAAG=`). Ele lê pontos, polilinhas 3D, polilinhas com elevação, linhas com Z e faces 3D.
- **A curva entra só pelos vértices.** Ela não é tratada como linha de quebra, então um triângulo pode atravessar a
  curva. Linha de quebra de verdade só na camada `OG-BREUKLIJN`, pela fonte `BRON=PUNTEN`.
- **Mensagens de erro em holandês,** mesmo com a tela em inglês.

## Pedidos do JP em 17/11 (fila depois da 001)

- **Objetivo geral:** mais rápido e menos travado que o AutoCAD. Toda entrega mede o tempo contra o AutoCAD no
  computador do JP (abrir o desenho, criar a superfície, navegar).
- **Comandos do dia a dia** já existem no Open CAD Studio com o nome do AutoCAD (`PASTEORIG`, `HIDEOBJECTS`,
  `ISOLATEOBJECTS`, `MATCHPROP`, `LAYISO` e outros). Falta testar cada um contra o comportamento do AutoCAD e montar
  a lista completa com a equipe.
- **Atalhos:** `ALIASEDIT` edita e importa o `.pgp` do AutoCAD. O arquivo de atalhos de cada um fica no computador
  dele, não no repositório (o `.pgp` padrão é da Autodesk).
- **002-estilos:** presets de estilo e aparência (superfície, curvas, rótulos e outros), com os estilos Agral de
  fábrica.
- **Aparência:** paleta do Hydrus (`docs/design.md`); layout decidido depois com o JP.

## Definições do JP em 18/11 (via sessão mãe), para as próximas specs

- **DWG comum:** todo desenho gerado é um DWG normal, salvo na pasta do trabalho, que abre e se edita no Agral CAD e
  no AutoCAD. (A superfície já é assim: curvas e borda são polilinhas comuns; a definição vai num XRECORD que o
  AutoCAD ignora sem estragar o arquivo.)
- **Patamares do maciço (para a spec de corredor):** o "daylight bench" do Civil 3D só aceita uma altura e uma
  largura. O JP quer uma tabela de patamares, cada um com cota ou altura, largura e caimento para os lados e para a
  frente:
  - berma de cerca de 3 m pelo menos a cada 8 m de altura;
  - platô da descarga de fundo e da casa de máquinas, na cota que ele escolher, com 6 a 12 m de largura.
