use trek_rs::TrekResponse;

/// Calculate content quality score based on trek-rs TrekResponse (0-100)
pub fn calculate_quality_score(response: &TrekResponse) -> u8 {
    let mut score = 30u8; // Base score

    // Title quality (0-15 points)
    if !response.metadata.title.is_empty() {
        let title_len = response.metadata.title.len();
        if title_len > 10 && title_len < 100 {
            score += 15;
        } else if title_len > 5 {
            score += 8;
        }
    }

    // Content length (0-20 points)
    let content_len = response.content.len();
    if content_len > 2000 {
        score += 20;
    } else if content_len > 1000 {
        score += 15;
    } else if content_len > 500 {
        score += 10;
    } else if content_len > 200 {
        score += 5;
    }

    // Author/byline (0-10 points)
    if !response.metadata.author.is_empty() {
        score += 10;
    }

    // Publication date (0-10 points)
    if !response.metadata.published.is_empty() {
        score += 10;
    }

    // Word count (0-10 points)
    if response.metadata.word_count > 500 {
        score += 10;
    } else if response.metadata.word_count > 200 {
        score += 5;
    }

    // Meta tags (0-5 points)
    if !response.meta_tags.is_empty() {
        score += 5;
    }

    score.min(100)
}

/// Estimate reading time in minutes based on word count
pub fn estimate_reading_time(word_count: usize) -> Option<u32> {
    if word_count == 0 {
        return None;
    }

    // Average reading speed: 200-250 words per minute
    let reading_time = (word_count as f32 / 225.0).ceil() as u32;
    Some(reading_time.max(1))
}

/// Count words in text
#[allow(dead_code)]
pub fn count_words(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

/// Get trek-rs version
pub fn get_trek_version() -> String {
    // Use the actual version we're using
    "0.1.1".to_string()
}
