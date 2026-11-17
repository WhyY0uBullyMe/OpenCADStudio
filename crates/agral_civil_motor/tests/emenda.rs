use agral_civil_motor::emenda::{emendar_pedacos, OpcoesEmenda};
use agral_civil_motor::P3;

fn op() -> OpcoesEmenda {
    OpcoesEmenda { folga_maxima: 5.0, limite: None }
}

fn reta(x0: f64, x1: f64, y: f64, z: f64) -> Vec<P3> {
    let n = ((x1 - x0).abs() as usize).max(1);
    (0..=n).map(|i| [x0 + (x1 - x0) * i as f64 / n as f64, y, z]).collect()
}

#[test]
fn dois_pedacos_da_mesma_cota_com_buraco_de_2_m_viram_um() {
    let r = emendar_pedacos(&[reta(0.0, 10.0, 0.0, 10.0), reta(12.0, 20.0, 0.0, 10.0)], &op());
    assert_eq!(r.emendas, 1);
    assert_eq!(r.curvas.len(), 1);
    let c = &r.curvas[0];
    assert_eq!((c[0][0], c[c.len() - 1][0]), (0.0, 20.0));
}

#[test]
fn pedaco_ao_contrario_e_virado_antes_de_emendar() {
    let r = emendar_pedacos(&[reta(0.0, 10.0, 0.0, 10.0), reta(20.0, 12.0, 0.0, 10.0)], &op());
    assert_eq!(r.curvas.len(), 1);
    assert!(r.curvas[0].windows(2).all(|w| w[1][0] > w[0][0]));
}

#[test]
fn buraco_maior_que_a_folga_nao_emenda() {
    let r = emendar_pedacos(&[reta(0.0, 10.0, 0.0, 10.0), reta(16.0, 20.0, 0.0, 10.0)], &op());
    assert_eq!(r.emendas, 0);
    assert_eq!(r.curvas.len(), 2);
}

#[test]
fn cotas_diferentes_nao_emendam() {
    let r = emendar_pedacos(&[reta(0.0, 10.0, 0.0, 10.0), reta(12.0, 20.0, 0.0, 10.5)], &op());
    assert_eq!(r.emendas, 0);
}

#[test]
fn emenda_que_cruzaria_outra_curva_nao_e_feita() {
    let parede = vec![[11.0, -5.0, 11.0], [11.0, 5.0, 11.0]];
    let r = emendar_pedacos(&[reta(0.0, 10.0, 0.0, 10.0), reta(12.0, 20.0, 0.0, 10.0), parede], &op());
    assert_eq!(r.emendas, 0);
}

#[test]
fn pontas_que_apontam_para_lados_opostos_nao_emendam() {
    // Pedaços paralelos lado a lado: as pontas estão perto, mas emendar faria um grampo.
    let r = emendar_pedacos(&[reta(0.0, 10.0, 0.0, 10.0), reta(10.0, 0.0, 2.0, 10.0)], &op());
    assert_eq!(r.emendas, 0);
}

#[test]
fn curva_quase_fechada_e_fechada() {
    // 40 pontos sem repetir o primeiro: falta o último trecho, buraco de 3,1 m.
    let c: Vec<P3> = (0..40)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / 40.0;
            [20.0 * a.cos(), 20.0 * a.sin(), 10.0]
        })
        .collect();
    let r = emendar_pedacos(&[c], &op());
    assert_eq!(r.emendas, 1);
    let f = &r.curvas[0];
    assert_eq!(f[0], f[f.len() - 1]);
}

#[test]
fn ponta_na_borda_da_area_nao_emenda() {
    let limite = vec![[0.0, -50.0, 0.0], [11.0, -50.0, 0.0], [11.0, 50.0, 0.0], [0.0, 50.0, 0.0], [0.0, -50.0, 0.0]];
    let o = OpcoesEmenda { folga_maxima: 5.0, limite: Some(limite) };
    // As duas pontas ficam a menos de 1 m da borda em x = 11: a curva sai da área ali, não é buraco.
    let r = emendar_pedacos(&[reta(0.0, 10.5, 0.0, 10.0), reta(10.6, 0.0, 3.0, 10.0)], &o);
    assert_eq!(r.emendas, 0);
}
