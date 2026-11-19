# 004 Compute Materials (volumes por seção)

Pedido do JP (18/11): "tem que lançar um compute materials", com opção de cálculo por superfície, "igual no Civil 3D",
para tocar os projetos agora; incrementar depois. Seções com a aparência do template dele (TEMPLATE_JP.dwt).

## Requisitos
- R1 `COMPUTEMATERIALS [eixo] [base] [comparar] [fator corte] [fator aterro]`. Sem argumentos abre a janela (como
  no AutoCAD; `-COMPUTEMATERIALS` pergunta na linha de comando), com os padrões: o eixo único, a superfície do eixo como base, a superfície do corredor (`<EIXO>-CORR`) como comparação,
  fatores 1. Qualquer par de superfícies serve (terreno x projeto, terreno x limpeza).
- R2 Em cada linha de seção do eixo (as de `CREATESECTIONS`): área de corte (comparação abaixo da base) e de aterro
  (comparação acima), só onde as duas superfícies existem.
- R3 Volumes pelo método das áreas médias entre seções vizinhas; fator de corte e de aterro aplicados ao volume;
  acumulados e volume líquido acumulado (corte ajustado menos aterro ajustado).
- R4 O critério fica gravado no eixo e o cálculo é refeito quando o eixo, as seções ou as superfícies mudam.
- R5 `CREATEVOLUMETABLE [eixo] [x,y]`: tabela de volumes no desenho (estaca, áreas, volumes, acumulados).
- R6 `EXPORTMATERIALS [eixo] [arquivo]`: CSV com ";" e vírgula decimal (abre direto no Excel em português).
- R7 Toolspace mostra corte, aterro e líquido do eixo.
- R8 Seções desenhadas nas camadas do template quando existirem (GRID MAIOR-SEC-AS, GRID MENOR-SEC-AS, TICK-SEC-AS,
  TERRENO NATURAL-AS, ATERRO-AS, PERFIL GRID-TXT-AS) e texto SIMPLEX-AS.

## Fora do escopo agora
Volume por material (núcleo, espaldar, filtro), diagrama de massas, estilo de seção editável, rótulos iguais aos do
Civil 3D (os estilos do Civil 3D ficam em objetos próprios dele que não são lidos; falta print ou DWG explodido).

## Aceite
- Teste: plano a 100 e plano a 102 com seções a cada 20 m e 20 m para cada lado: aterro 80 m2 por seção; 100 m de
  eixo dão 8000 m3; fator 1,25 no aterro dá 10000 m3; corte zero.
- Teste: base acima da comparação vira corte.
- Comando sem argumentos funciona pela linha de comando com os padrões.
