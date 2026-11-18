//! Spec 005: o sistema do desenho (SIRGAS 2000 / UTM, fuso e hemisfério) e o link XYZ das imagens, num XRECORD
//! `AGRAL_CIVIL/GEO` chamado "GEO".

use document::objects::{XRecordEntry, XRecordValue};

pub const DICIONARIO: [&str; 2] = ["AGRAL_CIVIL", "GEO"];
pub const CHAVE: &str = "GEO";
const MARCA: &str = "AGRAL_GEO";

/// Serviços prontos para escolher (o link continua editável). O do Google fica fora dos termos de uso dele; o JP
/// decidiu usar para uso interno (DECISOES_E_REGRAS.md, 18/11).
pub const SERVICOS: [(&str, &str); 3] = [
    ("Google Satellite", "https://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}"),
    ("Esri World Imagery", "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}"),
    ("OpenStreetMap", "https://tile.openstreetmap.org/{z}/{x}/{y}.png"),
];

#[derive(Clone, Debug, PartialEq)]
pub struct GeoDwg {
    pub fuso: u8,
    pub sul: bool,
    pub url: String,
}

impl Default for GeoDwg {
    fn default() -> Self {
        GeoDwg { fuso: 22, sul: true, url: SERVICOS[0].1.into() }
    }
}

impl GeoDwg {
    pub fn fuso(&self) -> agral_civil_motor::geo::Fuso {
        agral_civil_motor::geo::Fuso { numero: self.fuso, sul: self.sul }
    }

    pub fn para_xrecord(&self) -> Vec<XRecordEntry> {
        vec![
            XRecordEntry::string(1, MARCA.to_string()),
            XRecordEntry::int16(70, self.fuso as i16),
            XRecordEntry::int16(71, self.sul as i16),
            XRecordEntry::string(300, self.url.clone()),
        ]
    }

    pub fn de_xrecord(e: &[XRecordEntry]) -> Option<Self> {
        if !matches!(e.first().map(|x| &x.value), Some(XRecordValue::String(s)) if s == MARCA) {
            return None;
        }
        let mut g = GeoDwg::default();
        for x in e {
            match (x.code, &x.value) {
                (70, XRecordValue::Int16(v)) => g.fuso = (*v).clamp(1, 60) as u8,
                (71, XRecordValue::Int16(v)) => g.sul = *v != 0,
                (300, XRecordValue::String(s)) => g.url = s.clone(),
                _ => {}
            }
        }
        Some(g)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn grava_e_le() {
        let g = GeoDwg { fuso: 23, sul: true, url: "https://x/{z}/{x}/{y}".into() };
        assert_eq!(GeoDwg::de_xrecord(&g.para_xrecord()), Some(g));
    }
}
