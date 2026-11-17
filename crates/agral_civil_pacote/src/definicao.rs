//! R5 e R7: a superfície e a lista de definição gravadas no DWG, num XRECORD do dicionário
//! `AGRAL_CIVIL/SURFACES` (um por superfície). As fontes são referências (handles) às entidades do desenho.

use ocs_plugin_api::host::{Handle, XRecordEntry, XRecordValue};

pub const DICIONARIO: [&str; 2] = ["AGRAL_CIVIL", "SURFACES"];
const MARCA: &str = "AGRAL_SURFACE";
const VERSAO: i16 = 1;

#[derive(Clone, Debug, PartialEq)]
pub struct ItemDwg {
    /// "CONTOURS" (mais tipos nas próximas entregas).
    pub tipo: String,
    pub ligado: bool,
    pub fontes: Vec<Handle>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SuperficieDwg {
    pub nome: String,
    pub descricao: String,
    pub comprimento_maximo: Option<f64>,
    pub itens: Vec<ItemDwg>,
    /// Entidades desenhadas pela última montagem (malhas e marcas de diagnóstico).
    pub desenho: Vec<Handle>,
    pub desatualizada: bool,
}

impl SuperficieDwg {
    pub fn para_xrecord(&self) -> Vec<XRecordEntry> {
        let mut e = vec![
            XRecordEntry::string(1, MARCA),
            XRecordEntry::int16(70, VERSAO),
            XRecordEntry::string(300, self.descricao.clone()),
            XRecordEntry::double(40, self.comprimento_maximo.unwrap_or(0.0)),
            XRecordEntry::int16(71, self.desatualizada as i16),
            XRecordEntry::new(90, XRecordValue::Int32(self.itens.len() as i32)),
        ];
        for it in &self.itens {
            e.push(XRecordEntry::string(2, it.tipo.clone()));
            e.push(XRecordEntry::int16(72, it.ligado as i16));
            e.push(XRecordEntry::new(91, XRecordValue::Int32(it.fontes.len() as i32)));
            e.extend(it.fontes.iter().map(|h| XRecordEntry::new(330, XRecordValue::Handle(*h))));
        }
        e.push(XRecordEntry::new(92, XRecordValue::Int32(self.desenho.len() as i32)));
        e.extend(self.desenho.iter().map(|h| XRecordEntry::new(331, XRecordValue::Handle(*h))));
        e
    }

    pub fn de_xrecord(nome: &str, e: &[XRecordEntry]) -> Result<Self, String> {
        let mut it = e.iter();
        let mut prox = |codigo: i32| -> Result<&XRecordValue, String> {
            let x = it.next().ok_or_else(|| format!("superfície {nome}: registro curto"))?;
            if x.code != codigo {
                return Err(format!("superfície {nome}: esperava o código {codigo}, veio {}", x.code));
            }
            Ok(&x.value)
        };
        let texto = |v: &XRecordValue| match v {
            XRecordValue::String(s) => Ok(s.clone()),
            _ => Err("texto esperado".to_string()),
        };
        let inteiro = |v: &XRecordValue| match v {
            XRecordValue::Int16(i) => Ok(*i as i64),
            XRecordValue::Int32(i) => Ok(*i as i64),
            XRecordValue::Int64(i) => Ok(*i),
            _ => Err("inteiro esperado".to_string()),
        };
        let handle = |v: &XRecordValue| match v {
            XRecordValue::Handle(h) => Ok(*h),
            _ => Err("handle esperado".to_string()),
        };
        if texto(prox(1)?)? != MARCA {
            return Err(format!("{nome} não é uma superfície do Agral Civil"));
        }
        let versao = inteiro(prox(70)?)?;
        if versao > VERSAO as i64 {
            return Err(format!("superfície {nome} gravada por versão mais nova do Agral Civil ({versao})"));
        }
        let descricao = texto(prox(300)?)?;
        let cm = match prox(40)? {
            XRecordValue::Double(d) => *d,
            _ => return Err("número esperado".into()),
        };
        let desatualizada = inteiro(prox(71)?)? != 0;
        let n = inteiro(prox(90)?)?;
        let mut itens = Vec::new();
        for _ in 0..n {
            let tipo = texto(prox(2)?)?;
            let ligado = inteiro(prox(72)?)? != 0;
            let k = inteiro(prox(91)?)?;
            let mut fontes = Vec::with_capacity(k as usize);
            for _ in 0..k {
                fontes.push(handle(prox(330)?)?);
            }
            itens.push(ItemDwg { tipo, ligado, fontes });
        }
        let k = inteiro(prox(92)?)?;
        let mut desenho = Vec::new();
        for _ in 0..k {
            desenho.push(handle(prox(331)?)?);
        }
        Ok(SuperficieDwg {
            nome: nome.to_string(),
            descricao,
            comprimento_maximo: (cm > 0.0).then_some(cm),
            itens,
            desenho,
            desatualizada,
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ida_e_volta_pelo_xrecord_devolve_a_mesma_superficie() {
        let s = SuperficieDwg {
            nome: "TN".into(),
            descricao: "Terreno natural".into(),
            comprimento_maximo: Some(50.0),
            itens: vec![
                ItemDwg { tipo: "CONTOURS".into(), ligado: true, fontes: vec![Handle::new(0x2A), Handle::new(0x2B)] },
                ItemDwg { tipo: "CONTOURS".into(), ligado: false, fontes: vec![] },
            ],
            desenho: vec![Handle::new(0x99)],
            desatualizada: true,
        };
        let e = s.para_xrecord();
        assert_eq!(SuperficieDwg::de_xrecord("TN", &e).unwrap(), s);
    }

    #[test]
    fn superficie_vazia_tambem_volta() {
        let s = SuperficieDwg { nome: "S1".into(), ..Default::default() };
        assert_eq!(SuperficieDwg::de_xrecord("S1", &s.para_xrecord()).unwrap(), s);
    }

    #[test]
    fn registro_que_nao_e_nosso_da_erro_claro() {
        let e = vec![XRecordEntry::string(1, "OUTRA_COISA")];
        assert!(SuperficieDwg::de_xrecord("X", &e).unwrap_err().contains("não é uma superfície"));
    }
}
