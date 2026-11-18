//! Pontos das polilinhas do desenho, como o Civil 3D usa para curvas de nível.
//! Polilinha suavizada (spline fit): usa os vértices da curva suavizada e ignora os de controle (achado no
//! `CURVAS_V1` da Fazenda Cedro, 17/11).

use agral_civil_motor::P3;
use document::EntityType;

/// Bit 16 do vértice de polilinha 2D: ponto de controle de spline (não é ponto da curva).
const VERTICE_DE_CONTROLE: u8 = 16;
const POLILINHA_FECHADA: u16 = 1;

pub fn pontos(e: &EntityType) -> Option<Vec<P3>> {
    let mut v: Vec<P3> = match e {
        EntityType::LwPolyline(p) => com_arcos(
            &p.vertices.iter().map(|q| ([q.location.x, q.location.y], q.bulge)).collect::<Vec<_>>(),
            p.is_closed,
            p.elevation,
        ),
        EntityType::Polyline2D(p) => {
            let v: Vec<([f64; 2], f64)> = p
                .vertices
                .iter()
                .filter(|q| q.flags.bits() & VERTICE_DE_CONTROLE == 0)
                .map(|q| ([q.location.x, q.location.y], q.bulge))
                .collect();
            com_arcos(&v, p.flags.bits() & POLILINHA_FECHADA != 0, p.elevation)
        }
        // Curva de nível desenhada como spline: pontos a cada 1 m sobre ela (antes ficava de fora).
        EntityType::Spline(_) => crate::modules::draw::inquiry::divide::pontos_a_cada(e, PASSO_DA_SPLINE)
            .map(|(mut v, fechada)| {
                if fechada && v.len() > 2 && v.first() != v.last() {
                    v.push(v[0]);
                }
                v
            })?,
        EntityType::Polyline3D(p) => {
            let mut v: Vec<P3> = p.vertices.iter().map(|q| [q.position.x, q.position.y, q.position.z]).collect();
            if p.flags.closed && v.len() > 2 {
                v.push(v[0]);
            }
            v
        }
        EntityType::Polyline(p) => p.vertices.iter().map(|q| [q.location.x, q.location.y, q.location.z]).collect(),
        _ => return None,
    };
    v.dedup();
    (v.len() >= 2).then_some(v)
}

/// Flecha máxima (m) entre o arco da polilinha e os pontos que o representam.
const FLECHA: f64 = 0.02;
const PASSO_DA_SPLINE: f64 = 1.0;

/// Vértices de polilinha 2D com os arcos (bulge) virando pontos, com flecha até [`FLECHA`]. Antes o arco virava
/// a corda, e a curva de nível gerada da superfície saía com "dentes" nas curvas suavizadas por arco
/// (JP, 18/11). Fechada repete o primeiro ponto.
fn com_arcos(v: &[([f64; 2], f64)], fechada: bool, z: f64) -> Vec<P3> {
    let n = v.len();
    let mut out: Vec<P3> = Vec::with_capacity(n);
    let trechos = if fechada && n > 2 { n } else { n.saturating_sub(1) };
    if n > 0 {
        out.push([v[0].0[0], v[0].0[1], z]);
    }
    for k in 0..trechos {
        let (a, b) = (v[k].0, v[(k + 1) % n].0);
        let bulge = v[k].1;
        let c = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        if bulge.abs() > 1e-9 && c > 1e-9 {
            let teta = 4.0 * bulge.atan();
            let r = c / (2.0 * (teta / 2.0).sin()).abs();
            // Centro: à esquerda da corda para bulge positivo (anti-horário).
            let (ux, uy) = ((b[0] - a[0]) / c, (b[1] - a[1]) / c);
            let d = c / 2.0 * (1.0 - bulge * bulge) / (2.0 * bulge);
            let centro = [(a[0] + b[0]) / 2.0 - uy * d, (a[1] + b[1]) / 2.0 + ux * d];
            let passo = if r > FLECHA { 2.0 * (1.0 - FLECHA / r).acos() } else { teta.abs() };
            let partes = ((teta.abs() / passo).ceil() as usize).clamp(2, 256);
            let a0 = (a[1] - centro[1]).atan2(a[0] - centro[0]);
            for j in 1..partes {
                let ang = a0 + teta * j as f64 / partes as f64;
                out.push([centro[0] + r * ang.cos(), centro[1] + r * ang.sin(), z]);
            }
        }
        out.push([b[0], b[1], z]);
    }
    out
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Meia volta com bulge 1 (semicírculo de raio 10): os pontos ficam no círculo, não na corda.
    #[test]
    fn arco_da_polilinha_vira_pontos_no_arco() {
        let v = com_arcos(&[([0.0, 0.0], 1.0), ([20.0, 0.0], 0.0)], false, 5.0);
        assert!(v.len() > 20, "{}", v.len());
        assert_eq!(v[0], [0.0, 0.0, 5.0]);
        assert_eq!(*v.last().unwrap(), [20.0, 0.0, 5.0]);
        for p in &v {
            let r = ((p[0] - 10.0).powi(2) + p[1].powi(2)).sqrt();
            assert!((r - 10.0).abs() < 1e-9, "r = {r}");
            assert!(p[1] <= 1e-9, "bulge positivo de (0,0) a (20,0) passa por baixo (anti-horário)");
        }
        // Flecha entre pontos seguidos até 2 cm.
        let seg = ((v[1][0] - v[0][0]).powi(2) + (v[1][1] - v[0][1]).powi(2)).sqrt();
        assert!(10.0 - (100.0 - seg * seg / 4.0).sqrt() <= FLECHA + 1e-9);
    }

    #[test]
    fn sem_arco_fica_igual_e_fechada_repete_o_primeiro() {
        let v = com_arcos(&[([0.0, 0.0], 0.0), ([1.0, 0.0], 0.0), ([1.0, 1.0], 0.0)], true, 2.0);
        assert_eq!(v, vec![[0.0, 0.0, 2.0], [1.0, 0.0, 2.0], [1.0, 1.0, 2.0], [0.0, 0.0, 2.0]]);
    }
}
