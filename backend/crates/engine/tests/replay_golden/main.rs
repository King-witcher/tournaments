//! Congela o replay de torneios já gravados: o histórico (configuração,
//! participantes, resultados, pareamentos, eventos de veto) está escrito aqui
//! como dado, na ordem em que a API o gravaria, e o estado esperado também é
//! literal. Se um teste falhar, uma mudança no engine alterou torneios em
//! andamento ou encerrados (ou passou a recusar o histórico deles). Não
//! atualize o esperado sem decidir antes o que acontece com os torneios
//! gravados (migration dos dados, versão da regra ou reverter a mudança).
//!
//! Só a API pública da crate, a mesma que a `api` usa. O estado vira texto
//! pelos getters públicos (`common.rs`), para um campo interno renomeado não
//! quebrar o teste.

mod common;
mod elimination;
mod tables;
mod veto;
