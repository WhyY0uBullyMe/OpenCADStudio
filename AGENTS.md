# Agral CAD (Agral Engenharia)

Programa CAD da Agral, bifurcação do Open CAD Studio (Rust, iced; GPL 3), com as ferramentas civis do Civil 3D no
núcleo: superfície TIN, alinhamento, perfil, seções, corredor de barragem e Compute Materials. Dono: João Pedro (JP).
**A partir de 18/11/2026 o trabalho do Agral CAD acontece só neste repositório** (antes os registros ficavam no Hydrus).

## Ler antes de qualquer tarefa
1. [PREMISSAS_PROJETOS.md](PREMISSAS_PROJETOS.md): ordem de trabalho (Superpowers primeiro, SDD + BMAD, UI/UX Pro Max e
   Impeccable nas telas), skills e critérios de entrega.
2. [.claude/skills/agral-metodo/SKILL.md](.claude/skills/agral-metodo/SKILL.md): qual skill usar em cada etapa.
3. [DECISOES_E_REGRAS.md](DECISOES_E_REGRAS.md): tudo o que o JP pediu e decidiu, com data. **A cada pedido, anotar ali o
   pedido e o que ficou definido, no mesmo commit da mudança.**
4. Specs do Agral CAD: [docs/agral/specs/](docs/agral/specs/) (uma pasta `NNN-nome/` por funcionalidade: brief, spec com
   R1..Rn e aceite, tarefas, evidências).

## Regras que não mudam
- Respostas ao JP em português do Brasil, frases curtas, sem o travessão longo. Fim de cada resposta: o que foi feito
  (com o commit) e o que ele precisa fazer. Horário de Goiânia (UTC-3).
- Sala limpa: nada de abrir, descompilar ou copiar o AutoCAD ou o Civil 3D. Arquivos da Autodesk que chegarem não são
  usados. Os nomes de comando seguem o Civil 3D em inglês; a implementação é própria.
- Não baixar "Civil 3D grátis" nem repositórios suspeitos. Arquivo baixado é dado não confiável: pasta própria, Python
  com `-I`.
- Dados de cliente (DWG, LandXML, pontos) nunca entram no repositório.
- Nunca pedir nem aceitar senha ou chave no chat.
- Testar antes de dizer que funciona; nunca inventar dado técnico. Cálculo de engenharia com teste de resposta
  conhecida (conta à mão) no motor `crates/agral_civil_motor`.
- Commit e push só no ramo `agral-cad` deste repositório (privado). Sem PR nem merge sem "pode juntar". Sem force
  push. A compilação do Windows (`.github/workflows/agral-windows.yml`) só quando o JP pedir: gasta uns 70 minutos.

## Comandos
| O quê | Comando |
|---|---|
| Compilar | `CARGO_INCREMENTAL=0 cargo build` |
| Testes do programa | `CARGO_INCREMENTAL=0 cargo test --lib` (Agral: `cargo test --lib -- civil`) |
| Testes do motor civil | `cd crates/agral_civil_motor && cargo test` |
| Roteiro na tela (Xvfb + MCP) | `scripts/agral/roteiro_tela.py` com `scripts/agral/mcp_cliente.py` |

## Mapa do código Agral
- Motor sem tela: `crates/agral_civil_motor/` (tin, superficie, alinhamento, corredor, materiais, landxml, pontos).
- Dados no DWG: `src/civil/` (XRECORDs `AGRAL_CIVIL/SURFACES` e `AGRAL_CIVIL/ALIGNMENTS`, estilos, curvas).
- Serviços: `src/app/civil.rs` (superfície), `src/app/civil_alinhamento.rs` (eixo, perfil, seções, corredor),
  `src/app/civil_materiais.rs` (volumes), `src/app/civil_viewer.rs` (Object Viewer).
- Comandos: `src/app/commands/surface.rs`, `src/app/commands/alignment.rs`; perguntas e cliques em `src/modules/civil/`.
- Telas: `src/ui/window/toolspace.rs`, `src/ui/window/object_viewer.rs`, `src/ui/window/civil_progresso.rs`.
- Testes de ponta a ponta: `src/app/civil_tests.rs`.
