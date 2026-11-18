# Agral CAD 003: tarefas e evidências

| R | O quê | Situação | Commit (fork) | Teste |
|---|---|---|---|---|
| R1 | Linhas de quebra (`ADDSURFACEBREAKLINES`, `-ADDSURFACEBREAKLINES`) | Feito | f8c1f67cc | `linha_de_quebra_manda_na_cota`: crista a 103 m dá 103 (sem ela, 100,5) |
| R2 | Arquivo de pontos (`ADDSURFACEPOINTFILE`, PNEZD/PENZD/NEZ/ENZ) | Feito | f62488d8a | `arquivo_de_pontos_entra_na_superficie_e_avisa_linhas_ruins`: cota 205,5 num plano conhecido; linha 122 ruim no aviso; arquivo sumido avisa |
| R3 | LandXML (`IMPORTLANDXMLSURFACE`, `EXPORTLANDXMLSURFACE`) | Feito | cded3aea1 | `landxml_ida_e_volta_mantem_pontos_triangulos_e_cotas` |
| R4 | Alinhamento (`CREATEALIGNMENT`, `ALIGNMENTSTATIONS`): retas e arcos, estacas a cada 20 m, rótulos km+m; dica do mouse com estaca e afastamento | Feito | c71640ce2, 8b… | `alinhamento_perfil_greide_e_secoes`, motor `reta_arco_reta...`, `estaca_e_offset_de_volta` |
| R5 | Perfil do terreno e vista de perfil (`CREATEPROFILEVIEW`): grade, terreno, greide, faixa com estaca, cotas e aterro/corte | Feito | c71640ce2 | cotas 101/105/109 num plano conhecido; motor `perfil_de_plano_inclinado_e_exato` |
| R6 | Greide (`CREATEPROFILE`): PIVs digitados ou clicados na vista, curva vertical | Feito | c71640ce2 | aterro +5 e corte -3; motor `greide_com_curva_vertical` |
| Seções | Linhas e vistas de seção (`CREATESECTIONS`): grade, terreno, eixo, cota do greide, título com estaca e TN | Feito | c71640ce2 | título "0+080.00  TN 105.000" e cota do greide |
| R7 | Assembly da barragem (`CREATEASSEMBLY`): crista com declividade para montante, taludes H:V, bermas pela tabela de patamares, corte | Feito | cd3c6036d, 61b74cc63 | motor: barragem de 10 m em terreno plano igual à conta à mão; corte de 2 m; sem bermas |
| R8 | Corredor (`CREATECORRIDOR`): linhas 3D por código, superfície `<EIXO>-CORR` com limite pela saia, volumes | Feito | 61b74cc63 | `corredor_de_barragem_em_terreno_plano`: aterro igual à conta (0,5%), nada fora da saia |
| R9 | Refazer em cadeia | Feito: greide, assembly ou superfície do terreno novos refazem perfil, corredor, superfície do corredor e seções | 61b74cc63 | greide mais baixo dá menos aterro |
| R10 | Tudo em segundo plano | Superfície: feito (barra de progresso) | 0897f9ecc | `montagem_em_segundo_plano_desenha_ao_terminar` |

Critério 1 (quebra muda o TIN): passou. Critério 2 (1.000 pontos): o teste usa 121 pontos; falta o de 1.000.
Critério 3 (LandXML): passou.

Também: tudo pergunta na linha de comando e funciona pelo MCP (`alinhamento_perfil_e_secoes_pela_linha_de_comando`);
superfície refeita atualiza perfis e seções; conferido na tela pelo roteiro (fotos em 18/11).

Critério 6 (corredor de barragem em terreno plano): passou. Critério 7 (na tela): conferido no roteiro com um vale
(`scripts/agral/gerar_vale.py`), fotos de 18/11.
