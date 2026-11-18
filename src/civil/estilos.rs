//! Estilos de superfície de fábrica da Agral (R15), os que o JP usa (18/11): curvas de 0,1, 0,5, 1, 2 e 5 m,
//! Apagado e Borda; mais Triângulos, para conferir a malha. Padrão: Curvas 1m.
//! Mestra a cada 5 intermediárias: palpite, a confirmar com o JP.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Estilo {
    pub nome: &'static str,
    pub curvas: bool,
    pub intermediaria: f64,
    pub mestra: f64,
    pub borda: bool,
    pub triangulos: bool,
}

pub const ESTILOS: &[Estilo] = &[
    Estilo { nome: "Curvas 1m", curvas: true, intermediaria: 1.0, mestra: 5.0, borda: true, triangulos: false },
    Estilo { nome: "Curvas 0.1m", curvas: true, intermediaria: 0.1, mestra: 0.5, borda: true, triangulos: false },
    Estilo { nome: "Curvas 0.5m", curvas: true, intermediaria: 0.5, mestra: 2.5, borda: true, triangulos: false },
    Estilo { nome: "Curvas 2m", curvas: true, intermediaria: 2.0, mestra: 10.0, borda: true, triangulos: false },
    Estilo { nome: "Curvas 5m", curvas: true, intermediaria: 5.0, mestra: 25.0, borda: true, triangulos: false },
    Estilo { nome: "Apagado", curvas: false, intermediaria: 1.0, mestra: 5.0, borda: false, triangulos: false },
    Estilo { nome: "Borda", curvas: false, intermediaria: 1.0, mestra: 5.0, borda: true, triangulos: false },
    Estilo { nome: "Triângulos", curvas: false, intermediaria: 1.0, mestra: 5.0, borda: true, triangulos: true },
];

pub fn estilo(nome: &str) -> &'static Estilo {
    ESTILOS.iter().find(|e| e.nome.eq_ignore_ascii_case(nome)).unwrap_or(&ESTILOS[0])
}

pub fn nomes() -> Vec<String> {
    ESTILOS.iter().map(|e| e.nome.to_string()).collect()
}

/// Cotas das curvas entre `zmin` e `zmax`, cada uma marcada se é mestra.
pub fn cotas(e: &Estilo, zmin: f64, zmax: f64) -> Vec<(f64, bool)> {
    if e.intermediaria <= 0.0 || zmax < zmin {
        return Vec::new();
    }
    let primeiro = (zmin / e.intermediaria).ceil() as i64;
    let ultimo = (zmax / e.intermediaria).floor() as i64;
    (primeiro..=ultimo)
        .map(|k| {
            let z = k as f64 * e.intermediaria;
            let razao = z / e.mestra;
            (z, (razao - razao.round()).abs() < 1e-6)
        })
        .collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn cotas_de_1_em_1_com_mestra_de_5() {
        let c = cotas(estilo("Curvas 1m"), 878.6, 891.2);
        assert_eq!(c.first(), Some(&(879.0, false)));
        assert_eq!(c.last(), Some(&(891.0, false)));
        let mestras: Vec<f64> = c.iter().filter(|x| x.1).map(|x| x.0).collect();
        assert_eq!(mestras, vec![880.0, 885.0, 890.0]);
    }

    #[test]
    fn curvas_de_0_1_nao_erram_a_mestra_por_arredondamento() {
        let c = cotas(estilo("Curvas 0.1m"), 900.0, 901.0);
        assert_eq!(c.len(), 11);
        let mestras: Vec<f64> = c.iter().filter(|x| x.1).map(|x| x.0).collect();
        assert_eq!(mestras.len(), 3, "{mestras:?}");
    }

    #[test]
    fn apagado_nao_desenha_nada() {
        let e = estilo("Apagado");
        assert!(!e.curvas && !e.borda && !e.triangulos);
    }

    #[test]
    fn estilo_desconhecido_cai_no_padrao() {
        assert_eq!(estilo("não existe").nome, "Curvas 1m");
    }
}

/// Camadas dos componentes da superfície, as do template da Agral (`TEMPLATE_JP.dwt`, 18/11): nome, cor (ACI) e
/// espessura em centésimos de mm (0 = padrão). Num desenho sem essas camadas, são criadas assim.
pub const CAMADA_MESTRA: (&str, u8, i16) = ("CURVA MESTRA-AS", 2, 15);
pub const CAMADA_INTERMEDIARIA: (&str, u8, i16) = ("CURVA INTERMEDIÁRIA-AS", 252, 9);
pub const CAMADA_BORDA: (&str, u8, i16) = ("BORDA SUPERFÍCIE-AS", 2, 0);
pub const CAMADA_TRIANGULOS: (&str, u8, i16) = ("TRIANGULAÇÃO-AS", 139, 9);
pub const CAMADAS: [(&str, u8, i16); 4] = [CAMADA_MESTRA, CAMADA_INTERMEDIARIA, CAMADA_BORDA, CAMADA_TRIANGULOS];
