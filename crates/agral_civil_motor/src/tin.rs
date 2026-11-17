//! Superfície TIN montada: vértices, triângulos, cota num ponto e curvas de nível.

use crate::P3;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Tin {
    pub vertices: Vec<P3>,
    /// Índices em `vertices`, sentido anti-horário.
    pub triangulos: Vec<[u32; 3]>,
    indice: Option<Grade>,
}

/// Grade simples para achar o triângulo de um ponto sem varrer todos.
#[derive(Clone, Debug)]
struct Grade {
    x0: f64,
    y0: f64,
    lado: f64,
    nx: usize,
    ny: usize,
    celulas: Vec<Vec<u32>>,
}

impl Tin {
    pub fn novo(vertices: Vec<P3>, triangulos: Vec<[u32; 3]>) -> Self {
        let mut t = Tin { vertices, triangulos, indice: None };
        t.indice = t.montar_grade();
        t
    }

    fn montar_grade(&self) -> Option<Grade> {
        if self.triangulos.is_empty() {
            return None;
        }
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for v in &self.vertices {
            x0 = x0.min(v[0]);
            y0 = y0.min(v[1]);
            x1 = x1.max(v[0]);
            y1 = y1.max(v[1]);
        }
        let area = ((x1 - x0) * (y1 - y0)).max(1e-9);
        let lado = (2.0 * area / self.triangulos.len() as f64).sqrt().max(1e-6);
        let nx = (((x1 - x0) / lado) as usize + 1).min(4096);
        let ny = (((y1 - y0) / lado) as usize + 1).min(4096);
        let lado = ((x1 - x0) / nx as f64).max((y1 - y0) / ny as f64).max(1e-6);
        let mut celulas = vec![Vec::new(); nx * ny];
        for (k, t) in self.triangulos.iter().enumerate() {
            let p = t.map(|i| self.vertices[i as usize]);
            let cel = |v: f64, o: f64, n: usize| (((v - o) / lado) as usize).min(n - 1);
            let (ia, ib) = (cel(p.iter().map(|q| q[0]).fold(f64::MAX, f64::min), x0, nx), cel(p.iter().map(|q| q[0]).fold(f64::MIN, f64::max), x0, nx));
            let (ja, jb) = (cel(p.iter().map(|q| q[1]).fold(f64::MAX, f64::min), y0, ny), cel(p.iter().map(|q| q[1]).fold(f64::MIN, f64::max), y0, ny));
            for j in ja..=jb {
                for i in ia..=ib {
                    celulas[j * nx + i].push(k as u32);
                }
            }
        }
        Some(Grade { x0, y0, lado, nx, ny, celulas })
    }

    /// Cota interpolada no plano do triângulo que contém (x, y); `None` fora da superfície.
    pub fn cota_em(&self, x: f64, y: f64) -> Option<f64> {
        let g = self.indice.as_ref()?;
        let i = ((x - g.x0) / g.lado).floor();
        let j = ((y - g.y0) / g.lado).floor();
        if i < 0.0 || j < 0.0 || i as usize >= g.nx || j as usize >= g.ny {
            // Ponto na borda máxima cai uma célula fora por arredondamento.
            let (ii, jj) = (i.min(g.nx as f64 - 1.0), j.min(g.ny as f64 - 1.0));
            if ii < 0.0 || jj < 0.0 || i > g.nx as f64 || j > g.ny as f64 {
                return None;
            }
            return self.cota_na_celula(g, ii as usize, jj as usize, x, y);
        }
        self.cota_na_celula(g, i as usize, j as usize, x, y)
    }

    fn cota_na_celula(&self, g: &Grade, i: usize, j: usize, x: f64, y: f64) -> Option<f64> {
        for &k in &g.celulas[j * g.nx + i] {
            let [a, b, c] = self.triangulos[k as usize].map(|v| self.vertices[v as usize]);
            let d = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
            if d.abs() < 1e-18 {
                continue;
            }
            let l1 = ((b[1] - c[1]) * (x - c[0]) + (c[0] - b[0]) * (y - c[1])) / d;
            let l2 = ((c[1] - a[1]) * (x - c[0]) + (a[0] - c[0]) * (y - c[1])) / d;
            let l3 = 1.0 - l1 - l2;
            let e = -1e-9;
            if l1 >= e && l2 >= e && l3 >= e {
                return Some(l1 * a[2] + l2 * b[2] + l3 * c[2]);
            }
        }
        None
    }

    pub fn faixa_de_cotas(&self) -> Option<(f64, f64)> {
        let mut usados = self.triangulos.iter().flatten().map(|&i| self.vertices[i as usize][2]);
        let z0 = usados.next()?;
        Some(usados.fold((z0, z0), |(a, b), z| (a.min(z), b.max(z))))
    }

    /// Curvas de nível na cota `z`, como polilinhas (fechadas repetem o primeiro ponto).
    ///
    /// Vértice exatamente na cota é tratado como um pouco acima, para a curva não se partir nos triângulos planos
    /// (o caso comum quando a superfície vem de curvas de nível).
    pub fn curvas_de_nivel(&self, z: f64) -> Vec<Vec<P3>> {
        let acima = |i: u32| self.vertices[i as usize][2] >= z;
        // Cada triângulo cortado dá um segmento entre duas arestas. Chave da aresta: par de vértices ordenado.
        let mut segmentos: Vec<([u32; 2], [u32; 2])> = Vec::new();
        for t in &self.triangulos {
            let mut cortes = [[0u32; 2]; 2];
            let mut n = 0;
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                if acima(a) != acima(b) && n < 2 {
                    cortes[n] = [a.min(b), a.max(b)];
                    n += 1;
                }
            }
            if n == 2 {
                segmentos.push((cortes[0], cortes[1]));
            }
        }
        let ponto = |e: [u32; 2]| -> P3 {
            let (a, b) = (self.vertices[e[0] as usize], self.vertices[e[1] as usize]);
            let t = ((z - a[2]) / (b[2] - a[2])).clamp(0.0, 1.0);
            [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1]), z]
        };
        let mut por_aresta: HashMap<[u32; 2], Vec<usize>> = HashMap::new();
        for (k, (a, b)) in segmentos.iter().enumerate() {
            por_aresta.entry(*a).or_default().push(k);
            por_aresta.entry(*b).or_default().push(k);
        }
        let mut usado = vec![false; segmentos.len()];
        let mut curvas = Vec::new();
        for inicio in 0..segmentos.len() {
            if usado[inicio] {
                continue;
            }
            usado[inicio] = true;
            let (a, b) = segmentos[inicio];
            let mut cadeia = std::collections::VecDeque::from([a, b]);
            // Anda para a frente a partir de b, depois para trás a partir de a.
            for frente in [true, false] {
                loop {
                    let ponta = if frente { *cadeia.back().unwrap() } else { *cadeia.front().unwrap() };
                    let prox = por_aresta.get(&ponta).and_then(|v| v.iter().copied().find(|&s| !usado[s]));
                    let Some(s) = prox else { break };
                    usado[s] = true;
                    let (p, q) = segmentos[s];
                    let outra = if p == ponta { q } else { p };
                    if frente {
                        cadeia.push_back(outra);
                    } else {
                        cadeia.push_front(outra);
                    }
                }
            }
            curvas.push(cadeia.into_iter().map(ponto).collect());
        }
        curvas
    }
}

/// Cruzamento próprio de dois segmentos em planta (tocar na ponta não conta).
pub fn segmentos_se_cruzam(a: P3, b: P3, c: P3, d: P3) -> bool {
    let o = |p: P3, q: P3, r: P3| (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
    let tol = 1e-9 * (1.0 + (b[0] - a[0]).abs() + (b[1] - a[1]).abs()).powi(2);
    let (d1, d2, d3, d4) = (o(a, b, c), o(a, b, d), o(c, d, a), o(c, d, b));
    ((d1 > tol && d2 < -tol) || (d1 < -tol && d2 > tol)) && ((d3 > tol && d4 < -tol) || (d3 < -tol && d4 > tol))
}
