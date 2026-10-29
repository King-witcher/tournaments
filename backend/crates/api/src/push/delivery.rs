//! O envio de um aviso por Web Push, que o despachante do tempo real chama:
//! uma requisição por inscrição das pessoas do aviso. O corpo é o `Notice`
//! em JSON com o idioma da inscrição (`{code, params, path, locale}`), que o
//! service worker usa para traduzir e montar o endereço (`/{locale}{path}`).

use serde::Serialize;
use serde_json::Value;
use sqlx::PgConnection;

use super::{
    db,
    sender::{Sent, WebPush},
};
use crate::{outbox::Notice, shared::Locale};

/// O corpo de um push: o aviso e o idioma da inscrição.
#[derive(Serialize)]
struct Payload<'a> {
    #[serde(flatten)]
    notice: &'a Notice,
    locale: Locale,
}

/// O corpo do push para uma inscrição em `locale`, com o nome do torneio na
/// tradução desse idioma quando ele tem (senão, o do idioma original, que já
/// vai no aviso).
fn payload(notice: &Notice, locale: Locale, names: &[(Locale, String)]) -> Vec<u8> {
    let mut body = serde_json::to_value(Payload { notice, locale }).expect("o aviso vira JSON");
    let translated = names.iter().find(|(l, _)| *l == locale);
    if let (Some((_, name)), Some(Value::Object(params))) = (translated, body.get_mut("params"))
        && params.contains_key("tournament")
    {
        params.insert("tournament".into(), Value::String(name.clone()));
    }
    serde_json::to_vec(&body).expect("o aviso vira JSON")
}

/// Manda `notice` para as inscrições de `users`. A inscrição que o serviço diz
/// não existir mais (404, 410) sai; erro de rede ou outra resposta fica no log
/// e não segura os outros envios. Roda na transação do despachante.
pub async fn deliver(
    conn: &mut PgConnection,
    push: &dyn WebPush,
    users: &[i64],
    notice: &Notice,
) -> Result<(), sqlx::Error> {
    let targets = db::of_users(conn, users).await?;
    if targets.is_empty() {
        return Ok(());
    }
    let names = translated_names(conn, notice).await?;
    let (mut delivered, mut gone) = (Vec::new(), Vec::new());
    for target in &targets {
        let payload = payload(notice, target.locale, &names);
        match push.send(target, &payload).await {
            Ok(Sent::Delivered) => delivered.push(target.id),
            Ok(Sent::Gone) => gone.push(target.id),
            Err(e) => tracing::warn!(id = target.id, error = %e, "Web Push falhou"),
        }
    }
    if !gone.is_empty() {
        db::remove(conn, &gone).await?;
    }
    if !delivered.is_empty() {
        db::mark_used(conn, &delivered).await?;
    }
    Ok(())
}

/// Os nomes do torneio do aviso nas traduções (o `path` começa pelos slugs da
/// comunidade e do torneio).
async fn translated_names(
    conn: &mut PgConnection,
    notice: &Notice,
) -> Result<Vec<(Locale, String)>, sqlx::Error> {
    let mut parts = notice.path.trim_start_matches('/').split('/');
    match (parts.next(), parts.next()) {
        (Some(tenant), Some(slug)) => db::translated_names(conn, tenant, slug).await,
        _ => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::outbox::NoticeKind;

    fn notice() -> Notice {
        Notice {
            kind: NoticeKind::CheckInOpened {
                tournament: "Copa".into(),
            },
            path: "/quake-br/copa".into(),
        }
    }

    #[test]
    fn the_name_follows_the_language_of_the_subscription() {
        let names = [(Locale::Es, "Taza".to_owned())];
        let body = |locale| -> serde_json::Value {
            serde_json::from_slice(&payload(&notice(), locale, &names)).unwrap()
        };
        assert_eq!(body(Locale::Es)["params"], json!({ "tournament": "Taza" }));
        assert_eq!(body(Locale::Pt)["params"], json!({ "tournament": "Copa" }));
        assert_eq!(body(Locale::Es)["locale"], "es");
    }
}
