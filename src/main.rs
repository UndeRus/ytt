use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use ytt::chatgpt::ChatGPT;
use ytt::{TranscriptError, TranscriptItem, YouTubeTranscript};

#[derive(Parser)]
#[command(name = "ytt")]
#[command(about = "YouTube Transcript API - Fetch transcripts from YouTube videos", long_about = None)]
struct Args {
    /// YouTube video URL or video ID
    video: String,

    /// Language codes (e.g., en, es, fr). Can specify multiple.
    #[arg(short, long)]
    languages: Option<Vec<String>>,

    /// Translate transcript to this language code
    #[arg(short, long)]
    translate: Option<String>,

    /// Output format: json, text, txt, srt, or markdown
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Show transcript text with timestamps (deprecated: timestamps removed by default)
    #[arg(long)]
    timestamps: bool,

    /// List available transcripts instead of fetching
    #[arg(long)]
    list: bool,

    /// Delay between requests in milliseconds (default: 500ms)
    #[arg(long, default_value = "500")]
    delay: u64,

    /// Clean up transcript using ChatGPT (requires OPENAI_API_KEY env var or --openai-key)
    #[arg(long)]
    cleanup: bool,

    /// OpenAI API key (alternative to OPENAI_API_KEY env var)
    #[arg(long)]
    openai_key: Option<String>,

    /// Output file path (if not specified, outputs to stdout)
    #[arg(short, long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(e) = run(args).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<(), TranscriptError> {
    let video_id = YouTubeTranscript::extract_video_id(&args.video)?;

    let api = YouTubeTranscript::with_delay(args.delay);

    if args.list {
        let transcript_list = api.list_transcripts(&video_id).await?;
        println!("Available transcripts for video: {}", video_id);
        println!("\nManually created:");
        for transcript in transcript_list.manually_created.values() {
            println!("  {} ({})", transcript.language, transcript.language_code);
        }
        println!("\nAuto-generated:");
        for transcript in transcript_list.generated.values() {
            println!("  {} ({})", transcript.language, transcript.language_code);
        }
        if !transcript_list.translation_languages.is_empty() {
            println!("\nTranslation languages:");
            for lang in &transcript_list.translation_languages {
                println!("  {} ({})", lang.language, lang.language_code);
            }
        }
        return Ok(());
    }

    println!("Fetching transcript for video: {}", video_id);

    let transcript = if let Some(target_lang) = &args.translate {
        let source_langs: Vec<&str> = args
            .languages
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_else(|| vec!["en"]);
        api.translate_transcript(&video_id, &source_langs, target_lang)
            .await?
    } else {
        let lang_codes: Option<Vec<&str>> = args
            .languages
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());
        api.fetch_transcript(&video_id, lang_codes).await?
    };

    // Determine if we need markdown formatting from ChatGPT
    let format_markdown = args.cleanup
        && (args.format.to_lowercase() == "markdown" || args.format.to_lowercase() == "md");

    // If cleanup is requested, send to ChatGPT first
    let transcript_items = if args.cleanup {
        eprintln!("Cleaning up transcript with ChatGPT...");
        let transcript_text: String = transcript
            .transcript
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let chatgpt = ChatGPT::new(args.openai_key.clone())?;
        let cleaned_text = chatgpt
            .cleanup_transcript(&transcript_text, format_markdown)
            .await?;

        // For cleanup, output the cleaned text directly as a single item
        // This preserves the cleaned flow better than trying to split it back
        vec![TranscriptItem {
            text: cleaned_text,
            start: transcript
                .transcript
                .first()
                .map(|i| i.start)
                .unwrap_or(0.0),
            duration: transcript.transcript.iter().map(|i| i.duration).sum(),
        }]
    } else {
        transcript.transcript
    };

    // Determine output destination
    let output_dest = if let Some(ref output_path) = args.output {
        OutputDestination::File(output_path.clone())
    } else {
        OutputDestination::Stdout
    };

    match args.format.to_lowercase().as_str() {
        "json" => output_json(&transcript_items, &output_dest)?,
        "srt" => output_srt(&transcript_items, &output_dest)?,
        "text" | "txt" => {
            if args.timestamps {
                output_text(&transcript_items, &output_dest)?;
            } else {
                output_text_only(&transcript_items, &output_dest)?;
            }
        }
        "markdown" | "md" => {
            output_markdown(&transcript_items, &output_dest, args.timestamps)?;
        }
        _ => {
            eprintln!("Unknown format: '{}'. Using 'text' format.", args.format);
            eprintln!("Supported formats: json, text, txt, srt, markdown, md");
            if args.timestamps {
                output_text(&transcript_items, &output_dest)?;
            } else {
                output_text_only(&transcript_items, &output_dest)?;
            }
        }
    }

    Ok(())
}

enum OutputDestination {
    Stdout,
    File(String),
}

impl OutputDestination {
    fn writer(&self) -> Result<Box<dyn Write>, TranscriptError> {
        match self {
            OutputDestination::Stdout => Ok(Box::new(io::stdout())),
            OutputDestination::File(path) => {
                let file = File::create(path).map_err(|e| {
                    TranscriptError::IoError(format!("Failed to create file {}: {}", path, e))
                })?;
                Ok(Box::new(file))
            }
        }
    }
}

fn output_json(items: &[TranscriptItem], dest: &OutputDestination) -> Result<(), TranscriptError> {
    let json = serde_json::to_string_pretty(items)?;
    let mut writer = dest.writer()?;
    writeln!(writer, "{}", json)?;
    Ok(())
}

fn output_srt(items: &[TranscriptItem], dest: &OutputDestination) -> Result<(), TranscriptError> {
    let mut writer = dest.writer()?;

    for (index, item) in items.iter().enumerate() {
        writeln!(writer, "{}", index + 1)?;

        let start_time = format_srt_time(item.start);
        let end_time = format_srt_time(item.start + item.duration);

        writeln!(writer, "{} --> {}", start_time, end_time)?;
        writeln!(writer, "{}", item.text)?;
        writeln!(writer)?;
    }

    Ok(())
}

fn output_text(items: &[TranscriptItem], dest: &OutputDestination) -> Result<(), TranscriptError> {
    let mut writer = dest.writer()?;

    for item in items {
        writeln!(writer, "[{:.2}s] {}", item.start, item.text)?;
    }

    Ok(())
}

fn output_text_only(
    items: &[TranscriptItem],
    dest: &OutputDestination,
) -> Result<(), TranscriptError> {
    let mut writer = dest.writer()?;

    for item in items {
        writeln!(writer, "{}", item.text)?;
    }

    Ok(())
}

fn output_markdown(
    items: &[TranscriptItem],
    dest: &OutputDestination,
    timestamps: bool,
) -> Result<(), TranscriptError> {
    let mut writer = dest.writer()?;

    // If there's only one item and it contains markdown (from ChatGPT cleanup),
    // output it directly without adding extra formatting
    if items.len() == 1
        && (items[0].text.contains("**")
            || items[0].text.contains("##")
            || items[0].text.contains("*"))
    {
        // Already formatted by ChatGPT, just add heading if not present
        if !items[0].text.trim_start().starts_with("#") {
            writeln!(writer, "# Transcript\n")?;
        }
        writeln!(writer, "{}", items[0].text)?;
    } else {
        // Regular markdown output
        writeln!(writer, "# Transcript\n")?;

        for item in items {
            if timestamps {
                writeln!(writer, "**[{:.2}s]** {}", item.start, item.text)?;
            } else {
                writeln!(writer, "{}", item.text)?;
            }
            writeln!(writer)?;
        }
    }

    Ok(())
}

fn format_srt_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = seconds % 60.0;
    let secs_int = secs as u32;
    let millis = ((secs - secs_int as f64) * 1000.0) as u32;

    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs_int, millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_format_srt_time() {
        assert_eq!(format_srt_time(0.0), "00:00:00,000");
        assert_eq!(format_srt_time(65.5), "00:01:05,500");
        assert_eq!(format_srt_time(3661.123), "01:01:01,123");
    }

    #[test]
    fn test_output_destination_stdout() {
        let dest = OutputDestination::Stdout;
        let writer = dest.writer();
        assert!(writer.is_ok());
    }

    #[test]
    fn test_output_destination_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_output.txt");
        let dest = OutputDestination::File(file_path.to_string_lossy().to_string());
        let writer = dest.writer();
        assert!(writer.is_ok());
    }

    #[test]
    fn test_output_json() {
        let items = vec![
            TranscriptItem {
                text: "Hello".to_string(),
                start: 0.0,
                duration: 1.0,
            },
            TranscriptItem {
                text: "World".to_string(),
                start: 1.0,
                duration: 1.0,
            },
        ];

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        let dest = OutputDestination::File(file_path.to_string_lossy().to_string());

        assert!(output_json(&items, &dest).is_ok());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("\"text\": \"Hello\""));
        assert!(content.contains("\"start\": 0.0"));
    }

    #[test]
    fn test_output_srt() {
        let items = vec![
            TranscriptItem {
                text: "Hello".to_string(),
                start: 0.0,
                duration: 2.5,
            },
            TranscriptItem {
                text: "World".to_string(),
                start: 2.5,
                duration: 2.5,
            },
        ];

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.srt");
        let dest = OutputDestination::File(file_path.to_string_lossy().to_string());

        assert!(output_srt(&items, &dest).is_ok());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("1\n"));
        assert!(content.contains("00:00:00,000 --> 00:00:02,500"));
        assert!(content.contains("Hello"));
    }

    #[test]
    fn test_output_text_only() {
        let items = vec![TranscriptItem {
            text: "Hello world".to_string(),
            start: 0.0,
            duration: 1.0,
        }];

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let dest = OutputDestination::File(file_path.to_string_lossy().to_string());

        assert!(output_text_only(&items, &dest).is_ok());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content.trim(), "Hello world");
    }

    #[test]
    fn test_output_text_with_timestamps() {
        let items = vec![TranscriptItem {
            text: "Hello world".to_string(),
            start: 1.5,
            duration: 2.0,
        }];

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let dest = OutputDestination::File(file_path.to_string_lossy().to_string());

        assert!(output_text(&items, &dest).is_ok());
        let content = fs::read_to_string(&file_path).unwrap();
        // Check for timestamp format [X.XX] where X can be any digit
        assert!(content.contains("[1"));
        assert!(content.contains("Hello world"));
    }

    #[test]
    fn test_output_markdown() {
        let items = vec![TranscriptItem {
            text: "Hello world".to_string(),
            start: 0.0,
            duration: 1.0,
        }];

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        let dest = OutputDestination::File(file_path.to_string_lossy().to_string());

        assert!(output_markdown(&items, &dest, false).is_ok());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("# Transcript"));
        assert!(content.contains("Hello world"));
    }

    #[test]
    fn test_output_markdown_with_chatgpt_formatting() {
        let items = vec![TranscriptItem {
            text: "## Section\n\n**Bold text** and *italic*".to_string(),
            start: 0.0,
            duration: 1.0,
        }];

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        let dest = OutputDestination::File(file_path.to_string_lossy().to_string());

        assert!(output_markdown(&items, &dest, false).is_ok());
        let content = fs::read_to_string(&file_path).unwrap();
        // Should detect ChatGPT formatting and not add extra heading
        assert!(content.contains("## Section"));
    }
}
