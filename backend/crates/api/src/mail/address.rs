//! Forma canônica do endereço de email, a que é gravada, comparada e usada
//! no envio: o mesmo endereço escrito de outro jeito (maiúsculas,
//! `+etiqueta`, pontos no Gmail) não abre outra conta. O check
//! `users_email_format` do banco confere a mesma forma.

use super::disposable;
use crate::shared::ApiError;

const MAX_LEN: usize = 254;
const MAX_LOCAL_LEN: usize = 64;
const MAX_LABEL_LEN: usize = 63;

/// Endereço na forma canônica:
/// - sem espaços nas pontas e em minúsculas;
/// - sem o `+etiqueta` (subendereço), em qualquer provedor: a mensagem para o
///   endereço sem ela chega na mesma caixa;
/// - no Gmail, sem os pontos da parte local, que o Gmail ignora, e
///   `googlemail.com` vira `gmail.com`.
///
/// Fora do formato → 422 `email.invalid`: só ASCII, parte local com letras,
/// números, `.`, `_` e `-` (sem ponto nas pontas nem repetido; no Gmail, só
/// letras e números) e domínio com pelo menos um ponto. Aceitar mais do que
/// isso só serviria a endereços raros. Domínio descartável ou de apelidos →
/// 422 `email.disposable`.
pub fn normalize(raw: &str) -> Result<String, ApiError> {
    let invalid = || ApiError::invalid("email.invalid");
    let address = raw.trim().to_ascii_lowercase();
    if !address.is_ascii() || address.len() > MAX_LEN {
        return Err(invalid());
    }
    let (local, domain) = address.split_once('@').ok_or_else(invalid)?;
    if !valid_domain(domain) {
        return Err(invalid());
    }
    let local = local.split_once('+').map_or(local, |(base, _)| base);

    let (local, domain) = match domain {
        "gmail.com" | "googlemail.com" => {
            let local = local.replace('.', "");
            if !local.bytes().all(|b| b.is_ascii_alphanumeric()) {
                return Err(invalid());
            }
            (local, "gmail.com")
        }
        _ => (local.to_owned(), domain),
    };
    if !valid_local(&local) {
        return Err(invalid());
    }
    if disposable::is_blocked(domain) {
        return Err(ApiError::invalid("email.disposable"));
    }
    Ok(format!("{local}@{domain}"))
}

fn valid_local(local: &str) -> bool {
    !local.is_empty()
        && local.len() <= MAX_LOCAL_LEN
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
        && local
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Rótulos de letras, números e hífen (sem hífen nas pontas), pelo menos
/// dois, e o último só de letras: recusa IP, `localhost` e domínio sem ponto.
fn valid_domain(domain: &str) -> bool {
    let labels: Vec<&str> = domain.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|label| {
            (1..=MAX_LABEL_LEN).contains(&label.len())
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
        && labels
            .last()
            .is_some_and(|tld| tld.len() >= 2 && tld.bytes().all(|b| b.is_ascii_alphabetic()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(raw: &str) -> &'static str {
        normalize(raw).unwrap_err().code
    }

    #[test]
    fn lowercases_and_trims() {
        assert_eq!(
            normalize("  Fulano.Silva@Outlook.COM ").unwrap(),
            "fulano.silva@outlook.com"
        );
    }

    #[test]
    fn drops_subaddress_everywhere() {
        assert_eq!(
            normalize("ana+torneio@outlook.com").unwrap(),
            "ana@outlook.com"
        );
        assert_eq!(normalize("ana+a+b@uol.com.br").unwrap(), "ana@uol.com.br");
        assert_eq!(normalize("ana@icloud.com").unwrap(), "ana@icloud.com");
    }

    #[test]
    fn gmail_ignores_dots_and_googlemail() {
        for raw in [
            "Joao.Silva@gmail.com",
            "j.o.a.o.s.i.l.v.a@gmail.com",
            "joaosilva+quake@gmail.com",
            "joao.silva+x.y@googlemail.com",
        ] {
            assert_eq!(normalize(raw).unwrap(), "joaosilva@gmail.com", "{raw}");
        }
    }

    #[test]
    fn gmail_refuses_characters_gmail_does_not_have() {
        // Tirar o `_` apontaria para a caixa de outra pessoa.
        assert_eq!(code("joao_silva@gmail.com"), "email.invalid");
        assert_eq!(code("joao-silva@gmail.com"), "email.invalid");
        assert_eq!(code("...@gmail.com"), "email.invalid");
        assert_eq!(code("+x@gmail.com"), "email.invalid");
    }

    #[test]
    fn keeps_dots_outside_gmail() {
        assert_eq!(
            normalize("a.b-c_d@hotmail.com").unwrap(),
            "a.b-c_d@hotmail.com"
        );
    }

    #[test]
    fn refuses_malformed_addresses() {
        for bad in [
            "",
            "ana",
            "ana@",
            "@gmail.com",
            "ana@@gmail.com",
            "ana@gmail",
            "ana@localhost",
            "ana@127.0.0.1",
            "ana@[127.0.0.1]",
            "ana@-x.com",
            "ana@x-.com",
            "ana@x..com",
            "ana@x.c",
            "ana@x.c0m",
            ".ana@outlook.com",
            "ana.@outlook.com",
            "a..na@outlook.com",
            "a na@outlook.com",
            "\"ana\"@outlook.com",
            "ana!@outlook.com",
            "joão@outlook.com",
            "ana@dominío.com",
        ] {
            assert_eq!(code(bad), "email.invalid", "{bad:?}");
        }
        let long_local = format!("{}@outlook.com", "a".repeat(MAX_LOCAL_LEN + 1));
        assert_eq!(code(&long_local), "email.invalid");
        let long = format!("a@{}.com", "b.".repeat(130));
        assert_eq!(code(&long), "email.invalid");
    }

    #[test]
    fn refuses_disposable_and_relay_domains() {
        assert_eq!(code("x@mailinator.com"), "email.disposable");
        assert_eq!(code("x+y@yopmail.com"), "email.disposable");
        assert_eq!(code("x@duck.com"), "email.disposable");
    }
}
