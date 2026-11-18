# Agral CAD: com que tecnologia construir (pergunta do JP, 17/11/2026)

## Recomendação

**Rust, como já está.** O Open CAD Studio, que é a base escolhida pelo JP, é todo em Rust (cerca de 1,1 milhão de
linhas, tela em `iced`, desenho na placa de vídeo com `wgpu`). O Agral Civil também está em Rust.

## Por quê (medido nesta sessão)

| O quê | Resultado |
|---|---|
| Ler o `CURVAS_V1.dwg` (61 MB, 1,35 milhão de vértices) | 1 s |
| Montar a superfície com 1,1 milhão de vértices | 2 s (2,25 milhões de triângulos) |
| 1 milhão de pontos sintéticos | 1,5 s |
| Corrigir 1.208 cruzamentos | 22 s (dá para acelerar) |
| Compilar o Open CAD Studio inteiro do zero | 6 min, sem erro |

- **Velocidade de C++ sem os travamentos típicos de C++:** o compilador barra a maioria dos erros de memória.
  Num CAD com milhões de triângulos isso pesa.
- **Um `.exe` só para Windows,** sem instalar Python nem outro programa no computador.
- **Mesma linguagem da base:** não há ponte entre linguagens, e as correções do Open CAD Studio entram fácil.
- O motor de civil fica numa biblioteca sem tela (`agral_civil_motor`), testada sozinha. Se o Hydrus precisar de
  superfície, ele pode chamar o mesmo motor (via PyO3), sem copiar fórmula.

## Custos

- Compilar é lento (6 min o programa inteiro; o motor sozinho, segundos).
- A tela `iced` é menos madura que Qt ou a web. Vale para o CAD; telas de formulário continuam melhores no Hydrus.
- O Open CAD Studio muda todo dia: por isso o nosso código fica no motor e no pacote `agral.civil`, mexendo o mínimo
  no núcleo, para continuar puxando as correções do autor.

## O que não recomendo

- **Reescrever o Hydrus em Rust:** ele é Python com telas web e funciona; reescrever é meses sem ganho para o
  usuário. Só a parte pesada (superfície, e no futuro dam break) vale ir para Rust.
- **Trocar a base do CAD:** nenhuma outra opção aberta tem 2D, 3D e DWG juntos (ver `brief.md` e `plano-fork.md`).
