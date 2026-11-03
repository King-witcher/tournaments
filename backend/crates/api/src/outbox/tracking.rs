//! Se a requisição emitiu algum evento: a camada de `lib.rs` acorda o
//! despachante só depois das que emitiram (já com a transação encerrada). Fora
//! de uma requisição (agendador, socket), quem emite acorda por conta própria;
//! o agendador acorda o despachante a cada passada de qualquer forma. E de
//! qual aba veio a requisição (`origin`), que vai com cada evento dela.

use std::{cell::Cell, future::Future};

tokio::task_local! {
    static EMITTED: Cell<bool>;
    static ORIGIN: Option<String>;
}

/// Roda `fut` e devolve, junto com o resultado, se algum `emit` aconteceu
/// nela. `origin`: o id da conexão do WebSocket da aba que pediu (o
/// despachante não devolve a ela as mudanças de partida que ela mesma causou).
pub async fn track<F: Future>(origin: Option<String>, fut: F) -> (F::Output, bool) {
    ORIGIN
        .scope(
            origin,
            EMITTED.scope(Cell::new(false), async {
                let out = fut.await;
                (out, EMITTED.with(Cell::get))
            }),
        )
        .await
}

pub(super) fn mark() {
    let _ = EMITTED.try_with(|emitted| emitted.set(true));
}

/// A aba de onde veio a requisição em andamento, se ela disse.
pub fn origin() -> Option<String> {
    ORIGIN.try_with(Clone::clone).ok().flatten()
}
