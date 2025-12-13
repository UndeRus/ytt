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

# Cleaned text with timestamps
ytt mcbwS5Owclo --languages en --cleanup --timestamps
```

## What ChatGPT Does

The cleanup process:
- ✅ Fixes grammar errors
- ✅ Improves sentence structure
- ✅ Removes filler words and repetitions
- ✅ Makes text more readable
- ✅ Preserves original meaning and content
- ❌ Does NOT add information not in the original

## Example

**Before cleanup:**
```
There's nothing more heartbreaking than watching a talented writer create characters that readers just don't care about. Not because the story is bad, not because the writing is weak, but because the characters feel hollow, disconnected, like cardboard cutouts moving through the plot.
```

**After cleanup:**
```
There's nothing more heartbreaking than watching a talented writer create characters that readers simply don't care about. It's not because the story is bad or the writing is weak, but because the characters feel hollow and disconnected—like cardboard cutouts moving through the plot.
```

## Notes

- Requires an active OpenAI API key
- Uses `gpt-4o-mini` model (cost-effective)
- The cleaned transcript is returned as a single continuous text block
- Original timestamps are preserved in the first item (for JSON/SRT formats)
- Processing time depends on transcript length and API response time

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
- Check OpenAI pricing for current rates
