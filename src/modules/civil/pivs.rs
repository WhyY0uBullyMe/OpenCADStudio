//! Entrada do greide (`CREATEPROFILE`): um PIV por linha, "estaca,cota" ou "estaca,cota,curva"; ou clique na vista
//! de perfil (a estaca e a cota saem do ponto clicado). Enter termina e despacha `_SETPROFILE <eixo> e,z,L ...`.

use crate::command::{CadCommand, CmdResult};
use crate::scene::model::wire_model::WireModel;
use glam::DVec3;

/// Vista de perfil para ler e mostrar o clique: (x0, y0, estaca inicial, base das cotas, exagero).
pub type Vista = (f64, f64, f64, f64, f64);

/// Estaca e cota do ponto clicado na vista.
pub fn na_vista(v: Vista, p: DVec3) -> (f64, f64) {
    let (x0, y0, e0, base, ex) = v;
    (e0 + (p.x - x0), base + (p.y - y0) / ex)
}

/// Ponto do desenho da estaca e cota.
pub fn no_desenho(v: Vista, e: f64, z: f64) -> [f64; 3] {
    let (x0, y0, e0, base, ex) = v;
    [x0 + (e - e0), y0 + (z - base) * ex, 0.0]
}

/// Prévia do greide na vista: linha pelos PVIs (com o do cursor), um quadradinho em cada PVI e uma cruz
/// no cursor com as linhas de estaca e cota até a borda (o "onde estou clicando", JP 18/11).
pub fn previa(v: Vista, pivs: &[(f64, f64)], cursor: Option<(f64, f64)>, tam: f64) -> Vec<WireModel> {
    let mut out = Vec::new();
    let mut lista: Vec<(f64, f64)> = pivs.to_vec();
    if let Some(c) = cursor {
        lista.push(c);
        lista.sort_by(|a, b| a.0.total_cmp(&b.0));
    }
    if lista.len() >= 2 {
        let pts: Vec<[f64; 3]> = lista.iter().map(|(e, z)| no_desenho(v, *e, *z)).collect();
        out.push(WireModel::solid_f64("agral_greide_previa".into(), pts, [1.0, 0.35, 0.35, 1.0], false));
    }
    for (k, (e, z)) in pivs.iter().enumerate() {
        let c = no_desenho(v, *e, *z);
        let q = vec![
            [c[0] - tam, c[1] - tam, 0.0],
            [c[0] + tam, c[1] - tam, 0.0],
            [c[0] + tam, c[1] + tam, 0.0],
            [c[0] - tam, c[1] + tam, 0.0],
            [c[0] - tam, c[1] - tam, 0.0],
        ];
        out.push(WireModel::solid_f64(format!("agral_piv_{k}"), q, [1.0, 0.85, 0.2, 1.0], false));
    }
    if let Some((e, z)) = cursor {
        let c = no_desenho(v, e, z);
        let (x0, y0, ..) = v;
        out.push(WireModel::solid_f64("agral_piv_estaca".into(), vec![[c[0], y0, 0.0], [c[0], c[1], 0.0]], WireModel::CYAN, false));
        out.push(WireModel::solid_f64("agral_piv_cota".into(), vec![[x0, c[1], 0.0], [c[0], c[1], 0.0]], WireModel::CYAN, false));
    }
    out
}

/// Tamanho dos quadradinhos dos PVIs em unidades do desenho.
fn tamanho(v: Vista) -> f64 {
    (v.4.abs().max(1.0) * 0.4).clamp(0.5, 5.0)
}

pub struct PivsCommand {
    eixo: String,
    vista: Option<Vista>,
    pivs: Vec<String>,
}

impl PivsCommand {
    pub fn new(eixo: &str, vista: Option<Vista>) -> Self {
        PivsCommand { eixo: eixo.into(), vista, pivs: Vec::new() }
    }
}

/// "e,z" ou "e,z,L" (vírgula decimal aceita com ponto e vírgula: "e;z;L").
pub fn ler_piv(t: &str) -> Option<(f64, f64, f64)> {
    let sep = if t.contains(';') { ';' } else { ',' };
    let v: Vec<f64> = t
        .split(sep)
        .map(|x| x.trim().replace(',', "."))
        .map(|x| x.parse::<f64>())
        .collect::<Result<_, _>>()
        .ok()?;
    match v.as_slice() {
        [e, z] => Some((*e, *z, 0.0)),
        [e, z, l] if *l >= 0.0 => Some((*e, *z, *l)),
        _ => None,
    }
}

impl CadCommand for PivsCommand {
    fn name(&self) -> &'static str {
        "CREATEPROFILE"
    }

    fn prompt(&self) -> String {
        format!(
            "PVI {} of '{}': station,elevation[,vertical curve length]{} (Enter to finish):",
            self.pivs.len() + 1,
            self.eixo,
            if self.vista.is_some() { " or click in the profile view" } else { "" }
        )
    }

    fn wants_text_input(&self) -> bool {
        true
    }

    fn input_kind(&self) -> crate::command::InputKind {
        crate::command::InputKind::SingleToken
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        let t = text.trim();
        if t.is_empty() {
            return Some(self.on_enter());
        }
        let (e, z, l) = ler_piv(t)?;
        self.pivs.push(format!("{e},{z},{l}"));
        Some(CmdResult::NeedPoint)
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        if let Some(v) = self.vista {
            let (e, z) = na_vista(v, pt);
            self.pivs.push(format!("{:.3},{:.3},0", e, z));
        }
        CmdResult::NeedPoint
    }

    fn on_enter(&mut self) -> CmdResult {
        if self.pivs.is_empty() {
            return CmdResult::Cancel;
        }
        CmdResult::Dispatch(format!("_SETPROFILE {} {}", self.eixo, self.pivs.join(" ")))
    }

    fn on_preview_wires(&mut self, pt: DVec3) -> Vec<WireModel> {
        let Some(v) = self.vista else { return Vec::new() };
        let pivs: Vec<(f64, f64)> = self.pivs.iter().filter_map(|t| ler_piv(t)).map(|(e, z, _)| (e, z)).collect();
        previa(v, &pivs, Some(na_vista(v, pt)), tamanho(v))
    }
}

#[cfg(test)]
mod testes {
    #[test]
    fn le_piv_com_e_sem_curva() {
        assert_eq!(super::ler_piv("120,955.5"), Some((120.0, 955.5, 0.0)));
        assert_eq!(super::ler_piv("120;955,5;40"), Some((120.0, 955.5, 40.0)));
        assert_eq!(super::ler_piv("abc"), None);
    }
}
