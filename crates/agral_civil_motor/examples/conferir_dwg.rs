//! Confere um DWG salvo pelo Agral CAD: entidades por tipo e camada, e os XRECORD do Agral Civil.
//! Uso: cargo run --release -p agral_civil_motor --example conferir_dwg -- <arquivo.dwg>
use acadrust::entities::EntityType;
use std::collections::BTreeMap;

fn main() {
    let caminho = std::env::args().nth(1).expect("arquivo");
    let doc = acadrust::io::dwg::DwgReader::from_file(&caminho).expect("abrir").read().expect("ler");
    let mut n: BTreeMap<(String, String), (usize, usize, usize)> = BTreeMap::new();
    for e in doc.model_space_entities() {
        let tipo = format!("{:?}", e).split('(').next().unwrap_or("?").to_string();
        let c = n.entry((e.common().layer.clone(), tipo)).or_default();
        c.0 += 1;
        if let EntityType::Mesh(m) = e {
            c.1 += m.vertices.len();
            c.2 += m.faces.len();
        }
    }
    for ((camada, tipo), (q, v, f)) in &n {
        println!("{camada:24} {tipo:12} {q:6} {}", if *f > 0 { format!("({v} vértices, {f} faces)") } else { String::new() });
    }
    let xr = doc.objects.values().filter_map(|o| match o {
        acadrust::objects::ObjectType::XRecord(x) => Some(x),
        _ => None,
    });
    for x in xr {
        if x.entries.first().is_some_and(|e| matches!(&e.value, acadrust::objects::XRecordValue::String(s) if s == "AGRAL_SURFACE")) {
            println!("XRECORD de superfície: {} entradas", x.entries.len());
        }
    }
}
