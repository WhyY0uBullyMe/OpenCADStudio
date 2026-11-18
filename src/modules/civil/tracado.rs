//! `CREATEALIGNMENTLAYOUT`: traça o eixo clicando os PIs, como as Alignment Creation Tools do Civil 3D (pedido do
//! JP, 18/11). Primeiro o raio das curvas (0 = só tangentes), depois os PIs; Enter (ou espaço) termina. "U" tira o
//! último PI, "@dx,dy" é relativo ao último. No fim despacha `_ALIGNFROMPOINTS <nome> <raio> x,y;x,y;...`.

use crate::command::{CadCommand, CmdResult};
use crate::scene::model::wire_model::WireModel;
use glam::DVec3;

pub struct TracadoCommand {
    nome: String,
    raio: Option<f64>,
    pis: Vec<DVec3>,
}

impl TracadoCommand {
    pub fn new(nome: &str) -> Self {
        TracadoCommand { nome: nome.into(), raio: None, pis: Vec::new() }
    }

    fn terminar(&self) -> CmdResult {
        if self.pis.len() < 2 {
            return CmdResult::Cancel;
        }
        let pts: Vec<String> = self.pis.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
        let nome = if self.nome.contains(' ') { format!("\"{}\"", self.nome) } else { self.nome.clone() };
        CmdResult::Dispatch(format!("_ALIGNFROMPOINTS {nome} {} {}", self.raio.unwrap_or(0.0), pts.join(";")))
    }
}

impl CadCommand for TracadoCommand {
    fn name(&self) -> &'static str {
        "CREATEALIGNMENTLAYOUT"
    }

    fn prompt(&self) -> String {
        match (self.raio, self.pis.len()) {
            (None, _) => "Curve radius at each PI (0 = tangents only) <0>:".into(),
            (Some(_), 0) => format!("Alignment '{}': specify start point:", self.nome),
            (Some(_), 1) => "Specify next PI or [Undo]:".into(),
            _ => "Specify next PI or [Undo] <finish>:".into(),
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
        if self.raio.is_none() {
            let r = t.replace(',', ".").parse::<f64>().ok().filter(|r| *r >= 0.0)?;
            self.raio = Some(r);
            return Some(CmdResult::NeedPoint);
        }
        if t.eq_ignore_ascii_case("U") || t.eq_ignore_ascii_case("UNDO") {
            self.pis.pop();
            return Some(CmdResult::NeedPoint);
        }
        let (rel, xy) = match t.strip_prefix('@') {
            Some(r) => (true, r),
            None => (false, t),
        };
        let v: Vec<f64> = xy.split(',').map(|x| x.trim().parse::<f64>()).collect::<Result<_, _>>().ok()?;
        if v.len() < 2 {
            return None;
        }
        let mut p = DVec3::new(v[0], v[1], 0.0);
        if rel {
            p += self.pis.last().copied().unwrap_or(DVec3::ZERO);
        }
        Some(self.on_point(p))
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        if self.raio.is_none() {
            self.raio = Some(0.0);
        }
        if self.pis.last().is_none_or(|q| q.distance(pt) > 1e-9) {
            self.pis.push(DVec3::new(pt.x, pt.y, 0.0));
        }
        CmdResult::NeedPoint
    }

    fn on_enter(&mut self) -> CmdResult {
        if self.raio.is_none() {
            self.raio = Some(0.0);
            return CmdResult::NeedPoint;
        }
        self.terminar()
    }

    fn on_preview_wires(&mut self, pt: DVec3) -> Vec<WireModel> {
        if self.pis.is_empty() {
            return Vec::new();
        }
        let mut pis: Vec<[f64; 2]> = self.pis.iter().map(|p| [p.x, p.y]).collect();
        pis.push([pt.x, pt.y]);
        let v = agral_civil_motor::alinhamento::tangentes_com_curvas(&pis, self.raio.unwrap_or(0.0));
        let mut pontos: Vec<[f64; 3]> = Vec::new();
        if let Some(al) = agral_civil_motor::alinhamento::Alinhamento::de_polilinha(&v, false) {
            let passo = (al.comprimento() / 400.0).max(0.05);
            let mut e = 0.0;
            while e < al.comprimento() {
                let (p, _) = al.ponto_na_estaca(al.estaca_inicial + e);
                pontos.push([p[0], p[1], 0.0]);
                e += passo;
            }
            pontos.push([pt.x, pt.y, 0.0]);
        }
        // As tangentes (linha dos PIs) em cinza e o eixo com as curvas em ciano.
        let tangentes: Vec<[f64; 3]> = pis.iter().map(|p| [p[0], p[1], 0.0]).collect();
        vec![
            WireModel::solid_f64("agral_tracado_pis".into(), tangentes, [0.6, 0.6, 0.6, 1.0], false),
            WireModel::solid_f64("agral_tracado_eixo".into(), pontos, WireModel::CYAN, false),
        ]
    }
}
