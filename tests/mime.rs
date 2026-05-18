use std::time::{Duration, SystemTime};

use philharmonic_connector_impl_email_smtp::{
    EmailSmtpRequest, normalize_mime_envelope, normalize_mime_envelope_with,
};

const GENERATED_ID: &str = "<generated@philharmonic.local>";

fn fixed_time() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(0)
}

#[test]
fn existing_required_headers_pass_through_except_line_endings() {
    let body = concat!(
        "Date: Tue, 01 Jan 2030 00:00:00 +0000\r\n",
        "Message-Id: <already@example.com>\r\n",
        "MIME-Version: 1.0\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "From: alerts@example.com\r\n",
        "\r\n",
        "Hello."
    );

    let normalized = normalize_mime_envelope_with(body, fixed_time(), GENERATED_ID);

    assert_eq!(normalized.body, body);
    assert_eq!(normalized.message_id, "<already@example.com>");
}

#[test]
fn missing_required_headers_are_inserted() {
    let normalized =
        normalize_mime_envelope_with("Subject: Test\n\nHello.", fixed_time(), GENERATED_ID);

    assert!(normalized.body.starts_with(concat!(
        "MIME-Version: 1.0\r\n",
        "Date: Thu, 01 Jan 1970 00:00:00 GMT\r\n",
        "Message-Id: <generated@philharmonic.local>\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "Subject: Test\r\n",
        "\r\n"
    )));
    assert_eq!(normalized.message_id, GENERATED_ID);
}

#[test]
fn header_detection_is_case_insensitive() {
    let body = concat!(
        "mime-version: 1.0\n",
        "Date: Tue, 01 Jan 2030 00:00:00 +0000\n",
        "Message-ID: <already@example.com>\n",
        "Content-Type: text/plain; charset=utf-8\n",
        "\n",
        "Hello."
    );

    let normalized = normalize_mime_envelope_with(body, fixed_time(), GENERATED_ID);

    assert_eq!(normalized.body.matches("MIME-Version: 1.0").count(), 0);
    assert!(normalized.body.contains("mime-version: 1.0\r\n"));
    assert_eq!(normalized.message_id, "<already@example.com>");
}

#[test]
fn existing_date_is_not_reformatted() {
    let body = concat!(
        "Date: Fri, 21 Nov 1997 09:55:06 -0600\n",
        "Message-Id: <already@example.com>\n",
        "MIME-Version: 1.0\n",
        "Content-Type: text/plain; charset=utf-8\n",
        "\n",
        "Hello."
    );

    let normalized = normalize_mime_envelope_with(body, fixed_time(), GENERATED_ID);

    assert!(
        normalized
            .body
            .contains("Date: Fri, 21 Nov 1997 09:55:06 -0600\r\n")
    );
    assert!(!normalized.body.contains("Thu, 01 Jan 1970"));
}

#[test]
fn lf_only_line_endings_are_normalized_to_crlf() {
    let normalized = normalize_mime_envelope_with(
        "MIME-Version: 1.0\nDate: D\nMessage-Id: <m@id>\nContent-Type: text/plain\n\nA\nB",
        fixed_time(),
        GENERATED_ID,
    );

    assert_eq!(
        normalized.body,
        "MIME-Version: 1.0\r\nDate: D\r\nMessage-Id: <m@id>\r\nContent-Type: text/plain\r\n\r\nA\r\nB"
    );
}

#[test]
fn mixed_line_endings_are_normalized_to_crlf() {
    let normalized = normalize_mime_envelope_with(
        "MIME-Version: 1.0\r\nDate: D\nMessage-Id: <m@id>\rContent-Type: text/plain\r\n\r\nA\nB\rC",
        fixed_time(),
        GENERATED_ID,
    );

    assert_eq!(
        normalized.body,
        "MIME-Version: 1.0\r\nDate: D\r\nMessage-Id: <m@id>\r\nContent-Type: text/plain\r\n\r\nA\r\nB\r\nC"
    );
}

#[test]
fn from_header_is_preserved_without_rewrite() {
    let normalized = normalize_mime_envelope_with(
        "From: Original <original@example.com>\n\nHello.",
        fixed_time(),
        GENERATED_ID,
    );

    assert!(
        normalized
            .body
            .contains("From: Original <original@example.com>\r\n")
    );
}

#[test]
fn body_without_header_separator_is_treated_as_body() {
    let normalized = normalize_mime_envelope_with("Hello.", fixed_time(), GENERATED_ID);

    assert!(normalized.body.ends_with("\r\n\r\nHello."));
}

#[test]
fn empty_body_is_invalid_request() {
    let request = EmailSmtpRequest {
        mail_from: "alerts@example.com".to_owned(),
        recipients: vec!["ops@example.com".to_owned()],
        body: String::new(),
    };

    let error = request.prepare().expect_err("empty body rejected");
    assert!(error.to_string().contains("body required"));
}

#[test]
fn generated_message_id_uses_philharmonic_local_suffix() {
    let normalized = normalize_mime_envelope("Hello.");

    assert!(normalized.message_id.starts_with('<'));
    assert!(normalized.message_id.ends_with("@philharmonic.local>"));
}
