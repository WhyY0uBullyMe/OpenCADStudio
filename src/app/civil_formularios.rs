//! Janelas de configuração dos comandos civis (JP, 18/11: "quero botões e janelas que abrem para fazer as
//! configurações"). Cada janela é um formulário com os campos na ordem dos argumentos do comando, já preenchidos
//! com o que o eixo ou a superfície têm; OK despacha o comando. A forma "-COMANDO" continua perguntando na linha
//! de comando (roteiros e MCP).

use super::OpenCADStudio;
use crate::ui::window::civil_dialog::{Campo, Formulario};

/// Comandos que abrem janela quando chamados sozinhos (ou só com o nome do eixo ou da superfície).
pub(crate) const FORMULARIOS: &[&str] = &[
    "COMPUTEMATERIALS",
    "CREATESECTIONS",
    "CREATESAMPLELINES",
    "CREATESECTIONVIEWS",
    "CREATEPROFILEVIEW",
    "CREATEASSEMBLY",
    "CREATECORRIDOR",
    "ALIGNMENTSTATIONS",
    "EXTRACTSURFACEOBJECTS",
];

fn n(v: f64) -> String {
    let t = format!("{v:.4}");
    t.trim_end_matches('0').trim_end_matches('.').to_string()
}

impl OpenCADStudio {
    /// O formulário do comando, preenchido; `None` se faltar o básico (sem eixo ou sem superfície no desenho), e
    /// aí o comando segue pela linha de comando, que explica o que falta.
    pub(crate) fn civil_formulario(&mut self, i: usize, verbo: &str, arg: Option<&str>) -> Option<Formulario> {
        let superficies = self.civil_superficies(i);
        if verbo == "EXTRACTSURFACEOBJECTS" {
            let s = arg.map(str::to_string).or_else(|| self.civil_superficie_selecionada(i)).or_else(|| superficies.first().cloned())?;
            return Some(Formulario {
                titulo: "Extract Objects from Surface".into(),
                verbo: "EXTRACTSURFACEOBJECTS".into(),
                campos: vec![
                    Campo::lista("Surface", &s, superficies),
                    Campo::lista("Objects", "All", ["All", "Major", "Minor", "Border", "Triangles"].map(String::from).to_vec()),
                ],
                ponto: None,
                nota: "All = major and minor contours and the border. The copies are plain polylines; the surface does not change.".into(),
            });
        }
        let eixos: Vec<String> = self.civil_alinhamentos(i).into_iter().map(|a| a.nome).collect();
        let eixo = arg.map(str::to_string).or_else(|| self.civil_al_escolher(i, None).ok()).or_else(|| eixos.first().cloned())?;
        let a = self.civil_al_ler(i, &eixo).ok()?;
        let campo_eixo = Campo::lista("Alignment", &eixo, eixos);
        let sup_eixo = if a.superficie.is_empty() { superficies.first().cloned().unwrap_or_default() } else { a.superficie.clone() };
        let f = match verbo {
            "COMPUTEMATERIALS" => {
                let corr = crate::app::civil_alinhamento::superficie_do_corredor(&a.nome);
                let (base, comp, fc, fa) = match &a.materiais {
                    Some(m) => (m.base.clone(), m.comparar.clone(), m.fator_corte, m.fator_aterro),
                    None => (
                        sup_eixo.clone(),
                        superficies.iter().find(|s| s.eq_ignore_ascii_case(&corr)).cloned().unwrap_or_default(),
                        1.0,
                        1.0,
                    ),
                };
                Formulario {
                    titulo: "Compute Materials".into(),
                    verbo: "COMPUTEMATERIALS".into(),
                    campos: vec![
                        campo_eixo,
                        Campo::lista("Base surface (existing ground)", base, superficies.clone()),
                        Campo::lista("Comparison surface (design, corridor)", comp, superficies),
                        Campo::texto("Cut factor", n(fc)),
                        Campo::texto("Fill factor", n(fa)),
                    ],
                    ponto: None,
                    nota: "Cut where the comparison is below the base, fill where it is above, on the sample lines \
                           (CREATESECTIONS). Average end areas; the factors multiply the volumes."
                        .into(),
                }
            }
            "CREATESECTIONS" | "CREATESAMPLELINES" | "CREATESECTIONVIEWS" => {
                let s = a.secoes.clone();
                Formulario {
                    titulo: "Create Sample Lines and Section Views".into(),
                    verbo: "CREATESECTIONS".into(),
                    campos: vec![
                        campo_eixo,
                        Campo::texto("Interval (m)", n(s.as_ref().map_or(20.0, |s| s.intervalo))),
                        Campo::texto("Left swath width (m)", n(s.as_ref().map_or(30.0, |s| s.esquerda))),
                        Campo::texto("Right swath width (m)", n(s.as_ref().map_or(30.0, |s| s.direita))),
                        Campo::texto("Section views per row", s.as_ref().map_or(5, |s| s.colunas)),
                    ],
                    ponto: Some("Specify the upper left corner of the section views:".into()),
                    nota: "Sample lines at every interval and at the alignment's geometry points.".into(),
                }
            }
            "CREATEPROFILEVIEW" => Formulario {
                titulo: "Create Profile View".into(),
                verbo: "CREATEPROFILEVIEW".into(),
                campos: vec![
                    campo_eixo,
                    Campo::lista("Surface (existing ground)", sup_eixo, superficies),
                    Campo::texto("Vertical exaggeration", n(a.vista.as_ref().map_or(10.0, |v| v.exagero))),
                ],
                ponto: Some("Specify the lower left corner of the profile view:".into()),
                nota: String::new(),
            },
            "CREATEASSEMBLY" => {
                let m = a.assembly.clone().unwrap_or_default();
                Formulario {
                    titulo: "Dam Assembly".into(),
                    verbo: "CREATEASSEMBLY".into(),
                    campos: vec![
                        campo_eixo,
                        Campo::texto("Crest width (m)", n(m.crista)),
                        Campo::texto("Crest cross slope to upstream (%)", n(m.decl_crista * 100.0)),
                        Campo::lista("Upstream side", if m.montante_esquerda { "L" } else { "R" }, vec!["L".into(), "R".into()]),
                        Campo::texto("Upstream slope (H for 1 V)", n(m.talude_montante)),
                        Campo::texto("Downstream slope (H for 1 V)", n(m.talude_jusante)),
                        Campo::texto("Berm every (m of height, 0 = none)", n(m.altura_berma)),
                        Campo::texto("Berm width (m)", n(m.largura_berma)),
                        Campo::texto("Cut slope (H for 1 V)", n(m.talude_corte)),
                    ],
                    ponto: None,
                    nota: "L or R: the side of the alignment, looking forward along it, that faces the reservoir.".into(),
                }
            }
            "CREATECORRIDOR" => Formulario {
                titulo: "Create Corridor".into(),
                verbo: "CREATECORRIDOR".into(),
                campos: vec![campo_eixo, Campo::texto("Section frequency (m)", n(a.corredor.unwrap_or(5.0)))],
                ponto: None,
                nota: "Needs the design profile (CREATEPROFILE) and the dam assembly. Builds the <ALIGNMENT>-CORR surface.".into(),
            },
            "ALIGNMENTSTATIONS" => Formulario {
                titulo: "Alignment Stations".into(),
                verbo: "ALIGNMENTSTATIONS".into(),
                campos: vec![
                    campo_eixo,
                    Campo::texto("Start station (m)", n(a.estaca_inicial)),
                    Campo::texto("Station label interval (m)", n(a.intervalo)),
                ],
                ponto: None,
                nota: String::new(),
            },
            _ => return None,
        };
        Some(f)
    }
}
