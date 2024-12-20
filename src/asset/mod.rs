use hashbrown::HashMap;
use std::{
    mem,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, Sender},
};

use anyhow::{anyhow, Result};
use glam::UVec2;
use io::{get_default_reader, ErasedAssetReader};

use crate::{
    font::Font,
    handle::{DropEvent, Handle, HandleId},
};

mod io;

#[derive(Debug)]
pub enum AssetType {
    Raw,
    Texture,
    Font,
}

pub(crate) struct PendingTask {
    pub handle: Handle,
    pub asset_type: AssetType,
    pub path: PathBuf,
}

pub(crate) enum FetchedTask {
    CreateTexture {
        handle: Handle,
        data: Vec<u8>,
        size: UVec2,
    },
    RemoveTexture {
        handle: u64,
    },
    CreateFont {
        handle: Handle,
        font: Font,
    },
    RemoveFont {
        handle: u64,
    },
}

pub struct Asset {
    pub asset_type: AssetType,
    pub bytes: Option<Vec<u8>>,
}

pub struct AssetManager {
    reader: Box<dyn ErasedAssetReader>,
    pending: Vec<PendingTask>,
    asset_id: u64,
    receiver: Receiver<DropEvent>,
    sender: Sender<DropEvent>,
    assets: HashMap<HandleId, Asset>,
}

impl AssetManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let (sender, receiver) = channel();
        Self {
            reader: get_default_reader(path),
            pending: Vec::default(),
            asset_id: 0,
            sender,
            receiver,
            assets: Default::default(),
        }
    }

    fn alloc_handle(&mut self) -> Handle {
        let id = self.asset_id;
        self.asset_id += 1;
        let drop_sender = self.sender.clone();
        Handle::new(id, drop_sender)
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P, asset_type: AssetType) -> Handle {
        let handle = self.alloc_handle();
        let task = PendingTask {
            handle: handle.clone(),
            path: path.as_ref().to_owned(),
            asset_type,
        };
        self.pending.push(task);
        if self.pending.len() > 4096 {
            log::warn!("Too many pending tasks");
        }
        handle
    }

    pub fn insert(&mut self, asset: Asset) -> Handle {
        let handle = self.alloc_handle();
        self.assets.insert(handle.id(), asset);
        if self.assets.len() > 4096 {
            log::warn!("Too many assets");
        }
        handle
    }

    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Handle {
        self.load(path, AssetType::Texture)
    }

    pub fn load_font<P: AsRef<Path>>(&mut self, path: P) -> Handle {
        self.load(path, AssetType::Font)
    }

    pub fn load_bytes<P: AsRef<Path>>(&mut self, path: P) -> Handle {
        self.load(path, AssetType::Raw)
    }

    pub fn get_asset(&self, handle: &Handle) -> Option<&Asset> {
        self.assets.get(&handle.id())
    }

    pub fn remove_asset<P: AsRef<Path>>(&mut self, handle: &Handle) -> Option<Asset> {
        self.assets.remove(&handle.id())
    }

    /// Fetch pending assets
    /// return completed tasks
    pub(crate) async fn fetch(&mut self) -> Result<Vec<FetchedTask>> {
        let mut tasks = Vec::default();
        // remove dropped assets
        while let Ok(event) = self.receiver.try_recv() {
            let Some(asset) = self.assets.remove(&event.0) else {
                continue;
            };
            match asset.asset_type {
                AssetType::Raw => {
                    // nothing todo
                }
                AssetType::Texture => {
                    let fetched_task = FetchedTask::RemoveTexture { handle: event.0 };
                    tasks.push(fetched_task);
                }
                AssetType::Font => {
                    let fetched_task = FetchedTask::RemoveFont { handle: event.0 };
                    tasks.push(fetched_task);
                }
            }
        }
        let pending = mem::take(&mut self.pending);
        // TODO Use task pool to poll assets
        for task in pending {
            let bytes = self.reader.read(task.path.to_str().unwrap()).await?;
            match task.asset_type {
                AssetType::Raw => {
                    self.assets.insert(
                        task.handle.id(),
                        Asset {
                            bytes: Some(bytes),
                            asset_type: AssetType::Raw,
                        },
                    );
                }
                AssetType::Texture => {
                    let im = match image::ImageFormat::from_path(&task.path) {
                        Ok(f) => image::load_from_memory_with_format(&bytes, f),
                        _ => image::load_from_memory(&bytes),
                    }?;
                    let size = UVec2::new(im.width(), im.height());
                    let data = im.into_bytes();
                    let fetched_task = FetchedTask::CreateTexture {
                        handle: task.handle.clone(),
                        data,
                        size,
                    };
                    tasks.push(fetched_task);
                    self.assets.insert(
                        task.handle.id(),
                        Asset {
                            bytes: None,
                            asset_type: AssetType::Texture,
                        },
                    );
                }
                AssetType::Font => {
                    let font = Font::from_bytes(bytes).ok_or(anyhow!("Failed to load font"))?;
                    let fetched_task = FetchedTask::CreateFont {
                        handle: task.handle.clone(),
                        font,
                    };
                    tasks.push(fetched_task);
                    self.assets.insert(
                        task.handle.id(),
                        Asset {
                            bytes: None,
                            asset_type: AssetType::Font,
                        },
                    );
                }
            }
        }
        Ok(tasks)
    }
}
