use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::Utc;
use stheno_types::{Error, Id, Result};

use crate::dto::{CreatePresignedUploadInput, FileDTO, PresignedUploadDTO, UploadFileInput};
use crate::models::File;
use crate::provider::FileProvider;
use crate::UploadPipeline;

pub struct FileService {
    provider: Arc<dyn FileProvider>,
    files: RwLock<HashMap<String, File>>,
}

impl FileService {
    pub fn new(provider: Arc<dyn FileProvider>) -> Self {
        Self {
            provider,
            files: RwLock::new(HashMap::new()),
        }
    }

    pub async fn upload_file(&self, input: UploadFileInput) -> Result<FileDTO> {
        // Validate MIME type via magic bytes
        let detected = UploadPipeline::sniff_mime_type(&input.bytes)?;
        let mime_type_str = detected.as_str();

        if let Some(ref claimed) = input.content_type {
            if claimed != mime_type_str && !claimed.is_empty() {
                return Err(Error::Invalid {
                    code: "mime_mismatch",
                    message: format!(
                        "Claimed MIME type '{}' does not match sniffed binary MIME type '{}'",
                        claimed, mime_type_str
                    ),
                    field: Some("content_type".to_string()),
                });
            }
        }

        let file_id = Id::new();
        let upload_res = self
            .provider
            .upload(
                &file_id.to_string(),
                &input.filename,
                mime_type_str,
                &input.bytes,
            )
            .await?;

        let now = Utc::now();
        let file = File {
            id: file_id,
            filename: input.filename,
            url: upload_res.url.clone(),
            mime_type: mime_type_str.to_string(),
            size_bytes: upload_res.size_bytes,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let dto = FileDTO {
            id: file_id.to_string(),
            filename: file.filename.clone(),
            url: file.url.clone(),
            mime_type: file.mime_type.clone(),
            size_bytes: file.size_bytes,
            metadata: file.metadata.clone(),
            created_at: file.created_at,
            updated_at: file.updated_at,
        };

        {
            let mut map = self.files.write().unwrap();
            map.insert(file_id.to_string(), file);
        }

        Ok(dto)
    }

    pub async fn create_presigned_upload(
        &self,
        input: CreatePresignedUploadInput,
    ) -> Result<PresignedUploadDTO> {
        let file_id = Id::new();
        let expires_in = input.expires_in_seconds.unwrap_or(3600);

        let presigned = self
            .provider
            .get_presigned_upload_url(
                &file_id.to_string(),
                &input.filename,
                &input.content_type,
                expires_in,
            )
            .await?;

        let now = Utc::now();
        let file = File {
            id: file_id,
            filename: input.filename,
            url: presigned.file_url.clone(),
            mime_type: input.content_type,
            size_bytes: 0,
            metadata: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        {
            let mut map = self.files.write().unwrap();
            map.insert(file_id.to_string(), file);
        }

        Ok(PresignedUploadDTO {
            file_id: file_id.to_string(),
            upload_url: presigned.upload_url,
            file_url: presigned.file_url,
            headers: presigned.headers,
            expires_at: presigned.expires_at,
        })
    }

    pub async fn get_file(&self, id: &str) -> Result<FileDTO> {
        let map = self.files.read().unwrap();
        let file = map.get(id).ok_or_else(|| Error::NotFound {
            entity: "file",
            id: id.to_string(),
        })?;

        if file.deleted_at.is_some() {
            return Err(Error::NotFound {
                entity: "file",
                id: id.to_string(),
            });
        }

        Ok(FileDTO {
            id: file.id.to_string(),
            filename: file.filename.clone(),
            url: file.url.clone(),
            mime_type: file.mime_type.clone(),
            size_bytes: file.size_bytes,
            metadata: file.metadata.clone(),
            created_at: file.created_at,
            updated_at: file.updated_at,
        })
    }

    pub async fn delete_file(&self, id: &str) -> Result<()> {
        {
            let mut map = self.files.write().unwrap();
            let file = map.get_mut(id).ok_or_else(|| Error::NotFound {
                entity: "file",
                id: id.to_string(),
            })?;

            file.deleted_at = Some(Utc::now());
        }

        self.provider.delete(id).await?;
        Ok(())
    }

    pub async fn list_files(&self) -> Result<Vec<FileDTO>> {
        let map = self.files.read().unwrap();
        let list: Vec<FileDTO> = map
            .values()
            .filter(|f| f.deleted_at.is_none())
            .map(|f| FileDTO {
                id: f.id.to_string(),
                filename: f.filename.clone(),
                url: f.url.clone(),
                mime_type: f.mime_type.clone(),
                size_bytes: f.size_bytes,
                metadata: f.metadata.clone(),
                created_at: f.created_at,
                updated_at: f.updated_at,
            })
            .collect();
        Ok(list)
    }
}
