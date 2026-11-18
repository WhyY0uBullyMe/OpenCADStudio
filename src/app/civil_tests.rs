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

/// Como quem digita na linha de comando: o texto e o Enter.
fn digitar(app: &mut OpenCADStudio, t: &str) {
    let _ = app.update(Message::CommandInput(t.to_string()));
    let task = app.update(Message::CommandSubmit);
    app.drive_headless_task(task).expect("roda");
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

#[test]
fn versoes_de_linha_de_comando_perguntam_cada_valor() {
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    let i = app.active_tab;
    app.tabs[i].scene.ensure_layer("CURVA MESTRA-AS");
    let mut c = vec![curva(100.0, 0.0), curva(101.0, 10.0), curva(102.0, 20.0)];
    c.iter_mut().for_each(|e| e.common_mut().layer = "CURVA MESTRA-AS".into());
    app.tabs[i].scene.add_entities(c);
    // Como o MCP e a linha de comando mandam: o comando e depois cada resposta.
    for t in ["-CREATESURFACE", "TN"] {
        digitar(&mut app, t);
    }
    assert!(app.active_modal.is_none(), "não abre janela");
    for t in ["-ADDSURFACECONTOURS", "TN", "CURVA MESTRA-AS", "FIX=0", ""] {
        digitar(&mut app, t);
    }
    let s = crate::civil::superficies(&app.tabs[i].scene.document).remove(0);
    assert_eq!(s.nome, "TN");
    assert_eq!(s.itens.len(), 1);
    assert!(!s.itens[0].corrigir);
    assert_eq!(s.itens[0].fontes.len(), 3);
    for t in ["-SURFACEPROPERTIES", "TN", "Curvas 0.5m"] {
        digitar(&mut app, t);
    }
    assert_eq!(crate::civil::superficies(&app.tabs[i].scene.document)[0].estilo, "Curvas 0.5m");
}

#[test]
fn linha_de_quebra_manda_na_cota() {
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    let i = app.active_tab;
    app.tabs[i].scene.ensure_layer("CURVAS");
    app.tabs[i].scene.ensure_layer("QUEBRAS");
    app.tabs[i].scene.add_entities(vec![curva(100.0, 0.0), curva(101.0, 10.0), curva(102.0, 20.0)]);
    command(&mut app, "CREATESURFACE EG");
    command(&mut app, "ADDSURFACECONTOURS EG CURVAS FIX=0");
    let (_, z) = cota(&app, 50.0, 5.0).unwrap();
    assert!((z - 100.5).abs() < 1e-6, "sem a quebra o plano dá 100,5: {z}");
    // Crista a 103 m entre as curvas 100 e 101.
    let mut q = document::entities::Polyline3D::from_points(vec![
        document::types::Vector3::new(30.0, 5.0, 103.0),
        document::types::Vector3::new(70.0, 5.0, 103.0),
    ]);
    q.common.layer = "QUEBRAS".into();
    app.tabs[i].scene.add_entities(vec![EntityType::Polyline3D(q)]);
    command(&mut app, "ADDSURFACEBREAKLINES EG QUEBRAS");
    let s = crate::civil::superficies(&app.tabs[i].scene.document).remove(0);
    assert!(s.itens.iter().any(|it| it.tipo == "BREAKLINES" && it.fontes.len() == 1));
    let (_, z) = cota(&app, 50.0, 5.0).expect("cota sobre a quebra");
    assert!((z - 103.0).abs() < 1e-6, "sobre a crista a cota é a da quebra: {z}");
    let (_, z) = cota(&app, 40.0, 5.0).unwrap();
    assert!((z - 103.0).abs() < 1e-6, "ao longo da crista também: {z}");
}

#[test]
fn arquivo_de_pontos_entra_na_superficie_e_avisa_linhas_ruins() {
    let dir = std::env::temp_dir().join(format!("agral_pontos_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let arq = dir.join("levantamento.txt");
    // PNEZD: número, norte, este, cota, descrição. Plano z = 200 + x/10 num quadrado de 100 m; uma linha ruim.
    let mut t = String::new();
    let mut n = 1;
    for x in (0..=100).step_by(10) {
        for y in (0..=100).step_by(10) {
            t.push_str(&format!("{n},{y},{x},{:.3},TN\n", 200.0 + x as f64 / 10.0));
            n += 1;
        }
    }
    t.push_str("999,abc,10,200,RUIM\n");
    std::fs::write(&arq, t).unwrap();
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    command(&mut app, "CREATESURFACE PT");
    command(&mut app, &format!("ADDSURFACEPOINTFILE PT \"{}\" PNEZD", arq.display()));
    let s = crate::civil::superficies(&app.tabs[app.active_tab].scene.document).remove(0);
    assert!(s.itens.iter().any(|it| it.tipo == "POINTFILE_PNEZD"));
    let (_, z) = cota(&app, 55.0, 33.0).expect("cota dentro do levantamento");
    assert!((z - 205.5).abs() < 1e-6, "z = {z}");
    let avisou = app.command_line.history.iter().map(|h| h.text.clone()).any(|l| l.contains("1 lines of") && l.contains("line 122"));
    assert!(avisou, "a linha ruim aparece no aviso com o número");
    // Arquivo sumido: avisa e não apaga a superfície.
    std::fs::remove_file(&arq).unwrap();
    command(&mut app, "REBUILDSURFACE PT");
    assert!(app.command_line.history.iter().map(|h| h.text.clone()).any(|l| l.contains("could not be read")));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn landxml_ida_e_volta_mantem_pontos_triangulos_e_cotas() {
    let dir = std::env::temp_dir().join(format!("agral_landxml_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let arq = dir.join("tn.xml");
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    let i = app.active_tab;
    app.tabs[i].scene.ensure_layer("CURVAS");
    app.tabs[i].scene.add_entities(vec![curva(100.0, 0.0), curva(101.3, 10.0), curva(102.1, 20.0), curva(104.0, 35.0)]);
    command(&mut app, "CREATESURFACE TN");
    command(&mut app, "ADDSURFACECONTOURS TN CURVAS FIX=0");
    command(&mut app, &format!("EXPORTLANDXMLSURFACE TN \"{}\"", arq.display()));
    assert!(arq.is_file(), "arquivo gravado");
    command(&mut app, &format!("IMPORTLANDXMLSURFACE \"{}\"", arq.display()));
    let nomes = app.civil_superficies(i);
    assert!(nomes.iter().any(|n| n == "TN(2)"), "importada com nome livre: {nomes:?}");
    let (_, a) = app.tabs[i].civil_tins.get("TN").unwrap().clone();
    let (_, b) = app.tabs[i].civil_tins.get("TN(2)").unwrap().clone();
    assert_eq!(a.vertices.len(), b.vertices.len());
    assert_eq!(a.triangulos.len(), b.triangulos.len(), "mesma triangulação, sem triangular de novo");
    for (x, y) in [(5.0, 3.0), (33.0, 12.0), (50.0, 17.5), (77.0, 25.0), (90.0, 30.0), (12.0, 33.0), (61.0, 8.0), (45.0, 22.0), (20.0, 15.0), (85.0, 2.0)] {
        let (za, zb) = (a.cota_em(x, y), b.cota_em(x, y));
        assert_eq!(za.is_some(), zb.is_some(), "({x},{y})");
        if let (Some(za), Some(zb)) = (za, zb) {
            assert!((za - zb).abs() < 1e-9, "({x},{y}): {za} x {zb}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
