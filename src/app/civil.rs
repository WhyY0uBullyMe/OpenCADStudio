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

impl OpenCADStudio {
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

    /// `REBUILDSURFACE`: monta o TIN a partir da lista de definição e troca o desenho antigo pelo novo.
    pub(crate) fn civil_refazer(&mut self, i: usize, nome: &str) -> Result<String, String> {
        let t0 = Instant::now();
        let mut s = self.civil_ler(i, nome)?;
        let doc = &self.tabs[i].scene.document;
        let mut todas: Vec<Vec<[f64; 3]>> = Vec::new();
        let mut cruzamentos_achados: Vec<[f64; 3]> = Vec::new();
        let mut ligacoes: Vec<[[f64; 3]; 2]> = Vec::new();
        let (mut corrigidos, mut emendas, mut sumidas) = (0usize, 0usize, 0usize);
        // Limites (polilinhas fechadas): o externo também serve de borda para a emenda não juntar pontas da borda.
        let mut externos: Vec<Vec<[f64; 3]>> = Vec::new();
        let mut furos: Vec<Vec<[f64; 3]>> = Vec::new();
        for it in s.itens.iter().filter(|it| it.ligado && it.tipo.starts_with("BOUNDARY")) {
            for h in &it.fontes {
                match doc.get_entity(*h).and_then(curvas::pontos) {
                    Some(p) if it.tipo == "BOUNDARY_HIDE" => furos.push(p),
                    Some(p) => externos.push(p),
                    None => sumidas += 1,
                }
            }
        }
        for it in s.itens.iter().filter(|it| it.ligado && it.tipo == "CONTOURS") {
            let mut linhas: Vec<Vec<[f64; 3]>> = Vec::new();
            for h in &it.fontes {
                match doc.get_entity(*h).and_then(curvas::pontos) {
                    Some(p) => linhas.push(p),
                    None => sumidas += 1,
                }
            }
            let diag = diagnosticar_curvas(&linhas);
            cruzamentos_achados.extend(diag.cruzamentos.iter().map(|c| c.ponto));
            if it.corrigir && !diag.cruzamentos.is_empty() {
                linhas = corrigir_cruzamentos(&linhas, &Default::default()).curvas;
                corrigidos += diag.cruzamentos.len();
            }
            if it.emendar > 0.0 {
                let e = emendar_pedacos(&linhas, &OpcoesEmenda { folga_maxima: it.emendar, limite: externos.first().cloned() });
                emendas += e.emendas;
                ligacoes.extend(e.ligacoes);
                linhas = e.curvas;
            }
            todas.extend(linhas);
        }
        let restantes = diagnosticar_curvas(&todas).cruzamentos.len();
        let edits: Vec<&ItemDwg> = s.itens.iter().filter(|it| it.ligado && it.tipo.starts_with("EDIT_")).collect();
        let mut pontos_novos: Vec<[f64; 3]> = Vec::new();
        for it in &edits {
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
        let t_ler = t0.elapsed();
        let mut sup = Superficie::nova(&s.nome, &s.descricao);
        sup.opcoes.comprimento_maximo = s.comprimento_maximo;
        if !todas.is_empty() {
            sup.acrescentar(ItemDefinicao::curvas("contours", todas));
        }
        if !pontos_novos.is_empty() {
            sup.acrescentar(ItemDefinicao::pontos("edits", pontos_novos));
        }
        for l in &externos {
            sup.acrescentar(ItemDefinicao::limite_externo("boundary", l.clone()));
        }
        for f in &furos {
            sup.acrescentar(ItemDefinicao::furo("hide", f.clone()));
        }
        let mut montagem = sup.montar();
        // Edits que mexem no TIN montado, na ordem da definição.
        for it in &edits {
            match it.tipo.as_str() {
                "EDIT_DELETELINE" => {
                    for p in &it.pontos {
                        montagem.tin = apagar_aresta_perto(&montagem.tin, *p);
                    }
                }
                "EDIT_RAISELOWER" => {
                    let mut v = montagem.tin.vertices.clone();
                    v.iter_mut().for_each(|q| q[2] += it.valor);
                    montagem.tin = Tin::novo(v, montagem.tin.triangulos.clone());
                }
                _ => {}
            }
        }
        let t_montar = t0.elapsed();

        let estilo = crate::civil::estilos::estilo(&s.estilo);
        // A superfície é um objeto só, como no Civil 3D (pedido do JP, 18/11): o desenho do estilo vai num bloco
        // e entra no desenho como uma inserção. Dentro do bloco cada componente fica na sua camada do template.
        let conteudo = desenho_do_estilo(&montagem.tin, estilo, &s.nome);
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
        let t_desenhar = t0.elapsed();

        let mut msg = format!(
            "Surface '{}': {} points, {} triangles in {:.1} s (read {:.1}, build {:.1}, draw {:.1}).",
            s.nome,
            montagem.tin.vertices.len(),
            montagem.tin.triangulos.len(),
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

fn marcar(e: &mut EntityType, valor: &str) {
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
