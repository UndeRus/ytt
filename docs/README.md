# ytt - YouTube Transcript API (Rust)

A Rust implementation of the YouTube Transcript API, similar to the Python [youtube-transcript-api](https://pypi.org/project/youtube-transcript-api/) package.

## Features

- ✅ Fetch transcripts/captions from YouTube videos using InnerTube API (same as Python version)
- ✅ Support for multiple languages with priority fallback
- ✅ Handle both manually created and auto-generated transcripts (prioritizes manual)
- ✅ Multiple output formats: JSON, text, SRT
- ✅ Extract video ID from various YouTube URL formats
- ✅ Translation support for translatable transcripts
- ✅ Proper XML parsing with quick-xml
- ✅ Comprehensive error handling with specific error types
- ✅ Consent cookie handling for GDPR compliance
- ✅ Playability status checking
- ✅ List available transcripts for a video

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/ytt`.

## Usage

### Basic Usage

```bash
# Fetch transcript using video URL
ytt "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# Fetch transcript using video ID
ytt dQw4w9WgXcQ

# Specify language
ytt "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --languages en

# List available transcripts
ytt dQw4w9WgXcQ --list

# Translate transcript to another language
ytt dQw4w9WgXcQ --languages es --translate en

# Output as JSON
ytt dQw4w9WgXcQ --format json

# Output as SRT (subtitle format)
ytt dQw4w9WgXcQ --format srt

# Output with timestamps (default is without timestamps)
ytt dQw4w9WgXcQ --timestamps
```

### Command Line Options

- `video`: YouTube video URL or video ID (required)
- `-l, --languages <LANGUAGES>`: Language codes (e.g., en, es, fr). Can specify multiple. Prioritizes manually created transcripts.
- `-t, --translate <LANGUAGE>`: Translate transcript to this language code (requires source language)
- `--list`: List all available transcripts instead of fetching
- `-f, --format <FORMAT>`: Output format: `json`, `text`, `txt`, `srt`, `markdown`, or `md` (default: `text`)
- `-o, --output <OUTPUT>`: Output file path (if not specified, outputs to stdout)
- `--timestamps`: Show timestamps with transcript text (default: no timestamps)

### Examples

```bash
# Get English transcript
ytt "https://www.youtube.com/watch?v=dQw4w9WgXcQ" -l en

# Get Spanish transcript
ytt dQw4w9WgXcQ -l es

# Save as SRT file
ytt dQw4w9WgXcQ --format srt > transcript.srt

# Get JSON output
ytt dQw4w9WgXcQ --format json | jq .
```

## Library Usage

You can also use `ytt` as a library in your Rust projects:

```rust
use ytt::YouTubeTranscript;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = YouTubeTranscript::new();
    
    // Extract video ID from URL
    let video_id = YouTubeTranscript::extract_video_id(
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
    )?;
    
    // Fetch transcript
    let transcript = api.fetch_transcript(&video_id, Some(vec!["en"])).await?;
    
    // Access transcript data
    for item in transcript.transcript {
        println!("[{}s] {}", item.start, item.text);
    }
    
    Ok(())
}
```

## Output Formats

### Text Format (default - no timestamps)
```
Hello world
This is a transcript
Without timestamps
```

### Text Format (with --timestamps flag)
```
[0.00] Hello world
[2.50] This is a transcript
[5.00] With timestamps
```

### JSON Format
```json
[
  {
    "text": "Hello world",
    "start": 0.0,
    "duration": 2.5
  },
  {
    "text": "This is a transcript",
    "start": 2.5,
    "duration": 2.5
  }
]
```

### SRT Format
```
1
00:00:00,000 --> 00:00:02,500
Hello world

2
00:00:02,500 --> 00:00:05,000
This is a transcript
```

### Markdown Format
```markdown
# Transcript

Hello world

This is a transcript

With timestamps (if --timestamps flag is used):
**[0.00]** Hello world
**[2.50]** This is a transcript
```

## Supported URL Formats

- `https://www.youtube.com/watch?v=VIDEO_ID`
- `https://youtu.be/VIDEO_ID`
- `VIDEO_ID` (direct 11-character video ID)

## Improvements Over Initial Version

This implementation now closely matches the Python `youtube-transcript-api`:

- ✅ Uses InnerTube API (same approach as Python)
- ✅ Proper XML parsing with `quick-xml`
- ✅ Comprehensive error handling
- ✅ Consent cookie handling
- ✅ Translation support
- ✅ Manual vs generated transcript prioritization

## Limitations

- Requires the video to have transcripts/captions available
- Some videos may not have transcripts in all languages
- Auto-generated transcripts may have lower accuracy than manual ones
- Protected videos requiring tokens may not work

## License

MIT
