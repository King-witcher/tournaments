//! Um S3 falso num servidor axum do próprio teste, para as rotas usarem o
//! HTTP de verdade (o PUT do navegador, o HEAD e o DeleteObjects da API, o
//! GET da imagem). Guarda os objetos em memória e confere cada URL como o S3:
//! assinatura SigV4 refeita com as mesmas chaves e os cabeçalhos assinados
//! com os valores que chegaram (um PUT com outro tamanho ou outro tipo é
//! recusado), validade, o `If-None-Match: *` do PUT (412 com o objeto já
//! gravado) e o `Content-MD5` do DeleteObjects.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use api::shared::StorageConfig;
use axum::{
    Router,
    body::Bytes,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{NaiveDateTime, Utc};
use reqwest::Url;

const BUCKET: &str = "screenshots";
const REGION: &str = "us-east-1";
const KEY_ID: &str = "AKIDTEST";
const SECRET: &str = "segredo-de-teste";

#[derive(Clone, Debug)]
pub struct Object {
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

#[derive(Default)]
struct Bucket {
    objects: HashMap<String, Object>,
    /// Chaves que o DeleteObjects responde como erro.
    undeletable: BTreeSet<String>,
    /// Quantos DeleteObjects chegaram.
    delete_requests: usize,
}

#[derive(Clone)]
pub struct FakeS3 {
    addr: SocketAddr,
    bucket: Arc<Mutex<Bucket>>,
}

impl FakeS3 {
    pub async fn start() -> Self {
        let bucket = Arc::new(Mutex::new(Bucket::default()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = Router::new().fallback(handle).with_state(bucket.clone());
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self { addr, bucket }
    }

    pub fn config(&self) -> StorageConfig {
        StorageConfig {
            endpoint: Some(format!("http://{}", self.addr)),
            region: REGION.into(),
            bucket: BUCKET.into(),
            access_key_id: KEY_ID.into(),
            secret_access_key: SECRET.into(),
        }
    }

    pub fn keys(&self) -> BTreeSet<String> {
        self.bucket
            .lock()
            .unwrap()
            .objects
            .keys()
            .cloned()
            .collect()
    }

    pub fn object(&self, key: &str) -> Option<Object> {
        self.bucket.lock().unwrap().objects.get(key).cloned()
    }

    /// Grava direto, sem URL (o que um cliente qualquer teria mandado).
    pub fn put(&self, key: &str, content_type: &str, body: Vec<u8>) {
        let object = Object {
            content_type: Some(content_type.into()),
            body,
        };
        self.bucket
            .lock()
            .unwrap()
            .objects
            .insert(key.into(), object);
    }

    pub fn refuse_to_delete(&self, key: &str) {
        self.bucket.lock().unwrap().undeletable.insert(key.into());
    }

    pub fn delete_requests(&self) -> usize {
        self.bucket.lock().unwrap().delete_requests
    }
}

async fn handle(State(bucket): State<Arc<Mutex<Bucket>>>, req: Request) -> Response {
    let (parts, body) = req.into_parts();
    let body = axum::body::to_bytes(body, 10 * 1024 * 1024).await.unwrap();
    let host = parts.headers[header::HOST].to_str().unwrap().to_owned();
    let url: Url = format!("http://{host}{}", parts.uri).parse().unwrap();
    if let Err(why) = verify(&parts.method, &url, &parts.headers) {
        return (StatusCode::FORBIDDEN, why).into_response();
    }
    let path = url
        .path()
        .strip_prefix(&format!("/{BUCKET}/"))
        .unwrap_or("");
    let mut bucket = bucket.lock().unwrap();
    match parts.method {
        Method::PUT => {
            let only_new = parts
                .headers
                .get(header::IF_NONE_MATCH)
                .is_some_and(|v| v == "*");
            if only_new && bucket.objects.contains_key(path) {
                return StatusCode::PRECONDITION_FAILED.into_response();
            }
            let content_type = parts
                .headers
                .get(header::CONTENT_TYPE)
                .map(|v| v.to_str().unwrap().to_owned());
            let object = Object {
                content_type,
                body: body.to_vec(),
            };
            bucket.objects.insert(path.to_owned(), object);
            StatusCode::OK.into_response()
        }
        Method::HEAD | Method::GET => match bucket.objects.get(path) {
            Some(o) => {
                let ct = o.content_type.clone().unwrap_or_default();
                ([(header::CONTENT_TYPE, ct)], Bytes::from(o.body.clone())).into_response()
            }
            None => StatusCode::NOT_FOUND.into_response(),
        },
        Method::POST if url.query_pairs().any(|(k, _)| k == "delete") => {
            bucket.delete_requests += 1;
            if parts.headers.get("content-md5").is_none() {
                return (StatusCode::BAD_REQUEST, "sem Content-MD5").into_response();
            }
            let xml = String::from_utf8(body.to_vec()).unwrap();
            let mut errors = String::new();
            for key in keys_in(&xml) {
                if bucket.undeletable.contains(&key) {
                    errors.push_str(&format!(
                        "<Error><Key>{key}</Key><Code>AccessDenied</Code><Message>não</Message></Error>"
                    ));
                } else {
                    bucket.objects.remove(&key);
                }
            }
            let xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><DeleteResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">{errors}</DeleteResult>"#
            );
            xml.into_response()
        }
        _ => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    }
}

/// As chaves de um corpo `<Delete><Object><Key>…</Key></Object>…</Delete>`.
fn keys_in(xml: &str) -> Vec<String> {
    xml.split("<Key>")
        .skip(1)
        .filter_map(|rest| rest.split_once("</Key>").map(|(k, _)| k.to_owned()))
        .collect()
}

/// Confere a URL assinada como o S3: refaz a assinatura com a data, a
/// validade e os cabeçalhos assinados que chegaram, e confere a validade.
fn verify(method: &Method, url: &Url, headers: &HeaderMap) -> Result<(), String> {
    let query: BTreeMap<String, String> = url.query_pairs().into_owned().collect();
    let get = |name: &str| query.get(name).cloned().ok_or(format!("sem {name}"));
    let date = NaiveDateTime::parse_from_str(&get("X-Amz-Date")?, "%Y%m%dT%H%M%SZ")
        .map_err(|e| e.to_string())?
        .and_utc();
    let expires: u64 = get("X-Amz-Expires")?.parse().map_err(|_| "X-Amz-Expires")?;
    if Utc::now().timestamp() > date.timestamp() + expires as i64 {
        return Err("URL vencida".into());
    }
    let signed = get("X-Amz-SignedHeaders")?;
    let mut values = Vec::new();
    for name in signed.split(';').filter(|n| *n != "host") {
        let value = headers
            .get(name)
            .ok_or(format!("sem o cabeçalho assinado {name}"))?
            .to_str()
            .unwrap()
            .trim()
            .to_owned();
        values.push((name.to_owned(), value));
    }
    let rest: Vec<(&str, &str)> = query
        .iter()
        .filter(|(k, _)| !k.starts_with("X-Amz-"))
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let method = match *method {
        Method::GET => rusty_s3::Method::Get,
        Method::HEAD => rusty_s3::Method::Head,
        Method::PUT => rusty_s3::Method::Put,
        Method::POST => rusty_s3::Method::Post,
        Method::DELETE => rusty_s3::Method::Delete,
        _ => return Err("método".into()),
    };
    let mut bare = url.clone();
    bare.set_query(None);
    let again = rusty_s3::signing::sign(
        &jiff::Timestamp::from_second(date.timestamp()).unwrap(),
        method,
        bare,
        KEY_ID,
        SECRET,
        None,
        REGION,
        expires,
        rest.iter().copied(),
        values.iter().map(|(k, v)| (k.as_str(), v.as_str())),
    );
    let signature = |u: &Url| {
        u.query_pairs()
            .find(|(k, _)| k == "X-Amz-Signature")
            .map(|(_, v)| v.into_owned())
    };
    if signature(&again) != signature(url) {
        return Err("assinatura não confere".into());
    }
    Ok(())
}
