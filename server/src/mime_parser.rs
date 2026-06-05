use mailparse::{MailHeaderMap, ParsedMail, parse_mail};

/// Result of parsing a raw email into its constituent parts.
#[derive(Debug, Clone, Default)]
pub struct ParsedMessage {
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<ParsedAttachment>,
}

/// A single attachment extracted from a MIME message.
#[derive(Debug, Clone)]
pub struct ParsedAttachment {
    pub filename: Option<String>,
    pub content_type: String,
    pub body: Vec<u8>,
}

/// Parse a raw MIME email and extract text bodies and attachments.
#[tracing::instrument(skip(raw))]
pub fn parse_message(raw: &[u8]) -> Result<ParsedMessage, mailparse::MailParseError> {
    let parsed = parse_mail(raw)?;
    let mut result = ParsedMessage::default();
    walk_parts(&parsed, &mut result)?;
    tracing::debug!(
        attachments_count = result.attachments.len(),
        has_text = result.body_text.is_some(),
        has_html = result.body_html.is_some(),
        "parsed message"
    );
    Ok(result)
}

/// Recursively walk MIME parts, populating `result` with bodies and attachments.
///
/// Multipart containers are descended into; leaf parts are classified as
/// attachments or body content based on `Content-Disposition` and `Content-Type`.
#[tracing::instrument(skip(part, result))]
fn walk_parts(
    part: &ParsedMail<'_>,
    result: &mut ParsedMessage,
) -> Result<(), mailparse::MailParseError> {
    let ctype = part.ctype.mimetype.to_ascii_lowercase();

    if ctype.starts_with("multipart/") {
        for sub in &part.subparts {
            walk_parts(sub, result)?;
        }
        return Ok(());
    }

    let disp = part
        .get_headers()
        .get_first_value("Content-Disposition")
        .unwrap_or_default()
        .to_ascii_lowercase();

    // Heuristic: treat a part as an attachment if it has an explicit
    // attachment disposition, a Content-Id (typical for inline attachments),
    // or a filename parameter without being marked inline.
    let is_attachment = disp.contains("attachment")
        || part.get_headers().get_first_value("Content-Id").is_some()
        || (disp.contains("filename") && !disp.contains("inline"));

    if is_attachment {
        let filename = extract_filename(part);
        tracing::debug!(filename = ?filename, content_type = %ctype, "extracted attachment");
        result.attachments.push(ParsedAttachment {
            filename,
            content_type: ctype,
            body: part.get_body_raw()?,
        });
    } else if ctype == "text/plain" {
        result.body_text = Some(part.get_body()?);
    } else if ctype == "text/html" {
        result.body_html = Some(part.get_body()?);
    }

    Ok(())
}

/// Extract the filename from `Content-Disposition` (`filename`) or
/// `Content-Type` (`name`) headers.
fn extract_filename(part: &ParsedMail<'_>) -> Option<String> {
    if let Some(val) = part.get_headers().get_first_value("Content-Disposition")
        && let Some(name) = extract_param(&val, "filename")
    {
        return Some(name);
    }
    if let Some(val) = part.get_headers().get_first_value("Content-Type")
        && let Some(name) = extract_param(&val, "name")
    {
        return Some(name);
    }
    None
}

/// Extract a parameter value from a MIME header such as
/// `Content-Disposition: attachment; filename="data.bin"`.
fn extract_param(header: &str, param: &str) -> Option<String> {
    let lower = header.to_ascii_lowercase();
    let search = format!("{param}=\"");
    if let Some(start) = lower.find(&search) {
        let rest = &header[start + search.len()..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    let search2 = format!("{param}=");
    if let Some(start) = lower.find(&search2) {
        let rest = &header[start + search2.len()..];
        let end = rest.find(|c: char| c == ';' || c.is_whitespace()).unwrap_or(rest.len());
        return Some(rest[..end].trim_matches('"').to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text_email() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: text/plain\r\n\r\nThis is the body.";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.body_text, Some("This is the body.".to_string()));
        assert_eq!(parsed.body_html, None);
        assert!(parsed.attachments.is_empty());
    }

    #[test]
    fn test_parse_multipart_alternative() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/alternative; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nPlain text\r\n--boundary123\r\nContent-Type: text/html\r\n\r\n<html><body>HTML</body></html>\r\n--boundary123--";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.body_text, Some("Plain text".to_string()));
        assert_eq!(parsed.body_html, Some("<html><body>HTML</body></html>".to_string()));
        assert!(parsed.attachments.is_empty());
    }

    #[test]
    fn test_parse_with_attachment() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nSee attached\r\n--boundary123\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"data.bin\"\r\n\r\nBINARYDATA\r\n--boundary123--";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.body_text, Some("See attached".to_string()));
        assert_eq!(parsed.attachments.len(), 1);
        let att = &parsed.attachments[0];
        assert_eq!(att.filename, Some("data.bin".to_string()));
        assert_eq!(att.content_type, "application/octet-stream");
        assert_eq!(att.body, b"BINARYDATA");
    }

    #[test]
    fn test_parse_base64_attachment() {
        // "BINARY" in base64 is "QklOQVJZ"
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nSee attached\r\n--boundary123\r\nContent-Type: application/octet-stream\r\nContent-Transfer-Encoding: base64\r\nContent-Disposition: attachment; filename=\"data.bin\"\r\n\r\nQklOQVJZ\r\n--boundary123--";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.attachments.len(), 1);
        let att = &parsed.attachments[0];
        assert_eq!(att.filename, Some("data.bin".to_string()));
        assert_eq!(att.body, b"BINARY");
    }
}
