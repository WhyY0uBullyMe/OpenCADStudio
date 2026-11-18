//! `CREATEALIGNMENT <nome>` sem a camada: "Selecione a polilinha do eixo ou digite a camada". Clique numa
//! polilinha despacha `_ALIGNFROMHANDLE <nome> <handle>`; texto despacha `CREATEALIGNMENT <nome> <camada>`.

use crate::command::{CadCommand, CmdResult};
use document::{EntityType, Handle};
use glam::DVec3;

pub struct EixoFonteCommand {
    nome: String,
    valida: bool,
}

impl EixoFonteCommand {
    pub fn new(nome: &str) -> Self {
        EixoFonteCommand { nome: nome.into(), valida: false }
    }
}

impl CadCommand for EixoFonteCommand {
    fn name(&self) -> &'static str {
        "CREATEALIGNMENT"
    }

    fn prompt(&self) -> String {
        format!("Alignment '{}': select the polyline, or type its layer name:", self.nome)
    }

    fn needs_entity_pick(&self) -> bool {
        true
    }

    fn inject_before_entity_pick(&self) -> bool {
        true
    }

    fn inject_picked_entity(&mut self, entity: EntityType) {
        self.valida = crate::app::civil_alinhamento::vertices_com_bulge(&entity).is_some_and(|(v, _)| v.len() >= 2);
    }

    fn on_entity_pick(&mut self, handle: Handle, _point: DVec3) -> CmdResult {
        if handle.is_null() || !self.valida {
            return CmdResult::NeedPoint;
        }
        CmdResult::Dispatch(format!("_ALIGNFROMHANDLE {} {:X}", self.nome, handle.value()))
    }

    fn wants_text_input(&self) -> bool {
        true
    }

    fn input_kind(&self) -> crate::command::InputKind {
        crate::command::InputKind::FreeText
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        let t = text.trim();
        if t.is_empty() {
            return Some(CmdResult::Cancel);
        }
        let camada = if t.contains(' ') { format!("\"{t}\"") } else { t.to_string() };
        Some(CmdResult::Dispatch(format!("CREATEALIGNMENT {} {camada}", self.nome)))
    }

    fn on_point(&mut self, _pt: DVec3) -> CmdResult {
        CmdResult::NeedPoint
    }

    fn on_enter(&mut self) -> CmdResult {
        CmdResult::Cancel
    }
}
