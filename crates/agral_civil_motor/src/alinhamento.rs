//! Spec 003 R4 a R6 e seções: alinhamento horizontal (retas e arcos), estacas, perfil do terreno, greide com
//! curva vertical e seção transversal da superfície. Só geometria e cálculo; o desenho fica no programa.
//!
//! Convenções: estaca em metros ao longo do eixo, a partir de `estaca_inicial`; deslocamento (offset) positivo à
//! direita de quem anda no sentido do eixo; ângulos em radianos, anti-horário a partir do Este.

use crate::tin::Tin;

/// Um trecho do eixo.
#[derive(Clone, Debug, PartialEq)]
pub enum Elemento {
    Reta { a: [f64; 2], b: [f64; 2] },
    /// Arco do ponto `a` ao `b` com centro `c`; `anti` = anti-horário.
    Arco { a: [f64; 2], b: [f64; 2], c: [f64; 2], raio: f64, anti: bool },
}

impl Elemento {
    pub fn comprimento(&self) -> f64 {
        match self {
            Elemento::Reta { a, b } => dist(*a, *b),
            Elemento::Arco { raio, .. } => raio * self.varredura().abs(),
        }
    }

    /// Ângulo varrido do arco (com sinal: positivo anti-horário).
    fn varredura(&self) -> f64 {
        match self {
            Elemento::Reta { .. } => 0.0,
            Elemento::Arco { a, b, c, anti, .. } => {
                let t0 = (a[1] - c[1]).atan2(a[0] - c[0]);
                let t1 = (b[1] - c[1]).atan2(b[0] - c[0]);
                let mut d = t1 - t0;
                if *anti {
                    while d <= 0.0 {
                        d += std::f64::consts::TAU;
                    }
                } else {
                    while d >= 0.0 {
                        d -= std::f64::consts::TAU;
                    }
                }
                d
            }
        }
    }

    /// Ponto e direção (ângulo da tangente) a `s` metros do início do trecho.
    fn ponto(&self, s: f64) -> ([f64; 2], f64) {
        match self {
            Elemento::Reta { a, b } => {
                let l = dist(*a, *b).max(1e-12);
                let t = (s / l).clamp(0.0, 1.0);
                ([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t], (b[1] - a[1]).atan2(b[0] - a[0]))
            }
            Elemento::Arco { a, c, raio, anti, .. } => {
                let t0 = (a[1] - c[1]).atan2(a[0] - c[0]);
                let v = self.varredura();
                let f = if v.abs() < 1e-15 { 0.0 } else { (s / (raio * v.abs())).clamp(0.0, 1.0) };
                let t = t0 + v * f;
                let p = [c[0] + raio * t.cos(), c[1] + raio * t.sin()];
                let dir = if *anti { t + std::f64::consts::FRAC_PI_2 } else { t - std::f64::consts::FRAC_PI_2 };
                (p, dir)
            }
        }
    }

    /// Distância ao longo do trecho do pé da perpendicular de `p`, e o deslocamento (direita positiva).
    fn projetar(&self, p: [f64; 2]) -> (f64, f64) {
        match self {
            Elemento::Reta { a, b } => {
                let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                let l2 = (dx * dx + dy * dy).max(1e-24);
                let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0);
                let l = l2.sqrt();
                // Direita de quem anda de a para b: produto vetorial negativo.
                let cruz = dx * (p[1] - a[1]) - dy * (p[0] - a[0]);
                (t * l, -cruz / l)
            }
            Elemento::Arco { a, c, raio, anti, .. } => {
                let t0 = (a[1] - c[1]).atan2(a[0] - c[0]);
                let tp = (p[1] - c[1]).atan2(p[0] - c[0]);
                let v = self.varredura();
                let mut d = tp - t0;
                // Ângulo do ponto medido no sentido do arco, em [0, 2π).
                if v > 0.0 {
                    d = d.rem_euclid(std::f64::consts::TAU);
                } else {
                    d = (-d).rem_euclid(std::f64::consts::TAU);
                }
                let s = (d.min(v.abs())) * raio;
                let r = dist(p, *c);
                // Anti-horário: o centro fica à esquerda; afastar do centro é ir para a direita.
                let off = if *anti { r - raio } else { raio - r };
                (s, off)
            }
        }
    }
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (b[0] - a[0]).hypot(b[1] - a[1])
}

#[derive(Clone, Debug, PartialEq)]
pub struct Alinhamento {
    pub elementos: Vec<Elemento>,
    pub estaca_inicial: f64,
}

impl Alinhamento {
    /// A partir dos vértices de uma polilinha 2D com bulge (o "bulge" do DXF: tan(ângulo/4), positivo
    /// anti-horário) no segmento que sai de cada vértice.
    pub fn de_polilinha(vertices: &[([f64; 2], f64)], fechada: bool) -> Option<Self> {
        let n = vertices.len();
        if n < 2 {
            return None;
        }
        let mut el = Vec::new();
        let segs = if fechada { n } else { n - 1 };
        for k in 0..segs {
            let (a, bulge) = vertices[k];
            let b = vertices[(k + 1) % n].0;
            if dist(a, b) < 1e-9 {
                continue;
            }
            if bulge.abs() < 1e-12 {
                el.push(Elemento::Reta { a, b });
            } else {
                let theta = 4.0 * bulge.atan();
                let corda = dist(a, b);
                let raio = (corda / 2.0 / (theta / 2.0).sin()).abs();
                // Centro: do meio da corda, para o lado do giro, a distância raio·cos(θ/2).
                let m = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
                let (ux, uy) = ((b[0] - a[0]) / corda, (b[1] - a[1]) / corda);
                let h = raio * (theta / 2.0).cos() * if bulge > 0.0 { 1.0 } else { -1.0 };
                // Normal à esquerda da corda.
                let c = [m[0] - uy * h, m[1] + ux * h];
                el.push(Elemento::Arco { a, b, c, raio, anti: bulge > 0.0 });
            }
        }
        (!el.is_empty()).then_some(Alinhamento { elementos: el, estaca_inicial: 0.0 })
    }

    pub fn comprimento(&self) -> f64 {
        self.elementos.iter().map(Elemento::comprimento).sum()
    }

    pub fn estaca_final(&self) -> f64 {
        self.estaca_inicial + self.comprimento()
    }

    /// Ponto e direção na estaca (fora do eixo, prende no início ou no fim).
    pub fn ponto_na_estaca(&self, estaca: f64) -> ([f64; 2], f64) {
        let mut s = (estaca - self.estaca_inicial).max(0.0);
        for (k, e) in self.elementos.iter().enumerate() {
            let l = e.comprimento();
            if s <= l || k + 1 == self.elementos.len() {
                return e.ponto(s.min(l));
            }
            s -= l;
        }
        unreachable!("alinhamento sem elementos")
    }

    /// Ponto a `offset` metros à direita (negativo: esquerda) do eixo na estaca.
    pub fn ponto_com_offset(&self, estaca: f64, offset: f64) -> [f64; 2] {
        let (p, dir) = self.ponto_na_estaca(estaca);
        // Direita = direção girada -90°.
        [p[0] + offset * dir.sin(), p[1] - offset * dir.cos()]
    }

    /// Estaca e deslocamento do ponto (o trecho mais próximo).
    pub fn estaca_e_offset(&self, p: [f64; 2]) -> (f64, f64) {
        let mut melhor = (f64::INFINITY, 0.0, 0.0);
        let mut base = self.estaca_inicial;
        for e in &self.elementos {
            let (s, off) = e.projetar(p);
            let q = e.ponto(s).0;
            let d = dist(p, q);
            if d < melhor.0 {
                melhor = (d, base + s, off);
            }
            base += e.comprimento();
        }
        (melhor.1, melhor.2)
    }

    /// Estacas a cada `intervalo` m (múltiplos redondos), mais o início, o fim e os pontos de mudança (PC, PT).
    pub fn estacas(&self, intervalo: f64) -> Vec<f64> {
        let mut v = vec![self.estaca_inicial];
        if intervalo > 0.0 {
            let mut k = (self.estaca_inicial / intervalo).floor() + 1.0;
            while k * intervalo < self.estaca_final() - 1e-9 {
                v.push(k * intervalo);
                k += 1.0;
            }
        }
        let mut base = self.estaca_inicial;
        for e in &self.elementos {
            base += e.comprimento();
            v.push(base);
        }
        v.sort_by(|a, b| a.total_cmp(b));
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
        v
    }
}

/// Estaca no formato km+m do Civil 3D: 1234,5 m -> "1+234.50".
pub fn texto_da_estaca(estaca: f64) -> String {
    let neg = estaca < 0.0;
    let e = estaca.abs();
    let km = (e / 1000.0).floor();
    let m = e - km * 1000.0;
    format!("{}{}+{:06.2}", if neg { "-" } else { "" }, km as i64, m)
}

/// Perfil do terreno: (estaca, cota) ao longo do eixo, a cada `passo` m e nas estacas pedidas. Fora da superfície
/// a amostra fica de fora (o perfil tem buracos onde não há superfície).
pub fn perfil_do_terreno(tin: &Tin, al: &Alinhamento, passo: f64, extras: &[f64]) -> Vec<(f64, f64)> {
    let mut est: Vec<f64> = Vec::new();
    let passo = passo.max(0.01);
    let mut s = al.estaca_inicial;
    while s < al.estaca_final() {
        est.push(s);
        s += passo;
    }
    est.push(al.estaca_final());
    est.extend(extras.iter().copied().filter(|e| *e >= al.estaca_inicial && *e <= al.estaca_final()));
    est.sort_by(|a, b| a.total_cmp(b));
    est.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    est.into_iter()
        .filter_map(|e| {
            let (p, _) = al.ponto_na_estaca(e);
            tin.cota_em(p[0], p[1]).map(|z| (e, z))
        })
        .collect()
}

/// Ponto de inflexão vertical do greide: estaca, cota e comprimento da curva vertical (parábola; 0 = sem curva).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piv {
    pub estaca: f64,
    pub cota: f64,
    pub curva: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Greide {
    pub pivs: Vec<Piv>,
}

impl Greide {
    /// Greide em ordem de estaca; precisa de pelo menos 1 ponto (1 ponto = cota constante, como a crista).
    pub fn novo(mut pivs: Vec<Piv>) -> Option<Self> {
        pivs.sort_by(|a, b| a.estaca.total_cmp(&b.estaca));
        pivs.dedup_by(|a, b| (a.estaca - b.estaca).abs() < 1e-9);
        (!pivs.is_empty()).then_some(Greide { pivs })
    }

    fn rampa(&self, k: usize) -> f64 {
        let (a, b) = (self.pivs[k], self.pivs[k + 1]);
        (b.cota - a.cota) / (b.estaca - a.estaca)
    }

    /// Cota do greide na estaca; fora dos PIVs, segue a primeira e a última rampa.
    pub fn cota(&self, e: f64) -> f64 {
        let p = &self.pivs;
        if p.len() == 1 {
            return p[0].cota;
        }
        // Curva vertical num PIV interno: parábola simétrica de comprimento L entre as duas rampas.
        for k in 1..p.len() - 1 {
            let l = p[k].curva;
            if l > 0.0 && (e - p[k].estaca).abs() <= l / 2.0 {
                let (g1, g2) = (self.rampa(k - 1), self.rampa(k));
                let pvc = p[k].estaca - l / 2.0;
                let z_pvc = p[k].cota - g1 * l / 2.0;
                let x = e - pvc;
                return z_pvc + g1 * x + (g2 - g1) / (2.0 * l) * x * x;
            }
        }
        let k = match p.iter().position(|q| q.estaca > e) {
            Some(0) => 0,
            Some(k) => k - 1,
            None => p.len() - 2,
        };
        p[k].cota + self.rampa(k) * (e - p[k].estaca)
    }
}

/// Seção transversal da superfície na estaca: (offset, cota) de `-esquerda` a `+direita`, a cada `passo` m.
pub fn secao_do_terreno(tin: &Tin, al: &Alinhamento, estaca: f64, esquerda: f64, direita: f64, passo: f64) -> Vec<(f64, f64)> {
    let passo = passo.max(0.01);
    let mut v = Vec::new();
    let mut o = -esquerda.abs();
    while o < direita.abs() + 1e-9 {
        let p = al.ponto_com_offset(estaca, o);
        if let Some(z) = tin.cota_em(p[0], p[1]) {
            v.push((o, z));
        }
        o += passo;
    }
    v
}

#[cfg(test)]
mod testes {
    use super::*;

    fn quase(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn reta_arco_reta_tem_o_comprimento_da_polilinha() {
        // Reta de 100 m para leste, arco de 90° anti-horário de raio 50, reta de 100 m para norte.
        let v = vec![([0.0, 0.0], 0.0), ([100.0, 0.0], (std::f64::consts::FRAC_PI_2 / 4.0).tan()), ([150.0, 50.0], 0.0), ([150.0, 150.0], 0.0)];
        let al = Alinhamento::de_polilinha(&v, false).unwrap();
        let esperado = 100.0 + 50.0 * std::f64::consts::FRAC_PI_2 + 100.0;
        assert!(quase(al.comprimento(), esperado, 1e-6), "{}", al.comprimento());
        // Estacas: meio da primeira reta, meio do arco, fim.
        let (p, d) = al.ponto_na_estaca(50.0);
        assert!(quase(p[0], 50.0, 1e-9) && quase(p[1], 0.0, 1e-9) && quase(d, 0.0, 1e-9));
        let meio_arco = 100.0 + 25.0 * std::f64::consts::FRAC_PI_2;
        let (p, d) = al.ponto_na_estaca(meio_arco);
        let r = 50.0 / 2f64.sqrt();
        assert!(quase(p[0], 100.0 + r, 1e-6) && quase(p[1], 50.0 - r, 1e-6), "{p:?}");
        assert!(quase(d, std::f64::consts::FRAC_PI_4, 1e-9));
        let (p, _) = al.ponto_na_estaca(al.estaca_final());
        assert!(quase(p[0], 150.0, 1e-6) && quase(p[1], 150.0, 1e-6));
    }

    #[test]
    fn estaca_e_offset_de_volta() {
        let v = vec![([0.0, 0.0], 0.0), ([100.0, 0.0], (std::f64::consts::FRAC_PI_2 / 4.0).tan()), ([150.0, 50.0], 0.0), ([150.0, 150.0], 0.0)];
        let al = Alinhamento::de_polilinha(&v, false).unwrap();
        for (e, o) in [(30.0, 5.0), (30.0, -7.0), (120.0, 3.0), (140.0, -4.0), (250.0, 10.0)] {
            let p = al.ponto_com_offset(e, o);
            let (e2, o2) = al.estaca_e_offset(p);
            assert!(quase(e, e2, 1e-6) && quase(o, o2, 1e-6), "{e},{o} -> {e2},{o2}");
        }
        // Direita de quem anda para leste é o sul.
        let p = al.ponto_com_offset(30.0, 5.0);
        assert!(quase(p[1], -5.0, 1e-9));
    }

    #[test]
    fn estacas_a_cada_20_m_com_os_pontos_notaveis() {
        let al = Alinhamento::de_polilinha(&[([0.0, 0.0], 0.0), ([50.0, 0.0], 0.0)], false).unwrap();
        assert_eq!(al.estacas(20.0), vec![0.0, 20.0, 40.0, 50.0]);
        assert_eq!(texto_da_estaca(1234.5), "1+234.50");
        assert_eq!(texto_da_estaca(20.0), "0+020.00");
    }

    #[test]
    fn perfil_de_plano_inclinado_e_exato() {
        // Plano z = 100 + 0,05x - 0,02y num quadrado de 200 m.
        let z = |x: f64, y: f64| 100.0 + 0.05 * x - 0.02 * y;
        let v = vec![[0.0, 0.0, z(0.0, 0.0)], [200.0, 0.0, z(200.0, 0.0)], [200.0, 200.0, z(200.0, 200.0)], [0.0, 200.0, z(0.0, 200.0)]];
        let tin = Tin::novo(v, vec![[0, 1, 2], [0, 2, 3]]);
        let al = Alinhamento::de_polilinha(&[([10.0, 20.0], 0.0), ([180.0, 150.0], 0.0)], false).unwrap();
        let p = perfil_do_terreno(&tin, &al, 7.0, &al.estacas(20.0));
        assert!(p.len() > 20);
        for (e, zt) in p {
            let (q, _) = al.ponto_na_estaca(e);
            assert!(quase(zt, z(q[0], q[1]), 1e-6), "estaca {e}");
        }
        // Seção: na direita e na esquerda a cota segue o plano.
        for (o, zs) in secao_do_terreno(&tin, &al, 100.0, 15.0, 15.0, 1.0) {
            let q = al.ponto_com_offset(100.0, o);
            assert!(quase(zs, z(q[0], q[1]), 1e-6), "offset {o}");
        }
    }

    #[test]
    fn greide_com_curva_vertical() {
        // Rampa +2% até a estaca 100 (cota 102), depois -1%; curva de 40 m no PIV.
        let g = Greide::novo(vec![
            Piv { estaca: 0.0, cota: 100.0, curva: 0.0 },
            Piv { estaca: 100.0, cota: 102.0, curva: 40.0 },
            Piv { estaca: 200.0, cota: 101.0, curva: 0.0 },
        ])
        .unwrap();
        assert!(quase(g.cota(50.0), 101.0, 1e-9));
        assert!(quase(g.cota(150.0), 101.5, 1e-9));
        // Início e fim da curva: tangentes às rampas.
        assert!(quase(g.cota(80.0), 101.6, 1e-9));
        assert!(quase(g.cota(120.0), 101.8, 1e-9));
        // No PIV: cota da tangente menos a flecha e = (g2-g1)·L/8 = -0,03·40/8 = -0,15.
        assert!(quase(g.cota(100.0), 102.0 - 0.15, 1e-9));
        // Um ponto só: cota constante (crista).
        let c = Greide::novo(vec![Piv { estaca: 0.0, cota: 955.0, curva: 0.0 }]).unwrap();
        assert!(quase(c.cota(123.0), 955.0, 1e-12));
    }
}
