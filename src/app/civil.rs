//! Agral Civil no núcleo: ler, gravar, montar e apagar superfícies do desenho (no mesmo processo do programa, sem
//! o limite de mensagem e de tempo de um pacote).

use super::OpenCADStudio;
use crate::civil::curvas;
use crate::civil::definicao::{ItemDwg, SuperficieDwg, DICIONARIO};
use agral_civil_motor::correcao::corrigir_cruzamentos;
use agral_civil_motor::diagnostico::diagnosticar_curvas;
use agral_civil_motor::emenda::{emendar_pedacos, OpcoesEmenda};
use agral_civil_motor::superficie::{ItemDefinicao, Superficie};
use agral_civil_motor::tin::Tin;
use document::entities::{Circle, Mesh, MeshFace};
use document::xdata::{ExtendedDataRecord, XDataValue};
use document::{EntityType, Handle, Vector3};
use std::time::Instant;

pub(crate) const APP: &str = "AGRAL_CIVIL";
pub(crate) const CAMADA_DIAGNOSTICO: &str = "AGRAL-DIAG-CRUZAMENTO";
pub(crate) const CAMADA_EMENDAS: &str = "AGRAL-DIAG-EMENDA";
/// Camada da inserção da superfície (a do Civil 3D por padrão); os componentes ficam nas camadas do estilo.
pub(crate) const CAMADA_SUPERFICIE: &str = "C-TOPO";

/// Nome do bloco que guarda o desenho da superfície.
pub(crate) fn nome_do_bloco(superficie: &str) -> String {
    format!("AGRAL_SUPERFICIE_{superficie}")
}

/// Tira do desenho o que a última montagem da superfície desenhou: a inserção, o bloco e as marcas.
fn apagar_desenho(scene: &mut crate::scene::Scene, s: &SuperficieDwg) {
    let bloco = nome_do_bloco(&s.nome);
    scene.erase_entities(&s.desenho);
    if scene.document.block_records.get(&bloco).is_some() {
        crate::scene::block_service::erase(scene, &bloco);
    }
}
const VERTICES_POR_MALHA: usize = 100_000;

pub(crate) fn nome_valido(nome: &str) -> bool {
    !nome.trim().is_empty()
        && !nome.chars().any(|c| matches!(c, '<' | '>' | '/' | '\\' | '"' | ':' | ';' | '?' | '*' | '|' | ',' | '=' | '`') || c.is_whitespace())
}

/// Tempo com o mouse parado até a etiqueta da cota aparecer.
pub(crate) const ESPERA_COTA: std::time::Duration = std::time::Duration::from_millis(2000);

fn esperar(t: std::time::Duration) -> iced::Task<super::Message> {
    let (tx, rx) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        std::thread::sleep(t);
        let _ = tx.send(());
    });
    iced::Task::perform(async move { rx.await.ok() }, |_| super::Message::CivilCotaRelogio)
}

/// TINs montados em segundo plano para a cota no mouse: (aba, geração do documento, superfícies).
#[derive(Clone)]
pub(crate) struct TinsProntos(pub u64, pub u64, pub Vec<(String, Vec<Handle>, std::sync::Arc<Tin>)>);

impl std::fmt::Debug for TinsProntos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TinsProntos(tab {}, {} surfaces)", self.0, self.2.len())
    }
}

/// Cota da superfície no ponto, para a etiqueta do mouse: a primeira superfície montada que cobre o ponto e
/// cujo desenho ainda é o da montagem. Devolve (nome, cota).
pub(crate) fn cota_no_ponto(
    tins: &std::collections::HashMap<String, (Vec<Handle>, std::sync::Arc<Tin>)>,
    doc: &document::Document,
    x: f64,
    y: f64,
) -> Option<(String, f64)> {
    let mut nomes: Vec<&String> = tins.keys().collect();
    nomes.sort();
    nomes.into_iter().find_map(|n| {
        let (desenho, tin) = &tins[n];
        let vale = desenho.first().is_some_and(|h| doc.get_entity(*h).is_some());
        if !vale {
            return None;
        }
        tin.cota_em(x, y).map(|z| (n.clone(), z))
    })
}

/// Nome da superfície que esta entidade desenha (a inserção marcada com XDATA `SURFACE:<nome>`).
pub(crate) fn superficie_da_entidade(e: &EntityType) -> Option<String> {
    e.common().extended_data.records().iter().find(|r| r.application_name == APP).and_then(|r| {
        r.values.iter().find_map(|v| match v {
            XDataValue::String(t) => t.strip_prefix("SURFACE:").map(str::to_string),
            _ => None,
        })
    })
}

impl OpenCADStudio {
    /// Seções do Properties para a superfície selecionada, como o Civil 3D mostra a "TIN Surface" (pedido do
    /// JP, 18/11): informação, estilo (editável aqui) e estatística. O resto se edita em Surface Properties.
    pub(crate) fn civil_secoes_propriedades(&self, i: usize, nome: &str) -> Option<Vec<crate::scene::model::object::PropSection>> {
        use crate::scene::model::object::{PropSection, PropValue, Property};
        let s = self.civil_ler(i, nome).ok()?;
        let so_ler = |label: &str, v: String| Property { label: label.into(), field: "civil_surface_info", value: PropValue::ReadOnly(v) };
        let mut info = vec![
            so_ler("Name", s.nome.clone()),
            so_ler("Description", s.descricao.clone()),
            Property {
                label: "Style".into(),
                field: "civil_surface_style",
                value: PropValue::Choice {
                    selected: s.estilo.clone(),
                    options: crate::civil::estilos::nomes(),
                },
            },
            so_ler(
                "Maximum triangle length",
                s.comprimento_maximo.map_or("None".into(), |v| format!("{v:.3}")),
            ),
            so_ler("Out of date", if s.desatualizada { "Yes".into() } else { "No".into() }),
        ];
        let ligados = s.itens.iter().filter(|it| it.ligado).count();
        info.push(so_ler("Definition items", format!("{ligados} of {}", s.itens.len())));
        let mut secoes = vec![PropSection { title: "Surface".into(), props: info }];
        if let Some((_, tin)) = self.tabs[i].civil_tins.get(&s.nome) {
            let mut est = vec![
                so_ler("Number of points", tin.vertices.len().to_string()),
                so_ler("Number of triangles", tin.triangulos.len().to_string()),
            ];
            if let Some((z0, z1)) = tin.faixa_de_cotas() {
                est.push(so_ler("Minimum elevation", format!("{z0:.3}")));
                est.push(so_ler("Maximum elevation", format!("{z1:.3}")));
            }
            secoes.push(PropSection { title: "Statistics".into(), props: est });
        }
        Some(secoes)
    }

    /// Estilo trocado no Properties: grava e redesenha a superfície.
    pub(crate) fn civil_estilo_pelo_properties(&mut self, estilo: &str) -> iced::Task<super::Message> {
        let i = self.active_tab;
        let Some(nome) = self.civil_superficie_selecionada(i) else { return iced::Task::none() };
        let r = self.civil_ler(i, &nome).and_then(|s| {
            self.civil_propriedades(i, &nome, &s.nome, &s.descricao, estilo, s.comprimento_maximo)
        });
        match r {
            Ok(m) => self.command_line.push_info(&m),
            Err(e) => self.command_line.push_error(&e),
        }
        self.refresh_properties();
        iced::Task::none()
    }

    /// O mouse andou: esconde a etiqueta da cota e, se não houver relógio rodando, arma um para daqui a
    /// `ESPERA_COTA` (pedido do JP, 18/11: só aparece com o mouse parado uns 2 s, como no Civil 3D).
    pub(crate) fn civil_cota_mouse_andou(&mut self) -> iced::Task<super::Message> {
        self.civil_cota_mov = Instant::now();
        self.civil_cota_visivel = false;
        let nada = self.tabs.get(self.active_tab).is_none_or(|t| {
            t.civil_tins.is_empty() && crate::civil::alinhamentos(&t.scene.document).is_empty()
        });
        if self.civil_cota_relogio || nada {
            return iced::Task::none();
        }
        self.civil_cota_relogio = true;
        esperar(ESPERA_COTA)
    }

    /// O relógio tocou: mouse parado o bastante mostra a etiqueta; senão espera o que falta.
    pub(crate) fn civil_cota_relogio(&mut self) -> iced::Task<super::Message> {
        let parado = self.civil_cota_mov.elapsed();
        if parado >= ESPERA_COTA {
            self.civil_cota_relogio = false;
            self.civil_cota_visivel = true;
            iced::Task::none()
        } else {
            esperar(ESPERA_COTA - parado)
        }
    }

    /// Quando o mouse anda: se alguma superfície do desenho tem desenho mas não tem TIN na memória (desenho
    /// reaberto, desfazer), monta em segundo plano para a cota no mouse. Confere só quando o desenho mudou.
    pub(crate) fn civil_preparar_cota(&mut self) -> iced::Task<super::Message> {
        let i = self.active_tab;
        let Some(tab) = self.tabs.get_mut(i) else { return iced::Task::none() };
        if tab.civil_tins_pendente || tab.civil_tins_conferido == Some(tab.edit_revision) {
            return iced::Task::none();
        }
        tab.civil_tins_conferido = Some(tab.edit_revision);
        let doc = &tab.scene.document;
        let faltam: Vec<SuperficieDwg> = crate::civil::superficies(doc)
            .into_iter()
            .filter(|s| s.desenho.first().is_some_and(|h| doc.get_entity(*h).is_some()))
            .filter(|s| tab.civil_tins.get(&s.nome).is_none_or(|(d, _)| *d != s.desenho))
            .collect();
        if faltam.is_empty() {
            return iced::Task::none();
        }
        let entradas: Vec<(String, Vec<Handle>, EntradaSuperficie)> =
            faltam.iter().map(|s| (s.nome.clone(), s.desenho.clone(), self.civil_entrada(i, s))).collect();
        let tab = &mut self.tabs[i];
        tab.civil_tins_pendente = true;
        let (id, geracao) = (tab.id, tab.document_generation);
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let prontos = entradas
                .into_iter()
                .map(|(n, d, e)| (n, d, std::sync::Arc::new(montar_entrada(e, &std::sync::atomic::AtomicU8::new(0)).tin)))
                .collect();
            let _ = tx.send(TinsProntos(id, geracao, prontos));
        });
        iced::Task::perform(async move { rx.await.ok() }, |r| match r {
            Some(t) => super::Message::CivilTinsProntos(t),
            None => super::Message::Noop,
        })
    }

    pub(crate) fn civil_tins_prontos(&mut self, t: TinsProntos) {
        let TinsProntos(id, geracao, prontos) = t;
        let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) else { return };
        tab.civil_tins_pendente = false;
        if tab.document_generation != geracao {
            tab.civil_tins_conferido = None;
            return;
        }
        for (n, d, tin) in prontos {
            tab.civil_tins.insert(n, (d, tin));
        }
    }

    pub(crate) fn civil_superficies(&self, i: usize) -> Vec<String> {
        crate::civil::superficies(&self.tabs[i].scene.document).into_iter().map(|s| s.nome).collect()
    }

    pub(crate) fn civil_ler(&self, i: usize, nome: &str) -> Result<SuperficieDwg, String> {
        crate::civil::superficies(&self.tabs[i].scene.document)
            .into_iter()
            .find(|s| s.nome.eq_ignore_ascii_case(nome))
            .ok_or_else(|| format!("Surface '{nome}' not found."))
    }

    fn civil_gravar(&mut self, i: usize, s: &SuperficieDwg, nova: bool) -> Result<(), String> {
        let mut host = crate::app::plugin_host::HostSession::new(self, i);
        let e = s.para_xrecord();
        if nova {
            host.create_xrecord(&DICIONARIO, &s.nome, e)?;
        } else {
            host.update_xrecord(&DICIONARIO, &s.nome, e)?;
        }
        self.tabs[i].dirty = true;
        Ok(())
    }

    /// Nome da superfície: o dado, ou a única que existe.
    pub(crate) fn civil_escolher(&self, i: usize, nome: Option<&str>) -> Result<String, String> {
        // Sem nome: a superfície selecionada no desenho, como nos comandos da aba TIN Surface.
        let selecionada = if nome.is_none() { self.civil_superficie_selecionada(i) } else { None };
        let nome = nome.or(selecionada.as_deref());
        let todas = self.civil_superficies(i);
        let lista = || if todas.is_empty() { "(none)".to_string() } else { todas.join(", ") };
        if let Some(n) = nome {
            return todas
                .iter()
                .find(|s| s.eq_ignore_ascii_case(n))
                .cloned()
                .ok_or_else(|| format!("Surface '{n}' not found. Surfaces: {}", lista()));
        }
        match todas.len() {
            0 => Err("No surface in the drawing. Run CREATESURFACE first.".into()),
            1 => Ok(todas[0].clone()),
            _ => Err(format!("More than one surface; type its name. Surfaces: {}", lista())),
        }
    }

    pub(crate) fn civil_proximo_nome(&self, i: usize) -> String {
        let todas = self.civil_superficies(i);
        (1..).map(|k| format!("Surface{k}")).find(|n| !todas.iter().any(|s| s.eq_ignore_ascii_case(n))).unwrap()
    }

    pub(crate) fn civil_criar(&mut self, i: usize, nome: &str, descricao: &str, estilo: &str) -> Result<String, String> {
        let nome = nome.trim();
        if !nome_valido(nome) {
            return Err(format!("Invalid surface name '{nome}' (no spaces or <>/\\\":;?*|,=`)."));
        }
        if self.civil_superficies(i).iter().any(|s| s.eq_ignore_ascii_case(nome)) {
            return Err(format!("Surface '{nome}' already exists."));
        }
        self.push_undo_snapshot(i, "CREATESURFACE");
        let s = SuperficieDwg {
            nome: nome.to_string(),
            descricao: descricao.trim().to_string(),
            estilo: estilo.to_string(),
            ..Default::default()
        };
        self.civil_gravar(i, &s, true)?;
        Ok(format!("Surface '{nome}' created (empty TIN surface)."))
    }

    pub(crate) fn civil_acrescentar_curvas(
        &mut self,
        i: usize,
        nome: &str,
        item: ItemDwg,
    ) -> Result<String, String> {
        let doc = &self.tabs[i].scene.document;
        let fontes: Vec<Handle> =
            item.fontes.iter().copied().filter(|h| doc.get_entity(*h).and_then(curvas::pontos).is_some()).collect();
        if fontes.is_empty() {
            return Err("No polylines in the selection.".into());
        }
        let n = fontes.len();
        let mut s = self.civil_ler(i, nome)?;
        s.itens.push(ItemDwg { fontes, ..item });
        s.desatualizada = true;
        self.push_undo_snapshot(i, "ADDSURFACECONTOURS");
        self.civil_gravar(i, &s, false)?;
        let msg = self.civil_refazer(i, nome)?;
        Ok(format!("{n} contours added to '{nome}'. {msg}"))
    }

    /// Acrescenta um limite ou um edit à definição e refaz. Limite: só polilinhas fechadas.
    pub(crate) fn civil_acrescentar_item(&mut self, i: usize, nome: &str, mut item: ItemDwg) -> Result<String, String> {
        if item.tipo == "BREAKLINES" {
            let doc = &self.tabs[i].scene.document;
            item.fontes.retain(|h| doc.get_entity(*h).and_then(curvas::pontos).is_some_and(|p| p.len() >= 2));
            if item.fontes.is_empty() {
                return Err("Select polylines or lines for the breaklines.".into());
            }
        }
        if item.tipo.starts_with("BOUNDARY") {
            let doc = &self.tabs[i].scene.document;
            item.fontes.retain(|h| {
                doc.get_entity(*h).and_then(curvas::pontos).is_some_and(|p| p.len() > 3 && p.first() == p.last())
            });
            if item.fontes.is_empty() {
                return Err("Select closed polylines for the boundary.".into());
            }
        }
        let mut s = self.civil_ler(i, nome)?;
        let rotulo = item.tipo.clone();
        s.itens.push(item);
        s.desatualizada = true;
        self.push_undo_snapshot(i, "SURFACEDEFINITION");
        self.civil_gravar(i, &s, false)?;
        let msg = self.civil_refazer(i, nome)?;
        Ok(format!("{rotulo} added to '{nome}'. {msg}"))
    }

    /// Importa as superfícies do LandXML: cria uma superfície por `<Surface>` (nome livre: acrescenta (2), (3)...)
    /// com um item LANDXML que aponta para o arquivo, e monta cada uma.
    pub(crate) fn civil_importar_landxml(&mut self, i: usize, caminho: &str) -> Result<String, String> {
        let xml = std::fs::read_to_string(caminho).map_err(|e| format!("Could not read {caminho} ({e})."))?;
        let lidas = agral_civil_motor::landxml::ler_superficies(&xml).map_err(|e| format!("LandXML: {e}"))?;
        if lidas.is_empty() {
            return Err("The LandXML file has no TIN surface.".into());
        }
        let mut msgs = Vec::new();
        for l in lidas {
            let base: String = l.nome.chars().map(|c| if nome_valido(&c.to_string()) { c } else { '_' }).collect();
            let base = if base.is_empty() { "Surface".to_string() } else { base };
            let mut nome = base.clone();
            let mut k = 2;
            while self.civil_superficies(i).iter().any(|x| x.eq_ignore_ascii_case(&nome)) {
                nome = format!("{base}({k})");
                k += 1;
            }
            self.civil_criar(i, &nome, &l.descricao, "")?;
            let item = ItemDwg::novo("LANDXML", &format!("{caminho}|{}", l.nome));
            msgs.push(self.civil_acrescentar_item(i, &nome, item)?);
        }
        Ok(msgs.join(" "))
    }

    /// Grava a superfície montada em LandXML 1.2 (o Civil 3D abre).
    pub(crate) fn civil_exportar_landxml(&mut self, i: usize, nome: &str, caminho: &str) -> Result<String, String> {
        let s = self.civil_ler(i, nome)?;
        let tin = match self.tabs[i].civil_tins.get(&s.nome) {
            Some((_, t)) => t.clone(),
            None => {
                let e = self.civil_entrada(i, &s);
                std::sync::Arc::new(montar_entrada(e, &std::sync::atomic::AtomicU8::new(0)).tin)
            }
        };
        if tin.triangulos.is_empty() {
            return Err(format!("Surface '{}' has no triangles to export.", s.nome));
        }
        let xml = agral_civil_motor::landxml::escrever_superficie(&s.nome, &s.descricao, &tin);
        std::fs::write(caminho, xml).map_err(|e| format!("Could not write {caminho} ({e})."))?;
        Ok(format!(
            "Surface '{}' exported to {caminho} ({} points, {} triangles).",
            s.nome,
            tin.vertices.len(),
            tin.triangulos.len()
        ))
    }

    pub(crate) fn civil_apagar(&mut self, i: usize, nome: &str) -> Result<String, String> {
        let s = self.civil_ler(i, nome)?;
        self.push_undo_snapshot(i, "DELETESURFACE");
        apagar_desenho(&mut self.tabs[i].scene, &s);
        let mut host = crate::app::plugin_host::HostSession::new(self, i);
        host.delete_xrecord(&DICIONARIO, nome)?;
        self.tabs[i].dirty = true;
        Ok(format!("Surface '{nome}' deleted (the contour polylines were kept)."))
    }

    pub(crate) fn civil_propriedades(
        &mut self,
        i: usize,
        nome: &str,
        novo_nome: &str,
        descricao: &str,
        estilo: &str,
        comprimento_maximo: Option<f64>,
    ) -> Result<String, String> {
        let mut s = self.civil_ler(i, nome)?;
        let novo_nome = novo_nome.trim();
        let renomear = !novo_nome.eq_ignore_ascii_case(nome);
        if renomear {
            if !nome_valido(novo_nome) {
                return Err(format!("Invalid surface name '{novo_nome}'."));
            }
            if self.civil_superficies(i).iter().any(|x| x.eq_ignore_ascii_case(novo_nome)) {
                return Err(format!("Surface '{novo_nome}' already exists."));
            }
        }
        let refazer = s.comprimento_maximo != comprimento_maximo || renomear || !s.estilo.eq_ignore_ascii_case(estilo);
        s.estilo = estilo.to_string();
        self.push_undo_snapshot(i, "SURFACEPROPERTIES");
        s.descricao = descricao.trim().to_string();
        s.comprimento_maximo = comprimento_maximo;
        if renomear {
            apagar_desenho(&mut self.tabs[i].scene, &s);
            s.desenho.clear();
            let mut host = crate::app::plugin_host::HostSession::new(self, i);
            host.delete_xrecord(&DICIONARIO, nome)?;
            s.nome = novo_nome.to_string();
            s.desatualizada = true;
            self.civil_gravar(i, &s, true)?;
        } else {
            if refazer {
                s.desatualizada = true;
            }
            self.civil_gravar(i, &s, false)?;
        }
        if refazer {
            return self.civil_refazer(i, &s.nome);
        }
        Ok(format!("Surface '{}' updated.", s.nome))
    }

    pub(crate) fn civil_ligar_item(&mut self, i: usize, nome: &str, k: usize, ligado: bool) -> Result<String, String> {
        let mut s = self.civil_ler(i, nome)?;
        let it = s.itens.get_mut(k).ok_or("No such definition item.")?;
        it.ligado = ligado;
        s.desatualizada = true;
        self.push_undo_snapshot(i, "SURFACEDEFINITION");
        self.civil_gravar(i, &s, false)?;
        self.civil_refazer(i, nome)
    }

    pub(crate) fn civil_remover_item(&mut self, i: usize, nome: &str, k: usize) -> Result<String, String> {
        let mut s = self.civil_ler(i, nome)?;
        if k >= s.itens.len() {
            return Err("No such definition item.".into());
        }
        s.itens.remove(k);
        s.desatualizada = true;
        self.push_undo_snapshot(i, "SURFACEDEFINITION");
        self.civil_gravar(i, &s, false)?;
        self.civil_refazer(i, nome)
    }

    /// `EXTRACTSURFACEOBJECTS`: copia para o desenho, como entidades comuns, o que a superfície mostra (curvas
    /// mestras, intermediárias, borda, triângulos), como o Extract Objects do Civil 3D. A superfície não muda.
    pub(crate) fn civil_extrair(&mut self, i: usize, nome: &str, camadas: &[&str]) -> Result<String, String> {
        let s = self.civil_ler(i, nome)?;
        let bloco = nome_do_bloco(&s.nome);
        let doc = &self.tabs[i].scene.document;
        let Some(rec) = doc.block_records.get(&bloco) else {
            return Err(format!("Surface '{}' has nothing drawn; rebuild it first.", s.nome));
        };
        let copias: Vec<EntityType> = rec
            .entity_handles
            .iter()
            .filter_map(|h| doc.get_entity(*h))
            .filter(|e| camadas.iter().any(|c| e.common().layer.eq_ignore_ascii_case(c)))
            .map(|e| {
                let mut c = e.clone();
                let cm = c.common_mut();
                cm.handle = Handle::NULL;
                cm.owner_handle = Handle::NULL;
                cm.extended_data = Default::default();
                c
            })
            .collect();
        if copias.is_empty() {
            return Err("Nothing of that kind in the surface style.".into());
        }
        self.push_undo_snapshot(i, "EXTRACTSURFACEOBJECTS");
        let n = copias.len();
        self.tabs[i].scene.add_entities(copias);
        self.tabs[i].dirty = true;
        Ok(format!("{n} objects extracted from surface '{}' (layers: {}).", s.nome, camadas.join(", ")))
    }

    /// Lê do desenho o que a montagem da superfície precisa (rápido, na linha da tela). A montagem em si
    /// (`montar_entrada`) não toca no desenho e pode rodar em segundo plano.
    pub(crate) fn civil_entrada(&self, i: usize, s: &SuperficieDwg) -> EntradaSuperficie {
        let doc = &self.tabs[i].scene.document;
        let mut e = EntradaSuperficie {
            nome: s.nome.clone(),
            descricao: s.descricao.clone(),
            comprimento_maximo: s.comprimento_maximo,
            curvas: Vec::new(),
            quebras: Vec::new(),
            landxml: s
                .itens
                .iter()
                .filter(|it| it.ligado && it.tipo == "LANDXML")
                .map(|it| {
                    let (c, n) = it.descricao.rsplit_once('|').unwrap_or((it.descricao.as_str(), ""));
                    (c.to_string(), n.to_string())
                })
                .collect(),
            arquivos_pontos: s
                .itens
                .iter()
                .filter(|it| it.ligado && it.tipo.starts_with("POINTFILE"))
                .map(|it| (it.descricao.clone(), it.tipo.trim_start_matches("POINTFILE_").to_string()))
                .collect(),
            externos: Vec::new(),
            furos: Vec::new(),
            edits: s.itens.iter().filter(|it| it.ligado && it.tipo.starts_with("EDIT_")).cloned().collect(),
            sumidas: 0,
        };
        // Limites (polilinhas fechadas): o externo também serve de borda para a emenda não juntar pontas da borda.
        for it in s.itens.iter().filter(|it| it.ligado && it.tipo.starts_with("BOUNDARY")) {
            for h in &it.fontes {
                match doc.get_entity(*h).and_then(curvas::pontos) {
                    Some(p) if it.tipo == "BOUNDARY_HIDE" => e.furos.push(p),
                    Some(p) => e.externos.push(p),
                    None => e.sumidas += 1,
                }
            }
        }
        // Linhas de quebra: arestas obrigatórias do TIN, com a cota de cada vértice (R1 da spec 003).
        for it in s.itens.iter().filter(|it| it.ligado && it.tipo == "BREAKLINES") {
            for h in &it.fontes {
                match doc.get_entity(*h).and_then(curvas::pontos) {
                    Some(p) => e.quebras.push(p),
                    None => e.sumidas += 1,
                }
            }
        }
        for it in s.itens.iter().filter(|it| it.ligado && it.tipo == "CONTOURS") {
            let mut linhas = Vec::new();
            for h in &it.fontes {
                match doc.get_entity(*h).and_then(curvas::pontos) {
                    Some(p) => linhas.push(p),
                    None => e.sumidas += 1,
                }
            }
            e.curvas.push((linhas, it.corrigir, it.emendar));
        }
        e
    }

    /// `REBUILDSURFACE`: monta o TIN a partir da lista de definição e troca o desenho antigo pelo novo.
    pub(crate) fn civil_refazer(&mut self, i: usize, nome: &str) -> Result<String, String> {
        let t0 = Instant::now();
        let s = self.civil_ler(i, nome)?;
        if self.civil_assincrono {
            return Ok(self.civil_refazer_em_segundo_plano(i, s));
        }
        let entrada = self.civil_entrada(i, &s);
        let sumidas = entrada.sumidas;
        let t_ler = t0.elapsed();
        let m = montar_entrada(entrada, &std::sync::atomic::AtomicU8::new(0));
        self.civil_desenhar(i, s, m, sumidas, t0, t_ler)
    }

    /// Na janela: a montagem roda fora da linha da tela, com a barra de progresso (pedido do JP, 18/11: "parece
    /// que trava"). Uma montagem por vez; as outras esperam na fila.
    fn civil_refazer_em_segundo_plano(&mut self, i: usize, s: SuperficieDwg) -> String {
        let tab = self.tabs[i].id;
        if self.civil_montando.is_some() {
            if !self.civil_fila.iter().any(|(t, n)| *t == tab && n.eq_ignore_ascii_case(&s.nome)) {
                self.civil_fila.push((tab, s.nome.clone()));
            }
            return format!("Surface '{}' queued; another surface is being built.", s.nome);
        }
        let entrada = self.civil_entrada(i, &s);
        let fase = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0));
        let pendente = MontagemPendente {
            tab,
            geracao: self.tabs[i].document_generation,
            definicao: s.clone(),
            inicio: Instant::now(),
            fase: fase.clone(),
        };
        let nome = s.nome.clone();
        self.civil_montando = Some(pendente);
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let sumidas = entrada.sumidas;
            let m = montar_entrada(entrada, &fase);
            let _ = tx.send(MontagemPronta(std::sync::Arc::new(std::sync::Mutex::new(Some((m, sumidas))))));
        });
        self.civil_tarefas.push(iced::Task::perform(async move { rx.await.ok() }, |r| match r {
            Some(m) => super::Message::CivilMontagemPronta(m),
            None => super::Message::Noop,
        }));
        format!("Building surface '{nome}'...")
    }

    /// A montagem em segundo plano terminou: desenha (se o desenho e a definição ainda são os mesmos) e começa a
    /// próxima da fila.
    pub(crate) fn civil_montagem_pronta(&mut self, pronta: MontagemPronta) -> iced::Task<super::Message> {
        let Some(p) = self.civil_montando.take() else { return iced::Task::none() };
        let Some((m, sumidas)) = pronta.0.lock().ok().and_then(|mut g| g.take()) else { return iced::Task::none() };
        if let Some(i) = self.tabs.iter().position(|t| t.id == p.tab && t.document_generation == p.geracao) {
            match self.civil_ler(i, &p.definicao.nome) {
                // A definição mudou durante a montagem: monta de novo com a nova.
                Ok(atual) if atual != p.definicao => {
                    let msg = self.civil_refazer_em_segundo_plano(i, atual);
                    self.command_line.push_info(&msg);
                }
                Ok(atual) => {
                    let t_ler = std::time::Duration::ZERO;
                    match self.civil_desenhar(i, atual, m, sumidas, p.inicio, t_ler) {
                        Ok(msg) => self.command_line.push_info(&msg),
                        Err(e) => self.command_line.push_error(&e),
                    }
                    self.refresh_properties();
                }
                Err(_) => {}
            }
        }
        while self.civil_montando.is_none() && !self.civil_fila.is_empty() {
            let (tab, nome) = self.civil_fila.remove(0);
            if let Some(i) = self.tabs.iter().position(|t| t.id == tab) {
                if let Ok(s) = self.civil_ler(i, &nome) {
                    let msg = self.civil_refazer_em_segundo_plano(i, s);
                    self.command_line.push_info(&msg);
                }
            }
        }
        iced::Task::batch(std::mem::take(&mut self.civil_tarefas))
    }

    fn civil_desenhar(
        &mut self,
        i: usize,
        mut s: SuperficieDwg,
        m: MontagemSuperficie,
        sumidas: usize,
        t0: Instant,
        t_ler: std::time::Duration,
    ) -> Result<String, String> {
        let MontagemSuperficie { tin, avisos, cruzamentos_achados, ligacoes, corrigidos, emendas, restantes } = m;
        let montagem_tin = std::sync::Arc::new(tin);
        let t_montar = t0.elapsed();

        let estilo = crate::civil::estilos::estilo(&s.estilo);
        // A superfície é um objeto só, como no Civil 3D (pedido do JP, 18/11): o desenho do estilo vai num bloco
        // e entra no desenho como uma inserção. Dentro do bloco cada componente fica na sua camada do template.
        let conteudo = desenho_do_estilo(&montagem_tin, estilo, &s.nome);
        let scene = &mut self.tabs[i].scene;
        apagar_desenho(scene, &s);
        for camada in conteudo.iter().map(|e| e.common().layer.clone()).collect::<std::collections::BTreeSet<_>>() {
            let nova = !scene.document.layers.contains(&camada);
            scene.ensure_layer(&camada);
            // Camada que o template da Agral define: criada com a cor e a espessura dele.
            if let (true, Some((_, cor, esp))) = (nova, crate::civil::estilos::CAMADAS.iter().find(|c| c.0 == camada)) {
                if let Some(l) = scene.document.layers.get_mut(&camada) {
                    l.color = document::Color::Index(*cor);
                    if *esp > 0 {
                        l.line_weight = document::types::LineWeight::Value(*esp);
                    }
                }
            }
        }
        let bloco = nome_do_bloco(&s.nome);
        scene.ensure_layer(CAMADA_SUPERFICIE);
        let mut novas: Vec<EntityType> = Vec::new();
        if !conteudo.is_empty() {
            scene.define_block_raw(&bloco, Vector3::new(0.0, 0.0, 0.0), conteudo);
            let mut ins = document::entities::Insert::new(bloco.clone(), Vector3::new(0.0, 0.0, 0.0));
            ins.common.layer = CAMADA_SUPERFICIE.into();
            let mut e = EntityType::Insert(ins);
            marcar(&mut e, &format!("SURFACE:{}", s.nome));
            novas.push(e);
        }
        if !cruzamentos_achados.is_empty() {
            scene.ensure_layer(CAMADA_DIAGNOSTICO);
            for p in &cruzamentos_achados {
                let mut m = Circle::new();
                m.center = Vector3::new(p[0], p[1], p[2]);
                m.radius = 1.0;
                let mut e = EntityType::Circle(m);
                e.common_mut().layer = CAMADA_DIAGNOSTICO.into();
                marcar(&mut e, &format!("CROSSING:{}", s.nome));
                novas.push(e);
            }
        }
        if !ligacoes.is_empty() {
            scene.ensure_layer(CAMADA_EMENDAS);
            for [a, b] in &ligacoes {
                let mut l = document::entities::Line::new();
                l.start = Vector3::new(a[0], a[1], a[2]);
                l.end = Vector3::new(b[0], b[1], b[2]);
                let mut e = EntityType::Line(l);
                e.common_mut().layer = CAMADA_EMENDAS.into();
                marcar(&mut e, &format!("JOIN:{}", s.nome));
                novas.push(e);
            }
        }
        s.desenho = scene.add_entities(novas);
        s.desatualizada = false;
        self.civil_gravar(i, &s, false)?;
        self.tabs[i].civil_tins.insert(s.nome.clone(), (s.desenho.clone(), montagem_tin.clone()));
        let t_desenhar = t0.elapsed();

        let mut msg = format!(
            "Surface '{}': {} points, {} triangles in {:.1} s (read {:.1}, build {:.1}, draw {:.1}).",
            s.nome,
            montagem_tin.vertices.len(),
            montagem_tin.triangulos.len(),
            t_desenhar.as_secs_f64(),
            t_ler.as_secs_f64(),
            (t_montar - t_ler).as_secs_f64(),
            (t_desenhar - t_montar).as_secs_f64()
        );
        if !cruzamentos_achados.is_empty() {
            if corrigidos > 0 {
                msg.push_str(&format!(
                    " {corrigidos} contour crossings found and fixed for the surface (polylines unchanged; marked on layer {CAMADA_DIAGNOSTICO})."
                ));
            } else {
                msg.push_str(&format!(" WARNING: {} contour crossings (marked on layer {CAMADA_DIAGNOSTICO}).", cruzamentos_achados.len()));
            }
        }
        if restantes > 0 {
            msg.push_str(&format!(" {restantes} crossings remain."));
        }
        if emendas > 0 {
            msg.push_str(&format!(" {emendas} contour gaps joined (marked on layer {CAMADA_EMENDAS}; check them, a gap at the edge of the area should not be joined)."));
        }
        if sumidas > 0 {
            msg.push_str(&format!(" {sumidas} source objects no longer exist."));
        }
        for a in &avisos {
            msg.push_str(&format!(" WARNING: {a}"));
        }
        // Perfis e seções dos alinhamentos que usam esta superfície acompanham a superfície nova.
        for m in self.civil_alinhamentos_da_superficie(i, &s.nome) {
            msg.push_str(&format!(" {m}"));
        }
        Ok(msg)
    }
}

/// O que o estilo manda desenhar: curvas intermediárias e mestras, borda e triângulos.
fn desenho_do_estilo(tin: &Tin, estilo: &crate::civil::estilos::Estilo, nome: &str) -> Vec<EntityType> {
    use document::entities::{LwPolyline, Polyline3D};
    use crate::civil::estilos::{CAMADA_BORDA, CAMADA_INTERMEDIARIA, CAMADA_MESTRA, CAMADA_TRIANGULOS};
    use document::Vector2;
    let mut v = Vec::new();
    if estilo.triangulos {
        v.extend(malhas(tin, CAMADA_TRIANGULOS.0, nome));
    }
    if estilo.curvas {
        if let Some((zmin, zmax)) = tin.faixa_de_cotas() {
            for (z, mestra) in crate::civil::estilos::cotas(estilo, zmin, zmax) {
                let camada = if mestra { CAMADA_MESTRA.0 } else { CAMADA_INTERMEDIARIA.0 };
                for c in tin.curvas_de_nivel(z) {
                    let fechada = c.len() > 3 && c.first() == c.last();
                    let n = if fechada { c.len() - 1 } else { c.len() };
                    let mut p = LwPolyline::from_points(c[..n].iter().map(|q| Vector2::new(q[0], q[1])).collect());
                    p.elevation = z;
                    p.is_closed = fechada;
                    let mut e = EntityType::LwPolyline(p);
                    e.common_mut().layer = camada.to_string();
                    marcar(&mut e, &format!("CONTOUR:{nome}:{z}"));
                    v.push(e);
                }
            }
        }
    }
    if estilo.borda {
        for anel in tin.borda() {
            let mut p = Polyline3D::from_points(anel.iter().map(|q| Vector3::new(q[0], q[1], q[2])).collect());
            p.flags.closed = true;
            let mut e = EntityType::Polyline3D(p);
            e.common_mut().layer = CAMADA_BORDA.0.to_string();
            marcar(&mut e, &format!("BORDER:{nome}"));
            v.push(e);
        }
    }
    v
}

pub(crate) fn marcar(e: &mut EntityType, valor: &str) {
    let mut r = ExtendedDataRecord::new(APP);
    r.add_value(XDataValue::String(valor.to_string()));
    e.common_mut().extended_data.add_record(r);
}

/// O TIN como poucas entidades MESH (não um 3DFACE por triângulo).
fn malhas(tin: &Tin, camada: &str, nome: &str) -> Vec<EntityType> {
    let mut saida = Vec::new();
    let mut mapa: Vec<u32> = vec![u32::MAX; tin.vertices.len()];
    let mut usados: Vec<u32> = Vec::new();
    let mut m = Mesh::new();
    let fechar = |m: &mut Mesh, mapa: &mut Vec<u32>, usados: &mut Vec<u32>, saida: &mut Vec<EntityType>| {
        if !m.faces.is_empty() {
            let mut e = EntityType::Mesh(std::mem::replace(m, Mesh::new()));
            e.common_mut().layer = camada.to_string();
            marcar(&mut e, &format!("SURFACE:{nome}"));
            saida.push(e);
        }
        for k in usados.drain(..) {
            mapa[k as usize] = u32::MAX;
        }
    };
    for t in &tin.triangulos {
        if m.vertices.len() + 3 > VERTICES_POR_MALHA {
            fechar(&mut m, &mut mapa, &mut usados, &mut saida);
        }
        let f: Vec<usize> = t
            .iter()
            .map(|&k| {
                if mapa[k as usize] == u32::MAX {
                    let v = tin.vertices[k as usize];
                    m.vertices.push(Vector3::new(v[0], v[1], v[2]));
                    mapa[k as usize] = (m.vertices.len() - 1) as u32;
                    usados.push(k);
                }
                mapa[k as usize] as usize
            })
            .collect();
        m.faces.push(MeshFace::new(f));
    }
    fechar(&mut m, &mut mapa, &mut usados, &mut saida);
    saida
}

fn apagar_vertice_perto(curvas: &mut Vec<Vec<[f64; 3]>>, pontos: &mut Vec<[f64; 3]>, alvo: [f64; 3], raio: f64) {
    let d = |p: &[f64; 3]| ((p[0] - alvo[0]).powi(2) + (p[1] - alvo[1]).powi(2)).sqrt();
    let mut melhor: Option<(f64, usize, usize)> = None;
    for (k, c) in curvas.iter().enumerate() {
        for (j, p) in c.iter().enumerate() {
            if d(p) <= raio && melhor.is_none_or(|m| d(p) < m.0) {
                melhor = Some((d(p), k, j));
            }
        }
    }
    let melhor_ponto = pontos.iter().enumerate().filter(|(_, p)| d(p) <= raio).min_by(|a, b| d(a.1).partial_cmp(&d(b.1)).unwrap());
    match (melhor, melhor_ponto) {
        (Some(m), Some((j, p))) if d(p) < m.0 => {
            pontos.remove(j);
        }
        (_, Some((j, _))) if melhor.is_none() => {
            pontos.remove(j);
        }
        (Some((_, k, j)), _) => {
            let c = &mut curvas[k];
            let fechada = c.len() > 2 && c.first() == c.last();
            c.remove(j);
            if fechada && (j == 0 || j == c.len()) && !c.is_empty() {
                let primeiro = c[0];
                if let Some(u) = c.last_mut() {
                    *u = primeiro;
                }
            }
        }
        _ => {}
    }
}

/// Delete Line: tira os triângulos da aresta mais perto do ponto clicado.
fn apagar_aresta_perto(tin: &Tin, p: [f64; 3]) -> Tin {
    let dist = |a: [f64; 3], b: [f64; 3]| {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l2 = dx * dx + dy * dy;
        let t = if l2 > 0.0 { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
        ((p[0] - a[0] - t * dx).powi(2) + (p[1] - a[1] - t * dy).powi(2)).sqrt()
    };
    let mut melhor: Option<(f64, (u32, u32))> = None;
    for t in &tin.triangulos {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            let d = dist(tin.vertices[a as usize], tin.vertices[b as usize]);
            if melhor.is_none_or(|m| d < m.0) {
                melhor = Some((d, (a.min(b), a.max(b))));
            }
        }
    }
    let Some((_, aresta)) = melhor else { return tin.clone() };
    let tri: Vec<[u32; 3]> = tin
        .triangulos
        .iter()
        .copied()
        .filter(|t| !(0..3).any(|k| (t[k].min(t[(k + 1) % 3]), t[k].max(t[(k + 1) % 3])) == aresta))
        .collect();
    Tin::novo(tin.vertices.clone(), tri)
}

pub(crate) const FASE_CRUZAMENTOS: u8 = 1;
pub(crate) const FASE_CORRECAO: u8 = 2;
pub(crate) const FASE_EMENDA: u8 = 3;
pub(crate) const FASE_TRIANGULOS: u8 = 4;

/// Texto da fase para a barra de progresso.
pub(crate) fn texto_da_fase(f: u8) -> &'static str {
    match f {
        FASE_CRUZAMENTOS => "Checking contour crossings",
        FASE_CORRECAO => "Fixing contour crossings",
        FASE_EMENDA => "Joining contour gaps",
        FASE_TRIANGULOS => "Triangulating",
        _ => "Reading data",
    }
}

/// Montagem de superfície rodando em segundo plano.
pub(crate) struct MontagemPendente {
    pub tab: u64,
    pub geracao: u64,
    pub definicao: SuperficieDwg,
    pub inicio: Instant,
    pub fase: std::sync::Arc<std::sync::atomic::AtomicU8>,
}

/// Resultado da montagem em segundo plano (vai uma vez só, por isso o `Option` dentro).
#[derive(Clone)]
pub(crate) struct MontagemPronta(pub std::sync::Arc<std::sync::Mutex<Option<(MontagemSuperficie, usize)>>>);

impl std::fmt::Debug for MontagemPronta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MontagemPronta")
    }
}

/// O que a montagem precisa, já lido do desenho (sem referência ao documento).
pub(crate) struct EntradaSuperficie {
    nome: String,
    descricao: String,
    comprimento_maximo: Option<f64>,
    /// Um grupo por item de curvas: (curvas, corrigir, folga da emenda).
    curvas: Vec<(Vec<Vec<[f64; 3]>>, bool, f64)>,
    quebras: Vec<Vec<[f64; 3]>>,
    /// Superfícies de LandXML (caminho, nome da superfície no arquivo): a mesma triangulação do arquivo.
    landxml: Vec<(String, String)>,
    /// Arquivos de pontos (caminho, formato): lidos de novo a cada montagem, fora da linha da tela.
    arquivos_pontos: Vec<(String, String)>,
    externos: Vec<Vec<[f64; 3]>>,
    furos: Vec<Vec<[f64; 3]>>,
    edits: Vec<ItemDwg>,
    sumidas: usize,
}

pub(crate) struct MontagemSuperficie {
    pub tin: Tin,
    /// Avisos da montagem (arquivo sumido, linhas que não leram).
    avisos: Vec<String>,
    cruzamentos_achados: Vec<[f64; 3]>,
    ligacoes: Vec<[[f64; 3]; 2]>,
    corrigidos: usize,
    emendas: usize,
    restantes: usize,
}

/// Monta o TIN: diagnóstico, correção e emenda das curvas, edits, triangulação. Não toca no desenho.
pub(crate) fn montar_entrada(e: EntradaSuperficie, fase: &std::sync::atomic::AtomicU8) -> MontagemSuperficie {
    use std::sync::atomic::Ordering::Relaxed;
    let mut todas: Vec<Vec<[f64; 3]>> = Vec::new();
    let mut cruzamentos_achados: Vec<[f64; 3]> = Vec::new();
    let mut ligacoes: Vec<[[f64; 3]; 2]> = Vec::new();
    let (mut corrigidos, mut emendas) = (0usize, 0usize);
    for (mut linhas, corrigir, emendar) in e.curvas {
        fase.store(FASE_CRUZAMENTOS, Relaxed);
        let diag = diagnosticar_curvas(&linhas);
        cruzamentos_achados.extend(diag.cruzamentos.iter().map(|c| c.ponto));
        if corrigir && !diag.cruzamentos.is_empty() {
            fase.store(FASE_CORRECAO, Relaxed);
            linhas = corrigir_cruzamentos(&linhas, &Default::default()).curvas;
            corrigidos += diag.cruzamentos.len();
        }
        if emendar > 0.0 {
            fase.store(FASE_EMENDA, Relaxed);
            let r = emendar_pedacos(&linhas, &OpcoesEmenda { folga_maxima: emendar, limite: e.externos.first().cloned() });
            emendas += r.emendas;
            ligacoes.extend(r.ligacoes);
            linhas = r.curvas;
        }
        todas.extend(linhas);
    }
    let restantes = diagnosticar_curvas(&todas).cruzamentos.len();
    let mut pontos_novos: Vec<[f64; 3]> = Vec::new();
    for it in &e.edits {
        match it.tipo.as_str() {
            "EDIT_ADDPOINT" => pontos_novos.extend(it.pontos.iter().copied()),
            // Delete Point: tira o vértice mais perto do clique (até 2 m), das curvas e dos pontos acrescentados.
            "EDIT_DELETEPOINT" => {
                for p in &it.pontos {
                    apagar_vertice_perto(&mut todas, &mut pontos_novos, *p, 2.0);
                }
            }
            _ => {}
        }
    }
    let mut sup = Superficie::nova(&e.nome, &e.descricao);
    sup.opcoes.comprimento_maximo = e.comprimento_maximo;
    if !todas.is_empty() {
        sup.acrescentar(ItemDefinicao::curvas("contours", todas));
    }
    if !pontos_novos.is_empty() {
        sup.acrescentar(ItemDefinicao::pontos("edits", pontos_novos));
    }
    if !e.quebras.is_empty() {
        sup.acrescentar(ItemDefinicao::linhas_de_quebra("breaklines", e.quebras));
    }
    let mut avisos = Vec::new();
    for (caminho, formato) in &e.arquivos_pontos {
        match ler_pontos_do_arquivo(caminho, formato) {
            Ok((pts, ruins)) => {
                if !ruins.is_empty() {
                    let mostra: Vec<String> = ruins.iter().take(5).map(|r| format!("line {}: {}", r.linha, r.motivo)).collect();
                    avisos.push(format!(
                        "{} lines of {} could not be read ({}{}).",
                        ruins.len(),
                        caminho,
                        mostra.join("; "),
                        if ruins.len() > 5 { "; ..." } else { "" }
                    ));
                }
                if !pts.is_empty() {
                    sup.acrescentar(ItemDefinicao::pontos(caminho, pts));
                }
            }
            Err(m) => avisos.push(m),
        }
    }
    for l in &e.externos {
        sup.acrescentar(ItemDefinicao::limite_externo("boundary", l.clone()));
    }
    for f in &e.furos {
        sup.acrescentar(ItemDefinicao::furo("hide", f.clone()));
    }
    fase.store(FASE_TRIANGULOS, Relaxed);
    // LandXML: se é o único dado da superfície, fica a triangulação do arquivo (R3 da spec 003); com outros dados,
    // os vértices entram como pontos e tudo é triangulado junto.
    let mut do_arquivo: Vec<Tin> = Vec::new();
    for (caminho, nome) in &e.landxml {
        match ler_landxml(caminho, nome) {
            Ok(t) => do_arquivo.push(t),
            Err(m) => avisos.push(m),
        }
    }
    let so_landxml = sup.definicao.is_empty() && do_arquivo.len() == 1 && e.externos.is_empty() && e.furos.is_empty();
    let mut tin = if so_landxml {
        do_arquivo.pop().unwrap()
    } else {
        for t in do_arquivo {
            sup.acrescentar(ItemDefinicao::pontos("landxml", t.vertices));
        }
        sup.montar().tin
    };
    // Edits que mexem no TIN montado, na ordem da definição.
    for it in &e.edits {
        match it.tipo.as_str() {
            "EDIT_DELETELINE" => {
                for p in &it.pontos {
                    tin = apagar_aresta_perto(&tin, *p);
                }
            }
            "EDIT_RAISELOWER" => {
                let mut v = tin.vertices.clone();
                v.iter_mut().for_each(|q| q[2] += it.valor);
                tin = Tin::novo(v, tin.triangulos.clone());
            }
            _ => {}
        }
    }
    MontagemSuperficie { tin, avisos, cruzamentos_achados, ligacoes, corrigidos, emendas, restantes }
}

fn ler_landxml(caminho: &str, nome: &str) -> Result<Tin, String> {
    let xml = std::fs::read_to_string(caminho).map_err(|e| format!("LandXML {caminho} could not be read ({e})."))?;
    let todas = agral_civil_motor::landxml::ler_superficies(&xml).map_err(|e| format!("LandXML {caminho}: {e}"))?;
    todas
        .into_iter()
        .find(|s| nome.is_empty() || s.nome == nome)
        .map(|s| s.tin)
        .ok_or_else(|| format!("LandXML {caminho} has no surface '{nome}'."))
}

/// Formato do arquivo de pontos pelo nome (PNEZD, PENZD, NEZ, ENZ; padrão PNEZD).
pub(crate) fn formato_de_pontos(nome: &str) -> Option<agral_civil_motor::pontos::FormatoPontos> {
    use agral_civil_motor::pontos::FormatoPontos::*;
    Some(match nome.to_ascii_uppercase().as_str() {
        "" | "PNEZD" => Pnezd,
        "PENZD" => Penzd,
        "NEZ" => Nez,
        "ENZ" => Enz,
        _ => return None,
    })
}

fn ler_pontos_do_arquivo(
    caminho: &str,
    formato: &str,
) -> Result<(Vec<[f64; 3]>, Vec<agral_civil_motor::pontos::LinhaRuim>), String> {
    let f = formato_de_pontos(formato).ok_or_else(|| format!("Unknown point file format '{formato}'."))?;
    let bytes = std::fs::read(caminho).map_err(|e| format!("Point file {caminho} could not be read ({e}); its points were left out."))?;
    let texto = String::from_utf8(bytes).unwrap_or_else(|e| e.into_bytes().iter().map(|&b| b as char).collect());
    let r = agral_civil_motor::pontos::ler_arquivo_de_pontos(&texto, f);
    Ok((r.pontos.into_iter().map(|p| p.xyz).collect(), r.linhas_ruins))
}
