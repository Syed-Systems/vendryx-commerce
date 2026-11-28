//! S3-compatible file storage, including path-style endpoints with nonstandard regions.

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use reqwest::{header::HeaderValue, Method, Response, Url};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use stheno_file::{FileProvider, FileUploadResult, PresignedUrlResult};
use stheno_types::{Error, Result};

const MAX_PRESIGN_SECONDS: u64 = 7 * 24 * 60 * 60;
const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Serialize, Deserialize)]
pub struct S3Config {
    /// Public objects are stored in this bucket.
    pub bucket: String,
    /// Private objects require a separate, non-public bucket.
    #[serde(default)]
    pub private_bucket: Option<String>,
    pub region: String,
    pub endpoint: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub cdn_url: Option<String>,
}

pub struct S3FileProvider {
    config: S3Config,
    client: reqwest::Client,
}

impl S3FileProvider {
    pub fn new(config: S3Config) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub fn config(&self) -> &S3Config {
        &self.config
    }

    fn bucket(&self, is_private: bool) -> Result<&str> {
        if is_private {
            let bucket = self.config.private_bucket.as_deref().ok_or_else(|| {
                invalid_config("private object storage requires a private bucket")
            })?;
            if bucket == self.config.bucket {
                return Err(invalid_config(
                    "public and private objects must use separate buckets",
                ));
            }
            Ok(bucket)
        } else {
            Ok(&self.config.bucket)
        }
    }

    fn endpoint(&self) -> Result<Url> {
        let endpoint = self
            .config
            .endpoint
            .clone()
            .unwrap_or_else(|| format!("https://s3.{}.amazonaws.com", self.config.region));
        let url = Url::parse(&endpoint).map_err(|_| invalid_config("S3 endpoint is invalid"))?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid_config("S3 endpoint is invalid"));
        }
        Ok(url)
    }

    fn object_url(&self, bucket: &str, key: &str) -> Result<Url> {
        let mut url = self.endpoint()?;
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| invalid_config("S3 endpoint cannot contain object paths"))?;
        segments.pop_if_empty().push(bucket);
        for segment in key.split('/') {
            segments.push(segment);
        }
        drop(segments);
        Ok(url)
    }

    fn public_url(&self, key: &str) -> Result<String> {
        if let Some(cdn_url) = self.config.cdn_url.as_deref() {
            let mut url =
                Url::parse(cdn_url).map_err(|_| invalid_config("public file URL is invalid"))?;
            if !matches!(url.scheme(), "http" | "https")
                || url.host_str().is_none()
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return Err(invalid_config("public file URL is invalid"));
            }
            let mut segments = url
                .path_segments_mut()
                .map_err(|_| invalid_config("public file URL cannot contain object paths"))?;
            segments.pop_if_empty();
            for segment in key.split('/') {
                segments.push(segment);
            }
            drop(segments);
            return Ok(url.to_string());
        }

        Ok(self.object_url(&self.config.bucket, key)?.to_string())
    }

    async fn signed_request(
        &self,
        method: Method,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
        body: Option<&[u8]>,
    ) -> Result<Response> {
        let url = self.object_url(bucket, key)?;
        let host = host_header(&url)?;
        let amz_date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let date = &amz_date[..8];
        let payload_hash = body.map_or_else(
            || EMPTY_SHA256.to_string(),
            |bytes| hex::encode(Sha256::digest(bytes)),
        );
        let mut headers = vec![
            ("host", host.as_str()),
            ("x-amz-content-sha256", payload_hash.as_str()),
            ("x-amz-date", amz_date.as_str()),
        ];
        if let Some(content_type) = content_type {
            validate_header(content_type)?;
            headers.push(("content-type", content_type));
        }
        headers.sort_unstable_by_key(|(name, _)| *name);
        let canonical_headers = headers
            .iter()
            .map(|(name, value)| format!("{name}:{}\n", normalize_header(value)))
            .collect::<String>();
        let signed_headers = headers
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(";");
        let canonical_request = format!(
            "{}\n{}\n\n{}\n{}\n{}",
            method.as_str(),
            url.path(),
            canonical_headers,
            signed_headers,
            payload_hash
        );
        let scope = format!("{date}/{}/s3/aws4_request", self.config.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let signature = self.signature(date, &string_to_sign)?;
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
            self.config.access_key_id
        );

        let mut request = self
            .client
            .request(method, url.clone())
            .header(
                "Host",
                HeaderValue::from_str(&host).map_err(|_| invalid_config("S3 host is invalid"))?,
            )
            .header("x-amz-date", &amz_date)
            .header("x-amz-content-sha256", payload_hash)
            .header("Authorization", authorization);
        if let Some(content_type) = content_type {
            request = request.header(
                "Content-Type",
                HeaderValue::from_str(content_type)
                    .map_err(|_| invalid_input("content type is invalid"))?,
            );
        }
        if let Some(body) = body {
            request = request.body(body.to_vec());
        }
        request
            .send()
            .await
            .map_err(|_| provider_error("S3 request failed"))
    }

    fn signature(&self, date: &str, string_to_sign: &str) -> Result<String> {
        let date_key = hmac_sha256(
            format!("AWS4{}", self.config.secret_access_key).as_bytes(),
            date.as_bytes(),
        )?;
        let region_key = hmac_sha256(&date_key, self.config.region.as_bytes())?;
        let service_key = hmac_sha256(&region_key, b"s3")?;
        let signing_key = hmac_sha256(&service_key, b"aws4_request")?;
        let signature = hmac_sha256(&signing_key, string_to_sign.as_bytes())?;
        Ok(hex::encode(signature))
    }

    fn presigned_url(
        &self,
        method: &str,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
        expires_in_seconds: u64,
    ) -> Result<String> {
        if expires_in_seconds == 0 || expires_in_seconds > MAX_PRESIGN_SECONDS {
            return Err(invalid_input(
                "presigned URL expiry must be between 1 second and 7 days",
            ));
        }
        if let Some(content_type) = content_type {
            validate_header(content_type)?;
        }
        let mut url = self.object_url(bucket, key)?;
        let host = host_header(&url)?;
        let amz_date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let date = &amz_date[..8];
        let scope = format!("{date}/{}/s3/aws4_request", self.config.region);
        let credential = format!("{}/{scope}", self.config.access_key_id);
        let signed_headers = if content_type.is_some() {
            "content-type;host"
        } else {
            "host"
        };
        let mut query = [
            ("X-Amz-Algorithm", "AWS4-HMAC-SHA256".to_string()),
            ("X-Amz-Credential", credential),
            ("X-Amz-Date", amz_date.clone()),
            ("X-Amz-Expires", expires_in_seconds.to_string()),
            ("X-Amz-SignedHeaders", signed_headers.to_string()),
        ];
        query.sort_unstable_by_key(|(name, _)| *name);
        let canonical_query = query
            .iter()
            .map(|(name, value)| format!("{}={}", uri_encode(name), uri_encode(value)))
            .collect::<Vec<_>>()
            .join("&");
        let mut canonical_headers = String::new();
        if let Some(content_type) = content_type {
            canonical_headers.push_str(&format!(
                "content-type:{}\n",
                normalize_header(content_type)
            ));
        }
        canonical_headers.push_str(&format!("host:{host}\n"));
        let canonical_request = format!(
            "{method}\n{}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\nUNSIGNED-PAYLOAD",
            url.path()
        );
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let signature = self.signature(date, &string_to_sign)?;
        let full_query = format!("{canonical_query}&X-Amz-Signature={signature}");
        url.set_query(Some(&full_query));
        Ok(url.to_string())
    }

    async fn object_is_private(&self, file_id: &str) -> Result<bool> {
        let Some(bucket) = self.config.private_bucket.as_deref() else {
            return Ok(false);
        };
        match self
            .signed_request(
                Method::HEAD,
                bucket,
                &object_key("private", file_id)?,
                None,
                None,
            )
            .await?
            .status()
        {
            status if status.is_success() => Ok(true),
            reqwest::StatusCode::NOT_FOUND => Ok(false),
            status => Err(provider_status(status)),
        }
    }
}

#[async_trait]
impl FileProvider for S3FileProvider {
    fn provider_id(&self) -> &'static str {
        "s3"
    }

    async fn upload(
        &self,
        file_id: &str,
        _filename: &str,
        content_type: &str,
        bytes: &[u8],
        is_private: bool,
    ) -> Result<FileUploadResult> {
        let bucket = self.bucket(is_private)?;
        let prefix = if is_private { "private" } else { "uploads" };
        let key = object_key(prefix, file_id)?;
        let response = self
            .signed_request(Method::PUT, bucket, &key, Some(content_type), Some(bytes))
            .await?;
        ensure_success(response)?;
        let url = if is_private {
            format!("s3://{bucket}/{key}")
        } else {
            self.public_url(&key)?
        };
        Ok(FileUploadResult {
            url,
            size_bytes: i64::try_from(bytes.len())
                .map_err(|_| invalid_input("file is too large"))?,
        })
    }

    async fn delete(&self, file_id: &str) -> Result<()> {
        let public_key = object_key("uploads", file_id)?;
        ensure_success(
            self.signed_request(Method::DELETE, &self.config.bucket, &public_key, None, None)
                .await?,
        )?;
        if let Some(bucket) = self.config.private_bucket.as_deref() {
            let private_key = object_key("private", file_id)?;
            ensure_success(
                self.signed_request(Method::DELETE, bucket, &private_key, None, None)
                    .await?,
            )?;
        }
        Ok(())
    }

    async fn get_url(&self, file_id: &str) -> Result<String> {
        if self.object_is_private(file_id).await? {
            let bucket = self.bucket(true)?;
            return Ok(format!("s3://{bucket}/{}", object_key("private", file_id)?));
        }
        let key = object_key("uploads", file_id)?;
        match self
            .signed_request(Method::HEAD, &self.config.bucket, &key, None, None)
            .await?
            .status()
        {
            status if status.is_success() => self.public_url(&key),
            status => Err(provider_status(status)),
        }
    }

    async fn get_presigned_upload_url(
        &self,
        file_id: &str,
        _filename: &str,
        content_type: &str,
        expires_in_seconds: u64,
        is_private: bool,
    ) -> Result<PresignedUrlResult> {
        let bucket = self.bucket(is_private)?;
        let prefix = if is_private { "private" } else { "uploads" };
        let key = object_key(prefix, file_id)?;
        let upload_url =
            self.presigned_url("PUT", bucket, &key, Some(content_type), expires_in_seconds)?;
        let file_url = if is_private {
            format!("s3://{bucket}/{key}")
        } else {
            self.public_url(&key)?
        };
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        Ok(PresignedUrlResult {
            upload_url,
            file_url,
            headers,
            expires_at: Utc::now() + Duration::seconds(expires_in_seconds as i64),
        })
    }

    async fn get_presigned_download_url(
        &self,
        file_id: &str,
        expires_in_seconds: u64,
    ) -> Result<String> {
        let is_private = self.object_is_private(file_id).await?;
        let prefix = if is_private { "private" } else { "uploads" };
        let bucket = self.bucket(is_private)?;
        let key = object_key(prefix, file_id)?;
        let response = self
            .signed_request(Method::HEAD, bucket, &key, None, None)
            .await?;
        if !response.status().is_success() {
            return Err(provider_status(response.status()));
        }
        self.presigned_url("GET", bucket, &key, None, expires_in_seconds)
    }
}

fn object_key(prefix: &str, file_id: &str) -> Result<String> {
    if file_id.is_empty()
        || !file_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(invalid_input("file id is invalid"));
    }
    Ok(format!("{prefix}/{file_id}"))
}

fn host_header(url: &Url) -> Result<String> {
    let host = url
        .host_str()
        .ok_or_else(|| invalid_config("S3 endpoint has no host"))?;
    Ok(url
        .port()
        .map_or_else(|| host.to_string(), |port| format!("{host}:{port}")))
}

fn validate_header(value: &str) -> Result<()> {
    HeaderValue::from_str(value)
        .map(|_| ())
        .map_err(|_| invalid_input("content type is invalid"))
}

fn normalize_header(value: &str) -> String {
    value.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
}

fn sha256_hex(value: &[u8]) -> String {
    hex::encode(Sha256::digest(value))
}

fn hmac_sha256(key: &[u8], value: &[u8]) -> Result<Vec<u8>> {
    let mut mac =
        HmacSha256::new_from_slice(key).map_err(|_| provider_error("S3 signing failed"))?;
    mac.update(value);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn uri_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn ensure_success(response: Response) -> Result<()> {
    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        Err(provider_status(status))
    }
}

fn provider_status(status: reqwest::StatusCode) -> Error {
    Error::Internal(Box::new(std::io::Error::other(format!(
        "S3 request failed with HTTP {status}"
    ))))
}

fn provider_error(message: &'static str) -> Error {
    Error::Internal(Box::new(std::io::Error::other(message)))
}

fn invalid_config(message: &'static str) -> Error {
    Error::Invalid {
        code: "invalid_s3_config",
        message: message.to_string(),
        field: Some("storage".to_string()),
    }
}

fn invalid_input(message: &'static str) -> Error {
    Error::Invalid {
        code: "invalid_s3_input",
        message: message.to_string(),
        field: Some("file".to_string()),
    }
}
