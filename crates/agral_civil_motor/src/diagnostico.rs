//! R17: diagnóstico das curvas de nível antes de montar a superfície.
//!
//! Acha curvas que se cruzam (de cotas diferentes ou a mesma curva cruzando ela mesma), com o ponto do cruzamento,
//! e curvas que não são planas. Curva de nível cruzando outra é erro do dado: a superfície não pode passar pelas
//! duas cotas no mesmo ponto.

use crate::tin::segmentos_se_cruzam;
use crate::P3;

#[derive(Clone, Debug, PartialEq)]
pub struct Cruzamento {
    /// Índices das curvas na lista de entrada (`curva_a <= curva_b`).
    pub curva_a: usize,
    pub curva_b: usize,
    /// Índice do trecho (entre o vértice `trecho` e o seguinte) em cada curva.
    pub trecho_a: usize,
    pub trecho_b: usize,
    pub cota_a: f64,
    pub cota_b: f64,
    /// Ponto do cruzamento em planta (z = média das duas cotas, só para desenhar a marca).
    pub ponto: P3,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Diagnostico {
    pub cruzamentos: Vec<Cruzamento>,
    /// Curvas com vértices de cotas diferentes (curva de nível deveria ser plana).
    pub curvas_nao_planas: Vec<usize>,
}

pub fn diagnosticar_curvas(curvas: &[Vec<P3>]) -> Diagnostico {
    let mut d = Diagnostico::default();
    for (k, c) in curvas.iter().enumerate() {
        if c.iter().any(|p| (p[2] - c[0][2]).abs() > 1e-6) {
            d.curvas_nao_planas.push(k);
        }
    }
    // Trechos: (curva, índice do trecho, a, b).
    let trechos: Vec<(usize, usize, P3, P3)> = curvas
        .iter()
        .enumerate()
        .flat_map(|(k, c)| c.windows(2).enumerate().map(move |(i, w)| (k, i, w[0], w[1])))
        .filter(|t| t.2[0] != t.3[0] || t.2[1] != t.3[1])
        .collect();
    if trechos.len() < 2 {
        return d;
    }
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for t in &trechos {
        for p in [t.2, t.3] {
            x0 = x0.min(p[0]);
            y0 = y0.min(p[1]);
            x1 = x1.max(p[0]);
            y1 = y1.max(p[1]);
        }
    }
    let n = ((trechos.len() as f64).sqrt() as usize).clamp(1, 2048);
    let (lx, ly) = (((x1 - x0) / n as f64).max(1e-9), ((y1 - y0) / n as f64).max(1e-9));
    let cel = |x: f64, y: f64| {
        (
            (((x - x0) / lx).floor().max(0.0) as usize).min(n - 1),
            (((y - y0) / ly).floor().max(0.0) as usize).min(n - 1),
        )
    };
    let caixa = |a: P3, b: P3| (cel(a[0].min(b[0]), a[1].min(b[1])), cel(a[0].max(b[0]), a[1].max(b[1])));
    let mut grade: Vec<Vec<u32>> = vec![Vec::new(); n * n];
    for (k, t) in trechos.iter().enumerate() {
        let ((i0, j0), (i1, j1)) = caixa(t.2, t.3);
        for j in j0..=j1 {
            for i in i0..=i1 {
                grade[j * n + i].push(k as u32);
            }
        }
    }
    let mut candidatos: Vec<u32> = Vec::new();
    for (k, t) in trechos.iter().enumerate() {
        let ((i0, j0), (i1, j1)) = caixa(t.2, t.3);
        candidatos.clear();
        for j in j0..=j1 {
            for i in i0..=i1 {
                candidatos.extend(grade[j * n + i].iter().filter(|&&o| o as usize > k));
            }
        }
        candidatos.sort_unstable();
        candidatos.dedup();
        for &o in &candidatos {
            let u = &trechos[o as usize];
            if segmentos_se_cruzam(t.2, t.3, u.2, u.3) {
                let p = intersecao(t.2, t.3, u.2, u.3);
                d.cruzamentos.push(Cruzamento {
                    curva_a: t.0,
                    curva_b: u.0,
                    trecho_a: t.1,
                    trecho_b: u.1,
                    cota_a: t.2[2],
                    cota_b: u.2[2],
                    ponto: [p[0], p[1], (t.2[2] + u.2[2]) / 2.0],
                });
            }
        }
    }
    d.cruzamentos.sort_by(|a, b| (a.curva_a, a.curva_b, a.trecho_a).cmp(&(b.curva_a, b.curva_b, b.trecho_a)));
    d
}

fn intersecao(a: P3, b: P3, c: P3, d: P3) -> [f64; 2] {
    let (r, s) = ([b[0] - a[0], b[1] - a[1]], [d[0] - c[0], d[1] - c[1]]);
    let den = r[0] * s[1] - r[1] * s[0];
    let t = ((c[0] - a[0]) * s[1] - (c[1] - a[1]) * s[0]) / den;
    [a[0] + t * r[0], a[1] + t * r[1]]
}
