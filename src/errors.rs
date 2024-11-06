use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no entity")]
    NoEntity,
    #[error("no resource")]
    NoResource,
    #[error("no component")]
    NoComponent,
}
