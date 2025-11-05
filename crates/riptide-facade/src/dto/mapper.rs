//! Mapper trait for converting internal models to DTOs

/// Trait for converting internal extraction models to public DTOs
///
/// This trait provides a standard interface for mapping internal types
/// to their public DTO representations.
///
/// # Examples
///
/// ```
/// use riptide_facade::dto::{ToDto, Document};
///
/// struct InternalResult {
///     url: String,
///     title: String,
///     body: String,
/// }
///
/// impl ToDto<Document> for InternalResult {
///     fn to_dto(&self) -> Document {
///         Document::new(
///             self.url.clone(),
///             self.title.clone(),
///             self.body.clone(),
///         )
///     }
/// }
/// ```
pub trait ToDto<T> {
    /// Convert this type to a DTO
    fn to_dto(&self) -> T;
}

// Example implementation for common types
impl<T> ToDto<T> for T
where
    T: Clone,
{
    fn to_dto(&self) -> T {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::Document;

    struct MockInternal {
        url: String,
        title: String,
        content: String,
    }

    impl ToDto<Document> for MockInternal {
        fn to_dto(&self) -> Document {
            Document::new(self.url.clone(), self.title.clone(), self.content.clone())
        }
    }

    #[test]
    fn test_to_dto_conversion() {
        let internal = MockInternal {
            url: "https://example.com".to_string(),
            title: "Test".to_string(),
            content: "Content".to_string(),
        };

        let doc = internal.to_dto();
        assert_eq!(doc.url, "https://example.com");
        assert_eq!(doc.title, "Test");
    }
}
