//! Structured data types for specific schemas

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Format-specific structured data
///
/// This enum allows adding new schema types without breaking existing code.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum StructuredData {
    /// Event data (conferences, meetups, etc.)
    Event { event: Event },

    /// Product data (e-commerce items)
    Product { product: Product },

    // Future schemas can be added here without breaking existing code:
    // Article { article: Article },
    // Recipe { recipe: Recipe },
}

/// Event data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    /// Event title/name
    pub title: String,

    /// Event description
    pub description: String,

    /// Start date and time
    pub start_date: Option<DateTime<Utc>>,

    /// End date and time
    pub end_date: Option<DateTime<Utc>>,

    /// Physical or virtual location
    pub location: Option<String>,

    /// Organizer name or organization
    pub organizer: Option<String>,

    /// Event URL
    pub url: Option<String>,
}

/// Product data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    /// Product name
    pub name: String,

    /// Product description
    pub description: String,

    /// Price (as string to handle various formats)
    pub price: Option<String>,

    /// Currency code (USD, EUR, etc.)
    pub currency: Option<String>,

    /// Availability status
    pub availability: Option<String>,

    /// Product brand
    pub brand: Option<String>,

    /// SKU or product ID
    pub sku: Option<String>,

    /// Product images
    pub images: Vec<String>,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            start_date: None,
            end_date: None,
            location: None,
            organizer: None,
            url: None,
        }
    }
}

impl Default for Product {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            price: None,
            currency: None,
            availability: None,
            brand: None,
            sku: None,
            images: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = Event {
            title: "Rust Meetup".to_string(),
            description: "Monthly Rust gathering".to_string(),
            ..Default::default()
        };

        let data = StructuredData::Event { event };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("Rust Meetup"));
    }

    #[test]
    fn test_product_serialization() {
        let product = Product {
            name: "Laptop".to_string(),
            price: Some("999.99".to_string()),
            currency: Some("USD".to_string()),
            ..Default::default()
        };

        let data = StructuredData::Product { product };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("Laptop"));
        assert!(json.contains("999.99"));
    }
}
