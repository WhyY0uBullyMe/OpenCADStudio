# Agral CAD 001: como o pacote Agral Civil vai embutido no Open CAD Studio

Investigação de 17/11/2026 (sessão do Agral CAD). Código lido, sem mudar nada. Fontes clonadas fora do Hydrus, em
`/home/user/agral-cad-src/` (pasta da sessão, apagada quando ela acaba).

- Open CAD Studio: commit testado `21e5d29`. A ponta do repositório já estava em `cc71a61a0` no mesmo dia: o projeto
  muda todo dia.
- Open Infra Studio: commit `6795a90`.

## Fatos

1. **Dois jeitos de pôr uma aba no programa** (`src/modules/mod.rs`, `docs/plugin-architecture.md`):
   - **Módulo do núcleo** (`src/modules/<nome>/`, como Home e View): compilado dentro do executável.
   - **Pacote (plugin)**: biblioteca à parte (`.dll`) que roda **num processo separado** e conversa com o programa
     pelo contrato `crates/ocs_plugin_api` (versão do contrato: 24 no `21e5d29`; o Open Infra está na 23).
2. **Pacote de primeira mão já vem junto e ligado.** O programa procura pacotes na pasta `plugins/` ao lado do
   executável (`bundled_plugins_dir()` em `src/plugin/external.rs`). Os dessa pasta carregam sozinhos, sem o comando
   `PLUGINS`; só não carregam se o usuário desligar. Um pacote do usuário com o mesmo `id` passa por cima.
3. **Comandos sem prefixo funcionam.** Quando o programa não conhece um comando, ele pergunta a cada pacote ligado,
   um por um (`ocs_plugin_api/src/process/manager.rs`, `dispatch`). O `command_prefixes` do `plugin.toml` só aparece
   na tela de pacotes, não filtra nada. Então o pacote pode atender `CREATESURFACE` direto. Cuidado: um comando que
   o programa já tem é atendido por ele antes do pacote. Conferido: `CREATESURFACE`, `ADDSURFACECONTOURS`,
   `ADDSURFACEPOINTS`, `REBUILDSURFACE`, `IMPORTLANDXML`, `EXPORTLANDXML` e `EDITDRAWINGSETTINGS` não existem no
   Open CAD Studio (busca no código e em `docs/commands-reference.md`).
4. **Atalhos como no AutoCAD.** O programa tem tabela de atalhos (`assets/ocad.pgp`, comando `ALIASEDIT`) e importa
   arquivo `.pgp`. Podemos pôr os atalhos de civil na tabela de fábrica.
5. **Por que o plugin holandês trava em 120 mil triângulos** (`src/civiel/terrein.rs`): ele desenha cada triângulo
   como uma entidade `3DFACE` separada. Com 234 mil triângulos o desenho fica pesado demais, e o autor pôs um teto.
   Tirar só o teto não resolve: a superfície precisa ser desenhada como um objeto só (malha), não como milhares de
   entidades.
6. **Dados no DWG.** O contrato do pacote grava os dados do civil nas entidades (XDATA) e permite declarar classes
   próprias de objeto (`plugin_classes.rs`, até 256 por pacote). Isso atende R5 (reabrir igual).

## Proposta para a 001

| Parte | Onde fica | Por quê |
|---|---|---|
| Motor (TIN, curvas, leitura de pontos, LAS, DEM, LandXML) | Biblioteca Rust própria, sem tela (`agral-civil-motor`) | Testável sozinha com `cargo test`, como o Hydrus faz com os cálculos; pode servir o Hydrus depois |
| Comandos, árvore (Toolspace) e aba | Pacote de primeira mão `agral.civil` na pasta `plugins/` do instalador | Já vem ligado (R3) sem mexer no núcleo do Open CAD Studio, que muda todo dia; atualizar a base fica barato |
| Desenho da superfície | Uma malha por superfície, não um `3DFACE` por triângulo | Necessário para R14 (1 milhão de pontos) |

Para o JP isso é "embutido": vem no instalador, já ligado, com a aba própria, e ele não instala nada. Por dentro
ainda é um pacote, só que nosso e de fábrica.

**Plano B, se o processo separado ficar lento com superfícies grandes:** passar os comandos para um módulo do núcleo
(`src/modules/agral_civil/`). O motor não muda; só a ligação com a tela.

## Ainda não verificado

- Compilação do Open CAD Studio inteiro nesta sessão (em andamento, ver `build-ocs`).
- Se a malha (`Mesh` ou `PolyFaceMesh`) com 500 mil triângulos fica navegável na tela. Precisa de teste com o arquivo
  do JP.
- Velocidade da troca de dados entre o programa e o pacote com nuvens de pontos grandes.
