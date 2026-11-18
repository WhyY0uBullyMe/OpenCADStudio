//! Entrada do greide (`CREATEPROFILE`): um PIV por linha, "estaca,cota" ou "estaca,cota,curva"; ou clique na vista
//! de perfil (a estaca e a cota saem do ponto clicado). Enter termina e despacha `_SETPROFILE <eixo> e,z,L ...`.

use crate::command::{CadCommand, CmdResult};
use glam::DVec3;

pub struct PivsCommand {
    eixo: String,
    /// Vista de perfil para ler o clique: (x0, y0, estaca inicial, base das cotas, exagero).
    vista: Option<(f64, f64, f64, f64, f64)>,
    pivs: Vec<String>,
}

impl PivsCommand {
    pub fn new(eixo: &str, vista: Option<(f64, f64, f64, f64, f64)>) -> Self {
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
        if let Some((x0, y0, e0, base, ex)) = self.vista {
            let e = e0 + (pt.x - x0);
            let z = base + (pt.y - y0) / ex;
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
