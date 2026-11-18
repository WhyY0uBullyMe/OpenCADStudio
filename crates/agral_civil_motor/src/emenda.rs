//! R19: emendar pedaços de curva de nível. Pedido do JP (17/11): hoje a equipe emenda à mão, "coerente com a
//! superfície"; folga máxima combinada: 5 m (ajustável).
//!
//! Regras:
//! - só pedaços abertos e da mesma cota;
//! - a folga entre as pontas tem de ser até `folga_maxima`;
//! - a emenda continua a direção das duas pontas (não faz grampo): o ângulo entre a direção de saída de cada ponta e
//!   a linha da emenda tem de ser menor que 75°;
//! - a emenda não pode cruzar nenhuma curva;
//! - ponta a menos de 1 m do limite da área não é emendada (ali a curva sai da área, não é buraco);
//! - pedaço cujas duas pontas estão perto uma da outra é fechado.
//! As emendas mais curtas são feitas primeiro.

use crate::correcao::GradeTrechos;
use crate::P3;

#[derive(Clone, Debug)]
pub struct OpcoesEmenda {
    pub folga_maxima: f64,
    /// Limite externo da área (polilinha fechada), se houver.
    pub limite: Option<Vec<P3>>,
}

impl Default for OpcoesEmenda {
    fn default() -> Self {
        OpcoesEmenda { folga_maxima: 5.0, limite: None }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Emenda {
    pub curvas: Vec<Vec<P3>>,
    pub emendas: usize,
    /// Das emendas, quantas fecharam um pedaço nele mesmo.
    pub fechadas: usize,
    /// Índices (em `curvas`) das curvas que receberam emenda.
    pub alteradas: Vec<usize>,
    /// Cada emenda feita: as duas pontas ligadas (para marcar no desenho e o engenheiro conferir).
    pub ligacoes: Vec<[P3; 2]>,
}

const ANGULO_MAXIMO_GRAUS: f64 = 75.0;
const DISTANCIA_DA_BORDA: f64 = 1.0;

fn d2(a: P3, b: P3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

fn aberta(c: &[P3]) -> bool {
    c.len() >= 2 && d2(c[0], c[c.len() - 1]) > 1e-3
}

/// Ponta `fim` (false = início) e a direção de saída por ela (unitária).
fn ponta(c: &[P3], fim: bool) -> (P3, [f64; 2]) {
    let (p, q) = if fim { (c[c.len() - 1], c[c.len() - 2]) } else { (c[0], c[1]) };
    let l = d2(p, q).max(1e-12);
    (p, [(p[0] - q[0]) / l, (p[1] - q[1]) / l])
}

fn perto_da_borda(p: P3, limite: &Option<Vec<P3>>) -> bool {
    let Some(l) = limite else { return false };
    l.windows(2).any(|w| {
        let (dx, dy) = (w[1][0] - w[0][0], w[1][1] - w[0][1]);
        let l2 = dx * dx + dy * dy;
        let t = if l2 > 0.0 { (((p[0] - w[0][0]) * dx + (p[1] - w[0][1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
        ((p[0] - w[0][0] - t * dx).powi(2) + (p[1] - w[0][1] - t * dy).powi(2)).sqrt() < DISTANCIA_DA_BORDA
    })
}

pub fn emendar_pedacos(entrada: &[Vec<P3>], op: &OpcoesEmenda) -> Emenda {
    let mut curvas: Vec<Vec<P3>> = entrada.to_vec();
    let mut mudou = vec![false; curvas.len()];
    let mut emendas = 0;
    let mut fechadas = 0;
    let mut ligacoes: Vec<[P3; 2]> = Vec::new();
    let cos_max = ANGULO_MAXIMO_GRAUS.to_radians().cos();
    loop {
        // Candidatas: (folga, curva i, ponta de i é o fim?, curva j, ponta de j é o fim?)
        let mut cand: Vec<(f64, usize, bool, usize, bool)> = Vec::new();
        let abertas: Vec<usize> = (0..curvas.len()).filter(|&k| aberta(&curvas[k])).collect();
        let mut por_cota: std::collections::HashMap<i64, Vec<usize>> = std::collections::HashMap::new();
        for &k in &abertas {
            por_cota.entry((curvas[k][0][2] * 1000.0).round() as i64).or_default().push(k);
        }
        for grupo in por_cota.values() {
            for (x, &i) in grupo.iter().enumerate() {
                for &j in &grupo[x..] {
                    for fi in [false, true] {
                        for fj in [false, true] {
                            if i == j && fi == fj {
                                continue;
                            }
                            if i == j && fi {
                                continue; // o par (início, fim) da mesma curva entra uma vez só
                            }
                            let (p, dp) = ponta(&curvas[i], fi);
                            let (q, dq) = ponta(&curvas[j], fj);
                            let g = d2(p, q);
                            if g > op.folga_maxima || g < 1e-9 {
                                continue;
                            }
                            if i == j && comprimento(&curvas[i]) < 3.0 * g {
                                continue;
                            }
                            let v = [(q[0] - p[0]) / g, (q[1] - p[1]) / g];
                            // Saída de p na direção de q, e saída de q na direção de p.
                            if dp[0] * v[0] + dp[1] * v[1] < cos_max || -(dq[0] * v[0] + dq[1] * v[1]) < cos_max {
                                continue;
                            }
                            if perto_da_borda(p, &op.limite) || perto_da_borda(q, &op.limite) {
                                continue;
                            }
                            cand.push((g, i, fi, j, fj));
                        }
                    }
                }
            }
        }
        if cand.is_empty() {
            break;
        }
        cand.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let grade = GradeTrechos::nova(&curvas);
        let mut usada = vec![false; curvas.len()];
        let mut feitas: Vec<(usize, bool, usize, bool)> = Vec::new();
        for (_, i, fi, j, fj) in cand {
            if usada[i] || usada[j] {
                continue;
            }
            let (p, _) = ponta(&curvas[i], fi);
            let (q, _) = ponta(&curvas[j], fj);
            if grade.cruza(&[p, q], 0..2, usize::MAX, &curvas) {
                continue;
            }
            usada[i] = true;
            usada[j] = true;
            ligacoes.push([p, q]);
            feitas.push((i, fi, j, fj));
        }
        if feitas.is_empty() {
            break;
        }
        let mut apagar = Vec::new();
        for (i, fi, j, fj) in feitas {
            if i == j {
                let p = curvas[i][0];
                curvas[i].push(p);
                fechadas += 1;
            } else {
                // Orienta: i termina na ponta da emenda, j começa nela.
                let mut a = std::mem::take(&mut curvas[i]);
                let mut b = std::mem::take(&mut curvas[j]);
                if !fi {
                    a.reverse();
                }
                if fj {
                    b.reverse();
                }
                a.extend(b);
                curvas[i] = a;
                mudou[j] = false;
                apagar.push(j);
            }
            mudou[i] = true;
            emendas += 1;
        }
        apagar.sort_unstable();
        for j in apagar.into_iter().rev() {
            curvas.remove(j);
            mudou.remove(j);
        }
    }
    let alteradas = (0..curvas.len()).filter(|&k| mudou[k]).collect();
    Emenda { curvas, emendas, fechadas, alteradas, ligacoes }
}

fn comprimento(c: &[P3]) -> f64 {
    c.windows(2).map(|w| d2(w[0], w[1])).sum()
}
