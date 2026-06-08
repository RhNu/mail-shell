use std::collections::HashSet;

use mail_parser::{
    Addr, Address, Header, HeaderValue, Message, MessageParser, MimeHeaders, PartType,
};
use serde::{Deserialize, Serialize};

pub const SNAPSHOT_VERSION: i64 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotHeader {
    pub name: String,
    pub value: String,
    pub raw_value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedMailSnapshotV1 {
    pub version: i64,
    pub headers: Vec<SnapshotHeader>,
    pub message_id: Option<String>,
    pub subject: String,
    pub from: Vec<MailAddress>,
    pub to: Vec<MailAddress>,
    pub cc: Vec<MailAddress>,
    pub bcc: Vec<MailAddress>,
    pub reply_to: Vec<MailAddress>,
    pub sender: Vec<MailAddress>,
    pub in_reply_to: Vec<String>,
    pub references: Vec<String>,
    pub date: Option<String>,
    pub text_body: Vec<u32>,
    pub html_body: Vec<u32>,
    pub primary_body_text: Option<String>,
    pub primary_body_html: Option<String>,
    pub parts: Vec<SnapshotPart>,
}

impl ParsedMailSnapshotV1 {
    pub fn body_text(&self) -> Option<&str> {
        self.primary_body_text.as_deref()
    }

    pub fn body_html(&self) -> Option<&str> {
        self.primary_body_html.as_deref()
    }

    pub fn bind_attachment_id(
        &mut self,
        part_id: u32,
        attachment_id: String,
    ) -> Result<(), ParseError> {
        let Some(part) = self.parts.get_mut(part_id as usize) else {
            return Err(ParseError::InvalidAttachmentPart(part_id));
        };
        let SnapshotPartBody::Attachment {
            attachment_id: stored_id,
            ..
        } = &mut part.body
        else {
            return Err(ParseError::InvalidAttachmentPart(part_id));
        };
        *stored_id = Some(attachment_id);
        Ok(())
    }

    pub fn validate_for_storage(&self) -> Result<(), ParseError> {
        for part in &self.parts {
            if matches!(
                part.body,
                SnapshotPartBody::Attachment {
                    attachment_id: None,
                    ..
                }
            ) {
                return Err(ParseError::UnboundAttachment(part.id));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotPart {
    pub id: u32,
    pub headers: Vec<SnapshotHeader>,
    pub content_transfer_encoding: Option<String>,
    pub is_encoding_problem: bool,
    pub body: SnapshotPartBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SnapshotPartBody {
    Text {
        content: String,
    },
    Html {
        content: String,
    },
    Multipart {
        children: Vec<u32>,
    },
    Attachment {
        attachment_id: Option<String>,
        filename: Option<String>,
        content_type: String,
        size: usize,
        inline: bool,
    },
    Binary {
        size: usize,
        inline: bool,
    },
}

#[derive(Debug, Clone)]
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
    pub snapshot: ParsedMailSnapshotV1,
}

#[derive(Debug, Clone)]
pub struct ParsedAttachment {
    pub part_id: u32,
    pub filename: Option<String>,
    pub content_type: String,
    pub body: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("failed to parse email")]
    InvalidInput,
    #[error("MIME part {0} is not an attachment")]
    InvalidAttachmentPart(u32),
    #[error("MIME attachment part {0} has no persisted attachment id")]
    UnboundAttachment(u32),
}

#[tracing::instrument(skip(raw))]
pub fn parse_message(raw: &[u8]) -> Result<ParsedMessage, ParseError> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or(ParseError::InvalidInput)?;

    let snapshot = build_snapshot(&message);
    let mut result = ParsedMessage {
        message_id: snapshot.message_id.clone(),
        subject: snapshot.subject.clone(),
        from: snapshot.from.clone(),
        to: snapshot.to.clone(),
        cc: snapshot.cc.clone(),
        reply_to: snapshot.reply_to.clone(),
        in_reply_to: snapshot.in_reply_to.first().cloned(),
        date: snapshot.date.clone(),
        body_text: snapshot.primary_body_text.clone(),
        body_html: snapshot.primary_body_html.clone(),
        attachments: Vec::new(),
        snapshot,
    };

    for part_id in &message.attachments {
        let part = &message.parts[*part_id as usize];
        result.attachments.push(ParsedAttachment {
            part_id: *part_id,
            filename: part.attachment_name().map(|s| s.to_string()),
            content_type: format_content_type(part),
            body: part.contents().to_vec(),
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

fn build_snapshot(message: &Message<'_>) -> ParsedMailSnapshotV1 {
    let attachment_ids: HashSet<u32> = message.attachments.iter().copied().collect();
    let parts = message
        .parts
        .iter()
        .enumerate()
        .map(|(index, part)| {
            let id = index as u32;
            let body = if attachment_ids.contains(&id) {
                SnapshotPartBody::Attachment {
                    attachment_id: None,
                    filename: part.attachment_name().map(ToString::to_string),
                    content_type: format_content_type(part),
                    size: part.len(),
                    inline: matches!(part.body, PartType::InlineBinary(_)),
                }
            } else {
                match &part.body {
                    PartType::Text(content) => SnapshotPartBody::Text {
                        content: content.to_string(),
                    },
                    PartType::Html(content) => SnapshotPartBody::Html {
                        content: content.to_string(),
                    },
                    PartType::Multipart(children) => SnapshotPartBody::Multipart {
                        children: children.clone(),
                    },
                    PartType::Binary(content) => SnapshotPartBody::Binary {
                        size: content.len(),
                        inline: false,
                    },
                    PartType::InlineBinary(content) => SnapshotPartBody::Binary {
                        size: content.len(),
                        inline: true,
                    },
                    PartType::Message(content) => SnapshotPartBody::Binary {
                        size: content.raw_message().len(),
                        inline: false,
                    },
                }
            };

            SnapshotPart {
                id,
                headers: snapshot_headers(&part.headers, &message.raw_message),
                content_transfer_encoding: part
                    .content_transfer_encoding()
                    .map(ToString::to_string),
                is_encoding_problem: part.is_encoding_problem,
                body,
            }
        })
        .collect();

    ParsedMailSnapshotV1 {
        version: SNAPSHOT_VERSION,
        headers: snapshot_headers(message.headers(), &message.raw_message),
        message_id: message.message_id().map(|value| value.trim().to_string()),
        subject: message.subject().unwrap_or_default().to_string(),
        from: extract_addresses(message.from()),
        to: extract_addresses(message.to()),
        cc: extract_addresses(message.cc()),
        bcc: extract_addresses(message.bcc()),
        reply_to: extract_addresses(message.reply_to()),
        sender: extract_addresses(message.sender()),
        in_reply_to: extract_message_id_headers(message.in_reply_to()),
        references: extract_message_id_headers(message.references()),
        date: message.date().map(|value| value.to_rfc3339()),
        text_body: message.text_body.clone(),
        html_body: message.html_body.clone(),
        primary_body_text: message.body_text(0).map(|value| value.into_owned()),
        primary_body_html: message.body_html(0).map(|value| value.into_owned()),
        parts,
    }
}

fn snapshot_headers(headers: &[Header<'_>], raw: &[u8]) -> Vec<SnapshotHeader> {
    headers
        .iter()
        .map(|header| SnapshotHeader {
            name: header.name.as_str().to_string(),
            value: format_header_value(&header.value),
            raw_value: raw
                .get(header.offset_start as usize..header.offset_end as usize)
                .map(|value| String::from_utf8_lossy(value).into_owned())
                .unwrap_or_default(),
        })
        .collect()
}

fn format_content_type(part: &mail_parser::MessagePart<'_>) -> String {
    part.content_type()
        .map(|content_type| {
            let subtype = content_type
                .c_subtype
                .as_ref()
                .map(|value| value.as_ref())
                .unwrap_or("");
            if subtype.is_empty() {
                content_type.c_type.to_string()
            } else {
                format!("{}/{subtype}", content_type.c_type)
            }
        })
        .unwrap_or_default()
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

fn extract_message_id_headers(value: &HeaderValue<'_>) -> Vec<String> {
    match value {
        HeaderValue::Text(value) => vec![value.trim().to_string()],
        HeaderValue::TextList(values) => values
            .iter()
            .map(|value| value.trim().to_string())
            .collect(),
        _ => Vec::new(),
    }
}

fn format_header_value(value: &HeaderValue<'_>) -> String {
    match value {
        HeaderValue::Text(s) => s.to_string(),
        HeaderValue::TextList(list) => list.join(", "),
        HeaderValue::DateTime(dt) => dt.to_rfc3339(),
        HeaderValue::Address(addr) => format_address_value(addr),
        HeaderValue::ContentType(ct) => {
            let sub = ct.c_subtype.as_ref().map(|s| s.as_ref()).unwrap_or("");
            format!("{}/{}", ct.c_type, sub)
        }
        HeaderValue::Received(received) => format!("{received:?}"),
        HeaderValue::Empty => String::new(),
    }
}

fn format_address_value(addr: &Address<'_>) -> String {
    match addr {
        Address::List(list) => list
            .iter()
            .map(|a| {
                let name = a.name().unwrap_or_default();
                let address = a.address().unwrap_or_default();
                if name.is_empty() {
                    address.to_string()
                } else {
                    format!("{name} <{address}>")
                }
            })
            .collect::<Vec<_>>()
            .join(", "),
        Address::Group(groups) => groups
            .iter()
            .flat_map(|g| g.addresses.iter())
            .map(|a| {
                let name = a.name().unwrap_or_default();
                let address = a.address().unwrap_or_default();
                if name.is_empty() {
                    address.to_string()
                } else {
                    format!("{name} <{address}>")
                }
            })
            .collect::<Vec<_>>()
            .join(", "),
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

    #[test]
    fn test_snapshot_headers_simple_email() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: text/plain\r\n\r\nThis is the body.";
        let headers = parse_message(raw).unwrap().snapshot.headers;
        let names: Vec<&str> = headers.iter().map(|h| h.name.as_str()).collect();
        assert!(names.contains(&"From"));
        assert!(names.contains(&"To"));
        assert!(names.contains(&"Subject"));
        assert!(names.contains(&"Content-Type"));
        let from_header = headers.iter().find(|h| h.name == "From").unwrap();
        assert_eq!(from_header.value, "sender@example.com");
        let subject_header = headers.iter().find(|h| h.name == "Subject").unwrap();
        assert_eq!(subject_header.value, "Hello");
    }

    #[test]
    fn test_snapshot_returns_all_headers() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nMessage-ID: <123@example.com>\r\nDate: Mon, 08 Jun 2026 06:39:51 +0000\r\nContent-Type: text/plain\r\n\r\nBody.";
        let headers = parse_message(raw).unwrap().snapshot.headers;
        assert_eq!(headers.len(), 6);
    }

    #[test]
    fn test_snapshot_headers_multipart() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/alternative; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nPlain text\r\n--boundary123\r\nContent-Type: text/html\r\n\r\n<html><body>HTML</body></html>\r\n--boundary123--";
        let headers = parse_message(raw).unwrap().snapshot.headers;
        let names: Vec<&str> = headers.iter().map(|h| h.name.as_str()).collect();
        assert!(names.contains(&"From"));
        assert!(names.contains(&"Content-Type"));
    }

    #[test]
    fn test_snapshot_headers_invalid_input_returns_ok() {
        let headers = parse_message(b"totally not an email")
            .unwrap()
            .snapshot
            .headers;
        assert!(headers.len() <= 3);
    }

    #[test]
    fn snapshot_preserves_header_order_duplicates_and_raw_values() {
        let raw = b"Received: from first.example by mx.example\r\nReceived: from second.example by mx.example\r\nSubject: =?UTF-8?B?5rWL6K+V?=\r\nFrom: sender@example.com\r\nTo: recipient@example.com\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nBody";

        let parsed = parse_message(raw).unwrap();
        let headers = &parsed.snapshot.headers;

        assert_eq!(headers[0].name, "Received");
        assert_eq!(headers[1].name, "Received");
        let subject = headers
            .iter()
            .find(|header| header.name == "Subject")
            .unwrap();
        assert_eq!(subject.value, "测试");
        assert!(subject.raw_value.contains("=?UTF-8?B?5rWL6K+V?="));
        let content_type = headers
            .iter()
            .find(|header| header.name == "Content-Type")
            .unwrap();
        assert!(content_type.raw_value.contains("charset=utf-8"));
    }

    #[test]
    fn snapshot_preserves_multipart_relationships_and_primary_bodies() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/alternative; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nPlain text\r\n--boundary123\r\nContent-Type: text/html\r\n\r\n<html><body>HTML</body></html>\r\n--boundary123--";

        let parsed = parse_message(raw).unwrap();

        assert_eq!(parsed.snapshot.body_text(), Some("Plain text"));
        assert_eq!(
            parsed.snapshot.body_html(),
            Some("<html><body>HTML</body></html>")
        );
        assert_eq!(parsed.snapshot.parts.len(), 3);
        assert_eq!(parsed.snapshot.text_body, vec![1]);
        assert_eq!(parsed.snapshot.html_body, vec![2]);
        assert!(matches!(
            &parsed.snapshot.parts[0].body,
            SnapshotPartBody::Multipart { children } if children == &vec![1, 2]
        ));
    }

    #[test]
    fn snapshot_uses_attachment_placeholders_without_attachment_bytes() {
        let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nSee attached\r\n--boundary123\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"data.bin\"\r\n\r\nBINARYDATA\r\n--boundary123--";

        let mut parsed = parse_message(raw).unwrap();
        let attachment = &parsed.attachments[0];
        assert_eq!(attachment.body, b"BINARYDATA");
        parsed
            .snapshot
            .bind_attachment_id(attachment.part_id, "att-1".to_string())
            .unwrap();

        let json = serde_json::to_string(&parsed.snapshot).unwrap();
        assert!(json.contains("\"attachment_id\":\"att-1\""));
        assert!(!json.contains("BINARYDATA"));
        assert!(matches!(
            &parsed.snapshot.parts[attachment.part_id as usize].body,
            SnapshotPartBody::Attachment {
                attachment_id: Some(id),
                filename: Some(filename),
                size: 10,
                ..
            } if id == "att-1" && filename == "data.bin"
        ));
    }
}
