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

/// Superfície plana z = 100 + 0,05·x num quadrado de 200 m, de arquivo de pontos.
fn plano(app: &mut OpenCADStudio, dir: &std::path::Path) {
    let arq = dir.join("plano.txt");
    let mut t = String::new();
    let mut n = 1;
    for x in (0..=200).step_by(20) {
        for y in (0..=200).step_by(20) {
            t.push_str(&format!("{n},{y},{x},{:.3},TN\n", 100.0 + 0.05 * x as f64));
            n += 1;
        }
    }
    std::fs::write(&arq, t).unwrap();
    command(app, "CREATESURFACE TN");
    command(app, &format!("ADDSURFACEPOINTFILE TN \"{}\" PNEZD", arq.display()));
}

fn textos_do_bloco(app: &OpenCADStudio, bloco: &str) -> Vec<String> {
    let doc = &app.tabs[app.active_tab].scene.document;
    let rec = doc.block_records.get(bloco).expect("bloco desenhado");
    rec.entity_handles.iter().filter_map(|h| doc.get_entity(*h)).filter_map(|e| match e {
        EntityType::Text(t) => Some(t.value.clone()),
        _ => None,
    }).collect()
}

#[test]
fn alinhamento_perfil_greide_e_secoes() {
    let dir = std::env::temp_dir().join(format!("agral_alin_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    let i = app.active_tab;
    plano(&mut app, &dir);
    // Eixo de (20,100) a (180,100): 160 m para leste.
    app.tabs[i].scene.ensure_layer("EIXO");
    let mut p = LwPolyline::new();
    p.vertices = vec![LwVertex::new(Vector2::new(20.0, 100.0)), LwVertex::new(Vector2::new(180.0, 100.0))];
    p.common.layer = "EIXO".into();
    app.tabs[i].scene.add_entities(vec![EntityType::LwPolyline(p)]);
    command(&mut app, "CREATEALIGNMENT EIXO1 EIXO");
    let a = app.civil_al_ler(i, "EIXO1").expect("alinhamento gravado");
    assert_eq!(a.superficie, "TN", "a única superfície vira a do alinhamento");
    let al = a.geometria().unwrap();
    assert!((al.comprimento() - 160.0).abs() < 1e-9);
    let rot = textos_do_bloco(&app, "AGRAL_ALINHAMENTO_EIXO1");
    for e in ["0+000.00", "0+020.00", "0+160.00"] {
        assert!(rot.iter().any(|t| t == e), "rótulo {e} em {rot:?}");
    }
    // Dica do mouse perto do eixo: estaca 80 (x = 100), 5 m à direita (sul, andando para leste).
    let dica = crate::app::civil_alinhamento::dica_do_eixo(&app.tabs[i].scene.document, [100.0, 95.0], 50.0).unwrap();
    assert!(dica.contains("Station: 0+080.00") && dica.contains("Offset: 5.000 R"), "{dica}");
    assert!(crate::app::civil_alinhamento::dica_do_eixo(&app.tabs[i].scene.document, [100.0, 300.0], 50.0).is_none());
    // Vista de perfil: faixa com a cota do terreno 101 + 0,05·estaca.
    command(&mut app, "CREATEPROFILEVIEW EIXO1 TN 10 0,-200");
    let faixa = textos_do_bloco(&app, "AGRAL_PERFIL_EIXO1");
    for (e, z) in [(0.0, 101.0), (80.0, 105.0), (160.0, 109.0)] {
        let _ = e;
        assert!(faixa.iter().any(|t| t == &format!("{z:.3}")), "cota {z:.3} na faixa");
    }
    // Mouse dentro da vista de perfil: estaca e cota do ponto, e a cota do terreno ali.
    let v = app.civil_al_ler(i, "EIXO1").unwrap().vista.unwrap();
    let p = [v.origem[0] + 80.0, v.origem[1] + (105.0 - v.base) * v.exagero];
    let dica = crate::app::civil_alinhamento::dica_da_vista(&app.tabs[i].scene.document, &app.tabs[i].civil_tins, p).expect("dica na vista");
    assert!(dica.contains("Station: 0+080.00") && dica.contains("Elevation: 105.000") && dica.contains("Terrain: 105.000"), "{dica}");
    assert!(crate::app::civil_alinhamento::dica_da_vista(&app.tabs[i].scene.document, &app.tabs[i].civil_tins, [v.origem[0] - 50.0, p[1]]).is_none());
    // Greide horizontal a 106 m: aterro na estaca 0 (+5) e corte na 160 (-3).
    command(&mut app, "CREATEPROFILE EIXO1 0,106 160,106");
    let faixa = textos_do_bloco(&app, "AGRAL_PERFIL_EIXO1");
    assert!(faixa.iter().any(|t| t == "+5.000"), "aterro de 5 m na estaca 0: {faixa:?}");
    assert!(faixa.iter().any(|t| t == "-3.000"), "corte de 3 m na estaca 160");
    assert_eq!(app.civil_al_ler(i, "EIXO1").unwrap().pivs.len(), 2);
    // Seções a cada 40 m, 30 m para cada lado.
    command(&mut app, "CREATESECTIONS EIXO1 40 30 30 3 400,0");
    let sec = textos_do_bloco(&app, "AGRAL_SECOES_EIXO1");
    assert!(sec.iter().any(|t| t == "0+080.00  TN 105.000"), "título da seção 80 com a cota do terreno: {sec:?}");
    assert!(sec.iter().any(|t| t == "106.000"), "cota do greide na seção");
    // A superfície muda (sobe 2 m): perfil e seções acompanham.
    command(&mut app, "_SURFACEEDIT TN EDIT_RAISELOWER 2");
    let faixa = textos_do_bloco(&app, "AGRAL_PERFIL_EIXO1");
    assert!(faixa.iter().any(|t| t == "103.000"), "perfil refeito com a superfície nova: {faixa:?}");
    // Desfazer e apagar.
    command(&mut app, "DELETEALIGNMENT EIXO1");
    assert!(app.civil_al_ler(i, "EIXO1").is_err());
    assert!(app.tabs[i].scene.document.block_records.get("AGRAL_PERFIL_EIXO1").is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn alinhamento_perfil_e_secoes_pela_linha_de_comando() {
    let dir = std::env::temp_dir().join(format!("agral_alin_lc_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    let i = app.active_tab;
    plano(&mut app, &dir);
    app.tabs[i].scene.ensure_layer("EIXO");
    let mut p = LwPolyline::new();
    p.vertices = vec![LwVertex::new(Vector2::new(20.0, 100.0)), LwVertex::new(Vector2::new(180.0, 100.0))];
    p.common.layer = "EIXO".into();
    app.tabs[i].scene.add_entities(vec![EntityType::LwPolyline(p)]);
    // Nome padrão (Enter) e a camada digitada no lugar de clicar.
    for t in ["CREATEALIGNMENT", "", "EIXO"] {
        digitar(&mut app, t);
    }
    assert!(app.civil_al_ler(i, "EIXO1").is_ok(), "eixo criado pela linha de comando");
    // Vista de perfil: alinhamento e superfície padrão, exagero 5, canto digitado.
    for t in ["CREATEPROFILEVIEW", "", "", "5", "0,-300"] {
        digitar(&mut app, t);
    }
    let a = app.civil_al_ler(i, "EIXO1").unwrap();
    assert_eq!(a.vista.as_ref().map(|v| v.exagero), Some(5.0));
    assert!(app.tabs[i].scene.document.block_records.get("AGRAL_PERFIL_EIXO1").is_some());
    // Greide pela linha de comando.
    for t in ["CREATEPROFILE", "0,106", "160,106,0", ""] {
        digitar(&mut app, t);
    }
    assert_eq!(app.civil_al_ler(i, "EIXO1").unwrap().pivs.len(), 2);
    // Seções com os padrões e o canto digitado.
    for t in ["CREATESECTIONS", "", "40", "", "", "", "400,0"] {
        digitar(&mut app, t);
    }
    let s = app.civil_al_ler(i, "EIXO1").unwrap().secoes.expect("seções");
    assert_eq!((s.intervalo, s.esquerda, s.direita, s.colunas), (40.0, 30.0, 30.0, 5));
    assert!(app.tabs[i].active_cmd.is_none(), "nenhum comando ficou pendurado");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn corredor_de_barragem_em_terreno_plano() {
    let dir = std::env::temp_dir().join(format!("agral_corr_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let arq = dir.join("plano100.txt");
    let mut t = String::new();
    let mut n = 1;
    for x in (0..=200).step_by(50) {
        for y in (0..=200).step_by(50) {
            t.push_str(&format!("{n},{y},{x},100.000,TN\n"));
            n += 1;
        }
    }
    std::fs::write(&arq, t).unwrap();
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    let i = app.active_tab;
    command(&mut app, "CREATESURFACE TN");
    command(&mut app, &format!("ADDSURFACEPOINTFILE TN \"{}\" PNEZD", arq.display()));
    app.tabs[i].scene.ensure_layer("EIXO");
    let mut p = LwPolyline::new();
    p.vertices = vec![LwVertex::new(Vector2::new(20.0, 100.0)), LwVertex::new(Vector2::new(180.0, 100.0))];
    p.common.layer = "EIXO".into();
    app.tabs[i].scene.add_entities(vec![EntityType::LwPolyline(p)]);
    command(&mut app, "CREATEALIGNMENT EIXO1 EIXO");
    command(&mut app, "CREATEPROFILEVIEW EIXO1 TN 10 0,-300");
    command(&mut app, "CREATEPROFILE EIXO1 0,110 160,110");
    // Crista 6 m sem declividade, montante 3:1 à esquerda, jusante 2:1, berma de 3 m a cada 8 m.
    command(&mut app, "CREATEASSEMBLY EIXO1 6 0 L 3 2 8 3 1");
    command(&mut app, "CREATECORRIDOR EIXO1 20");
    let a = app.civil_al_ler(i, "EIXO1").unwrap();
    let (va, vc) = a.volumes.expect("volumes do corredor");
    // Seção constante de 322 m² (crista 60 + montante 156 + jusante 106) em 160 m.
    let esperado = 322.0 * 160.0;
    assert!((va - esperado).abs() / esperado < 0.005, "aterro {va} x {esperado}");
    assert!(vc < 1.0, "corte {vc}");
    // Superfície do corredor: crista a 110 no eixo, saia a 100 a 36 m para montante (norte), nada fora da saia.
    let sup = crate::app::civil_alinhamento::superficie_do_corredor("EIXO1");
    let tin = app.civil_tin(i, &sup).expect("superfície do corredor montada");
    assert!((tin.cota_em(100.0, 100.0).unwrap() - 110.0).abs() < 1e-6);
    assert!((tin.cota_em(100.0, 101.0).unwrap() - 110.0).abs() < 1e-6, "crista plana");
    let z_talude = tin.cota_em(100.0, 100.0 + 3.0 + 12.0).unwrap();
    assert!((z_talude - 106.0).abs() < 1e-3, "no meio do talude de montante 3:1: {z_talude}");
    assert!(tin.cota_em(100.0, 100.0 + 40.0).is_none(), "fora da saia não tem superfície (sem dentes)");
    // Seções mostram o gabarito e a área.
    command(&mut app, "CREATESECTIONS EIXO1 40 50 50 3 400,0");
    let sec = textos_do_bloco(&app, "AGRAL_SECOES_EIXO1");
    assert!(sec.iter().any(|t| t.starts_with("Fill 322.0")), "área de aterro na seção: {sec:?}");
    // Mudou o greide: o corredor acompanha.
    command(&mut app, "CREATEPROFILE EIXO1 0,108 160,108");
    let (va2, _) = app.civil_al_ler(i, "EIXO1").unwrap().volumes.unwrap();
    assert!(va2 < va, "greide mais baixo, menos aterro");
    let linhas = app.civil_al_ler(i, "EIXO1").unwrap().desenho_corredor;
    command(&mut app, "DELETEALIGNMENT EIXO1");
    assert!(linhas.iter().all(|h| app.tabs[i].scene.document.get_entity(*h).is_none()), "apagar o eixo apaga as linhas do corredor");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn object_viewer_abre_a_superficie_em_3d_num_desenho_a_parte() {
    let mut app = OpenCADStudio::new_for_test();
    let _ = app.automation_op(r#"{"op":"new"}"#);
    let i = app.active_tab;
    app.tabs[i].scene.ensure_layer("CURVAS");
    app.tabs[i].scene.add_entities(vec![curva(100.0, 0.0), curva(101.0, 10.0), curva(102.0, 20.0)]);
    command(&mut app, "CREATESURFACE EG");
    command(&mut app, "ADDSURFACECONTOURS EG CURVAS FIX=0");
    let s = crate::civil::superficies(&app.tabs[i].scene.document).remove(0);
    let antes = app.tabs[i].scene.document.entities().count();
    app.tabs[i].scene.select_entity(s.desenho[0], true);
    let abas = app.tabs.len();
    command(&mut app, "OBJECTVIEWER 3");
    assert_eq!(app.tabs.len(), abas, "abre numa janela própria, não numa aba de desenho");
    assert_eq!(app.active_tab, i);
    assert!(app.show_object_viewer, "painel do viewer aberto");
    let cena = app.civil_cena.clone().expect("cena montada");
    assert!(!cena.triangulos.is_empty(), "a superfície vira triângulos");
    let zmax = cena.triangulos.iter().flat_map(|t| t.0.iter().map(|p| p[2])).fold(f32::MIN, f32::max);
    assert!((zmax - 106.0).abs() < 1e-3, "exagero 3x a partir de 100: 102 vira 106 ({zmax})");
    assert_eq!(app.tabs[i].scene.document.entities().count(), antes, "o desenho não muda");
    // A aba Home existe e é a primeira.
    let mods = crate::modules::registry::all_modules();
    assert_eq!(mods[0].id(), "home");
    let titulos: Vec<_> = mods[0].ribbon_groups().iter().map(|g| g.title).collect();
    for t in ["Palettes", "Create Ground Data", "Create Design", "Profile & Section Views", "Draw", "Modify", "Layers"] {
        assert!(titulos.contains(&t), "{t} em {titulos:?}");
    }
}
