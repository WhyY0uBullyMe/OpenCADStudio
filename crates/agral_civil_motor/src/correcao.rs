//! R18: corrigir curvas de nível que se cruzam. Objetivo do JP (17/11): "quero que ela não cruze, apenas isso".
//!
//! Método, repetido em rodadas até não sobrar cruzamento:
//! 0. **Barriga (regra do JP, 17/11):** entre dois cruzamentos seguidos, uma das curvas faz uma barriga para dentro
//!    da outra (em geral a de cima entrando na de baixo). A curva que mais se afasta do próprio caminho é a da
//!    barriga: o trecho é cortado e refeito seguindo a direção que a curva tinha antes e depois dele (sem canto).
//!    A outra curva não muda. Se a curva refeita ainda cruzar a outra, cai no passo 1.
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
    pub barrigas_cortadas: usize,
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
    let mut grade: Option<GradeTrechos> = None;
    loop {
        let diag = diagnosticar_curvas(&r.curvas);
        if diag.cruzamentos.is_empty() {
            break;
        }
        rodada += 1;
        let cortar_tudo = rodada > op.rodadas;
        let mut tocada = vec![false; r.curvas.len()];
        let mut progresso = false;
        // Índice de todas as curvas, refeito de 10 em 10 rodadas (um índice velho só deixa passar cruzamento que a
        // rodada seguinte acha de novo).
        if grade.is_none() || rodada % 10 == 1 {
            grade = Some(GradeTrechos::nova(&r.curvas));
        }
        let indice = grade.as_ref().unwrap();

        if !cortar_tudo {
            let mut pares: BTreeMap<(usize, usize), Vec<&Cruzamento>> = BTreeMap::new();
            for c in diag.cruzamentos.iter().filter(|c| c.curva_a != c.curva_b) {
                pares.entry((c.curva_a, c.curva_b)).or_default().push(c);
            }
            for ((a, b), cs) in pares {
                if tocada[a] || tocada[b] || cs.len() < 2 {
                    continue;
                }
                match separar_lente(&r.curvas, a, b, &cs, op.folga, indice) {
                    Some(Separacao::Barriga { em_a, curva }) => {
                        let k = if em_a { a } else { b };
                        r.curvas[k] = curva;
                        mudou[r.origem[k]] = true;
                        r.barrigas_cortadas += 1;
                    }
                    Some(Separacao::Lente(na, nb)) => {
                        r.curvas[a] = na;
                        r.curvas[b] = nb;
                        mudou[r.origem[a]] = true;
                        mudou[r.origem[b]] = true;
                        r.lentes_separadas += 1;
                    }
                    None => continue,
                }
                tocada[a] = true;
                tocada[b] = true;
                progresso = true;
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
                // Cortar muda os índices das curvas: o índice precisa ser refeito.
                grade = None;
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

enum Separacao {
    /// Só uma curva muda (a da barriga).
    Barriga { em_a: bool, curva: Vec<P3> },
    /// As duas mudam (linha do meio).
    Lente(Vec<P3>, Vec<P3>),
}

/// Refaz o trecho da curva entre as posições `lo` e `hi` (o da lente) por uma curva de Hermite que liga o vértice
/// antes ao vértice depois, com a direção que a curva tinha fora dele. Devolve a curva nova e o quanto o trecho
/// antigo se afastava dela (o tamanho da barriga).
fn refazer_trecho(c: &[P3], lo: f64, hi: f64) -> Option<(Vec<P3>, std::ops::Range<usize>, f64)> {
    let (i0, i1) = (lo.floor() as usize, hi.floor() as usize + 1);
    if i1 >= c.len() {
        return None;
    }
    let (p0, p1) = (c[i0], c[i1]);
    let corda = dist(p0, p1);
    if corda <= 1e-9 {
        return None;
    }
    let unit = |a: P3, b: P3| {
        let l = dist(a, b).max(1e-12);
        [(b[0] - a[0]) / l, (b[1] - a[1]) / l]
    };
    let t0 = if i0 > 0 { unit(c[i0 - 1], p0) } else { unit(p0, p1) };
    let t1 = if i1 + 1 < c.len() { unit(p1, c[i1 + 1]) } else { unit(p0, p1) };
    let n = (i1 - i0 + 1).max(4);
    let novo: Vec<P3> = (0..n)
        .map(|k| {
            let t = k as f64 / (n - 1) as f64;
            let (t2, t3) = (t * t, t * t * t);
            let (h00, h10, h01, h11) = (2.0 * t3 - 3.0 * t2 + 1.0, t3 - 2.0 * t2 + t, -2.0 * t3 + 3.0 * t2, t3 - t2);
            [
                h00 * p0[0] + h10 * corda * t0[0] + h01 * p1[0] + h11 * corda * t1[0],
                h00 * p0[1] + h10 * corda * t0[1] + h01 * p1[1] + h11 * corda * t1[1],
                p0[2] + t * (p1[2] - p0[2]),
            ]
        })
        .collect();
    let desvio = c[i0 + 1..i1]
        .iter()
        .map(|p| novo.windows(2).map(|w| dist_trecho(*p, w[0], w[1])).fold(f64::INFINITY, f64::min))
        .fold(0.0, f64::max);
    let curva: Vec<P3> = c[..i0].iter().copied().chain(novo).chain(c[i1 + 1..].iter().copied()).collect();
    Some((curva, i0..i0 + n, desvio))
}

fn dist_trecho(p: P3, a: P3, b: P3) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((p[0] - a[0] - t * dx).powi(2) + (p[1] - a[1] - t * dy).powi(2)).sqrt()
}

/// Grade com os trechos de todas as curvas, para conferir se um trecho refeito cruza alguma delas.
struct GradeTrechos {
    x0: f64,
    y0: f64,
    lx: f64,
    ly: f64,
    n: usize,
    celulas: Vec<Vec<(u32, u32)>>,
}

impl GradeTrechos {
    fn nova(curvas: &[Vec<P3>]) -> Self {
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        let mut total = 0usize;
        for c in curvas {
            total += c.len();
            for p in c {
                x0 = x0.min(p[0]);
                y0 = y0.min(p[1]);
                x1 = x1.max(p[0]);
                y1 = y1.max(p[1]);
            }
        }
        let n = ((total as f64).sqrt() as usize).clamp(1, 2048);
        let mut g = GradeTrechos {
            x0,
            y0,
            lx: ((x1 - x0) / n as f64).max(1e-9),
            ly: ((y1 - y0) / n as f64).max(1e-9),
            n,
            celulas: vec![Vec::new(); n * n],
        };
        for (k, c) in curvas.iter().enumerate() {
            for (i, w) in c.windows(2).enumerate() {
                let ((i0, j0), (i1, j1)) = g.caixa(w[0], w[1]);
                for jj in j0..=j1 {
                    for ii in i0..=i1 {
                        g.celulas[jj * n + ii].push((k as u32, i as u32));
                    }
                }
            }
        }
        g
    }

    fn cel(&self, x: f64, y: f64) -> (usize, usize) {
        (
            (((x - self.x0) / self.lx).floor().max(0.0) as usize).min(self.n - 1),
            (((y - self.y0) / self.ly).floor().max(0.0) as usize).min(self.n - 1),
        )
    }

    fn caixa(&self, a: P3, b: P3) -> ((usize, usize), (usize, usize)) {
        (self.cel(a[0].min(b[0]), a[1].min(b[1])), self.cel(a[0].max(b[0]), a[1].max(b[1])))
    }

    /// O trecho `faixa` da curva nova (que substitui a curva `k`) cruza alguma curva?
    /// O índice só diz quais curvas estão perto; o teste usa o desenho atual delas (pode ter mudado nesta rodada).
    fn cruza(&self, nova: &[P3], faixa: std::ops::Range<usize>, k: usize, curvas: &[Vec<P3>]) -> bool {
        let ini = faixa.start.saturating_sub(1);
        let fim = (faixa.end + 1).min(nova.len());
        let trecho = &nova[ini..fim];
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for p in trecho {
            x0 = x0.min(p[0]);
            y0 = y0.min(p[1]);
            x1 = x1.max(p[0]);
            y1 = y1.max(p[1]);
        }
        let ((i0, j0), (i1, j1)) = self.caixa([x0, y0, 0.0], [x1, y1, 0.0]);
        let mut perto: Vec<u32> = Vec::new();
        for jj in j0..=j1 {
            for ii in i0..=i1 {
                perto.extend(self.celulas[jj * self.n + ii].iter().map(|&(c, _)| c));
            }
        }
        perto.sort_unstable();
        perto.dedup();
        let na_caixa = |u: &[P3]| u[0][0].max(u[1][0]) >= x0 && u[0][0].min(u[1][0]) <= x1 && u[0][1].max(u[1][1]) >= y0 && u[0][1].min(u[1][1]) <= y1;
        let mut outros: Vec<[P3; 2]> = Vec::new();
        for c in perto {
            if c as usize == k {
                continue;
            }
            if let Some(u) = curvas.get(c as usize) {
                outros.extend(u.windows(2).filter(|w| na_caixa(w)).map(|w| [w[0], w[1]]));
            }
        }
        // A própria curva nova fora do trecho (laço consigo mesma).
        outros.extend(nova[..ini + 1].windows(2).chain(nova[fim - 1..].windows(2)).filter(|w| na_caixa(w)).map(|w| [w[0], w[1]]));
        trecho
            .windows(2)
            .any(|w| outros.iter().any(|u| crate::tin::segmentos_se_cruzam(w[0], w[1], u[0], u[1])))
    }
}

/// Separa a primeira lente válida do par (A, B).
fn separar_lente(
    curvas: &[Vec<P3>],
    ia: usize,
    ib: usize,
    cs: &[&Cruzamento],
    folga: f64,
    grade: &GradeTrechos,
) -> Option<Separacao> {
    let (a, b) = (&curvas[ia][..], &curvas[ib][..]);
    let mut pts: Vec<(f64, f64, P3)> = cs
        .iter()
        .map(|c| (posicao(a, c.trecho_a, c.ponto), posicao(b, c.trecho_b, c.ponto), c.ponto))
        .collect();
    pts.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
    // 0. Barriga: do cruzamento i ao j (j - i ímpar: a curva sai e volta para o seu lado), refazer a curva que mais
    // se afasta do próprio caminho; empate fica com a de cima. Tenta primeiro a barriga curta, depois as maiores,
    // porque a curva costuma entrar e sair da vizinha várias vezes seguidas.
    let cota = |em_a: bool| if em_a { a[0][2] } else { b[0][2] };
    for i in 0..pts.len() - 1 {
        // Barriga curta: um cruzamento de entrada e um de saída. Barriga longa refeita costuma invadir uma terceira
        // curva (medido no CURVAS_V1, 17/11), então o resto vai para a linha do meio.
        for j in (i + 1..pts.len().min(i + 2)).step_by(2) {
            let (pa1, pa2) = (pts[i].0, pts[j].0);
            let lo = pts[i..=j].iter().map(|q| q.1).fold(f64::INFINITY, f64::min);
            let hi = pts[i..=j].iter().map(|q| q.1).fold(f64::NEG_INFINITY, f64::max);
            let fora = pts.iter().enumerate().any(|(k, q)| (k < i || k > j) && q.1 > lo && q.1 < hi);
            if fora || pa2 - pa1 < 1e-12 {
                continue;
            }
            let mut opcoes: Vec<(bool, (Vec<P3>, std::ops::Range<usize>, f64))> = Vec::new();
            if let Some(x) = refazer_trecho(a, pa1, pa2) {
                opcoes.push((true, x));
            }
            if let Some(x) = refazer_trecho(b, lo, hi) {
                opcoes.push((false, x));
            }
            opcoes.sort_by(|x, y| {
                if (y.1 .2 - x.1 .2).abs() > 1e-6 {
                    y.1 .2.partial_cmp(&x.1 .2).unwrap()
                } else {
                    cota(y.0).partial_cmp(&cota(x.0)).unwrap()
                }
            });
            if let Some((em_a, (curva, faixa, _))) = opcoes.into_iter().next() {
                // O trecho refeito não pode cruzar nenhuma curva (nem a vizinha, nem uma terceira).
                let k = if em_a { ia } else { ib };
                if !grade.cruza(&curva, faixa, k, curvas) {
                    return Some(Separacao::Barriga { em_a, curva });
                }
            }
        }
    }
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
        return Some(Separacao::Lente(nova_a, nova_b));
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
