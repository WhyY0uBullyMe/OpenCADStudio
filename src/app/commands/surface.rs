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

/// Comando interno que a seleção na tela dispara de volta (o usuário não digita).
const APLICAR_CURVAS: &str = "_ADDCONTOURDATA";

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

    pub(crate) fn civil_abrir_toolspace(&mut self) {
        if !self.show_toolspace {
            self.show_toolspace = true;
            self.dock_open_at_default(crate::ui::dock::PanelId::Toolspace);
        }
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
                    self.dock_open_at_default(crate::ui::dock::PanelId::Toolspace);
                }
            }
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
