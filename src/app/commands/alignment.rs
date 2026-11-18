//! Comandos de alinhamento, perfil e seções (spec 003). Todos têm forma de roteiro (com argumentos, para o lote e
//! o MCP) e forma interativa (sem argumentos: seleciona, pergunta e pede o ponto).
//!
//! - `CREATEALIGNMENT [nome] [camada]`: da polilinha da camada, ou da selecionada.
//! - `ALIGNMENTSTATIONS <eixo> <estaca inicial> [intervalo]`.
//! - `CREATEPROFILEVIEW [eixo] [superfície] [exagero] [x,y]`.
//! - `CREATEPROFILE [eixo] [e,z[,L] ...]`: o greide; sem PIVs, pede um por linha ou clique na vista.
//! - `CREATESECTIONS [eixo] [intervalo] [esquerda] [direita] [colunas] [x,y]`: linhas e vistas de seção.
//! - `REBUILDALIGNMENT [eixo]`, `DELETEALIGNMENT <eixo>`.

use crate::app::{Message, OpenCADStudio};
use crate::civil::alinhamentos::Secoes;
use agral_civil_motor::alinhamento::Piv;
use iced::Task;

fn ponto2(t: &str) -> Option<[f64; 2]> {
    let v: Vec<f64> = t.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    (v.len() >= 2).then(|| [v[0], v[1]])
}

fn num(t: Option<&&str>, padrao: f64) -> Option<f64> {
    match t {
        None => Some(padrao),
        Some(t) => t.replace(',', ".").parse().ok(),
    }
}

impl OpenCADStudio {
    fn civil_al_resultado(&mut self, r: Result<String, String>) {
        match r {
            Ok(m) => self.command_line.push_info(&m),
            Err(e) => self.command_line.push_error(&e),
        }
        self.refresh_properties();
    }

    /// Abre o EDITSAMPLELINES do eixo com as estacas atuais das sample lines.
    fn civil_editar_secoes(&mut self, i: usize, eixo: &str, modo: crate::modules::civil::perfil_edicao::Modo) -> Result<(), String> {
        let a = self.civil_al_ler(i, eixo)?;
        let s = a.secoes.clone().ok_or_else(|| format!("Alignment '{eixo}' has no sample lines: run CREATESECTIONS first."))?;
        let al = a.geometria().ok_or("Alignment without geometry.")?;
        let estacas = s.estacas_no(&al);
        let c = crate::modules::civil::secoes_edicao::SecoesEdicaoCommand::new(&a.nome, al, s.esquerda, s.direita, estacas, modo);
        self.civil_instalar(i, Box::new(c));
        Ok(())
    }

    /// Abre o EDITPROFILE do eixo com o greide e a vista de perfil atuais.
    fn civil_editar_perfil(&mut self, i: usize, eixo: &str, modo: crate::modules::civil::perfil_edicao::Modo) -> Result<(), String> {
        let a = self.civil_al_ler(i, eixo)?;
        if a.pivs.is_empty() {
            return Err(format!("Alignment '{eixo}' has no design profile yet: use CREATEPROFILE."));
        }
        let vista = a.vista.as_ref().map(|v| (v.origem[0], v.origem[1], a.estaca_inicial, v.base, v.exagero));
        if vista.is_none() {
            self.command_line.push_info("No profile view: type station,elevation (or create the profile view to click in it).");
        }
        let pivs = a.pivs.iter().map(|p| (p.estaca, p.cota, p.curva)).collect();
        let c = crate::modules::civil::perfil_edicao::PerfilEdicaoCommand::new(&a.nome, vista, pivs, modo);
        self.civil_instalar(i, Box::new(c));
        Ok(())
    }

    pub(super) fn civil_instalar(&mut self, i: usize, c: Box<dyn crate::command::CadCommand>) {
        self.command_line.push_info(&c.prompt());
        self.tabs[i].active_cmd = Some(c);
    }

    /// Padrão "<nome>" para a pergunta do alinhamento: o único do desenho, se houver um só.
    fn civil_pergunta_eixo(&self, i: usize) -> String {
        match self.civil_al_escolher(i, None) {
            Ok(n) => format!("Alignment name <{n}>:"),
            Err(_) => "Alignment name:".into(),
        }
    }

    pub(super) fn civil_pergunta_superficie(&self, i: usize, eixo: Option<&str>) -> String {
        let da = eixo.and_then(|e| self.civil_al_ler(i, e).ok()).map(|a| a.superficie).filter(|s| !s.is_empty());
        match da.or_else(|| self.civil_escolher(i, None).ok()) {
            Some(n) => format!("Surface name <{n}>:"),
            None => "Surface name:".into(),
        }
    }

    fn civil_pedir_ponto(&mut self, i: usize, nome: &'static str, texto: &str, pendente: String) {
        use crate::command::CadCommand;
        let c = crate::modules::civil::pegar_ponto::PegarPontoCommand::new(nome, texto, pendente, false);
        self.command_line.push_info(&c.prompt());
        self.tabs[i].active_cmd = Some(Box::new(c));
    }

    pub(super) fn dispatch_alinhamento(&mut self, verbo: &str, args: &[&str], i: usize) -> Option<Task<Message>> {
        match verbo {
            // Do objeto (Civil 3D: Create Alignment from Objects): polilinhas já selecionadas viram eixos na hora;
            // sem seleção, pede para escolher e a escolhida já vira (pedido do JP, 18/11).
            "CREATEALIGNMENTENTITIES" | "CREATEALIGNFROMOBJECTS" => {
                let sel: Vec<_> = self.tabs[i]
                    .scene
                    .selected_handles_in_order()
                    .into_iter()
                    .filter(|h| {
                        self.tabs[i].scene.document.get_entity(*h).and_then(crate::app::civil_alinhamento::vertices_com_bulge).is_some()
                    })
                    .collect();
                if sel.is_empty() {
                    let nome = args.first().map(|s| s.to_string()).unwrap_or_else(|| self.civil_proximo_eixo(i));
                    let c = crate::modules::civil::eixo_fonte::EixoFonteCommand::new(&nome);
                    self.civil_instalar(i, Box::new(c));
                } else {
                    let mut msgs = Vec::new();
                    for (k, h) in sel.into_iter().enumerate() {
                        let nome = match (k, args.first()) {
                            (0, Some(n)) => n.to_string(),
                            _ => self.civil_proximo_eixo(i),
                        };
                        match self.civil_criar_alinhamento(i, &nome, h) {
                            Ok(m) => msgs.push(m),
                            Err(e) => msgs.push(format!("ERROR: {e}")),
                        }
                    }
                    self.tabs[i].scene.deselect_all();
                    let erro = msgs.iter().all(|m| m.starts_with("ERROR"));
                    let texto = msgs.join(" ");
                    self.civil_al_resultado(if erro { Err(texto) } else { Ok(texto) });
                }
            }
            // Compute Materials (spec 004): terreno x corredor ou qualquer par de superfícies, nas linhas de seção.
            "COMPUTEMATERIALS" if args.len() >= 3 => {
                let r = match (num(args.get(3), 1.0), num(args.get(4), 1.0)) {
                    (Some(fc), Some(fa)) => self.civil_computar_materiais(i, args[0], args[1], args[2], fc, fa),
                    _ => Err("Usage: COMPUTEMATERIALS alignment base_surface compare_surface [cut_factor] [fill_factor]".into()),
                };
                self.civil_al_resultado(r);
            }
            "COMPUTEMATERIALS" => {
                let eixo = args.first().map(|s| s.to_string()).or_else(|| self.civil_al_escolher(i, None).ok());
                let a = eixo.as_deref().and_then(|e| self.civil_al_ler(i, e).ok());
                let base = a.as_ref().map(|a| a.materiais.as_ref().map_or(a.superficie.clone(), |m| m.base.clone())).filter(|s| !s.is_empty());
                let comparar = a.as_ref().and_then(|a| {
                    a.materiais.as_ref().map(|m| m.comparar.clone()).or_else(|| {
                        let c = crate::app::civil_alinhamento::superficie_do_corredor(&a.nome);
                        self.civil_escolher(i, Some(&c)).ok()
                    })
                });
                let (fc, fa) = a.as_ref().and_then(|a| a.materiais.as_ref()).map_or((1.0, 1.0), |m| (m.fator_corte, m.fator_aterro));
                let padrao = |t: &str, v: Option<String>| match v {
                    Some(v) => format!("{t} <{v}>:"),
                    None => format!("{t}:"),
                };
                let mut perguntas = Vec::new();
                let verbo = match (&eixo, args.first()) {
                    (Some(e), Some(_)) => format!("COMPUTEMATERIALS {e}"),
                    _ => {
                        perguntas.push(padrao("Alignment name", eixo.clone()));
                        "COMPUTEMATERIALS".into()
                    }
                };
                perguntas.push(padrao("Base surface (existing ground)", base));
                perguntas.push(padrao("Comparison surface (design, corridor)", comparar));
                perguntas.push(format!("Cut factor <{fc}>:"));
                perguntas.push(format!("Fill factor <{fa}>:"));
                let c = crate::modules::civil::perguntas::PerguntasCommand::com("COMPUTEMATERIALS", &verbo, perguntas);
                self.civil_instalar(i, Box::new(c));
            }
            "CREATEVOLUMETABLE" if args.len() >= 2 => {
                let r = match ponto2(args[1]) {
                    Some(p) => self.civil_tabela_volumes(i, args[0], p),
                    None => Err("Usage: CREATEVOLUMETABLE alignment x,y".into()),
                };
                self.civil_al_resultado(r);
            }
            "CREATEVOLUMETABLE" => {
                let mut c = crate::modules::civil::perguntas::PerguntasCommand::com(
                    "CREATEVOLUMETABLE",
                    "CREATEVOLUMETABLE",
                    vec![self.civil_pergunta_eixo(i)],
                );
                if let Some(e) = args.first() {
                    c = crate::modules::civil::perguntas::PerguntasCommand::com("CREATEVOLUMETABLE", &format!("CREATEVOLUMETABLE {e}"), Vec::new());
                }
                self.civil_instalar(i, Box::new(c.e_ponto("Specify the upper left corner of the volume table:")));
            }
            "EXPORTMATERIALS" => match self.civil_al_escolher(i, args.first().copied()) {
                Ok(n) => {
                    if let Some(caminho) = args.get(1) {
                        let r = self.civil_exportar_materiais(i, &n, std::path::Path::new(caminho));
                        self.civil_al_resultado(r);
                    } else {
                        let sugestao = format!("Volumes_{n}.csv");
                        return Some(iced::Task::perform(
                            async move {
                                let h = crate::sys::file_dialog()
                                    .set_title("Export Materials")
                                    .add_filter("CSV (*.csv)", &["csv"])
                                    .set_file_name(&sugestao)
                                    .save_file()
                                    .await?;
                                Some(crate::sys::handle_path(&h))
                            },
                            move |p| match p {
                                Some(p) => Message::Command(format!("EXPORTMATERIALS {n} \"{}\"", p.display())),
                                None => Message::Noop,
                            },
                        ));
                    }
                }
                Err(e) => self.command_line.push_error(&e),
            },
            // SAMPLEMORESOURCES <eixo> <superfície>... | NONE (sem as extras). Sozinho, abre a janela.
            "SAMPLEMORESOURCES" => {
                let r = match args.split_first() {
                    Some((eixo, resto)) => {
                        let fontes: Vec<String> = resto.iter().filter(|f| !f.eq_ignore_ascii_case("NONE")).map(|f| f.to_string()).collect();
                        self.civil_mais_fontes(i, eixo, fontes)
                    }
                    None => Err("Usage: SAMPLEMORESOURCES alignment surface... (or NONE)".into()),
                };
                self.civil_al_resultado(r);
            }
            // EDITSAMPLELINES [eixo]: mover, incluir e apagar sample lines clicando na planta.
            "EDITSAMPLELINES" => match self.civil_al_escolher(i, args.first().copied()) {
                Ok(eixo) => {
                    let modo = args.get(1).and_then(|m| crate::modules::civil::perfil_edicao::Modo::de(m));
                    if let Err(e) = self.civil_editar_secoes(i, &eixo, modo.unwrap_or(crate::modules::civil::perfil_edicao::Modo::Mover)) {
                        self.command_line.push_error(&e);
                    }
                }
                Err(e) => self.command_line.push_error(&e),
            },
            "_SETSAMPLELINES" => {
                let editar = args.last().and_then(|a| a.strip_prefix("+EDIT=")).and_then(crate::modules::civil::perfil_edicao::Modo::de);
                let args: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with("+EDIT=")).collect();
                let r = match args.split_first() {
                    Some((eixo, resto)) => {
                        let est: Option<Vec<f64>> = resto.iter().map(|t| t.replace(',', ".").parse::<f64>().ok()).collect();
                        match est {
                            Some(e) => self.civil_estacas_das_secoes(i, eixo, e),
                            None => Err("Stations must be numbers.".into()),
                        }
                    }
                    None => Err("Usage: _SETSAMPLELINES alignment station...".into()),
                };
                let ok = r.is_ok();
                self.civil_al_resultado(r);
                if let (true, Some(m), Some(eixo)) = (ok, editar, args.first()) {
                    let _ = self.civil_editar_secoes(i, eixo, m);
                }
            }
            // Traçado por PIs (Civil 3D: Alignment Creation Tools).
            "CREATEALIGNMENTLAYOUT" => {
                let nome = args.first().map(|s| s.to_string()).unwrap_or_else(|| self.civil_proximo_eixo(i));
                let c = crate::modules::civil::tracado::TracadoCommand::new(&nome);
                self.civil_instalar(i, Box::new(c));
            }
            "_ALIGNFROMPOINTS" => {
                let r = (|| {
                    let [nome, raio, pts] = args else { return Err("Usage: _ALIGNFROMPOINTS name radius x,y;x,y;...".to_string()) };
                    let raio: f64 = raio.parse().map_err(|_| "Invalid radius.".to_string())?;
                    let pis: Vec<[f64; 2]> = pts
                        .split(';')
                        .map(|p| {
                            let v: Vec<f64> = p.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                            (v.len() >= 2).then(|| [v[0], v[1]]).ok_or_else(|| format!("Invalid point '{p}'."))
                        })
                        .collect::<Result<_, _>>()?;
                    let v = agral_civil_motor::alinhamento::tangentes_com_curvas(&pis, raio);
                    self.civil_criar_alinhamento_de_vertices(i, nome, v, false)
                })();
                self.civil_al_resultado(r);
            }
            // Sem nada: escolhe o jeito (do objeto ou traçando os PIs).
            "CREATEALIGNMENT" if args.is_empty() && self.tabs[i].scene.selected_handles_in_order().len() != 1 => {
                let c = crate::modules::civil::perguntas::PerguntasCommand::com(
                    "CREATEALIGNMENT",
                    "_ALIGNMETHOD",
                    vec!["Create alignment [Entities/Layout] <Entities>:".into()],
                );
                self.civil_instalar(i, Box::new(c));
            }
            "_ALIGNMETHOD" => {
                let layout = args.first().is_some_and(|a| a.to_ascii_uppercase().starts_with('L'));
                return self.dispatch_alinhamento(if layout { "CREATEALIGNMENTLAYOUT" } else { "CREATEALIGNMENTENTITIES" }, &[], i);
            }
            "CREATEALIGNMENT" => {
                if args.is_empty() {
                    let sel = self.tabs[i].scene.selected_handles_in_order();
                    let nome = self.civil_proximo_eixo(i);
                    if sel.len() == 1 {
                        // Uma polilinha já selecionada vira o eixo (o jeito mais rápido na tela).
                        let r = self.civil_criar_alinhamento(i, &nome, sel[0]);
                        self.tabs[i].scene.deselect_all();
                        self.civil_al_resultado(r);
                    } else {
                        let c = crate::modules::civil::perguntas::PerguntasCommand::com(
                            "CREATEALIGNMENT",
                            "CREATEALIGNMENT",
                            vec![format!("Alignment name <{nome}>:")],
                        );
                        self.civil_instalar(i, Box::new(c));
                    }
                } else if let Some(camada) = args.get(1) {
                    let fonte = self.tabs[i]
                        .scene
                        .document
                        .entities()
                        .filter(|e| e.common().layer.eq_ignore_ascii_case(camada))
                        .find(|e| crate::app::civil_alinhamento::vertices_com_bulge(e).is_some_and(|(v, _)| v.len() >= 2))
                        .map(|e| e.common().handle);
                    let r = match fonte {
                        Some(h) => self.civil_criar_alinhamento(i, args[0], h),
                        None => Err(format!("No polyline on layer {camada}.")),
                    };
                    self.civil_al_resultado(r);
                } else {
                    let c = crate::modules::civil::eixo_fonte::EixoFonteCommand::new(args[0]);
                    self.civil_instalar(i, Box::new(c));
                }
            }
            "_ALIGNFROMHANDLE" => {
                let r = match (args.first(), args.get(1).and_then(|h| u64::from_str_radix(h, 16).ok())) {
                    (Some(n), Some(h)) => self.civil_criar_alinhamento(i, n, document::Handle::new(h)),
                    _ => Err("Usage: _ALIGNFROMHANDLE name handle".into()),
                };
                self.civil_al_resultado(r);
            }
            "_ALIGNFROMSEL" => {
                let nome = args.first().map(|s| s.to_string()).unwrap_or_else(|| self.civil_proximo_eixo(i));
                let sel = self.tabs[i].scene.selected_handles_in_order();
                let r = match sel.first() {
                    Some(h) => self.civil_criar_alinhamento(i, &nome, *h),
                    None => Err("Nothing selected.".into()),
                };
                self.tabs[i].scene.deselect_all();
                self.civil_al_resultado(r);
            }
            "ALIGNMENTSTATIONS" if !args.is_empty() => {
                let r = match (args.first(), num(args.get(1), 0.0), num(args.get(2), 20.0)) {
                    (Some(n), Some(e0), Some(iv)) => self.civil_estacas(i, n, e0, iv),
                    _ => Err("Usage: ALIGNMENTSTATIONS alignment start_station [interval]".into()),
                };
                self.civil_al_resultado(r);
            }
            "CREATEPROFILEVIEW" if args.is_empty() => {
                let um = self.civil_al_escolher(i, None).ok();
                let c = crate::modules::civil::perguntas::PerguntasCommand::com(
                    "CREATEPROFILEVIEW",
                    "CREATEPROFILEVIEW",
                    vec![self.civil_pergunta_eixo(i), self.civil_pergunta_superficie(i, um.as_deref()), "Vertical exaggeration <10>:".into()],
                )
                .e_ponto("Specify the lower left corner of the profile view:");
                self.civil_instalar(i, Box::new(c));
            }
            "CREATEPROFILEVIEW" => {
                let r = (|| -> Result<Option<String>, String> {
                    let eixo = self.civil_al_escolher(i, args.first().copied())?;
                    let sup = match args.get(1) {
                        Some(s) => self.civil_ler(i, s)?.nome,
                        None => {
                            let a = self.civil_al_ler(i, &eixo)?;
                            if a.superficie.is_empty() { self.civil_escolher(i, None)? } else { a.superficie }
                        }
                    };
                    let ex = num(args.get(2), 10.0).ok_or("Vertical exaggeration must be a number.")?;
                    match args.get(3).and_then(|t| ponto2(t)) {
                        Some(o) => self.civil_vista_perfil(i, &eixo, &sup, ex, o).map(Some),
                        None => {
                            self.civil_pedir_ponto(
                                i,
                                "CREATEPROFILEVIEW",
                                "Specify the lower left corner of the profile view:",
                                format!("CREATEPROFILEVIEW {eixo} {sup} {ex}"),
                            );
                            Ok(None)
                        }
                    }
                })();
                match r {
                    Ok(Some(m)) => self.civil_al_resultado(Ok(m)),
                    Ok(None) => {}
                    Err(e) => self.civil_al_resultado(Err(e)),
                }
            }
            "CREATEPROFILE" if args.is_empty() && self.civil_al_escolher(i, None).is_err() => {
                let c = crate::modules::civil::perguntas::PerguntasCommand::com("CREATEPROFILE", "CREATEPROFILE", vec![self.civil_pergunta_eixo(i)]);
                self.civil_instalar(i, Box::new(c));
            }
            "CREATEPROFILE" => match self.civil_al_escolher(i, args.first().copied()) {
                Ok(eixo) if args.len() > 1 => {
                    return Some(self.dispatch_command(&format!("_SETPROFILE {eixo} {}", args[1..].join(" "))));
                }
                Ok(eixo) => {
                    let a = self.civil_al_ler(i, &eixo).ok();
                    let vista = a.as_ref().and_then(|a| a.vista.as_ref().map(|v| (v.origem[0], v.origem[1], a.estaca_inicial, v.base, v.exagero)));
                    use crate::command::CadCommand;
                    let c = crate::modules::civil::pivs::PivsCommand::new(&eixo, vista);
                    self.command_line.push_info(&c.prompt());
                    self.tabs[i].active_cmd = Some(Box::new(c));
                }
                Err(e) => self.command_line.push_error(&e),
            },
            "EDITPROFILE" => match self.civil_al_escolher(i, args.first().copied()) {
                Ok(eixo) => {
                    let modo = args.get(1).and_then(|m| crate::modules::civil::perfil_edicao::Modo::de(m));
                    if let Err(e) = self.civil_editar_perfil(i, &eixo, modo.unwrap_or(crate::modules::civil::perfil_edicao::Modo::Mover)) {
                        self.command_line.push_error(&e);
                    }
                }
                Err(e) => self.command_line.push_error(&e),
            },
            "_SETPROFILE" => {
                // "+EDIT=<modo>" no fim: veio do EDITPROFILE, que continua aberto no mesmo modo.
                let editar = args.last().and_then(|a| a.strip_prefix("+EDIT=")).and_then(crate::modules::civil::perfil_edicao::Modo::de);
                let args: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with("+EDIT=")).collect();
                let args = args.as_slice();
                let r = match args.first() {
                    Some(eixo) => {
                        let pivs: Option<Vec<Piv>> = args[1..]
                            .iter()
                            .map(|t| crate::modules::civil::pivs::ler_piv(t).map(|(e, z, l)| Piv { estaca: e, cota: z, curva: l }))
                            .collect();
                        match pivs {
                            Some(p) => self.civil_definir_greide(i, eixo, p),
                            None => Err("PVIs must be station,elevation[,curve length].".into()),
                        }
                    }
                    None => Err("Usage: CREATEPROFILE alignment e,z[,L] ...".into()),
                };
                let ok = r.is_ok();
                self.civil_al_resultado(r);
                if let (true, Some(m), Some(eixo)) = (ok, editar, args.first()) {
                    let _ = self.civil_editar_perfil(i, eixo, m);
                }
            }
            "CREATESECTIONS" | "CREATESAMPLELINES" | "CREATESECTIONVIEWS" if args.is_empty() => {
                let c = crate::modules::civil::perguntas::PerguntasCommand::com(
                    "CREATESECTIONS",
                    "CREATESECTIONS",
                    vec![
                        self.civil_pergunta_eixo(i),
                        "Interval (m) <20>:".into(),
                        "Left width (m) <30>:".into(),
                        "Right width (m) <30>:".into(),
                        "Columns <5>:".into(),
                    ],
                )
                .e_ponto("Specify the upper left corner of the section views:");
                self.civil_instalar(i, Box::new(c));
            }
            "CREATESECTIONS" | "CREATESAMPLELINES" | "CREATESECTIONVIEWS" => {
                let r = (|| -> Result<Option<String>, String> {
                    let eixo = self.civil_al_escolher(i, args.first().copied())?;
                    let iv = num(args.get(1), 20.0).ok_or("Interval must be a number.")?;
                    let esq = num(args.get(2), 30.0).ok_or("Left width must be a number.")?;
                    let dir = num(args.get(3), 30.0).ok_or("Right width must be a number.")?;
                    let col = num(args.get(4), 5.0).ok_or("Columns must be a number.")?.max(1.0) as usize;
                    match args.get(5).and_then(|t| ponto2(t)) {
                        Some(o) => self
                            .civil_definir_secoes(i, &eixo, Secoes::nova(iv, esq, dir, o, col))
                            .map(Some),
                        None => {
                            self.civil_pedir_ponto(
                                i,
                                "CREATESECTIONS",
                                "Specify the upper left corner of the section views:",
                                format!("CREATESECTIONS {eixo} {iv} {esq} {dir} {col}"),
                            );
                            Ok(None)
                        }
                    }
                })();
                match r {
                    Ok(Some(m)) => self.civil_al_resultado(Ok(m)),
                    Ok(None) => {}
                    Err(e) => self.civil_al_resultado(Err(e)),
                }
            }
            // CREATEASSEMBLY [eixo] [crista] [decl %] [montante L|R] [talude montante] [talude jusante] [altura berma]
            //                [largura berma] [talude corte]
            "CREATEASSEMBLY" if args.is_empty() => {
                let c = crate::modules::civil::perguntas::PerguntasCommand::com(
                    "CREATEASSEMBLY",
                    "CREATEASSEMBLY",
                    vec![
                        self.civil_pergunta_eixo(i),
                        "Crest width (m) <6>:".into(),
                        "Crest cross slope toward upstream (%) <1>:".into(),
                        "Upstream side [L/R] <L>:".into(),
                        "Upstream slope H:V (H for 1 V) <3>:".into(),
                        "Downstream slope H:V <2>:".into(),
                        "Berm every (m of height, 0 = none) <8>:".into(),
                        "Berm width (m) <3>:".into(),
                        "Cut slope H:V <1>:".into(),
                    ],
                );
                self.civil_instalar(i, Box::new(c));
            }
            "CREATEASSEMBLY" => {
                let r = (|| -> Result<String, String> {
                    let eixo = self.civil_al_escolher(i, args.first().copied())?;
                    let n = |k: usize, d: f64| num(args.get(k), d).ok_or_else(|| format!("Value {} is not a number.", args[k]));
                    let lado = args.get(3).map(|t| t.to_ascii_uppercase()).unwrap_or_else(|| "L".into());
                    let asm = agral_civil_motor::corredor::AssemblyBarragem {
                        crista: n(1, 6.0)?,
                        decl_crista: n(2, 1.0)? / 100.0,
                        montante_esquerda: !lado.starts_with('R') && !lado.starts_with('D'),
                        talude_montante: n(4, 3.0)?,
                        talude_jusante: n(5, 2.0)?,
                        altura_berma: n(6, 8.0)?,
                        largura_berma: n(7, 3.0)?,
                        talude_corte: n(8, 1.0)?,
                        alcance: 500.0,
                    };
                    self.civil_definir_assembly(i, &eixo, asm)
                })();
                self.civil_al_resultado(r);
            }
            // CREATECORRIDOR [eixo] [frequência das seções, m]
            "CREATECORRIDOR" if args.is_empty() => {
                let c = crate::modules::civil::perguntas::PerguntasCommand::com(
                    "CREATECORRIDOR",
                    "CREATECORRIDOR",
                    vec![self.civil_pergunta_eixo(i), "Section frequency (m) <5>:".into()],
                );
                self.civil_instalar(i, Box::new(c));
            }
            "CREATECORRIDOR" => {
                let r = self
                    .civil_al_escolher(i, args.first().copied())
                    .and_then(|e| num(args.get(1), 5.0).ok_or("Frequency must be a number.".to_string()).map(|p| (e, p)))
                    .and_then(|(e, p)| self.civil_criar_corredor(i, &e, p));
                self.civil_al_resultado(r);
            }
            "REBUILDCORRIDOR" | "REBUILDALIGNMENT" => {
                let r = self.civil_al_escolher(i, args.first().copied()).and_then(|n| self.civil_redesenhar_alinhamento(i, &n));
                self.civil_al_resultado(r);
            }
            // OBJECTVIEWER [exagero vertical]: os objetos selecionados em 3D num desenho à parte.
            "OBJECTVIEWER" | "_OBJECTVIEWERSEL" => {
                let ex = num(args.first(), 1.0).unwrap_or(1.0).max(0.01);
                let sel = self.tabs[i].scene.selected_handles_in_order();
                if sel.is_empty() && verbo == "OBJECTVIEWER" {
                    use crate::command::CadCommand;
                    use crate::modules::draw::select::SelectObjectsCommand;
                    let c = SelectObjectsCommand::filtered("OBJECTVIEWER", &format!("_OBJECTVIEWERSEL {ex}"), "Select objects to view in 3D:", |_| true);
                    self.command_line.push_info(&c.prompt());
                    self.tabs[i].active_cmd = Some(Box::new(c));
                } else {
                    self.tabs[i].scene.deselect_all();
                    match self.civil_object_viewer(i, &sel, ex) {
                        Ok(t) => return Some(t),
                        Err(e) => self.command_line.push_error(&e),
                    }
                }
            }
            "DELETEALIGNMENT" => {
                let r = match args.first() {
                    Some(n) => self.civil_apagar_alinhamento(i, n),
                    None => Err("Usage: DELETEALIGNMENT alignment".into()),
                };
                self.civil_al_resultado(r);
            }
            // Versões de linha de comando, que perguntam cada valor.
            "ALIGNMENTSTATIONS" if args.is_empty() => {
                let c = crate::modules::civil::perguntas::PerguntasCommand::com(
                    "ALIGNMENTSTATIONS",
                    "ALIGNMENTSTATIONS",
                    vec![self.civil_pergunta_eixo(i), "Start station (m) <0>:".into(), "Station label interval (m) <20>:".into()],
                );
                self.civil_instalar(i, Box::new(c));
            }
            _ => return None,
        }
        Some(Task::none())
    }
}
