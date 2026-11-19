//! Spec 003 R4 a R6 e seções: o alinhamento gravado no DWG, num XRECORD do dicionário `AGRAL_CIVIL/ALIGNMENTS`
//! (um por alinhamento). Guarda a geometria (copiada da polilinha, como o Civil 3D faz: depois a polilinha pode
//! ser apagada), as estacas, o greide, a vista de perfil e as seções, e o que foi desenhado para cada parte.
//!
//! Formato: pares (código 1 = chave, valores). Chave desconhecida é ignorada, então versões novas acrescentam
//! campos sem quebrar as antigas.

use agral_civil_motor::alinhamento::{Alinhamento, Greide, Piv};
use agral_civil_motor::corredor::AssemblyBarragem;
use document::objects::{XRecordEntry, XRecordValue};
use document::Handle;

pub const DICIONARIO: [&str; 2] = ["AGRAL_CIVIL", "ALIGNMENTS"];
const MARCA: &str = "AGRAL_ALIGNMENT";

#[derive(Clone, Debug, PartialEq)]
pub struct VistaPerfil {
    /// Canto de baixo à esquerda da vista no desenho.
    pub origem: [f64; 2],
    pub exagero: f64,
    /// Cota da linha de baixo e da de cima da grade (calculadas ao desenhar; servem para ler cliques e a dica do
    /// mouse na vista).
    pub base: f64,
    pub topo: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Secoes {
    pub intervalo: f64,
    pub esquerda: f64,
    pub direita: f64,
    /// Canto de cima à esquerda da grade das vistas de seção.
    pub origem: [f64; 2],
    pub colunas: usize,
    /// Agral CAD: superfícies a mais mostradas nas vistas de seção (Sample More Sources), além da do eixo.
    pub fontes: Vec<String>,
    /// Agral CAD: estacas das sample lines escolhidas à mão (EDITSAMPLELINES); vazio = a cada intervalo.
    pub estacas: Vec<f64>,
}

impl Secoes {
    pub fn nova(intervalo: f64, esquerda: f64, direita: f64, origem: [f64; 2], colunas: usize) -> Self {
        Secoes { intervalo, esquerda, direita, origem, colunas, fontes: Vec::new(), estacas: Vec::new() }
    }

    /// Estacas das sample lines: as escolhidas à mão (dentro do eixo), ou a cada intervalo e nos pontos notáveis.
    pub fn estacas_no(&self, al: &agral_civil_motor::alinhamento::Alinhamento) -> Vec<f64> {
        if self.estacas.is_empty() {
            return al.estacas(self.intervalo);
        }
        let mut v: Vec<f64> = self.estacas.iter().copied().filter(|e| *e >= al.estaca_inicial - 1e-6 && *e <= al.estaca_final() + 1e-6).collect();
        v.sort_by(|a, b| a.total_cmp(b));
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
        v
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CriterioMateriais {
    pub base: String,
    pub comparar: String,
    pub fator_corte: f64,
    pub fator_aterro: f64,
    /// Canto de cima à esquerda da tabela de volumes desenhada, se houver.
    pub tabela: Option<[f64; 2]>,
    /// Totais do último cálculo (corte, aterro), m³, já com os fatores.
    pub totais: Option<(f64, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AlinhamentoDwg {
    pub nome: String,
    pub descricao: String,
    pub vertices: Vec<([f64; 2], f64)>,
    pub fechada: bool,
    pub estaca_inicial: f64,
    /// Intervalo das estacas rotuladas (padrão 20 m).
    pub intervalo: f64,
    /// Superfície do terreno para o perfil e as seções.
    pub superficie: String,
    pub pivs: Vec<Piv>,
    pub vista: Option<VistaPerfil>,
    pub secoes: Option<Secoes>,
    /// Assembly da barragem e o corredor (espaçamento das seções do corredor, m).
    pub assembly: Option<AssemblyBarragem>,
    pub corredor: Option<f64>,
    /// Volumes do último corredor montado (aterro, corte), m³.
    pub volumes: Option<(f64, f64)>,
    /// Critério do Compute Materials (spec 004): superfície base, de comparação e fatores de corte e aterro.
    pub materiais: Option<CriterioMateriais>,
    /// Tabela de volumes desenhada (bloco e referência).
    pub desenho_volumes: Vec<Handle>,
    /// Desenho da planta (eixo, estacas), da vista de perfil e das seções.
    pub desenho: Vec<Handle>,
    pub desenho_perfil: Vec<Handle>,
    pub desenho_secoes: Vec<Handle>,
    /// Linhas do corredor (polilinhas 3D no espaço do modelo, fontes da superfície do corredor).
    pub desenho_corredor: Vec<Handle>,
}

impl AlinhamentoDwg {
    pub fn novo(nome: &str, vertices: Vec<([f64; 2], f64)>, fechada: bool) -> Self {
        AlinhamentoDwg {
            nome: nome.into(),
            descricao: String::new(),
            vertices,
            fechada,
            estaca_inicial: 0.0,
            intervalo: 20.0,
            superficie: String::new(),
            pivs: Vec::new(),
            vista: None,
            secoes: None,
            assembly: None,
            corredor: None,
            volumes: None,
            materiais: None,
            desenho_volumes: Vec::new(),
            desenho: Vec::new(),
            desenho_perfil: Vec::new(),
            desenho_secoes: Vec::new(),
            desenho_corredor: Vec::new(),
        }
    }

    pub fn geometria(&self) -> Option<Alinhamento> {
        let mut a = Alinhamento::de_polilinha(&self.vertices, self.fechada)?;
        a.estaca_inicial = self.estaca_inicial;
        Some(a)
    }

    pub fn greide(&self) -> Option<Greide> {
        Greide::novo(self.pivs.clone())
    }

    pub fn para_xrecord(&self) -> Vec<XRecordEntry> {
        let mut e = vec![XRecordEntry::string(1, MARCA.to_string()), XRecordEntry::int16(70, 1)];
        let chave = |e: &mut Vec<XRecordEntry>, k: &str| e.push(XRecordEntry::string(1, k.to_string()));
        chave(&mut e, "name");
        e.push(XRecordEntry::string(300, self.nome.clone()));
        chave(&mut e, "desc");
        e.push(XRecordEntry::string(300, self.descricao.clone()));
        chave(&mut e, "geometry");
        e.push(XRecordEntry::int16(71, self.fechada as i16));
        e.push(XRecordEntry::new(90, XRecordValue::Int32(self.vertices.len() as i32)));
        for (p, b) in &self.vertices {
            e.push(XRecordEntry::new(10, XRecordValue::Point3D(p[0], p[1], 0.0)));
            e.push(XRecordEntry::double(42, *b));
        }
        chave(&mut e, "stations");
        e.push(XRecordEntry::double(40, self.estaca_inicial));
        e.push(XRecordEntry::double(41, self.intervalo));
        chave(&mut e, "surface");
        e.push(XRecordEntry::string(300, self.superficie.clone()));
        chave(&mut e, "pvis");
        e.push(XRecordEntry::new(90, XRecordValue::Int32(self.pivs.len() as i32)));
        for p in &self.pivs {
            e.push(XRecordEntry::new(11, XRecordValue::Point3D(p.estaca, p.cota, p.curva)));
        }
        if let Some(v) = &self.vista {
            chave(&mut e, "profileview");
            e.push(XRecordEntry::new(12, XRecordValue::Point3D(v.origem[0], v.origem[1], v.base)));
            e.push(XRecordEntry::double(40, v.exagero));
            e.push(XRecordEntry::double(41, v.topo));
        }
        if let Some(s) = &self.secoes {
            chave(&mut e, "sections");
            e.push(XRecordEntry::new(13, XRecordValue::Point3D(s.origem[0], s.origem[1], 0.0)));
            e.push(XRecordEntry::new(14, XRecordValue::Point3D(s.intervalo, s.esquerda, s.direita)));
            e.push(XRecordEntry::new(90, XRecordValue::Int32(s.colunas as i32)));
            if !s.fontes.is_empty() {
                e.push(XRecordEntry::new(91, XRecordValue::Int32(s.fontes.len() as i32)));
                e.extend(s.fontes.iter().map(|f| XRecordEntry::string(300, f.clone())));
            }
            if !s.estacas.is_empty() {
                e.push(XRecordEntry::new(92, XRecordValue::Int32(s.estacas.len() as i32)));
                e.extend(s.estacas.iter().map(|v| XRecordEntry::double(40, *v)));
            }
        }
        if let Some(a) = &self.assembly {
            chave(&mut e, "assembly");
            for v in [a.crista, a.decl_crista, if a.montante_esquerda { 1.0 } else { 0.0 }, a.talude_montante, a.talude_jusante, a.altura_berma, a.largura_berma, a.talude_corte, a.alcance] {
                e.push(XRecordEntry::double(40, v));
            }
        }
        if let Some(p) = self.corredor {
            chave(&mut e, "corridor");
            e.push(XRecordEntry::double(40, p));
            // Sem volumes: -1 (volume nunca é negativo). Antes era NaN, que o DXF escreve como texto inválido.
            let (va, vc) = self.volumes.unwrap_or((-1.0, -1.0));
            e.push(XRecordEntry::double(41, va));
            e.push(XRecordEntry::double(42, vc));
        }
        if let Some(m) = &self.materiais {
            chave(&mut e, "materials");
            e.push(XRecordEntry::string(300, m.base.clone()));
            e.push(XRecordEntry::string(300, m.comparar.clone()));
            e.push(XRecordEntry::double(40, m.fator_corte));
            e.push(XRecordEntry::double(41, m.fator_aterro));
            if let Some((c, a)) = m.totais {
                e.push(XRecordEntry::double(42, c));
                e.push(XRecordEntry::double(43, a));
            }
            if let Some(t) = m.tabela {
                e.push(XRecordEntry::new(15, XRecordValue::Point3D(t[0], t[1], 0.0)));
            }
        }
        for (k, hs) in [
            ("drawing_volumes", &self.desenho_volumes),
            ("drawing", &self.desenho),
            ("drawing_profile", &self.desenho_perfil),
            ("drawing_sections", &self.desenho_secoes),
            ("drawing_corridor", &self.desenho_corredor),
        ] {
            chave(&mut e, k);
            e.push(XRecordEntry::new(90, XRecordValue::Int32(hs.len() as i32)));
            e.extend(hs.iter().map(|h| XRecordEntry::new(330, XRecordValue::Handle(*h))));
        }
        e
    }

    pub fn de_xrecord(nome: &str, e: &[XRecordEntry]) -> Result<Self, String> {
        if !matches!(e.first().map(|x| &x.value), Some(XRecordValue::String(s)) if s == MARCA) {
            return Err(format!("alinhamento {nome}: marca ausente"));
        }
        let mut a = AlinhamentoDwg::novo(nome, Vec::new(), false);
        let mut k = 2;
        let txt = |v: &XRecordValue| if let XRecordValue::String(s) = v { s.clone() } else { String::new() };
        let num = |v: &XRecordValue| match v {
            XRecordValue::Double(d) => *d,
            XRecordValue::Int16(i) => *i as f64,
            XRecordValue::Int32(i) => *i as f64,
            _ => 0.0,
        };
        let pt = |v: &XRecordValue| if let XRecordValue::Point3D(x, y, z) = v { [*x, *y, *z] } else { [0.0; 3] };
        let cont = |v: &XRecordValue| if let XRecordValue::Int32(n) = v { (*n).max(0) as usize } else { 0 };
        let val = |k: usize| e.get(k).map(|x| &x.value).ok_or_else(|| format!("alinhamento {nome}: registro curto"));
        while k < e.len() {
            let XRecordValue::String(chave) = &e[k].value else {
                k += 1;
                continue;
            };
            if e[k].code != 1 {
                k += 1;
                continue;
            }
            k += 1;
            match chave.as_str() {
                "name" => {
                    a.nome = txt(val(k)?);
                    k += 1;
                }
                "desc" => {
                    a.descricao = txt(val(k)?);
                    k += 1;
                }
                "geometry" => {
                    a.fechada = num(val(k)?) != 0.0;
                    let n = cont(val(k + 1)?);
                    k += 2;
                    for _ in 0..n {
                        let p = pt(val(k)?);
                        let b = num(val(k + 1)?);
                        a.vertices.push(([p[0], p[1]], b));
                        k += 2;
                    }
                }
                "stations" => {
                    a.estaca_inicial = num(val(k)?);
                    a.intervalo = num(val(k + 1)?);
                    k += 2;
                }
                "surface" => {
                    a.superficie = txt(val(k)?);
                    k += 1;
                }
                "pvis" => {
                    let n = cont(val(k)?);
                    k += 1;
                    for _ in 0..n {
                        let p = pt(val(k)?);
                        a.pivs.push(Piv { estaca: p[0], cota: p[1], curva: p[2] });
                        k += 1;
                    }
                }
                "profileview" => {
                    let o = pt(val(k)?);
                    let mut v = VistaPerfil { origem: [o[0], o[1]], exagero: num(val(k + 1)?), base: o[2], topo: o[2] };
                    k += 2;
                    if e.get(k).is_some_and(|x| x.code == 41) {
                        v.topo = num(val(k)?);
                        k += 1;
                    }
                    a.vista = Some(v);
                }
                "sections" => {
                    let o = pt(val(k)?);
                    let p = pt(val(k + 1)?);
                    let mut sec = Secoes::nova(p[0], p[1], p[2], [o[0], o[1]], cont(val(k + 2)?).max(1));
                    k += 3;
                    if e.get(k).is_some_and(|x| x.code == 91) {
                        let n = cont(val(k)?);
                        k += 1;
                        for _ in 0..n {
                            sec.fontes.push(txt(val(k)?));
                            k += 1;
                        }
                    }
                    if e.get(k).is_some_and(|x| x.code == 92) {
                        let n = cont(val(k)?);
                        k += 1;
                        for _ in 0..n {
                            sec.estacas.push(num(val(k)?));
                            k += 1;
                        }
                    }
                    a.secoes = Some(sec);
                }
                "assembly" => {
                    let mut v = [0.0; 9];
                    for x in v.iter_mut() {
                        *x = num(val(k)?);
                        k += 1;
                    }
                    a.assembly = Some(AssemblyBarragem {
                        crista: v[0],
                        decl_crista: v[1],
                        montante_esquerda: v[2] != 0.0,
                        talude_montante: v[3],
                        talude_jusante: v[4],
                        altura_berma: v[5],
                        largura_berma: v[6],
                        talude_corte: v[7],
                        alcance: v[8],
                    });
                }
                "corridor" => {
                    a.corredor = Some(num(val(k)?));
                    let (va, vc) = (num(val(k + 1)?), num(val(k + 2)?));
                    a.volumes = (va.is_finite() && vc.is_finite() && va >= 0.0 && vc >= 0.0).then_some((va, vc));
                    k += 3;
                }
                "materials" => {
                    a.materiais = Some(CriterioMateriais {
                        base: txt(val(k)?),
                        comparar: txt(val(k + 1)?),
                        fator_corte: num(val(k + 2)?),
                        fator_aterro: num(val(k + 3)?),
                        tabela: None,
                        totais: None,
                    });
                    k += 4;
                    if e.get(k).is_some_and(|x| x.code == 42) {
                        let t = (num(val(k)?), num(val(k + 1)?));
                        if let Some(m) = a.materiais.as_mut() {
                            m.totais = Some(t);
                        }
                        k += 2;
                    }
                    if e.get(k).is_some_and(|x| x.code == 15) {
                        let t = pt(val(k)?);
                        if let Some(m) = a.materiais.as_mut() {
                            m.tabela = Some([t[0], t[1]]);
                        }
                        k += 1;
                    }
                }
                "drawing" | "drawing_profile" | "drawing_sections" | "drawing_corridor" | "drawing_volumes" => {
                    let n = cont(val(k)?);
                    k += 1;
                    let mut hs = Vec::new();
                    for _ in 0..n {
                        if let XRecordValue::Handle(h) = val(k)? {
                            hs.push(*h);
                        }
                        k += 1;
                    }
                    match chave.as_str() {
                        "drawing" => a.desenho = hs,
                        "drawing_profile" => a.desenho_perfil = hs,
                        "drawing_sections" => a.desenho_secoes = hs,
                        "drawing_volumes" => a.desenho_volumes = hs,
                        _ => a.desenho_corredor = hs,
                    }
                }
                _ => {}
            }
        }
        Ok(a)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn grava_e_le_igual() {
        let mut a = AlinhamentoDwg::novo("EIXO1", vec![([0.0, 0.0], 0.0), ([100.0, 0.0], 0.2), ([150.0, 50.0], 0.0)], false);
        a.descricao = "Eixo da barragem".into();
        a.estaca_inicial = 20.0;
        a.superficie = "TN".into();
        a.pivs = vec![Piv { estaca: 20.0, cota: 955.0, curva: 0.0 }, Piv { estaca: 100.0, cota: 956.0, curva: 30.0 }];
        a.vista = Some(VistaPerfil { origem: [500.0, -200.0], exagero: 10.0, base: 950.0, topo: 970.0 });
        let mut sec = Secoes::nova(20.0, 30.0, 40.0, [800.0, -200.0], 4);
        sec.fontes = vec!["LIMPEZA".into()];
        sec.estacas = vec![0.0, 35.5, 80.0];
        a.secoes = Some(sec);
        a.desenho = vec![Handle::new(0x2A)];
        a.desenho_perfil = vec![Handle::new(0x2B), Handle::new(0x2C)];
        a.assembly = Some(AssemblyBarragem { crista: 8.0, montante_esquerda: false, ..Default::default() });
        a.corredor = Some(5.0);
        a.volumes = Some((12345.6, 7.8));
        a.desenho_corredor = vec![Handle::new(0x40)];
        a.materiais = Some(CriterioMateriais { base: "TN".into(), comparar: "EIXO1-CORR".into(), fator_corte: 1.0, fator_aterro: 1.25, tabela: Some([10.0, -20.0]), totais: Some((5.5, 1234.0)) });
        a.desenho_volumes = vec![Handle::new(0x41)];
        let b = AlinhamentoDwg::de_xrecord("EIXO1", &a.para_xrecord()).unwrap();
        assert_eq!(a, b);
        assert!(b.geometria().unwrap().comprimento() > 150.0);
    }
}
