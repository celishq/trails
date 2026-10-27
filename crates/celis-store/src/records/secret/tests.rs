use super::Secret;

#[test]
fn each_form_of_a_secret_is_found() {
    for (text, kind) in [
        ("API_KEY=sk-live-7f3a9c2e81d4b06a", "an API key"),
        ("-----BEGIN OPENSSH PRIVATE KEY-----", "a private key"),
        ("AKIAIOSFODNN7EXAMPLE", "an AWS key"),
        ("ghp_0123456789abcdefghijABCDEFGHIJ012345", "a GitHub token"),
        ("xoxb-1234567890-abcdef", "a Slack token"),
        ("password = \"hunter2hunter2x9\"", "a secret value"),
        ("{\"token\": \"a1b2c3d4e5f6g7h8\"}", "a secret value"),
    ] {
        assert_eq!(Secret::find(text), Some(Secret { kind }), "{text}");
    }
}

#[test]
fn a_secret_named_by_where_it_lives_is_none() {
    for text in [
        "API_KEY in .env",
        "return os.environ['API_KEY']",
        "token = os.getenv(\"TOKEN\")",
        "password_hash_v2 is slow on purpose",
        "sk_values_for_the_test_suite",
        "the task-scheduler-for-reports",
    ] {
        assert_eq!(Secret::find(text), None, "{text}");
    }
}
