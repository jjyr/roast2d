use super::AssetReader;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

/// Copy from bevy <https://github.com/bevyengine/bevy/blob/99ab0285e459753838d0e0716fda9be7b4976a4c/crates/bevy_asset/src/io/file/mod.rs#L18>
fn get_base_path() -> PathBuf {
    if let Ok(manifest_dir) = env::var("ROAST2D_ASSET_ROOT") {
        PathBuf::from(manifest_dir)
    } else if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        env::current_exe()
            .map(|path| path.parent().map(ToOwned::to_owned).unwrap())
            .unwrap()
    }
}

pub(crate) struct FileAssetReader {
    root_path: PathBuf,
}

impl AssetReader for FileAssetReader {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        let root_path = get_base_path().join(path);
        Self { root_path }
    }

    fn get_full_path(&self, path: &str) -> PathBuf {
        self.root_path.join(path)
    }

    async fn read<'a>(&'a self, path: &'a str) -> Result<Vec<u8>> {
        let b: Vec<_> = fs::read(self.get_full_path(path)).unwrap();
        Ok(b)
    }
}
