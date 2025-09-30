use crate::memimg::bank::BankCommand;
use crate::memimg::storage::TextConverter;

/// JSON converter for BankCommand
pub struct BankJsonConverter;

impl TextConverter<BankCommand> for BankJsonConverter {
    fn parse(&self, text: &str) -> Result<BankCommand, Box<dyn std::error::Error + Send + Sync>> {
        serde_json::from_str(text).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
    }

    fn format(&self, command: &BankCommand) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure JSON is on a single line
        let json = serde_json::to_string(command).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        Ok(json.replace('\n', " "))
    }
}
