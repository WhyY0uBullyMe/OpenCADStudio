//! R6, R7, R8, R13: superfície vazia com lista de itens de definição, refeita a partir da lista.

use crate::tin::Tin;
use crate::P3;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use std::collections::HashMap;

/// De onde veio o dado de relevo. Base pública nunca é apresentada como levantamento.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Origem {
    #[default]
    Levantamento,
    BasePublica,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TipoItem {
    /// Pontos soltos (arquivo, objetos do desenho, nuvem, DEM).
    Pontos(Vec<P3>),
    /// Curvas de nível: entram como linhas de quebra (R8).
    Curvas(Vec<Vec<P3>>),
    /// Linhas de quebra 3D (R13).
    LinhasDeQuebra(Vec<Vec<P3>>),
    /// Limite externo: polilinha fechada; só fica o que está dentro (R13).
    LimiteExterno(Vec<P3>),
    /// Furo: polilinha fechada; sai o que está dentro (R13).
    Furo(Vec<P3>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemDefinicao {
    /// Nome mostrado na árvore (arquivo, camada, seleção).
    pub fonte: String,
    pub tipo: TipoItem,
    pub ligado: bool,
    pub origem: Origem,
}

impl ItemDefinicao {
    fn novo(fonte: &str, tipo: TipoItem) -> Self {
        ItemDefinicao { fonte: fonte.to_string(), tipo, ligado: true, origem: Origem::Levantamento }
    }
    pub fn pontos(fonte: &str, p: Vec<P3>) -> Self {
        Self::novo(fonte, TipoItem::Pontos(p))
    }
    pub fn curvas(fonte: &str, c: Vec<Vec<P3>>) -> Self {
        Self::novo(fonte, TipoItem::Curvas(c))
    }
    pub fn linhas_de_quebra(fonte: &str, l: Vec<Vec<P3>>) -> Self {
        Self::novo(fonte, TipoItem::LinhasDeQuebra(l))
    }
    pub fn limite_externo(fonte: &str, p: Vec<P3>) -> Self {
        Self::novo(fonte, TipoItem::LimiteExterno(p))
    }
    pub fn furo(fonte: &str, p: Vec<P3>) -> Self {
        Self::novo(fonte, TipoItem::Furo(p))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OpcoesSuperficie {
    /// Comprimento máximo de aresta na borda (m). Triângulo de borda mais comprido é tirado, de fora para dentro.
    pub comprimento_maximo: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct Superficie {
    pub nome: String,
    pub descricao: String,
    pub definicao: Vec<ItemDefinicao>,
    pub opcoes: OpcoesSuperficie,
    versao: u64,
    versao_montada: u64,
    pub tin: Tin,
}

#[derive(Clone, Debug, Default)]
pub struct Montagem {
    pub tin: Tin,
    /// Problemas achados nos dados (curvas cruzadas, pontos repetidos com cotas diferentes).
    pub avisos: Vec<String>,
    /// Trechos de curva ou linha de quebra que ficaram de fora por cruzar outros (para mostrar no desenho).
    pub trechos_em_conflito: Vec<[P3; 2]>,
}

impl Superficie {
    pub fn nova(nome: &str, descricao: &str) -> Self {
        Superficie {
            nome: nome.to_string(),
            descricao: descricao.to_string(),
            definicao: Vec::new(),
            opcoes: OpcoesSuperficie::default(),
            versao: 0,
            versao_montada: 0,
            tin: Tin::default(),
        }
    }

    pub fn acrescentar(&mut self, item: ItemDefinicao) {
        self.definicao.push(item);
        self.versao += 1;
    }

    pub fn remover(&mut self, k: usize) -> ItemDefinicao {
        self.versao += 1;
        self.definicao.remove(k)
    }

    /// Avisar que a definição mudou por fora (ligar/desligar item, trocar opção).
    pub fn mudou(&mut self) {
        self.versao += 1;
    }

    pub fn desatualizada(&self) -> bool {
        self.versao != self.versao_montada
    }

    /// `RebuildSurface`: monta e guarda o resultado.
    pub fn refazer(&mut self) -> Vec<String> {
        let m = self.montar();
        self.tin = m.tin;
        self.versao_montada = self.versao;
        m.avisos
    }

    /// Monta o TIN a partir da lista de definição, sem mudar a superfície.
    pub fn montar(&self) -> Montagem {
        let mut avisos = Vec::new();
        let mut vertices = Vertices::default();
        let mut restricoes: Vec<[usize; 2]> = Vec::new();
        let mut limites: Vec<&[P3]> = Vec::new();
        let mut furos: Vec<&[P3]> = Vec::new();

        for item in self.definicao.iter().filter(|i| i.ligado) {
            match &item.tipo {
                TipoItem::Pontos(p) => p.iter().for_each(|&q| {
                    vertices.indice(q);
                }),
                TipoItem::Curvas(linhas) | TipoItem::LinhasDeQuebra(linhas) => {
                    for l in linhas {
                        let idx: Vec<usize> = l.iter().map(|&q| vertices.indice(q)).collect();
                        restricoes.extend(idx.windows(2).filter(|w| w[0] != w[1]).map(|w| [w[0], w[1]]));
                    }
                }
                TipoItem::LimiteExterno(p) => limites.push(p),
                TipoItem::Furo(p) => furos.push(p),
            }
        }
        if vertices.cotas_diferentes > 0 {
            avisos.push(format!(
                "{} pontos repetidos na mesma posição com cota diferente; ficou a primeira cota",
                vertices.cotas_diferentes
            ));
        }
        if vertices.xyz.len() < 3 {
            return Montagem { tin: Tin::default(), avisos, ..Default::default() };
        }

        // Coordenadas locais: UTM tem números grandes; tirar a origem preserva a precisão do Delaunay.
        let (ox, oy) = (vertices.xyz[0][0].floor(), vertices.xyz[0][1].floor());
        let locais: Vec<Point2<f64>> = vertices.xyz.iter().map(|v| Point2::new(v[0] - ox, v[1] - oy)).collect();
        let mut em_conflito: Vec<[usize; 2]> = Vec::new();
        let cdt = match ConstrainedDelaunayTriangulation::<Point2<f64>>::try_bulk_load_cdt(
            locais,
            restricoes,
            |e| em_conflito.push(e),
        ) {
            Ok(c) => c,
            Err(e) => {
                avisos.push(format!("não foi possível triangular: {e:?}"));
                return Montagem { tin: Tin::default(), avisos, ..Default::default() };
            }
        };
        let conflitos = em_conflito.len();
        if conflitos > 0 {
            avisos.push(format!(
                "{conflitos} trechos de curva ou linha de quebra cruzam outros e ficaram de fora; confira as curvas"
            ));
        }
        debug_assert_eq!(cdt.num_vertices(), vertices.xyz.len());

        let mut tri: Vec<[u32; 3]> =
            cdt.inner_faces().map(|f| f.vertices().map(|v| v.fix().index() as u32)).collect();
        let xyz = vertices.xyz;

        if let Some(max) = self.opcoes.comprimento_maximo {
            tri = tirar_borda_comprida(&xyz, tri, max);
        }
        if !limites.is_empty() || !furos.is_empty() {
            tri.retain(|t| {
                let c = centroide(&xyz, t);
                limites.iter().all(|l| dentro(c, l)) && !furos.iter().any(|f| dentro(c, f))
            });
        }
        let trechos_em_conflito = em_conflito.iter().map(|e| [xyz[e[0]], xyz[e[1]]]).collect();
        Montagem { tin: Tin::novo(xyz, tri), avisos, trechos_em_conflito }
    }
}

/// Junta pontos iguais em planta (até 1 mm), para os índices das curvas baterem com os vértices.
#[derive(Default)]
struct Vertices {
    xyz: Vec<P3>,
    mapa: HashMap<(i64, i64), usize>,
    cotas_diferentes: usize,
}

impl Vertices {
    fn indice(&mut self, p: P3) -> usize {
        let chave = ((p[0] * 1000.0).round() as i64, (p[1] * 1000.0).round() as i64);
        if let Some(&k) = self.mapa.get(&chave) {
            if (self.xyz[k][2] - p[2]).abs() > 1e-6 {
                self.cotas_diferentes += 1;
            }
            return k;
        }
        self.xyz.push(p);
        self.mapa.insert(chave, self.xyz.len() - 1);
        self.xyz.len() - 1
    }
}

fn centroide(v: &[P3], t: &[u32; 3]) -> [f64; 2] {
    let p = t.map(|i| v[i as usize]);
    [(p[0][0] + p[1][0] + p[2][0]) / 3.0, (p[0][1] + p[1][1] + p[2][1]) / 3.0]
}

/// Ponto dentro de polígono (par-ímpar).
fn dentro(p: [f64; 2], poly: &[P3]) -> bool {
    let mut d = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + n - 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] {
            d = !d;
        }
    }
    d
}

/// Tira, da borda para dentro, triângulos com aresta de borda maior que `max` (como o "Maximum triangle length").
fn tirar_borda_comprida(v: &[P3], mut tri: Vec<[u32; 3]>, max: f64) -> Vec<[u32; 3]> {
    let comp = |a: u32, b: u32| {
        let (p, q) = (v[a as usize], v[b as usize]);
        ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt()
    };
    let chave = |a: u32, b: u32| (a.min(b), a.max(b));
    loop {
        let mut uso: HashMap<(u32, u32), u32> = HashMap::new();
        for t in &tri {
            for k in 0..3 {
                *uso.entry(chave(t[k], t[(k + 1) % 3])).or_default() += 1;
            }
        }
        let antes = tri.len();
        tri.retain(|t| {
            !(0..3).any(|k| {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                uso[&chave(a, b)] == 1 && comp(a, b) > max
            })
        });
        if tri.len() == antes {
            return tri;
        }
    }
}
