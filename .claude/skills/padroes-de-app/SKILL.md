---
name: padroes-de-app
description: Checklist das coisas lógicas que todo app precisa ter (criar, abrir, apagar, começar do zero, desfazer, confirmar, estados vazios, navegação, salvamento, erros). Usar ao criar ou revisar qualquer tela, ferramenta ou fluxo do Hydrus, e antes de dar uma funcionalidade por concluída.
---

# Padrões de app (Hydrus)

Uma funcionalidade só está pronta quando o usuário consegue fazer o ciclo inteiro sem ajuda e sem surpresa.
Para cada tela ou fluxo novo, passar por esta lista. Item que não se aplica: dizer por quê.

## 1. Ciclo de vida dos dados
- [ ] **Criar**: existe um jeito óbvio de começar (botão com verbo: "Novo projeto", "Traçar a bacia").
- [ ] **Novo começa do zero**: um item novo não herda dados do último aberto (nada de estado global, cache de página ou sessionStorage vazando). Testar: abrir um cheio, criar um novo, conferir que está vazio.
- [ ] **Abrir e voltar**: o que foi salvo volta igual ao reabrir (recarregar a página prova).
- [ ] **Editar**: todo dado digitado pode ser mudado depois, inclusive cadastro e nomes.
- [ ] **Apagar**: todo item que o usuário cria pode ser apagado (estudo, ferramenta do projeto, documento, imagem, página, modelo). Apagar só o que foi pedido: apagar uma ferramenta não apaga as outras do projeto.
- [ ] **Apagar com confirmação** e dizendo o que some; o que for grande vai para a lixeira ou tem desfazer.
- [ ] **Duplicar / cenário**: quando faz sentido comparar alternativas, dá para copiar sem mexer no original.

## 2. Segurança contra erro do usuário
- [ ] **Desfazer** (Ctrl+Z ou botão) para mudanças feitas por clique, pelo agente ou em massa.
- [ ] Ação irreversível pede confirmação; ação reversível não pede (não encher de diálogos).
- [ ] Sair com alterações não salvas: avisa ou salva sozinho.
- [ ] Campos validam na hora, com unidade e faixa plausível; mensagem diz o que fazer, não só "inválido".

## 3. Estados da tela
- [ ] **Vazio**: explica o que fazer primeiro ("Busque os dados pela coordenada…"), nunca tela em branco ou "–" sem contexto.
- [ ] **Carregando**: mostra que está trabalhando e quanto falta quando demora (> 2 s).
- [ ] **Erro**: diz o que aconteceu em português e o próximo passo (internet, chave, dado faltando). Nunca só "Erro 500".
- [ ] **Sem internet**: o que depende de rede avisa; o resto continua funcionando.
- [ ] **Parcial**: resultado com dado faltando aparece marcado ("conferir"), não como se estivesse certo.

## 4. Navegação
- [ ] Sempre dá para voltar (botão "Voltar ao projeto", Esc fecha diálogo).
- [ ] A posição volta onde estava (página/aba aberta, filtro, rolagem) ao recarregar.
- [ ] Atalhos de teclado nos fluxos repetitivos (Alt+setas nas páginas, Enter confirma, Esc cancela).
- [ ] Um item clicável parece clicável; um desabilitado diz por quê (title ou nota).
- [ ] Nada importante escondido atrás de botão flutuante, rodapé fixo ou rolagem lateral.

## 5. Salvamento e dados
- [ ] Salvamento automático com indicador ("Salvo 15:06").
- [ ] Estudos antigos abrem com o mesmo resultado (compatibilidade); campo novo tem valor padrão que não muda o cálculo antigo.
- [ ] Exportar e importar o que o usuário pode precisar levar para outro lugar (JSON, CSV, PDF, DOCX).
- [ ] Números com vírgula decimal, unidades sempre visíveis, sem travessão "—".

## 6. Consistência
- [ ] O mesmo nome para a mesma coisa em todas as telas (botão, título, toast).
- [ ] Mesmo componente para a mesma função (tabela, botão primário, aviso).
- [ ] Tema claro e escuro legíveis; impressão e relatório sempre em fundo branco.

## Como revisar
1. Abrir a tela como usuário novo (projeto vazio) e como usuário com projeto cheio.
2. Fazer o ciclo: criar, preencher, recarregar, editar, apagar, criar outro.
3. Anotar cada item da lista que falhou, com a tela e o passo.
4. Corrigir o que é pequeno e certo; listar o resto em `HYDRUS_CONTEXT.md` como pendência.
5. Teste automático para os ciclos críticos (ver `test/estudo.mjs`: projeto novo vazio, apagar dados da ferramenta).
