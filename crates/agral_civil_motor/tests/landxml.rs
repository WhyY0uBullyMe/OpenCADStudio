use agral_civil_motor::landxml::{escrever_superficie, ler_superficies};
use agral_civil_motor::tin::Tin;

/// Arquivo sintético no formato LandXML 1.2 (P = Norte Este Cota; F com i="1" é face invisível).
const EXEMPLO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LandXML xmlns="http://www.landxml.org/schema/LandXML-1.2" version="1.2">
  <Units><Metric linearUnit="meter" areaUnit="squareMeter" volumeUnit="cubicMeter"/></Units>
  <Surfaces>
    <Surface name="TN" desc="Terreno natural">
      <Definition surfType="TIN">
        <Pnts>
          <P id="1">7500000 500000 800</P>
          <P id="2">7500000 500010 801</P>
          <P id="3">7500010 500010 802</P>
          <P id="4">7500010 500000 801</P>
        </Pnts>
        <Faces>
          <F>1 2 3</F>
          <F>1 3 4</F>
          <F i="1">2 3 4</F>
        </Faces>
      </Definition>
    </Surface>
  </Surfaces>
</LandXML>"#;

#[test]
fn le_superficie_do_landxml_com_norte_este_e_sem_face_invisivel() {
    let s = ler_superficies(EXEMPLO).unwrap();
    assert_eq!(s.len(), 1);
    assert_eq!(s[0].nome, "TN");
    assert_eq!(s[0].descricao, "Terreno natural");
    let t = &s[0].tin;
    assert_eq!(t.vertices[0], [500000.0, 7500000.0, 800.0]);
    assert_eq!(t.triangulos.len(), 2);
    let z = t.cota_em(500005.0, 7500002.0).unwrap();
    assert!((z - 800.7).abs() < 1e-9, "{z}");
}

#[test]
fn escreve_e_le_de_volta_igual() {
    let t = Tin::novo(
        vec![[500000.0, 7500000.0, 800.0], [500010.0, 7500000.0, 801.5], [500000.0, 7500010.0, 802.25]],
        vec![[0, 1, 2]],
    );
    let xml = escrever_superficie("TN", "teste", &t);
    let lida = ler_superficies(&xml).unwrap();
    assert_eq!(lida[0].tin.vertices, t.vertices);
    assert_eq!(lida[0].tin.triangulos, t.triangulos);
    assert!(xml.contains("surfType=\"TIN\""));
}

#[test]
fn arquivo_quebrado_da_erro_claro() {
    assert!(ler_superficies("<LandXML><Surfaces>").is_err());
    let e = ler_superficies(&EXEMPLO.replace("<F>1 2 3</F>", "<F>1 2 9</F>")).unwrap_err();
    assert!(e.contains('9'), "{e}");
}
