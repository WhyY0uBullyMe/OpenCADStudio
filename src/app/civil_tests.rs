//! Agral Civil: cota da superfície no mouse (pedido do JP, 18/11). A superfície montada guarda o TIN na
//! memória; ao reabrir o desenho o TIN é montado de novo em segundo plano.

use super::*;
use document::entities::{LwPolyline, LwVertex};
use document::types::Vector2;
use document::EntityType;

fn curva(z: f64, y: f64) -> EntityType {
    let mut p = LwPolyline::new();
    p.vertices = (0..=10).map(|k| LwVertex::new(Vector2::new(k as f64 * 10.0, y))).collect();
    p.elevation = z;
    p.common.layer = "CURVAS".into();
    EntityType::LwPolyline(p)
}

fn command(app: &mut OpenCADStudio, cmd: &str) {
    let task = app.update(Message::Command(cmd.to_string()));
    app.drive_headless_task(task).expect("the command runs headless");
}

fn cota(app: &OpenCADStudio, x: f64, y: f64) -> Option<(String, f64)> {
    let tab = &app.tabs[app.active_tab];
    crate::app::civil::cota_no_ponto(&tab.civil_tins, &tab.scene.document, x, y)
}

#[test]
fn cota_no_mouse_depois_de_montar_e_de_reabrir() {
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    let i = app.active_tab;
    app.tabs[i].scene.ensure_layer("CURVAS");
    // Plano: z = 100 + y/10 (curvas 100, 101 e 102 a cada 10 m).
    app.tabs[i].scene.add_entities(vec![curva(100.0, 0.0), curva(101.0, 10.0), curva(102.0, 20.0)]);
    command(&mut app, "CREATESURFACE EG");
    command(&mut app, "ADDSURFACECONTOURS EG CURVAS FIX=0");
    let (nome, z) = cota(&app, 45.0, 15.0).expect("cota dentro da superfície");
    assert_eq!(nome, "EG");
    assert!((z - 101.5).abs() < 1e-6, "z = {z}");
    assert!(cota(&app, 45.0, 50.0).is_none(), "fora da superfície não mostra cota");

    // A etiqueta só aparece com o mouse parado (2 s); mexer o mouse esconde.
    let _ = app.civil_cota_mouse_andou();
    assert!(app.civil_cota_relogio, "mouse andou sobre desenho com superfície arma o relógio");
    let _ = app.civil_cota_relogio();
    assert!(!app.civil_cota_visivel, "ainda não deu o tempo");
    app.civil_cota_mov -= crate::app::civil::ESPERA_COTA;
    let _ = app.civil_cota_relogio();
    assert!(app.civil_cota_visivel && !app.civil_cota_relogio);
    let _ = app.civil_cota_mouse_andou();
    assert!(!app.civil_cota_visivel, "mexeu, escondeu");

    // Properties da superfície selecionada: "TIN Surface" com o estilo editável, não "Block Reference".
    let s = crate::civil::superficies(&app.tabs[i].scene.document).remove(0);
    app.tabs[i].scene.select_entity(s.desenho[0], true);
    app.refresh_properties();
    let painel = &app.tabs[i].properties;
    assert_eq!(painel.title, "TIN Surface");
    let campos: Vec<&str> = painel.sections.iter().flat_map(|x| x.props.iter().map(|p| p.label.as_str())).collect();
    for c in ["Name", "Style", "Number of triangles", "Minimum elevation"] {
        assert!(campos.contains(&c), "falta {c} em {campos:?}");
    }
    let task = app.update(Message::PropGeomChoiceChanged { field: "civil_surface_style", value: "Curvas 0.5m".into() });
    app.drive_headless_task(task).expect("estilo trocado");
    let s = crate::civil::superficies(&app.tabs[i].scene.document).remove(0);
    assert_eq!(s.estilo, "Curvas 0.5m");
    assert!(!s.desatualizada, "trocar o estilo redesenha a superfície");
    app.tabs[i].scene.deselect_all();

    // Como ao reabrir o desenho: sem TIN na memória; o mouse andando manda montar em segundo plano.
    app.tabs[i].civil_tins.clear();
    app.tabs[i].civil_tins_conferido = None;
    assert!(cota(&app, 45.0, 15.0).is_none());
    let task = app.civil_preparar_cota();
    app.drive_headless_task(task).expect("montagem em segundo plano");
    let (_, z) = cota(&app, 45.0, 15.0).expect("cota depois da remontagem");
    assert!((z - 101.5).abs() < 1e-6, "z = {z}");

    // Desfazer a montagem: o desenho volta ao de antes e o TIN guardado deixa de valer.
    command(&mut app, "U");
    assert!(cota(&app, 45.0, 15.0).is_none(), "TIN velho não pode dar cota");
}

#[test]
fn divgrips_e_spl2pl_refazem_a_polilinha() {
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    let i = app.active_tab;
    app.tabs[i].scene.ensure_layer("CURVAS");
    let h = app.tabs[i].scene.add_entities(vec![curva(100.0, 0.0)])[0];
    // Curva de 100 m; vértice a cada 30 m: 0, 30, 60, 90 e 100. Original fica.
    command(&mut app, &format!("_REFAZERCURVA DIVGRIPS {:X} 30 N", h.value()));
    let polis: Vec<_> = app.tabs[i].scene.document.entities().filter(|e| matches!(e, EntityType::LwPolyline(_))).cloned().collect();
    assert_eq!(polis.len(), 2);
    let EntityType::LwPolyline(nova) = &polis[1] else { unreachable!() };
    let xs: Vec<f64> = nova.vertices.iter().map(|v| (v.location.x * 1e6).round() / 1e6).collect();
    assert_eq!(xs, vec![0.0, 30.0, 60.0, 90.0, 100.0]);
    assert_eq!(nova.elevation, 100.0);
    assert_eq!(nova.common.layer, "CURVAS");
    // SPL2PL com 5 vértices apagando a original.
    command(&mut app, &format!("_REFAZERCURVA SPL2PL {:X} 5 S", h.value()));
    assert!(app.tabs[i].scene.document.get_entity(h).is_none(), "original apagada");
    let ultima = app.tabs[i].scene.document.entities().filter_map(|e| match e { EntityType::LwPolyline(p) => Some(p.vertices.len()), _ => None }).last();
    assert_eq!(ultima, Some(5));
    // Atalhos do JP na tabela de fábrica.
    let pgp = include_str!("../../assets/ocad.pgp");
    for a in ["PPP,      *PASTEORIG", "SZ,       *SELECTSIMILAR", "HH,       *HIDEOBJECTS"] {
        assert!(pgp.contains(a), "{a}");
    }
}

#[test]
fn montagem_em_segundo_plano_desenha_ao_terminar() {
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    app.civil_assincrono = true;
    let i = app.active_tab;
    app.tabs[i].scene.ensure_layer("CURVAS");
    app.tabs[i].scene.add_entities(vec![curva(100.0, 0.0), curva(101.0, 10.0), curva(102.0, 20.0)]);
    command(&mut app, "CREATESURFACE EG");
    // O comando volta na hora; a montagem roda fora e desenha quando termina (o teste roda as tarefas até o fim).
    command(&mut app, "ADDSURFACECONTOURS EG CURVAS FIX=0");
    assert!(app.civil_montando.is_none(), "terminou");
    let s = crate::civil::superficies(&app.tabs[i].scene.document).remove(0);
    assert!(!s.desatualizada && !s.desenho.is_empty(), "desenhada depois da montagem");
    assert!(cota(&app, 45.0, 15.0).is_some());
}
