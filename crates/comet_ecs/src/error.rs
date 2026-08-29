use crate::Entity;
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EcsError {
    #[error("entity {0:?} does not exist")]
    EntityNotFound(Entity),
    #[error("{dependent} needs {missing}")]
    MissingNeededComponent {
        dependent: &'static str,
        missing: &'static str,
    },
}
