//! DIVGRIPS e SPL2PL, Lisps do JP refeitos no programa (pedido de 18/11):
//! - DIVGRIPS: refaz a polilinha com vértices a cada X m ao longo dela (`DIVGRIPS.lsp` do JP).
//! - SPL2PL: spline (ou qualquer curva) para polilinha com a quantidade de vértices escolhida (o `Spl2pl.vlx`
//!   é compilado e não foi lido; feito pela descrição do JP).
//!
//! Pede a curva, o valor e se apaga a original; devolve tudo para `_REFAZERCURVA <modo> <handle> <valor> <S|N>`.

use crate::command::{CadCommand, CmdResult};
use document::{EntityType, Handle};
use glam::DVec3;

pub const APLICAR: &str = "_REFAZERCURVA";

pub struct RefazerCurvaCommand {
    nome: &'static str,
    curva: Option<Handle>,
    valida: bool,
    valor: Option<String>,
}

impl RefazerCurvaCommand {
    pub fn new(nome: &'static str) -> Self {
        RefazerCurvaCommand { nome, curva: None, valida: false, valor: None }
    }

    fn divgrips(&self) -> bool {
        self.nome == "DIVGRIPS"
    }
}

impl CadCommand for RefazerCurvaCommand {
    fn name(&self) -> &'static str {
        self.nome
    }

    fn prompt(&self) -> String {
        match (self.curva, &self.valor) {
            (None, _) => "Select polyline, spline or curve:".into(),
            (Some(_), None) if self.divgrips() => "Distance between vertices:".into(),
            (Some(_), None) => "Number of vertices:".into(),
            (Some(_), Some(_)) => "Delete original? [Yes/No] <No>:".into(),
        }
    }

    fn needs_entity_pick(&self) -> bool {
        self.curva.is_none()
    }

    fn inject_before_entity_pick(&self) -> bool {
        true
    }

    fn inject_picked_entity(&mut self, entity: EntityType) {
        self.valida = crate::modules::draw::inquiry::divide::pontos_em_n(&entity, 2).is_some();
    }

    fn on_entity_pick(&mut self, handle: Handle, _point: DVec3) -> CmdResult {
        if !handle.is_null() && self.valida {
            self.curva = Some(handle);
        }
        CmdResult::NeedPoint
    }

    fn on_point(&mut self, _pt: DVec3) -> CmdResult {
        CmdResult::NeedPoint
    }

    fn wants_text_input(&self) -> bool {
        self.curva.is_some()
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        let t = text.trim();
        let h = self.curva?;
        match &self.valor {
            None => {
                let t = t.replace(',', ".");
                let ok = if self.divgrips() {
                    t.parse::<f64>().ok().is_some_and(|v| v.is_finite() && v > 0.0)
                } else {
                    t.parse::<usize>().ok().is_some_and(|n| (2..=1_000_000).contains(&n))
                };
                if !ok {
                    return None;
                }
                self.valor = Some(t);
                Some(CmdResult::NeedPoint)
            }
            Some(v) => {
                let apagar = match t.to_ascii_uppercase().as_str() {
                    "" | "N" | "NO" | "NAO" | "NÃO" => "N",
                    "Y" | "YES" | "S" | "SIM" => "S",
                    _ => return None,
                };
                Some(CmdResult::Dispatch(format!("{APLICAR} {} {:X} {v} {apagar}", self.nome, h.value())))
            }
        }
    }

    fn on_enter(&mut self) -> CmdResult {
        match (&self.curva, &self.valor) {
            (Some(_), Some(_)) => self.on_text_input("N").unwrap_or(CmdResult::Cancel),
            _ => CmdResult::Cancel,
        }
    }
}

inventory::submit!(crate::command::CommandRegistration { names: &["DIVGRIPS", "SPL2PL"] });
