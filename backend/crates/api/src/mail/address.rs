//! Forma canônica do endereço de email, a que é gravada, comparada e usada
//! no envio: o mesmo endereço escrito de outro jeito (maiúsculas,
//! `+etiqueta`, pontos no Gmail) não abre outra conta. Com `EMAIL_STRICT`
//! desligado (dev e staging), as variações ficam como foram escritas, para
//! várias contas de teste usarem a mesma caixa. O check `users_email_format`
//! do banco aceita as duas formas.

use super::disposable;
use crate::shared::ApiError;

const MAX_LEN: usize = 254;
const MAX_LOCAL_LEN: usize = 64;
const MAX_LABEL_LEN: usize = 63;

/// Endereço na forma que é gravada. Nos dois modos, sem espaços nas pontas e
/// em minúsculas. Com `strict` (produção), também:
/// - sem o `+etiqueta` (subendereço), em qualquer provedor: a mensagem para o
///   endereço sem ela chega na mesma caixa;
/// - no Gmail, sem os pontos da parte local, que o Gmail ignora, e
///   `googlemail.com` vira `gmail.com`;
/// - domínio descartável ou de apelidos → 422 `email.disposable`.
///
/// Sem `strict`, a etiqueta e os pontos ficam, e nenhum domínio é recusado.
///
/// Fora do formato, nos dois modos → 422 `email.invalid`: só ASCII, parte
/// local com letras, números, `.`, `_` e `-` (sem ponto nas pontas nem
/// repetido; no Gmail, só letras, números e pontos), mais a `+etiqueta` de
/// letras, números, `.`, `_`, `-` e `+`, e domínio com pelo menos um ponto.
/// Aceitar mais do que isso só serviria a endereços raros.
pub fn normalize(raw: &str, strict: bool) -> Result<String, ApiError> {
    let invalid = || ApiError::invalid("email.invalid");
    let address = raw.trim().to_ascii_lowercase();
    if !address.is_ascii() || address.len() > MAX_LEN {
        return Err(invalid());
    }
    let (local, domain) = address.split_once('@').ok_or_else(invalid)?;
    if !valid_domain(domain) {
        return Err(invalid());
    }
    let (base, tag) = match local.split_once('+') {
        Some((base, tag)) => (base, Some(tag)),
        None => (local, None),
    };
    let gmail = matches!(domain, "gmail.com" | "googlemail.com");
    // O Gmail ignora os pontos e não tem outros sinais.
    if gmail && !base.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.') {
        return Err(invalid());
    }

    if !strict {
        let tag_ok = tag.is_none_or(|tag| {
            !tag.is_empty()
                && tag
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'+'))
        });
        if !valid_local(base) || !tag_ok || local.len() > MAX_LOCAL_LEN {
            return Err(invalid());
        }
        return Ok(address);
    }

    let (local, domain) = if gmail {
        (base.replace('.', ""), "gmail.com")
    } else {
        (base.to_owned(), domain)
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
        normalize(raw, true).unwrap_err().code
    }

    /// O código do erro sem `strict` (dev e staging).
    fn lax_code(raw: &str) -> &'static str {
        normalize(raw, false).unwrap_err().code
    }

    #[test]
    fn lowercases_and_trims() {
        assert_eq!(
            normalize("  Fulano.Silva@Outlook.COM ", true).unwrap(),
            "fulano.silva@outlook.com"
        );
    }

    #[test]
    fn drops_subaddress_everywhere() {
        assert_eq!(
            normalize("ana+torneio@outlook.com", true).unwrap(),
            "ana@outlook.com"
        );
        assert_eq!(
            normalize("ana+a+b@uol.com.br", true).unwrap(),
            "ana@uol.com.br"
        );
        assert_eq!(normalize("ana@icloud.com", true).unwrap(), "ana@icloud.com");
    }

    #[test]
    fn gmail_ignores_dots_and_googlemail() {
        for raw in [
            "Joao.Silva@gmail.com",
            "j.o.a.o.s.i.l.v.a@gmail.com",
            "joaosilva+quake@gmail.com",
            "joao.silva+x.y@googlemail.com",
        ] {
            assert_eq!(
                normalize(raw, true).unwrap(),
                "joaosilva@gmail.com",
                "{raw}"
            );
        }
    }

    #[test]
    fn gmail_refuses_characters_gmail_does_not_have() {
        // Tirar o `_` apontaria para a caixa de outra pessoa.
        assert_eq!(code("joao_silva@gmail.com"), "email.invalid");
        assert_eq!(code("joao-silva@gmail.com"), "email.invalid");
        assert_eq!(code("...@gmail.com"), "email.invalid");
        assert_eq!(code("+x@gmail.com"), "email.invalid");
        for bad in [
            "joao_silva@gmail.com",
            "joao-silva@gmail.com",
            "+x@gmail.com",
        ] {
            assert_eq!(lax_code(bad), "email.invalid", "{bad}");
        }
    }

    #[test]
    fn keeps_dots_outside_gmail() {
        assert_eq!(
            normalize("a.b-c_d@hotmail.com", true).unwrap(),
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
            assert_eq!(lax_code(bad), "email.invalid", "sem strict: {bad:?}");
        }
        let long_local = format!("{}@outlook.com", "a".repeat(MAX_LOCAL_LEN + 1));
        assert_eq!(code(&long_local), "email.invalid");
        assert_eq!(lax_code(&long_local), "email.invalid");
        let long = format!("a@{}.com", "b.".repeat(130));
        assert_eq!(code(&long), "email.invalid");
        assert_eq!(lax_code(&long), "email.invalid");
    }

    #[test]
    fn refuses_disposable_and_relay_domains() {
        assert_eq!(code("x@mailinator.com"), "email.disposable");
        assert_eq!(code("x+y@yopmail.com"), "email.disposable");
        assert_eq!(code("x@duck.com"), "email.disposable");
    }

    #[test]
    fn without_strict_keeps_variations() {
        for (raw, kept) in [
            ("  Giuseppe+Teste@Gmail.com ", "giuseppe+teste@gmail.com"),
            ("g.i.u+a.b+c@gmail.com", "g.i.u+a.b+c@gmail.com"),
            ("ana@googlemail.com", "ana@googlemail.com"),
            ("ana+x_y-z@outlook.com", "ana+x_y-z@outlook.com"),
        ] {
            assert_eq!(normalize(raw, false).unwrap(), kept, "{raw}");
        }
        // Cada variação é outro endereço: cabe mais de uma conta na mesma caixa.
        assert_ne!(
            normalize("giuseppe@gmail.com", false).unwrap(),
            normalize("giuseppe+2@gmail.com", false).unwrap()
        );
    }

    #[test]
    fn without_strict_accepts_disposable_but_not_a_bad_tag() {
        assert_eq!(
            normalize("x@mailinator.com", false).unwrap(),
            "x@mailinator.com"
        );
        assert_eq!(normalize("x@duck.com", false).unwrap(), "x@duck.com");
        for bad in [
            "ana+@outlook.com",
            "ana+a b@outlook.com",
            "ana+!@outlook.com",
        ] {
            assert_eq!(lax_code(bad), "email.invalid", "{bad}");
        }
    }
}
