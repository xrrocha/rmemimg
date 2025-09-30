use rmemimg::memimg::bank::{Bank, BankCommand, GetAccount, GetBalance};
use rmemimg::memimg::bank_storage::BankJsonConverter;
use rmemimg::memimg::{EventStorage, MemImgProcessor, TextFileEventStorage};
use rust_decimal::Decimal;

// In-memory event storage for testing
struct MemoryEventStorage {
    events: Vec<BankCommand>,
}

impl MemoryEventStorage {
    fn new() -> Self {
        Self { events: Vec::new() }
    }
}

impl EventStorage for MemoryEventStorage {
    type Event = BankCommand;

    fn replay<F>(&mut self, consumer: &mut F) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnMut(Self::Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        for event in &self.events {
            consumer(event.clone())?;
        }
        Ok(())
    }

    fn append(&mut self, event: &Self::Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.events.push(event.clone());
        Ok(())
    }
}

#[test]
fn executes_and_serializes_successful_command() {
    let bank = Bank::new();
    let storage = Box::new(MemoryEventStorage::new());
    let mut processor = MemImgProcessor::new(bank, storage).unwrap();

    let cmd1 = BankCommand::CreateAccount {
        id: "acc1".to_string(),
        name: "Alice".to_string(),
    };
    let cmd2 = BankCommand::CreateAccount {
        id: "acc2".to_string(),
        name: "Bob".to_string(),
    };

    processor.execute_command(cmd1).unwrap();
    processor.execute_command(cmd2).unwrap();

    assert_eq!(processor.system().accounts.len(), 2);
}

#[test]
fn initializes_from_previous_commands() {
    let storage = Box::new(MemoryEventStorage::new());
    let bank1 = Bank::new();
    let mut processor1 = MemImgProcessor::new(bank1, storage).unwrap();

    processor1
        .execute_command(BankCommand::CreateAccount {
            id: "acc1".to_string(),
            name: "Alice".to_string(),
        })
        .unwrap();

    processor1
        .execute_command(BankCommand::Deposit {
            account_id: "acc1".to_string(),
            amount: Decimal::new(100, 0),
        })
        .unwrap();

    // Extract storage to reuse
    let storage = std::mem::replace(
        &mut processor1.event_storage,
        Box::new(MemoryEventStorage::new()),
    );

    // Create new processor with same storage
    let bank2 = Bank::new();
    let processor2 = MemImgProcessor::new(bank2, storage).unwrap();

    assert_eq!(processor2.system().accounts.len(), 1);
    assert_eq!(
        processor2.system().accounts.get("acc1").unwrap().balance,
        Decimal::new(100, 0)
    );
}

#[test]
fn executes_query() {
    let bank = Bank::new();
    let storage = Box::new(MemoryEventStorage::new());
    let mut processor = MemImgProcessor::new(bank, storage).unwrap();

    processor
        .execute_command(BankCommand::CreateAccount {
            id: "acc1".to_string(),
            name: "Alice".to_string(),
        })
        .unwrap();

    processor
        .execute_command(BankCommand::Deposit {
            account_id: "acc1".to_string(),
            amount: Decimal::new(100, 0),
        })
        .unwrap();

    let query = GetAccount {
        account_id: "acc1".to_string(),
    };
    let result = processor.execute_query(&query).unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap().balance, Decimal::new(100, 0));
}

#[test]
fn signals_failure_on_failed_query() {
    let bank = Bank::new();
    let storage = Box::new(MemoryEventStorage::new());
    let processor = MemImgProcessor::new(bank, storage).unwrap();

    let query = GetBalance {
        account_id: "nonexistent".to_string(),
    };
    let result = processor.execute_query(&query);

    assert!(result.is_err());
}

#[test]
fn rolls_back_partial_updates_on_failed_command() {
    let bank = Bank::new();
    let storage = Box::new(MemoryEventStorage::new());
    let mut processor = MemImgProcessor::new(bank, storage).unwrap();

    processor
        .execute_command(BankCommand::CreateAccount {
            id: "acc1".to_string(),
            name: "Alice".to_string(),
        })
        .unwrap();

    processor
        .execute_command(BankCommand::Deposit {
            account_id: "acc1".to_string(),
            amount: Decimal::new(100, 0),
        })
        .unwrap();

    // Try to withdraw more than balance - should fail
    let result = processor.execute_command(BankCommand::Withdrawal {
        account_id: "acc1".to_string(),
        amount: Decimal::new(200, 0),
    });

    assert!(result.is_err());
    // Balance should remain 100
    assert_eq!(
        processor.system().accounts.get("acc1").unwrap().balance,
        Decimal::new(100, 0)
    );
}

#[test]
fn transfer_rolls_back_on_insufficient_funds() {
    let bank = Bank::new();
    let storage = Box::new(MemoryEventStorage::new());
    let mut processor = MemImgProcessor::new(bank, storage).unwrap();

    processor
        .execute_command(BankCommand::CreateAccount {
            id: "acc1".to_string(),
            name: "Alice".to_string(),
        })
        .unwrap();

    processor
        .execute_command(BankCommand::CreateAccount {
            id: "acc2".to_string(),
            name: "Bob".to_string(),
        })
        .unwrap();

    processor
        .execute_command(BankCommand::Deposit {
            account_id: "acc1".to_string(),
            amount: Decimal::new(50, 0),
        })
        .unwrap();

    // Try transfer more than available - should fail and rollback
    let result = processor.execute_command(BankCommand::Transfer {
        from_account_id: "acc1".to_string(),
        to_account_id: "acc2".to_string(),
        amount: Decimal::new(100, 0),
    });

    assert!(result.is_err());
    // Both balances should remain as before
    assert_eq!(
        processor.system().accounts.get("acc1").unwrap().balance,
        Decimal::new(50, 0)
    );
    assert_eq!(
        processor.system().accounts.get("acc2").unwrap().balance,
        Decimal::ZERO
    );
}

#[test]
fn successful_transfer() {
    let bank = Bank::new();
    let storage = Box::new(MemoryEventStorage::new());
    let mut processor = MemImgProcessor::new(bank, storage).unwrap();

    processor
        .execute_command(BankCommand::CreateAccount {
            id: "acc1".to_string(),
            name: "Alice".to_string(),
        })
        .unwrap();

    processor
        .execute_command(BankCommand::CreateAccount {
            id: "acc2".to_string(),
            name: "Bob".to_string(),
        })
        .unwrap();

    processor
        .execute_command(BankCommand::Deposit {
            account_id: "acc1".to_string(),
            amount: Decimal::new(100, 0),
        })
        .unwrap();

    processor
        .execute_command(BankCommand::Transfer {
            from_account_id: "acc1".to_string(),
            to_account_id: "acc2".to_string(),
            amount: Decimal::new(30, 0),
        })
        .unwrap();

    assert_eq!(
        processor.system().accounts.get("acc1").unwrap().balance,
        Decimal::new(70, 0)
    );
    assert_eq!(
        processor.system().accounts.get("acc2").unwrap().balance,
        Decimal::new(30, 0)
    );
}

#[test]
fn text_file_storage_round_trip() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_bank_events.json");

    // Clean up if exists
    let _ = std::fs::remove_file(&test_file);

    {
        let bank = Bank::new();
        let storage = Box::new(
            TextFileEventStorage::new(&test_file, BankJsonConverter).unwrap(),
        );
        let mut processor = MemImgProcessor::new(bank, storage).unwrap();

        processor
            .execute_command(BankCommand::CreateAccount {
                id: "acc1".to_string(),
                name: "Alice".to_string(),
            })
            .unwrap();

        processor
            .execute_command(BankCommand::Deposit {
                account_id: "acc1".to_string(),
                amount: Decimal::new(250, 0),
            })
            .unwrap();
    }

    // Reload from file
    {
        let bank = Bank::new();
        let storage = Box::new(
            TextFileEventStorage::new(&test_file, BankJsonConverter).unwrap(),
        );
        let processor = MemImgProcessor::new(bank, storage).unwrap();

        assert_eq!(processor.system().accounts.len(), 1);
        assert_eq!(
            processor.system().accounts.get("acc1").unwrap().balance,
            Decimal::new(250, 0)
        );
    }

    // Clean up
    let _ = std::fs::remove_file(&test_file);
}
