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

    fn civil_instalar(&mut self, i: usize, c: Box<dyn crate::command::CadCommand>) {
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

    fn civil_pergunta_superficie(&self, i: usize, eixo: Option<&str>) -> String {
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
            "ALIGNMENTSTATIONS" => {
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
            "_SETPROFILE" => {
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
                self.civil_al_resultado(r);
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
                            .civil_definir_secoes(i, &eixo, Secoes { intervalo: iv, esquerda: esq, direita: dir, origem: o, colunas: col })
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
            "REBUILDALIGNMENT" => {
                let r = self.civil_al_escolher(i, args.first().copied()).and_then(|n| self.civil_redesenhar_alinhamento(i, &n));
                self.civil_al_resultado(r);
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
            "-CREATEPROFILEVIEW" | "-CREATESECTIONS" | "-ALIGNMENTSTATIONS" => {
                use crate::modules::civil::perguntas::PerguntasCommand;
                let c = match verbo {
                    "-CREATEPROFILEVIEW" => PerguntasCommand::new(
                        "-CREATEPROFILEVIEW",
                        "CREATEPROFILEVIEW",
                        &["Alignment name:", "Surface name:", "Vertical exaggeration <10>:"],
                    ),
                    "-CREATESECTIONS" => PerguntasCommand::new(
                        "-CREATESECTIONS",
                        "CREATESECTIONS",
                        &["Alignment name:", "Interval (m) <20>:", "Left width (m) <30>:", "Right width (m) <30>:", "Columns <5>:"],
                    ),
                    _ => PerguntasCommand::new(
                        "-ALIGNMENTSTATIONS",
                        "ALIGNMENTSTATIONS",
                        &["Alignment name:", "Start station (m) <0>:", "Station label interval (m) <20>:"],
                    ),
                };
                use crate::command::CadCommand;
                self.command_line.push_info(&c.prompt());
                self.tabs[i].active_cmd = Some(Box::new(c));
            }
            _ => return None,
        }
        Some(Task::none())
    }
}
