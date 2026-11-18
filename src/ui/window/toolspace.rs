//! Toolspace (Prospector) do Agral CAD: a árvore das superfícies do desenho, como no Civil 3D.
//! Surfaces > superfície > Definition > Contours (e os outros tipos de dado), com menu no botão direito.
//! Cada item do menu roda um comando digitável, então tudo também funciona pela linha de comando.

use crate::app::Message;
use crate::civil::definicao::SuperficieDwg;
use crate::ui::dock::PanelId;
use crate::ui::kit;
use crate::ui::kit::menu::Item;
use iced::widget::{column, text};
use iced::Element;
use std::collections::HashSet;

/// Tipos de dado da definição, na ordem do Civil 3D. Só Contours já está pronto; os outros vêm nas próximas entregas.
const TIPOS: &[(&str, &str)] = &[
    ("Boundaries", ""),
    ("Breaklines", ""),
    ("Contours", "CONTOURS"),
    ("DEM Files", ""),
    ("Drawing Objects", ""),
    ("Edits", ""),
    ("Point Files", ""),
    ("Point Clouds", ""),
];

fn cmd(c: String) -> Message {
    Message::Command(c)
}

fn menu(itens: Vec<(String, String)>) -> Element<'static, Message> {
    kit::menu::view(itens.into_iter().map(|(rotulo, c)| Item::new(rotulo, cmd(c))).collect())
}

fn com_menu<'a>(linha: Element<'a, Message>, itens: Vec<(String, String)>) -> Element<'a, Message> {
    iced_aw::ContextMenu::new(linha, move || menu(itens.clone())).into()
}

/// Uma linha que dobra: chave única no painel, nível, e se está aberta.
fn dobra<'a>(rotulo: String, chave: &str, nivel: u16, fechados: &HashSet<String>) -> (kit::TreeRow<'a>, bool) {
    let aberta = !fechados.contains(chave);
    (kit::tree_row(rotulo).depth(nivel).fold(aberta, Message::ToolspaceFold(chave.to_string())), aberta)
}

pub fn view<'a>(
    nome_desenho: &str,
    superficies: Vec<SuperficieDwg>,
    fechados: &HashSet<String>,
    width: f32,
    chrome: crate::ui::dock::Chrome,
) -> Element<'a, Message> {
    let title_bar = crate::ui::dock::title_bar(PanelId::Toolspace, "Toolspace - Prospector".into(), chrome);
    let mut arvore = column![];

    let (linha, aberto) = dobra(nome_desenho.to_string(), "drawing", 0, fechados);
    arvore = arvore.push(linha.view());
    if aberto {
        let (linha, abertas) = dobra(format!("Surfaces ({})", superficies.len()), "surfaces", 1, fechados);
        arvore = arvore.push(com_menu(linha.view(), vec![("Create Surface...".into(), "CREATESURFACE".into())]));
        if abertas {
            if superficies.is_empty() {
                arvore = arvore.push(
                    kit::tree_row("No surfaces. Right-click Surfaces > Create Surface...").depth(2).caret_room().muted(true).view(),
                );
            }
            for s in superficies {
                arvore = arvore.push(linhas_da_superficie(s, fechados));
            }
        }
    }
    let rodape = text("Right-click an item for its commands.").size(11);
    kit::palette(title_bar, kit::scroll_area(column![arvore, rodape].spacing(8)), width)
}

fn linhas_da_superficie<'a>(s: SuperficieDwg, fechados: &HashSet<String>) -> Element<'a, Message> {
    let n = s.nome.clone();
    let mut col = column![];
    let chave = format!("s:{n}");
    let rotulo = if s.desatualizada { format!("{n}  (out of date)") } else { n.clone() };
    let (linha, aberta) = dobra(rotulo, &chave, 2, fechados);
    let linha = linha.on_double_click(cmd(format!("SURFACEPROPERTIES {n}")));
    col = col.push(com_menu(
        linha.view(),
        vec![
            ("Surface Properties...".into(), format!("SURFACEPROPERTIES {n}")),
            ("Rebuild".into(), format!("REBUILDSURFACE {n}")),
            ("Zoom to".into(), format!("ZOOMSURFACE {n}")),
            ("Delete...".into(), format!("DELETESURFACE {n}")),
        ],
    ));
    if !aberta {
        return col.into();
    }
    let chave_def = format!("d:{n}");
    let (linha, def_aberta) = dobra("Definition".into(), &chave_def, 3, fechados);
    col = col.push(linha.view());
    if !def_aberta {
        return col.into();
    }
    for (rotulo, tipo) in TIPOS {
        if tipo.is_empty() {
            col = col.push(kit::tree_row(*rotulo).depth(4).caret_room().muted(true).view());
            continue;
        }
        let itens: Vec<_> = s.itens.iter().enumerate().filter(|(_, it)| it.tipo == *tipo).collect();
        let chave_tipo = format!("t:{n}:{tipo}");
        let (linha, tipo_aberto) = dobra(format!("{rotulo} ({})", itens.len()), &chave_tipo, 4, fechados);
        col = col.push(com_menu(linha.view(), vec![("Add...".into(), format!("ADDSURFACECONTOURS {n}"))]));
        if tipo_aberto {
            for (k, it) in itens {
                let nome_item = if it.descricao.is_empty() { format!("Contours {}", k + 1) } else { it.descricao.clone() };
                let mut detalhes = format!("{nome_item}  ({} objects", it.fontes.len());
                if it.corrigir {
                    detalhes.push_str(", fix crossings");
                }
                if it.emendar > 0.0 {
                    detalhes.push_str(&format!(", join gaps {} m", it.emendar));
                }
                detalhes.push(')');
                if !it.ligado {
                    detalhes.push_str("  [off]");
                }
                let alternar = if it.ligado { ("Turn Off".to_string(), "OFF") } else { ("Turn On".to_string(), "ON") };
                col = col.push(com_menu(
                    kit::tree_row(detalhes).depth(5).caret_room().muted(!it.ligado).view(),
                    vec![
                        (alternar.0, format!("SURFACEDEFINITION {n} {k} {}", alternar.1)),
                        ("Remove from Definition".into(), format!("SURFACEDEFINITION {n} {k} REMOVE")),
                    ],
                ));
            }
        }
    }
    col.into()
}
