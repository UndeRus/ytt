use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranscriptError {
    #[error("Video unavailable: {0}")]
    VideoUnavailable(String),

    #[error("Transcripts disabled for video: {0}")]
    TranscriptsDisabled(String),

    #[error("No transcript found for video {0} in languages: {1:?}")]
    NoTranscriptFound(String, Vec<String>),

    #[error("Age restricted video: {0}")]
    AgeRestricted(String),

    #[error("IP blocked for video: {0}")]
    IpBlocked(String),

    #[error("Request blocked (bot detected) for video: {0}")]
    RequestBlocked(String),

    #[error("Video unplayable: {0} - {1}")]
    VideoUnplayable(String, String),

    #[error("Failed to create consent cookie for video: {0}")]
    FailedToCreateConsentCookie(String),

    #[error("YouTube data unparsable for video: {0}")]
    YouTubeDataUnparsable(String),

    #[error("Protected video requires token: {0}")]
    PoTokenRequired(String),

    #[error("Invalid video ID: {0}")]
    InvalidVideoId(String),

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Failed to parse XML: {0}")]
    XmlParseError(String),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(String),

    #[error("Translation not available: {0}")]
    NotTranslatable(String),

    #[error("Translation language not available: {0}")]
    TranslationLanguageNotAvailable(String),

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for TranscriptError {
    fn from(err: std::io::Error) -> Self {
        TranscriptError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for TranscriptError {
    fn from(err: serde_json::Error) -> Self {
        TranscriptError::JsonParseError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TranscriptError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TranscriptError::VideoUnavailable("test123".to_string());
        assert!(err.to_string().contains("test123"));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let transcript_err: TranscriptError = io_err.into();
        assert!(matches!(transcript_err, TranscriptError::IoError(_)));
    }

    #[test]
    fn test_error_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_err.is_err());
        if let Err(e) = json_err {
            let transcript_err: TranscriptError = e.into();
            assert!(matches!(transcript_err, TranscriptError::JsonParseError(_)));
        }
    }
}
