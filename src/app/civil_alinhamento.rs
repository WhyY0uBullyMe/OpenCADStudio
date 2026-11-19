//! Agral Civil: alinhamento, perfil e seções no desenho (spec 003 R4 a R6 e seções; pedido do JP em 18/11:
//! "sem essas coisas o programa não me serve ainda").
//!
//! Cada alinhamento é um XRECORD em `AGRAL_CIVIL/ALIGNMENTS` (`civil::alinhamentos`). O que ele desenha fica em
//! três blocos, cada um entrando no desenho como uma inserção marcada (como a superfície):
//! - planta: eixo, marcas e rótulos das estacas (`AGRAL_ALINHAMENTO_<nome>`, XDATA `ALIGNMENT:<nome>`);
//! - vista de perfil: grade, terreno, greide e a faixa de cotas (`AGRAL_PERFIL_<nome>`, `PROFILEVIEW:<nome>`);
//! - seções: linhas de seção na planta e as vistas de seção (`AGRAL_SECOES_<nome>`, `SECTIONS:<nome>`).

use super::OpenCADStudio;
use crate::app::civil::{marcar, montar_entrada};
use crate::civil::alinhamentos::{AlinhamentoDwg, Secoes, VistaPerfil, DICIONARIO};
use agral_civil_motor::alinhamento::{perfil_do_terreno, secao_do_terreno, texto_da_estaca, Alinhamento, Greide, Piv};
use agral_civil_motor::corredor::{secao_do_corredor, volumes, AssemblyBarragem, Codigo, SecaoCorredor};
use agral_civil_motor::tin::Tin;
use document::entities::{Line, LwPolyline, LwVertex, Text, TextHorizontalAlignment, TextVerticalAlignment};
use document::types::{Vector2, Vector3};
use document::EntityType;
use std::sync::Arc;

pub(crate) const CAMADA_EIXO: (&str, i16) = ("C-ALIN", 1);
pub(crate) const CAMADA_ESTACA: (&str, i16) = ("C-ALIN-ESTACA", 7);
pub(crate) const CAMADA_GRADE: (&str, i16) = ("C-PERFIL-GRADE", 8);
pub(crate) const CAMADA_TN: (&str, i16) = ("C-PERFIL-TN", 3);
pub(crate) const CAMADA_GREIDE: (&str, i16) = ("C-PERFIL-GREIDE", 1);
pub(crate) const CAMADA_TEXTO: (&str, i16) = ("C-PERFIL-TEXTO", 7);
pub(crate) const CAMADA_SECAO: (&str, i16) = ("C-SECAO-LINHA", 4);
pub(crate) const CAMADA_CORREDOR: (&str, i16) = ("C-CORREDOR", 6);
pub(crate) const CAMADA_SAIA: (&str, i16) = ("C-CORREDOR-SAIA", 30);
pub(crate) const CAMADA_FONTE: (&str, i16) = ("C-SECAO-FONTE", 2);

/// Nome da superfície do corredor do eixo.
pub(crate) fn superficie_do_corredor(eixo: &str) -> String {
    format!("{eixo}-CORR")
}

fn bloco(tipo: &str, nome: &str) -> String {
    format!("AGRAL_{tipo}_{nome}")
}

fn linha(a: [f64; 2], b: [f64; 2], camada: (&str, i16)) -> EntityType {
    let mut l = Line::new();
    l.start = Vector3::new(a[0], a[1], 0.0);
    l.end = Vector3::new(b[0], b[1], 0.0);
    l.common.layer = camada.0.into();
    EntityType::Line(l)
}

fn polilinha(pts: &[[f64; 2]], camada: (&str, i16)) -> Option<EntityType> {
    if pts.len() < 2 {
        return None;
    }
    let mut p = LwPolyline::new();
    p.vertices = pts.iter().map(|q| LwVertex::new(Vector2::new(q[0], q[1]))).collect();
    p.common.layer = camada.0.into();
    Some(EntityType::LwPolyline(p))
}

/// Texto com o ponto de inserção no meio (horizontal e vertical) ou à esquerda.
fn texto(valor: String, p: [f64; 2], altura: f64, rotacao: f64, centro: bool, camada: (&str, i16)) -> EntityType {
    let mut t = Text::new();
    t.value = valor;
    t.height = altura;
    t.rotation = rotacao;
    t.common.layer = camada.0.into();
    let v = Vector3::new(p[0], p[1], 0.0);
    t.insertion_point = v;
    if centro {
        t.horizontal_alignment = TextHorizontalAlignment::Center;
        t.vertical_alignment = TextVerticalAlignment::Middle;
        t.alignment_point = Some(v);
    }
    EntityType::Text(t)
}

/// Passo "redondo" da grade para cobrir `faixa` com umas 8 linhas.
fn passo_redondo(faixa: f64, linhas: f64) -> f64 {
    let bruto = (faixa / linhas).max(1e-6);
    let base = 10f64.powf(bruto.log10().floor());
    for m in [1.0, 2.0, 2.5, 5.0, 10.0] {
        if base * m >= bruto {
            return base * m;
        }
    }
    base * 10.0
}

/// Vértices com bulge de uma polilinha do desenho (2D, leve ou 3D sem bulge; linha).
pub(crate) fn vertices_com_bulge(e: &EntityType) -> Option<(Vec<([f64; 2], f64)>, bool)> {
    match e {
        EntityType::LwPolyline(p) => {
            Some((p.vertices.iter().map(|v| ([v.location.x, v.location.y], v.bulge)).collect(), p.is_closed))
        }
        EntityType::Polyline2D(p) => Some((
            p.vertices.iter().filter(|v| v.flags.bits() & 16 == 0).map(|v| ([v.location.x, v.location.y], v.bulge)).collect(),
            p.flags.bits() & 1 != 0,
        )),
        EntityType::Line(l) => Some((vec![([l.start.x, l.start.y], 0.0), ([l.end.x, l.end.y], 0.0)], false)),
        _ => crate::civil::curvas::pontos(e).map(|p| (p.iter().map(|q| ([q[0], q[1]], 0.0)).collect(), false)),
    }
}

/// Dica do mouse: o eixo mais perto (até `alcance` m) com a estaca e o afastamento (direita positiva).
pub(crate) fn dica_do_eixo(doc: &document::Document, p: [f64; 2], alcance: f64) -> Option<String> {
    crate::civil::alinhamentos(doc)
        .into_iter()
        .filter_map(|a| {
            let g = a.geometria()?;
            let (e, o) = g.estaca_e_offset(p);
            (o.abs() <= alcance && e > g.estaca_inicial - 1e-6 && e < g.estaca_final() + 1e-6).then(|| (o.abs(), a.nome, e, o))
        })
        .min_by(|x, y| x.0.total_cmp(&y.0))
        .map(|(_, n, e, o)| format!("{n}\nStation: {}\nOffset: {:.3} {}", texto_da_estaca(e), o.abs(), if o >= 0.0 { "R" } else { "L" }))
}

impl OpenCADStudio {
    pub(crate) fn civil_alinhamentos(&self, i: usize) -> Vec<AlinhamentoDwg> {
        crate::civil::alinhamentos(&self.tabs[i].scene.document)
    }

    pub(crate) fn civil_al_ler(&self, i: usize, nome: &str) -> Result<AlinhamentoDwg, String> {
        self.civil_alinhamentos(i)
            .into_iter()
            .find(|a| a.nome.eq_ignore_ascii_case(nome))
            .ok_or_else(|| format!("Alignment '{nome}' not found."))
    }

    /// Nome do alinhamento: o dado, ou o único que existe.
    pub(crate) fn civil_al_escolher(&self, i: usize, nome: Option<&str>) -> Result<String, String> {
        if let Some(n) = nome {
            return self.civil_al_ler(i, n).map(|a| a.nome);
        }
        let todos = self.civil_alinhamentos(i);
        // Com vários, vale o do objeto selecionado (planta, perfil, seções, corredor ou tabela).
        if todos.len() > 1 {
            if let Some((aba, n)) = self.civil_contexto(i).filter(|(aba, _)| *aba != "tin_surface") {
                let _ = aba;
                if let Some(a) = todos.iter().find(|a| a.nome.eq_ignore_ascii_case(&n)) {
                    return Ok(a.nome.clone());
                }
            }
        }
        match todos.as_slice() {
            [a] => Ok(a.nome.clone()),
            [] => Err("No alignments in the drawing. Use CREATEALIGNMENT first.".into()),
            _ => Err(format!(
                "Several alignments ({}): name the one to use.",
                todos.iter().map(|a| a.nome.as_str()).collect::<Vec<_>>().join(", ")
            )),
        }
    }

    pub(crate) fn civil_al_gravar(&mut self, i: usize, a: &AlinhamentoDwg, novo: bool) -> Result<(), String> {
        let mut host = crate::app::plugin_host::HostSession::new(self, i);
        let e = a.para_xrecord();
        if novo {
            host.create_xrecord(&DICIONARIO, &a.nome, e)?;
        } else {
            host.update_xrecord(&DICIONARIO, &a.nome, e)?;
        }
        self.tabs[i].dirty = true;
        Ok(())
    }

    /// TIN da superfície: o da memória se ainda vale, senão monta agora.
    pub(crate) fn civil_tin(&mut self, i: usize, nome: &str) -> Result<Arc<Tin>, String> {
        let s = self.civil_ler(i, nome)?;
        if let Some((d, t)) = self.tabs[i].civil_tins.get(&s.nome) {
            if *d == s.desenho && !t.triangulos.is_empty() {
                return Ok(t.clone());
            }
        }
        let e = self.civil_entrada(i, &s);
        let t = Arc::new(montar_entrada(e, &std::sync::atomic::AtomicU8::new(0)).tin);
        if t.triangulos.is_empty() {
            return Err(format!("Surface '{}' has no triangles.", s.nome));
        }
        self.tabs[i].civil_tins.insert(s.nome.clone(), (s.desenho.clone(), t.clone()));
        Ok(t)
    }

    pub(crate) fn civil_proximo_eixo(&self, i: usize) -> String {
        let nomes: Vec<String> = self.civil_alinhamentos(i).into_iter().map(|a| a.nome.to_uppercase()).collect();
        (1..).map(|k| format!("EIXO{k}")).find(|n| !nomes.contains(n)).unwrap()
    }

    /// `CREATEALIGNMENT`: alinhamento a partir de uma polilinha (retas e arcos), copiado da geometria dela.
    pub(crate) fn civil_criar_alinhamento(&mut self, i: usize, nome: &str, fonte: document::Handle) -> Result<String, String> {
        let nome = nome.trim();
        if !crate::app::civil::nome_valido(nome) {
            return Err(format!("Invalid alignment name '{nome}'."));
        }
        if self.civil_al_ler(i, nome).is_ok() {
            return Err(format!("Alignment '{nome}' already exists."));
        }
        let e = self.tabs[i].scene.document.get_entity(fonte).ok_or("The polyline no longer exists.")?;
        let (v, fechada) = vertices_com_bulge(e).ok_or("Select a polyline, 3D polyline or line.")?;
        self.civil_criar_alinhamento_de_vertices(i, nome, v, fechada)
    }

    /// Eixo a partir dos vértices (com bulge), sem objeto de origem: o traçado por PIs (`CREATEALIGNMENTLAYOUT`).
    pub(crate) fn civil_criar_alinhamento_de_vertices(
        &mut self,
        i: usize,
        nome: &str,
        v: Vec<([f64; 2], f64)>,
        fechada: bool,
    ) -> Result<String, String> {
        let nome = nome.trim();
        if !crate::app::civil::nome_valido(nome) {
            return Err(format!("Invalid alignment name '{nome}'."));
        }
        if self.civil_al_ler(i, nome).is_ok() {
            return Err(format!("Alignment '{nome}' already exists."));
        }
        let mut a = AlinhamentoDwg::novo(nome, v, fechada);
        a.geometria().ok_or("The polyline is too short for an alignment.")?;
        // Superfície padrão: a única do desenho, se houver uma só.
        if let [s] = self.civil_superficies(i).as_slice() {
            a.superficie = s.clone();
        }
        self.push_undo_snapshot(i, "CREATEALIGNMENT");
        self.civil_al_gravar(i, &a, true)?;
        let msg = self.civil_redesenhar_alinhamento(i, nome)?;
        Ok(format!("Alignment '{nome}' created. {msg}"))
    }

    /// `ALIGNMENTSTATIONS`: estaca inicial e intervalo dos rótulos.
    pub(crate) fn civil_estacas(&mut self, i: usize, nome: &str, inicial: f64, intervalo: f64) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, nome)?;
        if !(intervalo > 0.0) {
            return Err("The station interval must be more than zero.".into());
        }
        self.push_undo_snapshot(i, "ALIGNMENTSTATIONS");
        a.estaca_inicial = inicial;
        a.intervalo = intervalo;
        self.civil_al_gravar(i, &a, false)?;
        self.civil_redesenhar_alinhamento(i, nome)
    }

    /// `CREATEPROFILEVIEW`: vista de perfil do terreno (e do greide, se houver) com o canto em `origem`.
    pub(crate) fn civil_vista_perfil(&mut self, i: usize, nome: &str, superficie: &str, exagero: f64, origem: [f64; 2]) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, nome)?;
        let s = self.civil_ler(i, superficie)?;
        if !(exagero > 0.0) {
            return Err("The vertical exaggeration must be more than zero.".into());
        }
        self.push_undo_snapshot(i, "CREATEPROFILEVIEW");
        a.superficie = s.nome;
        a.vista = Some(VistaPerfil { origem, exagero, base: 0.0, topo: 0.0 });
        self.civil_al_gravar(i, &a, false)?;
        self.civil_redesenhar_alinhamento(i, nome)
    }

    /// `CREATEPROFILE`: o greide (PIVs de estaca, cota e curva vertical).
    pub(crate) fn civil_definir_greide(&mut self, i: usize, nome: &str, pivs: Vec<Piv>) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, nome)?;
        let g = Greide::novo(pivs).ok_or("Type at least one PVI (station,elevation).")?;
        self.push_undo_snapshot(i, "CREATEPROFILE");
        a.pivs = g.pivs;
        self.civil_al_gravar(i, &a, false)?;
        let msg = self.civil_redesenhar_alinhamento(i, nome)?;
        Ok(format!("Design profile of '{nome}': {} PVIs. {msg}", a.pivs.len()))
    }

    /// `CREATESECTIONS`: linhas de seção na planta a cada `intervalo` m e as vistas de seção em grade a partir de
    /// `origem` (canto de cima à esquerda).
    pub(crate) fn civil_definir_secoes(&mut self, i: usize, nome: &str, s: Secoes) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, nome)?;
        if !(s.intervalo > 0.0 && s.esquerda >= 0.0 && s.direita >= 0.0 && s.esquerda + s.direita > 0.0) {
            return Err("Interval and widths must be positive.".into());
        }
        if a.superficie.is_empty() {
            return Err(format!("Alignment '{nome}' has no surface yet: create its profile view first, or name the surface."));
        }
        self.push_undo_snapshot(i, "CREATESECTIONS");
        // As superfícies a mais ficam; as estacas escolhidas à mão só se o intervalo não mudou.
        let mut s = s;
        if let Some(antes) = &a.secoes {
            if s.fontes.is_empty() {
                s.fontes = antes.fontes.clone();
            }
            if s.estacas.is_empty() && (antes.intervalo - s.intervalo).abs() < 1e-9 {
                s.estacas = antes.estacas.clone();
            }
        }
        a.secoes = Some(s);
        self.civil_al_gravar(i, &a, false)?;
        self.civil_redesenhar_alinhamento(i, nome)
    }

    /// `SAMPLEMORESOURCES`: superfícies a mais desenhadas nas vistas de seção (Civil 3D: Sample More Sources).
    pub(crate) fn civil_mais_fontes(&mut self, i: usize, nome: &str, fontes: Vec<String>) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, nome)?;
        let mut certas = Vec::new();
        for f in fontes {
            let n = self.civil_escolher(i, Some(&f))?;
            if !n.eq_ignore_ascii_case(&a.superficie) && !certas.iter().any(|c: &String| c.eq_ignore_ascii_case(&n)) {
                certas.push(n);
            }
        }
        let s = a.secoes.as_mut().ok_or_else(|| format!("Alignment '{nome}' has no sample lines: run CREATESECTIONS first."))?;
        self.push_undo_snapshot(i, "SAMPLEMORESOURCES");
        s.fontes = certas.clone();
        self.civil_al_gravar(i, &a, false)?;
        let msg = self.civil_redesenhar_alinhamento(i, nome)?;
        Ok(if certas.is_empty() {
            format!("Sections of '{nome}' show only '{}'. {msg}", a.superficie)
        } else {
            format!("Sections of '{nome}' also show: {}. {msg}", certas.join(", "))
        })
    }

    /// `EDITSAMPLELINES`: as estacas das sample lines escolhidas à mão (vazio volta ao intervalo).
    pub(crate) fn civil_estacas_das_secoes(&mut self, i: usize, nome: &str, estacas: Vec<f64>) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, nome)?;
        let s = a.secoes.as_mut().ok_or_else(|| format!("Alignment '{nome}' has no sample lines: run CREATESECTIONS first."))?;
        self.push_undo_snapshot(i, "EDITSAMPLELINES");
        s.estacas = estacas;
        let n = s.estacas.len();
        self.civil_al_gravar(i, &a, false)?;
        let msg = self.civil_redesenhar_alinhamento(i, nome)?;
        Ok(format!("Sample lines of '{nome}': {}. {msg}", if n == 0 { "every interval".to_string() } else { format!("{n} stations") }))
    }

    /// `CREATEASSEMBLY`: o assembly da barragem do eixo.
    pub(crate) fn civil_definir_assembly(&mut self, i: usize, nome: &str, asm: AssemblyBarragem) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, nome)?;
        if !(asm.crista > 0.0 && asm.talude_montante > 0.0 && asm.talude_jusante > 0.0 && asm.talude_corte > 0.0) {
            return Err("Crest width and slopes must be positive.".into());
        }
        self.push_undo_snapshot(i, "CREATEASSEMBLY");
        a.assembly = Some(asm);
        self.civil_al_gravar(i, &a, false)?;
        let msg = self.civil_redesenhar_alinhamento(i, nome)?;
        Ok(format!("Assembly of '{nome}' set. {msg}"))
    }

    /// `CREATECORRIDOR`: corredor do eixo com seções a cada `passo` m (e nas estacas notáveis).
    pub(crate) fn civil_criar_corredor(&mut self, i: usize, nome: &str, passo: f64) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, nome)?;
        if !(passo > 0.0) {
            return Err("The corridor frequency must be more than zero.".into());
        }
        if a.pivs.is_empty() {
            return Err(format!("Alignment '{nome}' has no design profile: use CREATEPROFILE first (the crest elevation)."));
        }
        if a.superficie.is_empty() {
            return Err(format!("Alignment '{nome}' has no surface: create its profile view first."));
        }
        if a.assembly.is_none() {
            a.assembly = Some(AssemblyBarragem::default());
        }
        self.push_undo_snapshot(i, "CREATECORRIDOR");
        a.corredor = Some(passo);
        self.civil_al_gravar(i, &a, false)?;
        self.civil_redesenhar_alinhamento(i, nome)
    }

    /// Desenha o corredor: polilinhas 3D por código (crista, bermas, saia, eixo), o limite pela saia, e a superfície
    /// do corredor (linhas como linhas de quebra, limite externo pela saia: sem "dentes").
    fn civil_desenhar_corredor(&mut self, i: usize, a: &mut AlinhamentoDwg, secoes: &[SecaoCorredor]) -> Result<String, String> {
        let scene = &mut self.tabs[i].scene;
        scene.erase_entities(&a.desenho_corredor);
        for (camada, cor) in [CAMADA_CORREDOR, CAMADA_SAIA] {
            if !scene.document.layers.contains(camada) {
                scene.ensure_layer(camada);
                if let Some(l) = scene.document.layers.get_mut(camada) {
                    l.color = document::Color::Index(cor as u8);
                }
            }
        }
        let al = a.geometria().ok_or("Alignment without geometry.")?;
        // Trilhas: (lado, código) -> trechos contínuos de pontos 3D.
        let mut trilhas: std::collections::BTreeMap<(i8, String), Vec<Vec<[f64; 3]>>> = Default::default();
        let mut vistas: std::collections::BTreeSet<(i8, String)> = Default::default();
        for s in secoes {
            let mut desta: std::collections::BTreeSet<(i8, String)> = Default::default();
            for (o, z, c) in &s.pontos {
                let lado = if *o < -1e-9 { -1 } else if *o > 1e-9 { 1 } else { 0 };
                let chave = (lado, format!("{c:?}"));
                let p = al.ponto_com_offset(s.estaca, *o);
                let t = trilhas.entry(chave.clone()).or_default();
                if !vistas.contains(&chave) || t.is_empty() {
                    t.push(Vec::new());
                }
                t.last_mut().unwrap().push([p[0], p[1], *z]);
                desta.insert(chave);
            }
            // Código que sumiu nesta estaca fecha o trecho: a próxima vez começa outro.
            vistas = desta;
        }
        let mut novas: Vec<EntityType> = Vec::new();
        for ((lado, codigo), trechos) in &trilhas {
            for t in trechos.iter().filter(|t| t.len() >= 2) {
                let mut pl = document::entities::Polyline3D::from_points(t.iter().map(|p| Vector3::new(p[0], p[1], p[2])).collect());
                pl.common.layer = if codigo == "Saia" { CAMADA_SAIA.0.into() } else { CAMADA_CORREDOR.0.into() };
                let mut e = EntityType::Polyline3D(pl);
                marcar(&mut e, &format!("CORRIDOR:{}:{lado}:{codigo}", a.nome));
                novas.push(e);
            }
        }
        // Limite: saia esquerda em frente e a direita de volta (só onde as duas existem em todas as seções).
        let mut limite: Vec<[f64; 3]> = Vec::new();
        let completas: Vec<&SecaoCorredor> = secoes.iter().filter(|s| s.completa).collect();
        for s in &completas {
            let (o, z, _) = s.pontos[0];
            let p = al.ponto_com_offset(s.estaca, o);
            limite.push([p[0], p[1], z]);
        }
        for s in completas.iter().rev() {
            let (o, z, _) = *s.pontos.last().unwrap();
            let p = al.ponto_com_offset(s.estaca, o);
            limite.push([p[0], p[1], z]);
        }
        let tem_limite = limite.len() >= 4;
        if tem_limite {
            let mut pl = document::entities::Polyline3D::from_points(limite.iter().map(|p| Vector3::new(p[0], p[1], p[2])).collect());
            pl.flags.closed = true;
            pl.common.layer = CAMADA_SAIA.0.into();
            let mut e = EntityType::Polyline3D(pl);
            marcar(&mut e, &format!("CORRIDOR:{}:BOUNDARY", a.nome));
            novas.push(e);
        }
        let handles = self.tabs[i].scene.add_entities(novas);
        a.desenho_corredor = handles.clone();
        // Superfície do corredor: cria se não existe; troca as fontes pelas linhas novas.
        let nome_sup = superficie_do_corredor(&a.nome);
        if self.civil_ler(i, &nome_sup).is_err() {
            self.civil_criar(i, &nome_sup, &format!("Corridor of {}", a.nome), "")?;
        }
        let mut sup = self.civil_ler(i, &nome_sup)?;
        sup.itens.retain(|it| it.descricao != "corridor");
        let (linhas, lim): (Vec<_>, Vec<_>) = if tem_limite {
            let (l, b) = handles.split_at(handles.len() - 1);
            (l.to_vec(), b.to_vec())
        } else {
            (handles.clone(), Vec::new())
        };
        let mut q = crate::civil::definicao::ItemDwg::novo("BREAKLINES", "corridor");
        q.fontes = linhas;
        sup.itens.push(q);
        if !lim.is_empty() {
            let mut b = crate::civil::definicao::ItemDwg::novo("BOUNDARY_OUTER", "corridor");
            b.fontes = lim;
            sup.itens.push(b);
        }
        sup.desatualizada = true;
        {
            let mut host = crate::app::plugin_host::HostSession::new(self, i);
            host.update_xrecord(&crate::civil::definicao::DICIONARIO, &sup.nome, sup.para_xrecord())?;
        }
        let msg_sup = self.civil_refazer(i, &nome_sup)?;
        let (va, vc) = volumes(secoes);
        a.volumes = Some((va, vc));
        let incompletas = secoes.len() - completas.len();
        Ok(format!(
            "Corridor: {} sections, fill {:.1} m3, cut {:.1} m3{}. {msg_sup}",
            secoes.len(),
            va,
            vc,
            if incompletas > 0 { format!(" ({incompletas} sections did not reach the terrain)") } else { String::new() }
        ))
    }

    pub(crate) fn civil_apagar_alinhamento(&mut self, i: usize, nome: &str) -> Result<String, String> {
        let a = self.civil_al_ler(i, nome)?;
        self.push_undo_snapshot(i, "DELETEALIGNMENT");
        let scene = &mut self.tabs[i].scene;
        scene.erase_entities(&a.desenho_corredor);
        for (tipo, hs) in [("ALINHAMENTO", &a.desenho), ("PERFIL", &a.desenho_perfil), ("SECOES", &a.desenho_secoes), ("VOLUMES", &a.desenho_volumes)] {
            scene.erase_entities(hs);
            let b = bloco(tipo, &a.nome);
            if scene.document.block_records.get(&b).is_some() {
                crate::scene::block_service::erase(scene, &b);
            }
        }
        let mut host = crate::app::plugin_host::HostSession::new(self, i);
        host.delete_xrecord(&DICIONARIO, &a.nome)?;
        self.tabs[i].dirty = true;
        Ok(format!("Alignment '{}' deleted.", a.nome))
    }

    /// Troca um dos três desenhos do alinhamento pelo novo conteúdo; devolve os handles da inserção.
    pub(crate) fn civil_al_trocar_desenho(&mut self, i: usize, tipo: &str, marca: &str, nome: &str, antigo: &[document::Handle], conteudo: Vec<EntityType>) -> Vec<document::Handle> {
        let scene = &mut self.tabs[i].scene;
        scene.erase_entities(antigo);
        let b = bloco(tipo, nome);
        if scene.document.block_records.get(&b).is_some() {
            crate::scene::block_service::erase(scene, &b);
        }
        if conteudo.is_empty() {
            return Vec::new();
        }
        for (camada, cor) in [CAMADA_EIXO, CAMADA_ESTACA, CAMADA_GRADE, CAMADA_TN, CAMADA_GREIDE, CAMADA_TEXTO, CAMADA_SECAO, CAMADA_CORREDOR, CAMADA_FONTE] {
            if !scene.document.layers.contains(camada) && conteudo.iter().any(|e| e.common().layer == camada) {
                scene.ensure_layer(camada);
                if let Some(l) = scene.document.layers.get_mut(camada) {
                    l.color = document::Color::Index(cor as u8);
                }
            }
        }
        let primeira = conteudo[0].common().layer.clone();
        scene.define_block_raw(&b, Vector3::new(0.0, 0.0, 0.0), conteudo);
        let camada = match tipo {
            "ALINHAMENTO" => CAMADA_EIXO.0,
            "PERFIL" => CAMADA_GRADE.0,
            "SECOES" => CAMADA_SECAO.0,
            _ => primeira.as_str(),
        };
        scene.ensure_layer(camada);
        let mut ins = document::entities::Insert::new(b, Vector3::new(0.0, 0.0, 0.0));
        ins.common.layer = camada.into();
        let mut e = EntityType::Insert(ins);
        marcar(&mut e, &format!("{marca}:{nome}"));
        scene.add_entities(vec![e])
    }

    /// Redesenha tudo do alinhamento: planta, vista de perfil (se houver) e seções (se houver).
    pub(crate) fn civil_redesenhar_alinhamento(&mut self, i: usize, nome: &str) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, nome)?;
        let al = a.geometria().ok_or("Alignment without geometry.")?;
        let mut msgs = vec![format!("Length {:.3} m, stations {} to {}.", al.comprimento(), texto_da_estaca(al.estaca_inicial), texto_da_estaca(al.estaca_final()))];
        let planta = desenho_da_planta(&al, a.intervalo);
        a.desenho = self.civil_al_trocar_desenho(i, "ALINHAMENTO", "ALIGNMENT", &a.nome.clone(), &a.desenho.clone(), planta);
        let greide = a.greide();
        let tin = if a.superficie.is_empty() { None } else { Some(self.civil_tin(i, &a.superficie.clone())) };
        if let Some(v) = a.vista.clone() {
            match &tin {
                Some(Ok(t)) => {
                    let (ents, resumo, base, topo) = desenho_do_perfil(t, &al, greide.as_ref(), a.intervalo, &v, &a.nome, &a.superficie);
                    if let Some(vv) = a.vista.as_mut() {
                        vv.base = base;
                        vv.topo = topo;
                    }
                    a.desenho_perfil = self.civil_al_trocar_desenho(i, "PERFIL", "PROFILEVIEW", &a.nome.clone(), &a.desenho_perfil.clone(), ents);
                    msgs.push(resumo);
                }
                Some(Err(e)) => msgs.push(format!("Profile view not drawn: {e}")),
                None => {}
            }
        }
        // Corredor: seções a cada passo e nas estacas notáveis.
        let asm = a.assembly.clone().unwrap_or_default();
        let corredor_ok = a.corredor.is_some() && greide.is_some() && matches!(&tin, Some(Ok(_)));
        if let (Some(passo), Some(g), Some(Ok(t))) = (a.corredor, greide.as_ref(), &tin) {
            // Só onde há greide; as pontas do greide entram como seção.
            let (ga, gb) = g.faixa();
            let mut est: Vec<f64> = al.estacas(passo);
            if g.pivs.len() > 1 {
                est.extend([ga, gb]);
                est.retain(|e| *e >= ga - 1e-6 && *e <= gb + 1e-6);
                est.sort_by(|a, b| a.total_cmp(b));
                est.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
            }
            let secoes: Vec<SecaoCorredor> = est.into_iter().map(|e| secao_do_corredor(t, &al, e, g.cota(e), &asm)).collect();
            match self.civil_desenhar_corredor(i, &mut a, &secoes) {
                Ok(m) => msgs.push(m),
                Err(e) => msgs.push(format!("Corridor not drawn: {e}")),
            }
        }
        if let Some(s) = a.secoes.clone() {
            match &tin {
                Some(Ok(t)) => {
                    let gabarito = corredor_ok.then_some(&asm);
                    let mut extras: Vec<(String, Arc<Tin>)> = Vec::new();
                    for f in &s.fontes {
                        match self.civil_tin(i, f) {
                            Ok(x) => extras.push((f.clone(), x)),
                            Err(e) => msgs.push(format!("Source '{f}' not shown: {e}")),
                        }
                    }
                    let (mut ents, n) = desenho_das_secoes(t, &al, greide.as_ref(), &s, gabarito, &extras);
                    camadas_do_template(&self.tabs[i].scene.document, &mut ents);
                    a.desenho_secoes = self.civil_al_trocar_desenho(i, "SECOES", "SECTIONS", &a.nome.clone(), &a.desenho_secoes.clone(), ents);
                    msgs.push(format!("{n} cross sections."));
                }
                Some(Err(e)) => msgs.push(format!("Sections not drawn: {e}")),
                None => {}
            }
        }
        self.civil_al_gravar(i, &a, false)?;
        if a.materiais.as_ref().is_some_and(|m| m.tabela.is_some()) {
            match self.civil_desenhar_tabela_volumes(i, &a.nome) {
                Ok(m) => msgs.push(m),
                Err(e) => msgs.push(format!("Volume table not drawn: {e}")),
            }
        }
        Ok(msgs.join(" "))
    }

    /// Depois de montar a superfície: redesenha perfis e seções dos alinhamentos que usam ela.
    pub(crate) fn civil_alinhamentos_da_superficie(&mut self, i: usize, superficie: &str) -> Vec<String> {
        let nomes: Vec<String> = self
            .civil_alinhamentos(i)
            .into_iter()
            .filter(|a| a.superficie.eq_ignore_ascii_case(superficie) && (a.vista.is_some() || a.secoes.is_some()))
            .map(|a| a.nome)
            .collect();
        nomes.into_iter().filter_map(|n| self.civil_redesenhar_alinhamento(i, &n).ok().map(|m| format!("Alignment '{n}' updated: {m}"))).collect()
    }
}

/// Camadas das seções no padrão do template do JP (TEMPLATE_JP.dwt), quando o desenho tem elas: grade, terreno,
/// gabarito e textos; texto em SIMPLEX-AS. Desenho sem o template fica nas camadas C-* do Agral CAD.
fn camadas_do_template(doc: &document::Document, ents: &mut [EntityType]) {
    let troca = [
        (CAMADA_GRADE.0, "GRID MENOR-SEC-AS"),
        (CAMADA_TN.0, "TERRENO NATURAL-AS"),
        (CAMADA_CORREDOR.0, "ATERRO-AS"),
        (CAMADA_TEXTO.0, "PERFIL GRID-TXT-AS"),
        (CAMADA_EIXO.0, "ALINHAMENTO-AS"),
    ];
    let estilo = doc.text_styles.get("SIMPLEX-AS").is_some();
    for e in ents.iter_mut() {
        let atual = e.common().layer.clone();
        if let Some((_, nova)) = troca.iter().find(|(de, para)| atual == *de && doc.layers.contains(para)) {
            e.common_mut().layer = nova.to_string();
        }
        if let (EntityType::Text(t), true) = (&mut *e, estilo) {
            t.style = "SIMPLEX-AS".into();
        }
    }
}

/// Planta: eixo, marcas de estaca (a cada intervalo; a cada 5 intervalos maior) e rótulos.
fn desenho_da_planta(al: &Alinhamento, intervalo: f64) -> Vec<EntityType> {
    let mut v = Vec::new();
    // Eixo: amostrado (arcos viram muitos trechos curtos).
    let l = al.comprimento();
    let n = ((l / 0.5).ceil() as usize).clamp(2, 200_000);
    let pts: Vec<[f64; 2]> = (0..=n).map(|k| al.ponto_na_estaca(al.estaca_inicial + l * k as f64 / n as f64).0).collect();
    v.extend(polilinha(&pts, CAMADA_EIXO));
    let alt = (intervalo / 8.0).clamp(0.5, 5.0);
    for e in al.estacas(intervalo) {
        let maior = (e / (intervalo * 5.0)).fract().abs() < 1e-9 || (e - al.estaca_inicial).abs() < 1e-9 || (e - al.estaca_final()).abs() < 1e-9;
        let m = if maior { alt * 1.6 } else { alt };
        let a = al.ponto_com_offset(e, -m);
        let b = al.ponto_com_offset(e, m);
        v.push(linha(a, b, CAMADA_ESTACA));
        let (_, dir) = al.ponto_na_estaca(e);
        let p = al.ponto_com_offset(e, -m - alt * 1.2);
        // Rótulo perpendicular ao eixo, sempre legível (nunca de cabeça para baixo, em eixo para oeste ou sul).
        let mut rot = dir + std::f64::consts::FRAC_PI_2;
        if rot.cos() < -1e-9 {
            rot += std::f64::consts::PI;
        }
        v.push(texto(texto_da_estaca(e), p, alt * 0.8, rot, true, CAMADA_ESTACA));
    }
    v
}

/// Passo da grade de cotas: redondo, umas 8 linhas, e nunca tão apertado que os rótulos se encavalem.
fn passo_das_cotas(faixa: f64, exagero: f64, alt: f64) -> f64 {
    let mut dz = passo_redondo(faixa.max(1.0), 8.0);
    while dz * exagero < alt * 2.4 {
        dz = passo_redondo(dz * 1.01, 1.0);
    }
    dz
}

/// Vista de perfil: moldura, grade (cotas dos dois lados, estacas), terreno, greide e a faixa embaixo com estaca,
/// cota do terreno e, se houver greide, cota do greide e aterro/corte, como nos perfis de projeto.
fn desenho_do_perfil(
    tin: &Tin,
    al: &Alinhamento,
    greide: Option<&Greide>,
    intervalo: f64,
    v: &VistaPerfil,
    nome: &str,
    superficie: &str,
) -> (Vec<EntityType>, String, f64, f64) {
    let mut out = Vec::new();
    let estacas = al.estacas(intervalo);
    let tn = perfil_do_terreno(tin, al, (intervalo / 10.0).max(0.5), &estacas);
    if tn.is_empty() {
        return (out, format!("The alignment does not cross surface '{superficie}': no terrain profile."), v.base, v.topo);
    }
    let gz = |e: f64| greide.and_then(|g| g.cota_em(e));
    let mut zmin = tn.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let mut zmax = tn.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    if let Some(g) = greide {
        for z in estacas.iter().filter_map(|e| g.cota_em(*e)).chain(g.pivs.iter().map(|p| p.cota)) {
            zmin = zmin.min(z);
            zmax = zmax.max(z);
        }
    }
    let largura = al.comprimento();
    // Texto proporcional ao espaço entre estacas, para os rótulos verticais da faixa caberem lado a lado.
    let alt = (intervalo * 0.15).clamp(0.5, 8.0);
    let (x0, y0, ex) = (v.origem[0], v.origem[1], v.exagero);
    let dz = passo_das_cotas(zmax - zmin, ex, alt);
    let base = (zmin / dz).floor() * dz - dz;
    let topo = (zmax / dz).ceil() * dz + dz;
    let px = |e: f64| x0 + (e - al.estaca_inicial);
    let py = |z: f64| y0 + (z - base) * ex;
    let rotulo = |out: &mut Vec<EntityType>, t: String, p: [f64; 2], direita: bool| {
        out.push(texto(t, p, alt, 0.0, false, CAMADA_TEXTO));
        if let EntityType::Text(t) = out.last_mut().unwrap() {
            t.horizontal_alignment = if direita { TextHorizontalAlignment::Right } else { TextHorizontalAlignment::Left };
            t.vertical_alignment = TextVerticalAlignment::Middle;
            t.alignment_point = Some(t.insertion_point);
        }
    };
    // Grade de cotas com rótulo dos dois lados.
    let mut z = base;
    while z <= topo + 1e-9 {
        out.push(linha([x0, py(z)], [x0 + largura, py(z)], CAMADA_GRADE));
        rotulo(&mut out, format!("{z:.2}"), [x0 - alt * 0.6, py(z)], true);
        rotulo(&mut out, format!("{z:.2}"), [x0 + largura + alt * 0.6, py(z)], false);
        z += dz;
    }
    for e in &estacas {
        out.push(linha([px(*e), py(base)], [px(*e), py(topo)], CAMADA_GRADE));
    }
    // Moldura.
    out.extend(polilinha(&[[x0, py(base)], [x0 + largura, py(base)], [x0 + largura, py(topo)], [x0, py(topo)], [x0, py(base)]], CAMADA_TEXTO));
    // Terreno e greide.
    out.extend(polilinha(&tn.iter().map(|(e, z)| [px(*e), py(*z)]).collect::<Vec<_>>(), CAMADA_TN));
    if let Some(g) = greide {
        // Só entre o primeiro e o último PVI (dentro do eixo).
        let (ga, gb) = if g.pivs.len() > 1 { g.faixa() } else { (al.estaca_inicial, al.estaca_final()) };
        let (ga, gb) = (ga.max(al.estaca_inicial), gb.min(al.estaca_final()));
        let n = (((gb - ga) / 0.5).ceil() as usize).clamp(2, 200_000);
        let pts: Vec<[f64; 2]> = if gb > ga {
            (0..=n)
                .map(|k| {
                    let e = ga + (gb - ga) * k as f64 / n as f64;
                    [px(e), py(g.cota(e))]
                })
                .collect()
        } else {
            Vec::new()
        };
        out.extend(polilinha(&pts, CAMADA_GREIDE));
        for p in &g.pivs {
            if p.estaca >= al.estaca_inicial - 1e-9 && p.estaca <= al.estaca_final() + 1e-9 {
                let c = [px(p.estaca), py(p.cota)];
                out.push(linha([c[0] - alt * 0.4, c[1] - alt * 0.4], [c[0] + alt * 0.4, c[1] + alt * 0.4], CAMADA_GREIDE));
                out.push(linha([c[0] - alt * 0.4, c[1] + alt * 0.4], [c[0] + alt * 0.4, c[1] - alt * 0.4], CAMADA_GREIDE));
                out.push(texto(format!("PVI {} / {:.3}", texto_da_estaca(p.estaca), p.cota), [c[0], c[1] + alt * 1.5], alt * 0.8, 0.0, true, CAMADA_GREIDE));
            }
        }
    }
    // Faixa de cotas embaixo: só as linhas que têm dado.
    let mut linhas_faixa = vec!["Station", "Terrain"];
    if greide.is_some() {
        linhas_faixa.extend(["Design", "Fill(+)/Cut(-)"]);
    }
    let h = alt * 7.0;
    let fundo = y0 - h * linhas_faixa.len() as f64;
    for (k, r) in linhas_faixa.iter().enumerate() {
        rotulo(&mut out, r.to_string(), [x0 - alt * 0.6, y0 - h * (k as f64 + 0.5)], true);
        out.push(linha([x0, y0 - h * (k as f64 + 1.0)], [x0 + largura, y0 - h * (k as f64 + 1.0)], CAMADA_GRADE));
    }
    out.push(linha([x0, y0], [x0, fundo], CAMADA_TEXTO));
    out.push(linha([x0 + largura, y0], [x0 + largura, fundo], CAMADA_TEXTO));
    out.push(linha([x0, fundo], [x0 + largura, fundo], CAMADA_TEXTO));
    for e in &estacas {
        out.push(linha([px(*e), y0], [px(*e), fundo], CAMADA_GRADE));
        let x = px(*e);
        let q = al.ponto_na_estaca(*e).0;
        let zt = tin.cota_em(q[0], q[1]);
        let mut col = vec![texto_da_estaca(*e), zt.map_or(String::new(), |z| format!("{z:.3}"))];
        if greide.is_some() {
            col.push(gz(*e).map_or(String::new(), |z| format!("{z:.3}")));
            col.push(match (zt, gz(*e)) {
                (Some(t), Some(g)) => format!("{:+.3}", g - t),
                _ => String::new(),
            });
        }
        for (k, c) in col.into_iter().enumerate().filter(|(_, c)| !c.is_empty()) {
            out.push(texto(c, [x, y0 - h * (k as f64 + 0.5)], alt * 0.8, std::f64::consts::FRAC_PI_2, true, CAMADA_TEXTO));
        }
    }
    out.push(texto(
        format!("PROFILE {nome}   Surface: {superficie}   V {ex}x"),
        [x0 + largura / 2.0, py(topo) + alt * 2.5],
        alt * 1.6,
        0.0,
        true,
        CAMADA_TEXTO,
    ));
    (out, format!("Profile view: elevations {zmin:.2} to {zmax:.2}, {} stations.", estacas.len()), base, topo)
}

/// Dica do mouse dentro da vista de perfil: estaca e cota do ponto, e as cotas do terreno e do greide ali.
pub(crate) fn dica_da_vista(
    doc: &document::Document,
    tins: &std::collections::HashMap<String, (Vec<document::Handle>, Arc<Tin>)>,
    p: [f64; 2],
) -> Option<String> {
    for a in crate::civil::alinhamentos(doc) {
        let (Some(v), Some(g)) = (a.vista.as_ref(), a.geometria()) else { continue };
        if !(v.exagero > 0.0) || v.topo <= v.base {
            continue;
        }
        let (x0, y0) = (v.origem[0], v.origem[1]);
        if p[0] < x0 || p[0] > x0 + g.comprimento() || p[1] < y0 || p[1] > y0 + (v.topo - v.base) * v.exagero {
            continue;
        }
        let e = g.estaca_inicial + (p[0] - x0);
        let z = v.base + (p[1] - y0) / v.exagero;
        let mut t = format!("{} profile\nStation: {}\nElevation: {z:.3}", a.nome, texto_da_estaca(e));
        let q = g.ponto_na_estaca(e).0;
        if let Some((_, tin)) = tins.get(&a.superficie) {
            if let Some(zt) = tin.cota_em(q[0], q[1]) {
                t.push_str(&format!("\nTerrain: {zt:.3}"));
            }
        }
        if let Some(gr) = a.greide() {
            if let Some(z) = gr.cota_em(e) {
                t.push_str(&format!("\nDesign: {z:.3}"));
            }
        }
        return Some(t);
    }
    None
}

/// Linhas de seção na planta e as vistas de seção (escala 1:1), em grade.
fn desenho_das_secoes(
    tin: &Tin,
    al: &Alinhamento,
    greide: Option<&Greide>,
    s: &Secoes,
    gabarito: Option<&AssemblyBarragem>,
    extras: &[(String, Arc<Tin>)],
) -> (Vec<EntityType>, usize) {
    let mut out = Vec::new();
    let estacas = s.estacas_no(al);
    let alt = (s.intervalo / 8.0).clamp(0.5, 5.0);
    let largura = s.esquerda + s.direita;
    let passo = (largura / 200.0).max(0.25);
    let secoes: Vec<(f64, Vec<(f64, f64)>)> =
        estacas.iter().map(|e| (*e, secao_do_terreno(tin, al, *e, s.esquerda, s.direita, passo))).collect();
    let corr: Vec<Option<SecaoCorredor>> = estacas
        .iter()
        .map(|e| match (gabarito, greide) {
            (Some(a), Some(g)) => g.cota_em(*e).map(|z| secao_do_corredor(tin, al, *e, z, a)),
            _ => None,
        })
        .collect();
    // Altura de cada vista: a maior faixa de cotas entre as seções, para a grade ficar alinhada.
    let mut faixa: f64 = 2.0;
    for (e, sec) in &secoes {
        let mut lo = sec.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let mut hi = sec.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        if let Some(g) = greide {
            if let Some(z) = g.cota_em(*e) {
                lo = lo.min(z);
                hi = hi.max(z);
            }
        }
        if let Some(Some(c)) = corr.get(secoes.iter().position(|x| x.0 == *e).unwrap_or(0)) {
            for p in &c.pontos {
                lo = lo.min(p.1);
                hi = hi.max(p.1);
            }
        }
        if lo.is_finite() {
            faixa = faixa.max(hi - lo);
        }
    }
    let dz = passo_redondo(faixa, 4.0);
    let altura = (faixa / dz).ceil() * dz + 2.0 * dz;
    let (cw, ch) = (largura + alt * 8.0, altura + alt * 6.0);
    let colunas = s.colunas.max(1);
    let mut feitas = 0;
    for (k, (e, sec)) in secoes.iter().enumerate() {
        // Linha de seção na planta.
        let a = al.ponto_com_offset(*e, -s.esquerda);
        let b = al.ponto_com_offset(*e, s.direita);
        out.push(linha(a, b, CAMADA_SECAO));
        if sec.len() < 2 {
            continue;
        }
        feitas += 1;
        let (col, lin) = (k % colunas, k / colunas);
        let x0 = s.origem[0] + col as f64 * cw + alt * 6.0;
        let y_topo = s.origem[1] - lin as f64 * ch - alt * 3.0;
        let lo = sec.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let lo = greide.and_then(|g| g.cota_em(*e)).map_or(lo, |z| lo.min(z));
        let base = (lo / dz).floor() * dz - dz;
        let y0 = y_topo - altura;
        let px = |o: f64| x0 + (o + s.esquerda);
        let py = |z: f64| y0 + (z - base);
        // Grade: cotas a cada dz e afastamentos a cada 5 ou 10 m.
        let mut z = base;
        while z <= base + altura + 1e-9 {
            out.push(linha([x0, py(z)], [x0 + largura, py(z)], CAMADA_GRADE));
            out.push(texto(format!("{z:.1}"), [x0 - alt * 0.3, py(z)], alt * 0.5, 0.0, false, CAMADA_TEXTO));
            if let EntityType::Text(t) = out.last_mut().unwrap() {
                t.horizontal_alignment = TextHorizontalAlignment::Right;
                t.vertical_alignment = TextVerticalAlignment::Middle;
                t.alignment_point = Some(t.insertion_point);
            }
            z += dz;
        }
        let dof = passo_redondo(largura, 6.0);
        let mut o = -(s.esquerda / dof).floor() * dof;
        while o <= s.direita + 1e-9 {
            out.push(linha([px(o), py(base)], [px(o), py(base + altura)], CAMADA_GRADE));
            out.push(texto(format!("{o:.0}"), [px(o), py(base) - alt * 0.6], alt * 0.5, 0.0, true, CAMADA_TEXTO));
            o += dof;
        }
        // Eixo, terreno e o ponto do greide.
        out.push(linha([px(0.0), py(base)], [px(0.0), py(base + altura)], CAMADA_EIXO));
        out.extend(polilinha(&sec.iter().map(|(o, z)| [px(*o), py(*z)]).collect::<Vec<_>>(), CAMADA_TN));
        // Sample More Sources: as outras superfícies, cada uma com o nome na ponta direita.
        for (nome_f, t2) in extras {
            let s2 = secao_do_terreno(t2, al, *e, s.esquerda, s.direita, passo);
            if let Some(ult) = s2.last() {
                out.extend(polilinha(&s2.iter().map(|(o, z)| [px(*o), py(*z)]).collect::<Vec<_>>(), CAMADA_FONTE));
                out.push(texto(nome_f.clone(), [px(ult.0) + alt * 0.4, py(ult.1)], alt * 0.5, 0.0, false, CAMADA_FONTE));
            }
        }
        if let Some(Some(c)) = corr.get(k) {
            // Gabarito do corredor (só o que cabe na largura da vista) e as áreas.
            let pts: Vec<[f64; 2]> = c
                .pontos
                .iter()
                .filter(|p| p.0 >= -s.esquerda - 1e-9 && p.0 <= s.direita + 1e-9)
                .map(|p| [px(p.0), py(p.1)])
                .collect();
            out.extend(polilinha(&pts, CAMADA_CORREDOR));
            out.push(texto(
                format!("Cut {:.2} m²  Fill {:.2} m²", c.area_corte, c.area_aterro),
                [x0 + largura / 2.0, y0 - alt * 1.8],
                alt * 0.6,
                0.0,
                true,
                CAMADA_TEXTO,
            ));
        }
        if let Some(zg) = greide.and_then(|g| g.cota_em(*e)) {
            let c = [px(0.0), py(zg)];
            out.push(linha([c[0] - alt, c[1]], [c[0] + alt, c[1]], CAMADA_GREIDE));
            out.push(texto(format!("{zg:.3}"), [c[0], c[1] + alt * 0.6], alt * 0.5, 0.0, true, CAMADA_GREIDE));
        }
        let zt = tin.cota_em(al.ponto_na_estaca(*e).0[0], al.ponto_na_estaca(*e).0[1]);
        let titulo = match zt {
            Some(z) => format!("{}  TN {:.3}", texto_da_estaca(*e), z),
            None => texto_da_estaca(*e),
        };
        out.push(texto(titulo, [x0 + largura / 2.0, y_topo + alt * 1.2], alt * 0.8, 0.0, true, CAMADA_TEXTO));
    }
    (out, feitas)
}
