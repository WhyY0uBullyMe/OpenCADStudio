//! R9: arquivos de pontos de texto (PNEZD, PENZD, NEZ, ENZ).
//!
//! Separador: vírgula, ponto e vírgula, tabulação ou espaço. Decimal com ponto ou com vírgula (vírgula decimal só
//! quando o separador não é vírgula). Linha que não lê vai para `linhas_ruins`, com número e texto, nunca some.

use crate::P3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatoPontos {
    /// Número, Norte, Este, cota, descrição.
    Pnezd,
    /// Número, Este, Norte, cota, descrição.
    Penzd,
    /// Norte, Este, cota.
    Nez,
    /// Este, Norte, cota.
    Enz,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PontoLevantado {
    pub numero: Option<String>,
    pub xyz: P3,
    pub descricao: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinhaRuim {
    /// Número da linha no arquivo, começando em 1.
    pub linha: usize,
    pub texto: String,
    pub motivo: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LeituraPontos {
    pub pontos: Vec<PontoLevantado>,
    pub linhas_ruins: Vec<LinhaRuim>,
}

pub fn ler_arquivo_de_pontos(texto: &str, formato: FormatoPontos) -> LeituraPontos {
    let mut r = LeituraPontos::default();
    for (i, bruta) in texto.lines().enumerate() {
        let linha = bruta.trim().trim_start_matches('\u{feff}');
        if linha.is_empty() {
            continue;
        }
        match ler_linha(linha, formato) {
            Ok(p) => r.pontos.push(p),
            Err(motivo) => r.linhas_ruins.push(LinhaRuim { linha: i + 1, texto: bruta.to_string(), motivo }),
        }
    }
    r
}

fn ler_linha(linha: &str, formato: FormatoPontos) -> Result<PontoLevantado, String> {
    let (com_numero, com_descricao) = match formato {
        FormatoPontos::Pnezd | FormatoPontos::Penzd => (true, true),
        FormatoPontos::Nez | FormatoPontos::Enz => (false, false),
    };
    let minimo = 3 + com_numero as usize;
    let (campos, decimal_virgula) = separar(linha, minimo + com_descricao as usize);
    if campos.len() < minimo {
        return Err(format!("esperava {minimo} colunas, achou {}", campos.len()));
    }
    let off = com_numero as usize;
    let num = |k: usize| -> Result<f64, String> {
        let s = campos[off + k].trim();
        let s = if decimal_virgula { s.replace(',', ".") } else { s.to_string() };
        s.parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| format!("'{}' não é número", campos[off + k].trim()))
    };
    let (a, b, z) = (num(0)?, num(1)?, num(2)?);
    let xyz = match formato {
        FormatoPontos::Pnezd | FormatoPontos::Nez => [b, a, z],
        FormatoPontos::Penzd | FormatoPontos::Enz => [a, b, z],
    };
    let numero = com_numero.then(|| campos[0].trim().to_string());
    let descricao = if com_descricao {
        campos.get(minimo).map(|d| d.trim().to_string()).filter(|d| !d.is_empty())
    } else {
        None
    };
    Ok(PontoLevantado { numero, xyz, descricao })
}

/// Separa em até `max_campos` campos; o último fica com o resto (descrição com espaço ou separador).
/// Devolve também se a vírgula é decimal.
fn separar(linha: &str, max_campos: usize) -> (Vec<&str>, bool) {
    let sep: Option<char> = if linha.contains(';') {
        Some(';')
    } else if linha.contains('\t') {
        Some('\t')
    } else if linha.contains(',') {
        Some(',')
    } else {
        None
    };
    match sep {
        Some(c) => (linha.splitn(max_campos, c).collect(), c != ','),
        None => {
            // Espaço: vários espaços seguidos contam como um.
            let mut campos = Vec::new();
            let mut resto = linha.trim_start();
            while !resto.is_empty() {
                if campos.len() + 1 == max_campos {
                    campos.push(resto.trim_end());
                    break;
                }
                let fim = resto.find(char::is_whitespace).unwrap_or(resto.len());
                campos.push(&resto[..fim]);
                resto = resto[fim..].trim_start();
            }
            (campos, true)
        }
    }
}
