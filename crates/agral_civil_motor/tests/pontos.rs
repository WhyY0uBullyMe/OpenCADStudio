use agral_civil_motor::pontos::{ler_arquivo_de_pontos, FormatoPontos};

#[test]
fn pnezd_com_virgula_le_numero_norte_este_cota_descricao() {
    let texto = "1,7500100.5,500200.25,812.3,TN\n2,7500101,500201,813,TN\n";
    let r = ler_arquivo_de_pontos(texto, FormatoPontos::Pnezd);
    assert_eq!(r.pontos.len(), 2);
    assert!(r.linhas_ruins.is_empty());
    let p = &r.pontos[0];
    assert_eq!(p.numero.as_deref(), Some("1"));
    assert_eq!(p.xyz, [500200.25, 7500100.5, 812.3]);
    assert_eq!(p.descricao.as_deref(), Some("TN"));
}

#[test]
fn penzd_troca_a_ordem_de_norte_e_este() {
    let r = ler_arquivo_de_pontos("7 500200 7500100 800 BORDO", FormatoPontos::Penzd);
    assert_eq!(r.pontos[0].xyz, [500200.0, 7500100.0, 800.0]);
}

#[test]
fn nez_e_enz_sem_numero_nem_descricao() {
    let nez = ler_arquivo_de_pontos("7500100\t500200\t800", FormatoPontos::Nez);
    assert_eq!(nez.pontos[0].xyz, [500200.0, 7500100.0, 800.0]);
    assert!(nez.pontos[0].numero.is_none());
    let enz = ler_arquivo_de_pontos("500200 7500100 800", FormatoPontos::Enz);
    assert_eq!(enz.pontos[0].xyz, [500200.0, 7500100.0, 800.0]);
}

#[test]
fn ponto_e_virgula_com_decimal_em_virgula() {
    let r = ler_arquivo_de_pontos("1;7500100,5;500200,25;812,30;TN", FormatoPontos::Pnezd);
    assert_eq!(r.pontos[0].xyz, [500200.25, 7500100.5, 812.3]);
}

#[test]
fn linha_ruim_e_contada_e_mostrada_nao_some() {
    let texto = "PONTO,N,E,Z,D\n1,7500100,500200,800,TN\n2,abc,500201,801,TN\n\n3,7500102,500202\n4,7500103,500203,803,TN\n";
    let r = ler_arquivo_de_pontos(texto, FormatoPontos::Pnezd);
    assert_eq!(r.pontos.len(), 2);
    // Cabeçalho, número inválido e coluna faltando. Linha vazia não conta.
    let linhas: Vec<usize> = r.linhas_ruins.iter().map(|l| l.linha).collect();
    assert_eq!(linhas, vec![1, 3, 5]);
    assert!(r.linhas_ruins[1].texto.contains("abc"));
}

#[test]
fn descricao_com_espaco_fica_inteira() {
    let r = ler_arquivo_de_pontos("1 7500100 500200 800 PE DE TALUDE", FormatoPontos::Pnezd);
    assert_eq!(r.pontos[0].descricao.as_deref(), Some("PE DE TALUDE"));
}
