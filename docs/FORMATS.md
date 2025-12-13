# Output Formats Explained

## Supported Formats

### 1. Text/TXT (`text` or `txt`)
Plain text format without timestamps (default).

**Example:**
```
There's nothing more heartbreaking than
watching a talented writer create
characters that readers just don't care about.
```

**Usage:**
```bash
ytt video_id -f text
ytt video_id -f txt
ytt video_id  # default is text
ytt video_id -o transcript.txt
```

---

### 2. Markdown (`markdown` or `md`)
Markdown formatted transcript with a heading. Great for documentation, blogs, or GitHub READMEs.

**Example:**
```markdown
# Transcript

There's nothing more heartbreaking than

watching a talented writer create

characters that readers just don't care about.
```

**With timestamps (`--timestamps`):**
```markdown
# Transcript

**[0.08s]** There's nothing more heartbreaking than

**[1.84s]** watching a talented writer create

**[3.92s]** characters that readers just don't care about.
```

**With ChatGPT cleanup (`--cleanup`):**
ChatGPT will add proper Markdown formatting including:
- **Bold** for emphasis
- *Italics* for subtle emphasis
- Headings (##, ###) for sections
- Bullet points or numbered lists
- Blockquotes for notable quotes

**Usage:**
```bash
ytt video_id -f markdown -o transcript.md
ytt video_id -f md --timestamps -o transcript.md
ytt video_id --cleanup -f markdown -o cleaned.md
```

---

### 3. JSON (`json`)
Structured JSON format with timestamps, text, start time, and duration. Perfect for programmatic processing.

**Example:**
```json
[
  {
    "text": "There's nothing more heartbreaking than",
    "start": 0.08,
    "duration": 3.839
  },
  {
    "text": "watching a talented writer create",
    "start": 1.839,
    "duration": 4.081
  }
]
```

**Usage:**
```bash
ytt video_id -f json -o transcript.json
ytt video_id -f json | jq .
ytt video_id --cleanup -f json -o cleaned.json
```

---

### 4. SRT (`srt`)
**SRT (SubRip Subtitle)** is a subtitle file format used by video players and editing software. It's the standard format for subtitles/captions.

**What is SRT?**
- SRT stands for "SubRip Subtitle"
- It's a plain text format for subtitles
- Widely supported by video players (VLC, Media Player, YouTube, etc.)
- Used in video editing software (Premiere Pro, Final Cut Pro, etc.)
- Each subtitle entry has:
  - A sequence number
  - Start and end timestamps (HH:MM:SS,mmm format)
  - The subtitle text

**Example:**
```
1
00:00:00,080 --> 00:00:03,919
There's nothing more heartbreaking than

2
00:00:01,839 --> 00:00:05,919
watching a talented writer create

3
00:00:03,919 --> 00:00:08,000
characters that readers just don't care about.
```

**Format Structure:**
```
[Sequence Number]
[Start Time] --> [End Time]
[Subtitle Text]

[Blank Line]
```

**Common Uses:**
- Adding subtitles to videos
- Creating captions for accessibility
- Video editing workflows
- Sharing transcripts with timing information
- Importing into video editing software

**Usage:**
```bash
ytt video_id -f srt -o subtitles.srt
ytt video_id -f srt -o video_captions.srt
```

**Importing SRT files:**
- **YouTube**: Upload as caption file
- **VLC Media Player**: Automatically loads `.srt` files with same name as video
- **Premiere Pro**: File → Import → Select `.srt` file
- **Final Cut Pro**: Import subtitle file
- **HandBrake**: Can embed SRT subtitles into video

---

## Format Comparison

| Format | Timestamps | Structure | Best For |
|--------|-----------|-----------|----------|
| **Text/TXT** | No (optional) | Plain text | Reading, simple notes |
| **Markdown** | Optional | Markdown | Documentation, blogs, GitHub |
| **JSON** | Yes | Structured data | Programming, APIs, processing |
| **SRT** | Yes | Subtitle format | Video editing, subtitles, captions |

---

## Output to Files

All formats can be saved to files using the `-o` flag:

```bash
# Text file
ytt video_id -o transcript.txt

# Markdown file
ytt video_id -f markdown -o transcript.md

# JSON file
ytt video_id -f json -o transcript.json

# SRT subtitle file
ytt video_id -f srt -o subtitles.srt
```

If `-o` is not specified, output goes to stdout (terminal).

## Combining Formats with Other Options

```bash
# Cleanup + Markdown
ytt video_id --cleanup -f markdown -o cleaned.md

# Translation + JSON
ytt video_id --languages es --translate en -f json -o translated.json

# Timestamps + SRT
ytt video_id --timestamps -f srt -o subtitles.srt
```
