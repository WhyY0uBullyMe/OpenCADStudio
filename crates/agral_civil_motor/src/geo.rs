//! Geografia (spec 005): UTM no elipsoide GRS80/WGS84 (SIRGAS 2000 na prática é o mesmo, diferença de
//! centímetros), pelas séries de Krüger (erro de milímetros dentro do fuso), e a grade de tiles XYZ (Web Mercator).

use std::f64::consts::PI;

const A: f64 = 6_378_137.0;
const F: f64 = 1.0 / 298.257_222_101; // GRS80 (SIRGAS 2000)
const K0: f64 = 0.9996;
const E0: f64 = 500_000.0;

/// Fuso UTM: número 1 a 60 e hemisfério sul (norte falso 10 000 000 m).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fuso {
    pub numero: u8,
    pub sul: bool,
}

impl Fuso {
    pub fn meridiano_central(&self) -> f64 {
        (self.numero as f64) * 6.0 - 183.0
    }
    fn n0(&self) -> f64 {
        if self.sul { 10_000_000.0 } else { 0.0 }
    }
    /// Fuso de uma longitude (graus).
    pub fn da_longitude(lon: f64, sul: bool) -> Fuso {
        Fuso { numero: (((lon + 180.0) / 6.0).floor() as i32 + 1).clamp(1, 60) as u8, sul }
    }
}

fn constantes() -> (f64, f64, [f64; 3], [f64; 3], [f64; 3]) {
    let n = F / (2.0 - F);
    let (n2, n3) = (n * n, n * n * n);
    let a_ret = A / (1.0 + n) * (1.0 + n2 / 4.0 + n2 * n2 / 64.0);
    let alfa = [n / 2.0 - 2.0 * n2 / 3.0 + 5.0 * n3 / 16.0, 13.0 * n2 / 48.0 - 3.0 * n3 / 5.0, 61.0 * n3 / 240.0];
    let beta = [n / 2.0 - 2.0 * n2 / 3.0 + 37.0 * n3 / 96.0, n2 / 48.0 + n3 / 15.0, 17.0 * n3 / 480.0];
    let delta = [2.0 * n - 2.0 * n2 / 3.0 - 2.0 * n3, 7.0 * n2 / 3.0 - 8.0 * n3 / 5.0, 56.0 * n3 / 15.0];
    (n, a_ret, alfa, beta, delta)
}

/// Latitude e longitude (graus) para leste e norte UTM no fuso.
pub fn para_utm(lat: f64, lon: f64, fuso: Fuso) -> (f64, f64) {
    let (n, a_ret, alfa, _, _) = constantes();
    let phi = lat.to_radians();
    let dl = (lon - fuso.meridiano_central()).to_radians();
    let c = 2.0 * n.sqrt() / (1.0 + n);
    let t = (phi.sin().atanh() - c * (c * phi.sin()).atanh()).sinh();
    let xi = (t / dl.cos()).atan();
    let eta = (dl.sin() / (1.0 + t * t).sqrt()).atanh();
    let (mut e, mut nn) = (eta, xi);
    for (j, a) in alfa.iter().enumerate() {
        let k = 2.0 * (j as f64 + 1.0);
        e += a * (k * xi).cos() * (k * eta).sinh();
        nn += a * (k * xi).sin() * (k * eta).cosh();
    }
    (E0 + K0 * a_ret * e, fuso.n0() + K0 * a_ret * nn)
}

/// Leste e norte UTM no fuso para latitude e longitude (graus).
pub fn de_utm(leste: f64, norte: f64, fuso: Fuso) -> (f64, f64) {
    let (_, a_ret, _, beta, delta) = constantes();
    let xi = (norte - fuso.n0()) / (K0 * a_ret);
    let eta = (leste - E0) / (K0 * a_ret);
    let (mut xl, mut el) = (xi, eta);
    for (j, b) in beta.iter().enumerate() {
        let k = 2.0 * (j as f64 + 1.0);
        xl -= b * (k * xi).sin() * (k * eta).cosh();
        el -= b * (k * xi).cos() * (k * eta).sinh();
    }
    let chi = (xl.sin() / el.cosh()).asin();
    let mut phi = chi;
    for (j, d) in delta.iter().enumerate() {
        phi += d * (2.0 * (j as f64 + 1.0) * chi).sin();
    }
    let lon = fuso.meridiano_central() + (el.sinh() / xl.cos()).atan().to_degrees();
    (phi.to_degrees(), lon)
}

/// Posição no mundo Web Mercator em tiles do zoom `z` (x para leste, y para baixo; inteiro = o tile).
pub fn em_tiles(lat: f64, lon: f64, z: u8) -> (f64, f64) {
    let n = (1u64 << z) as f64;
    let phi = lat.clamp(-85.051_128_78, 85.051_128_78).to_radians();
    let x = (lon + 180.0) / 360.0 * n;
    let y = (1.0 - (phi.tan() + 1.0 / phi.cos()).ln() / PI) / 2.0 * n;
    (x, y)
}

/// Latitude e longitude de uma posição em tiles do zoom `z`.
pub fn de_tiles(x: f64, y: f64, z: u8) -> (f64, f64) {
    let n = (1u64 << z) as f64;
    let lon = x / n * 360.0 - 180.0;
    let lat = (PI * (1.0 - 2.0 * y / n)).sinh().atan().to_degrees();
    (lat, lon)
}

/// Tamanho do pixel no chão (m) num tile de 256 px no zoom `z`, na latitude.
pub fn metros_por_pixel(lat: f64, z: u8) -> f64 {
    2.0 * PI * A * lat.to_radians().cos() / (256.0 * (1u64 << z) as f64)
}

/// Endereço do tile: troca `{x}`, `{y}`, `{z}` (e `{-y}`, a linha contada de baixo, como no TMS).
pub fn endereco(modelo: &str, x: u32, y: u32, z: u8) -> String {
    let n = (1u32 << z) - 1;
    modelo.replace("{x}", &x.to_string()).replace("{-y}", &(n - y).to_string()).replace("{y}", &y.to_string()).replace("{z}", &z.to_string())
}

/// Plano da imagem de uma área UTM: zoom dos tiles, tamanho em pixels e os tiles que ela usa.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanoImagem {
    pub z: u8,
    pub largura: u32,
    pub altura: u32,
    /// Tiles (x, y) que cobrem a área.
    pub tiles: Vec<(u32, u32)>,
}

/// Escolhe o zoom pela resolução: a mais fina que cabe em `max_px` pixels no lado maior e em `max_tiles`
/// tiles, sem passar de `z_max`. `None` se a área for vazia.
pub fn planejar(e0: f64, n0: f64, e1: f64, n1: f64, fuso: Fuso, max_px: u32, max_tiles: usize, z_max: u8) -> Option<PlanoImagem> {
    let (w, h) = ((e1 - e0).abs(), (n1 - n0).abs());
    if !(w > 0.0 && h > 0.0) {
        return None;
    }
    let (e0, e1, n0, n1) = (e0.min(e1), e0.max(e1), n0.min(n1), n0.max(n1));
    let (lat_c, _) = de_utm((e0 + e1) / 2.0, (n0 + n1) / 2.0, fuso);
    let mut z = z_max;
    loop {
        let res = metros_por_pixel(lat_c, z).max(w.max(h) / max_px as f64);
        let (largura, altura) = ((w / res).ceil().max(1.0) as u32, (h / res).ceil().max(1.0) as u32);
        let mut lo = (f64::INFINITY, f64::INFINITY);
        let mut hi = (f64::NEG_INFINITY, f64::NEG_INFINITY);
        for (e, n) in [(e0, n0), (e1, n0), (e0, n1), (e1, n1), ((e0 + e1) / 2.0, n0), ((e0 + e1) / 2.0, n1)] {
            let (lat, lon) = de_utm(e, n, fuso);
            let (x, y) = em_tiles(lat, lon, z);
            lo = (lo.0.min(x), lo.1.min(y));
            hi = (hi.0.max(x), hi.1.max(y));
        }
        let tiles: Vec<(u32, u32)> = (lo.0.floor() as u32..=hi.0.floor() as u32)
            .flat_map(|x| (lo.1.floor() as u32..=hi.1.floor() as u32).map(move |y| (x, y)))
            .collect();
        if tiles.len() <= max_tiles || z == 0 {
            return Some(PlanoImagem { z, largura, altura, tiles });
        }
        z -= 1;
    }
}

/// Monta a imagem RGBA (linha de cima primeiro) da área UTM, reprojetando cada pixel de Web Mercator: fica no
/// lugar certo do desenho, sem giro. `tile(x, y)` devolve o tile 256 x 256 em RGBA (ou `None`: fica transparente).
pub fn montar_imagem(
    e0: f64,
    n0: f64,
    e1: f64,
    n1: f64,
    fuso: Fuso,
    plano: &PlanoImagem,
    mut tile: impl FnMut(u32, u32) -> Option<Vec<u8>>,
) -> Vec<u8> {
    let (e0, e1, n0, n1) = (e0.min(e1), e0.max(e1), n0.min(n1), n0.max(n1));
    let (w, h) = (plano.largura as usize, plano.altura as usize);
    let mut cache: std::collections::HashMap<(u32, u32), Option<Vec<u8>>> = Default::default();
    let mut out = vec![0u8; w * h * 4];
    for j in 0..h {
        let n = n1 - (j as f64 + 0.5) * (n1 - n0) / h as f64;
        for i in 0..w {
            let e = e0 + (i as f64 + 0.5) * (e1 - e0) / w as f64;
            let (lat, lon) = de_utm(e, n, fuso);
            let (x, y) = em_tiles(lat, lon, plano.z);
            let (tx, ty) = (x.floor() as u32, y.floor() as u32);
            let t = cache.entry((tx, ty)).or_insert_with(|| tile(tx, ty).filter(|b| b.len() >= 256 * 256 * 4));
            if let Some(b) = t {
                let px = (((x - tx as f64) * 256.0) as usize).min(255);
                let py = (((y - ty as f64) * 256.0) as usize).min(255);
                let k = (py * 256 + px) * 4;
                out[(j * w + i) * 4..(j * w + i) * 4 + 4].copy_from_slice(&b[k..k + 4]);
            }
        }
    }
    out
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Arco de meridiano por integração numérica (Simpson), independente das séries.
    fn arco(lat: f64) -> f64 {
        let e2 = F * (2.0 - F);
        let m = |p: f64| A * (1.0 - e2) / (1.0 - e2 * p.sin().powi(2)).powf(1.5);
        let (n, b) = (20_000, lat.to_radians());
        let h = b / n as f64;
        let mut s = m(0.0) + m(b);
        for k in 1..n {
            s += m(k as f64 * h) * if k % 2 == 1 { 4.0 } else { 2.0 };
        }
        s * h / 3.0
    }

    #[test]
    fn no_meridiano_central_o_norte_e_o_arco() {
        let f = Fuso { numero: 22, sul: true };
        let (e, n) = para_utm(-16.0, -51.0, f);
        assert!((e - 500_000.0).abs() < 1e-6);
        assert!((n - (10_000_000.0 - K0 * arco(16.0))).abs() < 0.001, "{n}");
        let (_, n45) = para_utm(45.0, -51.0, Fuso { numero: 22, sul: false });
        assert!((n45 - K0 * arco(45.0)).abs() < 0.001);
    }

    #[test]
    fn ida_e_volta_abaixo_de_um_milimetro() {
        let f = Fuso { numero: 22, sul: true };
        for (lat, lon) in [(-16.6869, -49.2648), (-10.0, -53.9), (-25.4, -48.1), (-1.0, -51.0)] {
            let (e, n) = para_utm(lat, lon, f);
            let (lat2, lon2) = de_utm(e, n, f);
            let (e2, n2) = para_utm(lat2, lon2, f);
            assert!((e - e2).abs() < 1e-3 && (n - n2).abs() < 1e-3, "{lat},{lon}");
            assert!((lat - lat2).abs() < 1e-8 && (lon - lon2).abs() < 1e-8);
        }
        assert_eq!(Fuso::da_longitude(-49.26, true).numero, 22);
        assert_eq!(Fuso::da_longitude(-47.9, true).numero, 23);
    }

    #[test]
    fn grade_de_tiles() {
        assert_eq!(em_tiles(0.0, 0.0, 0), (0.5, 0.5));
        let (x, y) = em_tiles(-16.6869, -49.2648, 1);
        assert_eq!((x as u32, y as u32), (0, 1), "sudoeste no zoom 1");
        let (x, y) = em_tiles(-16.6869, -49.2648, 15);
        let (lat, lon) = de_tiles(x, y, 15);
        assert!((lat + 16.6869).abs() < 1e-9 && (lon + 49.2648).abs() < 1e-9);
        // Conta à mão: x = 130,7352 / 360 · 32768 = 11899,8; y = (1 + 0,094022) / 2 · 32768 = 17924,5.
        assert_eq!((x as u32, y as u32), (11899, 17924));
        assert_eq!(endereco("https://t/{z}/{x}/{y}.png", 3, 5, 4), "https://t/4/3/5.png");
        assert_eq!(endereco("{z}/{x}/{-y}", 3, 5, 4), "4/3/10");
        assert!((metros_por_pixel(0.0, 0) - 156_543.03).abs() < 0.01);
    }

    /// Sem rede: tiles sintéticos, vermelho no tile de um ponto conhecido e azul nos outros; o pixel do ponto cai
    /// no lugar UTM certo da imagem.
    #[test]
    fn imagem_reprojetada_no_lugar_certo() {
        let f = Fuso { numero: 22, sul: true };
        let (ep, np) = para_utm(-16.6869, -49.2648, f);
        let (e0, n0, e1, n1) = (ep - 300.0, np - 200.0, ep + 300.0, np + 200.0);
        let plano = planejar(e0, n0, e1, n1, f, 512, 64, 17).unwrap();
        assert!(plano.largura <= 512 && plano.tiles.len() <= 64, "{plano:?}");
        let (xp, yp) = em_tiles(-16.6869, -49.2648, plano.z);
        let alvo = (xp.floor() as u32, yp.floor() as u32);
        let img = montar_imagem(e0, n0, e1, n1, f, &plano, |x, y| {
            let cor = if (x, y) == alvo { [255u8, 0, 0, 255] } else { [0u8, 0, 255, 255] };
            Some(cor.iter().copied().cycle().take(256 * 256 * 4).collect())
        });
        let (w, h) = (plano.largura as usize, plano.altura as usize);
        // O ponto fica no centro da área.
        let k = ((h / 2) * w + w / 2) * 4;
        assert_eq!(&img[k..k + 4], &[255, 0, 0, 255]);
        // O canto de cima à esquerda (300 m a oeste, 200 m ao norte) está fora do tile do ponto no zoom escolhido.
        assert_eq!(&img[0..4], &[0, 0, 255, 255]);
    }
}
