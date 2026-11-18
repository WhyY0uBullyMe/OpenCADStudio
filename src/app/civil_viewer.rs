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
        // Cena do viewer: triângulos das malhas e linhas 3D, com a cor de cada objeto (ou da camada).
        let doc = &self.tabs[i].scene.document;
        let rgb = |e: &EntityType| -> [u8; 3] {
            let indice = match &e.common().color {
                document::Color::Index(n) if (1..=255).contains(n) => Some(*n),
                _ => doc.layers.get(&e.common().layer).and_then(|l| match &l.color {
                    document::Color::Index(n) if (1..=255).contains(n) => Some(*n),
                    _ => None,
                }),
            };
            indice.and_then(crate::ui::color_select::math::aci_rgb).map_or([220, 225, 230], |(r, g, b)| [r, g, b])
        };
        let mut cena = crate::ui::window::object_viewer::Cena::default();
        let f3 = |x: f64, y: f64, z: f64| [x as f32, y as f32, z as f32];
        for e in &copias {
            let c = rgb(e);
            match e {
                EntityType::Mesh(m) => {
                    for face in &m.faces {
                        let v: Vec<[f32; 3]> = face
                            .vertices
                            .iter()
                            .filter_map(|&k| m.vertices.get(k))
                            .map(|p| f3(p.x, p.y, p.z))
                            .collect();
                        for k in 1..v.len().saturating_sub(1) {
                            cena.triangulos.push(([v[0], v[k], v[k + 1]], c));
                        }
                    }
                }
                EntityType::Polyline3D(p) => {
                    let mut v: Vec<[f32; 3]> = p.vertices.iter().map(|q| f3(q.position.x, q.position.y, q.position.z)).collect();
                    if p.flags.closed && v.len() > 2 {
                        v.push(v[0]);
                    }
                    cena.linhas.push((v, c));
                }
                EntityType::Line(l) => cena.linhas.push((vec![f3(l.start.x, l.start.y, l.start.z), f3(l.end.x, l.end.y, l.end.z)], c)),
                other => {
                    if let Some(p) = crate::civil::curvas::pontos(other) {
                        cena.linhas.push((p.iter().map(|q| f3(q[0], q[1], q[2])).collect(), c));
                    }
                }
            }
        }
        if cena.triangulos.is_empty() && cena.linhas.is_empty() {
            return Err("Nothing in the selection can be shown in 3D.".into());
        }
        cena.titulo = if exagero != 1.0 { format!("{} objects, Z x{exagero}", handles.len()) } else { format!("{} objects", handles.len()) };
        cena.enquadrar();
        let (nt, nl) = (cena.triangulos.len(), cena.linhas.len());
        self.civil_cena = Some(std::sync::Arc::new(cena));
        // Janela própria (como no Civil 3D); se já estiver aberta, só troca o conteúdo.
        let id = crate::ui::dock::PanelId::ObjectViewer;
        if !self.show_object_viewer {
            self.pallet_as_window(id);
            self.show_object_viewer = true;
        }
        self.command_line.push_info(&format!(
            "Object Viewer: {nt} triangles, {nl} lines{}. Drag to orbit, wheel to zoom, double click to reset.",
            if fora > 0 { format!(" ({fora} block references left out)") } else { String::new() }
        ));
        Ok(self.raise_pallet_window(id).unwrap_or_else(Task::none))
    }
}
