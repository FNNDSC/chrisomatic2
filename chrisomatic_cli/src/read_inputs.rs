use std::{
    io::Read,
    path::{Path, PathBuf},
};

use chrisomatic_spec::GivenManifest;
use futures::{StreamExt, TryStreamExt};

pub(crate) async fn read_inputs(files: &[PathBuf]) -> color_eyre::Result<GivenManifest> {
    if let Some(name) = files.first().and_then(|p| p.to_str())
        && files.len() == 1
        && name == "-"
    {
        read_stdin()
    } else {
        read_files(files).await
    }
}

async fn read_files(files: &[PathBuf]) -> color_eyre::Result<GivenManifest> {
    let manifests: Vec<_> = futures::stream::iter(files)
        .map(|p| Ok(read_file(p)))
        .try_buffer_unordered(8)
        .try_collect()
        .await?;
    let manifest = chrisomatic_spec::reduce(manifests)?;
    Ok(manifest)
}

async fn read_file(path: &Path) -> color_eyre::Result<GivenManifest> {
    let data = fs_err::tokio::read(path).await?;
    let manifest = toml::from_slice(&data)?;
    Ok(manifest)
}

fn read_stdin() -> color_eyre::Result<GivenManifest> {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf)?;
    let manifest = toml::from_slice(&buf)?;
    Ok(manifest)
}
