use crate::error::{Result, TranscriptError};
use serde::{Deserialize, Serialize};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: String,
}

pub struct ChatGPT {
    client: reqwest::Client,
    api_key: String,
}

impl ChatGPT {
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let api_key = api_key
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| TranscriptError::HttpError(
                "OpenAI API key not found. Set OPENAI_API_KEY environment variable or use --openai-key flag".to_string()
            ))?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
        })
    }

    pub async fn cleanup_transcript(
        &self,
        transcript_text: &str,
        format_markdown: bool,
    ) -> Result<String> {
        let format_instruction = if format_markdown {
            "Format the cleaned transcript using Markdown syntax. Use appropriate markdown elements like:\n\
            - **Bold** for emphasis on important points\n\
            - *Italics* for subtle emphasis\n\
            - Headings (##, ###) to organize sections if the transcript has clear topics\n\
            - Bullet points (-) or numbered lists (1.) for lists\n\
            - Blockquotes (>) for notable quotes\n\
            - Line breaks between paragraphs\n\
            Make it well-structured and readable with proper markdown formatting.\n\n"
        } else {
            ""
        };

        let prompt = format!(
            "Please clean up and improve the following transcript. \
            Fix any grammar errors, improve sentence structure, remove filler words and repetitions, \
            and make it more readable while preserving the original meaning and content. \
            Do not add any information that wasn't in the original transcript.\n\n\
            IMPORTANT: Remove all references to products, websites, courses, training programs, \
            email addresses, social media handles, or any promotional content that the presenter may offer. \
            Focus only on the educational or informational content.\n\n\
            {}\
            Transcript:\n\n{}",
            format_instruction,
            transcript_text
        );

        let request = ChatRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful assistant that cleans up and improves transcripts while preserving their original meaning. You remove promotional content like product mentions, website URLs, course offers, and training programs.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.3,
        };

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| TranscriptError::HttpError(format!("Failed to call OpenAI API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(TranscriptError::HttpError(format!(
                "OpenAI API error ({}): {}",
                status, error_text
            )));
        }

        let chat_response: ChatResponse = response.json().await.map_err(|e| {
            TranscriptError::JsonParseError(format!("Failed to parse OpenAI response: {}", e))
        })?;

        let cleaned_text = chat_response
            .choices
            .first()
            .and_then(|choice| Some(choice.message.content.clone()))
            .ok_or_else(|| TranscriptError::HttpError("No response from OpenAI API".to_string()))?;

        Ok(cleaned_text.trim().to_string())
    }
}
