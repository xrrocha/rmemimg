# Memory Image in Rust

> When people start an enterprise application, one of the earliest questions is "how do we talk to the database". These days they may ask a slightly different question: "what kind of database should we use - relational or one of these NOSQL databases?".
>
> But there's another question to consider: "should we use a database at all?"
>
> -- Martin Fowler

This repository contains a Rust implementation of the **Memory Image** architectural pattern, inspired by the Kotlin implementation found at [kmemimg](https://rrocha.me/projects/kmemimg/).

## What is a Memory Image?

A memory image is a pattern where the entire application state is kept in main memory. Instead of persisting the domain objects themselves, the sequence of commands that modify the state are persisted. This provides several advantages:

*   **Performance:** By keeping the state in memory, read operations are incredibly fast.
*   **Simplicity:** It can simplify the design by removing the need for an Object-Relational Mapper (ORM) and reducing the impedance mismatch between the object-oriented domain model and a relational database.
*   **Rich Domain Models:** It allows for the creation of rich domain models without being constrained by the limitations of a database.

The key idea is:

> Serialize all state-modifying commands on persistent storage.
>
> Reconstruct in-memory application state by replaying the deserialized commands onto an empty initial state.

## Memory Image Processor in Rust

The core of this implementation is the `MemImgProcessor`. It manages the in-memory system state, applies commands, and replays events from storage.

Here's a simplified look at the key components:

```rust
// Trait for commands that mutate system state
pub trait Command: Debug {
    type System: Clone;

    fn apply_to(&self, system: &mut Self::System) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

// Trait for queries that extract data from system state
pub trait Query: Debug {
    type System;
    type Result;

    fn extract_from(&self, system: &Self::System) -> Result<Self::Result, Box<dyn std::error::Error + Send + Sync>>;
}

// Memory Image Processor - manages in-memory system state with event sourcing
pub struct MemImgProcessor<S, C, E>
where
    S: Clone,
    C: Command<System = S>,
    E: EventStorage<Event = C>,
{
    pub system: S,
    pub event_storage: Box<E>,
}
```

## Example: Bank Domain Model

This repository includes a simple banking application to demonstrate the memory image pattern. The domain model consists of a `Bank` that holds a collection of `Account`s. The state of the bank is modified by applying `BankCommand`s such as `CreateAccount`, `Deposit`, and `Transfer`.

```rust
// Bank domain model
#[derive(Debug, Clone)]
pub struct Bank {
    pub accounts: HashMap<String, Account>,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub balance: Amount,
}

// Commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BankCommand {
    CreateAccount { id: String, name: String },
    Deposit { account_id: String, amount: Amount },
    Withdrawal { account_id: String, amount: Amount },
    Transfer { from_account_id: String, to_account_id: String, amount: Amount },
}
```

## Building and Running

**Building the project:**

```bash
cargo build
```

**Running the application:**

```bash
cargo run
```

This will execute the `main` function in `src/main.rs`, which creates a bank, executes some transactions, and prints the final balances. The events are stored in a file named `bank_events.json`.

**Running the tests:**

```bash
cargo test
```

## Development Conventions

*   **Domain Logic:** The domain logic is kept separate from the persistence mechanism.
*   **Generic Event Storage:** The `EventStorage` trait is generic and can be implemented for different storage backends.
*   **Command and Query Separation:** The pattern separates commands (which modify state) from queries (which read state).
*   **Transactional Command Execution:** The `MemImgProcessor` uses a shadow copy mechanism to ensure that commands are applied atomically.
*   **Testing:** The project has a suite of tests that cover the core functionality of the `MemImgProcessor` and the banking application.
''
