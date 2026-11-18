# Agral CAD 001: tarefas e evidências

"Pode fazer" do JP em 17/11/2026. Código na bifurcação `WhyY0uBullyMe/OpenCADStudio` (feita pelo JP em 17/11), branch
`agral-cad` sobre o commit `21e5d29`, pasta `crates/agral_civil_motor`. A cópia provisória em patches foi tirada do
Hydrus.

## Feito (motor, sem tela)

| Tarefa | Requisito | Evidência |
|---|---|---|
| Ler arquivo de pontos PNEZD, PENZD, NEZ, ENZ; separador e decimal; linha ruim contada e mostrada | R9 | `tests/pontos.rs`, 6 testes |
| Superfície vazia; lista de itens; ligar, desligar, tirar; marca de desatualizada | R6, R7 | `tests/superficie.rs` |
| Curvas de nível como linha de quebra (Delaunay restrita, `spade`) | R8 | teste "nenhum triângulo cruza" + teste de controle que mostra a Delaunay pura cruzando |
| Linhas de quebra, limite externo, furo, comprimento máximo de borda | R13 | `tests/superficie.rs` |
| Curvas cruzadas: aviso, sem travar | R8 | `curvas_que_se_cruzam_geram_aviso_e_nao_travam` |
| Curvas geradas da superfície passam pelas de entrada (desvio < 0,25 m) | critério 3 | `curvas_geradas_passam_pelas_curvas_de_entrada` |
| 1 milhão de pontos de curvas: 1.997.934 triângulos em 1,5 s (container de 4 núcleos, release) | R14 | `tests/desempenho.rs` (rodar com `--release -- --ignored`) |
| LandXML: ler (Norte Este Cota, sem face invisível) e escrever, ida e volta igual | R16 | `tests/landxml.rs`, 3 testes |

Caso real: `caso-cedro.md` (185 mil triângulos, nenhum cruzando curva; triângulos planos a resolver).

Resultado em 17/11: 19 testes passam, 0 falham (`cargo test -p agral_civil_motor`), mais o de desempenho.

| Diagnóstico de curvas que se cruzam | R17 | `tests/diagnostico.rs`, 4 testes |
| Correção de curvas que se cruzam: zero cruzamento | R18 | `tests/correcao.rs`, 5 testes; `CURVAS_V1`: 1.208 para 0 (`caso-cedro-v1.md`) |

| Emendar pedaços da mesma cota (folga até 5 m, sem grampo, sem cruzar, longe da borda) | R19 | `tests/emenda.rs`, 8 testes; Cedro: ver `caso-cedro-v1.md` |

| Pacote `agral.civil` embutido (pasta `plugins` do `.exe`), comandos `CREATESURFACE`, `ADDSURFACECONTOURS`, `REBUILDSURFACE` | R1 (parte), R3, R5, R6, R7 | lote do Open CAD Studio: Cedro área 2, 186 mil triângulos em 2,2 s, salva e reabre igual; `CURVAS_V1`, 2,24 milhões em 21 s |
| Cruzamentos marcados no desenho (camada `AGRAL-DIAG-CRUZAMENTO`) | R17 | `CURVAS_V1`: 1.208 círculos |
| ZIP para Windows pelo GitHub Actions da bifurcação | R3 | `.github/workflows/agral-windows.yml`; 1ª compilação 17/11 passou (testes do motor e do pacote também no Windows), run 44150061413, 31 min |

## Limites conhecidos (a fazer)

- Limite e furo tiram o triângulo inteiro pelo centro; o Civil 3D corta o triângulo na linha. Fazer o corte.
- Curva de nível: falta afinar e completar vértices (R8).
- Pontos repetidos na mesma posição com cota diferente: fica a primeira e sai aviso. Conferir com o Civil 3D.

## Achados do pacote (17/11)

- Cada mensagem entre o programa e o pacote vai até 64 MB: o desenho é lido e escrito em lotes.
- O comando de pacote tem até 30 s; desenhar a malha de 2,24 milhões de triângulos leva 17 s (1,5 s por malha de
  100 mil vértices). Investigar onde o programa gasta esse tempo, ou desenhar por padrão só curvas e borda (como o
  estilo padrão do Civil 3D) e os triângulos só quando o estilo pedir.

## Próximas tarefas

1. Tirar triângulos planos até passar no critério 3 com o caso Cedro.
   (O Open CAD Studio inteiro já compilou em 17/11.)
2. LAS e LAZ com filtro de classe (R10); ASCII Grid e GeoTIFF (R12).
3. Pacote `agral.civil` de fábrica: comandos `CreateSurface` e os outros, árvore Toolspace (R1 a R3).
4. Superfície desenhada como malha; gravar a definição no DWG e reabrir igual (R5).
5. `EditDrawingSettings` com SIRGAS 2000 e fuso UTM (R4).
6. Estilo Agral (R15) e teste com o arquivo real do JP (critérios 1, 2 e 4).
