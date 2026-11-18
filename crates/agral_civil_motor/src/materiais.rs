//! Compute Materials (spec Hydrus `docs/specs/agral-cad/004-compute-materials`): corte e aterro entre duas
//! superfícies nas linhas de seção de um eixo, volumes pelo método das áreas médias e fatores de empolamento.

use crate::alinhamento::Alinhamento;
use crate::tin::Tin;

/// Uma linha da tabela de volumes. Volumes do trecho que termina nesta estaca (zero na primeira), já com os fatores.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LinhaMaterial {
    pub estaca: f64,
    pub area_corte: f64,
    pub area_aterro: f64,
    pub vol_corte: f64,
    pub vol_aterro: f64,
    pub acum_corte: f64,
    pub acum_aterro: f64,
}

impl LinhaMaterial {
    /// Volume líquido acumulado: corte menos aterro (positivo sobra material, negativo falta).
    pub fn liquido(&self) -> f64 {
        self.acum_corte - self.acum_aterro
    }
}

/// Áreas (corte, aterro) em m² na seção da estaca, de `esquerda` a `direita` do eixo: aterro onde `comparar` fica
/// acima de `base`, corte onde fica abaixo. Só conta onde as duas superfícies existem.
pub fn areas_na_secao(base: &Tin, comparar: &Tin, al: &Alinhamento, estaca: f64, esquerda: f64, direita: f64) -> (f64, f64) {
    let (esq, dir) = (esquerda.abs(), direita.abs());
    let passo = ((esq + dir) / 400.0).clamp(0.05, 0.5);
    let n = ((esq + dir) / passo).ceil().max(1.0) as usize;
    let dif = |o: f64| {
        let p = al.ponto_com_offset(estaca, o);
        Some(comparar.cota_em(p[0], p[1])? - base.cota_em(p[0], p[1])?)
    };
    let (mut corte, mut aterro) = (0.0, 0.0);
    let mut ant: Option<(f64, f64)> = None;
    for k in 0..=n {
        let o = -esq + (esq + dir) * k as f64 / n as f64;
        let atual = dif(o).map(|d| (o, d));
        if let (Some((o0, d0)), Some((o1, d1))) = (ant, atual) {
            let l = o1 - o0;
            if d0 * d1 >= 0.0 {
                let a = (d0 + d1) / 2.0 * l;
                if a > 0.0 { aterro += a } else { corte -= a }
            } else {
                // Troca de sinal no trecho: corta no zero.
                let t = d0 / (d0 - d1);
                let (a0, a1) = (d0 * t * l / 2.0, d1 * (1.0 - t) * l / 2.0);
                for a in [a0, a1] {
                    if a > 0.0 { aterro += a } else { corte -= a }
                }
            }
        }
        ant = atual;
    }
    (corte, aterro)
}

/// Tabela de volumes a partir das áreas por estaca (em ordem de estaca): áreas médias entre vizinhas, fatores
/// aplicados ao volume (Civil 3D: cut factor, fill factor).
pub fn tabela(areas: &[(f64, f64, f64)], fator_corte: f64, fator_aterro: f64) -> Vec<LinhaMaterial> {
    let mut out: Vec<LinhaMaterial> = Vec::with_capacity(areas.len());
    for (k, &(estaca, area_corte, area_aterro)) in areas.iter().enumerate() {
        let mut l = LinhaMaterial { estaca, area_corte, area_aterro, ..Default::default() };
        if let Some(a) = k.checked_sub(1).map(|j| out[j]) {
            let d = estaca - a.estaca;
            l.vol_corte = (a.area_corte + area_corte) / 2.0 * d * fator_corte;
            l.vol_aterro = (a.area_aterro + area_aterro) / 2.0 * d * fator_aterro;
            l.acum_corte = a.acum_corte + l.vol_corte;
            l.acum_aterro = a.acum_aterro + l.vol_aterro;
        }
        out.push(l);
    }
    out
}

#[cfg(test)]
mod testes {
    use super::*;

    fn plano(z: f64) -> Tin {
        let v = vec![[-1000.0, -1000.0, z], [1000.0, -1000.0, z], [1000.0, 1000.0, z], [-1000.0, 1000.0, z]];
        Tin::novo(v, vec![[0, 1, 2], [0, 2, 3]])
    }

    fn eixo() -> Alinhamento {
        Alinhamento::de_polilinha(&[([0.0, 0.0], 0.0), ([100.0, 0.0], 0.0)], false).unwrap()
    }

    /// Spec 004, aceite: 2 m de aterro em 40 m de largura = 80 m2; 100 m dão 8000 m3; fator 1,25 dá 10000 m3.
    #[test]
    fn aterro_entre_planos_e_fator() {
        let (b, c, al) = (plano(100.0), plano(102.0), eixo());
        let (corte, aterro) = areas_na_secao(&b, &c, &al, 40.0, 20.0, 20.0);
        assert!(corte.abs() < 1e-9 && (aterro - 80.0).abs() < 1e-9, "{corte} {aterro}");
        let areas: Vec<_> = al.estacas(20.0).into_iter().map(|e| { let (c, a) = areas_na_secao(&b, &c, &al, e, 20.0, 20.0); (e, c, a) }).collect();
        assert_eq!(areas.len(), 6);
        let t = tabela(&areas, 1.0, 1.0);
        assert!((t.last().unwrap().acum_aterro - 8000.0).abs() < 1e-6);
        let t = tabela(&areas, 1.0, 1.25);
        assert!((t.last().unwrap().acum_aterro - 10000.0).abs() < 1e-6);
        assert!((t.last().unwrap().liquido() + 10000.0).abs() < 1e-6);
        assert_eq!(t[0].vol_aterro, 0.0);
    }

    /// Base acima da comparação é corte; superfície inclinada que cruza a base divide corte e aterro no zero.
    #[test]
    fn corte_e_troca_de_sinal() {
        let al = eixo();
        let (corte, aterro) = areas_na_secao(&plano(102.0), &plano(100.0), &al, 50.0, 10.0, 10.0);
        assert!((corte - 40.0).abs() < 1e-9 && aterro.abs() < 1e-9);
        // Comparação z = 100 + y/10 (y é o afastamento à esquerda, positivo): de -10 a 10 vai de 99 a 101.
        let v = vec![[-1000.0, -1000.0, 0.0], [1000.0, -1000.0, 0.0], [1000.0, 1000.0, 200.0], [-1000.0, 1000.0, 200.0]];
        let inclinada = Tin::novo(v, vec![[0, 1, 2], [0, 2, 3]]);
        let (corte, aterro) = areas_na_secao(&plano(100.0), &inclinada, &al, 50.0, 10.0, 10.0);
        assert!((corte - 5.0).abs() < 1e-6 && (aterro - 5.0).abs() < 1e-6, "{corte} {aterro}");
    }
}
