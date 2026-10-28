//! Se a requisição emitiu algum evento: a camada de `lib.rs` acorda o
//! despachante só depois das que emitiram (já com a transação encerrada). Fora
//! de uma requisição (agendador, socket), quem emite acorda por conta própria;
//! o agendador acorda o despachante a cada passada de qualquer forma.

use std::{cell::Cell, future::Future};

tokio::task_local! {
    static EMITTED: Cell<bool>;
}

/// Roda `fut` e devolve, junto com o resultado, se algum `emit` aconteceu
/// nela.
pub async fn track<F: Future>(fut: F) -> (F::Output, bool) {
    EMITTED
        .scope(Cell::new(false), async {
            let out = fut.await;
            (out, EMITTED.with(Cell::get))
        })
        .await
}

pub(super) fn mark() {
    let _ = EMITTED.try_with(|emitted| emitted.set(true));
}
