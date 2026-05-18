use philharmonic_connector_impl_email_smtp::{
    ConnectionMode, DeliveryAttempt, EmailSmtpConfig, TlsChannel, TlsPlan, TlsStrictness,
    TransportMode,
};
use serde_json::json;

fn config(value: serde_json::Value) -> EmailSmtpConfig {
    serde_json::from_value(value).expect("valid config JSON")
}

#[test]
fn connection_mode_matrix_matches_locked_policy() {
    let cases = [
        (
            json!({"host":"smtp.example.com","port":587,"connection_mode":"starttls","username":"u","password":"p"}),
            vec![DeliveryAttempt {
                port: 587,
                transport_mode: TransportMode::Starttls,
                tls_strictness: TlsStrictness::Strict,
            }],
        ),
        (
            json!({"host":"smtp.example.com","port":465,"connection_mode":"smtps","username":"u","password":"p"}),
            vec![DeliveryAttempt {
                port: 465,
                transport_mode: TransportMode::Smtps,
                tls_strictness: TlsStrictness::Strict,
            }],
        ),
        (
            json!({"host":"smtp.example.com","port":587,"connection_mode":"auto","username":"u","password":"p"}),
            vec![DeliveryAttempt {
                port: 587,
                transport_mode: TransportMode::Starttls,
                tls_strictness: TlsStrictness::Strict,
            }],
        ),
        (
            json!({"host":"smtp.example.com","port":465,"connection_mode":"auto","username":"u","password":"p"}),
            vec![DeliveryAttempt {
                port: 465,
                transport_mode: TransportMode::Smtps,
                tls_strictness: TlsStrictness::Strict,
            }],
        ),
        (
            json!({"host":"smtp.example.com","port":2525,"connection_mode":"auto","username":"u","password":"p"}),
            vec![DeliveryAttempt {
                port: 2525,
                transport_mode: TransportMode::Starttls,
                tls_strictness: TlsStrictness::Strict,
            }],
        ),
        (
            json!({"host":"smtp.example.com","connection_mode":"auto","username":"u","password":"p"}),
            vec![
                DeliveryAttempt {
                    port: 587,
                    transport_mode: TransportMode::Starttls,
                    tls_strictness: TlsStrictness::Strict,
                },
                DeliveryAttempt {
                    port: 465,
                    transport_mode: TransportMode::Smtps,
                    tls_strictness: TlsStrictness::Strict,
                },
            ],
        ),
        (
            json!({"host":"smtp.example.com","port":465,"connection_mode":"starttls","username":"u","password":"p"}),
            vec![DeliveryAttempt {
                port: 465,
                transport_mode: TransportMode::Starttls,
                tls_strictness: TlsStrictness::Strict,
            }],
        ),
        (
            json!({"host":"smtp.example.com","port":587,"connection_mode":"smtps","username":"u","password":"p"}),
            vec![DeliveryAttempt {
                port: 587,
                transport_mode: TransportMode::Smtps,
                tls_strictness: TlsStrictness::Strict,
            }],
        ),
    ];

    for (input, expected) in cases {
        let prepared = config(input).prepare().expect("config prepares");
        assert_eq!(prepared.attempts(), expected.as_slice());
    }
}

#[test]
fn port_25_is_rejected_for_every_mode() {
    for mode in ["starttls", "smtps", "auto"] {
        let error = config(json!({
            "host": "smtp.example.com",
            "port": 25,
            "connection_mode": mode,
            "username": "u",
            "password": "p"
        }))
        .prepare()
        .expect_err("port 25 rejected");
        assert!(error.to_string().contains("port 25"));
    }
}

#[test]
fn username_and_password_are_required() {
    let missing_username = config(json!({
        "host": "smtp.example.com",
        "port": 587,
        "username": "",
        "password": "p"
    }))
    .prepare()
    .expect_err("missing username rejected");
    assert!(missing_username.to_string().contains("username required"));

    let missing_password = config(json!({
        "host": "smtp.example.com",
        "port": 587,
        "username": "u",
        "password": ""
    }))
    .prepare()
    .expect_err("missing password rejected");
    assert!(missing_password.to_string().contains("password required"));
}

#[test]
fn explicit_mode_without_port_is_invalid() {
    for mode in ["starttls", "smtps"] {
        let error = config(json!({
            "host": "smtp.example.com",
            "connection_mode": mode,
            "username": "u",
            "password": "p"
        }))
        .prepare()
        .expect_err("explicit mode without port rejected");
        assert!(error.to_string().contains("port required"));
    }
}

#[test]
fn tls_strictness_selects_verification_and_opportunistic_starttls() {
    let cases = [
        (
            "strict",
            DeliveryAttempt {
                port: 587,
                transport_mode: TransportMode::Starttls,
                tls_strictness: TlsStrictness::Strict,
            },
            TlsPlan {
                channel: TlsChannel::StarttlsRequired,
                skips_server_identity: false,
            },
        ),
        (
            "sloppy",
            DeliveryAttempt {
                port: 587,
                transport_mode: TransportMode::Starttls,
                tls_strictness: TlsStrictness::Sloppy,
            },
            TlsPlan {
                channel: TlsChannel::StarttlsRequired,
                skips_server_identity: true,
            },
        ),
        (
            "opportunistic",
            DeliveryAttempt {
                port: 587,
                transport_mode: TransportMode::Starttls,
                tls_strictness: TlsStrictness::Opportunistic,
            },
            TlsPlan {
                channel: TlsChannel::StarttlsOpportunistic,
                skips_server_identity: false,
            },
        ),
        (
            "opportunistic_sloppy",
            DeliveryAttempt {
                port: 587,
                transport_mode: TransportMode::Starttls,
                tls_strictness: TlsStrictness::OpportunisticSloppy,
            },
            TlsPlan {
                channel: TlsChannel::StarttlsOpportunistic,
                skips_server_identity: true,
            },
        ),
    ];

    for (strictness, expected_attempt, expected_plan) in cases {
        let prepared = config(json!({
            "host": "smtp.example.com",
            "port": 587,
            "username": "u",
            "password": "p",
            "tls_strictness": strictness
        }))
        .prepare()
        .expect("config prepares");

        assert_eq!(prepared.attempts(), &[expected_attempt]);
        assert_eq!(prepared.attempts()[0].tls_plan(), expected_plan);
    }
}

#[test]
fn serde_lowercase_enums_round_trip_from_json() {
    let value = json!({
        "host": "smtp.example.com",
        "port": 587,
        "connection_mode": "starttls",
        "username": "u",
        "password": "p",
        "tls_strictness": "opportunistic_sloppy"
    });

    let parsed = config(value);
    assert_eq!(parsed.connection_mode, ConnectionMode::Starttls);
    assert_eq!(parsed.tls_strictness, TlsStrictness::OpportunisticSloppy);
}
