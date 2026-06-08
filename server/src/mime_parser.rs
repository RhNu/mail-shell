use mail_parser::{Addr, Address, HeaderValue, MessageParser, MimeHeaders};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MailAddress {
    pub name: Option<String>,
    pub address: Option<String>,
}

impl MailAddress {
    pub fn email(&self) -> &str {
        self.address.as_deref().unwrap_or_default()
    }

    pub fn display(&self) -> String {
        match (&self.name, &self.address) {
            (Some(name), Some(addr)) => format!("{name} <{addr}>"),
            (None, Some(addr)) => addr.clone(),
            (Some(name), None) => name.clone(),
            (None, None) => String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParsedMessage {
    pub message_id: Option<String>,
    pub subject: String,
    pub from: Vec<MailAddress>,
    pub to: Vec<MailAddress>,
    pub cc: Vec<MailAddress>,
    pub reply_to: Vec<MailAddress>,
    pub in_reply_to: Option<String>,
    pub date: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<ParsedAttachment>,
}

#[derive(Debug, Clone)]
pub struct ParsedAttachment {
    pub filename: Option<String>,
    pub content_type: String,
    pub body: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("failed to parse email")]
    InvalidInput,
}

#[tracing::instrument(skip(raw))]
pub fn parse_message(raw: &[u8]) -> Result<ParsedMessage, ParseError> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or(ParseError::InvalidInput)?;

    let mut result = ParsedMessage {
        message_id: message.message_id().map(|s| s.trim().to_string()),
        subject: message.subject().unwrap_or_default().to_string(),
        from: extract_addresses(message.from()),
        to: extract_addresses(message.to()),
        cc: extract_addresses(message.cc()),
        reply_to: extract_addresses(message.reply_to()),
        in_reply_to: extract_message_id_header(message.in_reply_to()),
        date: message.date().map(|d| d.to_rfc3339()),
        body_text: message.body_text(0).map(|s| s.into_owned()),
        body_html: message.body_html(0).map(|s| s.into_owned()),
        attachments: Vec::new(),
    };

    for part_id in &message.attachments {
        let part = &message.parts[*part_id as usize];
        result.attachments.push(ParsedAttachment {
            filename: part.attachment_name().map(|s| s.to_string()),
            content_type: part
                .content_type()
                .map(|c| {
                    let sub = c.c_subtype.as_ref().map(|s| s.as_ref()).unwrap_or("");
                    format!("{}/{}", c.c_type, sub)
                })
                .unwrap_or_default(),
            body: match &part.body {
                mail_parser::PartType::Binary(b) | mail_parser::PartType::InlineBinary(b) => {
                    b.to_vec()
                }
                mail_parser::PartType::Text(text) | mail_parser::PartType::Html(text) => {
                    text.as_bytes().to_vec()
                }
                _ => Vec::new(),
            },
        });
    }

    tracing::debug!(
        subject = %result.subject,
        from_count = result.from.len(),
        to_count = result.to.len(),
        cc_count = result.cc.len(),
        attachments_count = result.attachments.len(),
        has_text = result.body_text.is_some(),
        has_html = result.body_html.is_some(),
        "parsed message"
    );

    Ok(result)
}

fn extract_addresses(addr: Option<&Address<'_>>) -> Vec<MailAddress> {
    let Some(addr) = addr else {
        return Vec::new();
    };
    match addr {
        Address::List(list) => list.iter().map(addr_to_mail_address).collect(),
        Address::Group(groups) => groups
            .iter()
            .flat_map(|g| g.addresses.iter())
            .map(addr_to_mail_address)
            .collect(),
    }
}

fn addr_to_mail_address(a: &Addr<'_>) -> MailAddress {
    MailAddress {
        name: a.name().map(|s| s.to_string()),
        address: a.address().map(|s| s.to_string()),
    }
}

fn extract_message_id_header(value: &HeaderValue<'_>) -> Option<String> {
    match value {
        HeaderValue::Text(s) => Some(s.trim().to_string()),
        HeaderValue::TextList(list) => list.first().map(|s| s.trim().to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text_email() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: text/plain\r\n\r\nThis is the body.";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.body_text.as_deref(), Some("This is the body."));
        assert!(parsed.attachments.is_empty());
        assert_eq!(parsed.subject, "Hello");
        assert_eq!(parsed.from.len(), 1);
        assert_eq!(
            parsed.from[0].address.as_deref(),
            Some("sender@example.com")
        );
        assert_eq!(parsed.to.len(), 1);
        assert_eq!(
            parsed.to[0].address.as_deref(),
            Some("recipient@example.com")
        );
    }

    #[test]
    fn test_parse_multipart_alternative() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/alternative; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nPlain text\r\n--boundary123\r\nContent-Type: text/html\r\n\r\n<html><body>HTML</body></html>\r\n--boundary123--";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.body_text.as_deref(), Some("Plain text"));
        assert_eq!(
            parsed.body_html.as_deref(),
            Some("<html><body>HTML</body></html>")
        );
        assert!(parsed.attachments.is_empty());
    }

    #[test]
    fn test_parse_with_attachment() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nSee attached\r\n--boundary123\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"data.bin\"\r\n\r\nBINARYDATA\r\n--boundary123--";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.body_text.as_deref(), Some("See attached"));
        assert_eq!(parsed.attachments.len(), 1);
        let att = &parsed.attachments[0];
        assert_eq!(att.filename, Some("data.bin".to_string()));
        assert!(att.content_type.starts_with("application/octet-stream"));
        assert_eq!(att.body, b"BINARYDATA");
    }

    #[test]
    fn test_parse_text_attachment_preserves_body() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nSee attached\r\n--boundary123\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Disposition: attachment; filename=\"hello.txt\"\r\n\r\nattachment text\r\n--boundary123--";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.attachments.len(), 1);
        assert_eq!(parsed.attachments[0].body, b"attachment text");
    }

    #[test]
    fn test_parse_base64_attachment() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nSee attached\r\n--boundary123\r\nContent-Type: application/octet-stream\r\nContent-Transfer-Encoding: base64\r\nContent-Disposition: attachment; filename=\"data.bin\"\r\n\r\nQklOQVJZ\r\n--boundary123--";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.attachments.len(), 1);
        let att = &parsed.attachments[0];
        assert_eq!(att.body, b"BINARY");
    }

    #[test]
    fn test_rfc2047_subject_decoding_discord_verify() {
        let raw = b"Date: Mon, 08 Jun 2026 06:39:51 +0000\r\nFrom: Discord <noreply@discord.com>\r\nTo: discord+rhnu@rhnu.org\r\nMessage-ID: <verify@example.com>\r\nSubject: =?UTF-8?B?6aqM6K+BIERpc2NvcmQg55qE55S15a2Q6YKu5Lu25Zyw5Z2A?=\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nVerify your address.";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.subject, "验证 Discord 的电子邮件地址");
        assert_eq!(parsed.from.len(), 1);
        assert_eq!(parsed.from[0].name.as_deref(), Some("Discord"));
        assert_eq!(
            parsed.from[0].address.as_deref(),
            Some("noreply@discord.com")
        );
        assert_eq!(parsed.to.len(), 1);
        assert_eq!(
            parsed.to[0].address.as_deref(),
            Some("discord+rhnu@rhnu.org")
        );
        assert!(parsed.date.is_some());
        assert!(parsed.message_id.is_some());
    }

    #[test]
    fn test_rfc2047_subject_decoding_discord_email_change() {
        let raw = b"From: Discord <noreply@discord.com>\r\nTo: discord+rhnu@rhnu.org\r\nSubject: Discord =?UTF-8?B?55S15a2Q6YKu5Lu25Zyw5Z2A5bey5pu05pS5?=\r\nContent-Type: multipart/alternative; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nChanged\r\n--boundary123\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<p>Changed</p>\r\n--boundary123--";
        let parsed = parse_message(raw).unwrap();
        assert_eq!(parsed.subject, "Discord 电子邮件地址已更改");
        assert_eq!(parsed.from.len(), 1);
        assert_eq!(parsed.from[0].name.as_deref(), Some("Discord"));
        assert_eq!(
            parsed.from[0].address.as_deref(),
            Some("noreply@discord.com")
        );
        assert!(parsed.body_text.is_some());
        assert!(parsed.body_html.is_some());
    }

    #[test]
    fn test_address_display_format() {
        let addr = MailAddress {
            name: Some("Discord".to_string()),
            address: Some("noreply@discord.com".to_string()),
        };
        assert_eq!(addr.display(), "Discord <noreply@discord.com>");
        assert_eq!(addr.email(), "noreply@discord.com");

        let addr_no_name = MailAddress {
            name: None,
            address: Some("a@b.com".to_string()),
        };
        assert_eq!(addr_no_name.display(), "a@b.com");
    }

    #[test]
    fn test_parse_lenient_on_invalid_input() {
        let result = parse_message(b"totally not an email");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.subject, "");
        assert!(parsed.from.is_empty());
    }
}
