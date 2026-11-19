//! Compute Materials do Agral CAD (spec Hydrus `docs/agral/specs/004-compute-materials`): corte e aterro entre
//! duas superfícies nas linhas de seção do eixo, tabela de volumes no desenho e CSV para o Excel.

use super::OpenCADStudio;
use crate::civil::alinhamentos::{AlinhamentoDwg, CriterioMateriais};
use agral_civil_motor::alinhamento::texto_da_estaca;
use agral_civil_motor::materiais::{areas_na_secao, tabela, LinhaMaterial};
use document::entities::{Line, Text, TextHorizontalAlignment, TextVerticalAlignment};
use document::types::Vector3;
use document::EntityType;

/// Camada da tabela: a do template do JP se existir, senão a do Agral CAD.
const CAMADA_TEMPLATE: &str = "TÍTULO-TXT-AS";
const CAMADA_PADRAO: (&str, u8) = ("C-VOLUMES", 7);

/// Número com vírgula decimal (desenho e Excel em português).
pub(crate) fn br(v: f64, casas: usize) -> String {
    format!("{v:.casas$}").replace('.', ",")
}

impl OpenCADStudio {
    /// Linhas da tabela de volumes do eixo pelo critério gravado.
    pub(crate) fn civil_materiais_linhas(&mut self, i: usize, a: &AlinhamentoDwg) -> Result<Vec<LinhaMaterial>, String> {
        let m = a.materiais.as_ref().ok_or_else(|| format!("Alignment '{}' has no material criteria: run COMPUTEMATERIALS.", a.nome))?;
        let s = a.secoes.as_ref().ok_or_else(|| format!("Alignment '{}' has no sample lines: run CREATESECTIONS first.", a.nome))?;
        let al = a.geometria().ok_or("Alignment without geometry.")?;
        let base = self.civil_tin(i, &m.base)?;
        let comparar = self.civil_tin(i, &m.comparar)?;
        let areas: Vec<(f64, f64, f64)> = s
            .estacas_no(&al)
            .into_iter()
            .map(|e| {
                let (c, at) = areas_na_secao(&base, &comparar, &al, e, s.esquerda, s.direita);
                (e, c, at)
            })
            .collect();
        Ok(tabela(&areas, m.fator_corte, m.fator_aterro))
    }

    /// `COMPUTEMATERIALS`: grava o critério no eixo, calcula e redesenha a tabela se ela já existir.
    pub(crate) fn civil_computar_materiais(
        &mut self,
        i: usize,
        eixo: &str,
        base: &str,
        comparar: &str,
        fator_corte: f64,
        fator_aterro: f64,
    ) -> Result<String, String> {
        if !(fator_corte > 0.0 && fator_aterro > 0.0) {
            return Err("Cut and fill factors must be more than zero.".into());
        }
        let mut a = self.civil_al_ler(i, eixo)?;
        let base = self.civil_escolher(i, Some(base))?;
        let comparar = self.civil_escolher(i, Some(comparar))?;
        if base.eq_ignore_ascii_case(&comparar) {
            return Err("Choose two different surfaces.".into());
        }
        let tabela_antiga = a.materiais.as_ref().and_then(|m| m.tabela);
        a.materiais = Some(CriterioMateriais { base, comparar, fator_corte, fator_aterro, tabela: tabela_antiga, totais: None });
        let linhas = self.civil_materiais_linhas(i, &a)?;
        let u = linhas.last().copied().unwrap_or_default();
        if let Some(m) = a.materiais.as_mut() {
            m.totais = Some((u.acum_corte, u.acum_aterro));
        }
        self.push_undo_snapshot(i, "COMPUTEMATERIALS");
        self.civil_al_gravar(i, &a, false)?;
        if tabela_antiga.is_some() {
            self.civil_desenhar_tabela_volumes(i, &a.nome)?;
        }
        Ok(resumo(&a, &linhas))
    }

    /// `CREATEVOLUMETABLE`: desenha (ou move) a tabela de volumes com o canto de cima à esquerda em `origem`.
    pub(crate) fn civil_tabela_volumes(&mut self, i: usize, eixo: &str, origem: [f64; 2]) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, eixo)?;
        let m = a.materiais.as_mut().ok_or_else(|| format!("Alignment '{eixo}' has no material criteria: run COMPUTEMATERIALS first."))?;
        m.tabela = Some(origem);
        self.push_undo_snapshot(i, "CREATEVOLUMETABLE");
        self.civil_al_gravar(i, &a, false)?;
        self.civil_desenhar_tabela_volumes(i, &a.nome)
    }

    /// Redesenha a tabela de volumes do eixo no lugar gravado (nada se não houver tabela).
    pub(crate) fn civil_desenhar_tabela_volumes(&mut self, i: usize, eixo: &str) -> Result<String, String> {
        let mut a = self.civil_al_ler(i, eixo)?;
        let Some(origem) = a.materiais.as_ref().and_then(|m| m.tabela) else { return Ok(String::new()) };
        let linhas = self.civil_materiais_linhas(i, &a)?;
        let camada = if self.tabs[i].scene.document.layers.contains(CAMADA_TEMPLATE) {
            CAMADA_TEMPLATE.to_string()
        } else {
            let scene = &mut self.tabs[i].scene;
            if !scene.document.layers.contains(CAMADA_PADRAO.0) {
                scene.ensure_layer(CAMADA_PADRAO.0);
                if let Some(l) = scene.document.layers.get_mut(CAMADA_PADRAO.0) {
                    l.color = document::Color::Index(CAMADA_PADRAO.1);
                }
            }
            CAMADA_PADRAO.0.to_string()
        };
        let estilo = self.tabs[i].scene.document.text_styles.get("SIMPLEX-AS").map(|_| "SIMPLEX-AS".to_string());
        let ents = desenho_da_tabela(&a, &linhas, origem, &camada, estilo.as_deref());
        let u = linhas.last().copied().unwrap_or_default();
        if let Some(m) = a.materiais.as_mut() {
            m.totais = Some((u.acum_corte, u.acum_aterro));
        }
        let antigo = a.desenho_volumes.clone();
        a.desenho_volumes = self.civil_al_trocar_desenho(i, "VOLUMES", "VOLUMES", &a.nome.clone(), &antigo, ents);
        self.civil_al_gravar(i, &a, false)?;
        Ok(format!("Volume table of '{}' drawn ({} stations).", a.nome, linhas.len()))
    }

    /// `EXPORTMATERIALS`: CSV com ";" e vírgula decimal.
    pub(crate) fn civil_exportar_materiais(&mut self, i: usize, eixo: &str, caminho: &std::path::Path) -> Result<String, String> {
        let a = self.civil_al_ler(i, eixo)?;
        let linhas = self.civil_materiais_linhas(i, &a)?;
        let m = a.materiais.as_ref().expect("conferido acima");
        let mut csv = format!(
            "Eixo;{};Base;{};Comparação;{};Fator corte;{};Fator aterro;{}\n",
            a.nome,
            m.base,
            m.comparar,
            br(m.fator_corte, 3),
            br(m.fator_aterro, 3)
        );
        csv.push_str("Estaca;Área corte (m²);Área aterro (m²);Volume corte (m³);Volume aterro (m³);Acumulado corte (m³);Acumulado aterro (m³);Líquido acumulado (m³)\n");
        for l in &linhas {
            csv.push_str(&format!(
                "{};{};{};{};{};{};{};{}\n",
                texto_da_estaca(l.estaca),
                br(l.area_corte, 2),
                br(l.area_aterro, 2),
                br(l.vol_corte, 2),
                br(l.vol_aterro, 2),
                br(l.acum_corte, 2),
                br(l.acum_aterro, 2),
                br(l.liquido(), 2)
            ));
        }
        // BOM para o Excel ler os acentos em UTF-8.
        std::fs::write(caminho, format!("\u{feff}{csv}")).map_err(|e| format!("Could not write {}: {e}", caminho.display()))?;
        Ok(format!("{} stations exported to {}.", linhas.len(), caminho.display()))
    }
}

fn resumo(a: &AlinhamentoDwg, linhas: &[LinhaMaterial]) -> String {
    let m = a.materiais.as_ref().unwrap();
    let u = linhas.last().copied().unwrap_or_default();
    format!(
        "Materials of '{}' ({} x {}, {} sample lines): cut {:.1} m3, fill {:.1} m3, net {:.1} m3 (factors cut {}, fill {}).",
        a.nome,
        m.base,
        m.comparar,
        linhas.len(),
        u.acum_corte,
        u.acum_aterro,
        u.liquido(),
        m.fator_corte,
        m.fator_aterro
    )
}

/// Tabela de volumes em linhas e textos (o bloco AGRAL_VOLUMES_<eixo>).
fn desenho_da_tabela(a: &AlinhamentoDwg, linhas: &[LinhaMaterial], o: [f64; 2], camada: &str, estilo: Option<&str>) -> Vec<EntityType> {
    const H: f64 = 2.0;
    const ALT_LINHA: f64 = 4.0;
    let cabecalho = [
        ("ESTACA", 22.0),
        ("ÁREA CORTE (m²)", 30.0),
        ("ÁREA ATERRO (m²)", 30.0),
        ("VOL. CORTE (m³)", 30.0),
        ("VOL. ATERRO (m³)", 30.0),
        ("ACUM. CORTE (m³)", 32.0),
        ("ACUM. ATERRO (m³)", 32.0),
        ("LÍQUIDO (m³)", 30.0),
    ];
    let largura: f64 = cabecalho.iter().map(|c| c.1).sum();
    let mut out = Vec::new();
    let linha = |a: [f64; 2], b: [f64; 2]| {
        let mut l = Line::new();
        l.start = Vector3::new(a[0], a[1], 0.0);
        l.end = Vector3::new(b[0], b[1], 0.0);
        l.common.layer = camada.into();
        EntityType::Line(l)
    };
    let texto = |v: String, p: [f64; 2], h: f64| {
        let mut t = Text::new();
        t.value = v;
        t.height = h;
        t.common.layer = camada.into();
        if let Some(s) = estilo {
            t.style = s.into();
        }
        let q = Vector3::new(p[0], p[1], 0.0);
        t.insertion_point = q;
        t.horizontal_alignment = TextHorizontalAlignment::Center;
        t.vertical_alignment = TextVerticalAlignment::Middle;
        t.alignment_point = Some(q);
        EntityType::Text(t)
    };
    let m = a.materiais.as_ref().unwrap();
    // Título, cabeçalho e uma linha por estaca; no fim, o total.
    let n = linhas.len() + 3;
    let y = |k: usize| o[1] - k as f64 * ALT_LINHA;
    for k in 0..=n {
        out.push(linha([o[0], y(k)], [o[0] + largura, y(k)]));
    }
    out.push(linha([o[0], y(0)], [o[0], y(n)]));
    out.push(linha([o[0] + largura, y(0)], [o[0] + largura, y(n)]));
    let mut x = o[0];
    for (_, w) in &cabecalho {
        x += w;
        if x < o[0] + largura - 1e-9 {
            out.push(linha([x, y(1)], [x, y(n)]));
        }
    }
    let meio = |k: usize| y(k) - ALT_LINHA / 2.0;
    out.push(texto(format!("VOLUMES - {} ({} x {})", a.nome, m.base, m.comparar), [o[0] + largura / 2.0, meio(0)], H * 1.2));
    let colunas = |vals: [String; 8], k: usize, h: f64, out: &mut Vec<EntityType>| {
        let mut x = o[0];
        for (v, (_, w)) in vals.into_iter().zip(cabecalho.iter()) {
            out.push(texto(v, [x + w / 2.0, meio(k)], h));
            x += w;
        }
    };
    colunas(cabecalho.map(|c| c.0.to_string()), 1, H * 0.8, &mut out);
    for (k, l) in linhas.iter().enumerate() {
        colunas(
            [
                texto_da_estaca(l.estaca),
                br(l.area_corte, 2),
                br(l.area_aterro, 2),
                br(l.vol_corte, 2),
                br(l.vol_aterro, 2),
                br(l.acum_corte, 2),
                br(l.acum_aterro, 2),
                br(l.liquido(), 2),
            ],
            k + 2,
            H,
            &mut out,
        );
    }
    let u = linhas.last().copied().unwrap_or_default();
    colunas(
        ["TOTAL".into(), String::new(), String::new(), br(u.acum_corte, 2), br(u.acum_aterro, 2), String::new(), String::new(), br(u.liquido(), 2)],
        n - 1,
        H,
        &mut out,
    );
    out
}
