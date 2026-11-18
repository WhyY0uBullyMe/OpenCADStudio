# Agral CAD 002: o fluxo da barragem no CAD (brief)

Situação: brief de 18/11/2026, a partir do relato do JP de como ele faz hoje no Civil 3D (caso Murilo). Serve para
conferir se o Agral CAD e o Tools cobrem o fluxo inteiro. Não autoriza código; cada parte vira spec.

## Como o JP faz hoje (18/11)

1. Traz as **curvas** (geralmente do Metashape).
2. Transforma as curvas em **superfície do terreno**.
3. **Acha o lugar** da barragem.
4. Gera o **aterro** com as dimensões que ele define:
   - largura da crista;
   - declividade da crista: geralmente 1% para dentro (lado de montante), na crista inteira;
   - bermas, se houver;
   - largura da base e taludes.
   - Isso vira um **corredor**.
5. Faz a **superfície do corredor**.
6. Faz uma **superfície nova**, chamada por padrão "Junção" (o nome tem de poder mudar), que junta o terreno e o
   aterro:
   - cola primeiro o terreno e depois o aterro (Paste Surface);
   - quem vem por último manda: o aterro corta o terreno.
7. **Limpa os triângulos ruins.** Dão "dentes" e triângulos errados, geralmente fora da saia. Ele liga os triângulos só
   da superfície do aterro, apaga os que estão sobrando e atualiza.
8. Calcula o **volume** do aterro.
9. **Extrai as curvas** da Junção, da cota da crista até o fundo, e gera a **planilha de cota, área e volume** (volume
   na cota e acumulado), conforme a folga.

## Conferência: o que já temos e o que falta

| Passo | No Agral CAD ou no Tools | Situação |
|---|---|---|
| 1. Importar curvas (DWG ou DXF) | Agral CAD `AddSurfaceContours` | especificado na 001 (R8) |
| 2. Superfície das curvas | Agral CAD, curva como linha de quebra, sem teto de triângulos | especificado na 001 (R6 a R8, R14) |
| 3. Achar o lugar | Hydrus "Achar o lugar"; o otimizador por custo vem depois | existe (por relevo público); falta usar a superfície do levantamento |
| 4. Aterro: crista, declividade de 1% para montante, taludes, bermas, base | modelo da barragem na ficha (decisão de 18/11) + **corredor** no Agral CAD, com a tabela de patamares | **a especificar**: alinhamento, perfil e corredor; declividade transversal da crista entra no assembly |
| 5. Superfície do corredor | Agral CAD, a partir das linhas do corredor, com o **limite automático pela linha de saia** | **a especificar** |
| 6. Superfície "Junção" | Agral CAD: superfície nova com o nome editável (padrão "Junção") e a definição **"colar superfície" em ordem**: terreno primeiro e aterro por último, e o último manda | **falta na 001**: entra como requisito novo |
| 7. Triângulos ruins | **Dois caminhos:** (a) prevenção: a superfície do aterro já nasce cortada pela linha de saia, que é o que tira a maioria dos "dentes"; (b) edição: apagar triângulos e linhas à mão, guardada como item da definição, para sobreviver ao refazer | **falta na 001**: estava fora do escopo; passa a entrar |
| 8. Volume do aterro | Agral CAD: **volume entre duas superfícies** (terreno e Junção, ou terreno e aterro), com corte e aterro separados | **falta na 001**: estava fora do escopo; passa a entrar |
| 9. Curvas da Junção e a planilha de cota, área e volume | Agral CAD extrai as áreas por cota da Junção (ou calcula o volume abaixo de cada cota direto na superfície, que é mais exato); o Tools monta a planilha no motor do reservatório (`cavPts`, `cavVdeH`), com volume na cota e acumulado | motor do Tools **existe**; falta a ponte que leva as áreas do CAD para o estudo |
| Tudo em cadeia | Mudou a crista ou um talude: refaz corredor, superfície do corredor, Junção, volume e planilha | regra R24 da spec 010; **a especificar no CAD** |

## O que isso pede de novo

1. **Superfície por colagem (Paste Surface), com ordem.** Nome editável, padrão "Junção".
2. **Edição de triângulos** (apagar linha, apagar triângulo), guardada na definição da superfície.
3. **Limite pela linha de saia** na superfície do corredor, para não nascer com dentes.
4. **Volume entre superfícies** (corte, aterro e líquido).
5. **Áreas por cota** de uma superfície, para a planilha de cota, área e volume do reservatório.
6. **Assembly da barragem:** crista com declividade transversal (padrão 1% para montante), taludes, bermas pela tabela
   de patamares, base.
7. **Cadeia:** cada uma dessas peças sabe de quem depende e se refaz no "Regenerar".

## Caso de teste

O caso Murilo (Fazenda G5), do começo ao fim. **Falta:** as curvas do Metashape e o DWG do Civil 3D com o corredor, a
Junção e a planilha de volume do Murilo, para comparar os números (volume do aterro e a planilha) com os do Civil 3D.

## Situação no Agral CAD (sessão do Agral CAD, 18/11, ao trazer este brief da sessão mãe)

- Edição de triângulos: **Delete Line** já existe e fica guardada na definição (`EDIT_DELETELINE`, sobrevive ao
  refazer). Falta apagar triângulo inteiro e Swap Edge.
- Paste Surface, volume entre superfícies, áreas por cota, limite pela saia e assembly da barragem: a fazer.
- Os arquivos do Murilo no Drive passam do limite de 10 MB do conector; aguardando o JP mandar de outro jeito.
