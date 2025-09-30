use crate::memimg::processor::{Command, Query};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Amount = Decimal;

#[derive(Debug, Clone)]
pub struct Bank {
    pub accounts: HashMap<String, Account>,
}

impl Bank {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }
}

impl Default for Bank {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub balance: Amount,
}

impl Account {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            balance: Amount::ZERO,
        }
    }
}

// Commands

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BankCommand {
    CreateAccount { id: String, name: String },
    Deposit { account_id: String, amount: Amount },
    Withdrawal { account_id: String, amount: Amount },
    Transfer { from_account_id: String, to_account_id: String, amount: Amount },
}

impl Command for BankCommand {
    type System = Bank;

    fn apply_to(&self, bank: &mut Self::System) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            BankCommand::CreateAccount { id, name } => {
                bank.accounts.insert(id.clone(), Account::new(id.clone(), name.clone()));
                Ok(())
            }
            BankCommand::Deposit { account_id, amount } => {
                let account = bank.accounts.get_mut(account_id)
                    .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                        Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Account not found: {}", account_id)))
                    })?;
                account.balance += *amount;
                Ok(())
            }
            BankCommand::Withdrawal { account_id, amount } => {
                let account = bank.accounts.get_mut(account_id)
                    .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                        Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Account not found: {}", account_id)))
                    })?;

                if account.balance < *amount {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Insufficient funds: {} < {}", account.balance, amount)
                    )));
                }

                account.balance -= *amount;
                Ok(())
            }
            BankCommand::Transfer { from_account_id, to_account_id, amount } => {
                // Operation order deliberately set to exercise rollback (deposit first)
                {
                    let to_account = bank.accounts.get_mut(to_account_id)
                        .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                            Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Account not found: {}", to_account_id)))
                        })?;
                    to_account.balance += *amount;
                }

                {
                    let from_account = bank.accounts.get_mut(from_account_id)
                        .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                            Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Account not found: {}", from_account_id)))
                        })?;

                    if from_account.balance < *amount {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Insufficient funds: {} < {}", from_account.balance, amount)
                        )));
                    }

                    from_account.balance -= *amount;
                }

                Ok(())
            }
        }
    }
}

// Queries

#[derive(Debug)]
pub struct GetAccount {
    pub account_id: String,
}

impl Query for GetAccount {
    type System = Bank;
    type Result = Option<Account>;

    fn extract_from(&self, bank: &Self::System) -> Result<Self::Result, Box<dyn std::error::Error + Send + Sync>> {
        Ok(bank.accounts.get(&self.account_id).cloned())
    }
}

#[derive(Debug)]
pub struct GetBalance {
    pub account_id: String,
}

impl Query for GetBalance {
    type System = Bank;
    type Result = Amount;

    fn extract_from(&self, bank: &Self::System) -> Result<Self::Result, Box<dyn std::error::Error + Send + Sync>> {
        bank.accounts
            .get(&self.account_id)
            .map(|acc| acc.balance)
            .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Account not found: {}", self.account_id)
                ))
            })
    }
}

#[derive(Debug)]
pub struct ListAccounts;

impl Query for ListAccounts {
    type System = Bank;
    type Result = Vec<Account>;

    fn extract_from(&self, bank: &Self::System) -> Result<Self::Result, Box<dyn std::error::Error + Send + Sync>> {
        Ok(bank.accounts.values().cloned().collect())
    }
}
