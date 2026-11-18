//! Lista o que um DWG/DWT tem: camadas, tipos de linha, estilos de texto e cota, blocos, layouts e unidades.
use acadrust::entities::EntityType;
use std::collections::BTreeMap;
fn main() {
    let c = std::env::args().nth(1).expect("arquivo");
    let doc = acadrust::io::dwg::DwgReader::from_file(&c).expect("abrir").read().expect("ler");
    let h = &doc.header;
    println!("versão {:?}; INSUNITS {:?}; LUNITS {:?} LUPREC {:?}; LTSCALE {:?}", doc.version, h.insertion_units, h.linear_unit_format, h.linear_unit_precision, h.linetype_scale);
    println!("resumo: {:?}", doc.summary_info);
    println!("camadas ({}):", doc.layers.iter().count());
    for l in doc.layers.iter() {
        println!("  {} | cor {:?} | linha {} | espessura {:?} | plot {:?} | off/frz {:?}/{:?}", l.name, l.color, l.line_type, l.line_weight, l.is_plottable, l.is_off(), l.is_frozen());
    }
    println!("tipos de linha: {}", doc.line_types.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(", "));
    println!("estilos de texto: {}", doc.text_styles.iter().map(|t| format!("{} ({}, h={})", t.name, t.font_file, t.height)).collect::<Vec<_>>().join("; "));
    println!("estilos de cota: {}", doc.dim_styles.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(", "));
    let blocos: Vec<String> = doc.block_records.iter().map(|b| b.name.clone()).filter(|n| !n.starts_with('*')).collect();
    println!("blocos ({}): {}", blocos.len(), blocos.join(", "));
    let mut por: BTreeMap<String, usize> = BTreeMap::new();
    for e in doc.entities() {
        *por.entry(format!("{}", format!("{:?}", e).split('(').next().unwrap_or("?"))).or_default() += 1;
    }
    println!("entidades (todas as áreas): {por:?}");
    for e in doc.entities() {
        match e {
            EntityType::AttributeDefinition(a) => println!("atributo: tag={} padrão={:?} prompt={:?}", a.tag, a.default_value, a.prompt),
            EntityType::Text(t) => println!("texto: {:?}", t.value),
            EntityType::MText(t) => println!("mtext: {:?}", t.value),
            _ => {}
        }
    }
}
