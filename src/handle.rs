use std::{
    hash::Hash,
    sync::{mpsc::Sender, Arc},
};

pub type HandleId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle(pub(crate) Arc<StrongHandle>);

impl Handle {
    pub(crate) fn new(id: u64, drop_sender: Sender<DropEvent>) -> Self {
        Self(Arc::new(StrongHandle { id, drop_sender }))
    }

    pub(crate) fn id(&self) -> HandleId {
        self.0.id
    }
}

#[derive(Debug)]
pub(crate) struct DropEvent(pub u64);

#[derive(Debug)]
pub(crate) struct StrongHandle {
    id: u64,
    drop_sender: Sender<DropEvent>,
}

impl Hash for StrongHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&self.id, state)
    }
}

impl Eq for StrongHandle {}

impl PartialEq for StrongHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Drop for StrongHandle {
    fn drop(&mut self) {
        let _ = self.drop_sender.send(DropEvent(self.id));
    }
}
