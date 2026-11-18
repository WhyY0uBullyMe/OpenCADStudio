//! Versões de linha de comando dos comandos de superfície (`-CREATESURFACE`, `-ADDSURFACECONTOURS`...), como o
//! AutoCAD faz com o hífen: perguntam cada valor na linha de comando em vez de abrir a janela. Servem para quem
//! digita, para roteiros e para o MCP (que manda cada resposta separada, porque espaço vale Enter).
//! Ao responder tudo, despacha `<VERBO> <resp1> <resp2>...`.

use crate::command::{CadCommand, CmdResult};
use glam::DVec3;

pub struct PerguntasCommand {
    nome: &'static str,
    verbo: &'static str,
    perguntas: &'static [&'static str],
    respostas: Vec<String>,
}

impl PerguntasCommand {
    pub fn new(nome: &'static str, verbo: &'static str, perguntas: &'static [&'static str]) -> Self {
        PerguntasCommand { nome, verbo, perguntas, respostas: Vec::new() }
    }

    /// A pergunta da vez; `<padrão>` no fim do texto é o valor que o Enter aceita.
    fn padrao(&self) -> Option<&'static str> {
        let p = self.perguntas.get(self.respostas.len())?;
        let (_, d) = p.rsplit_once('<')?;
        d.strip_suffix(">:")
    }

    fn responder(&mut self, r: String) -> CmdResult {
        self.respostas.push(r);
        if self.respostas.len() < self.perguntas.len() {
            return CmdResult::NeedPoint;
        }
        let args: Vec<String> = self
            .respostas
            .iter()
            .filter(|r| !r.is_empty())
            .map(|r| if r.contains(' ') { format!("\"{r}\"") } else { r.clone() })
            .collect();
        CmdResult::Dispatch(format!("{} {}", self.verbo, args.join(" ")))
    }
}

impl CadCommand for PerguntasCommand {
    fn name(&self) -> &'static str {
        self.nome
    }

    fn prompt(&self) -> String {
        self.perguntas.get(self.respostas.len()).copied().unwrap_or("").to_string()
    }

    fn wants_text_input(&self) -> bool {
        true
    }

    /// Nome de camada e de estilo pode ter espaço ("CURVA MESTRA-AS", "Curvas 0.5m"): só o Enter responde.
    fn input_kind(&self) -> crate::command::InputKind {
        crate::command::InputKind::FreeText
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        let t = text.trim();
        if t.is_empty() {
            return Some(self.on_enter());
        }
        Some(self.responder(t.to_string()))
    }

    fn on_point(&mut self, _pt: DVec3) -> CmdResult {
        CmdResult::NeedPoint
    }

    fn on_enter(&mut self) -> CmdResult {
        match self.padrao() {
            Some(d) => self.responder(if d.eq_ignore_ascii_case("none") { String::new() } else { d.to_string() }),
            None => CmdResult::Cancel,
        }
    }
}
