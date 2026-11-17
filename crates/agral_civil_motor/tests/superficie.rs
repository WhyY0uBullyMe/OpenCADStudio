use agral_civil_motor::superficie::{ItemDefinicao, Superficie};
use agral_civil_motor::tin::{segmentos_se_cruzam, Tin};
use agral_civil_motor::P3;

/// Cone invertido de curvas poligonais: curvas circulares concêntricas, cota sobe para fora. Origem em coordenadas UTM reais,
/// para pegar erro de precisão com números grandes.
fn curvas_circulares(n_curvas: usize, pontos_por_curva: usize) -> Vec<Vec<P3>> {
    let (cx, cy) = (500_000.0, 7_500_000.0);
    (1..=n_curvas)
        .map(|k| {
            let r = 10.0 * k as f64;
            let z = 800.0 + k as f64;
            let mut c: Vec<P3> = (0..pontos_por_curva)
                .map(|i| {
                    // Cada curva girada um pouco: os vértices não ficam alinhados de uma curva para a outra (caso real).
                    let a = std::f64::consts::TAU * i as f64 / pontos_por_curva as f64 + 0.45 * k as f64;
                    [cx + r * a.cos(), cy + r * a.sin(), z]
                })
                .collect();
            c.push(c[0]);
            c
        })
        .collect()
}

#[test]
fn superficie_nova_e_vazia_e_aparece_com_nome() {
    let s = Superficie::nova("TN", "Terreno natural");
    assert_eq!(s.nome, "TN");
    assert!(s.definicao.is_empty());
    let tin = s.montar().tin;
    assert_eq!(tin.triangulos.len(), 0);
}

#[test]
fn pontos_viram_triangulos_e_cota_interpolada_e_plana() {
    let mut s = Superficie::nova("TN", "");
    // Plano z = 800 + 0,1 x + 0,2 y (em coordenadas locais da grade).
    let mut pts = vec![];
    for i in 0..10 {
        for j in 0..10 {
            let (x, y) = (i as f64 * 5.0, j as f64 * 5.0);
            pts.push([500_000.0 + x, 7_500_000.0 + y, 800.0 + 0.1 * x + 0.2 * y]);
        }
    }
    s.acrescentar(ItemDefinicao::pontos("grade.txt", pts));
    let r = s.montar();
    assert_eq!(r.tin.triangulos.len(), 2 * 9 * 9);
    let z = r.tin.cota_em(500_000.0 + 12.3, 7_500_000.0 + 31.7).unwrap();
    assert!((z - (800.0 + 1.23 + 6.34)).abs() < 1e-6, "z = {z}");
    assert!(r.tin.cota_em(499_000.0, 7_500_000.0).is_none(), "fora da superfície não tem cota");
}

#[test]
fn nenhum_triangulo_cruza_uma_curva_de_nivel() {
    let curvas = curvas_circulares(8, 8);
    let mut s = Superficie::nova("TN", "");
    s.acrescentar(ItemDefinicao::curvas("curvas.dwg", curvas.clone()));
    let r = s.montar();
    assert!(r.avisos.is_empty(), "{:?}", r.avisos);
    let cruzamentos = arestas_que_cruzam(&r.tin, &curvas);
    assert_eq!(cruzamentos, 0);
}

#[test]
fn sem_linha_de_quebra_a_delaunay_pura_cruzaria_as_curvas() {
    // Prova de que o teste anterior pega o erro: com as mesmas curvas entrando como pontos soltos
    // (como no plugin holandês), há triângulos cruzando curvas.
    let curvas = curvas_circulares(8, 8);
    let mut s = Superficie::nova("TN", "");
    s.acrescentar(ItemDefinicao::pontos("curvas como pontos", curvas.iter().flatten().copied().collect()));
    let r = s.montar();
    assert!(arestas_que_cruzam(&r.tin, &curvas) > 0);
}

#[test]
fn tirar_ou_desligar_item_refaz_a_superficie() {
    let mut s = Superficie::nova("TN", "");
    s.acrescentar(ItemDefinicao::pontos("a", vec![[0.0, 0.0, 1.0], [10.0, 0.0, 1.0], [0.0, 10.0, 1.0]]));
    s.acrescentar(ItemDefinicao::pontos("b", vec![[10.0, 10.0, 5.0]]));
    assert_eq!(s.montar().tin.triangulos.len(), 2);
    s.definicao[1].ligado = false;
    assert_eq!(s.montar().tin.triangulos.len(), 1);
    s.remover(0);
    assert_eq!(s.montar().tin.triangulos.len(), 0);
}

#[test]
fn surface_marcada_desatualizada_ate_refazer() {
    let mut s = Superficie::nova("TN", "");
    assert!(!s.desatualizada());
    s.acrescentar(ItemDefinicao::pontos("a", vec![[0.0, 0.0, 1.0], [10.0, 0.0, 1.0], [0.0, 10.0, 1.0]]));
    assert!(s.desatualizada());
    let _ = s.refazer();
    assert!(!s.desatualizada());
}

#[test]
fn comprimento_maximo_de_triangulo_tira_triangulo_comprido_da_borda() {
    // Pontos em "L": sem limite, o fecho convexo cria triângulos longos no canto vazio.
    let mut pts = vec![];
    for i in 0..=10 {
        pts.push([i as f64 * 10.0, 0.0, 1.0]);
        pts.push([0.0, i as f64 * 10.0, 1.0]);
        pts.push([i as f64 * 10.0, 10.0, 1.0]);
        pts.push([10.0, i as f64 * 10.0, 1.0]);
    }
    let mut s = Superficie::nova("TN", "");
    s.acrescentar(ItemDefinicao::pontos("L", pts));
    let sem = s.montar().tin.triangulos.len();
    s.opcoes.comprimento_maximo = Some(15.0);
    let com = s.montar();
    assert!(com.tin.triangulos.len() < sem);
    assert!(com.tin.cota_em(60.0, 60.0).is_none());
    assert!(com.tin.cota_em(5.0, 5.0).is_some());
}

#[test]
fn limite_externo_e_furo() {
    let mut pts = vec![];
    for i in 0..=20 {
        for j in 0..=20 {
            pts.push([i as f64, j as f64, 1.0]);
        }
    }
    let mut s = Superficie::nova("TN", "");
    s.acrescentar(ItemDefinicao::pontos("grade", pts));
    s.acrescentar(ItemDefinicao::limite_externo(
        "limite",
        vec![[2.0, 2.0, 0.0], [18.0, 2.0, 0.0], [18.0, 18.0, 0.0], [2.0, 18.0, 0.0], [2.0, 2.0, 0.0]],
    ));
    s.acrescentar(ItemDefinicao::furo(
        "lagoa",
        vec![[8.0, 8.0, 0.0], [12.0, 8.0, 0.0], [12.0, 12.0, 0.0], [8.0, 12.0, 0.0], [8.0, 8.0, 0.0]],
    ));
    let t = s.montar().tin;
    assert!(t.cota_em(1.0, 1.0).is_none(), "fora do limite");
    assert!(t.cota_em(10.0, 10.0).is_none(), "dentro do furo");
    assert!(t.cota_em(5.0, 5.0).is_some());
    assert!(t.cota_em(15.0, 15.0).is_some());
}

#[test]
fn curvas_que_se_cruzam_geram_aviso_e_nao_travam() {
    let mut s = Superficie::nova("TN", "");
    s.acrescentar(ItemDefinicao::curvas(
        "ruins",
        vec![
            vec![[0.0, 0.0, 10.0], [10.0, 10.0, 10.0]],
            vec![[0.0, 10.0, 11.0], [10.0, 0.0, 11.0]],
            vec![[0.0, 5.0, 9.0], [5.0, -5.0, 9.0]],
        ],
    ));
    let r = s.montar();
    assert!(!r.avisos.is_empty());
    assert!(r.tin.triangulos.len() > 0);
}

#[test]
fn curvas_geradas_passam_pelas_curvas_de_entrada() {
    // Critério 3: a curva gerada da superfície coincide com a de entrada (desvio < 1/4 do intervalo).
    let curvas = curvas_circulares(6, 48);
    let mut s = Superficie::nova("TN", "");
    s.acrescentar(ItemDefinicao::curvas("curvas", curvas.clone()));
    let r = s.montar();
    for entrada in &curvas[1..curvas.len() - 1] {
        let z = entrada[0][2];
        let geradas = r.tin.curvas_de_nivel(z);
        assert!(!geradas.is_empty(), "sem curva gerada na cota {z}");
        for g in &geradas {
            for p in g {
                let d = distancia_a_polilinha(p, entrada);
                assert!(d < 0.25, "desvio {d} m na cota {z}");
            }
        }
    }
}

fn arestas_que_cruzam(tin: &Tin, curvas: &[Vec<P3>]) -> usize {
    let mut n = 0;
    for t in &tin.triangulos {
        for k in 0..3 {
            let a = tin.vertices[t[k] as usize];
            let b = tin.vertices[t[(k + 1) % 3] as usize];
            for c in curvas {
                for w in c.windows(2) {
                    if segmentos_se_cruzam(a, b, w[0], w[1]) {
                        n += 1;
                    }
                }
            }
        }
    }
    n
}

fn distancia_a_polilinha(p: &P3, linha: &[P3]) -> f64 {
    linha
        .windows(2)
        .map(|w| {
            let (ax, ay, bx, by) = (w[0][0], w[0][1], w[1][0], w[1][1]);
            let (dx, dy) = (bx - ax, by - ay);
            let t = (((p[0] - ax) * dx + (p[1] - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
            ((p[0] - ax - t * dx).powi(2) + (p[1] - ay - t * dy).powi(2)).sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}
