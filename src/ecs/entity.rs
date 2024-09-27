/// Entity
/// Use this to get entity from engine.
/// The index of EntityRef may be changed due to reordering,
///  so it is suggest to only use id to build relation.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Ent {
    pub(crate) index: u32,
}

impl Default for Ent {
    fn default() -> Self {
        Self { index: u32::MAX }
    }
}
