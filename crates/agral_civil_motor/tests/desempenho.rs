//! R14: 1 milhão de pontos em menos de 1 minuto. Rodar em release:
//! `cargo test -p agral_civil_motor --release --test desempenho -- --ignored --nocapture`
use agral_civil_motor::superficie::{ItemDefinicao, Superficie};
use std::time::Instant;

#[test]
#[ignore]
fn um_milhao_de_pontos_de_curvas_em_menos_de_um_minuto() {
    // 1000 curvas ondulantes de 1000 vértices: 1 milhão de pontos, todos como linha de quebra.
    let (x0, y0) = (500_000.0, 7_500_000.0);
    let curvas: Vec<Vec<[f64; 3]>> = (0..1000)
        .map(|k| {
            (0..1000)
                .map(|i| {
                    let x = i as f64 * 2.0;
                    let y = k as f64 * 2.0 + 0.6 * (x / 37.0 + k as f64 * 0.1).sin();
                    [x0 + x, y0 + y, 700.0 + k as f64 * 0.5]
                })
                .collect()
        })
        .collect();
    let mut s = Superficie::nova("TN", "");
    s.acrescentar(ItemDefinicao::curvas("curvas", curvas));
    let t = Instant::now();
    let avisos = s.refazer();
    let montar = t.elapsed();
    let t = Instant::now();
    let curvas = s.tin.curvas_de_nivel(950.25);
    let gerar = t.elapsed();
    println!(
        "{} vértices, {} triângulos, montar {:?}, curva {:?} ({} trechos), avisos {:?}",
        s.tin.vertices.len(),
        s.tin.triangulos.len(),
        montar,
        gerar,
        curvas.len(),
        avisos
    );
    assert!(s.tin.triangulos.len() > 1_900_000);
    assert!(montar.as_secs() < 60);
    assert!(avisos.is_empty());
}
