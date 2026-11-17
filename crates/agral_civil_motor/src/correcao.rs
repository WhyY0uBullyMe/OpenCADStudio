//! R18: corrigir curvas de nível que se cruzam. Objetivo do JP (17/11): "quero que ela não cruze, apenas isso".
//!
//! Método, repetido em rodadas até não sobrar cruzamento:
//! 1. **Lente:** entre dois cruzamentos seguidos das curvas A e B, as duas trocaram de lado. Nesse trecho as duas
//!    passam a seguir a linha do meio entre elas, cada uma deslocada `folga` para o lado que tinha fora da lente.
//!    Nenhuma das duas é tratada como a certa; fora da lente nada muda.
//! 2. **Corte:** o que não forma lente (ponta de curva que cruza uma vez, laço da curva nela mesma) ou o que ainda
//!    sobrar no fim das rodadas perde o trecho que cruza, na curva mais curta. Tirar trecho nunca cria cruzamento
//!    novo, então o resultado termina sempre sem cruzamento.
//!
//! As curvas de entrada não são alteradas; o resultado é uma lista nova (pedaços cortados viram curvas separadas).

use crate::diagnostico::{diagnosticar_curvas, Cruzamento};
use crate::P3;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct OpcoesCorrecao {
    /// Distância (m) entre as duas curvas no trecho separado.
    pub folga: f64,
    /// Rodadas de separação antes de cortar o que sobrar.
    pub rodadas: usize,
}

impl Default for OpcoesCorrecao {
    fn default() -> Self {
        OpcoesCorrecao { folga: 0.05, rodadas: 50 }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Correcao {
    pub curvas: Vec<Vec<P3>>,
    /// Índices (em `curvas`) das curvas que mudaram, para pôr na camada de revisão.
    pub alteradas: Vec<usize>,
    /// Curva de entrada de onde veio cada curva do resultado.
    pub origem: Vec<usize>,
    pub lentes_separadas: usize,
    pub trechos_cortados: usize,
    pub rodadas: usize,
    /// Cruzamentos que sobraram (deve ser 0).
    pub restantes: usize,
}

pub fn corrigir_cruzamentos(entrada: &[Vec<P3>], op: &OpcoesCorrecao) -> Correcao {
    let mut r = Correcao {
        curvas: entrada.to_vec(),
        origem: (0..entrada.len()).collect(),
        ..Default::default()
    };
    let mut mudou = vec![false; entrada.len()];
    let mut rodada = 0;
    loop {
        let diag = diagnosticar_curvas(&r.curvas);
        if diag.cruzamentos.is_empty() {
            break;
        }
        rodada += 1;
        let cortar_tudo = rodada > op.rodadas;
        let mut tocada = vec![false; r.curvas.len()];
        let mut progresso = false;

        if !cortar_tudo {
            let mut pares: BTreeMap<(usize, usize), Vec<&Cruzamento>> = BTreeMap::new();
            for c in diag.cruzamentos.iter().filter(|c| c.curva_a != c.curva_b) {
                pares.entry((c.curva_a, c.curva_b)).or_default().push(c);
            }
            for ((a, b), cs) in pares {
                if tocada[a] || tocada[b] || cs.len() < 2 {
                    continue;
                }
                if let Some((na, nb)) = separar_lente(&r.curvas[a], &r.curvas[b], &cs, op.folga) {
                    r.curvas[a] = na;
                    r.curvas[b] = nb;
                    tocada[a] = true;
                    tocada[b] = true;
                    mudou[a] = true;
                    mudou[b] = true;
                    r.lentes_separadas += 1;
                    progresso = true;
                }
            }
        }
        if !progresso {
            // Corte: um trecho por curva nesta rodada, na curva mais curta do cruzamento.
            let mut cortes: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
            for c in &diag.cruzamentos {
                let (k, t) = if c.curva_a == c.curva_b || comprimento(&r.curvas[c.curva_a]) <= comprimento(&r.curvas[c.curva_b]) {
                    (c.curva_a, c.trecho_a)
                } else {
                    (c.curva_b, c.trecho_b)
                };
                if !tocada[k] {
                    tocada[k] = true;
                    cortes.entry(k).or_default().push(t);
                }
            }
            // Corta de trás para frente para os índices das curvas não andarem.
            for (k, trechos) in cortes.into_iter().rev() {
                let t = trechos[0];
                let curva = std::mem::take(&mut r.curvas[k]);
                let (antes, depois) = (curva[..=t].to_vec(), curva[t + 1..].to_vec());
                let o = r.origem[k];
                r.curvas.remove(k);
                r.origem.remove(k);
                for (i, pedaco) in [antes, depois].into_iter().filter(|p| p.len() >= 2).enumerate() {
                    r.curvas.insert(k + i, pedaco);
                    r.origem.insert(k + i, o);
                }
                mudou[o] = true;
                r.trechos_cortados += 1;
            }
        }
        if rodada > op.rodadas + 100_000 {
            break;
        }
    }
    r.rodadas = rodada;
    r.restantes = diagnosticar_curvas(&r.curvas).cruzamentos.len();
    r.alteradas = (0..r.curvas.len()).filter(|&k| mudou[r.origem[k]]).collect();
    r
}

fn comprimento(c: &[P3]) -> f64 {
    c.windows(2).map(|w| dist(w[0], w[1])).sum()
}

fn dist(a: P3, b: P3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Posição ao longo da curva: índice do trecho mais a fração dentro dele.
fn posicao(curva: &[P3], trecho: usize, p: P3) -> f64 {
    let l = dist(curva[trecho], curva[trecho + 1]);
    trecho as f64 + if l > 0.0 { (dist(curva[trecho], p) / l).clamp(0.0, 1.0) } else { 0.0 }
}

/// Separa a primeira lente válida do par (A, B). Devolve as duas curvas novas.
fn separar_lente(a: &[P3], b: &[P3], cs: &[&Cruzamento], folga: f64) -> Option<(Vec<P3>, Vec<P3>)> {
    let mut pts: Vec<(f64, f64, P3)> = cs
        .iter()
        .map(|c| (posicao(a, c.trecho_a, c.ponto), posicao(b, c.trecho_b, c.ponto), c.ponto))
        .collect();
    pts.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
    for i in 0..pts.len() - 1 {
        let (pa1, pb1, c1) = pts[i];
        let (pa2, pb2, c2) = pts[i + 1];
        let (lo, hi) = (pb1.min(pb2), pb1.max(pb2));
        // Lente só se, ao longo de B, não há outro cruzamento do par entre os dois.
        if pts.iter().any(|q| q.1 > lo && q.1 < hi) || pa2 - pa1 < 1e-12 {
            continue;
        }
        let (ia1, ia2) = (pa1.floor() as usize, pa2.floor() as usize);
        // O ponto de cruzamento entra em cada curva com a cota dela.
        let (a1, a2) = (com_cota(a, pa1, c1), com_cota(a, pa2, c2));
        let (b1, b2) = (com_cota(b, pb1, c1), com_cota(b, pb2, c2));
        let trecho_a: Vec<P3> = std::iter::once(a1).chain(a[ia1 + 1..=ia2].iter().copied()).chain(std::iter::once(a2)).collect();
        let (ib_lo, ib_hi) = (lo.floor() as usize, hi.floor() as usize);
        let mut meio_b: Vec<P3> = b[ib_lo + 1..=ib_hi].to_vec();
        if pb1 > pb2 {
            meio_b.reverse();
        }
        let trecho_b: Vec<P3> = std::iter::once(b1).chain(meio_b).chain(std::iter::once(b2)).collect();
        let n = trecho_a.len().max(trecho_b.len()).max(3);
        let ra = reamostrar(&trecho_a, n);
        let rb = reamostrar(&trecho_b, n);
        let meio: Vec<[f64; 2]> = (0..n).map(|i| [(ra[i][0] + rb[i][0]) / 2.0, (ra[i][1] + rb[i][1]) / 2.0]).collect();
        // Lado de A fora da lente: o vértice de A antes de c1 (ou depois de c2), em relação à linha do meio.
        let t0 = [meio[1][0] - meio[0][0], meio[1][1] - meio[0][1]];
        let fora = if dist(a[ia1], c1) > 1e-9 { a[ia1] } else { *a.get(ia2 + 1)? };
        let lado = (t0[0] * (fora[1] - c1[1]) - t0[1] * (fora[0] - c1[0])).signum();
        if lado == 0.0 {
            continue;
        }
        let mut na_lente_a = Vec::with_capacity(n);
        let mut na_lente_b = Vec::with_capacity(n);
        for i in 0..n {
            let (p, q) = (meio[i.saturating_sub(1)], meio[(i + 1).min(n - 1)]);
            let (tx, ty) = (q[0] - p[0], q[1] - p[1]);
            let l = (tx * tx + ty * ty).sqrt().max(1e-12);
            // Normal à esquerda da linha do meio.
            let (nx, ny) = (-ty / l, tx / l);
            let d = lado * folga / 2.0;
            na_lente_a.push([meio[i][0] + d * nx, meio[i][1] + d * ny, ra[i][2]]);
            na_lente_b.push([meio[i][0] - d * nx, meio[i][1] - d * ny, rb[i][2]]);
        }
        let nova_a: Vec<P3> = a[..=ia1].iter().copied().chain(na_lente_a).chain(a[ia2 + 1..].iter().copied()).collect();
        let mut lente_b = na_lente_b;
        if pb1 > pb2 {
            lente_b.reverse();
        }
        let nova_b: Vec<P3> = b[..=ib_lo].iter().copied().chain(lente_b).chain(b[ib_hi + 1..].iter().copied()).collect();
        return Some((nova_a, nova_b));
    }
    None
}

fn com_cota(curva: &[P3], pos: f64, c: P3) -> P3 {
    let k = (pos.floor() as usize).min(curva.len() - 2);
    let t = pos - k as f64;
    [c[0], c[1], curva[k][2] + t * (curva[k + 1][2] - curva[k][2])]
}

/// `n` pontos igualmente espaçados ao longo da polilinha (primeiro e último iguais aos dela).
fn reamostrar(p: &[P3], n: usize) -> Vec<P3> {
    let total = comprimento(p);
    if total <= 0.0 {
        return vec![p[0]; n];
    }
    let mut saida = Vec::with_capacity(n);
    let (mut k, mut acum) = (0usize, 0.0);
    for i in 0..n {
        let alvo = total * i as f64 / (n - 1) as f64;
        while k + 1 < p.len() - 1 && acum + dist(p[k], p[k + 1]) < alvo {
            acum += dist(p[k], p[k + 1]);
            k += 1;
        }
        let l = dist(p[k], p[k + 1]);
        let t = if l > 0.0 { ((alvo - acum) / l).clamp(0.0, 1.0) } else { 0.0 };
        saida.push([
            p[k][0] + t * (p[k + 1][0] - p[k][0]),
            p[k][1] + t * (p[k + 1][1] - p[k][1]),
            p[k][2] + t * (p[k + 1][2] - p[k][2]),
        ]);
    }
    saida
}
