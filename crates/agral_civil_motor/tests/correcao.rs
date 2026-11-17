use agral_civil_motor::correcao::{corrigir_cruzamentos, OpcoesCorrecao};
use agral_civil_motor::diagnostico::diagnosticar_curvas;
use agral_civil_motor::P3;

fn op() -> OpcoesCorrecao {
    OpcoesCorrecao { folga: 0.05, rodadas: 50 }
}

#[test]
fn curvas_sem_cruzamento_ficam_iguais() {
    let c = vec![vec![[0.0, 0.0, 10.0], [10.0, 0.0, 10.0]], vec![[0.0, 1.0, 11.0], [10.0, 1.0, 11.0]]];
    let r = corrigir_cruzamentos(&c, &op());
    assert_eq!(r.curvas, c);
    assert!(r.alteradas.is_empty());
}

#[test]
fn barriga_da_curva_de_cima_e_cortada_e_a_curva_refeita_sem_canto() {
    // Regra do JP (17/11): a curva de cima (B, 10,5) faz uma barriga para dentro da de baixo (A, 10,0).
    // A barriga é cortada e B volta a seguir o caminho que tinha antes e depois dela; A não muda.
    let a: Vec<P3> = (0..=10).map(|i| [i as f64, 0.0, 10.0]).collect();
    let b = vec![[0.0, 1.0, 10.5], [2.0, 1.0, 10.5], [3.0, 1.0, 10.5], [4.5, -1.0, 10.5], [5.5, -1.0, 10.5], [7.0, 1.0, 10.5], [8.0, 1.0, 10.5], [10.0, 1.0, 10.5]];
    let r = corrigir_cruzamentos(&[a.clone(), b.clone()], &op());
    assert!(diagnosticar_curvas(&r.curvas).cruzamentos.is_empty());
    assert_eq!(r.barrigas_cortadas, 1);
    assert_eq!((r.lentes_separadas, r.trechos_cortados), (0, 0));
    assert_eq!(r.curvas[0], a, "a curva de baixo não muda");
    assert_eq!(r.curvas[1][..3], b[..3]);
    assert_eq!(r.curvas[1][r.curvas[1].len() - 3..], b[5..]);
    for p in &r.curvas[1] {
        assert_eq!(p[2], 10.5);
        assert!((p[1] - 1.0).abs() < 0.05, "B refeita fora do alinhamento em {p:?}");
    }
}

#[test]
fn barriga_da_curva_de_baixo_tambem_e_achada() {
    // Mesma situação invertida: a de baixo é que entra na de cima.
    let a: Vec<P3> = (0..=10).map(|i| [i as f64, 1.0, 10.5]).collect();
    let b = vec![[0.0, 0.0, 10.0], [2.0, 0.0, 10.0], [3.0, 0.0, 10.0], [4.5, 2.0, 10.0], [5.5, 2.0, 10.0], [7.0, 0.0, 10.0], [8.0, 0.0, 10.0], [10.0, 0.0, 10.0]];
    let r = corrigir_cruzamentos(&[a.clone(), b], &op());
    assert!(diagnosticar_curvas(&r.curvas).cruzamentos.is_empty());
    assert_eq!(r.curvas[0], a);
    assert!(r.curvas[1].iter().all(|p| p[1].abs() < 0.05));
}

#[test]
fn barriga_que_nao_cabe_cai_na_linha_do_meio() {
    // As duas curvas se cruzam em X longo, sem barriga clara: refazer uma delas ainda cruzaria a outra.
    let a: Vec<P3> = vec![[0.0, 0.0, 10.0], [4.0, 0.0, 10.0], [5.0, 2.0, 10.0], [6.0, 0.0, 10.0], [10.0, 0.0, 10.0]];
    let b: Vec<P3> = vec![[0.0, 0.5, 10.5], [4.0, 0.5, 10.5], [5.0, -1.5, 10.5], [6.0, 0.5, 10.5], [10.0, 0.5, 10.5]];
    let r = corrigir_cruzamentos(&[a, b], &op());
    assert!(diagnosticar_curvas(&r.curvas).cruzamentos.is_empty());
}

#[test]
fn ponta_de_curva_que_cruza_uma_vez_e_cortada() {
    let a = vec![[0.0, 0.0, 10.0], [10.0, 0.0, 10.0]];
    let b = vec![[2.0, 5.0, 11.0], [2.0, 1.0, 11.0], [5.0, -1.0, 11.0]];
    let r = corrigir_cruzamentos(&[a, b], &op());
    assert!(diagnosticar_curvas(&r.curvas).cruzamentos.is_empty());
    assert_eq!(r.trechos_cortados, 1);
}

#[test]
fn laco_da_curva_nela_mesma_some() {
    let laco = vec![[0.0, 0.0, 5.0], [10.0, 10.0, 5.0], [10.0, 0.0, 5.0], [0.0, 10.0, 5.0]];
    let r = corrigir_cruzamentos(&[laco], &op());
    assert!(diagnosticar_curvas(&r.curvas).cruzamentos.is_empty());
}

#[test]
fn varias_curvas_onduladas_cruzando_terminam_sem_cruzamento() {
    // Curvas paralelas a 0,5 m com ondulação de 0,8 m: cruzam as vizinhas várias vezes (caso de spline exagerada).
    let curvas: Vec<Vec<P3>> = (0..6)
        .map(|k| {
            (0..=400)
                .map(|i| {
                    let x = i as f64 * 0.5;
                    [x, k as f64 * 0.5 + 0.8 * ((x + k as f64 * 30.0) / 9.0).sin(), 100.0 + k as f64 * 0.5]
                })
                .collect()
        })
        .collect();
    assert!(!diagnosticar_curvas(&curvas).cruzamentos.is_empty());
    let r = corrigir_cruzamentos(&curvas, &op());
    assert!(diagnosticar_curvas(&r.curvas).cruzamentos.is_empty(), "sobraram {}", r.restantes);
    eprintln!("barrigas {} lentes {} cortes {} rodadas {}", r.barrigas_cortadas, r.lentes_separadas, r.trechos_cortados, r.rodadas);
    assert_eq!(r.restantes, 0);
}
