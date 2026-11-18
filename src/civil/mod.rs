//! Agral Civil dentro do Agral CAD: superfícies como no Civil 3D (decisão do JP em 18/11: civil no núcleo, não
//! como pacote). O cálculo fica no `agral_civil_motor`; aqui ficam os dados gravados no DWG e a leitura das
//! entidades. Os comandos estão em `app/commands/surface.rs`, o painel em `ui/window/toolspace.rs`.

pub mod curvas;
pub mod definicao;
pub mod estilos;

use definicao::{SuperficieDwg, DICIONARIO};
use document::objects::ObjectType;
use document::{Document, Handle};

/// Dicionário `AGRAL_CIVIL/SURFACES` do desenho, se existir.
fn dicionario(doc: &Document) -> Option<Handle> {
    let mut atual = doc.header.named_objects_dict_handle;
    for nome in DICIONARIO {
        match doc.objects.get(&atual) {
            Some(ObjectType::Dictionary(d)) => atual = d.get(nome)?,
            _ => return None,
        }
    }
    matches!(doc.objects.get(&atual), Some(ObjectType::Dictionary(_))).then_some(atual)
}

/// Superfícies gravadas no desenho, em ordem de nome (só leitura; serve para o painel Toolspace).
pub fn superficies(doc: &Document) -> Vec<SuperficieDwg> {
    let Some(h) = dicionario(doc) else { return Vec::new() };
    let Some(ObjectType::Dictionary(d)) = doc.objects.get(&h) else { return Vec::new() };
    let mut v: Vec<SuperficieDwg> = d
        .entries
        .iter()
        .filter_map(|(nome, h)| match doc.objects.get(h) {
            Some(ObjectType::XRecord(x)) => SuperficieDwg::de_xrecord(nome, &x.entries).ok(),
            _ => None,
        })
        .collect();
    v.sort_by_key(|s| s.nome.to_lowercase());
    v
}
