//! Estilos de superfície de fábrica da Agral (R15). Como no Civil 3D, o estilo padrão mostra curvas de nível
//! (intermediárias e mestras) e a borda; os triângulos só aparecem quando o estilo pede.
//! Intervalos 1 m e 5 m: palpite até o JP passar o padrão Agral.

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
    Estilo { nome: "Agral - Standard", curvas: true, intermediaria: 1.0, mestra: 5.0, borda: true, triangulos: false },
    Estilo { nome: "Agral - Contours 0.5m and 2.5m", curvas: true, intermediaria: 0.5, mestra: 2.5, borda: true, triangulos: false },
    Estilo { nome: "Agral - Contours and Triangles", curvas: true, intermediaria: 1.0, mestra: 5.0, borda: true, triangulos: true },
    Estilo { nome: "Agral - Triangles", curvas: false, intermediaria: 1.0, mestra: 5.0, borda: true, triangulos: true },
    Estilo { nome: "Agral - Border Only", curvas: false, intermediaria: 1.0, mestra: 5.0, borda: true, triangulos: false },
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
        let c = cotas(estilo("Agral - Standard"), 878.6, 891.2);
        assert_eq!(c.first(), Some(&(879.0, false)));
        assert_eq!(c.last(), Some(&(891.0, false)));
        let mestras: Vec<f64> = c.iter().filter(|x| x.1).map(|x| x.0).collect();
        assert_eq!(mestras, vec![880.0, 885.0, 890.0]);
    }

    #[test]
    fn estilo_desconhecido_cai_no_padrao() {
        assert_eq!(estilo("não existe").nome, "Agral - Standard");
    }
}
