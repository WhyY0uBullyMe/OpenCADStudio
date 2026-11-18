# Agral CAD 001: caso real Fazenda Cedro, `CURVAS_V1.dwg` e LandXML do Civil 3D

Arquivos do JP no Drive (pasta compartilhada por link). Segundo o JP, as curvas são a saída direta do Global
Mapper, baixados em 17/11/2026 para a pasta de dados da sessão.
**Dado de cliente: não vai para nenhum repositório.**

- `CURVAS_V1.dwg` (61 MB, DWG R14): camada CONTOURS com 4.773 polilinhas 2D, curvas de 878,5 a 980 m a cada 0,5 m.
  4.564 delas estão **suavizadas como spline** (PEDIT Spline): 1.124.198 vértices da curva suavizada e 224.493
  vértices do polígono de controle (a curva original). 13 têm só o polígono de controle. Camada LAYER: 1 polilinha
  na cota 927,24.
- `Curvas - Murilo.xml` (172 MB): superfície "TN" exportada do Civil 3D 2026 a partir de
  `CURVAS_V1_CORREDOR_1M.dwg`; SIRGAS 2000 / UTM 22S (EPSG 31982); 984.057 pontos, 1.965.811 triângulos, limite
  externo próprio.

## Achado principal: a suavização spline cria os cruzamentos

| Leitura das curvas | Vértices | Cruzamentos entre curvas | Pares de curvas | Montagem |
|---|---|---|---|---|
| Errada (antes): controle e ajuste misturados | 1.348.434 | 145 mil trechos em conflito | | 2,3 s |
| Curva suavizada (o que aparece na tela) | 1.123.788 | **1.208**, todos entre cotas diferentes | 201 | 2,0 s, 2,24 milhões de triângulos |
| Polígono de controle (curva original) | 224.836 | **0** | 0 | 0,28 s |

Exemplo: curva 924,50 cruzando a 925,00 em E 780.882,27, N 8.175.285,50.

**Conclusão:** as curvas originais não se cruzam; a suavização faz a curva passar do ponto onde as curvas ficam
apertadas e invadir a vizinha. É provável que seja o "problemão" que o JP relatou. A primeira leitura do motor
também misturava os vértices de controle com os da curva; corrigido.

## Comparação com o Civil 3D (critério 4)

Grade de 8 m, 117 mil pontos dentro das duas superfícies:

| Leitura | Até 1 cm | Mediana | 95 % | 99 % | Máximo |
|---|---|---|---|---|---|
| Curva suavizada | 61,1 % | 0,007 m | 0,105 m | 0,351 m | 10,99 m |
| Polígono de controle | 50,9 % | 0,010 m | 0,118 m | 0,376 m | 11,16 m |

O Civil 3D usou a curva suavizada (fica mais perto). O critério 4 (tudo dentro de 1 cm no interior) **ainda não
passa**. Diferenças que faltam explicar: o Civil 3D tem 984 mil pontos e nós 1,12 milhão (ele afina vértices ou o
DWG de origem é outro, `CURVAS_V1_CORREDOR_1M`); triângulos planos; e onde as curvas se cruzam nenhuma das duas
superfícies está certa.

## Pendências

- Corrigir as curvas que se cruzam (R17): forma de correção a decidir com o JP.
- Triângulos planos (critério 3).
- 4 arestas de triângulo ainda cruzam curva na leitura pelo polígono de controle, sem conflito acusado: investigar.
- 13 polilinhas spline sem a curva suavizada gravada: calcular a spline a partir do controle.

## Correção dos cruzamentos (R18), 17/11

Decisão do JP: "eu quero que ela não cruze, apenas isso"; o método ficou comigo. Implementado em
`crates/agral_civil_motor/src/correcao.rs` (bifurcação, commit `b5f3d7040`), com 5 testes:

1. **Lente:** entre dois cruzamentos seguidos de duas curvas, as duas seguem a linha do meio, cada uma 2,5 cm para
   o lado que tinha fora da lente (folga total de 5 cm). Fora da lente nada muda.
2. **Corte:** ponta que cruza uma vez, laço da curva nela mesma e o que sobrar perdem o trecho que cruza, na curva
   mais curta. Tirar trecho nunca cria cruzamento novo, então o fim é sempre zero cruzamento.

Resultado no `CURVAS_V1.dwg` (curvas suavizadas, como o Civil 3D usa):

| | Antes | Depois |
|---|---|---|
| Cruzamentos entre curvas | 1.208 (201 pares) | **0** |
| Trechos em conflito na montagem | 1.032 | **0** |
| Comprimento total das curvas | 1.041.935,6 m | 1.041.873,6 m (62 m cortados em 44 trechos) |
| Curvas alteradas | | 262 de 4.773 |
| Tempo da correção | | 14 s (640 lentes, 46 rodadas) |

Pendências: 2 arestas de triângulo ainda cruzam curva sem conflito acusado (investigar a junção de pontos a 1 mm);
deixar a
correção mais rápida (hoje refaz o diagnóstico inteiro a cada rodada).

## Correção com a regra do JP (barriga), 17/11

O JP explicou: em geral a curva de cima faz uma barriga para dentro da de baixo; a barriga é cortada e a curva
refeita "conforme seria antes", sem canto. Implementado como primeiro passo da correção (commit `1cbaa9e5e`):

- Entre um cruzamento de entrada e o de saída, a curva que mais se afasta do próprio caminho é a da barriga (empate:
  a de cima). O trecho é refeito por uma curva suave que segue a direção de antes e de depois.
- Só vale se o trecho refeito não cruzar nenhuma curva. Barriga longa refeita costumava invadir uma terceira curva e
  gerava cortes (uma tentativa cortou 5,4 km), por isso ficou limitada a uma entrada e uma saída.

| Versão | Barrigas | Linha do meio | Cortes | Comprimento final | Tempo |
|---|---|---|---|---|---|
| Só linha do meio | 0 | 640 | 44 | 1.041.873,6 m | 14 s |
| **Barriga + linha do meio (atual)** | **53** | **548** | **34** | 1.041.693,0 m | 22 s |

Cruzamentos no fim: 0 nas duas. O comprimento cai mais na versão atual porque tirar barriga encurta a curva.
Neste arquivo a maioria dos cruzamentos é das duas curvas ondulando uma sobre a outra, sem barriga clara; por isso
a linha do meio ainda resolve a maior parte.

## Emendar pedaços (R19), 17/11

Folga máxima de 5 m (aprovada pelo JP). Regras: mesma cota, a emenda segue a direção das duas pontas (ângulo até
75°), não cruza curva, e ponta a menos de 1 m do limite da área não é emendada.

| Arquivo | Emendas entre pedaços | Curvas fechadas de novo | Cruzamentos depois |
|---|---|---|---|
| Área 2, com o limite (camada LAYER) | 0 | 0 | 0 |
| Área 2, sem limite | 57 | 7 | 0 |
| `CURVAS_V1` corrigido, com o limite do LandXML do Civil 3D | 90 | 663 | 0 |

- Na área 2 as pontas soltas estão na borda: sem o limite o programa emendaria 57 pontas que na verdade saem da
  área. **Por isso o limite tem de ser informado antes de emendar.**
- No `CURVAS_V1` as 663 curvas eram fechadas no polígono de controle; a suavização do Global Mapper deixou uma folga
  entre começo e fim.
- Ainda não conferido a olho pelo JP: fazer isso quando o comando estiver no CAD.
