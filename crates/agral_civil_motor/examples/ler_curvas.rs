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
    // VERTICES=controle: usa o polígono de controle da spline em vez da curva suavizada.
    let usar_controle = std::env::var("VERTICES").is_ok_and(|v| v == "controle");
    let mut sem_ajuste = 0;
    let mut flags_pl: BTreeMap<u16, usize> = BTreeMap::new();
    let mut flags_v: BTreeMap<u8, usize> = BTreeMap::new();
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
                *flags_pl.entry(p.flags.bits()).or_default() += 1;
                for q in &p.vertices {
                    *flags_v.entry(q.flags.bits()).or_default() += 1;
                }
                // Vértice de controle de spline não é ponto da curva (fica fora dela).
                let mut v: Vec<[f64; 3]> = p
                    .vertices
                    .iter()
                    .filter(|q| {
                        let controle = q.flags.bits() & 16 != 0;
                        if usar_controle { controle || p.flags.bits() & 4 == 0 } else { !controle }
                    })
                    .map(|q| [q.location.x, q.location.y, p.elevation])
                    .collect();
                if v.is_empty() && !p.vertices.is_empty() {
                    sem_ajuste += 1;
                }
                if p.flags.bits() & 1 != 0 && v.len() > 2 {
                    v.push(v[0]);
                }
                (p.common.layer.clone(), v)
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
    println!("polilinhas spline sem vértices de ajuste: {sem_ajuste}");
    let diag = agral_civil_motor::diagnostico::diagnosticar_curvas(&linhas);
    let difer = diag.cruzamentos.iter().filter(|c| (c.cota_a - c.cota_b).abs() > 1e-6).count();
    let mut pares: Vec<(usize, usize)> = diag.cruzamentos.iter().map(|c| (c.curva_a, c.curva_b)).collect();
    pares.sort();
    pares.dedup();
    println!(
        "diagnóstico: {} cruzamentos ({} entre cotas diferentes, {} na mesma cota), {} pares de curvas, {} curvas não planas",
        diag.cruzamentos.len(), difer, diag.cruzamentos.len() - difer, pares.len(), diag.curvas_nao_planas.len()
    );
    for c in diag.cruzamentos.iter().take(4) {
        println!("  {:.2} x {:.2} em {:.2} {:.2}", c.cota_a, c.cota_b, c.ponto[0], c.ponto[1]);
    }
    // Pedaços abertos: distância da ponta até a ponta mais próxima de outro pedaço na mesma cota.
    {
        let abertas: Vec<usize> = (0..linhas.len())
            .filter(|&k| {
                let l = &linhas[k];
                let (a, b) = (l[0], l[l.len() - 1]);
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() > 1e-3
            })
            .collect();
        let pontas: Vec<(usize, [f64; 3])> = abertas.iter().flat_map(|&k| [(k, linhas[k][0]), (k, *linhas[k].last().unwrap())]).collect();
        let mut faixas = [0usize; 6];
        let nomes = ["até 1 mm", "1 mm a 10 cm", "10 cm a 1 m", "1 a 5 m", "5 a 20 m", "mais de 20 m ou nenhuma"];
        for &(k, p) in &pontas {
            let d = pontas
                .iter()
                .filter(|(o, q)| *o != k && (q[2] - p[2]).abs() < 1e-6)
                .map(|(_, q)| ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt())
                .fold(f64::INFINITY, f64::min);
            let i = [0.001, 0.1, 1.0, 5.0, 20.0].iter().position(|&lim| d <= lim).unwrap_or(5);
            faixas[i] += 1;
        }
        println!("pedaços abertos: {} de {}; pontas por distância até outro pedaço da mesma cota:", abertas.len(), linhas.len());
        for (n, c) in nomes.iter().zip(faixas) {
            println!("  {n}: {c}");
        }
    }
    println!("marcas das polilinhas 2D: {flags_pl:?}; marcas dos vértices: {flags_v:?}");
    let mut maior = (0.0f64, 0usize, 0usize);
    for (k, l) in linhas.iter().enumerate() {
        for (i, w) in l.windows(2).enumerate() {
            let d = ((w[0][0] - w[1][0]).powi(2) + (w[0][1] - w[1][1]).powi(2)).sqrt();
            if d > maior.0 {
                maior = (d, k, i);
            }
        }
    }
    println!("maior trecho: {:.1} m, polilinha {} ({} vértices), trecho {}", maior.0, maior.1, linhas.get(maior.1).map_or(0, |l| l.len()), maior.2);
    for (k, (n, v, z0, z1)) in &por_camada {
        println!("camada {k}: {n} polilinhas, {v} vértices, cota {z0:.3} a {z1:.3}");
    }
    let vertices: usize = linhas.iter().map(|l| l.len()).sum();
    println!("usadas: {} polilinhas, {} vértices", linhas.len(), vertices);

    // CORRIGIR=1: corrige os cruzamentos (R18) antes de montar.
    if std::env::var("CORRIGIR").is_ok() {
        let t = Instant::now();
        let r = agral_civil_motor::correcao::corrigir_cruzamentos(&linhas, &Default::default());
        println!(
            "correção: {} barrigas cortadas, {} lentes separadas, {} trechos cortados, {} rodadas, {} curvas alteradas, sobraram {} cruzamentos, em {:?}",
            r.barrigas_cortadas, r.lentes_separadas, r.trechos_cortados, r.rodadas, r.alteradas.len(), r.restantes, t.elapsed()
        );
        let antes: f64 = linhas.iter().map(|l| l.windows(2).map(|w| ((w[0][0]-w[1][0]).powi(2)+(w[0][1]-w[1][1]).powi(2)).sqrt()).sum::<f64>()).sum();
        let depois: f64 = r.curvas.iter().map(|l| l.windows(2).map(|w| ((w[0][0]-w[1][0]).powi(2)+(w[0][1]-w[1][1]).powi(2)).sqrt()).sum::<f64>()).sum();
        println!("comprimento total das curvas: antes {antes:.1} m, depois {depois:.1} m");
        linhas = r.curvas;
    }
    // EMENDAR=<folga em m>: emenda os pedaços (R19), usando o LIMITE como borda da área se houver.
    if let Ok(f) = std::env::var("EMENDAR") {
        // LIMITE_ARQ=<arquivo com "E N" por linha> tem preferência sobre LIMITE=<camada>.
        let do_arquivo = std::env::var("LIMITE_ARQ").ok().map(|a| {
            std::fs::read_to_string(a)
                .unwrap()
                .lines()
                .filter_map(|l| {
                    let v: Vec<f64> = l.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                    (v.len() >= 2).then(|| [v[0], v[1], 0.0])
                })
                .collect::<Vec<_>>()
        });
        let limite = do_arquivo.or_else(|| std::env::var("LIMITE").ok().and_then(|l| limites_da_camada(&doc, &l).into_iter().next())).map(|mut l| {
            if l.first() != l.last() {
                l.push(l[0]);
            }
            l
        });
        let t = Instant::now();
        let antes = linhas.len();
        let e = agral_civil_motor::emenda::emendar_pedacos(
            &linhas,
            &agral_civil_motor::emenda::OpcoesEmenda { folga_maxima: f.parse().unwrap_or(5.0), limite },
        );
        let abertas = e.curvas.iter().filter(|c| ((c[0][0] - c[c.len() - 1][0]).powi(2) + (c[0][1] - c[c.len() - 1][1]).powi(2)).sqrt() > 1e-3).count();
        let cruz = agral_civil_motor::diagnostico::diagnosticar_curvas(&e.curvas).cruzamentos.len();
        println!(
            "emenda: {} emendas ({} fecharam o próprio pedaço), {} curvas viraram {}, ainda abertas {}, cruzamentos depois {}, em {:?}",
            e.emendas, e.fechadas, antes, e.curvas.len(), abertas, cruz, t.elapsed()
        );
        linhas = e.curvas;
    }
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
    let m = s.montar();
    if !m.trechos_em_conflito.is_empty() {
        let comp = |c: &[[f64; 3]; 2]| ((c[0][0] - c[1][0]).powi(2) + (c[0][1] - c[1][1]).powi(2)).sqrt();
        let mut ls: Vec<f64> = m.trechos_em_conflito.iter().map(comp).collect();
        ls.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("trechos em conflito: {}, comprimento mediano {:.3} m, máximo {:.3} m", ls.len(), ls[ls.len() / 2], ls[ls.len() - 1]);
        for c in m.trechos_em_conflito.iter().take(5) {
            println!("  {:.3} {:.3} {:.2} -> {:.3} {:.3} {:.2}", c[0][0], c[0][1], c[0][2], c[1][0], c[1][1], c[1][2]);
        }
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
    // CIVIL3D=<LandXML exportado do Civil 3D>: compara cotas ponto a ponto (critério 4: 1 cm no interior).
    if let Ok(xml) = std::env::var("CIVIL3D") {
        let t = Instant::now();
        let texto = std::fs::read_to_string(&xml).expect("ler LandXML");
        let c3d = agral_civil_motor::landxml::ler_superficies(&texto).expect("LandXML");
        drop(texto);
        let c = &c3d[0].tin;
        println!("Civil 3D: {} vértices, {} triângulos (lido em {:?})", c.vertices.len(), c.triangulos.len(), t.elapsed());
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for v in &c.vertices {
            x0 = x0.min(v[0]); y0 = y0.min(v[1]); x1 = x1.max(v[0]); y1 = y1.max(v[1]);
        }
        let mut difs: Vec<f64> = Vec::new();
        let passo = ((x1 - x0) * (y1 - y0) / 200_000.0).sqrt();
        let mut y = y0 + passo / 2.0;
        while y < y1 {
            let mut x = x0 + passo / 2.0;
            while x < x1 {
                if let (Some(a), Some(b)) = (c.cota_em(x, y), s.tin.cota_em(x, y)) {
                    difs.push((a - b).abs());
                }
                x += passo;
            }
            y += passo;
        }
        difs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q = |f: f64| difs[((difs.len() - 1) as f64 * f) as usize];
        let ate1cm = difs.iter().filter(|d| **d <= 0.01).count();
        println!(
            "comparação com o Civil 3D: {} pontos em grade de {:.1} m; até 1 cm: {:.1} %; mediana {:.3} m; 95 % {:.3} m; 99 % {:.3} m; máximo {:.3} m",
            difs.len(), passo, 100.0 * ate1cm as f64 / difs.len() as f64, q(0.5), q(0.95), q(0.99), q(1.0)
        );
    }
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
