use std::time::SystemTime;

use philharmonic_connector_common::Uuid;

const MIME_VERSION: &str = "mime-version";
const DATE: &str = "date";
const MESSAGE_ID: &str = "message-id";
const CONTENT_TYPE: &str = "content-type";

/// MIME body after minimal envelope fixing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedMime {
    /// CRLF-normalized MIME message bytes, represented as UTF-8 text.
    pub body: String,
    /// Message-Id header value used for this submission.
    pub message_id: String,
}

/// Apply Philharmonic's minimal SMTP MIME envelope fixes.
pub fn normalize_mime_envelope(body: &str) -> NormalizedMime {
    normalize_mime_envelope_with(
        body,
        SystemTime::now(),
        &format!("<{}@philharmonic.local>", Uuid::new_v4()),
    )
}

#[doc(hidden)]
pub fn normalize_mime_envelope_with(
    body: &str,
    now: SystemTime,
    generated_message_id: &str,
) -> NormalizedMime {
    let normalized_lf = normalize_to_lf(body);
    let (header_block, body_block) = split_header_body(&normalized_lf);
    let header_lines = header_lines(header_block);
    let headers = HeaderPresence::from_lines(&header_lines);

    let message_id = headers
        .message_id
        .clone()
        .unwrap_or_else(|| generated_message_id.to_owned());

    let mut fixed_headers = Vec::new();
    if !headers.has_mime_version {
        fixed_headers.push("MIME-Version: 1.0".to_owned());
    }
    if !headers.has_date {
        fixed_headers.push(format!("Date: {}", httpdate::fmt_http_date(now)));
    }
    if headers.message_id.is_none() {
        fixed_headers.push(format!("Message-Id: {generated_message_id}"));
    }
    if !headers.has_content_type {
        fixed_headers.push("Content-Type: text/plain; charset=utf-8".to_owned());
    }
    fixed_headers.extend(header_lines.into_iter().map(ToOwned::to_owned));

    let fixed_lf = if fixed_headers.is_empty() {
        format!("\n{body_block}")
    } else {
        format!("{}\n\n{body_block}", fixed_headers.join("\n"))
    };

    NormalizedMime {
        body: fixed_lf.replace('\n', "\r\n"),
        message_id,
    }
}

fn normalize_to_lf(body: &str) -> String {
    body.replace("\r\n", "\n").replace('\r', "\n")
}

fn split_header_body(body: &str) -> (&str, &str) {
    if let Some((headers, rest)) = body.split_once("\n\n") {
        (headers, rest)
    } else {
        ("", body)
    }
}

fn header_lines(header_block: &str) -> Vec<&str> {
    if header_block.is_empty() {
        Vec::new()
    } else {
        header_block.split('\n').collect()
    }
}

#[derive(Clone, Debug, Default)]
struct HeaderPresence {
    has_mime_version: bool,
    has_date: bool,
    message_id: Option<String>,
    has_content_type: bool,
}

impl HeaderPresence {
    fn from_lines(lines: &[&str]) -> Self {
        let mut presence = Self::default();
        for line in lines {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            match name.trim().to_ascii_lowercase().as_str() {
                MIME_VERSION => presence.has_mime_version = true,
                DATE => presence.has_date = true,
                MESSAGE_ID => presence.message_id = Some(value.trim().to_owned()),
                CONTENT_TYPE => presence.has_content_type = true,
                _ => {}
            }
        }
        presence
    }
}
