use crate::tokio::fs::File;

/// Possible response types for the application
#[allow(variant_size_differences)]
#[derive(Debug)]
pub enum ResponseType {
    /// Empty response
    Empty,
    /// String response
    String(String),
    /// File response
    File(File),
}
