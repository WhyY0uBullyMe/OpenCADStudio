//! `OBJECTVIEWER`: abre os objetos selecionados num desenho à parte, em 3D, para girar e olhar (pedido do JP,
//! 18/11: "aquela janelinha para olhar o 3D"). A superfície vira malha colorida por faixa de cota; eixos,
//! corredores e o resto entram como estão. Exagero vertical opcional. O desenho do viewer não é salvo: fechar
//! não pergunta nada, e o desenho original não muda.

use super::{Message, OpenCADStudio};
use document::{EntityType, Handle};
use iced::Task;

/// Cores ACI das faixas de cota, do mais baixo ao mais alto (azul, verde, amarelo, laranja, vermelho, branco).
const RAMPA: [u8; 10] = [150, 140, 130, 110, 90, 70, 50, 40, 30, 254];

fn exagerar(e: &mut EntityType, z0: f64, f: f64) {
    let z = |v: f64| z0 + (v - z0) * f;
    match e {
        EntityType::Mesh(m) => m.vertices.iter_mut().for_each(|v| v.z = z(v.z)),
        EntityType::Polyline3D(p) => p.vertices.iter_mut().for_each(|v| v.position.z = z(v.position.z)),
        EntityType::LwPolyline(p) => p.elevation = z(p.elevation),
        EntityType::Line(l) => {
            l.start.z = z(l.start.z);
            l.end.z = z(l.end.z);
        }
        EntityType::Point(p) => p.location.z = z(p.location.z),
        _ => {}
    }
}

impl OpenCADStudio {
    pub(crate) fn civil_object_viewer(&mut self, i: usize, handles: &[Handle], exagero: f64) -> Result<Task<Message>, String> {
        if handles.is_empty() {
            return Err("Select the objects to view.".into());
        }
        let mut copias: Vec<EntityType> = Vec::new();
        let mut fora = 0usize;
        for h in handles {
            let Some(e) = self.tabs[i].scene.document.get_entity(*h).cloned() else { continue };
            if let EntityType::Insert(ins) = &e {
                if let Some(nome) = crate::app::civil::superficie_da_entidade(&e) {
                    // Superfície: malhas por faixa de cota, cada uma com a sua cor.
                    let tin = self.civil_tin(i, &nome)?;
                    let Some((z0, z1)) = tin.faixa_de_cotas() else { continue };
                    let n = RAMPA.len();
                    let passo = ((z1 - z0) / n as f64).max(1e-9);
                    let mut faixas: Vec<Vec<[u32; 3]>> = vec![Vec::new(); n];
                    for t in &tin.triangulos {
                        let zc = t.iter().map(|&k| tin.vertices[k as usize][2]).sum::<f64>() / 3.0;
                        faixas[(((zc - z0) / passo) as usize).min(n - 1)].push(*t);
                    }
                    for (k, tri) in faixas.into_iter().enumerate().filter(|(_, t)| !t.is_empty()) {
                        let parte = agral_civil_motor::tin::Tin::novo(tin.vertices.clone(), tri);
                        for mut m in crate::app::civil::malhas(&parte, "SUPERFICIE", &nome) {
                            m.common_mut().color = document::Color::Index(RAMPA[k]);
                            copias.push(m);
                        }
                    }
                    continue;
                }
                // Desenhos do Agral (eixo, perfil, seções): o bloco está na origem, sem giro nem escala.
                let doc = &self.tabs[i].scene.document;
                match doc.block_records.get(&ins.block_name) {
                    Some(rec) if ins.block_name.starts_with("AGRAL_") => {
                        copias.extend(rec.entity_handles.iter().filter_map(|b| doc.get_entity(*b).cloned()));
                    }
                    _ => fora += 1,
                }
                continue;
            }
            copias.push(e);
        }
        if copias.is_empty() {
            return Err("Nothing in the selection can be shown in 3D.".into());
        }
        // Exagero vertical a partir da cota mais baixa.
        if exagero != 1.0 {
            let z0 = copias
                .iter()
                .filter_map(|e| match e {
                    EntityType::Mesh(m) => m.vertices.iter().map(|v| v.z).reduce(f64::min),
                    EntityType::Polyline3D(p) => p.vertices.iter().map(|v| v.position.z).reduce(f64::min),
                    _ => None,
                })
                .reduce(f64::min)
                .unwrap_or(0.0);
            copias.iter_mut().for_each(|e| exagerar(e, z0, exagero));
        }
        let camadas: std::collections::BTreeSet<String> = copias.iter().map(|e| e.common().layer.clone()).collect();
        for c in copias.iter_mut() {
            let cm = c.common_mut();
            cm.handle = Handle::NULL;
            cm.owner_handle = Handle::NULL;
            cm.extended_data = Default::default();
        }
        let n = copias.len();
        // Aba nova só para olhar (nunca troca o desenho atual).
        self.tab_counter += 1;
        self.tabs.push(crate::app::tab_document::DocumentTab::new_drawing(self.tab_counter));
        let j = self.tabs.len() - 1;
        self.active_tab = j;
        self.apply_display_defaults(j);
        for c in &camadas {
            self.tabs[j].scene.ensure_layer(c);
        }
        self.tabs[j].scene.add_entities(copias);
        self.tabs[j].tab_title = if exagero != 1.0 { format!("Object Viewer (Z x{exagero})") } else { "Object Viewer".into() };
        self.tabs[j].dirty = false;
        self.tabs[j].civil_visualizador = true;
        self.command_line.push_info(&format!(
            "Object Viewer: {n} objects{}. Orbit with Shift + middle mouse button; close the tab to go back.",
            if fora > 0 { format!(" ({fora} block references left out)") } else { String::new() }
        ));
        let mode = crate::modules::view::visual_style::mode_for_keyword("S");
        let mut t = vec![self.apply_view_preset("SEISO"), self.dispatch_command("ZOOM E")];
        if let Some(m) = mode {
            t.push(Task::done(Message::SetRenderMode(m)));
        }
        Ok(Task::batch(t))
    }
}
