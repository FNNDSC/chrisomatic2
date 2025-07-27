use std::{io::ErrorKind, path::PathBuf};

use color_eyre::eyre::Context;
use futures::TryStreamExt;

pub(crate) async fn default_files() -> color_eyre::Result<Vec<PathBuf>> {
    let path = PathBuf::from("chrisomatic.toml");
    match fs_err::tokio::metadata(&path).await {
        Ok(metadata) => {
            if metadata.is_file() {
                return Ok(vec![path]);
            }
            Ok(())
        }
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                Err(e)
            } else {
                Ok(())
            }
        }
    }?;
    let dir = PathBuf::from("./chrisomatic.d");
    match tokio::fs::read_dir(dir).await {
        Ok(read_dir) => {
            let files = tokio_stream::wrappers::ReadDirStream::new(read_dir)
                .map_err(color_eyre::Report::from)
                .try_filter_map(async |entry| {
                    let path = entry.path();
                    if let Some(p) = path.to_str()
                        && p.ends_with(".toml")
                        && entry
                            .file_type()
                            .await
                            .with_context(|| format!("Cannot read file type of {entry:?}"))?
                            .is_file()
                    {
                        Ok(Some(entry.path()))
                    } else {
                        Ok(None)
                    }
                })
                .try_collect()
                .await?;
            Ok(files)
        }
        Err(e) => {
            if matches!(e.kind(), ErrorKind::NotADirectory | ErrorKind::NotFound) {
                color_eyre::eyre::bail!(
                    "chrisomatic.toml is not a file and chrisomatic.d is not a directory"
                );
            }
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, path::Path};

    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[tokio::test]
    #[serial]
    async fn test_default_files_dir() {
        let cwd = std::env::current_dir().unwrap();
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("chrisomatic.d");
        tokio::fs::create_dir(&dir).await.unwrap();
        tokio::join!(
            touch(dir.join("a.toml")),
            touch(dir.join("b.toml")),
            touch(dir.join("c.whatever"))
        );
        std::env::set_current_dir(temp.path()).unwrap();
        let files = default_files().await.unwrap();
        let actual: HashSet<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        let expected = HashSet::from(["a.toml", "b.toml"]);
        assert_eq!(actual, expected);
        std::env::set_current_dir(cwd).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_default_files_file() {
        let cwd = std::env::current_dir().unwrap();
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("chrisomatic.toml");
        tokio::fs::write(file, "test data").await.unwrap();

        std::env::set_current_dir(temp.path()).unwrap();
        let files = default_files().await.unwrap();

        assert_eq!(files.len(), 1);
        let actual_path = files.into_iter().next().unwrap();
        let actual = tokio::fs::read(actual_path).await.unwrap();
        assert_eq!(actual, b"test data");

        std::env::set_current_dir(cwd).unwrap();
    }

    async fn touch(path: impl AsRef<Path>) {
        tokio::fs::File::create(path).await.unwrap();
    }
}
