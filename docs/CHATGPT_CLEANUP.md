# ChatGPT Transcript Cleanup

The `ytt` tool can automatically clean up and improve transcripts using ChatGPT.

## Setup

### Option 1: Environment Variable (Recommended)

Set your OpenAI API key as an environment variable:

```bash
export OPENAI_API_KEY="your-api-key-here"
ytt mcbwS5Owclo --languages en --cleanup
```

### Option 2: Command Line Flag

Pass the API key directly:

```bash
ytt mcbwS5Owclo --languages en --cleanup --openai-key "your-api-key-here"
```

## Usage

### Basic Cleanup

```bash
ytt mcbwS5Owclo --languages en --cleanup
```

### With Different Formats

```bash
# Cleaned text output
ytt mcbwS5Owclo --languages en --cleanup

# Cleaned JSON output
ytt mcbwS5Owclo --languages en --cleanup --format json

# Cleaned Markdown output (with formatting)
ytt mcbwS5Owclo --languages en --cleanup --format markdown

# Cleaned text with timestamps
ytt mcbwS5Owclo --languages en --cleanup --timestamps
```

### Save Cleaned Transcripts

```bash
# Save cleaned transcript to file
ytt mcbwS5Owclo --languages en --cleanup -o cleaned.txt

# Save cleaned Markdown
ytt mcbwS5Owclo --languages en --cleanup -f markdown -o cleaned.md
```

## What ChatGPT Does

The cleanup process:
- ✅ Fixes grammar errors
- ✅ Improves sentence structure
- ✅ Removes filler words and repetitions
- ✅ Makes text more readable
- ✅ Preserves original meaning and content
- ✅ Removes promotional content (products, websites, courses, training programs, email addresses, social media handles)
- ✅ Formats as Markdown when using `-f markdown` (adds headings, bold, lists, etc.)
- ❌ Does NOT add information not in the original

## Prompt Details

When cleanup is requested, ChatGPT receives:

**System Message:**
```
You are a helpful assistant that cleans up and improves transcripts while preserving their original meaning. 
You remove promotional content like product mentions, website URLs, course offers, and training programs.
```

**User Prompt:**
```
Please clean up and improve the following transcript. 
Fix any grammar errors, improve sentence structure, remove filler words and repetitions, 
and make it more readable while preserving the original meaning and content. 
Do not add any information that wasn't in the original transcript.

IMPORTANT: Remove all references to products, websites, courses, training programs, 
email addresses, social media handles, or any promotional content that the presenter may offer. 
Focus only on the educational or informational content.

[If markdown format:]
Format the cleaned transcript using Markdown syntax. Use appropriate markdown elements like:
- **Bold** for emphasis on important points
- *Italics* for subtle emphasis
- Headings (##, ###) to organize sections if the transcript has clear topics
- Bullet points (-) or numbered lists (1.) for lists
- Blockquotes (>) for notable quotes
- Line breaks between paragraphs
Make it well-structured and readable with proper markdown formatting.

Transcript:

[transcript text]
```

## Example

**Before cleanup:**
```
There's nothing more heartbreaking than watching a talented writer create characters that readers just don't care about. Not because the story is bad, not because the writing is weak, but because the characters feel hollow, disconnected, like cardboard cutouts moving through the plot.
```

**After cleanup:**
```
There's nothing more heartbreaking than watching a talented writer create characters that readers simply don't care about. It's not because the story is bad or the writing is weak, but because the characters feel hollow and disconnected—like cardboard cutouts moving through the plot.
```

**After cleanup with Markdown:**
```markdown
## Character Development Issues

There's nothing more heartbreaking than watching a talented writer create **characters that readers simply don't care about**. It's not because:

- The story is bad
- The writing is weak

But because the characters feel **hollow and disconnected**—like cardboard cutouts moving through the plot.
```

## Notes

- Requires an active OpenAI API key
- Uses `gpt-4o-mini` model (cost-effective)
- The cleaned transcript is returned as a single continuous text block
- Original timestamps are preserved in the first item (for JSON/SRT formats)
- Processing time depends on transcript length and API response time
- Promotional content is automatically removed

## Error Handling

If the API key is missing:
```
Error: OpenAI API key not found. Set OPENAI_API_KEY environment variable or use --openai-key flag
```

If there's an API error, you'll see:
```
Error: OpenAI API error (401): Invalid API key
```

## Cost Considerations

- Uses GPT-4o-mini (lower cost than GPT-4)
- Cost depends on transcript length
- Typical transcript: ~$0.01-0.05 per cleanup
- Check OpenAI pricing for current rates: https://openai.com/pricing

## Best Practices

1. **Use Markdown format** for better structured output:
   ```bash
   ytt video_id --cleanup -f markdown -o cleaned.md
   ```

2. **Save to file** to preserve cleaned transcripts:
   ```bash
   ytt video_id --cleanup -o cleaned.txt
   ```

3. **Combine with other options**:
   ```bash
   ytt video_id --languages en --cleanup -f markdown -o cleaned.md
   ```

4. **Review cleaned content** - While ChatGPT does a good job, always review for accuracy
