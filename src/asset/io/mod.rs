use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};

use anyhow::Result;

#[cfg(not(target_arch = "wasm32"))]
mod file;
#[cfg(target_arch = "wasm32")]
mod web;

pub trait AssetReader: Send + Sync + 'static {
    fn new<P: AsRef<Path>>(path: P) -> Self
    where
        Self: Sized;
    fn get_full_path(&self, path: &str) -> PathBuf;
    async fn read<'a>(&'a self, path: &'a str) -> Result<Vec<u8>>;
}

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// From bevy https://github.com/bevyengine/bevy/blob/main/crates/bevy_asset/src/io/mod.rs#L186
/// Equivalent to AssetReader, but nessacery for trait object safe
pub trait ErasedAssetReader: Send + Sync + 'static {
    /// Returns a future to load the full file data at the provided path.
    fn read<'a>(&'a self, path: &'a str) -> BoxedFuture<Result<Vec<u8>>>;
}

impl<T: AssetReader> ErasedAssetReader for T {
    fn read<'a>(&'a self, path: &'a str) -> BoxedFuture<Result<Vec<u8>>> {
        Box::pin(async {
            let buf = Self::read(self, path).await?;
            Ok(buf)
        })
    }
}

pub(crate) fn get_default_reader<P: AsRef<Path>>(path: P) -> Box<dyn ErasedAssetReader> {
    #[cfg(not(target_arch = "wasm32"))]
    return Box::new(file::FileAssetReader::new(path));
    #[cfg(target_arch = "wasm32")]
    return Box::new(web::WebAssetReader::new(path));
}
