//! `EDITPROFILE`: mexe no greide depois de criado (JP, 18/11), clicando na vista de perfil, com prévia:
//! Move (clica no PVI e depois no lugar novo), Add (clica onde quer o PVI), Delete (clica no PVI), Curve
//! (clica no PVI e digita o comprimento da curva vertical). Cada mudança grava e redesenha na hora; Enter termina.
//! Também aceita "estaca,cota" digitado no lugar do clique.

use super::pivs::{ler_piv, na_vista, previa, Vista};
use crate::command::{CadCommand, CmdResult};
use crate::scene::model::wire_model::WireModel;
use glam::DVec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Modo {
    Mover,
    Incluir,
    Apagar,
    Curva,
}

impl Modo {
    pub fn token(self) -> &'static str {
        match self {
            Modo::Mover => "MOVE",
            Modo::Incluir => "ADD",
            Modo::Apagar => "DELETE",
            Modo::Curva => "CURVE",
        }
    }

    pub fn de(t: &str) -> Option<Modo> {
        let t = t.to_ascii_uppercase();
        if t.is_empty() {
            return None;
        }
        [Modo::Mover, Modo::Incluir, Modo::Apagar, Modo::Curva].into_iter().find(|m| m.token().starts_with(&t) || (t == "M" && *m == Modo::Mover))
    }
}

pub struct PerfilEdicaoCommand {
    eixo: String,
    vista: Option<Vista>,
    /// (estaca, cota, curva).
    pivs: Vec<(f64, f64, f64)>,
    modo: Modo,
    /// PVI escolhido (para mover ou para a curva).
    escolhido: Option<usize>,
}

impl PerfilEdicaoCommand {
    pub fn new(eixo: &str, vista: Option<Vista>, pivs: Vec<(f64, f64, f64)>, modo: Modo) -> Self {
        PerfilEdicaoCommand { eixo: eixo.into(), vista, pivs, modo, escolhido: None }
    }

    fn mais_perto(&self, e: f64, z: f64) -> Option<usize> {
        let ex = self.vista.map_or(1.0, |v| v.4);
        self.pivs
            .iter()
            .enumerate()
            .min_by(|a, b| {
                let d = |p: &(f64, f64, f64)| (p.0 - e).powi(2) + ((p.1 - z) * ex).powi(2);
                d(a.1).total_cmp(&d(b.1))
            })
            .map(|(k, _)| k)
    }

    fn gravar(&mut self) -> CmdResult {
        self.pivs.sort_by(|a, b| a.0.total_cmp(&b.0));
        let lista: Vec<String> = self.pivs.iter().map(|(e, z, l)| format!("{e:.3},{z:.3},{l}")).collect();
        CmdResult::Dispatch(format!("_SETPROFILE {} {} +EDIT={}", self.eixo, lista.join(" "), self.modo.token()))
    }

    /// Um ponto (clicado ou digitado) na vista.
    fn ponto(&mut self, e: f64, z: f64) -> CmdResult {
        match (self.modo, self.escolhido) {
            (Modo::Mover, Some(k)) => {
                let l = self.pivs[k].2;
                self.pivs[k] = (e, z, l);
                self.escolhido = None;
                self.gravar()
            }
            (Modo::Mover, None) | (Modo::Curva, None) => {
                self.escolhido = self.mais_perto(e, z);
                CmdResult::NeedPoint
            }
            (Modo::Incluir, _) => {
                self.pivs.push((e, z, 0.0));
                self.gravar()
            }
            (Modo::Apagar, _) => {
                if self.pivs.len() <= 1 {
                    return CmdResult::NeedPoint;
                }
                if let Some(k) = self.mais_perto(e, z) {
                    self.pivs.remove(k);
                }
                self.gravar()
            }
            (Modo::Curva, Some(_)) => CmdResult::NeedPoint,
        }
    }
}

impl CadCommand for PerfilEdicaoCommand {
    fn name(&self) -> &'static str {
        "EDITPROFILE"
    }

    fn prompt(&self) -> String {
        let opcoes = "[Move/Add/Delete/Curve] (Enter to finish)";
        match (self.modo, self.escolhido) {
            (Modo::Mover, None) => format!("Edit profile '{}': click the PVI to move or {opcoes}:", self.eixo),
            (Modo::Mover, Some(k)) => format!("New position of PVI {} (click, or station,elevation):", k + 1),
            (Modo::Incluir, _) => format!("Edit profile '{}': click where to add a PVI (or station,elevation) or {opcoes}:", self.eixo),
            (Modo::Apagar, _) => format!("Edit profile '{}': click the PVI to delete or {opcoes}:", self.eixo),
            (Modo::Curva, None) => format!("Edit profile '{}': click the PVI for the vertical curve or {opcoes}:", self.eixo),
            (Modo::Curva, Some(k)) => format!("Vertical curve length at PVI {} <{}>:", k + 1, self.pivs[k].2),
        }
    }

    fn wants_text_input(&self) -> bool {
        true
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        let t = text.trim();
        if t.is_empty() {
            return Some(self.on_enter());
        }
        if let (Modo::Curva, Some(k)) = (self.modo, self.escolhido) {
            let l = t.replace(',', ".").parse::<f64>().ok().filter(|l| *l >= 0.0)?;
            self.pivs[k].2 = l;
            self.escolhido = None;
            return Some(self.gravar());
        }
        if let Some(m) = Modo::de(t) {
            self.modo = m;
            self.escolhido = None;
            return Some(CmdResult::NeedPoint);
        }
        let (e, z, _) = ler_piv(t)?;
        Some(self.ponto(e, z))
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        let Some(v) = self.vista else { return CmdResult::NeedPoint };
        let (e, z) = na_vista(v, pt);
        self.ponto(e, z)
    }

    fn on_enter(&mut self) -> CmdResult {
        if let (Modo::Curva, Some(k)) = (self.modo, self.escolhido) {
            // Enter mantém o comprimento atual.
            let _ = k;
            self.escolhido = None;
            return CmdResult::NeedPoint;
        }
        CmdResult::Cancel
    }

    fn on_preview_wires(&mut self, pt: DVec3) -> Vec<WireModel> {
        let Some(v) = self.vista else { return Vec::new() };
        let c = na_vista(v, pt);
        let mut pivs: Vec<(f64, f64)> = self.pivs.iter().map(|p| (p.0, p.1)).collect();
        let tam = (v.4.abs().max(1.0) * 0.4).clamp(0.5, 5.0);
        match (self.modo, self.escolhido) {
            (Modo::Mover, Some(k)) => {
                pivs.remove(k);
                previa(v, &pivs, Some(c), tam)
            }
            (Modo::Incluir, _) => previa(v, &pivs, Some(c), tam),
            (Modo::Apagar, _) => {
                if let Some(k) = self.mais_perto(c.0, c.1) {
                    if pivs.len() > 1 {
                        pivs.remove(k);
                    }
                }
                let mut w = previa(v, &pivs, None, tam);
                w.extend(previa(v, &[], Some(c), tam));
                w
            }
            _ => {
                let mut w = previa(v, &pivs, None, tam);
                w.extend(previa(v, &[], Some(c), tam));
                w
            }
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn cmd(modo: Modo) -> PerfilEdicaoCommand {
        PerfilEdicaoCommand::new("EIXO1", Some((0.0, 0.0, 0.0, 100.0, 10.0)), vec![(0.0, 100.0, 0.0), (100.0, 105.0, 0.0)], modo)
    }

    fn despacho(r: CmdResult) -> String {
        match r {
            CmdResult::Dispatch(s) => s,
            _ => panic!("esperava despacho"),
        }
    }

    #[test]
    fn mover_incluir_apagar_e_curva() {
        let mut c = cmd(Modo::Mover);
        // Clique perto do segundo PVI (estaca 98, cota 104,8 -> y = 48), depois o lugar novo (estaca 120, cota 106).
        assert!(matches!(c.on_point(DVec3::new(98.0, 48.0, 0.0)), CmdResult::NeedPoint));
        let d = despacho(c.on_point(DVec3::new(120.0, 60.0, 0.0)));
        assert_eq!(d, "_SETPROFILE EIXO1 0.000,100.000,0 120.000,106.000,0 +EDIT=MOVE");
        let mut c = cmd(Modo::Incluir);
        let d = despacho(c.on_text_input("50,110").unwrap());
        assert!(d.contains("50.000,110.000,0 100.000"), "{d}");
        let mut c = cmd(Modo::Apagar);
        let d = despacho(c.on_point(DVec3::new(1.0, 0.0, 0.0)));
        assert_eq!(d, "_SETPROFILE EIXO1 100.000,105.000,0 +EDIT=DELETE");
        let mut c = cmd(Modo::Curva);
        c.on_point(DVec3::new(99.0, 50.0, 0.0));
        assert!(c.prompt().contains("PVI 2"));
        let d = despacho(c.on_text_input("40").unwrap());
        assert!(d.ends_with("100.000,105.000,40 +EDIT=CURVE"), "{d}");
        // Troca de modo pelo teclado.
        let mut c = cmd(Modo::Mover);
        c.on_text_input("A");
        assert!(c.prompt().contains("add"));
    }
}
