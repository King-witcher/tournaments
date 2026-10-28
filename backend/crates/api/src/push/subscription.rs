//! Formato da inscrição de push que a API aceita: o endpoint é para onde a
//! API manda requisições, então só `https` num nome de domínio (nada de IP,
//! `localhost` ou porta); as chaves, com o tamanho que o padrão define.
//! Guardadas em base64url sem padding (os checks da tabela conferem o mesmo).

use reqwest::Url;

use super::sender::decode;
use crate::shared::ApiError;

const INVALID: ApiError = ApiError::invalid("push.subscription_invalid");
const ENDPOINT_MAX: usize = 2048;

/// Endpoint, `p256dh` e `auth` como ficam no banco. 422
/// `push.subscription_invalid` fora do formato.
pub fn validate(
    endpoint: &str,
    p256dh: &str,
    auth: &str,
) -> Result<(String, String, String), ApiError> {
    if !endpoint_ok(endpoint) {
        return Err(INVALID);
    }
    let p256dh = canonical(p256dh, 65)
        .filter(|k| k.starts_with('B'))
        .ok_or(INVALID)?;
    let auth = canonical(auth, 16).ok_or(INVALID)?;
    Ok((endpoint.to_owned(), p256dh, auth))
}

fn endpoint_ok(endpoint: &str) -> bool {
    if endpoint.len() > ENDPOINT_MAX || !endpoint.chars().all(|c| c.is_ascii_graphic()) {
        return false;
    }
    let Ok(url) = Url::parse(endpoint) else {
        return false;
    };
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && url
            .domain()
            .is_some_and(|d| d.contains('.') && !d.ends_with("localhost"))
}

/// A chave em base64url sem padding, se decodifica para `len` bytes.
fn canonical(key: &str, len: usize) -> Option<String> {
    let trimmed = key.trim_end_matches('=');
    (decode(trimmed).ok()?.len() == len).then(|| trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const P256DH: &str =
        "BLn9b-VR0ca83knDNZ32dCHGyjJp-1riX9ZTN40MqV8K_LpQmLqxC_DoHvqvFXO_nGdAB4W9dogZb_sM-uV4JbY";
    const AUTH: &str = "_ordMnz7uTCmrpBTeUV4Bw";

    fn ok(endpoint: &str) -> bool {
        validate(endpoint, P256DH, AUTH).is_ok()
    }

    #[test]
    fn only_https_domains() {
        assert!(ok("https://fcm.googleapis.com/fcm/send/abc:def"));
        assert!(ok(
            "https://updates.push.services.mozilla.com/wpush/v2/gAAA"
        ));
        for bad in [
            "http://fcm.googleapis.com/x",
            "https://127.0.0.1/x",
            "https://[::1]/x",
            "https://localhost/x",
            "https://push.localhost/x",
            "https://intranet/x",
            "https://fcm.googleapis.com:8443/x",
            "https://user@fcm.googleapis.com/x",
            "https://fcm.googleapis.com/a b",
            "não é url",
        ] {
            assert!(!ok(bad), "{bad}");
        }
        assert!(!ok(&format!("https://a.com/{}", "x".repeat(ENDPOINT_MAX))));
    }

    #[test]
    fn keys_have_the_standard_sizes() {
        let (_, p256dh, auth) = validate(
            "https://a.com/x",
            &format!("{P256DH}="),
            &format!("{AUTH}=="),
        )
        .unwrap();
        assert_eq!((p256dh.as_str(), auth.as_str()), (P256DH, AUTH));
        let short = &P256DH[..80];
        assert!(validate("https://a.com/x", short, AUTH).is_err());
        assert!(validate("https://a.com/x", P256DH, &AUTH[..20]).is_err());
        assert!(validate("https://a.com/x", "não+é/base64", AUTH).is_err());
    }
}
