//! Agral Civil: pacote de fábrica do Agral CAD, com os comandos do Civil 3D em inglês (pedido do JP, 17/11).
//!
//! Entrega 001 (superfície), primeiro corte:
//! - `CREATESURFACE [nome] [descrição]`: superfície TIN vazia, gravada no DWG.
//! - `ADDSURFACECONTOURS [superfície] [camada]`: acrescenta curvas de nível (seleção prévia, seleção na tela ou
//!   todas as polilinhas da camada), diagnostica cruzamentos e refaz.
//! - `REBUILDSURFACE [superfície]`: refaz a partir da lista de definição.
//!
//! O cálculo fica no `agral_civil_motor` (sem tela, testado sozinho).

pub mod curvas;
pub mod definicao;

use agral_civil_motor::diagnostico::diagnosticar_curvas;
use agral_civil_motor::superficie::{ItemDefinicao, Superficie};
use agral_civil_motor::tin::Tin;
use definicao::{ItemDwg, SuperficieDwg, DICIONARIO};
use ocs_plugin_api::dialog::{DialogControl, DialogDef, DialogResponse, DialogRow, DialogValue};
use ocs_plugin_api::export_plugin;
use ocs_plugin_api::host::codec::entities::{Circle, Mesh, MeshFace};
use ocs_plugin_api::host::codec::Vector3;
use ocs_plugin_api::host::{
    BuiltinPlugin, CommandStep, EntityType, ExtendedDataRecord, Handle, HostApi, InteractiveCommand, LayerConfig,
    ReaderEntityKind, XDataValue,
};
use ocs_plugin_api::manifest::{ApiVersion, PluginManifest};
use ocs_plugin_api::ribbon::{CadModule, IconKind, ModuleEvent, RibbonGroup, RibbonItem, ToolDef};
use std::sync::Mutex;
use std::time::Instant;

const APP: &str = "AGRAL_CIVIL";
const CAMADA_DIAGNOSTICO: &str = "AGRAL-DIAG-CRUZAMENTO";
/// MESH com até 100 mil vértices cada (o mesmo limite que o Open Infra usa para manter o desenho leve).
const VERTICES_POR_MALHA: usize = 100_000;
/// O pacote roda em outro processo e cada mensagem tem até 64 MB: o desenho é lido e escrito em lotes.
/// Medido no `CURVAS_V1` (58 MB): o desenho inteiro numa mensagem dá 85 MB.
const ENTIDADES_POR_LOTE: usize = 200;

/// Lê as entidades em lotes, sem copiar o desenho inteiro para o pacote.
fn ler_em_lotes(host: &dyn HostApi, handles: &[Handle], mut f: impl FnMut(Handle, Option<&EntityType>)) -> Result<(), String> {
    for lote in handles.chunks(ENTIDADES_POR_LOTE) {
        let ents = host.entities_batch(lote)?;
        for (h, e) in lote.iter().zip(ents.iter()) {
            f(*h, e.as_ref());
        }
    }
    Ok(())
}

/// Polilinhas da camada, pela visão compartilhada do desenho (sem cópia).
fn polilinhas_da_camada(host: &dyn HostApi, camada: &str) -> Vec<Handle> {
    let leitor = host.document_reader();
    let mut v = Vec::new();
    leitor.for_each_entity(&mut |e| {
        if e.kind == ReaderEntityKind::Polyline && e.layer_name.eq_ignore_ascii_case(camada) {
            v.push(e.handle);
        }
    });
    v
}

static MANIFEST: PluginManifest = PluginManifest {
    id: "agral.civil",
    name: "Agral Civil",
    version: env!("CARGO_PKG_VERSION"),
    description: "Superfícies com os comandos do Civil 3D. Agral Engenharia.",
    api_version: ApiVersion::CURRENT,
    ribbon_order: 10,
    xdata_apps: &[APP],
    command_prefixes: &["CREATESURFACE", "ADDSURFACECONTOURS", "REBUILDSURFACE"],
};

/// Seleção feita na tela por `ADDSURFACECONTOURS`, entregue ao comando que ela dispara.
static SELECAO: Mutex<Vec<Handle>> = Mutex::new(Vec::new());
const MARCA_SELECAO: &str = "*SELECAO*";

struct Pacote;

impl BuiltinPlugin for Pacote {
    fn manifest(&self) -> &'static PluginManifest {
        &MANIFEST
    }

    fn ribbon(&self) -> Box<dyn CadModule> {
        Box::new(Aba)
    }

    fn dispatch(&self, host: &mut dyn HostApi, linha: &str) -> bool {
        let mut partes = linha.split_whitespace();
        let Some(cmd) = partes.next() else { return false };
        let args: Vec<&str> = partes.collect();
        let r = match cmd.to_ascii_uppercase().as_str() {
            "CREATESURFACE" => create_surface(host, &args),
            "ADDSURFACECONTOURS" => add_surface_contours(host, &args),
            "REBUILDSURFACE" => rebuild_surface(host, &args),
            _ => return false,
        };
        match r {
            Ok(Some(msg)) => host.push_info(&msg),
            Ok(None) => {}
            Err(e) => host.push_error(&e),
        }
        true
    }
}

// ── Superfícies gravadas ───────────────────────────────────────────────────

fn superficies(host: &dyn HostApi) -> Vec<String> {
    host.list_xrecords(&DICIONARIO).unwrap_or_default()
}

fn ler(host: &dyn HostApi, nome: &str) -> Result<SuperficieDwg, String> {
    let e = host.read_xrecord(&DICIONARIO, nome).map_err(|_| format!("Surface '{nome}' not found."))?;
    SuperficieDwg::de_xrecord(nome, &e)
}

fn gravar(host: &mut dyn HostApi, s: &SuperficieDwg, nova: bool) -> Result<(), String> {
    let e = s.para_xrecord();
    if nova {
        host.create_xrecord(&DICIONARIO, &s.nome, e)?;
    } else {
        host.update_xrecord(&DICIONARIO, &s.nome, e)?;
    }
    host.set_dirty();
    Ok(())
}

/// Superfície pelo nome dado, ou a única que existe.
fn escolher(host: &dyn HostApi, nome: Option<&str>) -> Result<String, String> {
    let todas = superficies(host);
    if let Some(n) = nome {
        return todas
            .iter()
            .find(|s| s.eq_ignore_ascii_case(n))
            .cloned()
            .ok_or_else(|| format!("Surface '{n}' not found. Surfaces: {}", lista(&todas)));
    }
    match todas.len() {
        0 => Err("No surface in the drawing. Run CREATESURFACE first.".into()),
        1 => Ok(todas[0].clone()),
        _ => Err(format!("More than one surface; type the name. Surfaces: {}", lista(&todas))),
    }
}

fn lista(v: &[String]) -> String {
    if v.is_empty() {
        "(none)".into()
    } else {
        v.join(", ")
    }
}

// ── CREATESURFACE ──────────────────────────────────────────────────────────

fn create_surface(host: &mut dyn HostApi, args: &[&str]) -> Result<Option<String>, String> {
    let todas = superficies(host);
    let padrao = (1..).map(|i| format!("Surface{i}")).find(|n| !todas.iter().any(|s| s.eq_ignore_ascii_case(n))).unwrap();
    let (nome, descricao) = if let Some(n) = args.first() {
        (n.to_string(), args[1..].join(" "))
    } else {
        // Como a janela "Create Surface" do Civil 3D: nome e descrição.
        let mut d = DialogDef::new("Create Surface");
        d.rows = vec![
            DialogRow::new(vec![DialogControl::InfoText { text: "Type: TIN surface".into(), align_h: None }]),
            DialogRow::new(vec![DialogControl::Text { id: "name".into(), label: "Name".into(), default: padrao.clone() }]),
            DialogRow::new(vec![DialogControl::Text { id: "desc".into(), label: "Description".into(), default: String::new() }]),
        ];
        d.submit_label = "OK".into();
        match host.show_dialog(d) {
            DialogResponse::Submitted(v) => {
                let txt = |k: &str| match v.get(k) {
                    Some(DialogValue::Text(s)) => s.trim().to_string(),
                    _ => String::new(),
                };
                let n = txt("name");
                (if n.is_empty() { padrao } else { n }, txt("desc"))
            }
            DialogResponse::Cancelled => return Ok(None),
        }
    };
    if nome.chars().any(|c| matches!(c, '<' | '>' | '/' | '\\' | '"' | ':' | ';' | '?' | '*' | '|' | ',' | '=' | '`')) {
        return Err(format!("Invalid surface name '{nome}'."));
    }
    if todas.iter().any(|s| s.eq_ignore_ascii_case(&nome)) {
        return Err(format!("Surface '{nome}' already exists."));
    }
    let s = SuperficieDwg { nome: nome.clone(), descricao, ..Default::default() };
    gravar(host, &s, true)?;
    Ok(Some(format!("Surface '{nome}' created (empty TIN). Add data with ADDSURFACECONTOURS.")))
}

// ── ADDSURFACECONTOURS ─────────────────────────────────────────────────────

struct SelecionarCurvas {
    superficie: String,
}

impl InteractiveCommand for SelecionarCurvas {
    fn prompt(&self) -> String {
        format!("Select contour polylines to add to surface '{}':", self.superficie)
    }
    fn on_point(&mut self, _pt: [f64; 3]) -> CommandStep {
        CommandStep::NeedPoint
    }
    fn is_selection_gathering(&self) -> bool {
        true
    }
    fn accepts_drag_selection(&self) -> bool {
        true
    }
    fn on_selection_complete(&mut self, handles: Vec<Handle>) -> CommandStep {
        if handles.is_empty() {
            return CommandStep::NeedPoint;
        }
        *SELECAO.lock().unwrap() = handles;
        CommandStep::Dispatch(format!("ADDSURFACECONTOURS {} {MARCA_SELECAO}", self.superficie))
    }
}

fn add_surface_contours(host: &mut dyn HostApi, args: &[&str]) -> Result<Option<String>, String> {
    let nome = escolher(host, args.first().copied())?;
    let fontes: Vec<Handle> = match args.get(1) {
        Some(&MARCA_SELECAO) => std::mem::take(&mut *SELECAO.lock().unwrap()),
        Some(camada) => polilinhas_da_camada(host, camada),
        None => {
            let sel = host.selection();
            if sel.is_empty() {
                host.start_interactive(Box::new(SelecionarCurvas { superficie: nome }));
                return Ok(None);
            }
            sel
        }
    };
    let mut validas: Vec<Handle> = Vec::new();
    ler_em_lotes(host, &fontes, |h, e| {
        if e.and_then(curvas::pontos).is_some() {
            validas.push(h);
        }
    })?;
    if validas.is_empty() {
        return Err("No polylines selected.".into());
    }
    let mut s = ler(host, &nome)?;
    let n = validas.len();
    s.itens.push(ItemDwg { tipo: "CONTOURS".into(), ligado: true, fontes: validas });
    s.desatualizada = true;
    gravar(host, &s, false)?;
    let msg = refazer(host, &nome)?;
    Ok(Some(format!("{n} contours added to '{nome}'. {msg}")))
}

// ── REBUILDSURFACE ─────────────────────────────────────────────────────────

fn rebuild_surface(host: &mut dyn HostApi, args: &[&str]) -> Result<Option<String>, String> {
    let nome = escolher(host, args.first().copied())?;
    refazer(host, &nome).map(Some)
}

/// Monta o TIN a partir da lista de definição, troca o desenho antigo pelo novo e grava.
fn refazer(host: &mut dyn HostApi, nome: &str) -> Result<String, String> {
    let t0 = Instant::now();
    let mut s = ler(host, nome)?;
    let mut linhas: Vec<Vec<[f64; 3]>> = Vec::new();
    let mut sumidas = 0usize;
    let fontes: Vec<Handle> =
        s.itens.iter().filter(|i| i.ligado && i.tipo == "CONTOURS").flat_map(|i| i.fontes.iter().copied()).collect();
    ler_em_lotes(host, &fontes, |_, e| match e.and_then(curvas::pontos) {
        Some(p) => linhas.push(p),
        None => sumidas += 1,
    })?;
    let t_ler = t0.elapsed();
    let diag = diagnosticar_curvas(&linhas);
    let mut sup = Superficie::nova(&s.nome, &s.descricao);
    sup.opcoes.comprimento_maximo = s.comprimento_maximo;
    if !linhas.is_empty() {
        sup.acrescentar(ItemDefinicao::curvas("contours", linhas));
    }
    let montagem = sup.montar();
    let t_montar = t0.elapsed();

    host.begin_undo_group("Rebuild surface");
    for h in s.desenho.drain(..) {
        host.remove_entity(h);
    }
    let camada = format!("C-TOPO-{}", s.nome);
    let _ = host.add_layer(LayerConfig { name: camada.clone(), ..Default::default() });
    let mut novas = malhas(&montagem.tin, &camada, &s.nome);
    if !diag.cruzamentos.is_empty() {
        let _ = host.add_layer(LayerConfig { name: CAMADA_DIAGNOSTICO.into(), ..Default::default() });
        for c in &diag.cruzamentos {
            let mut m = Circle::new();
            m.center = Vector3::new(c.ponto[0], c.ponto[1], c.ponto[2]);
            m.radius = 1.0;
            let mut e = EntityType::Circle(m);
            e.common_mut().layer = CAMADA_DIAGNOSTICO.into();
            marcar(&mut e, &format!("CROSSING:{}", s.nome));
            novas.push(e);
        }
    }
    // Uma malha por mensagem (cada uma fica bem abaixo do limite de 64 MB); as marcas vão em lotes.
    s.desenho.clear();
    let mut lote: Vec<EntityType> = Vec::new();
    for e in novas {
        if matches!(e, EntityType::Mesh(_)) {
            s.desenho.extend(host.add_entities(vec![e]));
        } else {
            lote.push(e);
            if lote.len() == ENTIDADES_POR_LOTE {
                s.desenho.extend(host.add_entities(std::mem::take(&mut lote)));
            }
        }
    }
    s.desenho.extend(host.add_entities(lote));
    s.desatualizada = false;
    gravar(host, &s, false)?;
    host.end_undo_group();
    host.bump_geometry();
    let t_gravar = t0.elapsed();

    let mut msg = format!(
        "Surface '{}': {} points, {} triangles in {:.2} s.",
        s.nome,
        montagem.tin.vertices.len(),
        montagem.tin.triangulos.len(),
        t0.elapsed().as_secs_f64()
    );
    msg.push_str(&format!(
        " (read {:.1} s, build {:.1} s, draw {:.1} s)",
        t_ler.as_secs_f64(),
        (t_montar - t_ler).as_secs_f64(),
        (t_gravar - t_montar).as_secs_f64()
    ));
    if !diag.cruzamentos.is_empty() {
        msg.push_str(&format!(
            " WARNING: {} contour crossings (marked on layer {CAMADA_DIAGNOSTICO}); crossing segments were left out.",
            diag.cruzamentos.len()
        ));
    }
    if sumidas > 0 {
        msg.push_str(&format!(" {sumidas} source objects no longer exist."));
    }
    Ok(msg)
}

fn marcar(e: &mut EntityType, valor: &str) {
    let mut r = ExtendedDataRecord::new(APP);
    r.add_value(XDataValue::String(valor.to_string()));
    e.common_mut().extended_data.add_record(r);
}

/// O TIN como entidades MESH (poucas, não um 3DFACE por triângulo).
fn malhas(tin: &Tin, camada: &str, nome: &str) -> Vec<EntityType> {
    let mut saida = Vec::new();
    let mut mapa: Vec<u32> = vec![u32::MAX; tin.vertices.len()];
    let mut m = Mesh::new();
    let fechar = |m: &mut Mesh, mapa: &mut Vec<u32>, saida: &mut Vec<EntityType>| {
        if !m.faces.is_empty() {
            let mut e = EntityType::Mesh(std::mem::replace(m, Mesh::new()));
            e.common_mut().layer = camada.to_string();
            marcar(&mut e, &format!("SURFACE:{nome}"));
            saida.push(e);
        }
        mapa.iter_mut().for_each(|x| *x = u32::MAX);
    };
    for t in &tin.triangulos {
        if m.vertices.len() + 3 > VERTICES_POR_MALHA {
            fechar(&mut m, &mut mapa, &mut saida);
        }
        let f: Vec<usize> = t
            .iter()
            .map(|&i| {
                if mapa[i as usize] == u32::MAX {
                    let v = tin.vertices[i as usize];
                    m.vertices.push(Vector3::new(v[0], v[1], v[2]));
                    mapa[i as usize] = (m.vertices.len() - 1) as u32;
                }
                mapa[i as usize] as usize
            })
            .collect();
        m.faces.push(MeshFace::new(f));
    }
    fechar(&mut m, &mut mapa, &mut saida);
    saida
}

// ── Aba ────────────────────────────────────────────────────────────────────

struct Aba;

impl CadModule for Aba {
    fn id(&self) -> &'static str {
        MANIFEST.id
    }

    fn title(&self) -> &'static str {
        "Agral Civil"
    }

    fn ribbon_groups(&self) -> &[RibbonGroup] {
        static G: std::sync::OnceLock<Vec<RibbonGroup>> = std::sync::OnceLock::new();
        G.get_or_init(|| {
            let ferramenta = |id: &'static str, rotulo: &'static str, letra: &'static str| {
                RibbonItem::LargeTool(ToolDef {
                    id,
                    label: rotulo,
                    icon: IconKind::Glyph(letra),
                    event: ModuleEvent::Command(id.to_string()),
                })
            };
            vec![RibbonGroup {
                title: "Surfaces",
                tools: vec![
                    ferramenta("CREATESURFACE", "Create Surface", "S"),
                    ferramenta("ADDSURFACECONTOURS", "Add Contours", "C"),
                    ferramenta("REBUILDSURFACE", "Rebuild", "R"),
                ],
            }]
        })
    }
}

export_plugin!(Pacote);
