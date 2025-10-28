/// Calculate basic content quality score (0-100)
/// This is a scraper-based implementation
pub fn calculate_basic_quality_score(
    title_len: usize,
    content_len: usize,
    has_author: bool,
    has_date: bool,
    word_count: usize,
) -> u8 {
    let mut score = 30u8; // Base score

    // Title quality (0-15 points)
    if title_len > 10 && title_len < 100 {
        score += 15;
    } else if title_len > 5 {
        score += 8;
    }

    // Content length (0-20 points)
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
    if has_author {
        score += 10;
    }

    // Publication date (0-10 points)
    if has_date {
        score += 10;
    }

    // Word count (0-10 points)
    if word_count > 500 {
        score += 10;
    } else if word_count > 200 {
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

/// Get extractor version (tl-based extraction engine)
pub fn get_extractor_version() -> String {
    "tl-0.7".to_string()
}

// Note: extract_links, extract_media, detect_language, and extract_categories
// are now implemented in extraction.rs using the tl parser
