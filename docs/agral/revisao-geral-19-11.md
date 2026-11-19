# Revisão geral do Agral CAD (19/11/2026)

Pedido do JP (18/11): revisão geral da estrutura, testes pelas premissas (SDD + BMAD, skills do protocolo),
funcionalidades, aparência e retorno visual. Feita com três revisões independentes (só leitura, sem ver a opinião
uma da outra): (1) erros e segurança no código civil, (2) spec contra código e lacunas de teste, (3) interface e
retorno visual. Cada achado foi conferido no código antes de corrigir.

## Corrigido nesta revisão
| Achado | Onde | Teste |
|---|---|---|
| DWG estragado (contagem negativa ou enorme) derrubava o programa ao abrir | `src/civil/definicao.rs` | `contagem_estragada_nao_derruba` |
| GEOMAP com área enorme montava bilhões de tiles e travava | `crates/agral_civil_motor/src/geo.rs` (`planejar`) | `imagem_reprojetada_no_lugar_certo` (1000 km) |
| Desenho da superfície montada em segundo plano ficava fora do desfazer | `src/app/civil.rs` (`civil_montagem_pronta`) | suíte |
| Falha dentro da montagem em segundo plano travava a fila para sempre | `src/app/civil.rs` (`catch_unwind`) | suíte |
| Arquivo indicado no DWG (LandXML, pontos) lido de caminho de rede, dispositivo ou gigante | `src/app/civil.rs` (`arquivo_aceito`) | `caminho_de_rede_e_dispositivo_nao_sao_lidos` |
| Greide além do fim do eixo somava volume que não existe no corredor | `src/app/civil_alinhamento.rs` | `revisao_greide_alem_do_eixo_e_corredor_proprio` |
| Eixo com o próprio corredor como terreno se remontava sem fim | `civil_alinhamento.rs` (trava de profundidade e recusa na vista) | idem |
| Intervalos minúsculos travavam (estacas, seções, corredor, bermas); contador de bermas estourava | `civil_alinhamento.rs`, `corredor.rs` | idem |
| "nan" e "inf" digitados entravam na geometria e no DWG | `commands/alignment.rs`, `commands/surface.rs`, `modules/civil/pivs.rs` | `le_piv_com_e_sem_curva` |
| NaN gravado no DWG para corredor sem volume | `src/civil/alinhamentos.rs` (-1 no lugar) | `grava_e_le_igual` |
| Totais do Compute Materials ficavam velhos sem a tabela | `civil_alinhamento.rs` | suíte |
| Menus do Toolspace sem o eixo (Stations, Dam Assembly) | `src/ui/window/toolspace.rs` | suíte |
| Apagar eixo sem perguntar | `toolspace.rs`, `civil_dialog.rs` (`ApagarEixo`) | suíte |
| Trocar o eixo na janela não recarregava os outros campos | `commands/surface.rs` | suíte |
| Enter não confirmava as janelas civis | `src/app/update/mod.rs` | `janelas_de_configuracao` |
| Padrão de emendar curvas diferente na janela e no comando | `civil_dialog.rs` (desligado nos dois) | suíte |
| Nomes diferentes para o mesmo comando na faixa; m2/m3; ordem corte/aterro; rótulo de estaca de cabeça para baixo; dica com duas cotas dentro da vista de perfil | `modules/civil/mod.rs`, `toolspace.rs`, `civil_alinhamento.rs`, `app/view/mod.rs` | suíte |
| Specs 004 e 005 desatualizadas (janela, JPEG); caminhos `docs/specs/agral-cad` | specs e comentários | |

## Pendente (por prioridade)
1. **Corredor com seções que não chegam ao terreno** ainda soma a área parcial no volume (só avisa a contagem).
   Decidir com o JP: tirar do volume ou completar até o alcance.
2. **Altura de texto única** para planta, perfil, seções e tabela (mm de plotagem x escala), com a escala do
   desenho nas configurações. Hoje cada desenho usa uma regra.
3. **Barra de progresso com Cancelar** genérica: GEOMAP (tiles), corredor e seções. Hoje só a superfície tem o
   cartão, e sem Cancelar. O corredor e as seções ainda rodam na linha principal.
4. **Curvas de nível na linha principal** e sem limite de quantidade de cotas (estilo de 0,1 m em relevo grande
   trava). Levar para o segundo plano e limitar.
5. **Palavras das perguntas clicáveis** (`[Add/Delete]` como botões, como no resto do programa) e validação a cada
   pergunta (hoje um erro na 2.ª do CREATEASSEMBLY só aparece na 9.ª).
6. **Idioma do desenho**: tabela em português com vírgula, faixa do perfil e seções em inglês. Escolher um (o
   português parece o certo para as entregas) e aplicar em todos os rótulos.
7. **Testes de resposta conhecida que faltam**: corredor em terreno inclinado e em eixo curvo, áreas médias com
   seções variáveis, larguras assimétricas no Compute Materials, ponto UTM fora do meridiano central contra o
   IBGE/PROJ, abrir e salvar DWG de verdade com superfície, eixo e materiais.
8. **Specs que faltam**: edições da superfície, EDITPROFILE, EDITSAMPLELINES, SAMPLEMORESOURCES (mudam resultado
   de engenharia), e uma spec de interface (painéis, janelas, abas por seleção). 004 e 005 sem `tarefas.md`.
9. **Desempenho**: registros civis lidos a cada quadro e a cada movimento do mouse; grades espaciais que um vértice
   perdido em (0,0) estraga; `tirar_borda_comprida` lento em superfície grande; Object Viewer perde precisão em UTM.
10. **Menores**: emendas aceitas na mesma rodada podem se cruzar; estaca errada perto do início de arco; Undo do
    CREATECORRIDOR dividido em dois na primeira vez; GEOLOCATION do Agral e GEOGRAPHICLOCATION do programa base são
    dois lugares para a mesma coisa.

## Situação das specs (resumo)
- 001 superfície: feitos os itens centrais (definição, curvas, pontos, LandXML, limites, diagnóstico). Faltam
  LAS/LAZ, DEM, objetos do desenho, recorte exato pelo limite e a comparação com o Civil 3D (depende do JP).
- 003 corredor: R1 a R8 feitos com teste de resposta conhecida; R9 (cadeia de dependências) e R10 (segundo plano)
  em parte.
- 004 Compute Materials e 005 imagem XYZ: feitos, com teste.
