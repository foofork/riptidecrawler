//! Quality assessment for extracted content

pub struct QualityAssessor;

impl QualityAssessor {
    /// Calculate quality score (0-100)
    pub fn calculate(text: &str, markdown: &Option<String>, title: &Option<String>) -> usize {
        let mut score = 0;

        // Title presence (20 points)
        if title.as_ref().is_some_and(|t| !t.trim().is_empty()) {
            score += 20;
        }

        // Content length (40 points)
        let text_length = text.len();
        if text_length > 2000 {
            score += 40;
        } else if text_length > 500 {
            score += 25;
        } else if text_length > 100 {
            score += 10;
        }

        // Markdown structure (20 points)
        if let Some(ref md) = markdown {
            let structure_indicators =
                md.matches('#').count() + md.matches('*').count() + md.matches('[').count();

            if structure_indicators > 10 {
                score += 20;
            } else if structure_indicators > 5 {
                score += 12;
            } else if structure_indicators > 2 {
                score += 6;
            }
        }

        // Word count (10 points)
        let word_count = text.split_whitespace().count();
        if word_count > 500 {
            score += 10;
        } else if word_count > 100 {
            score += 5;
        }

        // Sentence structure (10 points)
        let sentence_count = text.matches('.').count();
        if sentence_count > 10 {
            score += 10;
        } else if sentence_count > 3 {
            score += 5;
        }

        score.min(100)
    }
}
