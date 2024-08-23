use std::{
    collections::HashMap,
    mem,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, Sender},
};

use anyhow::Result;
use glam::UVec2;
use io::{get_default_reader, ErasedAssetReader};

use crate::handle::{DropEvent, Handle, HandleId};

mod io;

#[derive(Debug)]
pub enum AssetType {
    Raw,
    Texture,
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
}

pub struct AssetManager {
    reader: Box<dyn ErasedAssetReader>,
    pending: Vec<PendingTask>,
    asset_id: u64,
    receiver: Receiver<DropEvent>,
    sender: Sender<DropEvent>,
    assets: HashMap<HandleId, Vec<u8>>,
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

    pub(crate) fn alloc_handle(&mut self) -> Handle {
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
        handle
    }

    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Handle {
        self.load(path, AssetType::Texture)
    }

    pub fn load_bytes<P: AsRef<Path>>(&mut self, path: P) -> Handle {
        self.load(path, AssetType::Raw)
    }

    pub fn get_raw(&self, handle: &Handle) -> Option<&Vec<u8>> {
        self.assets.get(&handle.id())
    }

    pub fn remove_raw<P: AsRef<Path>>(&mut self, handle: &Handle) -> Option<Vec<u8>> {
        self.assets.remove(&handle.id())
    }

    /// Fetch pending assets
    /// return completed tasks
    pub(crate) async fn fetch(&mut self) -> Result<Vec<FetchedTask>> {
        let mut tasks = Vec::default();
        // remove dropped assets
        while let Ok(event) = self.receiver.try_recv() {
            self.assets.remove(&event.0);
            let fetched_task = FetchedTask::RemoveTexture { handle: event.0 };
            tasks.push(fetched_task);
        }
        let pending = mem::take(&mut self.pending);
        // TODO Use task pool to poll assets
        for task in pending {
            let raw = self.reader.read(task.path.to_str().unwrap()).await?;
            match task.asset_type {
                AssetType::Raw => {
                    self.assets.insert(task.handle.id(), raw);
                }
                AssetType::Texture => {
                    let im = match image::ImageFormat::from_path(&task.path) {
                        Ok(f) => image::load_from_memory_with_format(&raw, f),
                        _ => image::load_from_memory(&raw),
                    }?;
                    let size = UVec2::new(im.width(), im.height());
                    let data = im.into_bytes();
                    let fetched_task = FetchedTask::CreateTexture {
                        handle: task.handle.clone(),
                        data,
                        size,
                    };
                    tasks.push(fetched_task);
                }
            }
        }
        Ok(tasks)
    }
}
