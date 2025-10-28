//! Extraction modules for different content types

pub mod categories;
pub mod content;
pub mod language;
pub mod links;
pub mod media;
pub mod metadata;
pub mod title;

// Re-export extractors
pub use categories::CategoryExtractor;
pub use content::ContentExtractor;
pub use language::LanguageDetector;
pub use links::LinkExtractor;
pub use media::MediaExtractor;
pub use metadata::MetadataExtractor;
pub use title::TitleExtractor;
