use quick_xml::events::Event;
use quick_xml::Reader;
use std::str;

pub struct TranscriptParser {
    preserve_formatting: bool,
}

impl TranscriptParser {
    pub fn new(preserve_formatting: bool) -> Self {
        Self {
            preserve_formatting,
        }
    }

    pub fn parse(&self, xml: &str) -> Result<Vec<crate::TranscriptItem>, String> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut items = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"text" => {
                        if let Some(item) = self.parse_text_element(&mut reader, &e)? {
                            items.push(item);
                        }
                    }
                    b"p" => {
                        if let Some(item) = self.parse_p_element(&mut reader, &e)? {
                            items.push(item);
                        }
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("XML parse error: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(items)
    }

    fn parse_text_element(
        &self,
        reader: &mut Reader<&[u8]>,
        e: &quick_xml::events::BytesStart,
    ) -> Result<Option<crate::TranscriptItem>, String> {
        let start = e
            .attributes()
            .find(|a| {
                a.as_ref()
                    .map(|attr| attr.key.as_ref() == b"start")
                    .unwrap_or(false)
            })
            .and_then(|a| {
                a.ok()
                    .and_then(|attr| str::from_utf8(&attr.value).ok().map(|s| s.to_string()))
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .unwrap_or(0.0);

        let duration = e
            .attributes()
            .find(|a| {
                a.as_ref()
                    .map(|attr| attr.key.as_ref() == b"dur")
                    .unwrap_or(false)
            })
            .and_then(|a| {
                a.ok()
                    .and_then(|attr| str::from_utf8(&attr.value).ok().map(|s| s.to_string()))
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .unwrap_or(0.0);

        let mut text = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(e)) => {
                    let decoded = html_escape::decode_html_entities(
                        e.unescape()
                            .map_err(|e| format!("Failed to unescape: {}", e))?
                            .as_ref(),
                    );
                    text.push_str(&decoded);
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"text" => break,
                Ok(Event::Eof) => return Err("Unexpected EOF in text element".to_string()),
                Err(e) => return Err(format!("XML parse error: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        if text.trim().is_empty() {
            return Ok(None);
        }

        Ok(Some(crate::TranscriptItem {
            text: text.trim().to_string(),
            start,
            duration,
        }))
    }

    fn parse_p_element(
        &self,
        reader: &mut Reader<&[u8]>,
        e: &quick_xml::events::BytesStart,
    ) -> Result<Option<crate::TranscriptItem>, String> {
        let start = e
            .attributes()
            .find(|a| {
                a.as_ref()
                    .map(|attr| attr.key.as_ref() == b"t")
                    .unwrap_or(false)
            })
            .and_then(|a| {
                a.ok().and_then(|attr| {
                    str::from_utf8(&attr.value)
                        .ok()
                        .map(|s| s.to_string())
                        .and_then(|s| s.parse::<f64>().ok())
                        .map(|s| s / 1000.0) // Convert from milliseconds
                })
            })
            .unwrap_or(0.0);

        let duration = e
            .attributes()
            .find(|a| {
                a.as_ref()
                    .map(|attr| attr.key.as_ref() == b"d")
                    .unwrap_or(false)
            })
            .and_then(|a| {
                a.ok().and_then(|attr| {
                    str::from_utf8(&attr.value)
                        .ok()
                        .map(|s| s.to_string())
                        .and_then(|s| s.parse::<f64>().ok())
                        .map(|s| s / 1000.0) // Convert from milliseconds
                })
            })
            .unwrap_or(0.0);

        let mut text = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(e)) => {
                    let decoded = html_escape::decode_html_entities(
                        e.unescape()
                            .map_err(|e| format!("Failed to unescape: {}", e))?
                            .as_ref(),
                    );
                    text.push_str(&decoded);
                }
                Ok(Event::Start(e)) => {
                    // Handle nested tags like <s>, <br/>, etc.
                    match e.name().as_ref() {
                        b"s" | b"br" => {
                            if !text.ends_with(' ') {
                                text.push(' ');
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"p" => break,
                Ok(Event::Eof) => return Err("Unexpected EOF in p element".to_string()),
                Err(e) => return Err(format!("XML parse error: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        if text.trim().is_empty() {
            return Ok(None);
        }

        Ok(Some(crate::TranscriptItem {
            text: text.trim().to_string(),
            start,
            duration,
        }))
    }
}

mod html_escape {
    pub fn decode_html_entities(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '&' {
                let mut entity = String::new();
                while let Some(&next) = chars.peek() {
                    if next == ';' {
                        chars.next();
                        break;
                    }
                    entity.push(chars.next().unwrap());
                }

                result.push_str(&decode_entity(&entity));
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn decode_entity(entity: &str) -> String {
        match entity {
            "quot" => "\"".to_string(),
            "amp" => "&".to_string(),
            "apos" => "'".to_string(),
            "lt" => "<".to_string(),
            "gt" => ">".to_string(),
            "nbsp" => " ".to_string(),
            _ => {
                if entity.starts_with("#x") || entity.starts_with("#X") {
                    // Hex entity
                    if let Ok(num) = u32::from_str_radix(&entity[2..], 16) {
                        if let Some(ch) = char::from_u32(num) {
                            return ch.to_string();
                        }
                    }
                    format!("&{};", entity)
                } else if entity.starts_with('#') {
                    // Decimal entity
                    if let Ok(num) = entity[1..].parse::<u32>() {
                        if let Some(ch) = char::from_u32(num) {
                            return ch.to_string();
                        }
                    }
                    format!("&{};", entity)
                } else {
                    format!("&{};", entity)
                }
            }
        }
    }
}
