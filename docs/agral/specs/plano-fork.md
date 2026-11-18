# Agral CAD: plano da bifurcação do CADcraft

Situação: **decisão do JP em 17/11/2026: "vamos bifurcar o CADcraft então"**. Este plano é para ele corrigir. O código
só começa depois dos três itens da seção 6.

## 1. O que o JP disse

- O CADcraft está "meio bugadão": não tem função 3D, não tem alinhamento, "não tem nada" de civil.
- Do painel e das ferramentas dele, muita coisa funciona e serve de base.
- Definir os nossos próprios estilos.
- Bifurcar, investigar como as coisas são feitas e, se der, otimizar.

## 2. O que o código mostra (lido em 17/11/2026, commit 16341b6, sem executar)

- **Tamanho:** cerca de 115 mil linhas de Rust.
- **Camadas separadas:** geometria (`geom`), banco do desenho (`doc`), leitura e gravação (`io`), comandos (`engine`),
  tela (`ui-egui`) e ponte para IA (`mcp`). A tela não se mistura com o resto, e isso facilita acrescentar um módulo.
- **Comandos:** tudo é um comando registrado (`CommandSpec` em `crates/engine/src/cmd.rs`), com nome, menu, atalho e
  apelidos. Há cerca de 295. Linha de comando, menus, scripts e IA chamam os mesmos comandos. É o lugar certo para os
  comandos de civil.
- **Cota:** a geometria tem vetores 3D, e o desenho já tem ponto, polilinha 3D, face 3D e malha de faces. A
  coordenada Z existe.
- **3D:** não há vista 3D, órbita, sólidos, superfícies nem UCS. O próprio projeto estima 300 a 450 horas para isso.
- **Arquivos:** o DXF é o caminho nativo; o DWG passa por conversão. O que o programa não modela pode se perder ao
  gravar DWG. Dados próprios viajam em xdata.
- **Licença:** MIT ou Apache 2.0. O nome e a marca ArtCraft não entram: o nosso programa precisa de nome e marca
  próprios.
- **Regra do projeto original:** "clean room". Aprender pelo comportamento observado e por especificações públicas;
  nunca ler, desmontar ou copiar o que está dentro do AutoCAD.

## 3. Sobre ler o AutoCAD por dentro

Não faço e não recomendo: desmontar ou copiar o programa da Autodesk fere a licença dela e a regra do próprio
CADcraft, e contaminaria o nosso código. O que serve, e é permitido:

- observar o Civil 3D funcionando: comandos, perguntas, janelas, resultados;
- a ajuda e a documentação públicas da Autodesk;
- arquivos de um trabalho real exportados do Civil 3D (LandXML, DWG, relatório de volumes), usados como gabarito
  dos nossos testes.

## 4. Como o civil entra na bifurcação (proposta)

- **Um módulo novo, `civil`,** só com matemática e testes: superfície (TIN com linhas de quebra), alinhamento, perfil,
  corredor, sample lines, seções e volume. Fica separado do resto para facilitar trazer as novidades do CADcraft.
- **Objetos de civil no desenho:** Superfície, Alinhamento, Perfil, Corredor e Grupo de seções guardam os parâmetros;
  o desenho (curvas, eixo, estacas, rótulos) é gerado a partir deles. Mudou o parâmetro, redesenha.
- **Comandos com os nomes do Civil 3D,** em português e com os apelidos em inglês.
- **Painel "Toolspace":** a árvore com Superfícies, Alinhamentos, Corredores e Grupos de seções.
- **Estilos da Agral** já de fábrica: camadas, cores, textos, curvas, rótulos e carimbo.
- **Arquivos:** os objetos de civil gravados no DXF em dados próprios, mais importar e exportar LandXML, para
  conversar com o Civil 3D e com o Tools.
- **Vista 3D:** fica para depois. Planta, perfil e seção são desenhos 2D, e é neles que o projeto é feito.

## 5. Ordem proposta (cada passo com teste contra um caso real)

1. **Base:** bifurcar, compilar, rodar os testes do original, trocar nome e marca, gerar o instalador para Windows.
2. **Superfícies:** criar por LandXML, pontos e curvas; linhas de quebra; curvas de nível; administrar (listar,
   renomear, estilo, colar uma na outra); cota em qualquer ponto.
3. **Alinhamento:** criar pelos PIs, com curvas; estaqueamento; rótulos; tabela.
4. **Perfil:** terreno ao longo do alinhamento, greide, vista de perfil.
5. **Sample lines e seções:** por intervalo e por estaca; vistas de seção.
6. **Corredor:** seção-tipo ao longo do alinhamento (a barragem primeiro); superfície do corredor.
7. **Volume:** corte e aterro pelas seções e entre superfícies; relatório.
8. **Pranchas:** perfil e seções em folha, com o padrão Agral.

Correções de defeitos que o JP encontrar no uso entram junto, em qualquer passo.

## 6. O que falta para começar

1. **Compilar aqui:** nesta sessão, a compilação do código baixado foi barrada pelo controle de segurança. Preciso
   da liberação do JP para compilar e rodar os testes do CADcraft.
2. **Onde a bifurcação mora:** um repositório do JP no GitHub (bifurcar `storytold/cadcraft` para a conta dele, com o
   nome do nosso programa).
3. **O caso de teste:** um trabalho real com os arquivos do Civil 3D (LandXML da superfície, DWG, relatório de
   volumes), para os nossos resultados serem conferidos contra ele.

---

## 7. Depois do teste do JP no Open CAD Studio (17/11/2026): proposta de trocar a base

**O que o JP disse depois de testar:**
- o plugin de civil é holandês, e ele não consegue usar;
- só traz dados da Holanda, nada do Brasil;
- quer o nosso civil, mais parecido com o Civil 3D, embutido no programa e não como plugin: definir a coordenada e o
  sistema de coordenadas, os dados, e a caixa de ferramentas arrumada.

**O que o código mostra (lido em 17/11, sem executar):**

| | Open CAD Studio (commit 21e5d29) | CADcraft (commit 16341b6) |
|---|---|---|
| Tamanho | Cerca de 1,1 milhão de linhas de Rust | Cerca de 115 mil |
| Português do Brasil | Tradução pronta (`locales/pt-BR`, cerca de 10 mil linhas) | Não vi |
| 3D | Sólidos e câmera em perspectiva | Quase nada |
| DWG e DXF | Lê e grava R14 a 2018 | Grava R2000, sem conferência |
| Local geográfico | O desenho guarda o sistema de coordenadas (código EPSG) | Não tem |
| Pacotes | Aceita pacote próprio que já vem junto do programa, com aba e comandos; os dados ficam nas entidades e sobrevivem ao DWG | Não tem |
| Licença | GPL 3 | MIT ou Apache |

- O motor de civil do plugin holandês tem cerca de 28 mil linhas de Rust: superfície, alinhamento e perfil,
  corredor, seções, terraplenagem, LandXML, rótulos e folhas. É GPL 3, então pode ser adaptado.
- O que nele é da Holanda: os nomes, as mensagens, o padrão de camadas, a fonte de relevo, e os módulos de esgoto e
  de ferrovia.

**Proposta: o Agral CAD parte do Open CAD Studio, e não do CADcraft.**

1. **Pacote "Agral Civil",** com os comandos e os nomes do Civil 3D **em inglês**, como a equipe já usa (decisão
   do JP em 17/11, que troca o "em português"), e o painel de ferramentas (Surfaces, Alignments, Profiles,
   Corridors, Sample Line Groups).
2. **Embutido:** ele vai dentro do nosso instalador, já ligado, com a aba própria. O JP não instala plugin nenhum.
3. **Coordenadas do Brasil:** SIRGAS 2000 com o fuso UTM definido no desenho.
4. **Dados do Brasil:** os que o Hydrus já busca (relevo, hidrografia, CAR).
5. **Motor:** partir do motor de civil existente (GPL 3), traduzindo e tirando o que é da Holanda, em vez de escrever
   do zero. Primeiras mudanças: tirar o teto de 120 mil triângulos e tratar curva de nível como linha de quebra.
6. **Estilos da Agral** de fábrica.

**Consequência da licença:** o Agral CAD fica sob GPL 3. Para uso interno não muda nada. Se um dia for distribuído
para fora, o código tem de ir junto.

**Primeira entrega possível sem bifurcar nada:** como o JP já tem o Open CAD Studio instalado, o pacote Agral Civil
pode ser instalado nele para teste. O instalador próprio ("Agral CAD") vem depois.

**Compilação (17/11):** o motor de civil do Open Infra compilou e passou nos testes dele (514 passaram, 0 falharam,
9 ignorados). O Open CAD Studio inteiro ainda não foi compilado.

**Primeira entrega especificada:** superfície, em `001-superficie/spec.md`.
