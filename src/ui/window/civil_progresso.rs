//! Agral Civil: aviso de montagem de superfície em segundo plano, no canto de baixo à direita, sem travar a tela
//! (pedido do JP, 18/11: "bota uma barra de carregamento").

use crate::app::civil::{texto_da_fase, MontagemPendente};
use crate::app::Message;
use iced::time::Instant;
use iced::widget::{column, container, row, text, Space};
use iced::{Background, Border, Element, Fill, Length, Theme};
use std::sync::atomic::Ordering;

const LARGURA: f32 = 300.0;
const TRILHO: f32 = 268.0;
const BLOCO: f32 = 70.0;

pub fn view<'a>(m: &'a MontagemPendente, agora: Instant) -> Element<'a, Message> {
    let s = agora.saturating_duration_since(m.inicio).as_secs_f32();
    // Barra indeterminada: um bloco que vai e volta (a montagem não sabe quanto falta).
    let ida = (s * 0.8).fract();
    let pos = if (s * 0.8) as u32 % 2 == 0 { ida } else { 1.0 - ida } * (TRILHO - BLOCO);
    let barra = container(row![
        Space::new().width(Length::Fixed(pos)).height(Length::Fixed(5.0)),
        container(Space::new().width(Length::Fixed(BLOCO)).height(Length::Fixed(5.0))).style(|t: &Theme| {
            container::Style {
                background: Some(Background::Color(t.palette().primary.base.color)),
                border: Border { radius: 2.5.into(), ..Default::default() },
                ..Default::default()
            }
        }),
    ])
    .width(Length::Fixed(TRILHO))
    .style(|t: &Theme| container::Style {
        background: Some(Background::Color(t.palette().background.strong.color)),
        border: Border { radius: 2.5.into(), ..Default::default() },
        ..Default::default()
    });
    let fase = texto_da_fase(m.fase.load(Ordering::Relaxed));
    let cartao = container(
        column![
            text(format!("Building surface '{}'", m.definicao.nome)).size(13),
            barra,
            text(format!("{fase}... {s:.0} s")).size(11).style(|t: &Theme| iced::widget::text::Style {
                color: Some(crate::ui::style::readable::muted_text(t)),
            }),
        ]
        .spacing(8),
    )
    .padding([12, 16])
    .width(Length::Fixed(LARGURA))
    .style(|t: &Theme| {
        let p = t.palette();
        container::Style {
            background: Some(Background::Color(p.background.weak.color)),
            border: Border { color: p.primary.base.color, width: 1.0, radius: 6.0.into() },
            ..Default::default()
        }
    });
    container(cartao).align_right(Fill).align_bottom(Fill).padding([70, 24]).into()
}

/// Download da imagem de satélite: barra com a fração de tiles prontos (spec 005 R5).
pub fn view_imagem<'a>(b: &'a crate::app::civil_geo::Baixando, agora: Instant) -> Element<'a, Message> {
    let s = agora.saturating_duration_since(b.inicio).as_secs_f32();
    let feitos = b.feitos.load(Ordering::Relaxed).min(b.total);
    let frac = if b.total == 0 { 1.0 } else { feitos as f32 / b.total as f32 };
    let barra = container(
        container(Space::new().width(Length::Fixed((TRILHO * frac).max(4.0))).height(Length::Fixed(5.0))).style(|t: &Theme| {
            container::Style {
                background: Some(Background::Color(t.palette().primary.base.color)),
                border: Border { radius: 2.5.into(), ..Default::default() },
                ..Default::default()
            }
        }),
    )
    .width(Length::Fixed(TRILHO))
    .style(|t: &Theme| container::Style {
        background: Some(Background::Color(t.palette().background.strong.color)),
        border: Border { radius: 2.5.into(), ..Default::default() },
        ..Default::default()
    });
    let etapa = if feitos < b.total { format!("{feitos} of {} tiles", b.total) } else { "Reprojecting to UTM".to_string() };
    let cartao = container(
        column![
            text("Downloading satellite imagery").size(13),
            barra,
            text(format!("{etapa}... {s:.0} s")).size(11).style(|t: &Theme| iced::widget::text::Style {
                color: Some(crate::ui::style::readable::muted_text(t)),
            }),
        ]
        .spacing(8),
    )
    .padding([12, 16])
    .width(Length::Fixed(LARGURA))
    .style(|t: &Theme| {
        let p = t.palette();
        container::Style {
            background: Some(Background::Color(p.background.weak.color)),
            border: Border { color: p.primary.base.color, width: 1.0, radius: 6.0.into() },
            ..Default::default()
        }
    });
    container(cartao).align_right(Fill).align_bottom(Fill).padding([70, 24]).into()
}
