use rmemimg::memimg::bank::{Bank, BankCommand, GetBalance};
use rmemimg::memimg::bank_storage::BankJsonConverter;
use rmemimg::memimg::{MemImgProcessor, TextFileEventStorage};
use rust_decimal::Decimal;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Memory Image Pattern Demo ===\n");

    // Create bank and event storage
    let bank = Bank::new();
    let storage = Box::new(TextFileEventStorage::new("bank_events.json", BankJsonConverter)?);
    let mut processor = MemImgProcessor::new(bank, storage).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) })?;

    // Execute commands
    println!("Creating accounts...");
    processor.execute_command(BankCommand::CreateAccount {
        id: "alice".to_string(),
        name: "Alice".to_string(),
    }).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) })?;

    processor.execute_command(BankCommand::CreateAccount {
        id: "bob".to_string(),
        name: "Bob".to_string(),
    }).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) })?;

    println!("Depositing $1000 to Alice's account...");
    processor.execute_command(BankCommand::Deposit {
        account_id: "alice".to_string(),
        amount: Decimal::new(1000, 0),
    }).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) })?;

    println!("Transferring $300 from Alice to Bob...");
    processor.execute_command(BankCommand::Transfer {
        from_account_id: "alice".to_string(),
        to_account_id: "bob".to_string(),
        amount: Decimal::new(300, 0),
    }).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) })?;

    // Query balances
    let alice_balance = processor.execute_query(&GetBalance {
        account_id: "alice".to_string(),
    }).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) })?;
    let bob_balance = processor.execute_query(&GetBalance {
        account_id: "bob".to_string(),
    }).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) })?;

    println!("\n=== Final Balances ===");
    println!("Alice: ${}", alice_balance);
    println!("Bob: ${}", bob_balance);

    println!("\nAll commands saved to bank_events.json");
    println!("Try running again to see state restored from events!");

    Ok(())
}
