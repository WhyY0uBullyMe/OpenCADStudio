//! Spec 005: imagem de satélite por link XYZ. `GEOLOCATION` grava o fuso UTM e o link; `GEOMAP Aerial` traz a
//! imagem da área vista na tela e `GEOMAPIMAGE` a de dois cantos. O download e a reprojeção (Web Mercator para
//! UTM, pixel a pixel) rodam em segundo plano; a imagem vira um PNG ao lado do desenho e uma IMAGE no fundo.

use super::{Message, OpenCADStudio};
use crate::civil::geo::GeoDwg;
use agral_civil_motor::geo::{endereco, montar_imagem, planejar, PlanoImagem};
use document::EntityType;

pub(crate) const CAMADA_IMAGEM: (&str, u8) = ("C-IMAGEM", 7);
const MAX_PX: u32 = 4096;
const MAX_TILES: usize = 400;
const Z_MAX: u8 = 19;

/// Resultado do download, para a linha principal: (aba, área, PNG ou erro, avisos).
pub(crate) struct ImagemPronta(pub std::sync::Arc<std::sync::Mutex<Option<(u64, [f64; 4], Result<Vec<u8>, String>, String)>>>);

impl std::fmt::Debug for ImagemPronta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ImagemPronta")
    }
}

impl Clone for ImagemPronta {
    fn clone(&self) -> Self {
        ImagemPronta(self.0.clone())
    }
}

/// Download em andamento: começo, tiles prontos e o total (para o cartão de progresso).
pub(crate) struct Baixando {
    pub inicio: iced::time::Instant,
    pub feitos: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    pub total: usize,
}

/// Baixa um tile e devolve RGBA 256 x 256.
fn baixar_tile(url: &str) -> Result<Vec<u8>, String> {
    let agente = ureq::Agent::config_builder().timeout_global(Some(std::time::Duration::from_secs(20))).build().new_agent();
    let mut r = agente.get(url).header("User-Agent", "AgralCAD").call().map_err(|e| e.to_string())?;
    let bytes = r.body_mut().with_config().limit(8 * 1024 * 1024).read_to_vec().map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("not an image: {e}"))?;
    Ok(image::imageops::resize(&img.to_rgba8(), 256, 256, image::imageops::FilterType::Triangle).into_raw())
}

/// Monta a imagem (JPEG) da área (sem a linha principal): baixa os tiles do plano e reprojeta.
pub(crate) fn imagem_da_area(area: [f64; 4], geo: &GeoDwg, plano: &PlanoImagem, tile: impl FnMut(u32, u32) -> Option<Vec<u8>>) -> Result<Vec<u8>, String> {
    let rgba = montar_imagem(area[0], area[1], area[2], area[3], geo.fuso(), plano, tile);
    let img = image::RgbaImage::from_raw(plano.largura, plano.altura, rgba).ok_or("image size")?;
    // JPEG (qualidade 85): uma área de 4000 px fica com uns 2 MB; em PNG passava de 25 MB.
    let rgb = image::DynamicImage::ImageRgba8(img).to_rgb8();
    let mut jpg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpg, 85).encode_image(&rgb).map_err(|e| e.to_string())?;
    Ok(jpg)
}

impl OpenCADStudio {
    pub(crate) fn civil_geo(&self, i: usize) -> Option<GeoDwg> {
        crate::civil::geo(&self.tabs[i].scene.document)
    }

    /// `GEOLOCATION`: grava o fuso e o link.
    pub(crate) fn civil_geolocalizar(&mut self, i: usize, fuso: u8, sul: bool, url: &str) -> Result<String, String> {
        if !(1..=60).contains(&fuso) {
            return Err("UTM zone must be 1 to 60.".into());
        }
        if !(url.contains("{x}") && (url.contains("{y}") || url.contains("{-y}")) && url.contains("{z}")) {
            return Err("The tile link needs {x}, {y} and {z}.".into());
        }
        let g = GeoDwg { fuso, sul, url: url.trim().to_string() };
        let novo = self.civil_geo(i).is_none();
        self.push_undo_snapshot(i, "GEOLOCATION");
        let mut host = crate::app::plugin_host::HostSession::new(self, i);
        if novo {
            host.create_xrecord(&crate::civil::geo::DICIONARIO, crate::civil::geo::CHAVE, g.para_xrecord())?;
        } else {
            host.update_xrecord(&crate::civil::geo::DICIONARIO, crate::civil::geo::CHAVE, g.para_xrecord())?;
        }
        self.tabs[i].dirty = true;
        Ok(format!("Drawing on SIRGAS 2000 / UTM zone {fuso}{}. Use GEOMAP Aerial to bring the imagery.", if sul { "S" } else { "N" }))
    }

    /// Área vista na tela, em coordenadas do desenho (vista de cima).
    pub(crate) fn civil_area_da_tela(&self, i: usize) -> [f64; 4] {
        let cam = self.tabs[i].scene.camera.borrow();
        let (vw, vh) = self.tabs[i].scene.selection.borrow().view.vp_size;
        let h = f64::from(cam.ortho_size());
        let w = h * f64::from(vw.max(1.0)) / f64::from(vh.max(1.0));
        let c = cam.target;
        [c.x - w, c.y - h, c.x + w, c.y + h]
    }

    /// `GEOMAP Aerial` / `GEOMAPIMAGE`: planeja e baixa em segundo plano. Com `civil_assincrono` desligado (testes,
    /// lote), não baixa nada: só devolve o plano.
    pub(crate) fn civil_geomap(&mut self, i: usize, area: [f64; 4]) -> Result<String, String> {
        let geo = self.civil_geo(i).ok_or("Set the drawing's coordinate system first: GEOLOCATION.")?;
        let plano = planejar(area[0], area[1], area[2], area[3], geo.fuso(), MAX_PX, MAX_TILES, Z_MAX)
            .ok_or("Empty area.")?;
        let resumo = format!(
            "{} x {} px, zoom {}, {} tiles",
            plano.largura,
            plano.altura,
            plano.z,
            plano.tiles.len()
        );
        if !self.civil_assincrono {
            return Ok(format!("Imagery planned: {resumo}."));
        }
        if self.civil_baixando.is_some() {
            return Err("Imagery is already being downloaded; wait for it to finish.".into());
        }
        let tab = self.tabs[i].id;
        let feitos = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        self.civil_baixando = Some(Baixando { inicio: iced::time::Instant::now(), feitos: feitos.clone(), total: plano.tiles.len() });
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            // Baixa os tiles em 8 linhas ao mesmo tempo, depois monta.
            let fila = std::sync::Mutex::new(plano.tiles.clone());
            let prontos: std::sync::Mutex<std::collections::HashMap<(u32, u32), Result<Vec<u8>, String>>> = Default::default();
            std::thread::scope(|sc| {
                for _ in 0..8 {
                    sc.spawn(|| loop {
                        let Some((x, y)) = fila.lock().ok().and_then(|mut f| f.pop()) else { break };
                        let r = baixar_tile(&endereco(&geo.url, x, y, plano.z));
                        feitos.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if let Ok(mut p) = prontos.lock() {
                            p.insert((x, y), r);
                        }
                    });
                }
            });
            let mut prontos = prontos.into_inner().unwrap_or_default();
            let falhas = prontos.values().filter(|r| r.is_err()).count();
            let ultimo_erro = prontos.values().find_map(|r| r.as_ref().err().cloned()).unwrap_or_default();
            let r = imagem_da_area(area, &geo, &plano, |x, y| prontos.remove(&(x, y)).and_then(|r| r.ok()));
            let aviso = if falhas > 0 { format!("{falhas} tiles failed ({ultimo_erro}).") } else { String::new() };
            let r = if falhas == plano.tiles.len() { Err(format!("No tile could be downloaded: {ultimo_erro}")) } else { r };
            let _ = tx.send(ImagemPronta(std::sync::Arc::new(std::sync::Mutex::new(Some((tab, area, r, aviso))))));
        });
        self.civil_tarefas.push(iced::Task::perform(async move { rx.await.ok() }, |r| match r {
            Some(m) => Message::CivilImagemPronta(m),
            // Linha de download perdida: libera o cartão.
            None => Message::CivilImagemPronta(ImagemPronta(std::sync::Arc::new(std::sync::Mutex::new(None)))),
        }));
        Ok(format!("Downloading imagery ({resumo})... you can keep working."))
    }

    /// O download terminou: grava a imagem e põe a imagem no fundo.
    pub(crate) fn civil_imagem_pronta(&mut self, p: ImagemPronta) -> iced::Task<Message> {
        self.civil_baixando = None;
        let Some((tab, area, r, aviso)) = p.0.lock().ok().and_then(|mut g| g.take()) else { return iced::Task::none() };
        let Some(i) = self.tabs.iter().position(|t| t.id == tab) else { return iced::Task::none() };
        match r.and_then(|bytes| self.civil_inserir_imagem(i, area, &bytes)) {
            Ok(m) => self.command_line.push_output(&format!("{m} {aviso}")),
            Err(e) => self.command_line.push_error(&format!("Imagery not brought: {e}")),
        }
        iced::Task::none()
    }

    /// Grava a imagem (pasta `<desenho>_imagens` ao lado do DWG, ou a pasta do programa) e insere a IMAGE no fundo.
    pub(crate) fn civil_inserir_imagem(&mut self, i: usize, area: [f64; 4], bytes: &[u8]) -> Result<String, String> {
        let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
        let (pw, ph) = (img.width(), img.height());
        let pasta = match &self.tabs[i].current_path {
            Some(p) => {
                let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "desenho".into());
                p.with_file_name(format!("{stem}_imagens"))
            }
            None => crate::config::config_dir().ok_or("no folder for the image")?.join("imagens"),
        };
        std::fs::create_dir_all(&pasta).map_err(|e| e.to_string())?;
        let nome = format!("geomap_{}.jpg", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis()));
        let caminho = pasta.join(nome);
        std::fs::write(&caminho, bytes).map_err(|e| e.to_string())?;
        let scene = &mut self.tabs[i].scene;
        if !scene.document.layers.contains(CAMADA_IMAGEM.0) {
            scene.ensure_layer(CAMADA_IMAGEM.0);
            if let Some(l) = scene.document.layers.get_mut(CAMADA_IMAGEM.0) {
                l.color = document::Color::Index(CAMADA_IMAGEM.1);
            }
        }
        let (e0, n0, e1, n1) = (area[0].min(area[2]), area[1].min(area[3]), area[0].max(area[2]), area[1].max(area[3]));
        let mut r = document::entities::RasterImage::with_size(
            &caminho.to_string_lossy(),
            document::types::Vector3::new(e0, n0, 0.0),
            pw as f64,
            ph as f64,
            e1 - e0,
            n1 - n0,
        );
        r.flags = document::entities::ImageDisplayFlags::SHOW_IMAGE | document::entities::ImageDisplayFlags::SHOW_NOT_ALIGNED;
        r.common.layer = CAMADA_IMAGEM.0.into();
        let mut e = EntityType::RasterImage(r);
        crate::app::civil::marcar(&mut e, "GEOIMAGE:");
        self.push_undo_snapshot(i, "GEOMAP");
        let hs = self.tabs[i].scene.add_entities(vec![e]);
        let antes = self.tabs[i].scene.selected_handles_in_order();
        self.tabs[i].scene.replace_selection(hs.iter().copied().collect());
        let _ = self.dispatch_command("DRAWORDER BACK");
        self.tabs[i].scene.replace_selection(antes.into_iter().collect());
        self.tabs[i].dirty = true;
        Ok(format!("Imagery placed ({pw} x {ph} px) on layer {}, saved in {}.", CAMADA_IMAGEM.0, caminho.display()))
    }

    /// `GEOMAP Off`: apaga as imagens trazidas (os PNG ficam na pasta).
    pub(crate) fn civil_geomap_off(&mut self, i: usize) -> Result<String, String> {
        let hs: Vec<_> = self.tabs[i]
            .scene
            .document
            .entities()
            .filter(|e| crate::app::civil::marca_da_entidade(e).is_some_and(|(t, _)| t == "GEOIMAGE"))
            .map(|e| e.common().handle)
            .collect();
        if hs.is_empty() {
            return Ok("No imagery to remove.".into());
        }
        self.push_undo_snapshot(i, "GEOMAP");
        self.tabs[i].scene.erase_entities(&hs);
        self.tabs[i].dirty = true;
        Ok(format!("{} imagery removed (undo brings it back).", hs.len()))
    }
}

#[cfg(test)]
mod testes {
    /// Com rede (fica fora da suíte normal): baixa um tile de Goiânia e monta 256 x 256 RGBA.
    #[test]
    #[ignore]
    fn baixa_um_tile_de_verdade() {
        for url in [
            "https://mt1.google.com/vt/lyrs=s&x=11899&y=17924&z=15",
            "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/15/17924/11899",
        ] {
            let t = super::baixar_tile(url).expect("tile");
            assert_eq!(t.len(), 256 * 256 * 4);
            assert!(t.chunks(4).any(|p| p[0] != t[0] || p[1] != t[1]), "imagem com conteúdo");
        }
    }
}
