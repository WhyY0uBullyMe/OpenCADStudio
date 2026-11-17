use agral_civil_motor::diagnostico::diagnosticar_curvas;

#[test]
fn acha_curvas_de_cotas_diferentes_que_se_cruzam_e_onde() {
    let curvas = vec![
        vec![[0.0, 0.0, 10.0], [10.0, 10.0, 10.0]],
        vec![[0.0, 10.0, 11.0], [10.0, 0.0, 11.0]],
        vec![[0.0, 20.0, 12.0], [10.0, 20.0, 12.0]],
    ];
    let d = diagnosticar_curvas(&curvas);
    assert_eq!(d.cruzamentos.len(), 1);
    let c = &d.cruzamentos[0];
    assert_eq!((c.curva_a, c.curva_b), (0, 1));
    assert_eq!((c.cota_a, c.cota_b), (10.0, 11.0));
    assert!((c.ponto[0] - 5.0).abs() < 1e-9 && (c.ponto[1] - 5.0).abs() < 1e-9);
}

#[test]
fn curva_que_cruza_ela_mesma_tambem_aparece() {
    let laco = vec![[0.0, 0.0, 5.0], [10.0, 10.0, 5.0], [10.0, 0.0, 5.0], [0.0, 10.0, 5.0]];
    let d = diagnosticar_curvas(&[laco]);
    assert_eq!(d.cruzamentos.len(), 1);
    assert_eq!(d.cruzamentos[0].curva_a, d.cruzamentos[0].curva_b);
}

#[test]
fn curvas_que_so_se_tocam_na_ponta_ou_paralelas_nao_contam() {
    let curvas = vec![
        vec![[0.0, 0.0, 10.0], [10.0, 0.0, 10.0]],
        vec![[10.0, 0.0, 10.0], [20.0, 0.0, 10.0]],
        vec![[0.0, 1.0, 11.0], [20.0, 1.0, 11.0]],
    ];
    assert!(diagnosticar_curvas(&curvas).cruzamentos.is_empty());
}

#[test]
fn curva_que_nao_e_plana_e_avisada() {
    let curvas = vec![vec![[0.0, 0.0, 10.0], [10.0, 0.0, 10.5]]];
    let d = diagnosticar_curvas(&curvas);
    assert_eq!(d.curvas_nao_planas, vec![0]);
}
