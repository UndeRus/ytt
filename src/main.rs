use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
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

    /// Use video title as the basename for the output file
    #[arg(short = 'n', long)]
    name: bool,

    /// Include video URL at the start of markdown output (only works with -f md/markdown)
    #[arg(short = 'u', long)]
    url: bool,

    /// The provided URL is a playlist URL - fetch transcripts for all videos in the playlist
    #[arg(short = 'p', long)]
    playlist: bool,

    /// Maximum number of videos to process in playlist mode (ignored in normal mode)
    #[arg(short = 'm', long)]
    max: Option<usize>,
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
    let api = YouTubeTranscript::with_delay(args.delay);

    // Handle playlist mode
    if args.playlist {
        let playlist_id = YouTubeTranscript::extract_playlist_id(&args.video)?;
        eprintln!("Fetching video IDs from playlist: {}", playlist_id);
        let video_ids = api.get_playlist_video_ids(&playlist_id).await?;
        eprintln!("Found {} videos in playlist", video_ids.len());

        // Limit to max number if specified
        let videos_to_process: Vec<&String> = if let Some(max) = args.max {
            let limit = max.min(video_ids.len());
            if limit < video_ids.len() {
                eprintln!("Processing first {} videos (limited by --max)", limit);
            }
            video_ids.iter().take(limit).collect()
        } else {
            video_ids.iter().collect()
        };

        let total = videos_to_process.len();
        for (index, video_id) in videos_to_process.iter().enumerate() {
            eprintln!("\n[{}/{}] Processing video: {}", index + 1, total, video_id);
            if let Err(e) = process_single_video(&api, &args, video_id, Some(index + 1), Some(total)).await {
                eprintln!("Error processing video {}: {}", video_id, e);
                // Continue with next video instead of failing completely
                continue;
            }
        }
        return Ok(());
    }

    // Single video mode
    let video_id = YouTubeTranscript::extract_video_id(&args.video)?;
    process_single_video(&api, &args, &video_id, None, None).await
}

async fn process_single_video(
    api: &YouTubeTranscript,
    args: &Args,
    video_id: &str,
    video_index: Option<usize>,
    total_videos: Option<usize>,
) -> Result<(), TranscriptError> {
    if args.list {
        let transcript_list = api.list_transcripts(video_id).await?;
        if let (Some(idx), Some(total)) = (video_index, total_videos) {
            println!("[{}/{}] Available transcripts for video: {}", idx, total, video_id);
        } else {
            println!("Available transcripts for video: {}", video_id);
        }
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

    if video_index.is_none() {
        println!("Fetching transcript for video: {}", video_id);
    }

    let transcript = if let Some(target_lang) = &args.translate {
        let source_langs: Vec<&str> = args
            .languages
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_else(|| vec!["en"]);
        api.translate_transcript(video_id, &source_langs, target_lang)
            .await?
    } else {
        let lang_codes: Option<Vec<&str>> = args
            .languages
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());
        api.fetch_transcript(video_id, lang_codes).await?
    };

    // Determine if we need markdown formatting from ChatGPT
    let format_markdown = args.cleanup
        && (args.format.to_lowercase() == "markdown" || args.format.to_lowercase() == "md");

    // If cleanup is requested, send to ChatGPT first
    let transcript_items = if args.cleanup {
        if video_index.is_none() {
            eprintln!("Cleaning up transcript with ChatGPT...");
        }
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
    // For playlists, if -o is a directory or -n is used, each video gets its own file
    let output_dest = if let Some(ref output_path) = args.output {
        let path = Path::new(output_path);
        
        // Check if the path is a directory:
        // 1. If it exists and is a directory
        // 2. If it ends with a path separator (directory-like)
        let is_directory = if path.exists() {
            path.is_dir()
        } else {
            // If path doesn't exist, check if it ends with a separator (directory-like)
            let sep = std::path::MAIN_SEPARATOR;
            output_path.ends_with(sep) || output_path.ends_with('/')
        };
        
        if is_directory && args.name {
            // Combine directory with title as filename
            let title = transcript.title.as_ref()
                .ok_or_else(|| TranscriptError::YouTubeDataUnparsable(
                    "Failed to extract video title".to_string()
                ))?;
            let sanitized_title = sanitize_filename(title);
            let extension = match args.format.to_lowercase().as_str() {
                "json" => "json",
                "srt" => "srt",
                "markdown" | "md" => "md",
                "text" | "txt" => "txt",
                _ => "txt",
            };
            let filename = format!("{}.{}", sanitized_title, extension);
            let combined_path = path.join(filename);
            OutputDestination::File(combined_path.to_string_lossy().to_string())
        } else if is_directory && video_index.is_some() {
            // For playlist mode with directory output, use video_id as filename
            let extension = match args.format.to_lowercase().as_str() {
                "json" => "json",
                "srt" => "srt",
                "markdown" | "md" => "md",
                "text" | "txt" => "txt",
                _ => "txt",
            };
            let filename = format!("{}.{}", video_id, extension);
            let combined_path = path.join(filename);
            OutputDestination::File(combined_path.to_string_lossy().to_string())
        } else {
            // Use the path as-is (either it's a file path, or -n wasn't specified)
            // For playlist mode, this would overwrite, so we should handle it differently
            if video_index.is_some() {
                // In playlist mode with a file path, append video_id
                let path_buf = Path::new(output_path);
                let stem = path_buf.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                let extension = path_buf.extension().and_then(|s| s.to_str()).unwrap_or("txt");
                let parent = path_buf.parent().unwrap_or(Path::new("."));
                let new_filename = format!("{}_{}.{}", stem, video_id, extension);
                let combined_path = parent.join(new_filename);
                OutputDestination::File(combined_path.to_string_lossy().to_string())
            } else {
                OutputDestination::File(output_path.clone())
            }
        }
    } else if args.name {
        // Use video title as basename in current directory
        let title = transcript.title.as_ref()
            .ok_or_else(|| TranscriptError::YouTubeDataUnparsable(
                "Failed to extract video title".to_string()
            ))?;
        let sanitized_title = sanitize_filename(title);
        let extension = match args.format.to_lowercase().as_str() {
            "json" => "json",
            "srt" => "srt",
            "markdown" | "md" => "md",
            "text" | "txt" => "txt",
            _ => "txt",
        };
        let output_path = format!("{}.{}", sanitized_title, extension);
        OutputDestination::File(output_path)
    } else if video_index.is_some() {
        // Playlist mode without -o or -n: use video_id as filename
        let extension = match args.format.to_lowercase().as_str() {
            "json" => "json",
            "srt" => "srt",
            "markdown" | "md" => "md",
            "text" | "txt" => "txt",
            _ => "txt",
        };
        let output_path = format!("{}.{}", video_id, extension);
        OutputDestination::File(output_path)
    } else {
        OutputDestination::Stdout
    };

    let video_url = if args.url {
        Some(format!("https://www.youtube.com/watch?v={}", video_id))
    } else {
        None
    };
    let video_title = if args.url {
        transcript.title.as_ref().map(|s| s.as_str())
    } else {
        None
    };

    match args.format.to_lowercase().as_str() {
        "json" => output_json(&transcript_items, &output_dest)?,
        "srt" => output_srt(&transcript_items, &output_dest)?,
        "text" | "txt" => {
            if args.timestamps {
                output_text(&transcript_items, &output_dest, video_url.as_deref(), video_title)?;
            } else {
                output_text_only(&transcript_items, &output_dest, video_url.as_deref(), video_title)?;
            }
        }
        "markdown" | "md" => {
            let video_title = if args.url {
                transcript.title.as_ref().map(|s| s.as_str())
            } else {
                None
            };
            output_markdown(&transcript_items, &output_dest, args.timestamps, video_url.as_deref(), video_title)?;
        }
        _ => {
            eprintln!("Unknown format: '{}'. Using 'text' format.", args.format);
            eprintln!("Supported formats: json, text, txt, srt, markdown, md");
            if args.timestamps {
                output_text(&transcript_items, &output_dest, video_url.as_deref(), video_title)?;
            } else {
                output_text_only(&transcript_items, &output_dest, video_url.as_deref(), video_title)?;
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

fn output_text(items: &[TranscriptItem], dest: &OutputDestination, video_url: Option<&str>, video_title: Option<&str>) -> Result<(), TranscriptError> {
    let mut writer = dest.writer()?;

    // Prefix with title and URL if provided
    if let (Some(url), Some(title)) = (video_url, video_title) {
        writeln!(writer, "{}: {}", title, url)?;
        writeln!(writer)?;
    }

    for item in items {
        writeln!(writer, "[{:.2}s] {}", item.start, item.text)?;
    }

    Ok(())
}

fn output_text_only(
    items: &[TranscriptItem],
    dest: &OutputDestination,
    video_url: Option<&str>,
    video_title: Option<&str>,
) -> Result<(), TranscriptError> {
    let mut writer = dest.writer()?;

    // Prefix with title and URL if provided
    if let (Some(url), Some(title)) = (video_url, video_title) {
        writeln!(writer, "{}: {}", title, url)?;
        writeln!(writer)?;
    }

    for item in items {
        writeln!(writer, "{}", item.text)?;
    }

    Ok(())
}

fn output_markdown(
    items: &[TranscriptItem],
    dest: &OutputDestination,
    timestamps: bool,
    video_url: Option<&str>,
    video_title: Option<&str>,
) -> Result<(), TranscriptError> {
    let mut writer = dest.writer()?;

    // If URL and title are provided, prepend the markdown link
    if let (Some(url), Some(title)) = (video_url, video_title) {
        writeln!(writer, "![{}]({})\n", title, url)?;
    }

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

fn sanitize_filename(title: &str) -> String {
    // Replace invalid filesystem characters with underscores
    let sanitized: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Trim whitespace and replace multiple spaces/underscores with single underscore
    let sanitized = sanitized.trim();
    let sanitized = sanitized
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_");
    let sanitized = sanitized
        .chars()
        .fold(String::new(), |mut acc, c| {
            if c == '_' {
                if !acc.ends_with('_') {
                    acc.push(c);
                }
            } else {
                acc.push(c);
            }
            acc
        });

    // Limit length to 200 characters (reasonable for most filesystems)
    let sanitized = if sanitized.len() > 200 {
        &sanitized[..200]
    } else {
        &sanitized
    };

    sanitized.trim_end_matches('_').to_string()
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

        assert!(output_text_only(&items, &dest, None, None).is_ok());
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

        assert!(output_text(&items, &dest, None, None).is_ok());
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

        assert!(output_markdown(&items, &dest, false, None, None).is_ok());
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

        assert!(output_markdown(&items, &dest, false, None, None).is_ok());
        let content = fs::read_to_string(&file_path).unwrap();
        // Should detect ChatGPT formatting and not add extra heading
        assert!(content.contains("## Section"));
    }
}
