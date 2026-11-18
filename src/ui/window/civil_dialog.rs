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
            emendar: true,
            folga: "5".into(),
            limitar_borda: false,
            comprimento: "0".into(),
            erro: None,
        }
    }

    pub fn titulo(&self) -> String {
        match &self.modo {
            Modo::Criar => "Create Surface".into(),
            Modo::Curvas => "Add Contour Data".into(),
            Modo::Propriedades(n) => format!("Surface Properties - {n}"),
            Modo::Apagar(_) => "Delete Surface".into(),
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
        Modo::Apagar(n) => {
            corpo = corpo
                .push(text(format!("Delete surface '{n}'?")).size(13))
                .push(text("The surface and its drawing are removed. The contour polylines stay in the drawing. Undo (U) brings it back.").size(12));
        }
    }
    if let Some(e) = &state.erro {
        corpo = corpo.push(text(e.as_str()).size(12).style(text::danger));
    }
    let ok = if matches!(state.modo, Modo::Apagar(_)) { "Delete" } else { "OK" };
    let botoes = row![Space::new().width(Fill), dialog_button(ok.to_string(), m(CivilMsg::Ok), true), dialog_button("Cancel".to_string(), Message::CloseModal, false)].spacing(8);
    container(column![corpo, Space::new().height(Fill), botoes].spacing(10)).padding(16).width(sizing.width).into()
}
