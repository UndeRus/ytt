# Rate Limiting and Delays

## How Delays Help

Yes, **deliberately delaying requests can help resolve rate limiting issues**. YouTube's API has rate limits to prevent abuse, and making requests too quickly can trigger:

- IP blocks (429 errors)
- Bot detection (reCAPTCHA challenges)
- Temporary blocks

## Implementation

The application now includes configurable delays between requests:

1. **Before initial HTML fetch** - Prevents immediate rate limiting
2. **Between HTML fetch and InnerTube API call** - Spreads out API requests
3. **Before transcript fetch** - Reduces load on transcript endpoints
4. **Before retry after consent** - Prevents rapid retries

## Usage

### Default Delay (500ms)

```bash
ytt mcbwS5Owclo --languages en
```

### Custom Delay

```bash
# 1 second delay between requests
ytt mcbwS5Owclo --languages en --delay 1000

# 2 second delay (more conservative)
ytt mcbwS5Owclo --languages en --translate es --delay 2000

# 3 second delay (very conservative, for batch processing)
ytt mcbwS5Owclo --languages en --delay 3000
```

## Recommended Delays

| Use Case | Recommended Delay |
|----------|------------------|
| Single request | 500ms (default) |
| Multiple requests | 1000-2000ms |
| Batch processing | 2000-5000ms |
| After rate limit hit | 5000-10000ms |

## Best Practices

1. **Start with default delay** (500ms) - Works for most cases
2. **Increase delay if you hit rate limits** - Try 1000-2000ms
3. **Wait longer after IP block** - Wait 5-10 minutes before retrying
4. **Use delays for batch operations** - Process videos with delays between them
5. **Monitor for rate limit errors** - Adjust delay based on error frequency

## How It Works

The delay is applied at strategic points:

```
1. [DELAY] → Fetch HTML page
2. [DELAY] → Extract API key
3. [DELAY] → Call InnerTube API
4. [DELAY] → Fetch transcript XML
```

Each delay gives YouTube's servers time to process the previous request and reduces the chance of triggering rate limits.

## Testing Results

- ✅ **List transcripts** - Works reliably with 500ms+ delay
- ✅ **Fetch transcript** - Works reliably with 500ms+ delay  
- ⚠️ **Translation** - May need 1000-2000ms delay, especially after multiple requests
- ⚠️ **After IP block** - Wait 5-10 minutes before retrying

## Example: Batch Processing

```bash
#!/bin/bash
videos=("video1" "video2" "video3")

for video in "${videos[@]}"; do
    ytt "$video" --languages en --delay 2000
    sleep 2  # Additional delay between videos
done
```

## Library Usage

```rust
use ytt::YouTubeTranscript;

// Default 500ms delay
let api = YouTubeTranscript::new();

// Custom delay (1000ms = 1 second)
let api = YouTubeTranscript::with_delay(1000);

let transcript = api.fetch_transcript("video_id", Some(vec!["en"])).await?;
```

## Notes

- Delays are cumulative - if you set 1000ms delay, each request waits 1 second
- Delays don't guarantee no rate limiting, but significantly reduce the chance
- If you're still getting blocked, increase the delay or wait longer between sessions
- YouTube's rate limits can vary based on time of day, IP reputation, etc.
