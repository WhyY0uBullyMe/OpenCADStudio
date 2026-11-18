# Agral CAD 001: superfície como no Civil 3D

Situação: **aprovada pelo JP ("pode fazer", 17/11/2026), em execução.** Andamento em `tarefas.md`. Primeira entrega do pacote Agral Civil, dentro do Open CAD
Studio (decisão do JP em 17/11, `plano-fork.md` seção 7).

## O que o JP pediu (17/11)

- "Eu quero os mesmos comandos que usa no AutoCAD normal mesmo ... a gente usa ele em inglês ... ninguém sabe os
  comandos em português."
- "Importar, criar superfície, cria uma superfície vazia igual no Civil 3D hoje em dia, e consigo adicionar por
  contornos, por curvas de nível, por pontos, nuvem de pontos, ou blocos, ou DEM."
- Antes, no mesmo dia: embutido no programa, não como plugin; coordenada e sistema de coordenadas no desenho; dados
  do Brasil; caixa de ferramentas arrumada.

## Ponto de partida (fato)

- O motor de civil do Open Infra (GPL 3, `src/civiel/`) compilou e passou nos testes dele: 514 passaram, 0 falharam.
- Ele já tem TIN por Delaunay, curvas, LandXML e leitura de pontos, mas: nomes e mensagens em holandês, teto de 120 mil
  triângulos (o erro de 234.095 que o JP viu), curva de nível usada só como ponto solto, e não tem a ideia de
  "superfície vazia com itens de definição".

## Requisitos

**Comandos e tela**
- **R1.** Os comandos têm o nome do Civil 3D em inglês e aceitam o mesmo atalho de digitar na linha de comando. Os
  nomes exatos vêm da lista de comandos do Civil 3D da Agral (ver "O que falta do JP"); até lá, os nomes abaixo são
  provisórios: `CreateSurface`, `AddSurfaceContours`, `AddSurfacePoints`, `AddSurfacePointCloud`,
  `AddSurfaceDrawingObjects`, `AddSurfaceDEM`, `AddSurfaceBreaklines`, `AddSurfaceBoundaries`, `RebuildSurface`,
  `ImportLandXML`, `ExportLandXML`, `EditDrawingSettings`.
- **R2.** Painel de ferramentas (Toolspace) com a árvore como no Civil 3D: Surfaces > nome da superfície >
  Definition > Contours, Point Files, Point Clouds, Drawing Objects, DEM Files, Breaklines, Boundaries. Clique com o
  botão direito em cada item faz o mesmo que o comando.
- **R3.** Embutido: vem no nosso instalador, já ligado, sem passar pelo comando `PLUGINS`.

**Desenho**
- **R4.** `EditDrawingSettings` define o sistema de coordenadas do desenho: SIRGAS 2000 com o fuso UTM (18S a 25S),
  guardado no DWG. Sem isso definido, importar dado georreferenciado avisa e pergunta o fuso.
- **R5.** A superfície e os seus itens de definição ficam gravados no DWG e voltam iguais ao reabrir.

**Superfície**
- **R6.** `CreateSurface` cria uma superfície TIN **vazia**, com nome, descrição e estilo, que aparece na árvore.
- **R7.** Cada item de definição acrescentado entra numa lista em ordem; a superfície é refeita a partir da lista.
  Tirar ou desligar um item refaz a superfície. Mudou a fonte, a superfície fica marcada como desatualizada até
  `RebuildSurface` (ou refaz sozinha, se a opção estiver ligada).
- **R8. Curvas de nível (contornos):** polilinhas 2D com elevação ou 3D do desenho. Entram como **linhas de quebra**:
  nenhum triângulo cruza uma curva. Opções de afinar (tirar vértices em excesso) e de completar (pôr vértices onde
  a curva tem trecho longo), como no Civil 3D.
- **R9. Pontos:** arquivos de texto e CSV nos formatos PNEZD, PENZD, NEZ e ENZ, com vírgula, ponto e vírgula, espaço
  ou tabulação; decimal com ponto ou vírgula. Linha que não lê é contada e mostrada, não some calada.
- **R10. Nuvem de pontos:** LAS e LAZ, com filtro por classe (padrão: só a classe 2, chão) e redução por espaçamento.
- **R11. Objetos do desenho:** pontos, blocos (pela cota do ponto de inserção), linhas e polilinhas 3D (pelos
  vértices), textos com número (pelo valor).
- **R12. DEM:** GeoTIFF e ASCII Grid (`.asc`), com o sistema de coordenadas lido do arquivo e conferido com o do
  desenho.
- **R13. Linhas de quebra e limites:** linha de quebra a partir de polilinha 3D; limite externo e furos a partir de
  polilinha fechada; comprimento máximo de triângulo para não fechar a borda com triângulo comprido.
- **R14. Sem o teto de 120 mil triângulos.** Meta: superfície de 1 milhão de pontos montada em menos de 1 minuto num
  computador do escritório, e o desenho continua navegável.
- **R15. Estilo Agral de fábrica:** curvas intermediárias e mestras (padrão 1 m e 5 m, a confirmar com o JP), borda,
  e opção de mostrar triângulos e pontos.
- **R16. LandXML:** importar superfície de LandXML (inclusive a do Civil 3D) e exportar para o Civil 3D abrir.

**Diagnóstico e correção de curvas (pedido do JP em 17/11)**
- **R17.** Ao criar ou refazer a superfície com curvas, diagnosticar antes: curvas de cotas diferentes que se
  cruzam, curva que cruza ela mesma, curva que não é plana e polilinha suavizada (spline ou curve fit). Mostrar a
  lista com a cota das duas curvas e o ponto, pôr uma marca no desenho em cada cruzamento e dar zoom em cada um.
- **R18.** Oferecer a correção das polilinhas, sempre mostrando antes o que vai mudar, com desfazer, e sem apagar a
  original sem aviso. Forma de correção a decidir com o JP (ver `caso-cedro-v1.md`).
  Dois casos: (a) curva suavizada (spline ou curve fit) que passou a cruzar a vizinha; (b) curva que já veio
  cruzada do programa de processamento (Metashape, Global Mapper), sem curva original para voltar. O caso (b) é
  comum, segundo o JP (17/11).

- **R19.** Emendar pedaços de curva: juntar pedaços abertos da mesma cota cuja folga entre as pontas for menor que
  uma tolerância (a definir com o JP), sem criar cruzamento e sem emendar ponta que está na borda da área. Mostrar
  antes, com desfazer, na mesma camada nova da correção. Pedido do JP em 17/11 ("vai emendando na mão").

## Fora do escopo desta entrega

Alinhamento, perfil, corredor, sample lines e seções (próximas entregas, nessa ordem), volume entre superfícies,
rótulos de cota, edição manual de triângulos, ler DWG com objetos do Civil 3D por dentro (não se faz: regra de
sala limpa).

## Critérios de aceite

1. O arquivo de curvas que deu o erro dos 234.095 triângulos vira superfície sem erro (R8, R14).
2. Nenhum triângulo cruza uma curva de entrada (teste automático com as curvas do caso real).
3. As curvas geradas pela superfície coincidem com as de entrada: desvio máximo menor que um quarto do intervalo
   das curvas (a confirmar com o JP).
4. A mesma fonte montada no Civil 3D e exportada em LandXML dá, ponto a ponto, cotas iguais dentro de 1 cm no
   interior da superfície (R16, comparação com o caso do JP).
5. Salvar, fechar e reabrir o DWG devolve a mesma superfície e a mesma lista de definição (R5, R7).
6. Cada tipo de fonte (R8 a R12) tem teste com arquivo pequeno de exemplo; um arquivo com linhas ruins mostra quantas
   e quais (R9).
7. Os comandos de R1 funcionam digitados na linha de comando e pela árvore (R2).
8. Instalado pelo nosso instalador, o pacote já aparece ligado (R3). O JP testa no computador dele.

## O que falta do JP

1. O arquivo de curvas (DWG ou DXF) que deu o erro dos 234 mil triângulos.
2. Do mesmo lugar, a superfície do Civil 3D exportada em LandXML, para a comparação do critério 4.
3. Se tiver: um LAS ou LAZ de levantamento e um DEM (GeoTIFF) de um trabalho.
4. A lista dos nomes dos comandos de superfície do Civil 3D como a equipe usa (pode ser foto da linha de comando ou
   do menu), para R1.
5. Intervalos de curva do padrão Agral (R15).
6. Bifurcar o Open CAD Studio para a conta da Agral no GitHub (ou autorizar que eu faça), para o código ter onde
   morar.

## Regras

- GPL 3: o Agral CAD fica sob GPL 3. Uso interno sem mudança; distribuir para fora obriga a ir junto o código.
- Sala limpa: comportamento observado no Civil 3D, documentação pública e arquivos exportados servem de gabarito;
  nada de abrir, descompilar ou copiar o AutoCAD.
- Nenhum dado de relevo é apresentado como levantamento se veio de base pública (marcar a origem no item de
  definição).
