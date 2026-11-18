//! Spec 003 R7 e R8: assembly da barragem e a seção do corredor em cada estaca.
//!
//! Assembly: crista de largura `crista` com declividade transversal `decl_crista` (fração, padrão 1%) descendo para
//! o lado de montante; de cada borda da crista desce o talude (H:V) do lado, com bermas de `largura_berma` a cada
//! `altura_berma` m de queda (a tabela de patamares do JP: ~3 m a cada 8 m), até encontrar o terreno (saia). Se o
//! terreno estiver acima da borda, sobe em corte com o talude de corte, sem bermas.
//!
//! Offset positivo à direita do eixo (andando no sentido das estacas).

use crate::alinhamento::Alinhamento;
use crate::tin::Tin;

#[derive(Clone, Debug, PartialEq)]
pub struct AssemblyBarragem {
    pub crista: f64,
    /// Declividade transversal da crista (fração; 0,01 = 1%), descendo para montante.
    pub decl_crista: f64,
    /// Montante à esquerda do eixo (`true`) ou à direita.
    pub montante_esquerda: bool,
    /// Taludes H:V (horizontal por 1 vertical).
    pub talude_montante: f64,
    pub talude_jusante: f64,
    /// Bermas: a cada `altura_berma` m de queda, uma berma de `largura_berma` m (0 = sem bermas).
    pub altura_berma: f64,
    pub largura_berma: f64,
    /// Talude de corte H:V quando o terreno está acima da borda.
    pub talude_corte: f64,
    /// Até onde procurar o terreno (m de cada lado).
    pub alcance: f64,
}

impl Default for AssemblyBarragem {
    fn default() -> Self {
        AssemblyBarragem {
            crista: 6.0,
            decl_crista: 0.01,
            montante_esquerda: true,
            talude_montante: 3.0,
            talude_jusante: 2.0,
            altura_berma: 8.0,
            largura_berma: 3.0,
            talude_corte: 1.0,
            alcance: 500.0,
        }
    }
}

/// Ponto da seção do corredor: offset, cota e código.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Codigo {
    /// Saia (encontro com o terreno).
    Saia,
    /// Borda de berma; `k` conta de dentro (1 = mais perto da crista); interna e externa.
    BermaInterna(u8),
    BermaExterna(u8),
    Crista,
    Eixo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SecaoCorredor {
    pub estaca: f64,
    /// Da saia esquerda à saia direita, em ordem de offset.
    pub pontos: Vec<(f64, f64, Codigo)>,
    /// Áreas (m²) entre a seção e o terreno: aterro (seção acima) e corte.
    pub area_aterro: f64,
    pub area_corte: f64,
    /// Achou o terreno dos dois lados.
    pub completa: bool,
}

/// Um lado: da borda da crista para fora. `sinal` = -1 esquerda, +1 direita. Devolve os pontos de fora para
/// dentro do lado (sem a borda) e se achou o terreno.
fn lado(
    terreno: &dyn Fn(f64) -> Option<f64>,
    borda: (f64, f64),
    sinal: f64,
    talude: f64,
    a: &AssemblyBarragem,
) -> (Vec<(f64, f64, Codigo)>, bool) {
    let (o0, z0) = borda;
    let acima = |o: f64, z: f64| terreno(o).map(|t| z - t);
    let mut pts: Vec<(f64, f64, Codigo)> = Vec::new();
    let Some(d0) = acima(o0, z0) else { return (pts, false) };
    // Caminha por trechos retos procurando onde a seção cruza o terreno.
    let achar = |p: (f64, f64), q: (f64, f64)| -> Option<(f64, f64)> {
        let n = (((q.0 - p.0).abs() / 0.25).ceil() as usize).max(1);
        let mut prev = (p.0, p.1, acima(p.0, p.1)?);
        for k in 1..=n {
            let t = k as f64 / n as f64;
            let o = p.0 + (q.0 - p.0) * t;
            let z = p.1 + (q.1 - p.1) * t;
            let d = acima(o, z)?;
            if prev.2 == 0.0 {
                return Some((prev.0, prev.1));
            }
            if (d <= 0.0) != (prev.2 <= 0.0) || d == 0.0 {
                // Bissecção no trecho [prev, atual].
                let (mut lo, mut hi) = (0.0f64, 1.0f64);
                let (a0, b0) = ((prev.0, prev.1), (o, z));
                for _ in 0..40 {
                    let m = (lo + hi) / 2.0;
                    let om = a0.0 + (b0.0 - a0.0) * m;
                    let zm = a0.1 + (b0.1 - a0.1) * m;
                    let dm = acima(om, zm)?;
                    if (dm <= 0.0) == (prev.2 <= 0.0) {
                        lo = m;
                    } else {
                        hi = m;
                    }
                }
                let m = (lo + hi) / 2.0;
                return Some((a0.0 + (b0.0 - a0.0) * m, a0.1 + (b0.1 - a0.1) * m));
            }
            prev = (o, z, d);
        }
        None
    };
    if d0 < 0.0 {
        // Corte: sobe com o talude de corte até o terreno.
        let q = (o0 + sinal * a.alcance, z0 + a.alcance / a.talude_corte.max(1e-6));
        return match achar((o0, z0), q) {
            Some((o, z)) => (vec![(o, z, Codigo::Saia)], true),
            None => (pts, false),
        };
    }
    // Aterro: desce em degraus (talude, berma, talude, ...).
    let (mut o, mut z) = (o0, z0);
    let mut k = 0u8;
    let mut percorrido = 0.0;
    loop {
        let queda = if a.altura_berma > 0.0 && a.largura_berma > 0.0 { a.altura_berma } else { 1e6 };
        let fim = (o + sinal * queda * talude, z - queda);
        if let Some((oi, zi)) = achar((o, z), fim) {
            pts.push((oi, zi, Codigo::Saia));
            break;
        }
        percorrido += queda * talude;
        if percorrido > a.alcance {
            return (Vec::new(), false);
        }
        k += 1;
        pts.push((fim.0, fim.1, Codigo::BermaInterna(k)));
        let fim_berma = (fim.0 + sinal * a.largura_berma, fim.1);
        if let Some((oi, zi)) = achar(fim, fim_berma) {
            pts.push((oi, zi, Codigo::Saia));
            break;
        }
        pts.push((fim_berma.0, fim_berma.1, Codigo::BermaExterna(k)));
        o = fim_berma.0;
        z = fim_berma.1;
    }
    pts.reverse();
    (pts, true)
}

/// Seção do corredor na estaca, com a cota do greide `z_eixo` no eixo.
pub fn secao_do_corredor(tin: &Tin, al: &Alinhamento, estaca: f64, z_eixo: f64, a: &AssemblyBarragem) -> SecaoCorredor {
    let terreno = |o: f64| {
        let p = al.ponto_com_offset(estaca, o);
        tin.cota_em(p[0], p[1])
    };
    let meia = a.crista / 2.0;
    // Crista: desce para montante.
    let (z_esq, z_dir) = if a.montante_esquerda {
        (z_eixo - a.decl_crista * meia, z_eixo + a.decl_crista * meia)
    } else {
        (z_eixo + a.decl_crista * meia, z_eixo - a.decl_crista * meia)
    };
    let (tal_esq, tal_dir) = if a.montante_esquerda {
        (a.talude_montante, a.talude_jusante)
    } else {
        (a.talude_jusante, a.talude_montante)
    };
    let (esq, ok_e) = lado(&terreno, (-meia, z_esq), -1.0, tal_esq, a);
    let (mut dir, ok_d) = lado(&terreno, (meia, z_dir), 1.0, tal_dir, a);
    dir.reverse();
    let mut pontos = esq;
    pontos.push((-meia, z_esq, Codigo::Crista));
    pontos.push((0.0, z_eixo, Codigo::Eixo));
    pontos.push((meia, z_dir, Codigo::Crista));
    pontos.extend(dir);
    // Áreas: integra a diferença entre a seção e o terreno, de saia a saia.
    let (mut aterro, mut corte) = (0.0, 0.0);
    if pontos.len() >= 2 {
        let (o_ini, o_fim) = (pontos[0].0, pontos[pontos.len() - 1].0);
        let z_secao = |o: f64| -> f64 {
            for w in pontos.windows(2) {
                let (a, b) = (w[0], w[1]);
                if o >= a.0 - 1e-12 && o <= b.0 + 1e-12 {
                    let t = if (b.0 - a.0).abs() < 1e-12 { 0.0 } else { (o - a.0) / (b.0 - a.0) };
                    return a.1 + (b.1 - a.1) * t;
                }
            }
            f64::NAN
        };
        let passo = 0.05;
        let n = (((o_fim - o_ini) / passo).ceil() as usize).max(1);
        let h = (o_fim - o_ini) / n as f64;
        for k in 0..n {
            // Ponto médio de cada fatia.
            let o = o_ini + h * (k as f64 + 0.5);
            if let Some(t) = terreno(o) {
                let d = z_secao(o) - t;
                if d.is_finite() {
                    if d > 0.0 {
                        aterro += d * h;
                    } else {
                        corte -= d * h;
                    }
                }
            }
        }
    }
    SecaoCorredor { estaca, pontos, area_aterro: aterro, area_corte: corte, completa: ok_e && ok_d }
}

/// Volumes pelo método das áreas médias entre seções vizinhas: (aterro, corte) em m³.
pub fn volumes(secoes: &[SecaoCorredor]) -> (f64, f64) {
    let mut v = (0.0, 0.0);
    for w in secoes.windows(2) {
        let d = w[1].estaca - w[0].estaca;
        v.0 += (w[0].area_aterro + w[1].area_aterro) / 2.0 * d;
        v.1 += (w[0].area_corte + w[1].area_corte) / 2.0 * d;
    }
    v
}

#[cfg(test)]
mod testes {
    use super::*;

    fn plano(z: f64) -> Tin {
        let v = vec![[-1000.0, -1000.0, z], [1000.0, -1000.0, z], [1000.0, 1000.0, z], [-1000.0, 1000.0, z]];
        Tin::novo(v, vec![[0, 1, 2], [0, 2, 3]])
    }

    fn eixo() -> Alinhamento {
        Alinhamento::de_polilinha(&[([0.0, 0.0], 0.0), ([200.0, 0.0], 0.0)], false).unwrap()
    }

    #[test]
    fn barragem_de_10_m_em_terreno_plano_bate_com_a_conta_a_mao() {
        let tin = plano(100.0);
        let a = AssemblyBarragem { decl_crista: 0.0, ..Default::default() };
        let s = secao_do_corredor(&tin, &eixo(), 100.0, 110.0, &a);
        assert!(s.completa);
        // Montante (esquerda) 3:1: 8 m de queda = 24 m, berma de 3 m, mais 2 m de queda = 6 m. Saia em -3-24-3-6.
        let saia_e = s.pontos.first().unwrap();
        let saia_d = s.pontos.last().unwrap();
        assert!((saia_e.0 - (-36.0)).abs() < 1e-6 && (saia_e.1 - 100.0).abs() < 1e-6, "{saia_e:?}");
        // Jusante 2:1: 16 m, berma 3 m, 4 m: saia em 3+16+3+4 = 26.
        assert!((saia_d.0 - 26.0).abs() < 1e-6, "{saia_d:?}");
        assert!(s.pontos.iter().any(|p| p.2 == Codigo::BermaInterna(1) && (p.1 - 102.0).abs() < 1e-9));
        // Área: crista 6x10 + montante (24·8/2 + 24·2 + 3·2 + 6·2/2) + jusante (16·8/2 + 16·2 + 3·2 + 4·2/2).
        let esperado = 60.0 + (96.0 + 48.0 + 6.0 + 6.0) + (64.0 + 32.0 + 6.0 + 4.0);
        assert!((s.area_aterro - esperado).abs() < 0.01, "{} x {esperado}", s.area_aterro);
        assert!(s.area_corte < 1e-9);
        // Volume num trecho reto de 200 m com seção constante: A·L (critério 6 da spec: 0,5%).
        let secoes: Vec<_> = (0..=10).map(|k| secao_do_corredor(&tin, &eixo(), k as f64 * 20.0, 110.0, &a)).collect();
        let (va, vc) = volumes(&secoes);
        assert!((va - esperado * 200.0).abs() / (esperado * 200.0) < 0.005, "{va}");
        assert!(vc < 1e-6);
    }

    #[test]
    fn crista_desce_1_por_cento_para_montante() {
        let tin = plano(100.0);
        let a = AssemblyBarragem::default();
        let s = secao_do_corredor(&tin, &eixo(), 50.0, 110.0, &a);
        let cristas: Vec<_> = s.pontos.iter().filter(|p| p.2 == Codigo::Crista).collect();
        assert_eq!(cristas.len(), 2);
        assert!((cristas[0].1 - 109.97).abs() < 1e-9, "montante (esquerda) 3 m x 1% abaixo");
        assert!((cristas[1].1 - 110.03).abs() < 1e-9);
        let b = AssemblyBarragem { montante_esquerda: false, ..a };
        let s = secao_do_corredor(&tin, &eixo(), 50.0, 110.0, &b);
        let cristas: Vec<_> = s.pontos.iter().filter(|p| p.2 == Codigo::Crista).collect();
        assert!(cristas[0].1 > cristas[1].1, "montante à direita: desce para a direita");
    }

    #[test]
    fn terreno_acima_da_crista_vira_corte() {
        let tin = plano(112.0);
        let a = AssemblyBarragem { decl_crista: 0.0, ..Default::default() };
        let s = secao_do_corredor(&tin, &eixo(), 100.0, 110.0, &a);
        assert!(s.completa);
        // Corte 1:1 de 2 m: saia a 3 + 2 de cada lado.
        assert!((s.pontos.first().unwrap().0 + 5.0).abs() < 1e-6);
        assert!((s.pontos.last().unwrap().0 - 5.0).abs() < 1e-6);
        assert!(s.area_aterro < 1e-9);
        // Área de corte: trapézio (6 + 10)/2 · 2 = 16.
        assert!((s.area_corte - 16.0).abs() < 0.01, "{}", s.area_corte);
    }

    #[test]
    fn sem_bermas_e_saia_no_meio_do_talude() {
        let tin = plano(105.0);
        let a = AssemblyBarragem { decl_crista: 0.0, altura_berma: 0.0, ..Default::default() };
        let s = secao_do_corredor(&tin, &eixo(), 10.0, 110.0, &a);
        assert_eq!(s.pontos.len(), 5, "saia, crista, eixo, crista, saia: {:?}", s.pontos);
        assert!((s.pontos[0].0 + 18.0).abs() < 1e-6, "3 + 5·3");
        assert!((s.pontos[4].0 - 13.0).abs() < 1e-6, "3 + 5·2");
    }
}
