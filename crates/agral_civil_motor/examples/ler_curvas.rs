//! Lê as curvas de nível de um DWG ou DXF e monta a superfície com o motor (conferência com arquivo real).
//! Uso: cargo run --release -p agral_civil_motor --example ler_curvas -- <arquivo> [camadas] [saida.xml]
//! Variáveis: LIMITE=<camada do limite externo>, MAXLADO=<comprimento máximo de borda em m>.
//! Uso: ler-curvas <arquivo.dwg> [camada1,camada2,...]
use acadrust::entities::EntityType;
use agral_civil_motor::superficie::{ItemDefinicao, Superficie};
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let caminho = &args[1];
    let camadas: Option<Vec<String>> = args.get(2).map(|s| s.split(',').map(|c| c.to_uppercase()).collect());
    let t = Instant::now();
    let doc = if caminho.to_lowercase().ends_with(".dxf") {
        acadrust::io::dxf::DxfReader::from_file(caminho).expect("abrir DXF").read().expect("ler DXF")
    } else {
        acadrust::io::dwg::DwgReader::from_file(caminho).expect("abrir DWG").read().expect("ler DWG")
    };
    println!("leitura: {:?}", t.elapsed());

    let mut por_tipo: BTreeMap<String, usize> = BTreeMap::new();
    let mut por_camada: BTreeMap<String, (usize, usize, f64, f64)> = BTreeMap::new();
    let mut normais_fora = 0;
    let mut linhas: Vec<Vec<[f64; 3]>> = Vec::new();
    for e in doc.model_space_entities() {
        let nome = format!("{:?}", e).split('(').next().unwrap_or("?").to_string();
        *por_tipo.entry(nome).or_default() += 1;
        let (camada, pts): (String, Vec<[f64; 3]>) = match e {
            EntityType::LwPolyline(p) => {
                if (p.normal.z - 1.0).abs() > 1e-9 {
                    normais_fora += 1;
                }
                let mut v: Vec<[f64; 3]> = p.vertices.iter().map(|q| [q.location.x, q.location.y, p.elevation]).collect();
                if p.is_closed && !v.is_empty() {
                    v.push(v[0]);
                }
                (p.common.layer.clone(), v)
            }
            EntityType::Polyline2D(p) => {
                if (p.normal.z - 1.0).abs() > 1e-9 {
                    normais_fora += 1;
                }
                (p.common.layer.clone(), p.vertices.iter().map(|q| [q.location.x, q.location.y, p.elevation]).collect())
            }
            EntityType::Polyline3D(p) => {
                (p.common.layer.clone(), p.vertices.iter().map(|q| [q.position.x, q.position.y, q.position.z]).collect())
            }
            EntityType::Polyline(p) => {
                (p.common.layer.clone(), p.vertices.iter().map(|q| [q.location.x, q.location.y, q.location.z]).collect())
            }
            _ => continue,
        };
        let c = por_camada.entry(camada.clone()).or_insert((0, 0, f64::MAX, f64::MIN));
        c.0 += 1;
        c.1 += pts.len();
        for p in &pts {
            c.2 = c.2.min(p[2]);
            c.3 = c.3.max(p[2]);
        }
        if camadas.as_ref().map_or(true, |cs| cs.contains(&camada.to_uppercase())) && pts.len() >= 2 {
            linhas.push(pts);
        }
    }
    println!("entidades por tipo: {por_tipo:?}");
    println!("normais diferentes de +Z: {normais_fora}");
    for (k, (n, v, z0, z1)) in &por_camada {
        println!("camada {k}: {n} polilinhas, {v} vértices, cota {z0:.3} a {z1:.3}");
    }
    let vertices: usize = linhas.iter().map(|l| l.len()).sum();
    println!("usadas: {} polilinhas, {} vértices", linhas.len(), vertices);

    let mut s = Superficie::nova("TN", caminho);
    s.acrescentar(ItemDefinicao::curvas(caminho, linhas.clone()));
    if let Ok(lim) = std::env::var("LIMITE") {
        for l in limites_da_camada(&doc, &lim) {
            s.acrescentar(ItemDefinicao::limite_externo(&lim, l));
        }
    }
    if let Ok(m) = std::env::var("MAXLADO") {
        s.opcoes.comprimento_maximo = m.parse().ok();
    }
    let t = Instant::now();
    let avisos = s.refazer();
    println!(
        "superfície: {} vértices, {} triângulos em {:?}; avisos: {:?}",
        s.tin.vertices.len(),
        s.tin.triangulos.len(),
        t.elapsed(),
        avisos
    );
    // Conferência: nenhuma aresta de triângulo cruza uma curva (critério 2), por amostragem completa com grade.
    let cruz = agral_civil_motor::tin::contar_cruzamentos(&s.tin, &linhas);
    println!("arestas de triângulo que cruzam curva: {cruz}");
    // Critério 3: curvas geradas na cota de cada curva de entrada passam por ela.
    let mut por_cota: BTreeMap<i64, Vec<&Vec<[f64; 3]>>> = BTreeMap::new();
    for l in &linhas {
        if l.iter().all(|p| (p[2] - l[0][2]).abs() < 1e-6) {
            por_cota.entry((l[0][2] * 1000.0).round() as i64).or_default().push(l);
        }
    }
    let (mut pior, mut soma, mut n, mut fora) = (0.0f64, 0.0, 0usize, 0usize);
    let mut pior_onde = [0.0; 3];
    for (zk, entrada) in &por_cota {
        let z = *zk as f64 / 1000.0;
        for g in s.tin.curvas_de_nivel(z) {
            for p in &g {
                let d = entrada
                    .iter()
                    .flat_map(|l| l.windows(2))
                    .map(|w| dist(p, &w[0], &w[1]))
                    .fold(f64::INFINITY, f64::min);
                if d > pior {
                    pior = d;
                    pior_onde = *p;
                }
                soma += d;
                if d > 0.125 {
                    fora += 1;
                }
                n += 1;
            }
        }
    }
    println!(
        "curvas geradas x entrada: {} cotas, {} pontos, desvio médio {:.4} m, {} pontos acima de 0,125 m, pior {:.4} m em {:.2} {:.2} {:.2}",
        por_cota.len(), n, soma / n.max(1) as f64, fora, pior, pior_onde[0], pior_onde[1], pior_onde[2]
    );
    if let Some(saida) = args.get(3) {
        std::fs::write(saida, agral_civil_motor::landxml::escrever_superficie("TN", caminho, &s.tin)).unwrap();
        println!("LandXML gravado em {saida}");
    }
}

fn dist(p: &[f64; 3], a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((p[0] - a[0] - t * dx).powi(2) + (p[1] - a[1] - t * dy).powi(2)).sqrt()
}

fn limites_da_camada(doc: &acadrust::document::CadDocument, camada: &str) -> Vec<Vec<[f64; 3]>> {
    doc.model_space_entities()
        .filter_map(|e| match e {
            EntityType::Polyline3D(p) if p.common.layer.eq_ignore_ascii_case(camada) => {
                Some(p.vertices.iter().map(|q| [q.position.x, q.position.y, q.position.z]).collect())
            }
            EntityType::LwPolyline(p) if p.common.layer.eq_ignore_ascii_case(camada) => {
                Some(p.vertices.iter().map(|q| [q.location.x, q.location.y, p.elevation]).collect())
            }
            _ => None,
        })
        .collect()
}
