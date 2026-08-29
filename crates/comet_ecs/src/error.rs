use crate::Entity;
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EcsError {
    #[error("entity {0:?} does not exist")]
    EntityNotFound(Entity),
    #[error("bundle contains component {component} more than once")]
    DuplicateComponent { component: &'static str },
    #[error("{dependent} needs {missing}")]
    MissingNeededComponent {
        dependent: &'static str,
        missing: &'static str,
    },
}
