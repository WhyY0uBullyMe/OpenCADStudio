---
name: agral-metodo
description: Método de desenvolvimento da Agral Engenharia (Agral Gestão, Hydrus, Project Engine, futuro Agral Tools). Use no começo de QUALQUER tarefa nestes projetos, antes de planejar, codar, revisar ou responder ao João Pedro; diz a ordem de trabalho, as regras que não mudam, onde registrar decisões e qual skill do pacote usar em cada etapa.
---

# Método Agral

Programas da Agral Engenharia (dono: João Pedro). Textos para ele em português do Brasil, simples, sem o travessão longo.

## Antes de tudo
1. Ler `AGENTS.md` do repositório, `DECISOES_E_REGRAS.md` (o que o JP já pediu e decidiu), `docs/MAPA_AGRAL_TOOLS.md` (para onde vamos) e `docs/PROXIMOS_PASSOS.md` (onde paramos). Não perguntar de novo o que já está decidido lá.
2. Achar no mapa onde o pedido entra. Se não entra em lugar nenhum, anotar em "Pedidos sem lugar".

## Ordem de trabalho (cada pedido)
| Etapa | O que fazer | Skill do pacote |
|---|---|---|
| 1. Entender | Pedido vago ou grande: perguntas curtas, uma por vez; palpite que ele corrige | `brainstorming` |
| 2. Especificar (SDD + BMAD) | `docs/specs/NNN-nome/`: brief (problema, situação no código), spec (R1, R2..., fora do escopo, critérios de aceite testáveis), arquitetura se mexe em dados/servidor | `writing-plans`, `spec-to-code-compliance` (no fim, conferir o código contra a spec) |
| 3. Aprovar | Só implementar com **"pode fazer"**. Sem isso, só brief e spec | |
| 4. Implementar | Menor mudança completa; reaproveitar o que existe; teste antes do código | `ponytail`, `karpathy-guidelines`, `test-driven-development`, `executing-plans` |
| 5. Telas | Padrão visual do programa; celular a 390 px; estados vazios, confirmar antes de apagar, desfazer | `ui-ux-pro-max`, `frontend-design`, `padroes-de-app` |
| 6. Bugs | Reproduzir num teste, achar a causa, provar que o teste antigo falhava e o novo passa | `systematic-debugging` |
| 7. Segurança | Toda mudança em login, senha, cofre, arquivos enviados, rotas públicas, banco ou integração externa | `differential-review`, `sharp-edges`, `supply-chain-risk-auditor` (dependência nova) |
| 8. Conferir | Rodar os testes de verdade e mostrar o resultado antes de dizer "pronto"; dizer o que não foi testado | `verification-before-completion`, `fp-check` |
| 9. Revisar | Revisão antes de juntar mudanças grandes | `requesting-code-review`, `receiving-code-review` |
| 10. Registrar | Commit e push de cada parte pronta; atualizar spec, `docs/PROXIMOS_PASSOS.md`, `DECISOES_E_REGRAS.md` (o que foi pedido e definido, com data) e o mapa; as cópias de `DECISOES_E_REGRAS.md` e do mapa ficam iguais nos três repositórios | `finishing-a-development-branch` |
| 11. Juntar | PR e merge na `main` só com **"pode juntar"**; o servidor da nuvem se atualiza sozinho pela `main` | |

## Regras que não mudam
- Nunca commitar dados reais (banco, planilhas, CPF/CNPJ, senhas, chaves, tokens). Nunca pedir nem aceitar segredo no chat: senhas e chaves só pelas telas do programa, cifradas no servidor. Se chegar segredo no chat, não usar nem guardar; oferecer o caminho pela tela.
- Permissão conferida no servidor, não só na tela. Rota nova: decidir quem pode.
- Migrações só acrescentam; nada de apagar dados em cascata.
- Valores financeiros e resultados técnicos calculados pelo programa, nunca inventados pela IA. A IA sugere; o engenheiro ou o financeiro confere e decide.
- Estudos e dados antigos abrem com os mesmos resultados (Hydrus: regressão dos cálculos).
- Sem instalação global, publicação ou ação fora do repositório sem pedido explícito.
- Não contornar login, CAPTCHA, MFA ou bloqueio de robô de sites de terceiros.
- Teste que já falhava antes não é desculpa nem é escondido: comparar com a versão anterior e dizer.

## Ao responder ao JP
Curto, direto, em português simples: o que ficou pronto (com o commit), o que ele precisa fazer, o que ficou pendente e por quê, e a pergunta que falta (uma por vez). Dizer qual skill foi usada quando fizer diferença.
