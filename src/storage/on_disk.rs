use std::collections::HashMap;
use std::path::Path;
use tokio::fs::create_dir_all;
use tokio::io::AsyncWriteExt;

pub(crate) struct OnDiskStorage {
    path: String,
}

impl OnDiskStorage {
    pub(crate) fn create(path: String) -> Self {
        Self { path }
    }

    pub(crate) async fn get(
        &self,
        prefix: &str,
        key: &str,
    ) -> Result<Option<String>, std::io::Error> {
        let path = format!("{}/{}/{}", self.path, prefix, key);

        match tokio::fs::read_to_string(path).await {
            Ok(value) => Ok(Some(value)),
            Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub(crate) async fn get_all(
        &self,
        prefix: &str,
    ) -> Result<HashMap<String, String>, std::io::Error> {
        let path = format!("{}/{}", self.path, prefix);

        let mut map = HashMap::new();

        let mut dir_reader = tokio::fs::read_dir(&path).await?;
        while let Some(dir) = dir_reader.next_entry().await? {
            let filename = dir.file_name().to_str().unwrap_or_default().to_string();
            let content = tokio::fs::read_to_string(format!("{}/{}", path, filename)).await?;
            map.insert(filename, content);
        }

        Ok(map)
    }

    pub(crate) async fn save(
        &self,
        prefix: &str,
        key: &str,
        value: &str,
    ) -> Result<(), std::io::Error> {
        let filepath = format!("{}/{}/{}", self.path, prefix, key);
        let path = Path::new(&filepath);
        let parent = path.parent().expect("Unable to get parent path");

        create_dir_all(parent).await?;

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .await?;

        file.write_all(value.as_bytes()).await?;

        Ok(())
    }

    pub(crate) async fn delete(&self, prefix: &str, key: &str) -> Result<(), std::io::Error> {
        let path = format!("{}/{}/{}", self.path, prefix, key);

        tokio::fs::remove_file(path).await?;

        Ok(())
    }
}
