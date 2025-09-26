//! Topic-based semantic chunking

use anyhow::Result;
use std::collections::HashMap;
use crate::strategies::chunking::*;

/// Chunk content by semantic topics
pub async fn chunk_by_topics(
    content: &str,
    similarity_threshold: f64,
    config: &ChunkingConfig,
) -> Result<Vec<ContentChunk>> {
    let mut chunks = Vec::new();

    if content.is_empty() {
        return Ok(chunks);
    }

    // Split content into sentences for topic analysis
    let sentences = split_into_sentences(content);

    if sentences.is_empty() {
        return Ok(chunks);
    }

    // Analyze topics for each sentence
    let sentence_topics = analyze_sentence_topics(&sentences);

    // Group sentences by topic similarity
    let topic_groups = group_by_topic_similarity(&sentence_topics, similarity_threshold);

    // Create chunks from topic groups
    let mut chunk_index = 0;
    let mut current_pos = 0;

    for group in topic_groups {
        let chunk_content = group.sentences.join(" ");
        let token_count = count_tokens(&chunk_content);

        // Split large topic chunks if they exceed token limit
        if token_count > config.token_max {
            let sub_chunks = split_large_topic_chunk(&group, config).await?;
            for mut sub_chunk in sub_chunks {
                sub_chunk.chunk_index = chunk_index;
                sub_chunk.start_pos = current_pos;
                sub_chunk.end_pos = current_pos + sub_chunk.content.len();
                let content_len = sub_chunk.content.len(); // Store length before moving
                chunks.push(sub_chunk);
                current_pos += content_len;
                chunk_index += 1;
            }
        } else {
            let chunk = create_topic_chunk(
                &chunk_content,
                current_pos,
                token_count,
                chunk_index,
                &group.primary_topic,
                &group.topic_keywords,
            );
            current_pos += chunk_content.len();
            chunks.push(chunk);
            chunk_index += 1;
        }
    }

    // Update total chunk count
    let total_chunks = chunks.len();
    for chunk in &mut chunks {
        chunk.total_chunks = total_chunks;
    }

    Ok(chunks)
}

/// Sentence with topic analysis
#[derive(Debug, Clone)]
struct SentenceWithTopic {
    text: String,
    position: usize,
    topic_keywords: Vec<String>,
    #[allow(dead_code)]
    topic_score: HashMap<String, f64>,
}

/// Topic group containing related sentences
#[derive(Debug)]
struct TopicGroup {
    sentences: Vec<String>,
    primary_topic: String,
    topic_keywords: Vec<String>,
    #[allow(dead_code)]
    similarity_score: f64,
}

/// Analyze topics for each sentence
fn analyze_sentence_topics(sentences: &[String]) -> Vec<SentenceWithTopic> {
    sentences
        .iter()
        .enumerate()
        .map(|(pos, sentence)| {
            let keywords = extract_topic_keywords(sentence);
            let topic_scores = calculate_topic_scores(sentence, &keywords);

            SentenceWithTopic {
                text: sentence.clone(),
                position: pos,
                topic_keywords: keywords,
                topic_score: topic_scores,
            }
        })
        .collect()
}

/// Group sentences by topic similarity
fn group_by_topic_similarity(
    sentences: &[SentenceWithTopic],
    threshold: f64,
) -> Vec<TopicGroup> {
    let mut groups = Vec::new();
    let mut used_sentences = vec![false; sentences.len()];

    for (i, sentence) in sentences.iter().enumerate() {
        if used_sentences[i] {
            continue;
        }

        let mut group_sentences = vec![sentence.text.clone()];
        let mut group_keywords = sentence.topic_keywords.clone();
        used_sentences[i] = true;

        // Find similar sentences
        for (j, other_sentence) in sentences.iter().enumerate().skip(i + 1) {
            if used_sentences[j] {
                continue;
            }

            let similarity = calculate_topic_similarity(sentence, other_sentence);
            if similarity >= threshold {
                group_sentences.push(other_sentence.text.clone());
                group_keywords.extend(other_sentence.topic_keywords.clone());
                used_sentences[j] = true;
            }
        }

        // Deduplicate keywords and find primary topic
        group_keywords.sort();
        group_keywords.dedup();
        let primary_topic = find_primary_topic(&group_keywords);

        groups.push(TopicGroup {
            sentences: group_sentences,
            primary_topic,
            topic_keywords: group_keywords,
            similarity_score: threshold,
        });
    }

    groups
}

/// Create a topic-based chunk
fn create_topic_chunk(
    content: &str,
    start_pos: usize,
    token_count: usize,
    chunk_index: usize,
    primary_topic: &str,
    topic_keywords: &[String],
) -> ContentChunk {
    let end_pos = start_pos + content.len();
    let word_count = content.split_whitespace().count();
    let sentence_count = content.split('.').filter(|s| !s.trim().is_empty()).count();
    let has_complete_sentences = content.trim().ends_with('.') ||
                                 content.trim().ends_with('!') ||
                                 content.trim().ends_with('?');

    let metadata = ChunkMetadata {
        quality_score: 0.0, // Will be calculated
        sentence_count,
        word_count,
        has_complete_sentences,
        topic_keywords: topic_keywords.to_vec(),
        chunk_type: format!("topic_{}", sanitize_topic(primary_topic)),
    };

    let quality_score = calculate_topic_chunk_quality(content, &metadata, primary_topic);

    ContentChunk {
        id: format!("topic_{}_{}", chunk_index, sanitize_topic(primary_topic)),
        content: content.to_string(),
        start_pos,
        end_pos,
        token_count,
        chunk_index,
        total_chunks: 0, // Will be updated later
        metadata: ChunkMetadata {
            quality_score,
            ..metadata
        },
    }
}

/// Calculate topic similarity between two sentences
fn calculate_topic_similarity(s1: &SentenceWithTopic, s2: &SentenceWithTopic) -> f64 {
    if s1.topic_keywords.is_empty() || s2.topic_keywords.is_empty() {
        return 0.0;
    }

    // Calculate keyword overlap
    let mut common_keywords = 0;
    for keyword in &s1.topic_keywords {
        if s2.topic_keywords.contains(keyword) {
            common_keywords += 1;
        }
    }

    let total_unique_keywords = {
        let mut all_keywords = s1.topic_keywords.clone();
        all_keywords.extend(s2.topic_keywords.clone());
        all_keywords.sort();
        all_keywords.dedup();
        all_keywords.len()
    };

    if total_unique_keywords == 0 {
        return 0.0;
    }

    // Jaccard similarity
    let jaccard = common_keywords as f64 / total_unique_keywords as f64;

    // Boost similarity for adjacent sentences
    let position_bonus = if (s1.position as i32 - s2.position as i32).abs() <= 2 {
        0.1
    } else {
        0.0
    };

    (jaccard + position_bonus).min(1.0_f64)
}

/// Calculate topic scores for keywords
fn calculate_topic_scores(sentence: &str, keywords: &[String]) -> HashMap<String, f64> {
    let mut scores = HashMap::new();
    let word_count = sentence.split_whitespace().count() as f64;

    for keyword in keywords {
        // Count occurrences of the keyword
        let occurrences = sentence.to_lowercase()
            .matches(&keyword.to_lowercase())
            .count() as f64;

        // Calculate TF (term frequency)
        let tf = occurrences / word_count;

        // Simple scoring based on term frequency and position
        let position_score = if sentence.to_lowercase().starts_with(&keyword.to_lowercase()) {
            1.2
        } else {
            1.0
        };

        scores.insert(keyword.clone(), tf * position_score);
    }

    scores
}

/// Find the primary topic from keywords
fn find_primary_topic(keywords: &[String]) -> String {
    if keywords.is_empty() {
        return "general".to_string();
    }

    // For now, use the first keyword as primary topic
    // In a more sophisticated implementation, this could use
    // topic modeling algorithms like LDA
    keywords[0].clone()
}

/// Split large topic chunks that exceed token limits
async fn split_large_topic_chunk(
    group: &TopicGroup,
    config: &ChunkingConfig,
) -> Result<Vec<ContentChunk>> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut current_tokens = 0;
    let mut chunk_index = 0;

    for sentence in &group.sentences {
        let sentence_tokens = count_tokens(sentence);

        if current_tokens + sentence_tokens > config.token_max && !current_chunk.is_empty() {
            let chunk = create_topic_chunk(
                &current_chunk,
                0, // Position will be updated later
                current_tokens,
                chunk_index,
                &group.primary_topic,
                &group.topic_keywords,
            );
            chunks.push(chunk);

            current_chunk.clear();
            current_tokens = 0;
            chunk_index += 1;
        }

        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(sentence);
        current_tokens += sentence_tokens;
    }

    // Add final chunk
    if !current_chunk.is_empty() {
        let chunk = create_topic_chunk(
            &current_chunk,
            0, // Position will be updated later
            current_tokens,
            chunk_index,
            &group.primary_topic,
            &group.topic_keywords,
        );
        chunks.push(chunk);
    }

    Ok(chunks)
}

/// Calculate quality score specific to topic chunks
fn calculate_topic_chunk_quality(content: &str, metadata: &ChunkMetadata, topic: &str) -> f64 {
    let mut score = calculate_chunk_quality(content, metadata);

    // Bonus for strong topic coherence
    let topic_mentions = content.to_lowercase()
        .matches(&topic.to_lowercase())
        .count();

    if topic_mentions > 1 {
        score += 0.1;
    }

    // Bonus for topic keywords density
    let keyword_density = metadata.topic_keywords.len() as f64 / metadata.word_count as f64;
    if keyword_density > 0.05 {
        score += 0.1;
    }

    score.min(1.0_f64)
}

/// Sanitize topic name for use in IDs
fn sanitize_topic(topic: &str) -> String {
    topic
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .take(15)
        .collect()
}

/// Simple sentence splitting for topic analysis
fn split_into_sentences(content: &str) -> Vec<String> {
    content
        .split(['.', '!', '?'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.split_whitespace().count() >= 3)
        .collect()
}