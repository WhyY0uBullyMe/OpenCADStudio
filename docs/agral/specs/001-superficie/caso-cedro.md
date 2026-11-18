# Agral CAD 001: caso real Fazenda Cedro, área 2

Arquivo enviado pelo JP em 17/11/2026: `CURVAS_FAZENDA_CEDRO - ÁREA 2.dwg` (5,5 MB, DWG R14). **Dado de cliente:
não vai para nenhum repositório.** Lido com o leitor de DWG do Open CAD Studio (`acadrust`), pela ferramenta
`crates/agral_civil_motor/examples/ler_curvas.rs`.

## O que tem no arquivo

| Camada | Objetos | Vértices | Cotas |
|---|---|---|---|
| CONTOURS | 2.109 polilinhas 3D | 94.673 | 879,0 a 980,0 m, curvas a cada 0,5 m (203 cotas) |
| LAYER | 1 polilinha 3D | 81 | 878,7 a 979,0 m (parece o limite da área; a confirmar com o JP) |

Mais 1 bloco inserido. Nenhuma polilinha fora do plano horizontal.

## Resultado com o motor Agral Civil (17/11)

| Montagem | Triângulos | Tempo | Triângulo cruzando curva | Desvio médio das curvas geradas | Pontos com desvio > 0,125 m | Pior desvio |
|---|---|---|---|---|---|---|
| Só curvas | 186.374 | 0,12 s | 0 | 1,84 m | 7.250 de 167.695 | 439 m |
| Curvas + limite (camada LAYER) | 185.815 | 0,17 s | 0 | 0,025 m | 1.093 de 160.932 | 56 m |
| Curvas + borda máxima de 50 m | | | 0 | 0,041 m | 1.608 de 161.529 | 27 m |

O plugin holandês recusaria este arquivo (teto de 120 mil triângulos).

## Leitura

- **Critério 2 (nenhum triângulo cruza curva): atendido** no caso real.
- **Critério 3 (curva gerada coincide com a de entrada, desvio < 1/4 do intervalo = 0,125 m): ainda não.** Sem
  limite, a borda fecha com triângulos longos e planos (pior 439 m). Com o limite, sobram cerca de 0,7 % dos pontos
  fora, por **triângulos planos** no meio da área: os três vértices na mesma curva, em topo de morro, fundo de vale e
  dobra apertada de curva. É o problema que o Civil 3D trata com "Minimize flat areas".
- **Próxima tarefa do motor:** tirar os triângulos planos (inserir ponto nas arestas planas que não são curva, com
  cota interpolada entre as curvas vizinhas), e repetir esta medição até passar no critério 3.
