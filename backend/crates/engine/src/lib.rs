//! Motores de torneio: formatos de chaveamento, pontuação, veto, regras de
//! campeões e rating.
//!
//! Esta crate é pura: sem I/O, sem banco, sem relógio. Aleatoriedade entra
//! por seed/RNG passado como parâmetro.
//!
//! O estado de um formato é sempre reconstruível a partir da configuração,
//! dos participantes e do histórico de resultados (`replay`). Corrigir um
//! resultado é refazer o estado com o histórico corrigido.

pub mod champions;
pub mod core;
pub mod formats;
pub mod rating;
pub mod seeding;
pub mod series;
pub mod tournament;
pub mod veto;
