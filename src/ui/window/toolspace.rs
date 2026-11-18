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

/// Ícones próprios do Agral CAD (16 px, `assets/agral/toolspace/`), um para cada tipo da árvore.
macro_rules! ic {
    ($n:literal) => {
        include_bytes!(concat!("../../../assets/agral/toolspace/", $n, ".svg"))
    };
}
pub const IC_DESENHO: &[u8] = ic!("desenho");
pub const IC_PASTA: &[u8] = ic!("pasta");
pub const IC_SUPERFICIE: &[u8] = ic!("superficie");
pub const IC_DEFINICAO: &[u8] = ic!("definicao");

fn icone<'a>(bytes: &'static [u8], apagado: bool) -> Element<'a, Message> {
    if apagado {
        crate::ui::icons::semantic_disabled(bytes, 16.0)
    } else {
        crate::ui::icons::semantic(bytes, 16.0)
    }
}

fn icone_do_item(tipo: &str) -> &'static [u8] {
    match tipo {
        "CONTOURS" => ic!("curva_item"),
        "BOUNDARY_OUTER" => ic!("limite_externo"),
        "BOUNDARY_HIDE" => ic!("limite_furo"),
        "EDIT_ADDPOINT" => ic!("edit_ponto_mais"),
        "EDIT_DELETEPOINT" => ic!("edit_ponto_menos"),
        "EDIT_DELETELINE" => ic!("edit_linha_menos"),
        "EDIT_RAISELOWER" => ic!("edit_elevar"),
        "BREAKLINES" => ic!("quebras"),
        t if t.starts_with("POINTFILE") => ic!("arquivo_pontos"),
        _ => ic!("objetos"),
    }
}

/// Ramos da definição, na ordem do Civil 3D: rótulo, prefixo do tipo gravado (vazio = ainda não existe) e os itens
/// do menu do botão direito (rótulo, comando sem o nome da superfície).
const RAMOS: &[(&str, &str, &[u8], &[(&str, &str)])] = &[
    ("Boundaries", "BOUNDARY", ic!("limites"), &[("Add...", "ADDSURFACEBOUNDARIES")]),
    ("Breaklines", "BREAKLINES", ic!("quebras"), &[("Add...", "ADDSURFACEBREAKLINES")]),
    ("Contours", "CONTOURS", ic!("curvas"), &[("Add...", "ADDSURFACECONTOURS")]),
    ("DEM Files", "", ic!("dem"), &[]),
    ("Drawing Objects", "", ic!("objetos"), &[]),
    (
        "Edits",
        "EDIT_",
        ic!("edicoes"),
        &[
            ("Add Point", "ADDSURFACEPOINT"),
            ("Delete Point", "DELETESURFACEPOINT"),
            ("Delete Line", "DELETESURFACELINE"),
            ("Raise/Lower Surface...", "RAISELOWERSURFACE"),
        ],
    ),
    ("Point Files", "POINTFILE", ic!("arquivo_pontos"), &[("Add...", "ADDSURFACEPOINTFILE")]),
    ("Point Clouds", "", ic!("nuvem"), &[]),
];

fn rotulo_item(it: &crate::civil::definicao::ItemDwg, k: usize) -> String {
    let base = match it.tipo.as_str() {
        "CONTOURS" => format!("Contours {}", k + 1),
        "BREAKLINES" => format!("Breaklines {}", k + 1),
        t if t.starts_with("POINTFILE_") => format!("Point File {}", &t[10..]),
        "BOUNDARY_OUTER" => "Outer".into(),
        "BOUNDARY_HIDE" => "Hide".into(),
        "EDIT_ADDPOINT" => "Add Point".into(),
        "EDIT_DELETEPOINT" => "Delete Point".into(),
        "EDIT_DELETELINE" => "Delete Line".into(),
        "EDIT_RAISELOWER" => format!("Raise/Lower {:+.3} m", it.valor),
        t => t.into(),
    };
    let mut r = if it.descricao.is_empty() { base } else { format!("{base}: {}", it.descricao) };
    if !it.fontes.is_empty() {
        r.push_str(&format!("  ({} objects", it.fontes.len()));
        if it.corrigir {
            r.push_str(", fix crossings");
        }
        if it.emendar > 0.0 {
            r.push_str(&format!(", join gaps {} m", it.emendar));
        }
        r.push(')');
    } else if let Some(p) = it.pontos.first() {
        r.push_str(&format!("  ({:.2}, {:.2}{})", p[0], p[1], if it.tipo == "EDIT_ADDPOINT" { format!(", {:.2}", p[2]) } else { String::new() }));
    }
    if !it.ligado {
        r.push_str("  [off]");
    }
    r
}

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
    arvore = arvore.push(linha.icon(icone(IC_DESENHO, false)).view());
    if aberto {
        let (linha, abertas) = dobra(format!("Surfaces ({})", superficies.len()), "surfaces", 1, fechados);
        arvore = arvore.push(com_menu(linha.icon(icone(IC_PASTA, false)).view(), vec![("Create Surface...".into(), "CREATESURFACE".into())]));
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
    let linha = linha.icon(icone(IC_SUPERFICIE, false)).on_double_click(cmd(format!("SURFACEPROPERTIES {n}")));
    col = col.push(com_menu(
        linha.view(),
        vec![
            ("Surface Properties...".into(), format!("SURFACEPROPERTIES {n}")),
            ("Rebuild".into(), format!("REBUILDSURFACE {n}")),
            ("Extract Objects".into(), format!("EXTRACTSURFACEOBJECTS {n}")),
            ("Zoom to".into(), format!("ZOOMSURFACE {n}")),
            ("Delete...".into(), format!("DELETESURFACE {n}")),
        ],
    ));
    if !aberta {
        return col.into();
    }
    let chave_def = format!("d:{n}");
    let (linha, def_aberta) = dobra("Definition".into(), &chave_def, 3, fechados);
    col = col.push(linha.icon(icone(IC_DEFINICAO, false)).view());
    if !def_aberta {
        return col.into();
    }
    for (rotulo, prefixo, ic, menu) in RAMOS {
        if prefixo.is_empty() {
            col = col.push(kit::tree_row(*rotulo).depth(4).caret_room().icon(icone(ic, true)).muted(true).view());
            continue;
        }
        let itens: Vec<_> = s.itens.iter().enumerate().filter(|(_, it)| it.tipo.starts_with(prefixo)).collect();
        let chave_tipo = format!("t:{n}:{prefixo}");
        let (linha, tipo_aberto) = dobra(format!("{rotulo} ({})", itens.len()), &chave_tipo, 4, fechados);
        col = col.push(com_menu(linha.icon(icone(ic, false)).view(), menu.iter().map(|(r, c)| (r.to_string(), format!("{c} {n}"))).collect()));
        if tipo_aberto {
            for (k, it) in itens {
                let alternar = if it.ligado { ("Turn Off".to_string(), "OFF") } else { ("Turn On".to_string(), "ON") };
                col = col.push(com_menu(
                    kit::tree_row(rotulo_item(it, k))
                        .depth(5)
                        .caret_room()
                        .icon(icone(icone_do_item(&it.tipo), !it.ligado))
                        .muted(!it.ligado)
                        .view(),
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
