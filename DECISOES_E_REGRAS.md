# Decisões e regras do João Pedro (Agral Tools)

Referência para trocar de agente (Claude Code, Codex ou outro) sem perder o que foi pedido e combinado. Vale para os três repositórios: **Agral Gestão** (original deste arquivo), **Hydrus** e **Project Engine** (cópias iguais em `DECISOES_E_REGRAS.md`). Ao mudar aqui, atualizar as três cópias.

## Regra deste arquivo (pedido do João Pedro, 09/11/2026)

> "Anote como regra tudo que estamos adotando e mudanças que estamos fazendo em algum MD dentro de qualquer git, e explique o que foi solicitado e definido, para servir de referência caso eu precise trocar de agente. Coloque como regra para o outro fazer a mesma coisa."

**Todo agente, a cada pedido do João Pedro:**
1. Anota aqui, na seção "Registro", com a data: **o que ele pediu** (com as palavras dele quando forem importantes) e **o que ficou definido** (decisões dele e decisões do agente, marcadas como tal).
2. Se o pedido vira regra permanente, entra também em "Regras adotadas".
3. Faz isso **no mesmo commit** da mudança, e atualiza as cópias nos outros dois repositórios.
4. O detalhe técnico continua na spec (`docs/specs/NNN-...`), no `docs/PROXIMOS_PASSOS.md` e no mapa (`docs/MAPA_AGRAL_TOOLS.md`); aqui fica o resumo do que foi pedido e decidido.

## Regras adotadas (valem sempre)

| Desde | Regra |
|---|---|
| 06/11 | Método **SDD + BMAD**: spec escrita antes do código (brief, spec, arquitetura quando mexe em dados/servidor, tarefas, testes). Depois, 09/11: **Superpowers primeiro**, e **UI/UX Pro Max + Impeccable** nas telas (`PREMISSAS_PROJETOS.md`). |
| 06/11 | Implementar só com **"pode fazer"**; juntar na `main` só com **"pode juntar"**; publicar no servidor do escritório só com **"pode subir"**. "Pode fazer o resto tudo, só pare quando finalizar" (09/11) autoriza a lista inteira combinada naquele momento. |
| 06/11 | Nunca commitar dados reais (banco, planilhas, CPF/CNPJ, senhas, chaves, tokens). Nunca pedir nem aceitar segredo no chat: senhas e chaves só pelas telas do programa, cifradas no servidor. |
| 06/11 | Permissão conferida no servidor, não só na tela. Valores financeiros sempre calculados pelo programa, nunca pela IA. Migrações só acrescentam; nada de apagar dados em cascata. |
| 07/11 | Textos para o João Pedro em português do Brasil, simples, sem o travessão longo. |
| 07/11 | Acesso de fora só pelo login do Agral Gestão (link do Tailscale Funnel). Tailscale só entre o servidor da nuvem e o servidor do escritório; ninguém da equipe instala Tailscale. Banco principal na nuvem. |
| 07/11 | **Memória no Git**: ao terminar cada etapa (e antes de parar no meio), atualizar `docs/PROXIMOS_PASSOS.md` e a spec, e fazer commit e push, para o Codex ou outro agente continuar. |
| 07/11 | Servidor se atualiza sozinho pela `main` (a cada 5 min, volta sozinho se não abrir). Fluxo: branch, PR, "pode juntar", merge; ninguém precisa entrar no servidor para atualizar. |
| 09/11 | **Agral Tools é o produto final**: a junção do Agral Gestão, Hydrus e Project Engine. Mapa único em `docs/MAPA_AGRAL_TOOLS.md` (original no Agral Gestão, cópias iguais nos outros dois). Ler o mapa antes de começar algo novo. |
| 09/11 | Ordem: Agral Gestão redondo até dezembro (a Elaine larga a planilha em janeiro de 2027); depois o editor de memorial do Project Engine; o Hydrus um módulo por vez; juntar no Agral Tools por último. Painel tipo Craft Anything, conversor de arquivos e MCP em todos ficam para depois. |
| 09/11 | Commit (e push) assim que cada parte ficar pronta e testada. |
| 09/11 | Este arquivo: registrar tudo o que é pedido e definido (regra acima). |
| 09/11 | **Pacote Agral** (`pacote/` no Agral Gestão, instalado em `.claude/skills` dos três repositórios): ler `agral-metodo` antes de qualquer tarefa e usar a skill de cada etapa (brainstorming, spec, ponytail/TDD, ui-ux-pro-max, differential-review, verification). Dizer ao JP quais skills foram usadas. |

## Registro (mais recente primeiro)

### 18/11/2026, noite (Agral CAD)
- **JP:** "a partir de agora iremos subir no Agral CAD e operar apenas nele". Autorizou fazer os quatro pendentes
  (painéis por painel, janelas de configuração, abas por seleção com sample lines arrastáveis, imagem XYZ) e, ao
  terminar, uma **revisão geral** do programa: estrutura, testes pelas premissas (SDD + BMAD e skills do protocolo),
  funcionalidades, aparência e feedback visual.
  - Feito: método trazido para este repositório (AGENTS.md, CLAUDE.md, PREMISSAS_PROJETOS.md, este arquivo, skills em
    `.claude/skills`, mapa em `docs/agral/`, specs em `docs/agral/specs/`). Este arquivo passa a ser o registro do
    Agral CAD; o do Hydrus fica como histórico.
  - Feito (painéis): o pino agora é por grupo de painéis, não pela borda inteira. Grupo fixo ocupa o seu espaço;
    grupo recolhível só mostra o ícone e abre por cima ao passar o mouse, sem tirar o fixo do lugar nem mexer no
    desenho. Botão novo na barra de título ("Stack"): empilha o painel com o fixo do lado (os dois dividem a altura,
    com divisor arrastável) e, de novo, desempilha. Três testes do programa base esperavam o pino da borda inteira e
    foram atualizados para a regra nova.
  - Feito (janelas): como no AutoCAD, o comando sozinho (botão da faixa ou menu do Toolspace) abre uma janela já
    preenchida com o que o eixo ou a superfície têm, e o "-COMANDO" continua perguntando na linha de comando (roteiros
    e MCP). Janelas: Compute Materials, Create Sample Lines and Section Views, Create Profile View, Dam Assembly,
    Create Corridor, Alignment Stations e Extract Objects. Campo vazio avisa e não fecha; quando o comando precisa de
    um ponto (canto da vista), ele é pedido no desenho depois do OK.

### 18/11/2026 (Agral CAD, com o Claude)
- **Retorno do JP sobre o teste no Windows:** "abre normal", mas "tá muito mal estruturado": foi feito um plugin no
  mesmo programa; não há como administrar as superfícies; criou a superfície e o `ADDSURFACECONTOURS` disse que era
  preciso criar uma.
  - **Pedido:** "veja como é feito no CAD e crie aqui igualzinho" e "quero a minha versão do programa e não um plugin".
  - **Causa do erro (achada no código):** para pacote, o `show_dialog` do Open CAD Studio abre a janela e já responde
    "cancelado" na hora; o OK da janela "Create Surface" nunca chegava ao pacote, e a superfície não era criada, sem
    aviso. Os testes em lote passavam o nome no comando e não pegaram isso.
  - **Decisão do JP:** o Agral CAD é um programa próprio (nome, ícone e janela da Agral), com o civil **dentro do
    núcleo**, não como pacote. Isso troca a recomendação de 17/11 (pacote de fábrica). Ganho extra: sem o limite de
    64 MB por mensagem e de 30 s por comando, e sem a demora de mandar a malha para outro processo.
  - **Definido:** gerenciar as superfícies como no Civil 3D: painel Toolspace (Prospector) com a árvore
    Surfaces > superfície > Definition > Contours (e os outros tipos), botão direito com Add, Rebuild, Properties e
    Delete; janela Create Surface; tudo pelo comportamento público do Civil 3D (sala limpa).
  - **Feito (18/11):** civil passado para o núcleo do Agral CAD (aba Civil, Toolspace com a árvore e menus, janelas
    Create Surface, Add Contour Data, Surface Properties e Delete, estilos Agral de superfície); pacote removido;
    marca Agral CAD no título e na tela inicial, sem doação e links do projeto original. Testado na tela (Xvfb, com
    cliques): criar, acrescentar curvas e o menu do botão direito funcionam. Arquitetura em
    `docs/specs/agral-cad/001-superficie/arquitetura.md`.
  - **Definido por mim, a confirmar com o JP:** o estilo padrão mostra curvas (1 m e 5 m) e a borda, como o padrão do
    Civil 3D; triângulos só nos estilos que pedem.
- **Pedido do JP (18/11): "tenho vários estilos de superfície: 0.1m 0.5m 1m 2m 5m apagado e borda"**, e na compilação
  para Windows já mudar o estilo e o nome do programa.
  - **Feito:** estilos de fábrica Curvas 0.1m, 0.5m, 1m (padrão), 2m e 5m (com borda), Apagado e Borda, mais
    Triângulos para conferir a malha. **A confirmar:** mestra a cada 5 intermediárias (0,5, 2,5, 5, 10 e 25 m).
  - **Feito:** marca própria: logo AGRAL (letras e três curvas de nível), "Agral CAD" no título, na tela inicial, na
    janela Sobre (com o crédito ao Open CAD Studio, que a GPL 3 pede) e nas propriedades do `.exe`
    (`AgralCAD.exe`); tela inicial sem Reddit, patrocinadores, discussões e apoiadores do projeto original.
- **JP (18/11):** mandou o template dele (`TEMPLATE_JP.dwt`, "dá pra ver tudo aí") e pediu o ícone do programa
  direito e o logo da Agral "bonitinha"; o logo desenhado por mim "ficou muito feio".
  - **Feito:** logo oficial da Agral (`app/web/agral-cor.png` do Hydrus) vetorizado: letras AGRAL no canto do
    programa (brancas no tema escuro, escuras no claro); ícone com o "A" da Agral em branco sobre azul-marinho e um
    traço dourado (janela, Sobre e `.exe`).
  - **Feito:** o template vai junto do programa (pasta `templates`) e é o padrão do New Drawing; a superfície
    desenha nas camadas dele (CURVA MESTRA-AS, CURVA INTERMEDIÁRIA-AS, BORDA SUPERFÍCIE-AS, TRIANGULAÇÃO-AS).
    Conferido: o template não tem dado pessoal (sem autor; só o bloco de ponto com PONTO, DESC e COTA). Ele fica
    no repositório público da bifurcação, junto do programa.
- **Retorno do JP depois de testar no Windows (18/11):** "ficou bom, deu certo"; ícone "perfeito". Pedidos: superfície
  como objeto do Civil 3D (seleção, aba própria, Extract Objects, editar estilo, Boundaries, editar triângulos);
  painéis com fixar individual (Properties fixo, Toolspace ao passar o mouse); aparência mais bonita; logo do menu
  do programa feio; lugar para o agente de IA; Prospector completo; feature lines com Elevation Editor; Grading
  Creation Tools; datum e imagem de satélite por link XYZ. Lista e situação em
  `docs/specs/agral-cad/fila-pedidos-18-11.md`.
  - **Feito no mesmo dia:** superfície como bloco único com a aba TIN Surface, Extract Objects, Boundaries (Outer,
    Hide) e Edits (Add Point, Delete Point, Delete Line, Raise/Lower), testados em lote e na tela.
  - **Aviso dado ao JP:** imagem do Google por link XYZ fica fora dos termos do Google; o programa aceita qualquer
    link e a escolha do serviço é dele.
- **Decisões do JP (18/11, depois do aviso):**
  - **Imagem do Google por link XYZ: "pode fazer"**, é só para uso interno, não vai vender. Decisão dele, com o aviso
    dos termos de uso já dado.
  - **Cota ao passar o mouse sobre a superfície** (como no Civil 3D): pedido.
  - Quer a bifurcação **privada**, só para ele. (O GitHub não deixa bifurcação de projeto público ser privada; a
    saída é um repositório privado novo, criado pelo JP, e eu envio o código para lá.)
  - **Olhar as bifurcações do Open CAD Studio** atrás de trabalho mais elaborado para aproveitar (análise em
    andamento).
  - **Toolspace mais "apetitoso"**, ainda "meio bagunçadão".
  - **Prioridades para usar o programa como padrão:** corredor, assemblies, feature lines e Grading Creation Tools.
  - **Feito em seguida (18/11):** análise das bifurcações (`docs/specs/agral-cad/analise-forks.md`: ninguém tem
    corredor, assembly, feature line editável nem grading; dá para aproveitar imagem XYZ e alinhamento/volumes);
    cota no mouse sobre a superfície (etiqueta perto do cursor; ao reabrir o desenho o TIN é montado em segundo
    plano); Properties da superfície como "TIN Surface" com estilo editável e estatística; duplo clique abre Surface
    Properties; Toolspace abre na borda oposta ao Properties, escondendo sozinho (o fixar do Open CAD Studio vale por
    borda, não por painel).
- **JP (18/11, depois da cota no mouse):**
  - Comandos que a equipe usa: os que já fizemos, mais copiar com PASTEORIG, HIDEOBJECTS, LAYOFF, LAYON, PR
    (Properties), MEASURE e POLAR. Todos já existem na base (conferido em `docs/commands-reference.md`).
  - Vai mandar o `acad.pgp` (atalhos do AutoCAD) e os Lisps dele para virarem comandos do programa. Exemplos:
    spline para polilinha escolhendo a quantidade de vértices; quebrar a polilinha em vértices a cada 5 m, 2 m etc.
  - Vai criar o repositório privado no GitHub.
  - **Prioridade agora: aparência.** "Tá muito copiado do cara ainda", feio, feedback visual fraco. Melhorar aos
    poucos, do jeito que ele quer ver as coisas.
  - **Feito em seguida:** tema "Agral" (paleta do Hydrus) como padrão do Agral CAD.
- **JP mandou (18/11):** `acad.pgp` e os Lisps `DIVGRIPS.lsp`, `Spl2pl.vlx`, `TE_TableExport.lsp`, `lockup.lsp`
  (antes tinha mandado `acad2026.lsp` e `acadinfo.lsp`, que são da Autodesk: não usados, regra da sala limpa).
  - Atalhos próprios dele no `acad.pgp`: `PPP` = PASTEORIG, `SZ` = SELECTSIMILAR, `HH` = HIDEOBJECTS. Entraram na
    tabela de fábrica do Agral CAD. Os outros atalhos do arquivo são os de fábrica do AutoCAD; o Agral CAD já tem
    a maioria e importa um `.pgp` pelo ALIASEDIT.
  - **DIVGRIPS** (Lisp dele): refaz a polilinha com vértice a cada X m ao longo dela, pergunta se apaga a
    original. Refeito como comando do programa.
  - **SPL2PL** (`.vlx` compilado, não dá para ler): feito pela descrição dele, spline ou curva para polilinha com a
    quantidade de vértices escolhida.
  - `TE_TableExport` (tabela de linhas e textos para o Excel) e `lockup` (trava o desenho em blocos anônimos) são
    de terceiros. Ficam na fila, para perguntar se ele usa.
- **JP (18/11, resposta):**
  - O caso Murilo **não é teste agora**: no Civil 3D ele ainda não gerou corredor nem nada. Quer **poder gerar um
    corredor** no Agral CAD.
  - **TE**: usa às vezes, mas não quer o TE. Quer um **vínculo de tabela com o Excel nos dois sentidos** (editou no
    CAD, muda no Excel; editou no Excel, muda no CAD), talvez com os nossos próprios programas. Fila.
  - **LOCKUP**: usa. Fila.
  - Repositório privado: ainda não criou.
  - Pediu: **analisar o que o AutoCAD e o Civil 3D têm que nós não temos**, útil e indispensável para ele (análise
    pela documentação pública e pelo comportamento, sem abrir os programas).
  - Feito: análise em `docs/specs/agral-cad/analise-lacunas.md`.
- **JP (18/11, testando a cota no mouse):** a etiqueta "fica aparecendo o tempo inteiro" e longe do cursor. Quer como
  no Civil 3D: só depois de uns **2 s com o mouse parado**, janelinha **branca** com nome da superfície e cota.
- **JP (18/11): "faz o corredor... faz os dois, faz tudo"**: autoriza pontos, linhas de quebra, LandXML e o corredor
  (alinhamento, perfil, assembly), na ordem que eu achar melhor.
- **JP (18/11):** quando carrega, "parece que trava" e a parte de cima da janela fica branca: **barra de
  carregamento**. Usar sempre o método e as skills dos nossos projetos sem ele precisar repetir. Quer um
  **programa instalado de verdade** (instalador), leve, que não trave.
- **JP (18/11): "quero um programa profissionalzão"**: rodar um roteiro de teste, ver o que falta e está bugando,
  analisar o CAD e melhorar, sem esperar instrução detalhada.
  - Feito: roteiro de teste pela tela (`docs/specs/agral-cad/roteiro-tela-18-11.md`), com 5 achados corrigidos;
    montagem em segundo plano com aviso de progresso; instalador MSI.
- **JP (18/11): "a interface ainda está muito pobre"**, principalmente o Toolspace sem ícones (no Civil 3D a
  superfície tem um quadradinho com as curvas); ícones que não têm nada a ver. Criar um ícone para cada coisa.
  - Feito: ícones próprios (desenho nosso) em cada linha do Toolspace e em cada botão da aba TIN Surface.
- **JP (18/11): "termina o alinhamento, perfil e seções antes de me mandar; sem essas coisas o programa não me
  serve ainda".**
  - Feito (spec 003 R4 a R6 e seções): alinhamento com estacas, vista de perfil com faixa de cotas e aterro/corte,
    greide com curva vertical, linhas e vistas de seção; perguntam na linha de comando; Toolspace e botões.
- **JP (18/11): criou o repositório privado `WhyY0uBullyMe/AgralCad`** e pediu para trazer o Agral CAD. Feito: a
  branch `agral-cad` (com o histórico) foi enviada para lá; daqui em diante o trabalho vai para o privado. A
  compilação do Windows cancela a anterior a cada envio, porque repositório privado conta minutos do GitHub
  (Windows vale em dobro). A bifurcação pública (`WhyY0uBullyMe/OpenCADStudio`) o JP apaga quando quiser, depois
  de baixar a versão que está compilando nela.
- **JP (18/11): compilação do Windows "só quando eu TE mandar"**: a compilação passou a ser só à mão. **"Pode
  seguir pro corredor"**: feito o assembly da barragem e o corredor (spec 003 R7 a R9), com superfície do corredor
  limitada pela saia e volumes de aterro e corte.
- **JP (18/11): "uma abinha Home com meus negócios, igual no CAD"** e **Object Viewer** "para olhar o 3D".
  - Feito: aba Home, a primeira, com Palettes, Create Ground Data, Create Design, Profile & Section Views e os
    painéis de desenho, edição, camadas e área de transferência. `OBJECTVIEWER [exagero]` abre os objetos
    selecionados em 3D numa aba à parte (superfície colorida por faixa de cota), sem mexer no desenho e sem pedir
    para salvar.
- **JP (18/11, testando o perfil):** melhorar a aparência do perfil; dentro da vista de perfil, mouse parado mostra a
  cota do local; tirar o destaque laranja ao passar o mouse; o Object Viewer tem de abrir numa janela própria, não
  como outro DWG nas abas.
  - Feito: grade do perfil que não encavala rótulos, moldura, cotas dos dois lados, faixa só com o que existe;
    dica com estaca, cota, terreno e greide dentro da vista; destaque parado desligado (durante comandos continua);
    Object Viewer numa janela própria com 3D sombreado (girar, mover, zoom, duplo clique volta).
- **JP (18/11, testando):** grips de polilinha aparecem só em parte, "quero ver todos ou nenhum"; no Extract Objects,
  escolher todos, só mestras ou só intermediárias; várias formas de criar alinhamento (ferramenta de traçado ou
  selecionar o objeto, como o Create Alignment Entities); espaço vale Enter; curvas de nível geradas com "dentes";
  grips e quadradinho do cursor mais parecidos com o AutoCAD.
  - Feito: grips tudo ou nada (acima de 4096 somem todos os pontos médios; se nem os vértices cabem, nenhum grip);
    `EXTRACTSURFACEOBJECTS` pergunta `[All/MAjor/MInor/Border/Triangles] <All>`; `CREATEALIGNMENTENTITIES`
    (selecionada vira eixo na hora) e `CREATEALIGNMENTLAYOUT` (PIs clicados, raio nas curvas); `CREATEALIGNMENT`
    sem nada pergunta `[Entities/Layout]`; perguntas aceitam espaço como Enter, menos nome de camada e estilo;
    arcos (bulge) das curvas de origem viram pontos com flecha de 2 cm e spline passa a entrar (antes o arco virava
    reta e a spline ficava de fora); grips cheios azul/rosa/vermelho (GRIPCOLOR 150, GRIPHOVER 11, GRIPHOT 12) e
    quadradinho do cursor de 15 para 10 px.
  - Pendente: se o dente continuar no arquivo real, olhar triângulos planos (minimizar áreas planas) com o DWG dele.
- **JP (18/11, à noite):** Compute Materials "igual no Civil 3D", com cálculo por superfície, para tocar os projetos
  agora (incrementar depois); seções como as dele, pelo TEMPLATE_JP. No perfil: ver onde está clicando, não precisar
  ligar o greide desde o início do eixo e editar depois. Pedidos novos: ferramentas que mudam conforme o que está
  selecionado (seções: Sample More Sources), arrastar as sample lines, criar com mais controle; botões e janelas para
  configurar em vez de só perguntas na linha de comando; fixar ou recolher cada painel individualmente e deixar
  painéis fixos dividindo o mesmo lado; não achou como ligar as imagens de satélite.
  - Feito: spec `docs/specs/agral-cad/004-compute-materials`; `COMPUTEMATERIALS` (qualquer par de superfícies, padrão
    TN x corredor, fatores de corte e aterro), `CREATEVOLUMETABLE`, `EXPORTMATERIALS` (CSV para o Excel), totais no
    Toolspace; seções nas camadas do template (os estilos do Civil 3D ficam em objetos próprios dele, não lidos:
    para os rótulos iguais falta print ou DWG explodido). Greide só entre o primeiro e o último PVI; prévia ao clicar
    PVIs; `EDITPROFILE` (Move, Add, Delete, Curve).
  - A fazer, nesta ordem proposta: painéis fixos ou recolhíveis por painel e empilhados no mesmo lado; janelas de
    configuração (Compute Materials, seções, perfil); abas contextuais por seleção e sample lines arrastáveis;
    imagem de satélite por link XYZ (ainda não existe no programa).

### 17/11/2026 (Hydrus, com o Claude)
- **Pedidos do JP:**
  - **Campo entra na estrutura:**
    - geração de PPP pelo IBGE;
    - hoje usam o programa da CHC para juntar observações, corrigir a altura da antena e trocar a versão do RINEX
      quando o PPP não aceita;
    - o processamento seria o "Metashape nosso".
  - **Dentro do trabalho** (abaixo de cliente e fazenda) entram os ensaios de laboratório e de solo e a sondagem.
    Se não houver, o programa gera todo o resto como prévia, e o documento diz que não inclui fundação nem análise de
    estabilidade, por não haver os dados necessários.
  - **CAD nosso é prioridade agora: "estou parado".** Base sugerida: LibreCAD, numa versão nossa do Civil 3D, com
    alinhamento, corredor, administração de superfícies e sample lines, bem parecido com o Civil 3D.
- **Feito:** instalador 0.38.2 (prévia do Tools atualizada); análise das bases do CAD em
  `docs/specs/agral-cad/brief.md`; estrutura final atualizada com Campo, a regra da prévia e o CAD como prioridade.
- **Achado da análise:** nem o LibreCAD (GPL, só 2D) nem o CADcraft (MIT ou Apache, em "pre-alpha", DWG gravado sem
  conferência) têm funções de civil. O motor civil tem de ser nosso em qualquer caminho. Recomendação: motor civil no
  Tools e desenho no CADcraft. Aguardando a decisão do JP.
- **Decisão do JP (17/11): "vamos bifurcar o CADcraft então".**
  - Ele acha o CADcraft "meio bugadão", sem 3D e sem alinhamento, mas com painel e ferramentas que servem de base.
  - Quer os nossos próprios estilos.
  - Ofereceu passar o programa do AutoCAD para leitura por dentro. **Não será feito:** desmontar ou copiar o AutoCAD
    fere a licença da Autodesk e a regra de "clean room" do próprio CADcraft. Usaremos o comportamento observado, a
    documentação pública e arquivos reais exportados do Civil 3D como gabarito.
  - **Feito:** código do CADcraft baixado e lido (sem executar); plano em `docs/specs/agral-cad/plano-fork.md`.
  - **Pendente:** liberação para compilar nesta sessão, o repositório da bifurcação e um caso real de teste.
- **Pedido do JP (17/11):** pesquisar no GitHub quem já fez as funções do Civil 3D em código aberto.
  - **Achado:** o Open CAD Studio (CAD em Rust, GPL 3, 2D e 3D, DWG e DXF, instalador para Windows) com o plugin
    Open Infra Studio (GPL 3) declara ter superfície TIN, curvas, alinhamento, perfil, seções, corredor e
    terraplenagem. Nada foi instalado nem executado.
  - **Recomendação:** testar esse par num trabalho real antes de bifurcar o CADcraft. Detalhes em
    `docs/specs/agral-cad/brief.md`. **A decisão de bifurcar o CADcraft fica em espera até o teste.**
- **Teste do Open CAD Studio (17/11):** o JP instalou e perguntou como criar uma superfície. A superfície vem do
  plugin Open Infra Studio (aba "Open Infra - Civiel"): instalar pelo comando `PLUGINS`; importar pontos com
  `CIV_PUNTEN_IMPORT` ou `CIV_LANDXML`; criar a superfície com `CIV_TIN`; curvas com `CIV_HOOGTELIJNEN`. Passos
  tirados do README do plugin, não testados aqui.
- **Retorno do JP depois do teste (17/11):** o plugin é holandês e só tem dados da Holanda; ele não consegue usar.
  Quer o nosso civil, parecido com o Civil 3D, embutido no programa (não como plugin), com coordenada e sistema de
  coordenadas, dados do Brasil e a caixa de ferramentas arrumada.
  - **Proposta registrada** em `docs/specs/agral-cad/plano-fork.md`, seção 7: o Agral CAD parte do Open CAD Studio
    (GPL 3, já em português, 2D e 3D, DWG), com o pacote "Agral Civil" embutido. Aguardando a decisão do JP sobre a
    base e a liberação para compilar.
  - **Decisões do JP (17/11):** a base do Agral CAD é o **Open CAD Studio**, com o nosso civil embutido em
    português; e **pode compilar** e rodar os testes desse código nesta sessão.
  - **Compilado (17/11):** o motor de civil do Open Infra compilou aqui e passou nos testes dele (514 passaram, 0
    falharam, 9 ignorados). O Open CAD Studio inteiro ainda não foi compilado.
- **Pedido do JP (17/11): comandos iguais aos do AutoCAD e do Civil 3D, em inglês.** "Hoje a gente usa ele em
  inglês ... ninguém sabe os comandos em português."
  - **Definido:** os comandos têm os mesmos nomes do AutoCAD e do Civil 3D em inglês (`LINE`, `PLINE`, `OFFSET`,
    `CreateSurface`, `CreateAlignmentLayout`, `CreateCorridor`, `CreateSampleLines` e assim por diante). Isso
    substitui o "em português" da proposta da seção 7 do `plano-fork.md`.
  - **Suposição minha, a confirmar:** os menus podem ficar em inglês também, já que a equipe usa assim; o Open CAD
    Studio já tem inglês e português para escolher.
- **Pedido do JP (17/11): superfície como no Civil 3D.** Criar a superfície vazia e depois acrescentar dados a ela:
  curvas de nível (contornos), pontos, nuvem de pontos, blocos e DEM; e importar.
  - **Definido:** é a primeira entrega do Agral Civil. Spec em `docs/specs/agral-cad/001-superficie/spec.md`.
- **Pedido do JP (17/11):** passar o CAD para uma sessão filha deste Hydrus, para ele seguir com o resto aqui.
  - **Definido:** a sessão filha cuida só do Agral CAD, numa branch própria (`agral-cad`), para não misturar com o
    trabalho desta branch. Bifurcar o Open CAD Studio no GitHub da Agral continua dependendo do JP.
- **Sessão filha do Agral CAD (17/11):** investigado como o Agral Civil vai embutido no Open CAD Studio, sem mudar
  código. Achados e proposta em `docs/specs/agral-cad/001-superficie/investigacao-embutir.md`: motor em biblioteca
  própria testável; comandos e árvore num pacote de fábrica que vem ligado no instalador; comandos com o nome do
  Civil 3D funcionam sem prefixo; superfície desenhada como malha, não um `3DFACE` por triângulo (causa real do teto
  de 120 mil). Aguardando o "pode fazer" da spec 001 e os arquivos do JP.
- **Decisão do JP (17/11): "pode fazer, pode bifurcar você mesmo".** Spec 001 (superfície) aprovada; autorizou eu
  mesmo bifurcar o Open CAD Studio.
  - **Bifurcação travada:** esta sessão só tem acesso ao repositório do Hydrus, e a permissão para incluir o Open CAD
    Studio foi negada pelo controle automático. Falta o JP bifurcar pelo botão "Fork" do GitHub ou liberar o acesso.
  - **Feito:** motor da superfície (`agral_civil_motor`), com 19 testes passando; 1 milhão de pontos em 1,5 s. Cópia
    do código em `docs/specs/agral-cad/codigo/` até a bifurcação existir. Estado em
    `docs/specs/agral-cad/001-superficie/tarefas.md`.
- **JP enviou (17/11):** o DWG de curvas da Fazenda Cedro (área 2) e a pasta do Drive com `CURVAS_V1.dwg` (61 MB) e
  `Curvas - Murilo.xml` (172 MB).
  - **Feito:** o DWG da Cedro virou superfície no nosso motor: 185 mil triângulos em 0,17 s, nenhum cruzando curva.
    Achado: triângulos planos ainda deixam a curva gerada fora da original em 0,7 % dos pontos. Detalhes em
    `docs/specs/agral-cad/001-superficie/caso-cedro.md`. O arquivo do cliente não vai para o repositório.
  - **Os dois arquivos do Drive não foram baixados:** a ferramenta do Drive desta sessão traz o arquivo inteiro como
    texto para a conversa, e 61 MB e 172 MB não cabem. Pedido ao JP: mandar compactados pelo chat, como o DWG.
  - O Open CAD Studio inteiro compilou aqui (6 min, sem erro).
- **Retorno do JP (17/11):** o problema no plugin holandês não foi o teto, "simplesmente não deu" criar a superfície;
  pelo chat o arquivo grande não sobe ("muito grande"), "arruma um jeito de baixar"; a polilinha da camada LAYER é o
  limite da área; o `Curvas - Murilo.xml` é da mesma área das curvas.
  - **Feito:** baixados os dois arquivos direto do Drive (pasta compartilhada por link), na pasta de dados da sessão.
- **Pedido do JP (17/11):** ao criar superfície com curvas, **diagnosticar as curvas de cotas diferentes que se cruzam
  e corrigir as polilinhas**, "para resolver um problemão que eu estou tendo ultimamente".
  - **Definido:** entra na spec 001 como R17 (diagnóstico) e R18 (correção). Diagnóstico feito no motor, com testes.
  - **Achado no `CURVAS_V1.dwg`:** as curvas originais não se cruzam; a suavização spline das polilinhas cria 1.208
    cruzamentos em 201 pares de curvas. O Civil 3D usou a curva suavizada. Detalhes em
    `docs/specs/agral-cad/001-superficie/caso-cedro-v1.md`. **Forma da correção: aguardando o JP.**
- **JP bifurcou o Open CAD Studio (17/11):** `WhyY0uBullyMe/OpenCADStudio`, com o mesmo nome do original.
  - **Feito:** o código do Agral Civil está na branch `agral-cad` dessa bifurcação; tirada a cópia provisória
    (`docs/specs/agral-cad/codigo/`) do Hydrus. Nada na `main` da bifurcação.
- **JP (17/11):** "muitas vezes a polilinha já vem cruzada do programa de processamento (Metashape ou Global Mapper)".
  - **Consequência:** a correção (R18) não pode depender só do polígono de controle da spline; precisa de um método
    geral para curvas cruzadas de origem. Proposta enviada ao JP; aguardando a escolha e um arquivo de exemplo.
- **JP (17/11, áudio cortado):** as curvas "cruzam umas com as outras" e hoje a equipe "vai emendando na mão, que é
  muito trabalhoso".
  - **Medido nos dois DWG da Cedro:** curvas partidas em pedaços abertos (658 de 2.109 na área 2; 1.152 de 4.773 no
    V1), com cerca de 160 e 270 pontas a até 5 m de outro pedaço da mesma cota.
  - **Proposta:** R19, emendar automático os pedaços da mesma cota cuja folga for menor que uma tolerância, sem criar
    cruzamento, mostrando antes. Aguardando o JP confirmar o entendimento e a tolerância.
- **Decisão do JP (17/11): "eu quero que ela não cruze, apenas isso. E tem esse caso aí dentro."** O método da
  correção ficou comigo; o critério é zero cruzamento.
  - **Feito:** correção (R18) no motor: separa cada lente pela linha do meio e corta o trecho do que sobrar. No
    `CURVAS_V1.dwg`: 1.208 cruzamentos viram 0, com 62 m cortados de 1.042 km de curvas. Detalhes em
    `docs/specs/agral-cad/001-superficie/caso-cedro-v1.md`. Emendar (R19) segue aguardando a tolerância.
- **JP (17/11):** os DWG enviados (`CURVAS_V1` e a área 2 da Cedro) já são a saída do **Global Mapper** (não do
  Metashape). Não vai mandar outro arquivo.
  - **Consequência:** o caso de "curva que já vem cruzada do programa de processamento" já está testado: os 1.208
    cruzamentos do `CURVAS_V1` vêm da suavização spline feita pelo Global Mapper, e a correção zerou todos.
  - O JP não entendeu a pergunta da folga para emendar (R19); refeita em palavras simples.
- **Regra do JP para corrigir cruzamentos (17/11):** em geral é a curva de cima que faz uma "barriguinha" para dentro
  da de baixo; essa barriga "tem que ser cortada e alinhada conforme seria antes", sem fazer canto. **Emendar os
  pedaços: sim,** hoje a equipe faz à mão, "coerente com a superfície".
  - **Feito:** a correção corta primeiro a barriga (a curva que mais se afasta do próprio caminho; empate fica com a
    de cima) e refaz o trecho seguindo a direção de antes e de depois, conferindo que não cruza nenhuma curva. O que
    não for barriga clara vai para a linha do meio, e o resto é cortado. `CURVAS_V1`: 1.208 cruzamentos para 0, com
    53 barrigas, 548 lentes e 34 cortes, em 22 s. Emendar (R19): tolerância de 5 m, ajustável, a fazer.
- **Pergunta do JP (17/11): "avalie qual a melhor forma de construção desse app. Rust talvez?"**
  - **Recomendação:** Rust para o Agral CAD e o motor de civil (já é a linguagem da base); o Hydrus continua em
    Python e web. Detalhes em `docs/specs/agral-cad/avaliacao-tecnologia.md`.
- **Pedidos do JP (17/11) para o Agral CAD:**
  - comandos que a equipe usa muito: `PASTEORIG` (colar nas coordenadas originais, de um desenho para outro),
    `HIDEOBJECTS`, `ISOLATEOBJECTS`, `MATCHPROP` "e vários outros";
  - editar os atalhos (alias) dos comandos;
  - presets de estilo e aparência para superfície e outros objetos (estilos Agral);
  - aparência do programa na linha do Hydrus, "paleta bonita e bem ajeitada"; o layout será decidido depois;
  - **objetivo: um programa mais rápido e menos travado que o AutoCAD.**
  - **Conferido no código do Open CAD Studio:** `PASTEORIG`, `COPYBASE`, `HIDEOBJECTS`, `ISOLATEOBJECTS`,
    `UNISOLATEOBJECTS`, `MATCHPROP` (atalho `MA`), `LAYISO`, `QSELECT` e `SELECTSIMILAR` já existem com os nomes do
    AutoCAD; `ALIASEDIT` edita os atalhos e importa o `.pgp` do AutoCAD. Não testados na tela ainda. A confirmar:
    colar entre duas janelas separadas do programa (entre abas do mesmo programa o código trata).
  - **Definido:** estilos viram a spec 002; a aparência fica para depois da 001, a pedido do JP; a velocidade vira
    critério de aceite medido contra o AutoCAD no computador do JP.
- **JP (17/11): "pode seguir"; folga de 5 m para emendar aprovada.** A lista de comandos e o `acad.pgp` ele manda
  depois.
  - **Feito:** emendar pedaços (R19) no motor, 8 testes. Área 2 da Cedro, com o limite da área: nenhuma emenda (as
    pontas soltas estão na borda). `CURVAS_V1`, depois da correção e com o limite do LandXML: 90 emendas entre
    pedaços e 663 curvas fechadas de novo (eram fechadas no original e a suavização deixou folga), sem cruzamento.
  - **Feito:** pacote de fábrica `agral.civil` dentro do Open CAD Studio, com `CREATESURFACE`, `ADDSURFACECONTOURS`
    e `REBUILDSURFACE` (aba "Agral Civil"). Carrega sozinho da pasta `plugins` ao lado do `.exe`. Testado em lote:
    área 2 da Cedro (186 mil triângulos em 2,2 s; salva, reabre e refaz igual) e `CURVAS_V1` (2,24 milhões de
    triângulos em 21 s, 1.208 cruzamentos marcados). Gargalo achado: mandar a malha para a tela leva 17 dos 21 s.
  - **Feito:** receita no GitHub Actions da bifurcação que gera um ZIP para Windows com o pacote embutido, para o JP
    testar no computador dele.

### 16/11/2026 (Hydrus, com o Claude)
- **Retorno do JP sobre o protótipo da estrutura:** "ainda está bem fraco", mas a estrutura de Home e Demandas agradou.
  - **Ferramentas:** tem de ter todas as ferramentas, separadas por área (hidrologia, irrigação, outorga, estrutural
    e as outras).
  - **Extravasor:** a ferramenta fica livre para todos ("pode deixar livre"). Isso troca a decisão anterior de
    cálculo restrito. A revisão técnica do JP antes da entrega continua.
  - **Estudos x ferramentas:** o que é de estudo (por exemplo o estrutural) entra na parte de estudos; as ferramentas
    abrem cada uma solta.
  - **CAD: nova demanda.** Pensar o nosso CAD com base no CADcraft, que é de código aberto, aproveitando o que der e
    acrescentando o que é nosso. Antes de copiar, conferir a licença.
  - **O Agral Tools é o produto final de tudo:** junta o Studio (o Project Engine, que faz os projetos), os estudos e
    a parte de projetos do Gestão (o controle de demandas de quem opera projetos).
  - **Pedido:** reler tudo o que o JP já descreveu nos repositórios, pensar como implementar e apontar o que falta
    na estrutura.
- **Retorno do JP sobre o protótipo refeito (16/11):**
  - **Ferramentas é só o que é simples.** O que depende de um contexto maior (exemplo: CAPEX e OPEX) não é
    ferramenta: é estudo, e só aparece dentro de um trabalho.
  - **Uma aba inteira para o mapa,** que abre direto, "como o Word abre no documento": o globo já com as fazendas e
    as obras lançadas.
- **Retorno do JP (16/11): o mapa do protótipo estava feio, "um quadrinho".**
  - **Pedido:** o Mapa como um Google Earth, em tela inteira, com janelas soltas para mexer, em nível profissional,
    no padrão do Agral Gestão.
  - **Feito no protótipo:**
    - a aba Mapa usa o MapLibre (o motor do Terra), com globo e imagem de satélite real embutida;
    - janelas soltas (Lugares, Camadas, Propriedades), que arrastam, recolhem e fecham;
    - ferramentas de polígono, linha, ponto e medir funcionando;
    - identidade visual do Gestão (azul-marinho e dourado, Poppins e Lato) no protótipo inteiro.
  - **Imagem do protótipo:** Sentinel-2 cloudless, da EOX (licença aberta, 10 m por ponto), porque dá para embutir na
    página. No programa continua a imagem de alta resolução (decisão de 13/11).
- **Retorno do JP (16/11): "não quero a do Sentinel; quero as imagens do Google, que é o que presta".**
  - **Feito (0.38.1):** a prévia do Agral Tools entrou no programa (cartão "Prévia do Agral Tools" na tela inicial,
    `tools_previa.html`, gerada no build). Dentro do programa, o mapa usa a imagem do Google ao vivo, pelo mesmo
    endereço XYZ do QGIS e do Terra.
  - **Limite:** a página publicada no link não consegue buscar imagem de fora, então lá continua o pacote embutido.
    Não copiei imagens do Google para dentro de uma página publicada.
- **Retorno do JP (16/11): "agora o trem está começando a querer"; gostou da aparência, "vai funcionar bem para o meu
  grupo de trabalho".**
  - **Camadas que ele usa no mapa:** BHO; drenagens em geral onde não há BHO (a BHO é de Goiás); municípios do
    Brasil; divisas de estado.
  - **Começar o trabalho pelo mapa:** achar o lugar, lançar a coordenada, procurar o CAR, juntar.
  - **Painel como o do ArtCraft Launcher** para abrir as ferramentas.
  - **Opção "iniciar um projeto de barragem".**
  - **Pedido:** dizer o que já existe (há algum QGIS pronto?), se vai dar para ser a engine de projeto e propor o
    plano de construção. **Feito:** `docs/specs/agral-tools-interface/plano-de-construcao.md`, para ele corrigir.
- **Decisões e pedidos do JP (16/11, noite):**
  - **"A casca e os trabalhos, você entendeu bem, é desse estilo que eu quero."**
  - **QGIS nosso não é prioridade:** o Mapa (o Earth nosso) cobre o uso; fica para depois.
  - **CAD: "vou precisar de um CAD nosso mesmo".** Base num CAD aberto e simples, como o LibreCAD, levado para o
    estilo do Civil 3D e mais leve. Não é agora.
  - **Trabalhos:** buscar por cliente e por fazenda; lista ou quadrinhos, como um gerenciador de arquivos.
  - **Gerenciador de arquivos próprio,** que rode também no celular: ideia para depois, fora do Tools.
  - **O painel de entrada é a primeira tela,** com o que é pertinente.
  - **Ambiental é uma área separada de Projetos.** O ambiental indexa os projetos no processo e faz o preenchimento
    do processo e dos documentos que vão atrelados.
  - **Pedido:** investigar tudo o que ele descreveu (Gestão, Project Engine, Tools), dizer como a estrutura deve
    funcionar e preencher as lacunas da ideia. **Feito:** `docs/specs/agral-tools-interface/conceito.md`, para ele
    corrigir; o protótipo ganhou o painel de entrada, a busca e os quadrinhos em Trabalhos, e a área Ambiental com
    o dossiê do processo.
- **Rede e trabalho sem conexão (definido pelo JP em 16/11):**
  - **No escritório,** as máquinas ficam no cabo com o servidor: o programa vai direto pela rede local, sem passar
    pelo Tailscale (menos uma etapa, menos travamento, melhor resposta).
  - **Fora do escritório,** vai pelo Tailscale.
  - **Sem internet,** opera offline, guardando no próprio computador, e faz a cópia assim que conectar.
  - Vale também para o gerenciador de arquivos.
  - **Ambiental (confirmado pelo JP):** o propósito é acompanhar o processo e montar as partes que o compõem. Ele
    entende o que o processo pede (por exemplo, uma outorga) e quais documentos têm de ser produzidos, e tira esses
    documentos do projeto feito na área de Projetos. **É preciso ajustar o que se espera da entrega do projeto ao que
    o processo pede.**
  - **Cópia de segurança do Agral Server:** existe, pelo serviço contratado da Core. A conferir na instalação: a
    pasta de dados do Tools precisa estar entre as que a Core copia.
- **Pedido do JP (16/11, noite):** repensar e organizar a estrutura de tudo o que foi dito, mostrar o resultado
  final, para ele detalhar depois, e em seguida montar o caminho da construção. **Feito:**
  `docs/specs/agral-tools-interface/estrutura-final.html` (https://claude.ai/artifact/JDbzMAr6xkFfQbkermJ1h6), com
  o mapa do programa, as áreas, o trabalho por dentro, o Ambiental, a rede e o caminho em 7 etapas. Os outros
  documentos da pasta ficam como detalhe e histórico.
- **Respostas do JP às perguntas da revisão do desenho do Agral Tools:**
  - o Agral Server (Z:) é um computador com Windows ligado o tempo todo (provavelmente Windows Server), não um NAS.
    O servidor do Tools que a equipe compartilha roda nele;
  - o único cálculo restrito ao JP é o extravasor, e ele roda só no servidor.

### 15/11/2026 (Hydrus, com o Claude)
- **Decisões do JP sobre a estrutura (depois de avaliar a proposta do Project Engine de 10/11):**
  - **Motores num lugar só, no Hydrus.** As ideias mais recentes são as do Hydrus.
  - **Um programa só (Agral Tools), com o Gestão à parte.** O Project Engine vira uma parte do Tools: "o Studio
    bombado", aberto como uma ferramenta, só para gerar o memorial, no fim do trabalho. Ele junta os estudos já feitos:
    o estudo de uma barragem da fazenda, ou o estudo da fazenda inteira, com todas as peças que o compõem.
  - **O Gestão é o dono do cliente**, e tudo fica ligado a ele.
  - **Um login só, o do Gestão**, para os três programas.
  - **Ferramenta solta ou dentro do trabalho:** dá para abrir o Earth nosso (Terra) direto, sem nada, e fazer algo
    sem ligação com nenhum trabalho. O programa então pergunta se quer criar ou ligar aquilo a um trabalho.
  - **CAD:** programa próprio, CAD AI ou plugin no Civil 3D fica para outra conversa.
  - **Pedido:** estudar e propor a estrutura da interface do programa inteiro (como se chega às ferramentas, aos
    estudos e ao memorial), seguindo a lógica já definida.
- **Estrutura da interface, respostas do JP (15/11):**
  - **A entrada é o trabalho; o globo é só uma ferramenta.**
  - **Abas, como no Gestão:**
    - **Home:** resumo, avisos do que interessa a quem entrou e os trabalhos em que a pessoa está.
    - **Demandas:** quadro por etapa, como o departamento de projeto. Um trabalho pode esperar outro serviço, como
      sondagem ou ensaio de solo. Num trabalho concluído dá para abrir uma pendência, e a IA, que conhece o contexto,
      ajuda a resolver.
    - **Trabalhos:** Cliente > Fazenda > Trabalho, como no servidor e no Gestão.
  - **Confirmado pelo JP ("todas as correções estão certas"):**
    1. O trabalho do Tools é o mesmo card do Gestão (mesmo código e mesma etapa).
    2. As colunas de Demandas são as 6 de Projetos do Gestão. "Aguardando dados" ganha o motivo: sondagem,
       laboratório, cliente ou órgão.
    3. A pendência num trabalho concluído vira uma demanda nova, ligada a ele, na coluna Ajustes.
    4. A fazenda guarda o que é do lugar (perímetro, camadas, terreno). O "estudo da fazenda" vira um tipo de trabalho.
  - **Servidor Z: hoje (prints do JP):**
    - na raiz, uma pasta por departamento (01.CAMPO, 02.COMERCIAL, 05.PROJETOS, 09.AMBIENTAL e outras);
    - dentro de cada uma, uma pasta por cliente;
    - em Projetos, uma pasta por fazenda, separada por tipo de arquivo (DWG, EXCEL, JPEG, KML, PDF, SHP, TXT), mais a
      OS em Word;
    - não há pasta por trabalho.
  - **Decisão do JP sobre as pastas:**
    - **Não seguir o padrão de hoje** ("está bugadão, tem que começar a ordenar").
    - **O programa organiza sozinho, com uma pasta por trabalho dentro da fazenda,** criada quando o trabalho começa.
      Exemplo: Avaliação geral, Confinamento 01, Barragem 01, Barragem 02.
    - **Dentro da obra, uma pasta por dependência:** estrutural, extravasor, escadaria, sondagem, estabilidade e assim
      por diante. Dá para juntar o que for pequeno.
  - **Um card do Gestão para cada obra** (Barragem 01 e Barragem 02 são dois cards, cada um com a sua etapa). Logo, o
    trabalho do Tools = o card = uma obra ou estudo (Avaliação geral, Barragem 01, Confinamento 01).
  - **Árvore de pastas proposta e não corrigida pelo JP:**
    - `00 FAZENDA` (o que é do lugar), `AVALIACAO GERAL` e uma pasta por obra;
    - dentro da obra, da `01 HIDROLOGIA` até a `10 MEMORIAL`, mais `ENTREGAS` (R00, R01);
    - os cenários ficam no programa, com o cenário no nome do arquivo;
    - as pastas antigas não são mexidas sem confirmação.
  - **Estrutura da tela: opção A ("trabalho como espaço").** Abrir um trabalho mostra as dependências à esquerda, com
    o estado de cada uma, e a ferramenta no centro.
  - **Estudo da fazenda com obras dentro (pedido do JP):**
    - precisa de uma visão das obras, com as informações simples e as características de cada uma;
    - **estudo completo**, como o do Grupo Tomazini na Serra Branca, no Piauí: 2 canais, um reservatório, sistema de
      pivôs e custo;
    - **estudo preliminar**, sem dado de projeto, para saber se precisa de um reservatório de tantos m³: só volume,
      área, volume armazenado e volume do maciço.
  - **Correção do brief:** o estudo do Piauí citado em 13/11 como "Yara" é o do Grupo Tomazini (Serra Branca).
  - **Protótipo da estrutura:** https://claude.ai/artifact/631oGmieLJG89Jz91EDCcu
    (`docs/specs/agral-tools-interface/prototipo.html`).
  - **Ideia preliminar do JP: um "sistema inteligente" nos projetos complexos.**
    - Uma visão geral que mostra como tudo se liga: a irrigação com as barragens; a irrigação com o ambiental
      (sobreposição com reserva e árvores isoladas); a barragem com o estrutural e a fundação.
    - Mudar uma ponta refaz o resto. Exemplo: diminuir os pivôs deixa a barragem menor, economiza tanto e muda a
      fundação.
    - Objetivo: o melhor resultado pelo menor custo.
    - O JP aprovou o modelo do painel ("ficou bom o seu modelo").
    - **Ordenação das versões: "vai depender", mas em geral pelo menor custo por hectare irrigado.**
  - **Desenho da estrutura:** `docs/specs/agral-tools-interface/desenho.md`.
- **Pedido:** começar a v1.1 do Agral Terra pelas drenagens, com a hidrografia oficial como base e a rede pelo relevo só
  como sugestão ("pode ser, pode ser").
- **Feito (0.38.0):**
  - **Rede oficial em cascata:** IBGE 1:25.000, depois IBGE 1:100.000, depois a BHO 5K da ANA (grossa, com aviso).
  - **Nascentes:** saem do começo dos trechos.
  - **Relevo:** a drenagem pelo relevo virou sugestão, desligada, a 80 ha e sem APP.
  - **Correção pela imagem:** "não existe" num curso d'água ou numa nascente, "desenhar curso d'água" e "marcar
    nascente". Cada correção refaz a APP e a área útil, fica salva com a fazenda e aparece no relatório, com a nota de
    retificação do CAR.
  - Fazendas salvas na v1 são refeitas pela rede oficial ao abrir.
- **Fatos medidos:** no quadrado de Firminópolis (GO), o relevo a 20 ha dava 130 km de rio; a carta oficial tem
  77,5 km. Fora de GO, DF e de alguns estados, só existe a BHO 5K, que tem menos detalhe.
- **Definido:** o mapa de pivôs antigo continua com 20 ha (sem mudança silenciosa).
- **Achado:** os botões "Polígono" e "Medir" da barra do Terra não fazem nada. Fica para o item do desenho da v1.1.

### 14/11/2026 (Hydrus, com o Claude)
- **Pedido:** "o SICAR tá voltando vazio, ver o que está acontecendo"; ideia de ter a base offline, testar se houve
  alteração e baixar só a alteração, também para as bases hidrológicas.
  - **Feito (0.37.1):** a causa era a conexão. O SICAR só aceita TLS 1.2 com troca de chave RSA, que o Python recusava
    por padrão. Corrigido mantendo a conferência do certificado.
  - **Proposta:** bases locais atualizadas por diferença em `docs/specs/bases-offline/spec.md`, aguardando o "pode
    fazer".
  - **Decisão do JP:**
    - as bases locais só começam quando o servidor do escritório estiver ligado pelo Tailscale;
    - o armazenamento é no servidor local do escritório, nunca em nuvem paga, porque a base é grande.
- **Pedido (depois do primeiro uso do Agral Terra):**
  - "tem que dar para apagar a fazenda" (**feito**: confirmação, lixeira, sem cascata);
  - estruturar o programa inteiro como combinado e mandar o fluxograma de como o programa vai funcionar e de como
    fazemos a produção dele;
  - pedidos da v1.1 anotados em `docs/specs/agral-terra-v1/v1.1-pedidos.md`: polígono como no Earth, globo com os
    projetos, perguntas antes do encaixe, drenagens erradas.
- **Pedido e definido:**
  - entrevista do Agral Tools (brief em `docs/specs/plataforma-agral-tools/brief.md`);
  - login e gestão da equipe pelo Gestão na nuvem; motores e arquivos pesados no escritório;
  - primeiro "deu certo" = prospecção da fazenda na conversa com o cliente;
  - caminho B: mapa novo (Agral Terra) em MapLibre, do zero, no Hydrus;
  - tela que abre limpa (apresentação) e vira estúdio;
  - não estimar prazos em semanas;
  - método de usabilidade do Gestão (teste pesado).
- **Aprovado:** `docs/specs/agral-terra-v1/spec.md`, com "c pode fazer". Execução "direto" escolhida pelo JP.
- **Feito (0.37.0):** Agral Terra v1 com os 13 requisitos da spec, testes de ponta a ponta do critério de sucesso e teste pesado de usabilidade no padrão do Gestão (`docs/qa/hardtest-terra-2026-11-14.md`). Água no cartão: a regra do órgão e a conta
  rápida do escritório lado a lado.

### 13/11/2026 (Hydrus, com o Claude)
- **Pedido:** "está no momento de evoluir para um programa de verdade". Launcher com os nossos programas (como o ArtCraft Launcher do CADcraft); hierarquia cliente, fazenda, obra ou estudo; um "Earth nosso" (globo, vetorizar, medir, importar, camadas e subcamadas, objetos editáveis como o pivô); superfície por LandXML ou curvas; IA que monta mapa e imagem; relatórios ligados (Project Engine); CAD separado; Eberick, Slide e HEC-HMS/RAS no caminho de estudo da Agral, não generalistas; LiDAR sem curva sobreposta no volume; referências para o extravasor ("entradas e extravasores imensos"); listar as funcionalidades.
- **Feito (proposta, sem implementar):** `docs/specs/plataforma-agral-tools/PLANO.md`: diagnóstico do que limita hoje, estrutura (launcher, núcleo comum, hierarquia, Agral Terra, superfícies e LiDAR, relatórios, MCP), tecnologia recomendada (MapLibre, Cesium só para 3D pesado, Shapely, pyproj, pyogrio, rasterio, laspy, GeoPackage), lista de funcionalidades, referências do extravasor, fases e as decisões que faltam. Anotado em "Pedidos sem lugar" do mapa.

### 12/11/2026 (Hydrus, com o Claude)
- **Pedido (antes de uma reunião, prazo de 1 hora):** ferramenta de avaliação de fazenda "dentro daquele mapa interativo, parecido com o que nós temos para pivô ... um Google Earth com esteroides": escolher uma ou mais matrículas/imóveis do cliente; mostrar área agricultável e reserva em ha e %, desmatamento, embargo, APA; separar reserva e árvore isolada; perguntar se quer gerar o layout de pivôs; mandar os pivôs para o dimensionamento e para o estudo hidrológico; achar o melhor lugar de barragem pelo custo; custo do pivô por hectare (adutora, relevo); exportar tudo em KML para ligar e desligar na reunião. A parte de "trocar reserva por mato" foi descartada por ele ("ignora essa parte").
- **Feito nesta hora (decisão do Claude, entrega mínima):** bloco "Avaliação da fazenda" na tela Mapa e pivôs (`app/fazenda.py`, rota `/api/fazenda/avaliar`): área, uso e cobertura (MapBiomas, e WorldCover quando o MapBiomas não responde), reserva legal de referência de 20%, relevo e declividade, PRODES, embargos do IBAMA (sem nome e CPF), UCs federais, camadas no mapa, KMZ com pastas e a pergunta do encaixe dos pivôs. Teste `test/fazenda.mjs`.
- **Pedido (mesmo dia):** volume de água para irrigação pela conta do escritório: soja, 3 meses críticos (jun a ago em Goiás), 15 dias por mês a 10 mm/dia = 4.500 m³/ha, como reservatório seco; depois descontar a contribuição pela vazão; achar o melhor lugar de barragem dentro da fazenda, no curso com a maior contribuição, perto da divisa; traçar as adutoras e prever o custo; usar o MDT mais detalhado, sem vegetação, e fechar a curva para calcular o volume, se ficar bom.
- **Feito (0.32.0):** bloco "Água para irrigação" (área irrigada, mm/dia, dias/mês e meses editáveis; Q95 específica opcional desconta a entrada e a barragem guarda só o que falta); até 3 pontos de barragem pelo relevo ANADEM (terreno sem vegetação), com curva cota-área-volume fechada pelo relevo, lâmina, área do lago, crista e aterro estimados (crista 6 m, 1V:2,5H e 1V:2H, borda livre 1,5 m); adutoras em linha reta × 1,2 até os pivôs da fazenda; custo só com os preços informados (o programa não inventa R$/m nem R$/m³).
- **Pedido (mesmo dia):** "gere várias opções ... calcular por módulo ... fazer os módulos dos pivôs e ligar nas respectivas barragens, mostrar os potenciais e montar por bloco". **Feito:** até 5 barragens em cursos diferentes; cada pivô da fazenda vai para a barragem mais perto que ainda tem água até a lâmina máxima (10 m por padrão, editável); tabela por bloco com área, volume, lâmina, aterro, adutora, custo e potencial em ha de cada barragem; total em R$/ha irrigado; pivôs sem água listados.
- **Pedido (mesmo dia):** "precisa ir para a estrutura do cliente". **Feito (0.33.0):** botão "Salvar no cliente" na avaliação: escolhe o cliente (ou cria pelo nome) e um projeto novo ou existente; grava a avaliação, as barragens e os blocos, o mapa e os pivôs da fazenda (que entram na lista de pivôs do estudo). A página do projeto ganha o atalho "Avaliação da fazenda", e `pivos.html?p=ID` reabre tudo. O servidor não deixa a tela do estudo apagar esses campos ao salvar.
- **Pedido (mesmo dia):** "precisa ter como eu lançar coordenada lá dentro pra ir pra fazenda; pesquisar cidade, como no Google Earth; já bota a ferramenta pra rodar sem ser protótipo". **Feito (0.34.0):** caixa de busca no alto do mapa: coordenada em graus, minutos e segundos, decimal ou UTM, ou lugar (cidade, distrito, fazenda) pelo OpenStreetMap, só no Brasil e só o texto digitado; o marcador oferece "Puxar o CAR aqui". A tela deixou de ser protótipo: começa vazia em Goiás, sem os pivôs de exemplo, e o cartão do início virou "Mapa da fazenda e pivôs".
- **Pedido (mesmo dia):** "os pivôs estão entrando em APP, reserva e estruturas". **Feito (0.35.0):** a avaliação cria a camada "Restrições da avaliação" com papel de restrição: vegetação nativa (≥ 0,5 ha), água e construções pelo uso e cobertura; cursos d'água pelo relevo ANADEM (a partir de 20 ha de contribuição) como linhas, com a faixa de APP do encaixe (30 m por padrão); nascentes com 50 m. Fragmentos menores que 0,5 ha não restringem: são listados como possíveis árvores isoladas para o relatório ambiental. Reserva legal averbada e APP oficiais só com o shapefile do CAR.
- **Pedido (mesmo dia):** "tem que ir pra dentro do projeto do cliente, pra ficar salvo; os pivôs têm que ir no KML também; os mapas que exporta, as áreas de APP, tudo venham em polígonos; e não está vindo o relevo". **Feito (0.36.0):** a página de todo projeto tem o cartão "Mapa da fazenda e avaliação"; dentro do projeto o mapa salva sozinho (2,5 s depois de cada mudança); projeto sem mapa abre vazio. O KMZ é montado na hora com o que está no mapa e vem todo em vetor: perímetro, uso do solo por classe, declividade por classe, curvas de nível (ANADEM, suavizadas, equidistância automática), vegetação nativa, água, construções, nascentes, APP de curso d'água de 30 m em polígono, PRODES, embargos, UCs, pivôs (inclusive setoriais), barragens, lagos e adutoras dos blocos. O relevo aparece no mapa (curvas de nível ou declividade).
- **Fica para depois (spec a escrever):** árvore isolada e reserva do CAR (só pelo shapefile do imóvel), UCs estaduais e municipais, melhor lugar de barragem por custo, custo do pivô por hectare, matrícula do SIGEF/INCRA.

### 10/11/2026 (Hydrus, com o Claude)
- **Pedido:** "Ficou um caralho esse módulo do Geotech ... temos que repensar ele todo"; "a forma de se calcular é burra, não condiz com a realidade nem com o padrão que quero atingir, e nem com as etapas de projeto"; "tudo que o Codex fez ficou ruim e fora do nosso escopo ... até o negócio do pivô que o Codex fez ficou péssimo; o que você estava fazendo ... está 1000 vezes melhor"; "estude o curso ... siga o mesmo padrão de qualidade e eficiência das outras ferramentas ... refaça".
- **Definido:**
  - o Claude refaz o Geotech do zero, estudando o curso do Thiago Bretas (Modelagem Numérica de Obras Geotécnicas, Estabilidade de Taludes) e as entregas reais do Drive;
  - fórmulas e painéis próprios do Hydrus: o João Pedro usou o CadCraft para ver as lógicas, mas "vamos criar as nossas próprias fórmulas e painéis";
  - a oficina de pivô do Codex sai do programa, e o pivô segue a partir do mapa, do encaixe e da adutora do Claude;
  - o Codex sai do Hydrus por enquanto.
- **Feito (decisão do Claude, dentro do pedido):** módulo novo na aba Geotecnia com sete passos na ordem do projeto; equilíbrio-limite pelos cinco métodos do curso; percolação por elementos finitos com piping; critérios de filtro de Terzaghi; critério USACE por padrão, com os valores da Eletrobras marcados "a conferir"; nenhum parâmetro de material é preenchido sozinho, só pelo exemplo do curso, marcado com a origem. Os arquivos do Codex ficam sem link até o João Pedro confirmar a exclusão.
- **Pedido:** "Pode juntar tudo" (PR #10) e "Pode" (instalador 0.29.0). **Feito:** PR #10 na `main`; instalador 0.29.0 entregue; PR #15 com a troca de versão.

### 09/11/2026 (Hydrus, com o Claude)
- **Pedido:** "Preciso desenvolver um método porque estou trabalhando com vocês dois em paralelo ... eu quero que você seja o líder e ele responda a você". **Definido:**
  - o Claude coordena, revisa e integra no Hydrus; o Codex responde ao Claude; o João Pedro decide;
  - o canal entre os dois é o GitHub: relatório do Codex em `docs/agentes/RELATORIO_CODEX.md`, instruções do Claude em `docs/agentes/PARA_O_CODEX.md` e nos comentários do PR;
  - quem mexe em quê fica em `docs/EM_ANDAMENTO.md`: Claude com Extravasor, bacia e mapa de pivôs; Codex com as oficinas (Geotech, irrigação em Python, materiais, culturas, modelo Slide).
- **Feito:** PR #8 do Codex revisado e juntado no ramo da 0.28.0. Regressão 21/21. Falha conhecida do Codex, que já existia no ramo dele: `test_engenharia_tela`.
- **Pedido:** "Sim e sim". **Definido:**
  - abrir o PR do ramo `ccr-6c5bdbb5-8bml93` (0.28.0 com as oficinas) para a `main`; juntar só com "pode juntar";
  - "pode fazer" a revisão do Extravasor da spec `docs/specs/extravasor-fluxo-dimensionamento.md`. O Hydrus segue em paralelo ao Gestão por decisão do João Pedro, mesmo com a ordem do mapa.
- **Pedido (09/11):** "Eu prefiro que seja galeria fechada, não sendo trapezoidal. Mas que ele consiga fazer as duas opções". **Definido:** no dimensionamento automático do Extravasor, galeria fechada retangular é o padrão e o canal aberto retangular também é oferecido; trapezoidal fica fora das opções automáticas.
- **Pedido (09/11):** "não tenho foto, pode considerar uns 30cm de largura e a altura depende". **Definido:** soleira vertente em viga com 30 cm de espessura como padrão (editável); altura sobre o fundo informada por obra, sem padrão.
- **Pedido (10/11):** "Pode juntar tudo". **Feito:** PR #10 (Hydrus 0.28.0, oficinas do Codex, revisão do Extravasor com os três caminhos) juntado na `main`. A partir daqui os dois agentes partem da `main`.
- **Pendências dele:**
  - prints do SisCCoH (parede no caso sem ressalto, limite do regime);
  - apagar a chave do Gemini usada em 06/11.

### 09/11/2026 (Hydrus, repasse do Codex no PR #8)
Decisões que o João Pedro passou ao Codex e que ele consolidou no repasse; o Claude conferiu e juntou aqui:
- João Pedro decide; Claude coordena, revisa e organiza a integração. Codex segue a resposta do Claude antes de nova tarefa.
- Trabalhar a partir do ramo indicado pelo Claude. Nesta entrega, `ccr-6c5bdbb5-8bml93` (0.28.0). Nunca juntar na main por iniciativa do Codex.
- Antes de editar módulo, ler `docs/EM_ANDAMENTO.md`; se atribuído a outro agente, registrar a necessidade no relatório e não editar. Ausência do arquivo deve ser registrada.
- Um assunto por ramo e próximos PRs curtos. Relatório, decisões e contexto atualizados ao final; commit, push e PR autorizados para o trabalho solicitado.
- Nenhuma mudança silenciosa de cálculo; regressão 21/21 ou diferença explicada no PR. Separar implementado, testado e validado em uso real.
- Hydrus é o laboratório de motores e ferramentas que convergirá ao Agral Tools com Agral Project Engine e demandas do departamento de projetos do Agral Gestão. Gestão administrativo continua na nuvem.
- Agral Geotech permanece interno ao Hydrus por enquanto e será engine geotécnica do Tools.
- Prioridade desktop, simplicidade de operação e interface bonita. Mobile será descontinuado; não gerar APK sem novo pedido.
- Fluxo guiado e modo livre devem compartilhar dados/motores. Estudar as entregas reais do Drive antes de fechar as saídas predefinidas.
- João autorizou implementar incrementalmente as ferramentas, inclusive irrigação e estabilidade, usando SDD/BMAD e o método documentado no Agral Gestão. Isso não autoriza atropelar a coordenação posterior do Claude.
- Não copiar solvers defeituosos do ZIP como motores validados. Importação de modelos/resultados externos não constitui validação de cálculo próprio.
- Português do Brasil, sem travessão longo; nenhum segredo em arquivo, log ou chat.

### 09/11/2026
- **Pedido (10/11):** "já gera outra versão do apk". **Situação:** o APK é uma moldura que abre o programa do servidor, então o visual novo, o celular e a tela de carregamento chegam ao APK 0.3 instalado sem gerar outro. A chave que assina o APK de teste ficou no ambiente do outro agente (`/workspace/shared/agral-android-signing`) e não existe nas sessões da nuvem: um APK assinado com outra chave não instala por cima do atual (teria de desinstalar). **Definido:** não gerar APK agora (recomendação; o JP respondeu "pode" para juntar a tela de carregamento e não pediu o APK de novo). APK novo só para mudança do próprio aplicativo e com a chave original recuperada.
- **Pedido (com print do celular mostrando o programa "desmontado" enquanto abre):** "vamos adicionar uma tela de carregamento toda vez que vai iniciar, fica essa tela aí até carregar tudo". **Feito (na branch):** tela de abertura azul-marinho com a logo e um indicador girando, que aparece antes do programa e sai quando a primeira tela está desenhada (`patch51.py`; quem tira é o `ui50.js`). Se passar de 12 s, mostra "Está demorando" com o botão "Tentar de novo"; nunca fica presa para sempre (sai sozinha 8 s depois de tudo carregado).
- **Pedido:** "Precisa atualizar a versão mobile e não ser apenas uma transição do web pro mobile... a página não está dimensionada para o mobile e os tamanhos dos botões não estão certos". **Feito (na branch):** camada própria do celular: toque de 40 a 46 px em tudo, letra base de 16 px, caixas de marcar ao lado do texto, tabelas pequenas quebrando linha, listas em cartões, janelas com botões largos. **Regra nova:** tela só é dada como pronta depois de conferida como celular com o banco de teste CHEIO (`tests/demo_dados.py` e `tests/t_celular_medidas.js`), não com banco vazio.
- **Diagnóstico (a pedido):** a atualização automática do servidor parou porque uma rodada ficou travada e o serviço não tinha limite de tempo (`systemctl status agral-auto.timer` mostra "active (running)" e "Trigger: n/a"); enquanto ela não termina, o timer não dispara. **Correção (na branch, aguarda "pode juntar"):** limite de 30 min no serviço, timer pelo relógio (a cada 5 min), limite de 2 min no `git fetch` e instalações sem pergunta. No servidor: `sudo systemctl kill agral-auto.service` e `sudo agral-atualizar`.
- **Pedido:** "manda a fase 3 já, pode fazer logo tudo". **Feito (na branch):** fase 3 do rework 050: botão do próximo passo nas listas vazias; no celular, telas longas em seções que abrem e fecham e tabelas largas em cartões; Comercial em 2 grupos e Admin. em 3. Só interface; a base de dados não é tocada.
- **Pedido:** "pode juntar" a fase 2 do 050; o faturamento e as fazendas de exemplo do protótipo "são reais, pode usar os reais"; "não quero que mexa na minha base de dados, já comecei a usar lá". **Definido:** os exemplos reais restantes do protótipo (`FAT`, fazendas) ficam como estão. **Regra reforçada:** a base de dados em uso no servidor não é tocada: nada de limpar, reimportar, trocar nomes ou migrar que altere dado existente; migração só acrescenta. O rework 050 é só interface (conferido: o que está gravado no servidor substitui por inteiro os exemplos do protótipo, `patch5.py`).
- **Pedido:** "Pode fazer e sim" (fase 2 do rework 050 e juntar a fase 1). **Feito:** fase 1 na `main` (PR #44). Fase 2 na branch: menu da pessoa no cabeçalho; Financeiro em 5 grupos (Visão, Lançamentos, Pagar e receber, Bancos e cartões, Impostos); listas vazias com ícone; filtros recolhidos no celular; Início com "Precisa de atenção"; tabelas largas rolam na própria caixa; 15 processos de exemplo do protótipo sem nomes reais. **Definido:** continua só interface (nenhuma regra muda); fase 3 (botão do próximo passo nas listas vazias, telas longas, grupos em Comercial/Admin.) aguarda "pode fazer".
- **Pedido:** verificação completa e rework visual do programa inteiro ("LINDO e profissional"), com as skills de UI/UX. **Feito:** auditoria das 54 telas (computador e celular) e fase 1, a camada visual global (`tema50.css` por `patch49.py`): campos, botões, números, abas, tabelas, cartões e janelas no mesmo padrão. **Definido:** identidade mantida (azul-marinho, dourado, Poppins); fase 2 (cabeçalho, estados vazios, filtros no celular, Financeiro em grupos, Início, telas longas) aguarda "pode fazer". Spec: `docs/specs/050-rework-visual/spec.md`.
- **Pedido:** "organizar a instrução para a Core para a instalação do Tailscale, para fazermos a ligação". **Feito:** `docs/specs/002-nuvem-tailscale/instrucao-core.md` (para a Core: requisitos, portas só de saída, instalação com chave de uso único e etiqueta `tag:escritorio`, teste, o que não pedimos, perguntas, como desfazer) e `ponte-passos-jp.md` (o que o JP faz no painel: etiqueta, chave, envio seguro). **Definido:** duas etapas: agora só o Tailscale e o teste de ligação; o Agral Bridge (porta 8790, pastas combinadas) depois. As regras que fecham o acesso entram junto com o Bridge. A chave nunca passa pelo chat.
- **Pedido:** "pode fazer" para o 049 (Chromium no servidor e leitor do Veredas primeiro). **Feito:** leitor do Veredas (`pkg/app/portal_go.py`): lê processos e caixa de entrada de cada conta do Ipê/Veredas, uma vez por dia e pelo botão "Ler agora" na aba Portal GO do Monitor; situação nova vira movimentação e aviso. Só leitura, com lista fechada de cliques. Playwright 1.63.0 + Chromium instalados pela atualização do servidor. Testado no portal real com a conta da Agral.
- **Pedido:** reconhecimento geral do Portal GO, ver se o Ipê é bloqueio e achar jeito de interagir completamente; depois, ferramentas para tocar os projetos sozinho. **Achado:** não é bloqueio, o Ipê está fora do ar do lado da SEMAD (502 com a página oficial de indisponibilidade); com navegador automatizado (sem disfarce) a conta da Agral entra em todos os outros sistemas, inclusive Veredas (processos, caixa de entrada, empreendimentos). **Definido:** o robô só lê; ações que mudam algo no órgão (arquivar, assinar, enviar, gerar DARE) exigem aprovação de uma pessoa. Detalhes e proposta em `docs/specs/049-portal-go-reconhecimento/brief.md`.
- **Pedido:** "você já tem vários logins aí, eu te autorizo utilizar algum para testar" (Portal GO). **Feito:** um único teste com a conta da Agral (responsável técnico) no Portal Ambiental da SEMAD/GO: **login aceito** (o portal abriu a página inicial com os 18 sistemas). Entrar no Ipê (`/ipe/sso`) não respondeu desta máquina da nuvem, nem sem login, enquanto o portal responde normalmente; não insisti nem disfarcei o acesso. Próximo: repetir pelo Monitor no servidor da Agral (depois que ele atualizar) e, se o Ipê abrir de lá, fazer a 045 R2. Senhas não foram gravadas, registradas nem commitadas; contas de clientes não foram usadas.
- **Pedido:** "define essas skills e métodos como padrão meu". **Definido:** o Pacote Agral (skills + `agral-metodo`) é o padrão do JP em todos os projetos, não só nestes três. Instalação global por `pacote/instalar.sh --global` / `instalar.ps1 -Global` (skills em `~/.claude/skills` e bloco em `~/.claude/CLAUDE.md`); na nuvem, o script do ambiente "Padrão" faz isso em toda sessão nova. Revisão de segurança aprovada e "pode juntar" dado para as specs 045 a 048.
- **Pedido:** "se eu baixar, você consegue usar as features dele por aqui?" e "pode". **Feito:** Spec Kit e GStack instalados na sessão; script de preparação do ambiente "Padrão" (aprovado pelo JP) instala Spec Kit, Serena e GStack em toda sessão nova. Strix e Taskmaster só quando houver chave de IA cadastrada pela tela do ambiente. O que se instala no computador do JP vale quando ele usa o Claude Code no próprio computador.
- **Pedido:** "você está usando as skills que te passei?" e depois: listar e instalar as skills do vídeo do Instagram (@guilhermemorais.ia) e criar um pacote com elas e o nosso método. **Feito:** lista do vídeo lida pela página pública de incorporação do post (o agente não contorna login nem baixa vídeo): Ponytail, UI UX Pro Max, Spec Kit, Taskmaster, Serena, Understand Anything, Superpowers, GStack, Trail of Bits, Strix. Repositórios oficiais baixados e lidos antes; as que são skill em texto (Superpowers, Ponytail, UI UX Pro Max, Trail of Bits selecionadas) entraram no pacote com a skill `agral-metodo`; as que são ferramenta (GStack, Spec Kit, Taskmaster, Serena, Understand Anything, Strix) ficaram documentadas com o comando de instalação no computador do JP, porque precisam de Bun, Docker, uv ou chave de IA. Graft e find-skills seguem como antes (não estão neste ambiente).
- **"Pode fazer" das specs 045, 046 e 047; respostas: cobrança pelo BB; ART normalmente no nome do mesmo responsável técnico.** **Feito:** impostos da Prisma e Evoar (mês sem receita conta zero) e conta passo a passo; salários com histórico, INSS progressivo (tabela com vigência; cadastrar a de 2026 com o contador), FGTS, tempo de casa e média de 12 meses; colunas Em espera e Arquivo, perdidas fora do fluxo, coordenada no lead e formulário de cadastro para nota fiscal com LGPD; aba Ambiental na ficha do cliente com classificação por regras. **Decisões do agente:** salário informado no programa (importar da folha depois); "rendimento" reservado até o JP definir; leitura dos processos no portal só depois de um login real ok; classificação sem IA nesta versão. **Pergunta do JP:** "o Pluggy consegue puxar extrato?" Sim: saldo e transações das contas do BB (spec 034), sem baixar sozinho; falta configurar a conta e o consentimento.
- **Pedidos da tarde (14:46 a 15:14):** aba Ambiental no card do cliente com processos, protocolos, estados e últimas mensagens; o programa entender o tipo de processo e o sistema, e ler no portal da SEMAD/GO (entrada única) os processos abertos no nome do cliente (spec 045); ver por que os impostos não funcionam para Prisma e Evoar (**causa: meses sem receita contados como "faltando" no RBT12**), tela de impostos detalhada, cálculo de FGTS e INSS dos funcionários, tempo de casa, salário médio e futuramente o rendimento de cada pessoa (spec 046); coluna Em espera e Arquivo de propostas, perdidas fora do fluxo do CRM, lead novo abrindo na aba Geral com local, coordenada e link de cadastro, formulário bonito com destaque da LGPD pedindo o cadastro no nome de quem recebe a nota (spec 047); estudar meios de pagamento (cartão por link, boleto, Pix, TED), emissão de boleto e pedido/autorização de ART no Agral Tools (estudo 048). **Definido:** specs em rascunho, aguardando "pode fazer" e as respostas de cada uma.
- **Pedido:** monitor ambiental "não está dando muito certo / não funcionou até agora" (prints: login não confirmado, portal exige login). **Feito:** conferido nos sites com CPF falso; corrigidos o campo de tipo de documento e o redirecionamento http do SIRIEMA, a forma de saber se o login deu certo na SEMAD e o formato do CPF; aviso claro quando falta o endereço da consulta do processo (spec 036, seção Correção).
- **Pedido:** o JP mandou o arquivo de senhas ambientais pelo chat. **Definido:** o agente não importa nem guarda senhas vindas do chat (regra de 06/11); a tela do cofre passou a aceitar esse `.json` para o JP importar direto no servidor.
- **Feito ("pode fazer o resto tudo"):** contas fixas confirmadas pelo extrato, só sugere a baixa (spec 040); relatório mensal do dia 5, automático e sob pedido (spec 042); comercial com lead já com cadastro, em espera, arquivo e fechadas arquivando na virada do mês, e comissão de 10% visível só para o JP (spec 043); pessoas por setor, quem bate ponto, aviso aos responsáveis (Administrador e Financeiro, decisão do agente), lembrete na hora de bater e índice diário com desativar quem saiu (spec 044). Ficou para o JP decidir: ATO; quem mais é "responsável" por setor.
- **Pedido:** "pode fazer o resto tudo, só pare quando finalizar". **Definido:** fazer, na ordem do mapa, o que faltava da lista de 09/11: comprovantes, contas fixas pelo extrato, relatório do dia 5, rateio de custo de campo, comercial (lead com cadastro, espera, arquivo, comissão) e pessoas (áreas, quem bate ponto, avisos, índice diário). Specs 039 em diante.
- **Pedido:** comprovantes ("ela pega o comprovante, vê que dia que pagou, que dia que saiu da conta"). **Definido:** anexar PDF ou foto ao lançamento, com o dia em que saiu do banco; anexar dá a baixa se ainda estava em aberto (pode desmarcar); filtro "sem comprovante" (spec 039).
- **Pedido (reunião com a Elaine):** rastrear o custo real por serviço e por cliente, inclusive quando um gasto serve a vários ("foi no João, no Igor e na Edilane, ele divide o custo"). **Definido:** ratear um lançamento entre vários trabalhos; o financeiro continua com um lançamento só; o custo por serviço usa as partes (spec 041).
- **Pedido:** "pode juntar". **Feito:** PRs #39 (Agral Gestão), #3 (Hydrus) e #4 (Project Engine) juntados.
- **Pedido:** botão de olho para esconder os valores. **Definido:** esconde na tela (R$ •••) e lembra no aparelho; não muda permissão no servidor (spec 038).
- **Pedido:** cadastro de cartões; fatura sem duplicar a planilha. **Definido:** cartão por empresa e por quem usa; a fatura reconhece o cartão pelo final; observação por compra; o que já existia (agrupado como a planilha, pessoal fora, trocar as linhas já lançadas) continua (spec 037).
- **Pedido:** monitor ambiental "só suporta um login por navegador"; criar sessões isoladas e testar uma por uma; reconhecer os sistemas e de onde são; vários sistemas de estados diferentes e vários logins para o mesmo cliente. **Definido:** cada conta entra numa sessão própria no servidor; lista de sistemas com órgão e estado; várias contas por cliente/sistema/estado, e contas da Agral; testar todas, uma por uma; tela por cliente. No navegador da pessoa o limite do site continua (usar janela anônima) (spec 036).
- **Pedido:** filtros nos lançamentos (valor, empresa, categoria) e, na Visão geral, abrir uma despesa e editar; da reunião: separar receitas e despesas, busca, sugestão de razão social, observação por item. **Definido:** spec 035.
- **Pedido:** "temos um guia do projeto? ... para onde ir e não ficar girando em círculos". **Definido:** mapa `docs/MAPA_AGRAL_TOOLS.md` nos três repositórios. Agral Tools = produto final, junção de todos.
- **Pedido (lista do JP, 09/11):** relatório mensal de notas em PDF; extrato, aplicação e diário saindo dia 5; índice diário dos funcionários com desativar quem saiu; programas separados assistidos por IA e Agral Tools com MCP; olho para esconder valores; divisão de pessoas em administrativo, campo e projeto; comissão de 10% visível só para o JP; lead já com cadastro do cliente; espera (standby) no comercial, arquivar propostas e fechadas arquivadas na virada do mês; marcar quem bate ponto, avisar os responsáveis se não bateu e aviso na hora de bater; controle de pagamentos com comprovante; cadastro de cartões; painel tipo Craft Anything; conversor de arquivos. **Definido:** os de ferramentas (Craft, conversor, MCP em todos) ficam para depois; o resto entra na ordem do mapa.
- **Reunião com a Elaine (09/11):** trabalhar o sistema até dezembro e, a partir de janeiro de 2027, parar de usar a planilha. Cartões agrupados como ela lança; gastos pessoais marcados; categorias iguais às dela; ATO tem lógica própria (o JP vai detalhar).

### 08/11/2026
- Lote operacional, filtros tipo CRM, Pluggy, MCP com ações, contratos cancelados, prévia de documentos e outros feitos pelo Codex: ver `docs/FILA_ATUAL.md` e specs 017 a 034.

### 07/11/2026
- Cadastro único de clientes (um só para Agral, Prisma e Evoar) e cofre de senhas do ambiental (só o JP vê as senhas, outros só se ele liberar). Telefone e e-mail do cliente: Campo e Projeto não veem; quem tem Ambiental vê. Clientes sem CPF/CNPJ entram marcados "conferir" (spec 014).
- Atualização automática do servidor pela `main` (spec 002 R11b), em vez de dar SSH ao agente.
- Espaço em disco da nuvem: proposta menor e arquivos gerados descendo para o escritório (anotado; depende da ponte).
- Perfis: Administrador, Financeiro (Elaine), Campo e Projeto, com liberações por pessoa e valores escondidos de quem não pode ver.

### 06/11/2026
- Restaurado o repositório; adotado SDD + BMAD; nuvem com Tailscale (spec 002); ponto do RHiD (spec 001).
