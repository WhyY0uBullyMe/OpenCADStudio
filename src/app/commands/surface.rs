//! Comandos de superfície do Agral Civil, com os nomes do Civil 3D (pedido do JP, 17/11).
//!
//! Na tela: `CREATESURFACE` abre a janela Create Surface; `ADDSURFACECONTOURS` abre Add Contour Data e depois pede
//! "Select contours:"; `TOOLSPACE` abre o painel com a árvore. Em script, os mesmos comandos aceitam argumentos:
//! `CREATESURFACE TN Terreno natural`, `ADDSURFACECONTOURS TN CONTOURS` (camada; opções FIX=0 e JOIN=<m>).

use crate::app::{Message, OpenCADStudio};
use crate::civil::curvas;
use crate::civil::definicao::ItemDwg;
use crate::ui::window::civil_dialog::{CivilMsg, Modo, State};
use iced::Task;

/// Comandos internos que a seleção ou o clique na tela disparam de volta (o usuário não digita).
const APLICAR_CURVAS: &str = "_ADDCONTOURDATA";
const APLICAR_LIMITE: &str = "_ADDBOUNDARYDATA";
const APLICAR_EDIT: &str = "_SURFACEEDIT";

fn e_polilinha_fechada(e: &document::EntityType) -> bool {
    curvas::pontos(e).is_some_and(|p| p.len() > 3 && p.first() == p.last())
}

fn ler_ponto(t: &str) -> Option<[f64; 3]> {
    let v: Vec<f64> = t.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    match v.len() {
        2 => Some([v[0], v[1], 0.0]),
        3 => Some([v[0], v[1], v[2]]),
        _ => None,
    }
}

fn e_polilinha(e: &document::EntityType) -> bool {
    curvas::pontos(e).is_some()
}


impl OpenCADStudio {
    fn civil_resultado(&mut self, r: Result<String, String>) {
        match r {
            Ok(m) => self.command_line.push_output(&m),
            Err(e) => self.command_line.push_error(&e),
        }
    }

    fn civil_abrir(&mut self, st: State) {
        self.civil_dialog = Some(st);
        self.active_modal = Some(crate::app::ModalKind::Civil);
        self.reset_modal_geometry();
    }

    /// Nome da superfície selecionada no desenho (a inserção marcada com SURFACE:<nome>), se houver uma.
    pub(crate) fn civil_superficie_selecionada(&self, i: usize) -> Option<String> {
        let scene = &self.tabs[i].scene;
        scene
            .selected_handles_in_order()
            .into_iter()
            .find_map(|h| scene.document.get_entity(h).and_then(crate::app::civil::superficie_da_entidade))
    }

    /// `_REFAZERCURVA <DIVGRIPS|SPL2PL> <handle> <valor> <S|N>`: cria a polilinha nova (mesma camada e cor) e,
    /// se pedido, apaga a original. Um desfazer volta tudo.
    fn civil_refazer_curva(&mut self, i: usize, args: &[&str]) -> Result<String, String> {
        use crate::modules::draw::inquiry::divide::{polilinha_dos_pontos, pontos_a_cada, pontos_em_n};
        let [modo, h, valor, apagar] = args else { return Err("Usage: _REFAZERCURVA mode handle value Y|N".into()) };
        let h = u64::from_str_radix(h, 16).map(document::Handle::new).map_err(|_| "Invalid handle.".to_string())?;
        let e = self.tabs[i].scene.document.get_entity(h).cloned().ok_or("Object no longer exists.")?;
        let r = if modo.eq_ignore_ascii_case("DIVGRIPS") {
            valor.parse::<f64>().ok().and_then(|v| pontos_a_cada(&e, v))
        } else {
            valor.parse::<usize>().ok().and_then(|n| pontos_em_n(&e, n))
        };
        let (pts, fechada) = r.ok_or("Could not rebuild this object.")?;
        let nova = polilinha_dos_pontos(&e, &pts, fechada).ok_or("Could not rebuild this object.")?;
        self.push_undo_snapshot(i, if modo.eq_ignore_ascii_case("DIVGRIPS") { "DIVGRIPS" } else { "SPL2PL" });
        self.tabs[i].scene.add_entities(vec![nova]);
        let apagou = *apagar == "S";
        if apagou {
            self.tabs[i].scene.erase_entities(&[h]);
        }
        self.tabs[i].dirty = true;
        Ok(format!(
            "Polyline rebuilt with {} vertices{}.",
            pts.len(),
            if apagou { "; original deleted" } else { "" }
        ))
    }

    pub(crate) fn civil_abrir_toolspace(&mut self) {
        if !self.show_toolspace {
            self.show_toolspace = true;
            self.civil_encaixar_toolspace();
        }
    }

    /// Primeira vez: o Toolspace vai para a borda oposta à do Properties e essa borda passa a se esconder
    /// sozinha (pedido do JP, 18/11: Properties sempre fixo, Toolspace só ao passar o mouse). No Open CAD Studio o
    /// fixar vale por borda; depois disso o JP muda à vontade pelo alfinete.
    fn civil_encaixar_toolspace(&mut self) {
        use crate::app::config::DockSide;
        use crate::ui::dock::PanelId;
        if !self.dock.is_placed(PanelId::Toolspace) {
            let lado = match self.dock.location(PanelId::Properties) {
                Some((DockSide::Right, _)) => DockSide::Left,
                _ => DockSide::Right,
            };
            self.dock.dock(PanelId::Toolspace, lado, 0);
            match lado {
                DockSide::Left => self.dock.auto_hide.0 = true,
                DockSide::Right => self.dock.auto_hide.1 = true,
            }
            self.save_config();
        }
        self.dock_reveal(PanelId::Toolspace);
    }

    pub(super) fn dispatch_surface(&mut self, cmd: &str, i: usize) -> Option<Task<Message>> {
        let cmd = cmd.trim();
        let (verbo, resto) = cmd.split_once(char::is_whitespace).unwrap_or((cmd, ""));
        let verbo = verbo.to_ascii_uppercase();
        let args: Vec<&str> = resto.split_whitespace().collect();
        match verbo.as_str() {
            "TOOLSPACE" => {
                self.show_toolspace ^= true;
                if self.show_toolspace {
                    self.civil_encaixar_toolspace();
                }
            }
            // Lisps do JP refeitos no programa (18/11): DIVGRIPS (vértice a cada X m) e SPL2PL (N vértices).
            "DIVGRIPS" | "SPL2PL" => {
                let nome = if verbo == "DIVGRIPS" { "DIVGRIPS" } else { "SPL2PL" };
                let c = crate::modules::civil::refazer_curva::RefazerCurvaCommand::new(nome);
                use crate::command::CadCommand;
                self.command_line.push_info(&c.prompt());
                self.tabs[i].active_cmd = Some(Box::new(c));
            }
            crate::modules::civil::refazer_curva::APLICAR => match self.civil_refazer_curva(i, &args) {
                Ok(m) => self.command_line.push_info(&m),
                Err(e) => self.command_line.push_error(&e),
            },
            "CREATESURFACE" => {
                if let Some(nome) = args.first() {
                    // Script: CREATESURFACE <nome> [STYLE=<n>] [descrição]; <n> = posição na lista de estilos (1 = Standard).
                    let mut estilo = crate::civil::estilos::ESTILOS[0].nome.to_string();
                    let mut desc = Vec::new();
                    for a in &args[1..] {
                        match a.to_ascii_uppercase().strip_prefix("STYLE=").and_then(|v| v.parse::<usize>().ok()) {
                            Some(k) => estilo = crate::civil::estilos::ESTILOS.get(k.saturating_sub(1)).map_or(estilo, |e| e.nome.to_string()),
                            None => desc.push(*a),
                        }
                    }
                    let r = self.civil_criar(i, nome, &desc.join(" "), &estilo);
                    self.civil_resultado(r);
                } else {
                    let mut st = State::novo(Modo::Criar);
                    st.nome = self.civil_proximo_nome(i);
                    self.civil_abrir(st);
                }
                self.civil_abrir_toolspace();
            }
            "ADDSURFACECONTOURS" => {
                // Script: ADDSURFACECONTOURS <superfície> <camada> [FIX=0|1] [JOIN=<m>]
                if let (Some(nome), Some(camada)) = (args.first(), args.get(1)) {
                    let r = self.civil_escolher(i, Some(nome)).and_then(|nome| {
                        let mut item = ItemDwg::curvas(camada, Vec::new());
                        for a in &args[2..] {
                            let a = a.to_ascii_uppercase();
                            if let Some(v) = a.strip_prefix("FIX=") {
                                item.corrigir = v != "0";
                            } else if let Some(v) = a.strip_prefix("JOIN=") {
                                item.emendar = v.parse().unwrap_or(0.0);
                            }
                        }
                        item.fontes = self.tabs[i]
                            .scene
                            .document
                            .entities()
                            .filter(|e| e.common().layer.eq_ignore_ascii_case(camada) && e_polilinha(e))
                            .map(|e| e.common().handle)
                            .collect();
                        self.civil_acrescentar_curvas(i, &nome, item)
                    });
                    self.civil_resultado(r);
                    return Some(Task::none());
                }
                let todas = self.civil_superficies(i);
                if todas.is_empty() {
                    self.command_line.push_error("No surface in the drawing. Create one first (CREATESURFACE).");
                    let mut st = State::novo(Modo::Criar);
                    st.nome = self.civil_proximo_nome(i);
                    self.civil_abrir(st);
                    return Some(Task::none());
                }
                let mut st = State::novo(Modo::Curvas);
                st.superficie = args
                    .first()
                    .and_then(|n| todas.iter().find(|s| s.eq_ignore_ascii_case(n)).cloned())
                    .or_else(|| todas.first().cloned());
                st.superficies = todas;
                self.civil_abrir(st);
            }
            APLICAR_CURVAS => {
                // _ADDCONTOURDATA <superfície> <corrigir 0|1> <emendar m> [descrição]
                let (Some(nome), Some(fix), Some(join)) = (args.first(), args.get(1), args.get(2)) else {
                    return Some(Task::none());
                };
                let fontes = self.tabs[i].scene.selected_handles_in_order();
                let mut item = ItemDwg::curvas(&args[3..].join(" "), fontes);
                item.corrigir = *fix != "0";
                item.emendar = join.parse().unwrap_or(0.0);
                let r = self.civil_acrescentar_curvas(i, nome, item);
                self.tabs[i].scene.deselect_all();
                self.refresh_properties();
                self.civil_resultado(r);
            }
            "EXTRACTSURFACEOBJECTS" => {
                // EXTRACTSURFACEOBJECTS [superfície] [MAJOR] [MINOR] [BORDER] [TRIANGLES]; sem tipos = curvas e borda.
                use crate::civil::estilos::{CAMADA_BORDA, CAMADA_INTERMEDIARIA, CAMADA_MESTRA, CAMADA_TRIANGULOS};
                let mut nome = None;
                let mut camadas: Vec<&str> = Vec::new();
                for a in &args {
                    match a.to_ascii_uppercase().as_str() {
                        "MAJOR" => camadas.push(CAMADA_MESTRA.0),
                        "MINOR" => camadas.push(CAMADA_INTERMEDIARIA.0),
                        "BORDER" => camadas.push(CAMADA_BORDA.0),
                        "TRIANGLES" => camadas.push(CAMADA_TRIANGULOS.0),
                        _ => nome = Some(*a),
                    }
                }
                if camadas.is_empty() {
                    camadas = vec![CAMADA_MESTRA.0, CAMADA_INTERMEDIARIA.0, CAMADA_BORDA.0];
                }
                let nome = nome.map(str::to_string).or_else(|| self.civil_superficie_selecionada(i));
                let r = self.civil_escolher(i, nome.as_deref()).and_then(|n| self.civil_extrair(i, &n, &camadas));
                self.civil_resultado(r);
            }
            // Script: ADDSURFACEBOUNDARIES <superfície> OUTER|HIDE <camada> (polilinhas fechadas da camada)
            "ADDSURFACEBOUNDARIES" if args.len() >= 3 => {
                let camada = args[2];
                let fontes: Vec<_> = self.tabs[i]
                    .scene
                    .document
                    .entities()
                    .filter(|e| e.common().layer.eq_ignore_ascii_case(camada) && e_polilinha_fechada(e))
                    .map(|e| e.common().handle)
                    .collect();
                let tipo = if args[1].eq_ignore_ascii_case("HIDE") { "BOUNDARY_HIDE" } else { "BOUNDARY_OUTER" };
                let mut item = ItemDwg::novo(tipo, camada);
                item.fontes = fontes;
                let r = self.civil_escolher(i, Some(args[0])).and_then(|n| self.civil_acrescentar_item(i, &n, item));
                self.civil_resultado(r);
            }
            "ADDSURFACEBOUNDARIES" => match self.civil_escolher(i, args.first().copied()) {
                Ok(n) => {
                    let mut st = State::novo(Modo::Limite(n.clone()));
                    st.superficie = Some(n);
                    self.civil_abrir(st);
                }
                Err(e) => self.command_line.push_error(&e),
            },
            APLICAR_LIMITE => {
                // _ADDBOUNDARYDATA <superfície> OUTER|HIDE [descrição]
                let (Some(nome), Some(tipo)) = (args.first(), args.get(1)) else { return Some(Task::none()) };
                let mut item = ItemDwg::novo(
                    if tipo.eq_ignore_ascii_case("HIDE") { "BOUNDARY_HIDE" } else { "BOUNDARY_OUTER" },
                    &args[2..].join(" "),
                );
                item.fontes = self.tabs[i].scene.selected_handles_in_order();
                let r = self.civil_acrescentar_item(i, nome, item);
                self.tabs[i].scene.deselect_all();
                self.refresh_properties();
                self.civil_resultado(r);
            }
            "ADDSURFACEPOINT" | "DELETESURFACEPOINT" | "DELETESURFACELINE" => {
                match self.civil_escolher(i, args.first().copied()) {
                    Ok(n) => {
                        let (nome_cmd, texto, tipo, cota): (&'static str, &str, &str, bool) = match verbo.as_str() {
                            "ADDSURFACEPOINT" => ("ADDSURFACEPOINT", "Specify a location for the new point:", "EDIT_ADDPOINT", true),
                            "DELETESURFACEPOINT" => ("DELETESURFACEPOINT", "Select the point to delete:", "EDIT_DELETEPOINT", false),
                            _ => ("DELETESURFACELINE", "Select the triangle edge to delete:", "EDIT_DELETELINE", false),
                        };
                        let c = crate::modules::civil::pegar_ponto::PegarPontoCommand::new(nome_cmd, texto, format!("{APLICAR_EDIT} {n} {tipo}"), cota);
                        use crate::command::CadCommand;
                        self.command_line.push_info(&c.prompt());
                        self.tabs[i].active_cmd = Some(Box::new(c));
                    }
                    Err(e) => self.command_line.push_error(&e),
                }
            }
            "RAISELOWERSURFACE" => match self.civil_escolher(i, args.first().copied()) {
                Ok(n) => self.civil_abrir(State::novo(Modo::Elevar(n))),
                Err(e) => self.command_line.push_error(&e),
            },
            APLICAR_EDIT => {
                // _SURFACEEDIT <superfície> <EDIT_...> <x,y[,z]> | <valor>
                let (Some(nome), Some(tipo), Some(arg)) = (args.first(), args.get(1), args.get(2)) else { return Some(Task::none()) };
                let mut item = ItemDwg::novo(tipo, "");
                if *tipo == "EDIT_RAISELOWER" {
                    item.valor = arg.parse().unwrap_or(0.0);
                } else if let Some(p) = ler_ponto(arg) {
                    item.pontos.push(p);
                } else {
                    self.command_line.push_error("Invalid point.");
                    return Some(Task::none());
                }
                let r = self.civil_acrescentar_item(i, nome, item);
                self.civil_resultado(r);
            }
            "REBUILDSURFACE" => {
                let r = self.civil_escolher(i, args.first().copied()).and_then(|n| self.civil_refazer(i, &n));
                self.civil_resultado(r);
            }
            "ZOOMSURFACE" => match self.civil_escolher(i, args.first().copied()).and_then(|n| self.civil_ler(i, &n)) {
                Ok(s) => {
                    let malhas: Vec<_> = s.desenho.clone();
                    if !self.tabs[i].scene.zoom_to_entities(&malhas) {
                        self.command_line.push_error(&format!("Surface '{}' has nothing drawn yet.", s.nome));
                    }
                }
                Err(e) => self.command_line.push_error(&e),
            },
            "SURFACEPROPERTIES" => match self.civil_escolher(i, args.first().copied()).and_then(|n| self.civil_ler(i, &n)) {
                Ok(s) => {
                    let mut st = State::novo(Modo::Propriedades(s.nome.clone()));
                    st.nome = s.nome;
                    st.descricao = s.descricao;
                    st.comprimento = s.comprimento_maximo.map_or("0".into(), |c| c.to_string());
                    st.estilo = Some(crate::civil::estilos::estilo(&s.estilo).nome.to_string());
                    self.civil_abrir(st);
                }
                Err(e) => self.command_line.push_error(&e),
            },
            "DELETESURFACE" => match self.civil_escolher(i, args.first().copied()) {
                Ok(n) => self.civil_abrir(State::novo(Modo::Apagar(n))),
                Err(e) => self.command_line.push_error(&e),
            },
            "SURFACEDEFINITION" => {
                // SURFACEDEFINITION <superfície> <item> ON|OFF|REMOVE (menu do Toolspace)
                let (Some(nome), Some(k), Some(acao)) = (args.first(), args.get(1).and_then(|k| k.parse::<usize>().ok()), args.get(2)) else {
                    self.command_line.push_error("Usage: SURFACEDEFINITION <surface> <item> ON|OFF|REMOVE");
                    return Some(Task::none());
                };
                let r = match acao.to_ascii_uppercase().as_str() {
                    "ON" => self.civil_ligar_item(i, nome, k, true),
                    "OFF" => self.civil_ligar_item(i, nome, k, false),
                    "REMOVE" => self.civil_remover_item(i, nome, k),
                    _ => Err("Use ON, OFF or REMOVE.".into()),
                };
                self.civil_resultado(r);
            }
            _ => return None,
        }
        Some(Task::none())
    }

    pub(in crate::app) fn on_civil_dialog(&mut self, msg: CivilMsg) -> Task<Message> {
        let i = self.active_tab;
        let Some(st) = self.civil_dialog.as_mut() else { return Task::none() };
        match msg {
            CivilMsg::Nome(v) => st.nome = v,
            CivilMsg::Descricao(v) => st.descricao = v,
            CivilMsg::Superficie(v) => st.superficie = Some(v),
            CivilMsg::Estilo(v) => st.estilo = Some(v),
            CivilMsg::Corrigir(v) => st.corrigir = v,
            CivilMsg::Emendar(v) => st.emendar = v,
            CivilMsg::Folga(v) => st.folga = v,
            CivilMsg::LimitarBorda(v) => st.limitar_borda = v,
            CivilMsg::Comprimento(v) => st.comprimento = v,
            CivilMsg::TipoLimite(v) => st.tipo_limite = Some(v),
            CivilMsg::Ok => {
                let modo = st.modo.clone();
                let (nome, descricao) = (st.nome.clone(), st.descricao.clone());
                let estilo = st.estilo.clone().unwrap_or_default();
                let r: Result<Option<String>, String> = match &modo {
                    Modo::Criar => self.civil_criar(i, &nome, &descricao, &estilo).map(Some),
                    Modo::Apagar(n) => self.civil_apagar(i, n).map(Some),
                    Modo::Propriedades(n) => {
                        let c = st.comprimento.trim().replace(',', ".");
                        match c.parse::<f64>() {
                            Ok(v) if v >= 0.0 => self.civil_propriedades(i, n, &nome, &descricao, &estilo, (v > 0.0).then_some(v)).map(Some),
                            _ => Err("Maximum triangle length must be a number (0 = none).".into()),
                        }
                    }
                    Modo::Limite(n) => {
                        let tipo = if st.tipo_limite.as_deref() == Some("Hide") { "HIDE" } else { "OUTER" };
                        let pendente = format!("{APLICAR_LIMITE} {n} {tipo} {}", descricao.trim());
                        self.close_active_modal();
                        use crate::command::CadCommand;
                        use crate::modules::draw::select::SelectObjectsCommand;
                        let c = SelectObjectsCommand::filtered("ADDSURFACEBOUNDARIES", &pendente, "Select closed polylines:", e_polilinha_fechada);
                        self.command_line.push_info(&c.prompt());
                        self.tabs[i].active_cmd = Some(Box::new(c));
                        return Task::none();
                    }
                    Modo::Elevar(n) => match st.comprimento.trim().replace(',', ".").parse::<f64>() {
                        Ok(v) => {
                            let n = n.clone();
                            self.close_active_modal();
                            return self.dispatch_command(&format!("{APLICAR_EDIT} {n} EDIT_RAISELOWER {v}"));
                        }
                        Err(_) => Err("Type a number in meters.".into()),
                    },
                    Modo::Curvas => {
                        let folga = st.folga.trim().replace(',', ".").parse::<f64>();
                        match (st.superficie.clone(), folga) {
                            (None, _) => Err("Choose a surface.".into()),
                            (_, Err(_)) if st.emendar => Err("The gap must be a number in meters.".into()),
                            (Some(s), f) => {
                                let emendar = if st.emendar { f.unwrap_or(0.0).max(0.0) } else { 0.0 };
                                let pendente = format!(
                                    "{APLICAR_CURVAS} {s} {} {} {}",
                                    st.corrigir as u8,
                                    emendar,
                                    descricao.trim()
                                );
                                self.close_active_modal();
                                let preselecao: Vec<_> = self.tabs[i]
                                    .scene
                                    .selected_handles_in_order()
                                    .into_iter()
                                    .filter(|h| self.tabs[i].scene.document.get_entity(*h).is_some_and(e_polilinha))
                                    .collect();
                                if preselecao.is_empty() {
                                    use crate::command::CadCommand;
                                    use crate::modules::draw::select::SelectObjectsCommand;
                                    let c = SelectObjectsCommand::filtered("ADDSURFACECONTOURS", &pendente, "Select contours:", e_polilinha);
                                    self.command_line.push_info(&c.prompt());
                                    self.tabs[i].active_cmd = Some(Box::new(c));
                                    return Task::none();
                                }
                                return self.dispatch_command(&pendente);
                            }
                        }
                    }
                };
                match r {
                    Ok(m) => {
                        if let Some(m) = m {
                            self.command_line.push_output(&m);
                        }
                        self.close_active_modal();
                    }
                    Err(e) => {
                        if let Some(st) = self.civil_dialog.as_mut() {
                            st.erro = Some(e);
                        }
                    }
                }
            }
        }
        Task::none()
    }
}
