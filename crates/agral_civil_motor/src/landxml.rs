//! R16: superfície TIN em LandXML 1.x (importar a do Civil 3D e exportar para ele abrir).
//!
//! No LandXML o ponto é "Norte Este Cota" (Y X Z). Face com `i="1"` é invisível e não entra.

use crate::tin::Tin;
use crate::P3;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Clone, Debug)]
pub struct SuperficieLandXml {
    pub nome: String,
    pub descricao: String,
    pub tin: Tin,
}

pub fn ler_superficies(xml: &str) -> Result<Vec<SuperficieLandXml>, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| format!("LandXML inválido: {e}"))?;
    let mut saida = Vec::new();
    for s in doc.descendants().filter(|n| n.has_tag_name("Surface")) {
        let nome = s.attribute("name").unwrap_or("Superfície").to_string();
        let Some(def) = s.children().find(|n| n.has_tag_name("Definition")) else { continue };
        if def.attribute("surfType").is_some_and(|t| !t.eq_ignore_ascii_case("TIN")) {
            return Err(format!("superfície '{nome}': só TIN é aceita, veio {}", def.attribute("surfType").unwrap()));
        }
        let mut vertices: Vec<P3> = Vec::new();
        let mut por_id: HashMap<&str, u32> = HashMap::new();
        for p in def.descendants().filter(|n| n.has_tag_name("P")) {
            let id = p.attribute("id").ok_or_else(|| format!("superfície '{nome}': ponto sem id"))?;
            let v = numeros(p.text().unwrap_or(""));
            if v.len() < 3 {
                return Err(format!("superfície '{nome}': ponto {id} sem Norte, Este e cota"));
            }
            por_id.insert(id, vertices.len() as u32);
            vertices.push([v[1], v[0], v[2]]);
        }
        let mut triangulos = Vec::new();
        for f in def.descendants().filter(|n| n.has_tag_name("F")) {
            if f.attribute("i") == Some("1") {
                continue;
            }
            let ids: Vec<&str> = f.text().unwrap_or("").split_whitespace().collect();
            if ids.len() != 3 {
                return Err(format!("superfície '{nome}': face com {} pontos", ids.len()));
            }
            let mut t = [0u32; 3];
            for (k, id) in ids.iter().enumerate() {
                t[k] = *por_id.get(id).ok_or_else(|| format!("superfície '{nome}': face usa o ponto {id}, que não existe"))?;
            }
            triangulos.push(t);
        }
        let descricao = s.attribute("desc").unwrap_or("").to_string();
        saida.push(SuperficieLandXml { nome, descricao, tin: Tin::novo(vertices, triangulos) });
    }
    Ok(saida)
}

fn numeros(s: &str) -> Vec<f64> {
    s.split_whitespace().filter_map(|t| t.parse().ok()).collect()
}

pub fn escrever_superficie(nome: &str, descricao: &str, tin: &Tin) -> String {
    let mut x = String::with_capacity(64 * tin.vertices.len() + 32 * tin.triangulos.len() + 512);
    x.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    x.push_str("<LandXML xmlns=\"http://www.landxml.org/schema/LandXML-1.2\" version=\"1.2\">\n");
    x.push_str("  <Units><Metric linearUnit=\"meter\" areaUnit=\"squareMeter\" volumeUnit=\"cubicMeter\"/></Units>\n");
    x.push_str("  <Application name=\"Agral CAD\" manufacturer=\"Agral Engenharia\"/>\n");
    let _ = writeln!(x, "  <Surfaces>\n    <Surface name=\"{}\" desc=\"{}\">", esc(nome), esc(descricao));
    x.push_str("      <Definition surfType=\"TIN\">\n        <Pnts>\n");
    for (i, v) in tin.vertices.iter().enumerate() {
        // `{}` do Rust escreve o menor texto que lê de volta o mesmo f64: ida e volta sem perda.
        let _ = writeln!(x, "          <P id=\"{}\">{} {} {}</P>", i + 1, v[1], v[0], v[2]);
    }
    x.push_str("        </Pnts>\n        <Faces>\n");
    for t in &tin.triangulos {
        let _ = writeln!(x, "          <F>{} {} {}</F>", t[0] + 1, t[1] + 1, t[2] + 1);
    }
    x.push_str("        </Faces>\n      </Definition>\n    </Surface>\n  </Surfaces>\n</LandXML>\n");
    x
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
