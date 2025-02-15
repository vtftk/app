use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use anyhow::Context;
use log::{debug, error};
use mime_guess::mime;
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

/// Names of folders within storage
#[derive(Debug, Deserialize)]
pub enum StorageFolder {
    ThrowableImage,
    ImpactSound,
    ImpactImage,
    Sound,
}

impl StorageFolder {
    pub fn folder_name(&self) -> &'static str {
        match self {
            StorageFolder::ThrowableImage => "throwable_images",
            StorageFolder::ImpactSound => "impact_sounds",
            StorageFolder::ImpactImage => "impact_images",
            StorageFolder::Sound => "sounds",
        }
    }
}

#[derive(Clone)]
pub enum Storage {
    /// Default storage backend by the file system
    Fs(Arc<FsStorage>),
    /// Mock storage provider for testing
    #[cfg(test)]
    Mock(Arc<test::MockStorage>),
}

impl Storage {
    pub fn new_fs(app_handle: &AppHandle) -> anyhow::Result<Self> {
        let fs = FsStorage::new(app_handle)?;
        Ok(Storage::Fs(Arc::new(fs)))
    }

    /// Uploads a file to the provided storage folder returning a
    /// URL for accessing the file
    pub async fn upload_file(
        &self,
        folder: StorageFolder,
        name: String,
        data: Vec<u8>,
    ) -> anyhow::Result<String> {
        match self {
            Storage::Fs(fs_storage) => fs_storage.upload_file(folder, name, data).await,
            #[cfg(test)]
            Storage::Mock(mock_storage) => mock_storage.upload_file(folder, name, data).await,
        }
    }

    /// Handles attempting to delete a URL from storage if the URL
    /// is stored within storage
    pub async fn try_delete_file(&self, url: String) -> anyhow::Result<()> {
        match self {
            Storage::Fs(fs_storage) => fs_storage.try_delete_file(url).await,
            #[cfg(test)]
            Storage::Mock(mock_storage) => mock_storage.try_delete_file(url).await,
        }
    }

    pub async fn get_file(
        &self,
        folder: String,
        name: String,
    ) -> anyhow::Result<Option<StorageFile>> {
        match self {
            Storage::Fs(fs_storage) => fs_storage.get_file(folder, name).await,
            #[cfg(test)]
            Storage::Mock(mock_storage) => mock_storage.get_file(folder, name).await,
        }
    }
}

pub struct StorageFile {
    pub mime: mime::Mime,
    pub content: Vec<u8>,
}

/// [Storage] backed by the local disk
pub struct FsStorage {
    content_path: PathBuf,
}

impl FsStorage {
    pub fn new(app_handle: &AppHandle) -> anyhow::Result<Self> {
        let app_data_path = app_handle
            .path()
            .app_data_dir()
            .context("failed to get app data dir")?;
        let content_path = app_data_path.join("content");

        Ok(Self { content_path })
    }

    async fn upload_file(
        &self,
        folder: StorageFolder,
        name: String,
        data: Vec<u8>,
    ) -> anyhow::Result<String> {
        let folder_name = folder.folder_name();
        let folder_path = self.content_path.join(folder_name);

        if !folder_path.exists() {
            tokio::fs::create_dir_all(&folder_path)
                .await
                .context("failed to create content folder")?;
        }

        let file_path_name = Path::new(&name);
        let extension = file_path_name
            .extension()
            .context("missing file extension")?
            .to_string_lossy();

        let file_id = Uuid::new_v4();
        let file_name = format!("{}.{}", file_id, extension);

        let file_path = folder_path.join(&file_name);

        tokio::fs::write(&file_path, data)
            .await
            .context("save file")?;

        Ok(format!("backend://content/{}/{}", folder_name, file_name))
    }

    async fn try_delete_file(&self, url: String) -> anyhow::Result<()> {
        let url = match reqwest::Url::from_str(&url) {
            Ok(value) => value,
            Err(err) => {
                error!("invalid src url: {err:?}");
                return Ok(());
            }
        };

        // Ignore non-backend URLs
        if url.scheme() != "backend" {
            return Ok(());
        }

        if url.domain().is_none_or(|value| !value.eq("content")) {
            return Ok(());
        }

        let file_path = url.path();

        let file_path = self
            .content_path
            .join(file_path.strip_prefix("/").unwrap_or(file_path));

        debug!("attempt delete content: {:?} {:?}", url, file_path);

        if file_path.exists() {
            tokio::fs::remove_file(file_path)
                .await
                .context("failed to delete file")?;
        }

        Ok(())
    }

    async fn get_file(&self, folder: String, name: String) -> anyhow::Result<Option<StorageFile>> {
        let file_path = self.content_path.join(folder).join(name);

        if !Self::is_path_within(&self.content_path, &file_path) {
            return Err(anyhow::anyhow!("resolved path was not within content path"));
        }

        if !file_path.exists() {
            return Ok(None);
        }

        let mime = mime_guess::from_path(&file_path).first_or_octet_stream();

        let content = tokio::fs::read(file_path)
            .await
            .context("failed to read content file")?;

        Ok(Some(StorageFile { content, mime }))
    }

    fn is_path_within(base: &Path, other: &Path) -> bool {
        // Canonicalize both paths to ensure they are absolute and normalized
        if let (Ok(base_abs), Ok(other_abs)) = (base.canonicalize(), other.canonicalize()) {
            // Check if `other_abs` starts with `base_abs`
            other_abs.starts_with(base_abs)
        } else {
            // If canonicalization fails, treat as not within
            false
        }
    }
}

#[cfg(test)]
#[allow(unused)]
mod test {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{Storage, StorageFile, StorageFolder};
    use anyhow::Context;
    use tokio::sync::Mutex;

    impl Storage {
        /// Create a new mocked storage implementation
        pub fn new_mock() -> Self {
            Self::Mock(Default::default())
        }

        /// Access the underlying mocked storage variant
        pub fn mocked(&self) -> &MockStorage {
            match self {
                Storage::Fs(_) => panic!("expected mock storage"),
                Storage::Mock(mock_storage) => mock_storage,
            }
        }
    }

    /// Mocked storage implementation
    #[derive(Default)]
    pub struct MockStorage {
        // Next URL the mock storage should return
        next_url: Mutex<Option<String>>,

        /// Next file the mock storage should return
        next_storage_file: Mutex<Option<StorageFile>>,

        upload_file_calls: Mutex<Vec<(StorageFolder, String, Vec<u8>)>>,
        try_delete_file_calls: Mutex<Vec<String>>,
        get_file_calls: Mutex<Vec<(String, String)>>,
    }

    impl MockStorage {
        pub fn set_next_url(&self, value: Option<String>) {
            *self.next_url.blocking_lock() = value;
        }

        pub fn set_next_file(&self, value: Option<StorageFile>) {
            *self.next_storage_file.blocking_lock() = value;
        }

        pub fn upload_file_count(&self) -> usize {
            self.upload_file_calls.blocking_lock().len()
        }

        pub fn try_delete_file_count(&self) -> usize {
            self.try_delete_file_calls.blocking_lock().len()
        }

        pub fn get_file_count(&self) -> usize {
            self.get_file_calls.blocking_lock().len()
        }

        pub fn upload_file_last(&self) -> Option<(StorageFolder, String, Vec<u8>)> {
            self.upload_file_calls.blocking_lock().pop()
        }

        pub fn try_delete_file_last(&self) -> Option<String> {
            self.try_delete_file_calls.blocking_lock().pop()
        }

        pub fn get_file_last(&self) -> Option<(String, String)> {
            self.get_file_calls.blocking_lock().pop()
        }

        pub async fn upload_file(
            &self,
            folder: StorageFolder,
            name: String,
            data: Vec<u8>,
        ) -> anyhow::Result<String> {
            self.upload_file_calls
                .lock()
                .await
                .push((folder, name, data));
            let url = self
                .next_url
                .lock()
                .await
                .take()
                .context("no mock url specified")?;

            Ok(url)
        }

        pub async fn try_delete_file(&self, url: String) -> anyhow::Result<()> {
            self.try_delete_file_calls.lock().await.push(url);
            Ok(())
        }

        pub async fn get_file(
            &self,
            folder: String,
            name: String,
        ) -> anyhow::Result<Option<StorageFile>> {
            self.get_file_calls.lock().await.push((folder, name));
            let storage_file = self.next_storage_file.lock().await.take();

            Ok(storage_file)
        }
    }
}
