use crate::memimg::error::{FailureOutcome, MemImgError};
use crate::memimg::storage::EventStorage;
use std::fmt::Debug;

/// Trait for commands that mutate system state
pub trait Command: Debug {
    type System: Clone;

    fn apply_to(&self, system: &mut Self::System) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Trait for queries that extract data from system state
pub trait Query: Debug {
    type System;
    type Result;

    fn extract_from(&self, system: &Self::System) -> Result<Self::Result, Box<dyn std::error::Error + Send + Sync>>;
}

/// Memory Image Processor - manages in-memory system state with event sourcing
pub struct MemImgProcessor<S, C, E>
where
    S: Clone,
    C: Command<System = S>,
    E: EventStorage<Event = C>,
{
    pub system: S,
    pub event_storage: Box<E>,
}

impl<S, C, E> MemImgProcessor<S, C, E>
where
    S: Clone,
    C: Command<System = S>,
    E: EventStorage<Event = C>,
{
    /// Create a new processor, replaying all events from storage
    pub fn new(mut system: S, mut event_storage: Box<E>) -> Result<Self, MemImgError> {
        event_storage.replay(&mut |command: C| {
            command.apply_to(&mut system)
        }).map_err(|e| {
            MemImgError::SystemFailure(FailureOutcome::new(
                e,
                "replaying events",
                "EventStorage",
            ))
        })?;

        Ok(Self {
            system,
            event_storage,
        })
    }

    /// Execute a query against the current system state
    pub fn execute_query<Q>(&self, query: &Q) -> Result<Q::Result, MemImgError>
    where
        Q: Query<System = S>,
    {
        query.extract_from(&self.system).map_err(|e| {
            MemImgError::CommandFailure(FailureOutcome::new(
                e,
                "executing query",
                std::any::type_name::<Q>(),
            ))
        })
    }

    /// Execute a command with shadow-copy transaction semantics
    pub fn execute_command(&mut self, command: C) -> Result<(), MemImgError> {
        // Shadow copy: clone the entire system state
        let mut shadow = self.system.clone();

        // Apply command to shadow copy
        command.apply_to(&mut shadow).map_err(|e| {
            MemImgError::CommandFailure(FailureOutcome::new(
                e,
                "executing command",
                std::any::type_name::<C>(),
            ))
        })?;

        // Serialize command before committing
        self.event_storage.append(&command).map_err(|e| {
            MemImgError::SystemFailure(FailureOutcome::new(
                e,
                "serializing command",
                std::any::type_name::<C>(),
            ))
        })?;

        // Commit: swap shadow copy into main system
        self.system = shadow;

        Ok(())
    }

    /// Get immutable reference to system state
    pub fn system(&self) -> &S {
        &self.system
    }
}

impl<S, C, E> Drop for MemImgProcessor<S, C, E>
where
    S: Clone,
    C: Command<System = S>,
    E: EventStorage<Event = C>,
{
    fn drop(&mut self) {
        // EventStorage cleanup handled by its own Drop implementation
    }
}
