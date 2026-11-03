//! Quantas idas ao banco custa cada ação do caminho do clique (passo do veto,
//! envio e confirmação de partida) e a leitura da série: as queries (eventos
//! `sqlx::query`) e os pings das conexões reaproveitadas
//! (`test_before_acquire`), num torneio de 8 com duas séries no veto.

use std::{
    fmt::Debug,
    future::Future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{
    Registry,
    layer::{Context, Layer, SubscriberExt},
};

use super::*;

/// O resumo de cada query feita.
#[derive(Clone, Default)]
struct Log(Arc<Mutex<Vec<String>>>);

struct Summary(Option<String>);

impl Visit for Summary {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "summary" {
            self.0 = Some(value.to_owned());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "summary" {
            self.0 = Some(format!("{value:?}"));
        }
    }
}

impl<S: Subscriber> Layer<S> for Log {
    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        if event.metadata().target() != "sqlx::query" {
            return;
        }
        let mut summary = Summary(None);
        event.record(&mut summary);
        self.0.lock().unwrap().push(summary.0.unwrap_or_default());
    }
}

pub struct Meter {
    pings: Arc<AtomicUsize>,
}

pub struct Cost {
    pub queries: Vec<String>,
    pub pings: usize,
    pub micros: u128,
}

impl Meter {
    /// Um pool como o da API (`main.rs`), que conta os pings.
    pub async fn pool(db: &PgPool) -> (PgPool, Self) {
        let pings = Arc::new(AtomicUsize::new(0));
        let counted = pings.clone();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .before_acquire(move |_, _| {
                counted.fetch_add(1, Ordering::Relaxed);
                Box::pin(async { Ok(true) })
            })
            .connect_with((*db.connect_options()).clone())
            .await
            .unwrap();
        (pool, Self { pings })
    }

    /// O que `fut` custou, nesta thread (o teste roda num runtime de uma
    /// thread só).
    pub async fn measure<F: Future>(&self, fut: F) -> (F::Output, Cost) {
        let log = Log::default();
        let subscriber = Registry::default().with(log.clone());
        let guard = tracing::subscriber::set_default(subscriber);
        let pings = self.pings.load(Ordering::Relaxed);
        let start = Instant::now();
        let out = fut.await;
        let micros = start.elapsed().as_micros();
        drop(guard);
        let queries = log.0.lock().unwrap().clone();
        let pings = self.pings.load(Ordering::Relaxed) - pings;
        (
            out,
            Cost {
                queries,
                pings,
                micros,
            },
        )
    }
}

/// O que um pedido custou: as queries, os pings e o tempo.
async fn request(
    meter: &Meter,
    app: &Router,
    method: Method,
    uri: &str,
    who: &User,
    body: Option<Value>,
) -> (Value, Cost) {
    let ((status, body), cost) = meter.measure(call(app, method, uri, Some(who), body)).await;
    assert_eq!(status, StatusCode::OK, "{uri}: {body}");
    (body, cost)
}

/// A mediana dos tempos, em µs.
fn median(mut micros: Vec<u128>) -> u128 {
    micros.sort_unstable();
    micros[micros.len() / 2]
}

/// Imprime o custo (`cargo test matches::cost -- --nocapture`) e confere o
/// teto de idas ao banco.
pub fn check(label: &str, cost: &Cost, micros: Vec<u128>, max_queries: usize) {
    println!(
        "{label}: {} queries + {} pings; mediana {} µs em {} pedidos",
        cost.queries.len(),
        cost.pings,
        median(micros.clone()),
        micros.len()
    );
    for q in &cost.queries {
        println!("    {q}");
    }
    assert!(
        cost.queries.len() <= max_queries,
        "{label}: {} queries, o teto é {max_queries}. Uma query a mais no caminho de toda ação: dá para ler junto com outra?",
        cost.queries.len()
    );
    assert!(cost.pings <= 2, "{label}: {} pings", cost.pings);
}

/// Os dois jogadores da série `key` da chave `b` (lados a e b).
fn pair<'a>(p: &'a [User], b: &Value, key: &str) -> (&'a User, &'a User) {
    let (a, b2) = sides(b, find(b, key));
    let who = |id: &Value| p.iter().find(|u| *id == json!(u.id)).unwrap();
    (who(&a), who(&b2))
}

/// Torneio de 8 com `veto`, iniciado.
async fn eight(db: &PgPool, veto: Value) -> (Router, Meter, User, Vec<User>) {
    let (pool, meter) = Meter::pool(db).await;
    let app = app(pool);
    let ana = user(db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let p = players(&app, db, &ana, 8).await;
    let refs: Vec<&User> = p.iter().collect();
    start(&app, &ana, copa(with(single(3, 3), veto)), &refs).await;
    (app, meter, ana, p)
}

#[tokio::test]
async fn what_a_veto_step_costs() {
    let (db, _lease) = crate::db::test_db().await;
    let veto = json!({ "veto": { "map_mode": "veto", "champion_phase": true } });
    let (app, meter, ana, p) = eight(&db, veto).await;
    // Duas séries no veto.
    let b = bracket(&app, &ana).await;
    let keys = keys_in(&b, "ready");
    for key in &keys[..2] {
        let (a, b2) = pair(&p, &b, key);
        for u in [a, b2] {
            let uri = format!("{COPA}/matches/{key}/veto/ready");
            assert_eq!(post_empty(&app, &uri, u).await.0, StatusCode::OK);
        }
    }
    let (a, b2) = pair(&p, &b, &keys[0]);
    let uri = format!("{COPA}/matches/{}", keys[0]);
    get(&app, &uri, a).await;
    let mut micros = Vec::new();
    let mut last = None;
    for _ in 0..15 {
        let (_, cost) = request(&meter, &app, Method::GET, &uri, a, None).await;
        micros.push(cost.micros);
        last = Some(cost);
    }
    check("GET série", &last.unwrap(), micros, 9);

    // Os seis passos do veto de mapas (md3, 7 mapas), alternando os lados.
    let steps = format!("{uri}/veto/steps");
    let mut micros = Vec::new();
    let mut last = None;
    for _ in 0..6 {
        let d = get(&app, &uri, a).await.1;
        let who = if d["veto"]["turn"]["sides"] == json!(["a"]) {
            a
        } else {
            b2
        };
        let d = get(&app, &uri, who).await.1;
        let body = Some(json!({ "map": d["veto"]["options"][0] }));
        let (_, cost) = request(&meter, &app, Method::POST, &steps, who, body).await;
        micros.push(cost.micros);
        last = Some(cost);
    }
    check("POST passo do veto", &last.unwrap(), micros, 14);
}

#[tokio::test]
async fn what_reporting_and_confirming_cost() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, meter, ana, p) = eight(&db, json!({})).await;
    let b = bracket(&app, &ana).await;
    let (mut sent, mut confirmed) = ((Vec::new(), None), (Vec::new(), None));
    for key in keys_in(&b, "ready") {
        let (a, b2) = pair(&p, &b, &key);
        let uri = format!("{COPA}/matches/{key}/games/1");
        let body = Some(game("a", "awoken"));
        let (_, cost) = request(
            &meter,
            &app,
            Method::POST,
            &format!("{uri}/report"),
            a,
            body,
        )
        .await;
        sent.0.push(cost.micros);
        sent.1 = Some(cost);
        let (_, cost) = request(
            &meter,
            &app,
            Method::POST,
            &format!("{uri}/confirm"),
            b2,
            None,
        )
        .await;
        confirmed.0.push(cost.micros);
        confirmed.1 = Some(cost);
    }
    check("POST envio", &sent.1.unwrap(), sent.0, 16);
    check("POST confirmação", &confirmed.1.unwrap(), confirmed.0, 14);
}

/// O que o Next pede à API para montar a página da série (`MatchView` e o
/// layout).
#[tokio::test]
async fn what_the_series_page_costs() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, meter, ana, p) = eight(&db, json!({})).await;
    let b = bracket(&app, &ana).await;
    let key = &keys_in(&b, "ready")[0];
    let (a, _) = pair(&p, &b, key);
    let page = [
        "/me".to_owned(),
        "/tenants/quake-br".to_owned(),
        COPA.to_owned(),
        format!("{COPA}/matches/{key}"),
        format!("{COPA}/bracket"),
        format!("{COPA}/matches/{key}/screenshots"),
        "/maps".to_owned(),
    ];
    let mut micros = Vec::new();
    let mut total = Cost {
        queries: Vec::new(),
        pings: 0,
        micros: 0,
    };
    for round in 0..11 {
        let mut sum = Cost {
            queries: Vec::new(),
            pings: 0,
            micros: 0,
        };
        for uri in &page {
            let ((status, _), cost) = meter
                .measure(call(&app, Method::GET, uri, Some(a), None))
                .await;
            assert!(status.is_success(), "{uri}: {status}");
            sum.queries
                .extend(cost.queries.into_iter().map(|q| format!("{uri}: {q}")));
            sum.pings += cost.pings;
            sum.micros += cost.micros;
        }
        // A primeira volta aquece.
        if round > 0 {
            micros.push(sum.micros);
        }
        total = sum;
    }
    println!(
        "página da série ({} pedidos): {} queries + {} pings; mediana {} µs",
        page.len(),
        total.queries.len(),
        total.pings,
        median(micros)
    );
    assert!(total.queries.len() <= 30, "{} queries", total.queries.len());
}
