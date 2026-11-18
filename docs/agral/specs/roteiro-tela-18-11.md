# Agral CAD: roteiro de teste pela tela (18/11/2026)

Pedido do JP: "mete um script de teste e vê o que está faltando". O programa roda no Xvfb (Linux, sem placa de
vídeo) e um roteiro em Python o pilota pelo MCP (`--mcp`), com mouse pelo xdotool e fotos da janela. Roteiro no
fork: `scripts/agral/roteiro_tela.py` (com `gerar_morro.py` e `mcp_cliente.py`).

## Achados e correções

| # | Achado | Correção | Commit (fork) |
|---|---|---|---|
| 1 | Pelo MCP e na linha de comando o espaço vale Enter: `CREATESURFACE TN` abria a janela | Versões com hífen que perguntam cada valor: `-CREATESURFACE`, `-ADDSURFACECONTOURS`, `-ADDSURFACEBOUNDARIES`, `-SURFACEPROPERTIES`; nomes de camada com espaço aceitos | cf856dd4b |
| 2 | Ao abrir, perguntava se queria ser o CAD padrão | Tirado (o JP usa junto com o AutoCAD); FILEASSOC continua | cf856dd4b |
| 3 | Toolspace já colocado nunca ficava escondido | Ajuste uma vez também para quem já tinha; ao montar superfície não salta aberto | cf856dd4b, f40923aeb |
| 4 | Cota no mouse não aparecia ao só passar o mouse | O relógio arma em qualquer movimento do ponteiro; conferido: etiqueta branca junto do cursor depois de 2 s parado | f40923aeb |
| 5 | Instalador MSI não saía (versão inválida) | Versão 1.0.<compilação> | 2c717cb7c |

## Conferido funcionando na tela

Abrir DXF; criar superfície e acrescentar curvas pelos comandos; montagem em segundo plano; aba contextual TIN
Surface ao clicar; Properties "TIN Surface" com estilo, pontos, triângulos e cotas; duplo clique abre Surface
Properties; tema Agral.

## Ainda a melhorar (visto nas fotos)

- Janela Surface Properties sem título no topo.
- Triângulos compridos na borda convexa (como o Civil 3D sem comprimento máximo): sugerir comprimento máximo ou
  limite ao montar.
- Aviso de placa de vídeo aparece em máquina sem GPU (só na máquina de teste).
