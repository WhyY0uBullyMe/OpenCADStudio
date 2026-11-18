# 005 Imagem de satélite por link XYZ

Pedido do JP (18/11): "datum do lugar e imagem de satélite por trás, como o Geolocation"; "não estou achando pra
ligar as imagens". Link XYZ "do Google, como no QGIS": aprovado por ele para uso interno, com o aviso de que fica
fora dos termos de uso do Google (DECISOES_E_REGRAS.md, 18/11). O programa aceita qualquer link XYZ.

## Requisitos
- R1 `GEOLOCATION`: janela com o sistema do desenho (SIRGAS 2000 / UTM, fuso 1 a 60, hemisfério S ou N) e o link
  XYZ (`{x}`, `{y}`, `{z}`; opções prontas: Google Satélite, Esri World Imagery, OpenStreetMap). Gravado no DWG.
- R2 `GEOMAP` [Aerial/Off]: Aerial traz a imagem da área vista na tela; Off apaga as imagens trazidas.
- R3 `GEOMAPIMAGE`: imagem de uma área escolhida com dois cantos.
- R4 A imagem é reprojetada ponto a ponto de Web Mercator para UTM (fica no lugar certo, sem giro), num PNG ao lado
  do desenho (ou na pasta do programa se o desenho não foi salvo), como IMAGE no fundo, camada C-IMAGEM.
- R5 Download em segundo plano, com aviso na linha de comando; zoom escolhido pela resolução da tela, com limite de
  tamanho; falha de rede vira mensagem, não trava o programa.

## Aceite
- Conversão UTM ida e volta com erro abaixo de 1 mm; no meridiano central o norte é k0 vezes o arco de meridiano
  calculado por integração numérica.
- Tiles: o tile da estaca certa para pontos conhecidos (z = 0 e z = 1; Goiânia em z = 15).
- Montagem da imagem com tiles sintéticos (sem rede): pixel de cor conhecida cai no lugar UTM certo.
