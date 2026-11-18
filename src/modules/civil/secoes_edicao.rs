//! `EDITSAMPLELINES`: arrasta as sample lines na planta (JP, 18/11: "arrastar as sample lines e ter um melhor
//! controle"), como o EDITPROFILE faz com os PVIs: Move (clica na linha e depois no lugar novo), Add (clica onde
//! quer a linha), Delete (clica na linha). A prévia mostra a linha no cursor, perpendicular ao eixo, com a estaca.
//! Cada mudança grava e redesenha; Enter termina. Também aceita a estaca digitada.

use super::perfil_edicao::Modo;
use crate::command::{CadCommand, CmdResult};
use crate::scene::model::wire_model::WireModel;
use agral_civil_motor::alinhamento::{texto_da_estaca, Alinhamento};
use glam::DVec3;

pub struct SecoesEdicaoCommand {
    eixo: String,
    al: Alinhamento,
    esquerda: f64,
    direita: f64,
    estacas: Vec<f64>,
    modo: Modo,
    escolhida: Option<usize>,
}

impl SecoesEdicaoCommand {
    pub fn new(eixo: &str, al: Alinhamento, esquerda: f64, direita: f64, estacas: Vec<f64>, modo: Modo) -> Self {
        let modo = if modo == Modo::Curva { Modo::Mover } else { modo };
        SecoesEdicaoCommand { eixo: eixo.into(), al, esquerda, direita, estacas, modo, escolhida: None }
    }

    fn estaca_do_ponto(&self, p: DVec3) -> f64 {
        let (e, _) = self.al.estaca_e_offset([p.x, p.y]);
        e.clamp(self.al.estaca_inicial, self.al.estaca_final())
    }

    fn mais_perto(&self, e: f64) -> Option<usize> {
        self.estacas.iter().enumerate().min_by(|a, b| (a.1 - e).abs().total_cmp(&(b.1 - e).abs())).map(|(k, _)| k)
    }

    fn gravar(&mut self) -> CmdResult {
        self.estacas.sort_by(|a, b| a.total_cmp(b));
        self.estacas.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
        let lista: Vec<String> = self.estacas.iter().map(|e| format!("{e:.3}")).collect();
        CmdResult::Dispatch(format!("_SETSAMPLELINES {} {} +EDIT={}", self.eixo, lista.join(" "), self.modo.token()))
    }

    fn na_estaca(&mut self, e: f64) -> CmdResult {
        match (self.modo, self.escolhida) {
            (Modo::Mover, Some(k)) => {
                self.estacas[k] = e;
                self.escolhida = None;
                self.gravar()
            }
            (Modo::Mover, None) => {
                self.escolhida = self.mais_perto(e);
                CmdResult::NeedPoint
            }
            (Modo::Incluir, _) => {
                self.estacas.push(e);
                self.gravar()
            }
            (Modo::Apagar, _) => {
                if self.estacas.len() <= 1 {
                    return CmdResult::NeedPoint;
                }
                if let Some(k) = self.mais_perto(e) {
                    self.estacas.remove(k);
                }
                self.gravar()
            }
            (Modo::Curva, _) => CmdResult::NeedPoint,
        }
    }

    fn linha(&self, nome: String, e: f64, cor: [f32; 4]) -> WireModel {
        let a = self.al.ponto_com_offset(e, -self.esquerda);
        let b = self.al.ponto_com_offset(e, self.direita);
        WireModel::solid_f64(nome, vec![[a[0], a[1], 0.0], [b[0], b[1], 0.0]], cor, false)
    }
}

impl CadCommand for SecoesEdicaoCommand {
    fn name(&self) -> &'static str {
        "EDITSAMPLELINES"
    }

    fn prompt(&self) -> String {
        let opcoes = "[Move/Add/Delete] (Enter to finish)";
        match (self.modo, self.escolhida) {
            (Modo::Mover, Some(k)) => format!("New station of sample line {} (now {}): click or type the station:", k + 1, texto_da_estaca(self.estacas[k])),
            (Modo::Mover, _) => format!("Edit sample lines of '{}': click the line to move or {opcoes}:", self.eixo),
            (Modo::Incluir, _) => format!("Edit sample lines of '{}': click where to add a line (or type the station) or {opcoes}:", self.eixo),
            _ => format!("Edit sample lines of '{}': click the line to delete or {opcoes}:", self.eixo),
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
        if let Some(m) = Modo::de(t).filter(|m| *m != Modo::Curva) {
            self.modo = m;
            self.escolhida = None;
            return Some(CmdResult::NeedPoint);
        }
        // Estaca como número (m) ou como "1+234.5".
        let e = match t.split_once('+') {
            Some((km, m)) => km.trim().parse::<f64>().ok()? * 1000.0 + m.trim().replace(',', ".").parse::<f64>().ok()?,
            None => t.replace(',', ".").parse::<f64>().ok()?,
        };
        Some(self.na_estaca(e.clamp(self.al.estaca_inicial, self.al.estaca_final())))
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        let e = self.estaca_do_ponto(pt);
        self.na_estaca(e)
    }

    fn on_enter(&mut self) -> CmdResult {
        CmdResult::Cancel
    }

    fn on_preview_wires(&mut self, pt: DVec3) -> Vec<WireModel> {
        let e = self.estaca_do_ponto(pt);
        let mut w = Vec::new();
        for (k, est) in self.estacas.iter().enumerate() {
            let em_foco = Some(k) == self.escolhida || (self.escolhida.is_none() && self.modo != Modo::Incluir && self.mais_perto(e) == Some(k));
            let cor = if em_foco {
                if self.modo == Modo::Apagar { [1.0, 0.3, 0.3, 1.0] } else { [1.0, 0.85, 0.2, 1.0] }
            } else {
                [0.45, 0.6, 0.8, 1.0]
            };
            w.push(self.linha(format!("agral_sl_{k}"), *est, cor));
        }
        if self.modo == Modo::Incluir || self.escolhida.is_some() {
            w.push(self.linha("agral_sl_cursor".into(), e, WireModel::CYAN));
        }
        w
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn cmd(modo: Modo) -> SecoesEdicaoCommand {
        let al = Alinhamento::de_polilinha(&[([0.0, 0.0], 0.0), ([100.0, 0.0], 0.0)], false).unwrap();
        SecoesEdicaoCommand::new("EIXO1", al, 10.0, 10.0, vec![0.0, 50.0, 100.0], modo)
    }

    fn despacho(r: CmdResult) -> String {
        match r {
            CmdResult::Dispatch(s) => s,
            _ => panic!("esperava despacho"),
        }
    }

    #[test]
    fn move_inclui_e_apaga_sample_lines() {
        let mut c = cmd(Modo::Mover);
        // Clica perto da linha de 50 (x = 48, fora do eixo) e leva para 62.
        assert!(matches!(c.on_point(DVec3::new(48.0, 3.0, 0.0)), CmdResult::NeedPoint));
        assert!(c.prompt().contains("0+050.00"));
        assert_eq!(despacho(c.on_point(DVec3::new(62.0, -4.0, 0.0))), "_SETSAMPLELINES EIXO1 0.000 62.000 100.000 +EDIT=MOVE");
        let mut c = cmd(Modo::Incluir);
        assert_eq!(despacho(c.on_text_input("0+025").unwrap()), "_SETSAMPLELINES EIXO1 0.000 25.000 50.000 100.000 +EDIT=ADD");
        let mut c = cmd(Modo::Apagar);
        assert_eq!(despacho(c.on_point(DVec3::new(99.0, 0.0, 0.0))), "_SETSAMPLELINES EIXO1 0.000 50.000 +EDIT=DELETE");
        assert_eq!(c.on_preview_wires(DVec3::new(10.0, 0.0, 0.0)).len(), 2);
    }
}
