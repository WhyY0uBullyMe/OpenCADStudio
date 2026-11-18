//! Comandos que perguntam na linha de comando (`-CREATESURFACE`, `CREATEPROFILEVIEW` sem argumentos...), como o
//! AutoCAD: cada valor numa pergunta, com o padrão entre `< >` (o Enter aceita). Servem para quem digita, para
//! roteiros e para o MCP (que manda cada resposta separada, porque espaço vale Enter). A última etapa pode pedir
//! um ponto (clique ou "x,y" digitado). No fim despacha `<VERBO> <resp1> <resp2>... [x,y]`.

use crate::command::{CadCommand, CmdResult};
use glam::DVec3;

pub struct PerguntasCommand {
    nome: &'static str,
    verbo: String,
    perguntas: Vec<String>,
    ponto: Option<String>,
    respostas: Vec<String>,
}

impl PerguntasCommand {
    pub fn new(nome: &'static str, verbo: &str, perguntas: &[&str]) -> Self {
        PerguntasCommand {
            nome,
            verbo: verbo.into(),
            perguntas: perguntas.iter().map(|s| s.to_string()).collect(),
            ponto: None,
            respostas: Vec::new(),
        }
    }

    /// Perguntas com padrões calculados na hora (por exemplo, o único alinhamento do desenho).
    pub fn com(nome: &'static str, verbo: &str, perguntas: Vec<String>) -> Self {
        PerguntasCommand { nome, verbo: verbo.into(), perguntas, ponto: None, respostas: Vec::new() }
    }

    /// Depois das perguntas, pede um ponto com este texto.
    pub fn e_ponto(mut self, texto: &str) -> Self {
        self.ponto = Some(texto.into());
        self
    }

    fn no_ponto(&self) -> bool {
        self.respostas.len() >= self.perguntas.len() && self.ponto.is_some()
    }

    /// O padrão da pergunta da vez: `<padrão>` no fim do texto.
    fn padrao(&self) -> Option<String> {
        let p = self.perguntas.get(self.respostas.len())?;
        let (_, d) = p.rsplit_once('<')?;
        d.strip_suffix(">:").map(str::to_string)
    }

    fn despachar(&self, ponto: Option<String>) -> CmdResult {
        let mut args: Vec<String> = self
            .respostas
            .iter()
            .filter(|r| !r.is_empty())
            .map(|r| if r.contains(' ') { format!("\"{r}\"") } else { r.clone() })
            .collect();
        args.extend(ponto);
        CmdResult::Dispatch(format!("{} {}", self.verbo, args.join(" ")))
    }

    fn responder(&mut self, r: String) -> CmdResult {
        self.respostas.push(r);
        if self.respostas.len() < self.perguntas.len() || self.ponto.is_some() {
            return CmdResult::NeedPoint;
        }
        self.despachar(None)
    }
}

fn ler_xy(t: &str) -> Option<String> {
    let v: Vec<f64> = t.split(',').map(|x| x.trim().parse::<f64>()).collect::<Result<_, _>>().ok()?;
    (v.len() >= 2).then(|| format!("{},{}", v[0], v[1]))
}

impl CadCommand for PerguntasCommand {
    fn name(&self) -> &'static str {
        self.nome
    }

    fn prompt(&self) -> String {
        if self.no_ponto() {
            return self.ponto.clone().unwrap_or_default();
        }
        self.perguntas.get(self.respostas.len()).cloned().unwrap_or_default()
    }

    fn wants_text_input(&self) -> bool {
        true
    }

    /// Espaço vale Enter, como no AutoCAD (o JP usa muito, 18/11). Só nome de camada e de estilo, que pode ter
    /// espaço ("CURVA MESTRA-AS", "Curvas 0.5m"), espera o Enter.
    fn input_kind(&self) -> crate::command::InputKind {
        let texto_livre = !self.no_ponto()
            && self.perguntas.get(self.respostas.len()).is_some_and(|p| {
                let p = p.to_ascii_lowercase();
                p.contains("layer") || p.contains("style")
            });
        if texto_livre {
            crate::command::InputKind::FreeText
        } else {
            crate::command::InputKind::SingleToken
        }
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        let t = text.trim();
        if self.no_ponto() {
            return ler_xy(t).map(|p| self.despachar(Some(p)));
        }
        if t.is_empty() {
            return Some(self.on_enter());
        }
        Some(self.responder(t.to_string()))
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        if self.no_ponto() {
            return self.despachar(Some(format!("{},{}", pt.x, pt.y)));
        }
        CmdResult::NeedPoint
    }

    fn on_enter(&mut self) -> CmdResult {
        if self.no_ponto() {
            return CmdResult::NeedPoint;
        }
        match self.padrao() {
            Some(d) => self.responder(if d.eq_ignore_ascii_case("none") { String::new() } else { d }),
            None => CmdResult::Cancel,
        }
    }
}
