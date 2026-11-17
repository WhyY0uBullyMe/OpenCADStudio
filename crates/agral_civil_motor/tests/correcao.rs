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
fn lente_entre_dois_cruzamentos_e_separada_sem_mexer_fora_dela() {
    // A reta em y = 0; B passa por cima dela, mas entre x = 4 e x = 6 desce para y = -1 (invade o lado de A).
    let a: Vec<P3> = (0..=10).map(|i| [i as f64, 0.0, 10.0]).collect();
    let b = vec![[0.0, 1.0, 10.5], [3.0, 1.0, 10.5], [4.5, -1.0, 10.5], [5.5, -1.0, 10.5], [7.0, 1.0, 10.5], [10.0, 1.0, 10.5]];
    let r = corrigir_cruzamentos(&[a.clone(), b.clone()], &op());
    assert!(diagnosticar_curvas(&r.curvas).cruzamentos.is_empty());
    assert_eq!(r.lentes_separadas, 1);
    assert_eq!(r.trechos_cortados, 0);
    // Fora da lente nada muda, e cada curva fica na sua cota.
    assert_eq!(&r.curvas[0][..4], &a[..4]);
    assert_eq!(&r.curvas[0][r.curvas[0].len() - 3..], &a[8..]);
    assert_eq!(r.curvas[1][0], b[0]);
    assert!(r.curvas[0].iter().all(|p| p[2] == 10.0) && r.curvas[1].iter().all(|p| p[2] == 10.5));
    // Na lente, A continua abaixo de B (o lado que tinha fora dela).
    for p in &r.curvas[0] {
        let zb = r.curvas[1].iter().filter(|q| (q[0] - p[0]).abs() < 0.3).map(|q| q[1]).fold(f64::INFINITY, f64::min);
        assert!(p[1] < zb + 1e-9, "A em {p:?} passou para cima de B");
    }
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
    eprintln!("lentes {} cortes {} rodadas {}", r.lentes_separadas, r.trechos_cortados, r.rodadas);
    assert_eq!(r.restantes, 0);
}
