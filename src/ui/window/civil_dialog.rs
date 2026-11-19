//! Janelas do Agral Civil, como as do Civil 3D: Create Surface, Add Contour Data, Surface Properties e a
//! confirmação de Delete.

use crate::app::Message;
use crate::ui::style::form::{dialog_button, field_style};
use iced::widget::{checkbox, column, container, pick_list, row, text, text_input, Space};
use iced::{Element, Fill, Length};

#[derive(Debug, Clone, PartialEq)]
pub enum Modo {
    Criar,
    /// Add Contour Data para a superfície escolhida.
    Curvas,
    Propriedades(String),
    Apagar(String),
    /// Agral CAD: confirmação do DELETEALIGNMENT (apaga também perfil, seções, corredor e tabela).
    ApagarEixo(String),
    /// Add Boundaries: tipo e descrição, depois a seleção.
    Limite(String),
    /// Raise/Lower Surface: quanto sobe (negativo desce).
    Elevar(String),
    /// Agral CAD (JP, 18/11: "botões e janelas para fazer as configurações"): formulário de um comando, campo a
    /// campo na ordem dos argumentos; OK despacha o comando (e pede o ponto no desenho, se ele precisar).
    Formulario(Formulario),
}

/// Um campo do formulário: texto livre ou lista de opções.
#[derive(Debug, Clone, PartialEq)]
pub struct Campo {
    pub rotulo: String,
    pub valor: String,
    /// Vazio: campo de texto. Com opções: lista para escolher.
    pub opcoes: Vec<String>,
    /// Caixa de marcar: valor "1" ou "0"; marcada, entra no comando o próprio rótulo (um nome de superfície).
    pub marca: bool,
}

impl Campo {
    pub fn texto(rotulo: &str, valor: impl ToString) -> Self {
        Campo { rotulo: rotulo.into(), valor: valor.to_string(), opcoes: Vec::new(), marca: false }
    }
    pub fn marca(rotulo: &str, marcada: bool) -> Self {
        Campo { rotulo: rotulo.into(), valor: if marcada { "1" } else { "0" }.into(), opcoes: Vec::new(), marca: true }
    }
    pub fn lista(rotulo: &str, valor: impl ToString, opcoes: Vec<String>) -> Self {
        Campo { rotulo: rotulo.into(), valor: valor.to_string(), opcoes, marca: false }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Formulario {
    pub titulo: String,
    /// Comando despachado no OK: `<verbo> <campo1> <campo2> ...`.
    pub verbo: String,
    pub campos: Vec<Campo>,
    /// Texto da pergunta do ponto no desenho, depois do OK (canto de uma vista, por exemplo).
    pub ponto: Option<String>,
    pub nota: String,
}

impl Formulario {
    /// O comando com os valores (entre aspas se tiverem espaço); `Err` se faltar algum.
    pub fn comando(&self) -> Result<String, String> {
        let mut partes = vec![self.verbo.clone()];
        let aspas = |v: &str| if v.contains(' ') { format!("\"{v}\"") } else { v.to_string() };
        let tem_marcas = self.campos.iter().any(|c| c.marca);
        for c in &self.campos {
            if c.marca {
                if c.valor == "1" {
                    partes.push(aspas(&c.rotulo));
                }
                continue;
            }
            let v = c.valor.trim();
            if v.is_empty() {
                return Err(format!("Fill in '{}'.", c.rotulo));
            }
            partes.push(aspas(v));
        }
        if tem_marcas && !self.campos.iter().any(|c| c.marca && c.valor == "1") {
            partes.push("NONE".into());
        }
        Ok(partes.join(" "))
    }
}

#[derive(Debug, Clone)]
pub enum CivilMsg {
    Nome(String),
    Descricao(String),
    Superficie(String),
    Estilo(String),
    Corrigir(bool),
    Emendar(bool),
    Folga(String),
    LimitarBorda(bool),
    Comprimento(String),
    TipoLimite(String),
    /// Valor do campo `n` do formulário.
    Campo(usize, String),
    Ok,
}

pub(crate) struct State {
    pub modo: Modo,
    pub superficies: Vec<String>,
    pub superficie: Option<String>,
    pub estilo: Option<String>,
    pub nome: String,
    pub descricao: String,
    pub corrigir: bool,
    pub emendar: bool,
    pub folga: String,
    pub limitar_borda: bool,
    pub comprimento: String,
    pub tipo_limite: Option<String>,
    pub erro: Option<String>,
}

impl State {
    pub fn novo(modo: Modo) -> Self {
        State {
            modo,
            superficies: Vec::new(),
            superficie: None,
            estilo: Some(crate::civil::estilos::ESTILOS[0].nome.to_string()),
            nome: String::new(),
            descricao: String::new(),
            corrigir: true,
            // Mesmo padrão do -ADDSURFACECONTOURS (JOIN=0): emendar inventa trecho de curva, então é escolha do engenheiro.
            emendar: false,
            folga: "5".into(),
            limitar_borda: false,
            comprimento: "0".into(),
            tipo_limite: Some("Outer".into()),
            erro: None,
        }
    }

    pub fn titulo(&self) -> String {
        match &self.modo {
            Modo::Criar => "Create Surface".into(),
            Modo::Curvas => "Add Contour Data".into(),
            Modo::Propriedades(n) => format!("Surface Properties - {n}"),
            Modo::Apagar(_) => "Delete Surface".into(),
            Modo::ApagarEixo(_) => "Delete Alignment".into(),
            Modo::Limite(n) => format!("Add Boundaries - {n}"),
            Modo::Elevar(n) => format!("Raise/Lower Surface - {n}"),
            Modo::Formulario(f) => f.titulo.clone(),
        }
    }
}

fn m(msg: CivilMsg) -> Message {
    Message::Civil(msg)
}

fn campo<'a>(rotulo: &str, valor: &str, f: fn(String) -> CivilMsg) -> Element<'a, Message> {
    row![
        text(rotulo.to_string()).size(12).width(Length::Fixed(150.0)),
        text_input("", valor).on_input(move |v| m(f(v))).on_submit(m(CivilMsg::Ok)).style(field_style).size(12).width(Fill),
    ]
    .spacing(8)
    .align_y(iced::Center)
    .into()
}

fn seletor_estilo(state: &State) -> Element<'_, Message> {
    row![
        text("Style").size(12).width(Length::Fixed(150.0)),
        pick_list(state.estilo.clone(), crate::civil::estilos::nomes(), |s: &String| s.clone())
            .on_select(|s| m(CivilMsg::Estilo(s)))
            .text_size(12)
            .width(Fill),
    ]
    .spacing(8)
    .align_y(iced::Center)
    .into()
}

pub(crate) fn view(state: &State, sizing: crate::ui::modal::ModalSizing) -> Element<'_, Message> {
    let mut corpo = column![].spacing(10);
    match &state.modo {
        Modo::Criar => {
            corpo = corpo
                .push(row![text("Type").size(12).width(Length::Fixed(150.0)), text("TIN surface").size(12)].spacing(8))
                .push(text("Information").size(13))
                .push(campo("Name", &state.nome, CivilMsg::Nome))
                .push(campo("Description", &state.descricao, CivilMsg::Descricao))
                .push(seletor_estilo(state));
        }
        Modo::Curvas => {
            corpo = corpo
                .push(
                    row![
                        text("Surface").size(12).width(Length::Fixed(150.0)),
                        pick_list(state.superficie.clone(), state.superficies.clone(), |s: &String| s.clone())
                            .on_select(|s| m(CivilMsg::Superficie(s)))
                            .text_size(12)
                            .width(Fill),
                    ]
                    .spacing(8)
                    .align_y(iced::Center),
                )
                .push(campo("Description", &state.descricao, CivilMsg::Descricao))
                .push(text("Agral contour cleanup (the polylines in the drawing are not changed)").size(13))
                .push(checkbox(state.corrigir).label("Fix crossing contours (cut the bulge, then midline)").on_toggle(|v| m(CivilMsg::Corrigir(v))).text_size(12))
                .push(
                    row![
                        checkbox(state.emendar).label("Join contour gaps up to (m)").on_toggle(|v| m(CivilMsg::Emendar(v))).text_size(12),
                        text_input("", &state.folga).on_input(|v| m(CivilMsg::Folga(v))).style(field_style).size(12).width(Length::Fixed(70.0)),
                    ]
                    .spacing(8)
                    .align_y(iced::Center),
                )
                .push(text("After OK, select the contour polylines in the drawing.").size(12));
        }
        Modo::Propriedades(_) => {
            corpo = corpo
                .push(text("Information").size(13))
                .push(campo("Name", &state.nome, CivilMsg::Nome))
                .push(campo("Description", &state.descricao, CivilMsg::Descricao))
                .push(seletor_estilo(state))
                .push(text("Definition - Build options").size(13))
                .push(campo("Maximum triangle length (m, 0 = none)", &state.comprimento, CivilMsg::Comprimento));
        }
        Modo::Limite(_) => {
            corpo = corpo
                .push(
                    row![
                        text("Type").size(12).width(Length::Fixed(150.0)),
                        pick_list(state.tipo_limite.clone(), vec!["Outer".to_string(), "Hide".to_string()], |s: &String| s.clone())
                            .on_select(|s| m(CivilMsg::TipoLimite(s)))
                            .text_size(12)
                            .width(Fill),
                    ]
                    .spacing(8)
                    .align_y(iced::Center),
                )
                .push(campo("Description", &state.descricao, CivilMsg::Descricao))
                .push(text("Outer: only what is inside stays. Hide: a hole in the surface (a lake, a building).").size(12))
                .push(text("After OK, select closed polylines in the drawing.").size(12));
        }
        Modo::Elevar(_) => {
            corpo = corpo
                .push(campo("Amount to add (m)", &state.comprimento, CivilMsg::Comprimento))
                .push(text("Positive raises the surface, negative lowers it. It goes into Definition > Edits.").size(12));
        }
        Modo::Formulario(f) => {
            for (k, c) in f.campos.iter().enumerate() {
                if c.marca {
                    corpo = corpo.push(
                        checkbox(c.valor == "1")
                            .label(c.rotulo.clone())
                            .on_toggle(move |v| m(CivilMsg::Campo(k, if v { "1" } else { "0" }.into())))
                            .text_size(12),
                    );
                    continue;
                }
                let entrada: Element<'_, Message> = if c.opcoes.is_empty() {
                    text_input("", &c.valor)
                        .on_input(move |v| m(CivilMsg::Campo(k, v)))
                        .on_submit(m(CivilMsg::Ok))
                        .style(field_style)
                        .size(12)
                        .width(Fill)
                        .into()
                } else {
                    let sel = c.opcoes.iter().find(|o| o.eq_ignore_ascii_case(&c.valor)).cloned();
                    pick_list(sel, c.opcoes.clone(), |s: &String| s.clone())
                        .on_select(move |v| m(CivilMsg::Campo(k, v)))
                        .text_size(12)
                        .width(Fill)
                        .into()
                };
                corpo = corpo.push(
                    row![text(c.rotulo.clone()).size(12).width(Length::Fixed(230.0)), entrada].spacing(8).align_y(iced::Center),
                );
            }
            if !f.nota.is_empty() {
                corpo = corpo.push(text(f.nota.clone()).size(12));
            }
            if let Some(p) = &f.ponto {
                corpo = corpo.push(text(format!("After OK: {}", p.trim_end_matches(':').to_lowercase())).size(12));
            }
        }
        Modo::ApagarEixo(n) => {
            corpo = corpo
                .push(text(format!("Delete alignment '{n}'?")).size(13))
                .push(text("Its profile view, sample lines and section views, corridor (and its surface lines) and volume table go too. The source polyline stays. Undo (U) brings it all back.").size(12));
        }
        Modo::Apagar(n) => {
            corpo = corpo
                .push(text(format!("Delete surface '{n}'?")).size(13))
                .push(text("The surface and its drawing are removed. The contour polylines stay in the drawing. Undo (U) brings it back.").size(12));
        }
    }
    if let Some(e) = &state.erro {
        corpo = corpo.push(text(e.as_str()).size(12).style(text::danger));
    }
    let ok = if matches!(state.modo, Modo::Apagar(_) | Modo::ApagarEixo(_)) { "Delete" } else { "OK" };
    let botoes = row![Space::new().width(Fill), dialog_button(ok.to_string(), m(CivilMsg::Ok), true), dialog_button("Cancel".to_string(), Message::CloseModal, false)].spacing(8);
    container(column![corpo, Space::new().height(Fill), botoes].spacing(10)).padding(16).width(sizing.width).into()
}
