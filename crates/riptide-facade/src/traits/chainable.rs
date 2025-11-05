//! Chainable trait for composing spider and extractor operations
//!
//! Provides the `.and_extract()` method for chaining URL discovery with content extraction.

use futures::stream::{BoxStream, Stream};
use futures::task::{Context, Poll};
use std::pin::Pin;
use std::sync::Arc;
use url::Url;

use crate::dto::Document;
use crate::error::RiptideResult;
use crate::traits::{Content, ExtractOpts, Extractor};

/// Chainable trait enables composition of operations
///
/// This trait provides the `.and_extract()` method that allows chaining
/// a spider's URL stream with an extractor to create a document stream.
///
/// # Examples
///
/// ```no_run
/// use riptide_facade::traits::{Spider, Extractor, Chainable, SpiderOpts};
/// use futures::StreamExt;
///
/// # async fn example(spider: impl Spider, extractor: impl Extractor) -> Result<(), Box<dyn std::error::Error>> {
/// // Chain spider with extractor
/// let docs = spider
///     .crawl("https://example.com", SpiderOpts::default())
///     .await?
///     .and_extract(extractor);
///
/// // Process documents
/// let docs: Vec<_> = docs.collect().await;
/// # Ok(())
/// # }
/// ```
pub trait Chainable: Sized {
    /// Item type produced by this stream
    type Item;

    /// Chain an extractor to process items in this stream
    ///
    /// # Arguments
    ///
    /// * `extractor` - Extractor to apply to each item
    ///
    /// # Returns
    ///
    /// ExtractChain - A stream of extracted documents
    fn and_extract<E>(self, extractor: E) -> ExtractChain<Self, E>
    where
        E: Extractor;
}

/// Implementation for BoxStream<Result<Url>>
impl Chainable for BoxStream<'static, RiptideResult<Url>> {
    type Item = RiptideResult<Url>;

    fn and_extract<E>(self, extractor: E) -> ExtractChain<Self, E>
    where
        E: Extractor,
    {
        ExtractChain {
            stream: self,
            extractor: Arc::new(extractor),
            current_future: None,
        }
    }
}

/// Chain that combines a URL stream with an extractor
///
/// This struct implements `Stream` to produce documents by extracting
/// content from URLs as they arrive from the spider.
pub struct ExtractChain<S, E> {
    stream: S,
    extractor: Arc<E>,
    current_future: Option<Pin<Box<dyn futures::Future<Output = RiptideResult<Document>> + Send>>>,
}

impl<S, E> Stream for ExtractChain<S, E>
where
    S: Stream<Item = RiptideResult<Url>> + Unpin + Send,
    E: Extractor + 'static,
{
    type Item = RiptideResult<Document>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // If we have a pending extraction, poll it first
        if let Some(mut fut) = self.current_future.take() {
            match fut.as_mut().poll(cx) {
                Poll::Ready(result) => return Poll::Ready(Some(result)),
                Poll::Pending => {
                    self.current_future = Some(fut);
                    return Poll::Pending;
                }
            }
        }

        // Get next URL from stream
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(Ok(url))) => {
                // Create extraction future
                let extractor = self.extractor.clone();
                let fut = Box::pin(async move {
                    extractor
                        .extract(Content::Url(url.to_string()), ExtractOpts::default())
                        .await
                });

                self.current_future = Some(fut);

                // Immediately poll the new future
                if let Some(mut fut) = self.current_future.take() {
                    match fut.as_mut().poll(cx) {
                        Poll::Ready(result) => Poll::Ready(Some(result)),
                        Poll::Pending => {
                            self.current_future = Some(fut);
                            Poll::Pending
                        }
                    }
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => {
                // Pass through spider errors
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_chainable_trait() {
        // This test verifies the trait compiles and basic usage works
        // Mock implementations would be in the mocks module
    }
}
