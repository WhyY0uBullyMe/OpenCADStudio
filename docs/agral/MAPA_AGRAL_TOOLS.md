# Mapa do Agral Tools

Guia fixo de para onde vamos. Atualizado em 09/11/2026.

**Original neste repositório** (`Agral-Gestao/docs/MAPA_AGRAL_TOOLS.md`). Hydrus e Project Engine têm uma cópia igual em `docs/MAPA_AGRAL_TOOLS.md`: ao mudar o mapa, atualizar as três.

Como usar: antes de começar algo novo, achar aqui onde ele entra. Pedido que não cabe em nenhuma linha é anotado em "Pedidos sem lugar" (fim do documento) e discutido antes de virar spec. O detalhe de cada item fica na spec ou no documento de estado de cada programa; o mapa só aponta.

Legenda: **pronto** (no ar e testado) · **parcial** (existe, falta parte) · **testar** (pronto no código, falta uso real) · **próximo** (combinado para a próxima rodada) · **depois** (ideia guardada, sem data).

---

## 1. O que é o Agral Tools

**Decisão do João Pedro (09/11/2026): o Agral Tools é o produto final, a junção de todos os programas da Agral num só.** Hoje são três programas separados, cada um com repositório próprio. Eles crescem separados e vão se juntar depois; a forma de juntar (um programa só, um portal com os três, ou programas que conversam) ainda não foi escolhida.

| Programa | Para quê | Quem usa | Onde roda | Repositório |
|---|---|---|---|---|
| **Agral Gestão** | A empresa: dinheiro, clientes, propostas, contratos, campo, ambiental, pessoas | JP, Elaine (financeiro), Sara, equipe de campo e projetos | Nuvem (VPS HostGator), navegador, celular e APK | `WhyY0uBullyMe/Agral-Gestao` |
| **Hydrus** | Os cálculos de engenharia hídrica: hidrologia, barragem, extravasor, captação, escada, descarga | Engenheiros e técnicos | Windows, um por computador, quase todo offline | `WhyY0uBullyMe/Hydrus` |
| **Project Engine** | Produzir o projeto e o memorial: editor do documento com IA, Civil 3D, pranchas, quantitativos | Equipe de projetos (prepara) e JP (calcula e aprova) | Windows (servidor de equipe ainda a definir) | `WhyY0uBullyMe/agral-project-engine` |

O fluxo de um serviço passa pelos três:

**Lead → proposta → contrato** (Gestão) → **campo e dados** (Gestão) → **cálculos** (Hydrus) → **projeto, memorial e pranchas** (Project Engine) → **entrega** → **cobrança e recebimento** (Gestão).

Regras que valem para os três:
- O motor calcula, a IA ajuda, o engenheiro (ou o financeiro) confere e decide. Valores e resultados técnicos nunca são inventados pela IA.
- Estudos e dados antigos continuam abrindo com os mesmos resultados.
- Segredos fora do Git; login e permissão conferidos no servidor.
- Método: Superpowers primeiro, SDD + BMAD (spec antes do código), testes antes de dizer pronto. Premissas em `PREMISSAS_PROJETOS.md` de cada repositório.

---

## 2. Agral Gestão

Estado detalhado: `docs/FILA_ATUAL.md` (o que está pronto e o que falta testar) e `docs/specs/README.md` (uma spec por funcionalidade).

**Meta principal: a Elaine para de usar a planilha em janeiro de 2027.** Combinado na reunião de 09/11: até dezembro o programa fica redondo usando os lançamentos de verdade; em janeiro a planilha sai. Tudo que atrapalha essa troca vem primeiro.

### Financeiro
| Área | Situação |
|---|---|
| Lançamentos, contas a pagar e receber, categorias, mês a mês, impostos automáticos (RBT12) | pronto |
| Importar a planilha (CSV/XLSX), extratos do BB (OFX/CSV), Pluggy (saldo e extrato) | pronto; Pluggy **testar** com a conta real |
| Recebimento na entrega (parcela só agenda quando o projeto é entregue) e acompanhamento de recebimentos | pronto |
| Cartões e faturas: cadastro de cartões, lançar a fatura, classificar, agrupar e ligar notas | pronto (spec 037) |
| Investimentos (spec 024) | depois |

### Comercial
| Área | Situação |
|---|---|
| Funil (lead, reunião, proposta, contrato), filtros tipo CRM e visões salvas | pronto |
| Proposta em PowerPoint com mapa, fotos com licença e conferência de layout; contrato com conversa, prévia e ditado | pronto |
| Repasse do custo da operação no preço; propostas por visita (spec 023) | pronto / parcial |
| Cancelar contrato sem recriar a previsão; ver proposta e contrato dentro do programa | pronto |

### Clientes
| Área | Situação |
|---|---|
| Cadastro único (Agral, Prisma, Evoar), ligações com cards e financeiro, juntar cadastros, cofre de senhas | pronto; **testar**: importar a base e o `SENHAS.xlsx` no servidor |
| Formulário por link para o cliente preencher o cadastro; botão do WhatsApp | pronto |
| WhatsApp com conversas e agente dentro do programa (spec 016) | depois (adiado por custo) |

### Campo, Projetos e Ambiental
| Área | Situação |
|---|---|
| Departamento de Campo: agenda, diário, km, notas, custos sem nota, fotos, indicadores | pronto |
| Robô ambiental: SEMAD/GO, Ipê, Veredas, SIRIEMA, Web Outorga, SEMA/MT | parcial: falta entrar com as contas reais (CAPTCHA e MFA precisam de alguém) |
| Monitor dos órgãos por cliente: sistemas com órgão e estado, várias contas por cliente, uma sessão por conta, testar acessos um por um (spec 036) | pronto; **testar** com as contas reais |
| ATO (acompanhamento técnico de obra): parcelas mensais e aba própria no Campo | próximo (o JP vai detalhar) |

### Pessoas
| Área | Situação |
|---|---|
| Ponto do RHiD (espelho e apuração), banco de horas, notificações de ponto | pronto; **testar** contra o espelho real |

### Plataforma
| Área | Situação |
|---|---|
| Nuvem com login, perfis e valores escondidos por pessoa; atualização automática; backup | pronto |
| App no celular (PWA) e APK Android; agente com voz para ditar | pronto; **testar** no aparelho |
| Conector MCP (consulta e ações liberadas por pessoa) | pronto; **testar** ligado ao Claude/ChatGPT |
| App para iPhone | depois |
| Ponte com o servidor do escritório (arquivos do Z:, descer arquivos gerados) | depois (adiado pelo JP) |

### Próximo: pedidos de 09/11 (ainda sem spec)
Da lista do JP e da reunião com a Elaine. A ordem dentro do grupo ainda vai ser combinada no brainstorming.

**Financeiro (o que segura a troca da planilha):**
1. ~~Fatura do cartão sem duplicar: cadastro de cartões por empresa e por pessoa que usa; fatura de cada cartão; classificar; agrupar como a Elaine; marcar pessoais.~~ **pronto** (spec 037 + o que já existia: agrupado, pessoal e troca das linhas da planilha); observação por compra também.
2. ~~Receitas e despesas separadas nos lançamentos, com filtros avançados (valor, empresa, categoria, situação, datas) e busca; editar pela Visão geral.~~ **pronto** (spec 035, 09/11).
3. ~~Razão social com sugestões enquanto digita e observação em cada lançamento.~~ **pronto** (spec 035); falta observação em cada item da fatura do cartão (vai junto do item 1).
4. ~~Controle de pagamentos com comprovante (data em que saiu do banco).~~ **pronto** (spec 039).
5. ~~Contas fixas e assinaturas previstas que o extrato confirma (sem baixar sozinho).~~ **pronto** (spec 040).
6. ~~Relatório mensal em PDF (notas, extrato, aplicação, diário) todo dia 5.~~ **pronto** (spec 042).
7. ~~Rateio de um custo entre vários serviços.~~ **pronto** (spec 041).

**Comercial:**
8. ~~Criar o lead já com a opção de cadastrar o cliente.~~ **pronto** (spec 043).
9. ~~Em espera, arquivar propostas, fechadas arquivam na virada do mês.~~ **pronto** (spec 043).
10. ~~Comissão de 10% do JP aparecendo só para ele.~~ **pronto** (spec 043).
11. ~~Botão de olho para esconder os valores na tela.~~ **pronto** (spec 038).

**Pessoas:**
12. ~~Divisão das pessoas em administrativo, campo e projeto.~~ **pronto** (spec 044).
13. ~~Quem bate ponto; aviso aos responsáveis; lembrete na hora de bater.~~ **pronto** (spec 044).
14. ~~Índice diário por funcionário, com desativar quem saiu.~~ **pronto** (spec 044).

---

## 3. Hydrus

Estado detalhado: `HYDRUS_CONTEXT.md` (seção 17 em diante), `PRODUCT.md`; plano do Hydrus 2.0 em `CLAUDE.md`. Versão lançada mais recente registrada: 0.25.2 (30/10).

| Área | Situação |
|---|---|
| Barragem: dados do ponto, culturas e safras, Kc, lâmina (mm/ha), balanço do reservatório, otimizadores de área e calendário | pronto |
| Captação direta (igual ao Veredas da SEMAD), outorgas a montante, vazões de permanência (Q10 a Q95), estações da ANA do Brasil inteiro | pronto |
| Extravasor (simplificado e completo): bacia, Tc, IDF, vazão, entrada, soleira, trânsito da cheia (Puls), escada de dissipação, concreto x PEAD, descarga de fundo | pronto |
| Biblioteca de reservatórios; cálculo rápido sem projeto; agente de IA | pronto |
| Studio e memorial no modelo da Agral (A3 paisagem) | parcial: vai para o Project Engine |
| Ponte para o CAD (pacote do Extravasor para o Agral CAD AI) | pronto |
| Bacia pelo relevo saindo torta em alguns pontos (escolher o MDE) | próximo |
| Hydrus 2.0: clientes e cenários, irrigação completa (pivôs, adutoras, bombas), geotecnia, dam break, biblioteca técnica | depois, um módulo por vez (fases em `CLAUDE.md` §17) |

---

## 4. Project Engine

Estado detalhado: `docs/produto/DEFINICAO_FUNCIONAL.md` (entrevista de 05/11), `docs/PLANO_INICIAL.md`, `docs/ESTADO_ATUAL.md`, `docs/STATUS_IMPLEMENTACAO.md`.

**Primeiro objetivo: o memorial de barragem com o agente e edição humana.**

| Marco | O quê | Situação |
|---|---|---|
| 0 | Base independente: motores do Hydrus, Studio, agente, Agral CAD AI, plugin do Civil 3D, geometria, quantitativos | pronto (05/11) |
| 1 | Editor visual com páginas e conversa ao lado; DOCX e PDF fiéis ao modelo da Agral; alterações da IA para aceitar ou recusar; voz | próximo |
| 2 | Memorial de barragem completo: acervo e versões, resultados ligados aos estudos, capítulos editáveis, pendências | depois do 1 |
| 3 | Equipe: servidor, usuários, permissões, uma pessoa editando por vez, comentários, pedido de cálculo ao JP e aprovação | depois |
| 4 | Civil 3D por conversa (com desfazer), seções e pranchas, sondagens (PDF + KML com cota do LiDAR/RTK), materiais e serviços com SINAPI | depois |

Agral CAD AI (dentro do Project Engine): plugin testado pelo JP no Civil 3D 2026 (desenhou a estrutura em 3D). Geometria preliminar, sem armadura (NBR 6118) e sem estacas dimensionadas (NBR 6122).

---

## 5. Agral Tools (a junção)

Tudo aqui é **depois**: só começa quando os três programas estiverem firmes. Não implementar por conta própria.

- **Como juntar** os três (um programa, um portal ou programas que conversam), com um login só.
- **Ligações entre os programas:** contrato fechado no Gestão abre o projeto no Hydrus e no Project Engine; custo real do projeto comparado com o orçado; renovação de outorga vira proposta.
- **IA em todos:** cada programa assistido pela IA e com conector MCP, para usar pelo Claude ou ChatGPT.
- **Painel de interação** no estilo Craft Anything.
- **Conversor de arquivos.**
- **Gerenciador de arquivos do escritório** pela ponte (navegar, ver, baixar e enviar para a pasta do cliente).
- **Acompanhamento de atividade** (programa aberto x trabalho de fato), só para o JP, o pai dele e gestores com permissão.
- **GIS** com operação manual e assistida pela IA; leitor de layout (KML/KMZ, polígonos, áreas) comum aos programas.

---

## 6. Ordem combinada

1. **Agral Gestão até dezembro:** pedidos de 09/11 (financeiro primeiro), mais os testes com uso real (Pluggy, cofre, ponto, celular, MCP). Em janeiro de 2027 a planilha sai.
2. **Project Engine, Marco 1:** editor do memorial com o modelo real.
3. **Hydrus:** correções (bacia pelo relevo) e módulos do 2.0, um por vez, puxados pelo que o Project Engine precisar.
4. **Agral Tools:** juntar, depois que os três estiverem firmes.

Os itens do grupo 4 da conversa de 09/11 (programas separados com MCP, painel tipo Craft Anything, conversor de arquivos) ficam para depois, por decisão do JP.

---

## Pedidos da tarde de 09/11
- ~~Ambiental no card do cliente (045)~~ **pronto**; leitura dos processos abertos no portal aguarda login real.
- ~~Impostos da Prisma e Evoar, tela detalhada, FGTS e INSS, tempo de casa e salário médio (046)~~ **pronto**; "rendimento" da pessoa a definir.
- ~~Comercial: colunas Em espera e Arquivo, perdidas fora, lead com coordenada, formulário com LGPD (047)~~ **pronto**.
- Estudo: cobrança pelo BB (Pix, boleto, cartão por link) e pedido de ART com autorização (048), parte do Agral Tools.

## Pedidos de 10/11 para o futuro (anotados, sem spec, nada implementado)

### Mapa de clientes e análise de fazendas
- **Aba "Mapa de clientes" dentro de Clientes** (Agral Gestão): as fazendas dos clientes no mapa.
- **Levantamento no servidor do escritório** (feito por outro chat, que já mexia em algo parecido): varrer o que temos de cada cliente. A maioria tem algum KML; pelas coordenadas dá para achar a localização e, com ela, o CAR da fazenda.
- **Matrículas:** organizar e avaliar em nome de quem está cada matrícula, inclusive as de fora. Um cliente às vezes tem várias matrículas dentro da mesma fazenda.
- **Destino:** adiantar a base para uma futura **ferramenta de análise de fazendas** dos programas da Agral.
- **Como o levantamento deve chegar para caber no cadastro** (sugestão do agente, a confirmar): uma linha por fazenda, com cliente (nome e CPF/CNPJ quando houver), nome da fazenda, município, UF, coordenada (latitude e longitude do centro), CAR, e a lista de matrículas com o titular de cada uma e o cartório; mais o caminho do arquivo de onde saiu cada dado e se foi conferido ou deduzido. O cadastro hoje guarda por fazenda: nome, município, UF, CAR e uma matrícula (`cli_fazenda`). Faltam no cadastro: coordenada da fazenda, várias matrículas por fazenda e o titular de cada matrícula.
- **Cuidados:** são dados reais de clientes: não vão para o Git nem para o chat; ficam no servidor. O que for deduzido (CAR achado pela coordenada, titular lido de documento) entra como "a conferir", não como fato.

### Um agente por área
O João Pedro quer um agente de IA para cada área:
- **Financeiro e fiscal.**
- **Trabalhista.**
- **Ambiental:** avalia processos completos, dá relatório de situação e de pendências, e sugere intervenções conforme a lei e as atualizações publicadas no Diário Oficial. Base de conhecimento começada em `docs/conhecimento/licenciamento-outorga/`.
- **Engenharia.**
- **Onde ficam:** financeiro/fiscal e trabalhista no Agral Gestão; **ambiental e engenharia provavelmente no Agral Tools / Project Engine**.
- Regras que já valem para qualquer agente: consulta sozinho; tudo que altera dado pede a confirmação de uma pessoa; valores financeiros são calculados pelo programa, nunca inventados; nos portais, lê e prepara, e quem assina, protocola, envia ou paga é o operador.

## Pedidos sem lugar

Anotar aqui o que não couber nas linhas acima, com data, até ser discutido.

- 13/11/2026: "evoluir para um programa de verdade": launcher com programas focados (estilo ArtCraft/CADcraft), hierarquia cliente, fazenda, estudo da fazenda e obra, um "Earth nosso" com camadas e objetos editáveis, superfícies e LiDAR, operador de IA, relatórios ligados. Login e gestão da equipe pelo Gestão (nuvem); motores e arquivos pesados no escritório. Entrevista e proposta em `Hydrus/docs/specs/plataforma-agral-tools/` (`brief.md`, `PLANO.md`). Mexe na seção 5 (Agral Tools) e se liga aos pedidos de 10/11 (mapa de clientes, matrículas, agente de engenharia).
