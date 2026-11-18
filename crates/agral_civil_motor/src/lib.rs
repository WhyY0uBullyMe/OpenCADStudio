//! Agral Civil: motor de superfície do Agral CAD.
//!
//! Sem tela e sem o Open CAD Studio: só `std` e `spade`. A ligação com o desenho fica no pacote `agral.civil`.
//! Spec: Hydrus `docs/specs/agral-cad/001-superficie/spec.md`.

pub mod alinhamento;
pub mod correcao;
pub mod corredor;
pub mod diagnostico;
pub mod emenda;
pub mod landxml;
pub mod pontos;
pub mod superficie;
pub mod tin;

/// Ponto 3D em metros: x = Este (E), y = Norte (N), z = cota.
pub type P3 = [f64; 3];
