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
        EntityType::LwPolyline(p) => {
            let mut v: Vec<P3> = p.vertices.iter().map(|q| [q.location.x, q.location.y, p.elevation]).collect();
            if p.is_closed && v.len() > 2 {
                v.push(v[0]);
            }
            v
        }
        EntityType::Polyline2D(p) => {
            let mut v: Vec<P3> = p
                .vertices
                .iter()
                .filter(|q| q.flags.bits() & VERTICE_DE_CONTROLE == 0)
                .map(|q| [q.location.x, q.location.y, p.elevation])
                .collect();
            if p.flags.bits() & POLILINHA_FECHADA != 0 && v.len() > 2 {
                v.push(v[0]);
            }
            v
        }
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
