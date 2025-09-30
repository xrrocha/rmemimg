use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::marker::PhantomData;
use std::path::Path;

/// Trait for event storage backends
pub trait EventStorage {
    type Event;

    fn replay<F>(&mut self, consumer: &mut F) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnMut(Self::Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn append(&mut self, event: &Self::Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Trait for converting events to/from text format
pub trait TextConverter<T> {
    fn parse(&self, text: &str) -> Result<T, Box<dyn std::error::Error + Send + Sync>>;
    fn format(&self, value: &T) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

/// File-based event storage using line-oriented text format
pub struct TextFileEventStorage<E, C>
where
    C: TextConverter<E>,
{
    file_path: String,
    converter: C,
    writer: Option<File>,
    _phantom: PhantomData<E>,
}

impl<E, C> TextFileEventStorage<E, C>
where
    C: TextConverter<E>,
{
    pub fn new<P: AsRef<Path>>(path: P, converter: C) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = path.as_ref().to_string_lossy().to_string();

        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        }

        // Create file if it doesn't exist
        if !path.as_ref().exists() {
            File::create(&path).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        }

        Ok(Self {
            file_path,
            converter,
            writer: None,
            _phantom: PhantomData,
        })
    }
}

impl<E, C> EventStorage for TextFileEventStorage<E, C>
where
    C: TextConverter<E>,
{
    type Event = E;

    fn replay<F>(&mut self, consumer: &mut F) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnMut(Self::Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        let file = File::open(&self.file_path).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
            if !line.trim().is_empty() {
                let event = self.converter.parse(&line)?;
                consumer(event)?;
            }
        }

        Ok(())
    }

    fn append(&mut self, event: &Self::Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Lazy-open writer after replay
        if self.writer.is_none() {
            self.writer = Some(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.file_path)
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?,
            );
        }

        let text = self.converter.format(event)?;
        if let Some(writer) = &mut self.writer {
            writeln!(writer, "{}", text).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
            writer.flush().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        }

        Ok(())
    }
}

impl<E, C> Drop for TextFileEventStorage<E, C>
where
    C: TextConverter<E>,
{
    fn drop(&mut self) {
        if let Some(mut writer) = self.writer.take() {
            let _ = writer.flush();
        }
    }
}
