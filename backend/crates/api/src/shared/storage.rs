//! Armazenamento de objetos pela API S3 genérica (AWS S3; R2 e MinIO pelo
//! `endpoint`). O `rusty-s3` só assina (SigV4 em URL assinada) e o HTTP é o
//! `reqwest`. O navegador manda e lê os objetos direto do bucket pelas URLs
//! assinadas que a API entrega; a API só confere (HEAD) e apaga.
//!
//! - **Envio** (`put_url`): PUT com `Content-Type` e `Content-Length`
//!   assinados. O navegador põe o `Content-Length` do arquivo sozinho, e
//!   assinatura que não confere é recusada pelo bucket: assim o tamanho fica
//!   limitado no próprio bucket também no R2, que não aceita o POST com
//!   policy.
//! - **Leitura** (`get_url`): assinada no começo da hora e válida por duas,
//!   então a mesma página recarregada na mesma hora recebe a mesma URL (o
//!   navegador usa o cache) e toda URL entregue vale pelo menos uma hora.

use std::{sync::Arc, time::Duration};

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use reqwest::{StatusCode, Url, header};
use rusty_s3::{
    Bucket, Credentials, S3Action, UrlStyle,
    actions::{DeleteObjectsResponse, ObjectIdentifier},
};

/// De quanto em quanto tempo a URL de leitura muda (e o cache do navegador).
const READ_WINDOW_SECS: i64 = 60 * 60;
/// Validade da URL de leitura: a janela em que foi assinada e mais uma.
const READ_EXPIRES: Duration = Duration::from_secs(2 * READ_WINDOW_SECS as u64);
/// O que o navegador guarda de cada objeto (nunca muda: o nome é aleatório).
const READ_CACHE: &str = "private, max-age=3600, immutable";
/// Validade das requisições que a própria API faz (HEAD, DELETE).
const OWN_EXPIRES: Duration = Duration::from_secs(60);
/// Tempo máximo de uma ida ao bucket.
const TIMEOUT: Duration = Duration::from_secs(15);
/// Chaves por `DeleteObjects` (o limite do S3).
pub const DELETE_BATCH: usize = 1000;

/// As variáveis `S3_*`.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Endereço de um S3 compatível (R2, MinIO), com o bucket no caminho.
    /// Sem ele, a AWS da `region`, com o bucket no nome do host.
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

/// O bucket. Barato de clonar.
#[derive(Clone)]
pub struct Storage(Arc<Inner>);

struct Inner {
    bucket: Bucket,
    credentials: Credentials,
    http: reqwest::Client,
}

/// O que o HEAD diz de um objeto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInfo {
    pub content_type: Option<String>,
    pub size: u64,
}

impl Storage {
    pub fn new(config: StorageConfig) -> anyhow::Result<Self> {
        let (endpoint, style) = match &config.endpoint {
            Some(url) => (url.parse().context("S3_ENDPOINT inválido")?, UrlStyle::Path),
            None => (
                format!("https://s3.{}.amazonaws.com", config.region)
                    .parse()
                    .context("S3_REGION inválida")?,
                UrlStyle::VirtualHost,
            ),
        };
        let bucket = Bucket::new(endpoint, style, config.bucket, config.region)
            .map_err(|e| anyhow::anyhow!("bucket inválido: {e:?}"))?;
        Ok(Self(Arc::new(Inner {
            bucket,
            credentials: Credentials::new(config.access_key_id, config.secret_access_key),
            http: reqwest::Client::builder()
                .user_agent("tournaments-api")
                .timeout(TIMEOUT)
                .build()
                .context("cliente HTTP")?,
        })))
    }

    /// URL para o navegador mandar `key` por PUT, válida por `expires` a
    /// partir de `now`. A requisição precisa levar `Content-Type:
    /// content_type` e ter exatamente `size` bytes.
    pub fn put_url(
        &self,
        key: &str,
        content_type: &str,
        size: u64,
        expires: Duration,
        now: DateTime<Utc>,
    ) -> Url {
        let inner = &self.0;
        let mut action = inner.bucket.put_object(Some(&inner.credentials), key);
        let headers = action.headers_mut();
        headers.insert("content-length", size.to_string());
        headers.insert("content-type", content_type.to_owned());
        action.sign_with_time(expires, &timestamp(now))
    }

    /// URL de leitura de `key`, assinada no começo da hora de `now`.
    pub fn get_url(&self, key: &str, now: DateTime<Utc>) -> Url {
        let inner = &self.0;
        let mut action = inner.bucket.get_object(Some(&inner.credentials), key);
        action
            .query_mut()
            .insert("response-cache-control", READ_CACHE);
        let start = now.timestamp() - now.timestamp().rem_euclid(READ_WINDOW_SECS);
        action.sign_with_time(READ_EXPIRES, &from_second(start))
    }

    /// O tipo e o tamanho de `key`; `None` se não existe. Sem a permissão
    /// `s3:ListBucket` (a política do README não dá), o S3 responde 403 em
    /// vez de 404 para o objeto que não existe: os dois contam como ausente.
    pub async fn head(&self, key: &str) -> anyhow::Result<Option<ObjectInfo>> {
        let inner = &self.0;
        let url = inner
            .bucket
            .head_object(Some(&inner.credentials), key)
            .sign(OWN_EXPIRES);
        let res = inner.http.head(url).send().await?;
        match res.status() {
            StatusCode::NOT_FOUND | StatusCode::FORBIDDEN => return Ok(None),
            s if !s.is_success() => bail!("HEAD no bucket respondeu {s}"),
            _ => {}
        }
        let headers = res.headers();
        let text = |name| headers.get(name).and_then(|v| v.to_str().ok());
        let size = text(header::CONTENT_LENGTH)
            .and_then(|v| v.parse().ok())
            .context("HEAD sem Content-Length")?;
        Ok(Some(ObjectInfo {
            content_type: text(header::CONTENT_TYPE).map(str::to_owned),
            size,
        }))
    }

    /// Apaga `keys` (até `DELETE_BATCH`) numa requisição só. Objeto que não
    /// existe conta como apagado. Devolve as chaves que o bucket não apagou.
    pub async fn delete(&self, keys: &[String]) -> anyhow::Result<Vec<String>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let inner = &self.0;
        let objects: Vec<ObjectIdentifier> = keys
            .iter()
            .map(|k| ObjectIdentifier::new(k.clone()))
            .collect();
        let mut action = inner
            .bucket
            .delete_objects(Some(&inner.credentials), objects.iter());
        action.set_quiet(true);
        let url = action.sign(OWN_EXPIRES);
        let (body, md5) = action.body_with_md5();
        let res = inner
            .http
            .post(url)
            .header("content-md5", md5)
            .body(body)
            .send()
            .await?;
        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            bail!("DeleteObjects no bucket respondeu {status}: {text}");
        }
        let parsed = DeleteObjectsResponse::parse(&text)
            .map_err(|e| anyhow::anyhow!("resposta do DeleteObjects: {e}"))?;
        Ok(parsed.errors.into_iter().map(|e| e.key).collect())
    }
}

fn timestamp(at: DateTime<Utc>) -> jiff::Timestamp {
    from_second(at.timestamp())
}

fn from_second(second: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_second(second).unwrap_or(jiff::Timestamp::UNIX_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn storage(endpoint: Option<&str>) -> Storage {
        Storage::new(StorageConfig {
            endpoint: endpoint.map(str::to_owned),
            region: "us-east-1".into(),
            bucket: "prints".into(),
            access_key_id: "AKIAIOSFODNN7EXAMPLE".into(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".into(),
        })
        .unwrap()
    }

    fn at(rfc3339: &str) -> DateTime<Utc> {
        rfc3339.parse().unwrap()
    }

    fn query(url: &Url, name: &str) -> Option<String> {
        url.query_pairs()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.into_owned())
    }

    #[test]
    fn the_upload_signs_type_and_size() {
        let s = storage(None);
        let url = s.put_url(
            "prints/1/2/abc.webp",
            "image/webp",
            1234,
            Duration::from_secs(600),
            at("2026-10-28T12:34:56Z"),
        );
        assert_eq!(url.host_str(), Some("prints.s3.us-east-1.amazonaws.com"));
        assert_eq!(url.path(), "/prints/1/2/abc.webp");
        assert_eq!(
            query(&url, "X-Amz-SignedHeaders").as_deref(),
            Some("content-length;content-type;host")
        );
        assert_eq!(query(&url, "X-Amz-Expires").as_deref(), Some("600"));
        assert_eq!(
            query(&url, "X-Amz-Date").as_deref(),
            Some("20261028T123456Z")
        );
        // Outro tamanho é outra assinatura: o bucket recusa o envio que não
        // bate com o pedido.
        let other = s.put_url(
            "prints/1/2/abc.webp",
            "image/webp",
            1235,
            Duration::from_secs(600),
            at("2026-10-28T12:34:56Z"),
        );
        assert_ne!(
            query(&url, "X-Amz-Signature"),
            query(&other, "X-Amz-Signature")
        );
    }

    #[test]
    fn the_read_url_holds_for_the_hour() {
        let s = storage(Some("http://127.0.0.1:9000"));
        let key = "prints/1/2/abc.webp";
        let first = s.get_url(key, at("2026-10-28T12:00:01Z"));
        let later = s.get_url(key, at("2026-10-28T12:59:59Z"));
        let next = s.get_url(key, at("2026-10-28T13:00:00Z"));
        assert_eq!(first, later);
        assert_ne!(first, next);
        assert_eq!(
            first.as_str().split('?').next(),
            Some("http://127.0.0.1:9000/prints/prints/1/2/abc.webp")
        );
        assert_eq!(
            query(&first, "X-Amz-Date").as_deref(),
            Some("20261028T120000Z")
        );
        assert_eq!(query(&first, "X-Amz-Expires").as_deref(), Some("7200"));
        assert_eq!(
            query(&first, "response-cache-control").as_deref(),
            Some(READ_CACHE)
        );
    }
}
