//! Pede um ponto no desenho (e, se for o caso, a cota) para os edits de superfície, e devolve tudo para um
//! comando interno: `<pendente> x,y` ou `<pendente> x,y,z`.

use crate::command::{CadCommand, CmdResult};
use glam::DVec3;

pub struct PegarPontoCommand {
    nome: &'static str,
    texto: String,
    pendente: String,
    pedir_cota: bool,
    ponto: Option<DVec3>,
}

impl PegarPontoCommand {
    pub fn new(nome: &'static str, texto: &str, pendente: String, pedir_cota: bool) -> Self {
        PegarPontoCommand { nome, texto: texto.to_string(), pendente, pedir_cota, ponto: None }
    }
}

impl CadCommand for PegarPontoCommand {
    fn name(&self) -> &'static str {
        self.nome
    }

    fn prompt(&self) -> String {
        match self.ponto {
            Some(p) => format!("Specify elevation <{:.3}>:", p.z),
            None => self.texto.clone(),
        }
    }

    fn wants_text_input(&self) -> bool {
        self.ponto.is_some()
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        if self.pedir_cota {
            self.ponto = Some(pt);
            return CmdResult::NeedPoint;
        }
        CmdResult::Dispatch(format!("{} {},{}", self.pendente, pt.x, pt.y))
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        let p = self.ponto?;
        let t = text.trim().replace(',', ".");
        let z = if t.is_empty() { p.z } else { t.parse::<f64>().ok()? };
        Some(CmdResult::Dispatch(format!("{} {},{},{}", self.pendente, p.x, p.y, z)))
    }

    fn on_enter(&mut self) -> CmdResult {
        match self.ponto {
            // Enter na cota aceita a cota do ponto clicado.
            Some(p) => CmdResult::Dispatch(format!("{} {},{},{}", self.pendente, p.x, p.y, p.z)),
            None => CmdResult::Cancel,
        }
    }
}

/// Nome do comando (estático) para a linha de comando, a partir do verbo do formulário.
pub fn nome_estatico(verbo: &str) -> &'static str {
    match verbo {
        "CREATESECTIONS" => "CREATESECTIONS",
        "CREATEPROFILEVIEW" => "CREATEPROFILEVIEW",
        "CREATEVOLUMETABLE" => "CREATEVOLUMETABLE",
        _ => "AGRAL",
    }
}
