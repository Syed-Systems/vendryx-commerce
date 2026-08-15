use std::sync::Arc;
use stheno_file::{
    CreatePresignedUploadInput, FileDTO, FileService, PresignedUploadDTO, UploadFileInput,
};
use stheno_types::Result;

pub struct UploadFileWorkflow {
    file_service: Arc<FileService>,
}

impl UploadFileWorkflow {
    pub fn new(file_service: Arc<FileService>) -> Self {
        Self { file_service }
    }

    pub async fn run(&self, input: UploadFileInput) -> Result<FileDTO> {
        self.file_service.upload_file(input).await
    }
}

pub struct CreatePresignedUploadWorkflow {
    file_service: Arc<FileService>,
}

impl CreatePresignedUploadWorkflow {
    pub fn new(file_service: Arc<FileService>) -> Self {
        Self { file_service }
    }

    pub async fn run(&self, input: CreatePresignedUploadInput) -> Result<PresignedUploadDTO> {
        self.file_service.create_presigned_upload(input).await
    }
}

pub struct DeleteFileWorkflow {
    file_service: Arc<FileService>,
}

impl DeleteFileWorkflow {
    pub fn new(file_service: Arc<FileService>) -> Self {
        Self { file_service }
    }

    pub async fn run(&self, id: &str) -> Result<()> {
        self.file_service.delete_file(id).await
    }
}
