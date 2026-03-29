//! Comprehensive betterleaks rule coverage tests.
//!
//! Every rule from the betterleaks rule set is tested here with a true-positive
//! fixture. Test tokens are taken directly from betterleaks' own Go rule files
//! (`cmd/generate/config/rules/*.go`) where possible; others are constructed to
//! satisfy the published regex pattern.
//!
//! Test strategy: assert that the specific rule ID is present among the findings.
//! We do not assert exact counts because generic/catch-all rules may also fire,
//! but our deduplication logic suppresses them when a specific rule overlaps.

use chub_core::scan::scanner::{ScanOptions, Scanner};

fn scanner() -> Scanner {
    Scanner::new(ScanOptions::default())
}

/// Assert that scanning `input` against `file` produces at least one finding
/// with the given `rule_id`.
fn assert_detects(scanner: &Scanner, rule_id: &str, input: &str) {
    let findings = scanner.scan_text(input, "test.env", None);
    assert!(
        findings.iter().any(|f| f.rule_id == rule_id),
        "rule '{}' not detected.\ninput: {:?}\nfound: {:?}",
        rule_id,
        &input[..input.len().min(120)],
        findings
            .iter()
            .map(|f| f.rule_id.as_str())
            .collect::<Vec<_>>()
    );
}

/// Assert no finding with `rule_id` is produced (false-positive check).
fn assert_no_detect(scanner: &Scanner, rule_id: &str, input: &str) {
    let findings = scanner.scan_text(input, "test.env", None);
    assert!(
        !findings.iter().any(|f| f.rule_id == rule_id),
        "rule '{}' fired on false-positive input: {:?}",
        rule_id,
        &input[..input.len().min(120)],
    );
}

// ---------------------------------------------------------------------------
// 1Password
// ---------------------------------------------------------------------------

#[test]
fn test_1password_service_account_token() {
    let s = scanner();
    // Real token from betterleaks test suite
    assert_detects(
        &s,
        "1password-service-account-token",
        "PYTEST_SVC_ACCT_TOKEN=ops_eyJzaWduSW5BZGRyZXNzIjoiemFjaC1hbmQtbGVhbm5lLjFwYXNzd29yZC5jb20iLCJ1c2VyQXV0aCI6eyJtZXRob2QiOiJTUlBnLTQwOTYiLCJhbGciOiJQQkVLMmctSFMyNTYiLCJpdGVyYXRpb25zIjo2NTAwMDAsInNhbHQiOiJlYUZRQmNVemJyTHhnM2d4bHFQLVVBIn0sImVtYWlsIjoiMm9iNGRpeDdiNTdrYUAxcGFzc3dvcmRzZXJ2aWNlYWNjb3VudHMuY29tIiwic3JwWCI6ImVmZDY4YjNhZTkwMmRjZjRiMzEzYjE5MjYwZmY0OGUzMjU2ZDlhOGNkM2JmMmY3YzI2YzU1ZWJkNjZlZGU4NWEiLCJtdWsiOnsiYWxnIjoiQTI1NkdDTSIsImV4dCI6dHJ1ZSwiayI6IlMwaGE0SDhqbEhRblJCWmxvYnBmR1BneERmbS1pRGNkZWY0bFdYU0VSbmMiLCJrZXlfb3BzIjpbImRlY3J5cHQiLCJlbmNyeXB0Il0sImt0eSI6Im9jdCIsImtpZCI6Im1wIn0sInNlY3JldEtleSI6IkEzLUdHOUVRNi1LUzQ0QVctQU5QVkYtUkdQTDktQlNKUTMtR1NHR0giLCJ0aHJvdHRsZVNlY3JldCI6eyJzZWVkIjoiN2I0OTMxMmJiOTlkZTFiNjU5ODZkYzIzOWU4YWNmZWMxMTU0M2E2OGQxYmYwMjZmZTgzMjg3NWYxNmJlOWY2NiIsInV1aWQiOiJDV1RHQ0hMNlNWRkdSTlg0SzNENUJVSDZDSSJ9LCJkZXZpY2VVdWlkIjoiMnFld3JpaGtqbmt1Zmh6ZGdmZ2hnNmM1cGUifQ",
    );
}

#[test]
fn test_1password_secret_key() {
    let s = scanner();
    // From betterleaks whitepaper example
    assert_detects(
        &s,
        "1password-secret-key",
        "OP_SECRET_KEY=A3-ASWWYB-798JRYLJVD4-23DC2-86TVM-H43EB",
    );
    assert_detects(
        &s,
        "1password-secret-key",
        "OP_SECRET_KEY=A3-ASWWYB-798JRY-LJVD4-23DC2-86TVM-H43EB",
    );
}

// ---------------------------------------------------------------------------
// Age encryption
// ---------------------------------------------------------------------------

#[test]
fn test_age_secret_key() {
    let s = scanner();
    // bech32 charset for body; 58 chars required after prefix
    assert_detects(
        &s,
        "age-secret-key",
        "age-secret-key: AGE-SECRET-KEY-1QPZRY9X8GF2TVDW0S3JN54KHCE6MUA7LQPZRY9X8GF2TVDW0S3JN54KHCE",
    );
}

// ---------------------------------------------------------------------------
// AI / ML providers
// ---------------------------------------------------------------------------

#[test]
fn test_anthropic_admin_api_key() {
    let s = scanner();
    // 93 alnum/_/- chars followed by "AA"
    assert_detects(
        &s,
        "anthropic-admin-api-key",
        "ANTHROPIC_ADMIN_KEY=sk-ant-admin01-abc12fake-456def789ghij-klmnopqrstuvwx-3456yza789bcde-12fakehijklmnopby56aaaogaopaaaabc123xyzAA",
    );
}

#[test]
fn test_assemblyai_api_key() {
    let s = scanner();
    // 32 lowercase alnum in assemblyai context
    assert_detects(
        &s,
        "assemblyai-api-key",
        "assemblyai = fa0ed91518b345468f9df75714d3e82a",
    );
}

#[test]
fn test_cerebras_api_key() {
    let s = scanner();
    // 48 lowercase alnum after csk- prefix
    assert_detects(
        &s,
        "cerebras-api-key",
        "CEREBRAS_KEY=csk-6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f",
    );
}

#[test]
fn test_cursor_api_key() {
    let s = scanner();
    // 64 hex chars after key_ prefix
    assert_detects(
        &s,
        "cursor-api-key",
        "cursor_key = key_8c5a7657fc397e114def1b51dd52041a7b3c2d4e5f6a7b8c9d0e1f2a3b4c5d6e",
    );
}

#[test]
fn test_deepseek_api_key() {
    let s = scanner();
    // sk- + 32 lowercase hex in deepseek context
    assert_detects(
        &s,
        "deepseek-api-key",
        "DEEPSEEK_API_KEY=sk-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

#[test]
fn test_mistral_api_key() {
    let s = scanner();
    // 32 uppercase alnum in mistral context (no alphabetical sequence to avoid global stopword)
    assert_detects(
        &s,
        "mistral-api-key",
        "MISTRAL_API_KEY=A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6",
    );
}

#[test]
fn test_nvidia_api_key() {
    let s = scanner();
    // nvapi- prefix + 60-70 uppercase alnum/dash/underscore chars
    assert_detects(
        &s,
        "nvidia-api-key",
        "nvidia_key=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",
    );
}

#[test]
fn test_ollama_api_key() {
    let s = scanner();
    // 32 hex + "." + 24 alnum in ollama context
    assert_detects(
        &s,
        "ollama-api-key",
        "ollama key = 8bcdd9b4e28e4e1b8bf14a2eb8701220.a1b2c3d4e5f6a7b8c9d0e1f2",
    );
}

#[test]
fn test_togetherai_api_key() {
    let s = scanner();
    // tgp_v1_ prefix + 43 alnum/dash/underscore chars
    assert_detects(
        &s,
        "togetherai-api-key",
        "TOGETHER_API_KEY=tgp_v1_Tctm6OfOeNkwLIKkyxJxUHIqNKx2AvFr12345678901",
    );
}

#[test]
fn test_xai_api_key() {
    let s = scanner();
    // xai- + 70-120 chars
    assert_detects(
        &s,
        "xai-api-key",
        "XAI_KEY=xai-k4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pX",
    );
}

#[test]
fn test_weights_and_biases() {
    let s = scanner();
    // 40 hex chars in wandb context
    assert_detects(
        &s,
        "weights-and-biases-api-key",
        "wandb_api_key = 872ab943740b34157041da2529fb160d4e5f6a7b",
    );
}

// ---------------------------------------------------------------------------
// Airtable
// ---------------------------------------------------------------------------

#[test]
fn test_airtable_personnal_access_token() {
    let s = scanner();
    // pat + 14 alnum + . + 64 lowercase hex  (tests POSIX [:alnum:] fix)
    assert_detects(
        &s,
        "airtable-personnal-access-token",
        "AIRTABLE_KEY=patABCdef012345GH.abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
    );
}

// ---------------------------------------------------------------------------
// Alibaba / Algolia
// ---------------------------------------------------------------------------

#[test]
fn test_alibaba_access_key_id() {
    let s = scanner();
    // LTAI + 20 alnum chars
    assert_detects(
        &s,
        "alibaba-access-key-id",
        "ALIBABA_KEY=LTAI5abc1234defg5678hij0",
    );
}

#[test]
fn test_algolia_api_key() {
    let s = scanner();
    // 32 lowercase alnum in algolia context
    assert_detects(
        &s,
        "algolia-api-key",
        "algolia_api_key=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// ---------------------------------------------------------------------------
// Atlassian / Jira / Confluence
// ---------------------------------------------------------------------------

#[test]
fn test_atlassian_api_token() {
    let s = scanner();
    // From betterleaks test fixture
    assert_detects(
        &s,
        "atlassian-api-token",
        "JIRA_API_TOKEN=HXe8DGg1iJd2AopzyxkFB7F2",
    );
    // ATATT3 format (exactly 186 alnum/dash/underscore/equals chars after prefix)
    // Uses interspersed digits to avoid the "abcdefghijklmnopqrstuvwxyz" global stopword.
    assert_detects(
        &s,
        "atlassian-api-token",
        "JIRA_TOKEN=ATATT3A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5",
    );
}

// ---------------------------------------------------------------------------
// AWS Bedrock
// ---------------------------------------------------------------------------

#[test]
fn test_aws_bedrock_short_lived() {
    let s = scanner();
    assert_detects(
        &s,
        "aws-amazon-bedrock-api-key-short-lived",
        "BEDROCK_KEY=bedrock-api-key-YmVkcm9jay5hbWF6b25hd3MuY29t",
    );
}

// ---------------------------------------------------------------------------
// Azure
// ---------------------------------------------------------------------------

#[test]
fn test_azure_ad_client_secret() {
    let s = scanner();
    // From betterleaks test fixture: 3 alnum + digit + Q~ + 31-34 chars
    assert_detects(
        &s,
        "azure-ad-client-secret",
        "client_secret=bP88Q~rcBcYjzzOhg1Hnn76Wm3jGgakZiZ.8vMgR",
    );
}

// ---------------------------------------------------------------------------
// Bitbucket
// ---------------------------------------------------------------------------

#[test]
fn test_bitbucket_client_id() {
    let s = scanner();
    // 32 alnum in bitbucket context
    assert_detects(
        &s,
        "bitbucket-client-id",
        "BITBUCKET_CLIENT_ID=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// ---------------------------------------------------------------------------
// Cloud providers
// ---------------------------------------------------------------------------

#[test]
fn test_cloudflare_api_key() {
    let s = scanner();
    // 40 alnum in cloudflare context
    assert_detects(
        &s,
        "cloudflare-api-key",
        "CLOUDFLARE_API_KEY=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

#[test]
fn test_cloudflare_origin_ca_key() {
    let s = scanner();
    assert_detects(
        &s,
        "cloudflare-origin-ca-key",
        "CLOUDFLARE_ORIGIN_KEY=v1.0-aaa334dc886f30631ba0a610-0d98ef66290d7e50aac7c27b5986c99e6f3f1084c881d8ac0eae5de1d1aa0644076ff57022069b3237d19afe60ad045f207ef2b16387ee37b749441b2ae2e9ebe5b4606e846475d4a5",
    );
}

#[test]
fn test_databricks_api_token() {
    let s = scanner();
    // dapi + 32 hex
    assert_detects(
        &s,
        "databricks-api-token",
        "DATABRICKS_TOKEN=dapif13ac4b49d1cb31f69f678e39602e381",
    );
}

#[test]
fn test_datadog_access_token() {
    let s = scanner();
    // 40 alnum in datadog context
    assert_detects(
        &s,
        "datadog-access-token",
        "DATADOG_API_KEY=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

#[test]
fn test_digitalocean_access_token() {
    let s = scanner();
    // dop_v1_ prefix = Personal Access Token (rule: digitalocean-pat)
    assert_detects(
        &s,
        "digitalocean-pat",
        "DO_ACCESS_TOKEN=dop_v1_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
    // doo_v1_ prefix = OAuth Access Token (rule: digitalocean-access-token)
    assert_detects(
        &s,
        "digitalocean-access-token",
        "DO_TOKEN=doo_v1_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

#[test]
fn test_dynatrace_api_token() {
    let s = scanner();
    // dt0c01. + 24 alnum + . + 64 alnum
    assert_detects(
        &s,
        "dynatrace-api-token",
        "DYNATRACE_TOKEN=dt0c01.a1b2c3d4e5f6a7b8c9d0e1f2.a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

#[test]
fn test_flyio_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "flyio-access-token",
        r#""BindToParentToken": "FlyV1 fm2_lJPEEKnzKy0lkwV3B+WIlmrdwejEEFv5qmevHU4fMs+2Gr6oOiPC2SAyOTc0NWI4ZmJlNjBlNjJmZTgzNTkxOThhZWE4MjY0M5IMxAMBAgPEIH7VG8u74KwO62hmx8SZO8WaU5o1g3W2IVc7QN6T1VTr""#,
    );
}

#[test]
fn test_heroku_api_key() {
    let s = scanner();
    // UUID format in heroku context
    assert_detects(
        &s,
        "heroku-api-key",
        r#"heroku_api_key = "832d2129-a846-4e27-99f4-7004b6ad53ef""#,
    );
}

#[test]
fn test_heroku_api_key_v2() {
    let s = scanner();
    assert_detects(
        &s,
        "heroku-api-key-v2",
        r#"API_Key = "HRKU-AAy9Ppr_HD2pPuTyIiTYInO0hbzhoERRSO93ZQusSYHgaD7_WQ07FnF7L9FX""#,
    );
}

// ---------------------------------------------------------------------------
// Communications / Collaboration
// ---------------------------------------------------------------------------

#[test]
fn test_discord_api_token() {
    let s = scanner();
    // 64 hex in discord context
    assert_detects(
        &s,
        "discord-api-token",
        "DISCORD_BOT_TOKEN=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

#[test]
fn test_mattermost_access_token() {
    let s = scanner();
    // 26 alnum in mattermost context
    assert_detects(
        &s,
        "mattermost-access-token",
        "MATTERMOST_TOKEN=a1b2c3d4e5f6a7b8c9d0e1f2a3",
    );
}

#[test]
fn test_microsoft_teams_webhook() {
    let s = scanner();
    assert_detects(
        &s,
        "microsoft-teams-webhook",
        "TEAMS_WEBHOOK=https://mycompany.webhook.office.com/webhookb2/a1b2c3d4-a1b2-c3d4-e5f6-a7b8c9d0e1f2@a3b4c5d6-a7b8-c9d0-e1f2-a3b4c5d6e7f8/IncomingWebhook/a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6/a9b0c1d2-a3b4-c5d6-e7f8-a9b0c1d2e3f4",
    );
}

#[test]
fn test_slack_bot_token() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-bot-token",
        "SLACK_BOT_TOKEN=xoxb-781236542736-2364535789652-GkwFDQoHqzXDVsC6GzqYUypD",
    );
}

#[test]
fn test_slack_app_token() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-app-token",
        r#"SLACK_APP_TOKEN=xapp-1-A052FGTS2DL-5171572773297-610b6a11f4b7eb819e87b767d80e6575a3634791acb9a9ead051da879eb5b55e"#,
    );
}

#[test]
fn test_slack_user_token() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-user-token",
        r#"slack_token = "xoxp-41684372915-1320496754-45609968301-e708ba56e1517a99f6b5fb07349476ef""#,
    );
}

#[test]
fn test_slack_config_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-config-access-token",
        r#"access_token = "xoxe.xoxp-1-Mi0yLTM0MTQwNDE0MDE3Ni0zNjU5NDY0Njg4MTctNTE4MjA3NTQ5NjA4MC01NDEyOTYyODY5NzUxLThhMTBjZmI1ZWIzMGIwNTg0ZDdmMDI5Y2UxNzVlZWVhYzU2ZWQyZTZiODNjNDZiMGUxMzRlNmNjNDEwYmQxMjQ""#,
    );
}

#[test]
fn test_slack_legacy_token() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-legacy-token",
        r#"slack_token = "xoxs-3206092076-3204538285-3743137121-836b042620""#,
    );
}

#[test]
fn test_telegram_bot_api_token() {
    let s = scanner();
    // digits:A + 34 lowercase alnum/underscore/dash (no alphabetical sequence to avoid global stopword)
    assert_detects(
        &s,
        "telegram-bot-api-token",
        "TELEGRAM_TOKEN=1234567890:Ab1c2d3e4f5g6h7i8j9k0l1m2n3o4p5q6r7",
    );
}

// ---------------------------------------------------------------------------
// Developer tools
// ---------------------------------------------------------------------------

#[test]
fn test_doppler_api_token() {
    let s = scanner();
    // dp.pt. + 43 alnum
    assert_detects(
        &s,
        "doppler-api-token",
        "DOPPLER_TOKEN=dp.pt.a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d",
    );
}

#[test]
fn test_dropbox_api_token() {
    let s = scanner();
    // 15 alnum in dropbox context
    assert_detects(&s, "dropbox-api-token", "DROPBOX_TOKEN=a1b2c3d4e5f6a7b");
}

#[test]
fn test_github_fine_grained_pat() {
    let s = scanner();
    // github_pat_ + 82 word chars (high-entropy variant)
    assert_detects(
        &s,
        "github-fine-grained-pat",
        "GITHUB_TOKEN=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",
    );
}

#[test]
fn test_github_oauth() {
    let s = scanner();
    assert_detects(
        &s,
        "github-oauth",
        "GITHUB_OAUTH=gho_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2",
    );
}

#[test]
fn test_gitlab_pat() {
    let s = scanner();
    // glpat- + exactly 20 word/dash chars
    assert_detects(&s, "gitlab-pat", "GITLAB_TOKEN=glpat-k4Jm8nR2pX6sW9vB3fHa");
}

#[test]
fn test_gitlab_runner_authentication_token() {
    let s = scanner();
    // glrt- prefix + exactly 20 alnum/dash/underscore chars
    assert_detects(
        &s,
        "gitlab-runner-authentication-token",
        "GITLAB_RUNNER_TOKEN=glrt-a1b2c3d4e5f6a7b8c9d0",
    );
}

#[test]
fn test_grafana_service_account_token() {
    let s = scanner();
    assert_detects(
        &s,
        "grafana-service-account-token",
        r#"Authorization: Bearer glsa_pITqMOBIfNH2KL4PkXJqmTyQl0D9QGxF_486f63e1"#,
    );
}

#[test]
fn test_grafana_cloud_api_token() {
    let s = scanner();
    // glc_ prefixed token
    assert_detects(
        &s,
        "grafana-cloud-api-token",
        "loki_key: glc_eyJvIjoiNzQ0NTg3IiwibiI7InN0YWlrLTQ3NTgzMC1obC13cml0ZS1oYW5kc29uJG9raSIsImsiOiI4M2w3cmdYUlBoMTUyMW1lMU023nl5UDUiLCJtIjp7IOIiOiJ1cyJ9fQ==",
    );
}

#[test]
fn test_harness_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "harness-api-key",
        r#"HARNESS_TOKEN="pat.AbCdEfGhIjKlMnOpQrStUv.0123abcd4567ef890123abcd.ZyXwVuTsRqPoNmLkJiHg""#,
    );
}

#[test]
fn test_hashicorp_tf_api_token() {
    let s = scanner();
    // 14 alnum + .atlasv1. + 60-70 alnum (tests (?-i:atlasv1) strip)
    assert_detects(
        &s,
        "hashicorp-tf-api-token",
        r#"#token = "hE1hlYILrSqpqh.atlasv1.ARjZuyzl33F71WR55s6ln5GQ1HWIwTDDH3MiRjz7OnpCfaCb1RCF5zGaSncCWmJdcYA""#,
    );
}

#[test]
fn test_intercom_api_key() {
    let s = scanner();
    // 60 alnum in intercom context
    assert_detects(
        &s,
        "intercom-api-key",
        "INTERCOM_TOKEN=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0",
    );
}

#[test]
fn test_jfrog_api_key() {
    let s = scanner();
    // 73 alnum in jfrog context
    assert_detects(
        &s,
        "jfrog-api-key",
        "JFROG_API_KEY=a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a",
    );
}

#[test]
fn test_launchdarkly_access_token() {
    let s = scanner();
    // 40 alnum in launchdarkly context
    assert_detects(
        &s,
        "launchdarkly-access-token",
        "LAUNCHDARKLY_SDK_KEY=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

#[test]
fn test_lob_api_key() {
    let s = scanner();
    // live_ + 35 hex in lob context
    assert_detects(
        &s,
        "lob-api-key",
        "LOB_API_KEY=live_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f",
    );
}

#[test]
fn test_snyk_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "snyk-api-token",
        "SNYK_TOKEN=12345678-ABCD-ABCD-ABCD-1234567890AB",
    );
}

#[test]
fn test_sonar_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "sonar-api-token",
        r#"const SONAR_LOGIN = "12345678ABCDEFH1234567890ABCDEFH12345678""#,
    );
}

#[test]
fn test_sourcegraph_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "sourcegraph-access-token",
        "SOURCEGRAPH_TOKEN=sgp_AaD80dc6E02eCAE1_d3cba16CC0F18fA14A2EFB61CbDFceEBf9fAD16b",
    );
}

// ---------------------------------------------------------------------------
// Email / Marketing
// ---------------------------------------------------------------------------

#[test]
fn test_mailchimp_api_key() {
    let s = scanner();
    // 32 hex + -us + 2 digits in mailchimp context
    assert_detects(
        &s,
        "mailchimp-api-key",
        "MAILCHIMP_KEY=3012a5754bbd716926f99c028f7ea428-us18",
    );
}

#[test]
fn test_mailgun_private_api_token() {
    let s = scanner();
    // key- + 32 hex in mailgun context
    assert_detects(
        &s,
        "mailgun-private-api-token",
        "MAILGUN_API_KEY=key-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

#[test]
fn test_sendgrid_api_token() {
    let s = scanner();
    // SG. + 66 chars
    assert_detects(
        &s,
        "sendgrid-api-token",
        "SENDGRID_KEY=SG.a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3",
    );
}

#[test]
fn test_sendinblue_api_token() {
    let s = scanner();
    // xkeysib- + 64 hex + - + 16 alnum
    assert_detects(
        &s,
        "sendinblue-api-token",
        "SENDINBLUE_KEY=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",
    );
}

// ---------------------------------------------------------------------------
// Monitoring / Observability
// ---------------------------------------------------------------------------

#[test]
fn test_sentry_access_token() {
    let s = scanner();
    // 64 hex in sentry context
    assert_detects(
        &s,
        "sentry-access-token",
        "SENTRY_AUTH_TOKEN=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

#[test]
fn test_sentry_org_token() {
    let s = scanner();
    assert_detects(
        &s,
        "sentry-org-token",
        "SENTRY_TOKEN=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",
    );
}

#[test]
fn test_sentry_user_token() {
    let s = scanner();
    // sntryu_ + 64 hex
    assert_detects(
        &s,
        "sentry-user-token",
        "SENTRY_TOKEN=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",
    );
}

#[test]
fn test_new_relic_insert_key() {
    let s = scanner();
    // NRII- prefix + 32 chars
    assert_detects(
        &s,
        "new-relic-insert-key",
        "NEW_RELIC_INSERT_KEY=NRII-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

#[test]
fn test_sumologic_access_id() {
    let s = scanner();
    assert_detects(
        &s,
        "sumologic-access-id",
        r#"sumologic.accessId = "su9OL59biWiJu7""#,
    );
}

#[test]
fn test_sumologic_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "sumologic-access-token",
        "SUMOLOGIC_ACCESSKEY: 9RITWb3I3kAnSyUolcVJq4gwM17JRnQK8ugRaixFfxkdSl8ys17ZtEL3LotESKB7",
    );
}

// ---------------------------------------------------------------------------
// Payment providers
// ---------------------------------------------------------------------------

#[test]
fn test_stripe_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "stripe-access-token",
        "STRIPE_KEY=sk_test_51OuEMLAlTWGaDypq4P5cuDHbuKeG4tAGPYHJpEXQ7zE8mKK3jkhTFPvCxnSSK5zB5EQZrJsYdsatNmAHGgb0vSKD00GTMSWRHs",
    );
}

#[test]
fn test_square_access_token() {
    let s = scanner();
    // EAAA + 22+ word chars
    assert_detects(
        &s,
        "square-access-token",
        "SQUARE_TOKEN=EAAAk4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2",
    );
}

#[test]
fn test_coinbase_access_token() {
    let s = scanner();
    // 64 hex in coinbase context
    assert_detects(
        &s,
        "coinbase-access-token",
        "COINBASE_API_KEY=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

// ---------------------------------------------------------------------------
// Social / Communication APIs
// ---------------------------------------------------------------------------

#[test]
fn test_facebook_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "facebook-secret",
        r#"facebook_app_secret = "6dca6432e45d933e13650d1882bd5e69""#,
    );
}

#[test]
fn test_twitch_api_token() {
    let s = scanner();
    // 30 alnum in twitch context
    assert_detects(
        &s,
        "twitch-api-token",
        "TWITCH_TOKEN=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5",
    );
}

#[test]
fn test_twitter_api_key() {
    let s = scanner();
    // exactly 25 alnum in twitter context
    assert_detects(
        &s,
        "twitter-api-key",
        "TWITTER_API_KEY=a1b2c3d4e5f6a7b8c9d0e1f2a",
    );
}

// ---------------------------------------------------------------------------
// Okta / Identity
// ---------------------------------------------------------------------------

#[test]
fn test_okta_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "okta-access-token",
        r#""oktaApiToken": "00ebObu4zSNkyc6dimLvUwq4KpTEop-PCEnnfSTpD3""#,
    );
}

#[test]
fn test_openshift_user_token() {
    let s = scanner();
    assert_detects(
        &s,
        "openshift-user-token",
        "Authorization: Bearer sha256~kV46hPnEYhCWFnB85r5NrprAxggzgb6GOeLbgcKNsH0",
    );
}

// ---------------------------------------------------------------------------
// Package managers / CI
// ---------------------------------------------------------------------------

#[test]
fn test_pypi_upload_token() {
    let s = scanner();
    // pypi-AgEIcHlwaS5vcmc + 50+ word chars (no alphabetical sequence to avoid global stopword)
    assert_detects(
        &s,
        "pypi-upload-token",
        "PYPI_TOKEN=pypi-AgEIcHlwaS5vcmcA1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6",
    );
}

#[test]
fn test_rubygems_api_token() {
    let s = scanner();
    // rubygems_ + 48 hex
    assert_detects(
        &s,
        "rubygems-api-token",
        "GEM_KEY=rubygems_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4",
    );
}

#[test]
fn test_travisci_access_token() {
    let s = scanner();
    // 22 alnum in travis context
    assert_detects(
        &s,
        "travisci-access-token",
        "TRAVIS_TOKEN=a1b2c3d4e5f6a7b8c9d0e1",
    );
}

// ---------------------------------------------------------------------------
// Infrastructure / Secrets management
// ---------------------------------------------------------------------------

#[test]
fn test_vault_service_token() {
    let s = scanner();
    // hvs. + 90-120 word chars
    assert_detects(
        &s,
        "vault-service-token",
        "VAULT_TOKEN=hvs.k4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pXk4Jm8nR2pX",
    );
}

#[test]
fn test_vault_batch_token() {
    let s = scanner();
    assert_detects(
        &s,
        "vault-batch-token",
        "VAULT_TOKEN=hvb.AAAAAQJgxDgqsGNorpoOR7hPZ5SU-ynBvCl764jyRP_fnX7WvkdkDzGjbLNGdPdtlY33Als2P36yDZueqzfdGw9RsaTeaYXSH7E4RYSWuRoQ9YRKIw8o7mDDY2ZcT3KOB7RwtW1w1FN2eDqcy_sbCjXPaM1iBVH",
    );
}

// ---------------------------------------------------------------------------
// Miscellaneous services
// ---------------------------------------------------------------------------

#[test]
fn test_mapbox_api_token() {
    let s = scanner();
    // pk. + 60 alnum + . + 22 alnum in mapbox context
    assert_detects(
        &s,
        "mapbox-api-token",
        "MAPBOX_TOKEN=pk.a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0.a1b2c3d4e5f6a7b8c9d0e1",
    );
}

#[test]
fn test_netlify_access_token() {
    let s = scanner();
    // 40-46 alnum in netlify context
    assert_detects(
        &s,
        "netlify-access-token",
        "NETLIFY_TOKEN=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

#[test]
fn test_notion_api_token() {
    let s = scanner();
    // ntn_ + 11 digits + 35 alnum
    assert_detects(
        &s,
        "notion-api-token",
        "NOTION_KEY=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",
    );
}

#[test]
fn test_postman_api_token() {
    let s = scanner();
    // PMAK- + 24 hex - + 34 hex
    assert_detects(
        &s,
        "postman-api-token",
        "POSTMAN_KEY=PMAK-a1b2c3d4e5f6a7b8c9d0e1f2-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6ef",
    );
}

#[test]
fn test_rapidapi_access_token() {
    let s = scanner();
    // 50 alnum in rapidapi context
    assert_detects(
        &s,
        "rapidapi-access-token",
        "RAPIDAPI_KEY=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5",
    );
}

#[test]
fn test_shippo_api_token() {
    let s = scanner();
    // shippo_live_ + 40 hex
    assert_detects(
        &s,
        "shippo-api-token",
        "SHIPPO_TOKEN=shippo_live_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

#[test]
fn test_hubspot_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "hubspot-api-key",
        r#"const hubspotKey = "12345678-ABCD-ABCD-ABCD-1234567890AB""#,
    );
}

#[test]
fn test_typeform_api_token() {
    let s = scanner();
    // tfp_ + 59 alnum in typeform context
    assert_detects(
        &s,
        "typeform-api-token",
        "TYPEFORM_TOKEN=tfp_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0e1f2a3b4c5d6e7f8a9e",
    );
}

#[test]
fn test_twilio_api_key() {
    let s = scanner();
    // SK + 32 hex (uppercase SK, case-sensitive prefix)
    assert_detects(
        &s,
        "twilio-api-key",
        "TWILIO_API_KEY=SK1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d",
    );
}

#[test]
fn test_scalingo_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "scalingo-api-token",
        r#"scalingo_api_token = "tk-us-loys7ib9yrxcys_ta2sq85mjar6lgcsspkd9x61s7h5epf_-""#,
    );
}

#[test]
fn test_replicate_api_token() {
    let s = scanner();
    // r8_ prefix + exactly 37 alnum chars
    assert_detects(
        &s,
        "replicate-api-token",
        "REPLICATE_API_KEY=r8_WesXNvqsCpq7r1gpQABpB3NJvdR1234567890",
    );
}

#[test]
fn test_vercel_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "vercel-api-token",
        "VERCEL_TOKEN=DdZV6ZDZW6Vpl7n7JqtrCE5i",
    );
}

#[test]
fn test_vercel_personal_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "vercel-personal-access-token",
        "VERCEL_TOKEN=vcp_35UYJwYZDigYATKhxJUAhPqRhit2Xe3dtiG60LsUTHeklEXDQ94Jafpu",
    );
}

#[test]
fn test_greptile_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "greptile-api-key",
        r#"greptile_api_key = "Bc4UcqgG6mG5ARxNAOH7TV2C/tDWaB7Kpne/pockv3iQcbSN""#,
    );
}

#[test]
fn test_posthog_project_api_key() {
    let s = scanner();
    // phc_ prefix + exactly 43 alnum/dash/underscore chars
    assert_detects(
        &s,
        "posthog-project-api-key",
        "POSTHOG_API_KEY=phc_E123456789012345678901234567890123456789012",
    );
}

#[test]
fn test_posthog_personal_api_key() {
    let s = scanner();
    // phx_ prefix + exactly 47 alnum/dash/underscore chars
    assert_detects(
        &s,
        "posthog-personal-api-key",
        "POSTHOG_PERSONAL_KEY=phx_FNKCx83Ko0JQMuZH1zz94xgK798TCUybkf79ZKYKwKQ1234",
    );
}

#[test]
fn test_prefect_api_token() {
    let s = scanner();
    // pnu_ prefix + 36 chars
    assert_detects(
        &s,
        "prefect-api-token",
        "PREFECT_API_KEY=pnu_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8",
    );
}

#[test]
fn test_perplexity_api_key() {
    let s = scanner();
    // pplx- prefix + 48 alnum chars
    assert_detects(
        &s,
        "perplexity-api-key",
        "PERPLEXITY_API_KEY=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",
    );
}

#[test]
fn test_plaid_client_id() {
    let s = scanner();
    assert_detects(
        &s,
        "plaid-client-id",
        "PLAID_CLIENT_ID=a1b2c3d4e5f6a7b8c9d0e1f2",
    );
}

#[test]
fn test_plaid_secret_key() {
    let s = scanner();
    assert_detects(
        &s,
        "plaid-secret-key",
        "PLAID_SECRET=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5",
    );
}

#[test]
fn test_looker_client_id() {
    let s = scanner();
    assert_detects(
        &s,
        "looker-client-id",
        "LOOKER_CLIENT_ID=a1b2c3d4e5f6a7b8c9d0",
    );
}

#[test]
fn test_maxmind_license_key() {
    let s = scanner();
    assert_detects(
        &s,
        "maxmind-license-key",
        "MAXMIND_LICENSE_KEY=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",
    );
}

#[test]
fn test_figma_personal_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "figma-personal-access-token",
        "figma pat = figd_rh1234567890123456789012345678901234abcd",
    );
}

#[test]
fn test_fastly_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "fastly-api-token",
        "Fastly-token: fgsb3ef237afd6c1b9d91f81cdba64f3",
    );
}

// ---------------------------------------------------------------------------
// False positive checks
// ---------------------------------------------------------------------------

#[test]
fn test_no_false_positive_low_entropy_1password() {
    let s = scanner();
    // All-X key: entropy = 0, below threshold of 3.8
    assert_no_detect(
        &s,
        "1password-secret-key",
        "A3-XXXXXX-XXXXXXXXXXX-XXXXX-XXXXX-XXXXX",
    );
}

#[test]
fn test_no_false_positive_placeholder_stripe() {
    let s = scanner();
    assert_no_detect(&s, "stripe-access-token", "sk_test_YOUR_KEY_HERE");
}

// ---------------------------------------------------------------------------
// New tests for previously-untested rules
// ---------------------------------------------------------------------------

// adafruit-api-key: keyword context + 32 lowercase alnum/dash/underscore
#[test]
fn test_adafruit_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "adafruit-api-key",
        "adafruit_key = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// adobe-client-id: keyword context + 32 lowercase hex
#[test]
fn test_adobe_client_id() {
    let s = scanner();
    assert_detects(
        &s,
        "adobe-client-id",
        "adobe_client_id = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// adobe-client-secret: p8e- prefix + 32 alnum
#[test]
fn test_adobe_client_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "adobe-client-secret",
        "ADOBE_SECRET=p8e-a1b2C3d4E5f6A7b8C9d0E1f2A3b4C5d6",
    );
}

// airtable-api-key: keyword context + 17 lowercase alnum
#[test]
fn test_airtable_api_key() {
    let s = scanner();
    assert_detects(&s, "airtable-api-key", "airtable_key = a1b2c3d4e5f6a7b8c");
}

// alibaba-secret-key: keyword context + 30 lowercase alnum
#[test]
fn test_alibaba_secret_key() {
    let s = scanner();
    assert_detects(
        &s,
        "alibaba-secret-key",
        "alibaba_secret = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5",
    );
}

// anthropic-api-key: sk-ant-api03- + exactly 93 [a-zA-Z0-9_-] chars + AA at end
// Count: sk-ant-api03- (14) + 93 body chars + AA = must end with AA
#[test]
fn test_anthropic_api_key() {
    let s = scanner();
    // 93 alnum/dash/underscore chars then AA — carefully counted
    assert_detects(
        &s,
        "anthropic-api-key",
        "ANTHROPIC_KEY=sk-ant-api03-A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0BAA",
    );
}

// artifactory-api-key: AKCp + 69 alnum
#[test]
fn test_artifactory_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "artifactory-api-key",
        "ARTIFACTORY_KEY=AKCpA1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A7b8C9d4E5f6G7h8I",
    );
}

// artifactory-reference-token: cmVmd + 59 alnum
#[test]
fn test_artifactory_reference_token() {
    let s = scanner();
    assert_detects(
        &s,
        "artifactory-reference-token",
        "ARTIFACTORY_REF=cmVmdA1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A7b9C3d",
    );
}

// asana-client-id: keyword context + 16 digits
#[test]
fn test_asana_client_id() {
    let s = scanner();
    assert_detects(&s, "asana-client-id", "asana_client_id = 1234567890123456");
}

// asana-client-secret: keyword context + 32 lowercase alnum
#[test]
fn test_asana_client_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "asana-client-secret",
        "asana_secret = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// authress-service-client-access-key: sc_ prefix format
#[test]
fn test_authress_service_client_access_key() {
    let s = scanner();
    assert_detects(
        &s,
        "authress-service-client-access-key",
        "AUTHRESS_KEY=sc_abc12.def1.acc-ghij12345678.A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P",
    );
}

// aws-access-token: AKIA prefix + 16 uppercase alnum (A-Z2-7)
#[test]
fn test_aws_access_token() {
    let s = scanner();
    // Note: regex uses [A-Z2-7] for the body chars after the prefix
    assert_detects(
        &s,
        "aws-access-token",
        "AWS_ACCESS_KEY_ID=AKIAK4JM7NR2PX6SWT3B",
    );
}

// aws-amazon-bedrock-api-key-long-lived: ABSK + 109-269 base64 chars
#[test]
fn test_aws_amazon_bedrock_api_key_long_lived() {
    let s = scanner();
    assert_detects(
        &s,
        "aws-amazon-bedrock-api-key-long-lived",
        "BEDROCK_KEY=ABSKA1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8",
    );
}

// beamer-api-token: keyword context + b_ + 44 alnum/equals/dash/underscore
#[test]
fn test_beamer_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "beamer-api-token",
        "beamer_key = b_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d=",
    );
}

// bitbucket-client-secret: keyword context + 64 alnum/equals/dash/underscore
#[test]
fn test_bitbucket_client_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "bitbucket-client-secret",
        "bitbucket_secret = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

// bittrex-access-key: keyword context + 32 alnum
#[test]
fn test_bittrex_access_key() {
    let s = scanner();
    assert_detects(
        &s,
        "bittrex-access-key",
        "bittrex_key = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// bittrex-secret-key: identical regex to bittrex-access-key; the first matching rule wins.
// Both rules detect any bittrex context + 32 alnum, so we verify at least one fires.
#[test]
fn test_bittrex_secret_key() {
    let s = scanner();
    let findings = s.scan_text(
        "bittrex_secret = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
        "test.env",
        None,
    );
    assert!(
        findings
            .iter()
            .any(|f| f.rule_id == "bittrex-secret-key" || f.rule_id == "bittrex-access-key"),
        "expected bittrex-secret-key or bittrex-access-key; found: {:?}",
        findings
            .iter()
            .map(|f| f.rule_id.as_str())
            .collect::<Vec<_>>()
    );
}

// cisco-meraki-api-key: Meraki keyword context + 40 hex
#[test]
fn test_cisco_meraki_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "cisco-meraki-api-key",
        "Meraki_api_key = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

// clickhouse-cloud-api-secret-key: 4b1d + 38 alnum
#[test]
fn test_clickhouse_cloud_api_secret_key() {
    let s = scanner();
    assert_detects(
        &s,
        "clickhouse-cloud-api-secret-key",
        "CLICKHOUSE_KEY=4b1dA1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9",
    );
}

// clojars-api-token: CLOJARS_ + 60 alnum
#[test]
fn test_clojars_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "clojars-api-token",
        "CLOJARS_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A7b2C3d4",
    );
}

// cloudflare-global-api-key: keyword context + 37 hex
#[test]
fn test_cloudflare_global_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "cloudflare-global-api-key",
        "cloudflare_global_key = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a",
    );
}

// codecov-access-token: keyword context + 32 alnum
#[test]
fn test_codecov_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "codecov-access-token",
        "codecov_token = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// cohere-api-token: keyword context + 40 mixed-case alnum
#[test]
fn test_cohere_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "cohere-api-token",
        "cohere_key = A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0",
    );
}

// confluent-access-token: keyword context + 16 alnum
#[test]
fn test_confluent_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "confluent-access-token",
        "confluent_key = a1b2c3d4e5f6a7b8",
    );
}

// confluent-secret-key: keyword context + 64 alnum
#[test]
fn test_confluent_secret_key() {
    let s = scanner();
    assert_detects(
        &s,
        "confluent-secret-key",
        "confluent_secret = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

// contentful-delivery-api-token: keyword context + 43 alnum/equals/dash/underscore
#[test]
fn test_contentful_delivery_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "contentful-delivery-api-token",
        "contentful_token = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d",
    );
}

// curl-auth-header: curl command with Bearer token
#[test]
fn test_curl_auth_header() {
    let s = scanner();
    assert_detects(
        &s,
        "curl-auth-header",
        r#"curl -H "Authorization: Bearer A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6" https://api.example.com"#,
    );
}

// curl-auth-user: curl command with user:password
#[test]
fn test_curl_auth_user() {
    let s = scanner();
    assert_detects(
        &s,
        "curl-auth-user",
        r#"curl -u "myuser:S3cr3tP@ssw0rd1234" https://api.example.com"#,
    );
}

// deepgram-api-key: keyword context + 40 hex
#[test]
fn test_deepgram_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "deepgram-api-key",
        "deepgram_key = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

// defined-networking-api-token: dnkey context + dnkey-{26}-{52}
#[test]
fn test_defined_networking_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "defined-networking-api-token",
        "dnkey = dnkey-a1b2c3d4e5f6a7b8c9d0e1f2a5-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6",
    );
}

// digitalocean-refresh-token: dor_v1_ + 64 hex
#[test]
fn test_digitalocean_refresh_token() {
    let s = scanner();
    assert_detects(
        &s,
        "digitalocean-refresh-token",
        "DO_REFRESH=dor_v1_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

// discord-client-id: keyword context + 18 digits
#[test]
fn test_discord_client_id() {
    let s = scanner();
    assert_detects(
        &s,
        "discord-client-id",
        "discord_client_id = 123456789012345678",
    );
}

// discord-client-secret: keyword context + 32 alnum/equals/dash/underscore
#[test]
fn test_discord_client_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "discord-client-secret",
        "discord_secret = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// droneci-access-token: keyword context + 32 alnum
#[test]
fn test_droneci_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "droneci-access-token",
        "droneci_token = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// dropbox-long-lived-api-token: dropbox context + 11 alnum + AAAAAAAAAA + 43 alnum/dash/underscore/equals
#[test]
fn test_dropbox_long_lived_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "dropbox-long-lived-api-token",
        "dropbox_key = a1b2c3d4e5fAAAAAAAAAAa1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1=",
    );
}

// dropbox-short-lived-api-token: dropbox context + sl. + 135 alnum/dash/equals/underscore
#[test]
fn test_dropbox_short_lived_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "dropbox-short-lived-api-token",
        "dropbox_key = sl.a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9a1b2c3d4e5f6a7b8c9d",
    );
}

// duffel-api-token: duffel_test_ or duffel_live_ + 43 chars
#[test]
fn test_duffel_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "duffel-api-token",
        "DUFFEL_KEY=duffel_test_a1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v",
    );
}

// easypost-api-token: EZAK + 54 alnum
#[test]
fn test_easypost_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "easypost-api-token",
        "EASYPOST_KEY=EZAKa1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7",
    );
}

// easypost-test-api-token: EZTK + 54 alnum
#[test]
fn test_easypost_test_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "easypost-test-api-token",
        "EASYPOST_TEST_KEY=EZTKa1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7",
    );
}

// elevenlabs-api-key: keyword context + sk_ + 48 hex
#[test]
fn test_elevenlabs_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "elevenlabs-api-key",
        "elevenlabs_key = sk_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4",
    );
}

// endorlabs-api-key: keyword context + endr+ + 16 alnum/dash
#[test]
fn test_endorlabs_api_key() {
    let s = scanner();
    assert_detects(&s, "endorlabs-api-key", "endor_key = endr+A1b2C3d4E5f6G7h8");
}

// endorlabs-api-secret: keyword context + endr+ + 16 alnum/dash
#[test]
fn test_endorlabs_api_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "endorlabs-api-secret",
        "api_secret = endr+A1b2C3d4E5f6G7h8",
    );
}

// etsy-access-token: keyword context + 24 alnum
#[test]
fn test_etsy_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "etsy-access-token",
        "ETSY_KEY = a1b2c3d4e5f6a7b8c9d0e1f2",
    );
}

// facebook-access-token: 15-16 digits + | + 27-40 alnum/dash/underscore
#[test]
fn test_facebook_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "facebook-access-token",
        "facebook_token = 123456789012345|A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p",
    );
}

// facebook-page-access-token: EAAM or EAAC + 100+ alnum
#[test]
fn test_facebook_page_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "facebook-page-access-token",
        "fb_page_token = EAAMa1B2c3D4e5F6g7H8i9J0k1L2m3N4o5P6q7R8s9T0u1V2w3X4y5Z6a1B2c3D4e5F6g7H8i9J0k1L2m3N4o5P6q7R8s9T0u1V2w3X4y5Z6",
    );
}

// figma-personal-access-header-token: x-figma-token context + UUID format with uppercase hex
#[test]
fn test_figma_personal_access_header_token() {
    let s = scanner();
    assert_detects(
        &s,
        "figma-personal-access-header-token",
        "x-figma-token = 1A2B-3C4D5E6F-7A8B-9C0D-1E2F-3A4B5C6D7E8F",
    );
}

// finicity-api-token: keyword context + 32 hex
#[test]
fn test_finicity_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "finicity-api-token",
        "finicity_token = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// finicity-client-secret: keyword context + 20 alnum
#[test]
fn test_finicity_client_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "finicity-client-secret",
        "finicity_secret = a1b2c3d4e5f6a7b8c9d0",
    );
}

// finnhub-access-token: keyword context + 20 alnum
#[test]
fn test_finnhub_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "finnhub-access-token",
        "finnhub_token = a1b2c3d4e5f6a7b8c9d0",
    );
}

// flickr-access-token: keyword context + 32 alnum
#[test]
fn test_flickr_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "flickr-access-token",
        "flickr_token = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// flutterwave-encryption-key: FLWSECK_TEST- + 12 alnum from [a-h0-9]
#[test]
fn test_flutterwave_encryption_key() {
    let s = scanner();
    assert_detects(
        &s,
        "flutterwave-encryption-key",
        "FLWSECK_TEST-a1b2c3d4e5f6",
    );
}

// flutterwave-public-key: FLWPUBK_TEST- + 32 alnum from [a-h0-9] + -X
#[test]
fn test_flutterwave_public_key() {
    let s = scanner();
    assert_detects(
        &s,
        "flutterwave-public-key",
        "FLWPUBK_TEST-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6-X",
    );
}

// flutterwave-secret-key: FLWSECK_TEST- + 32 alnum from [a-h0-9] + -X
#[test]
fn test_flutterwave_secret_key() {
    let s = scanner();
    assert_detects(
        &s,
        "flutterwave-secret-key",
        "FLWSECK_TEST-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6-X",
    );
}

// frameio-api-token: fio-u- + 64 alnum/dash/equals/underscore
#[test]
fn test_frameio_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "frameio-api-token",
        "FRAMEIO_KEY=fio-u-A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A7b2C3d4E5f6G7h8",
    );
}

// freemius-secret-key: PHP array syntax with sk_ prefix; uses php file path
// Path-gated rule (path = *.php). Test against "test.php" filename.
#[test]
fn test_freemius_secret_key() {
    let s = scanner();
    let input = r#""secret_key" => "sk_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O""#;
    let findings = s.scan_text(input, "config.php", None);
    assert!(
        findings.iter().any(|f| f.rule_id == "freemius-secret-key"),
        "rule 'freemius-secret-key' not detected.\ninput: {:?}\nfound: {:?}",
        &input[..input.len().min(120)],
        findings
            .iter()
            .map(|f| f.rule_id.as_str())
            .collect::<Vec<_>>()
    );
}

// freshbooks-access-token: keyword context + 64 alnum
#[test]
fn test_freshbooks_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "freshbooks-access-token",
        "freshbooks_token = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

// gcp-api-key: AIza + 35 word/dash chars
#[test]
fn test_gcp_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "gcp-api-key",
        "GCP_API_KEY=AIzaA1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r",
    );
}

// generic-api-key: generic keyword + high-entropy value
#[test]
fn test_generic_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "generic-api-key",
        "api_key = X9kM2pR7vB4wZ1nQ8cF5tL0sY3hD6jA",
    );
}

// gitea-access-token: gitea_token keyword context + 40 hex
#[test]
fn test_gitea_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "gitea-access-token",
        "gitea_token = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

// github-app-token: ghu_ or ghs_ + 36 alnum
#[test]
fn test_github_app_token() {
    let s = scanner();
    assert_detects(
        &s,
        "github-app-token",
        "GITHUB_TOKEN=ghu_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2AB",
    );
}

// github-pat: ghp_ + 36 alnum
#[test]
fn test_github_pat() {
    let s = scanner();
    assert_detects(
        &s,
        "github-pat",
        "GITHUB_TOKEN=ghp_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2AB",
    );
}

// github-refresh-token: ghr_ + 36 alnum
#[test]
fn test_github_refresh_token() {
    let s = scanner();
    assert_detects(
        &s,
        "github-refresh-token",
        "GITHUB_REFRESH=ghr_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2AB",
    );
}

// gitlab-cicd-job-token: glcbt- + 1-5 alnum + _ + 20 alnum/dash/underscore
#[test]
fn test_gitlab_cicd_job_token() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-cicd-job-token",
        "CI_JOB_TOKEN=glcbt-abc12_k4Jm8nR2pX6sW9vB3fH7a",
    );
}

// gitlab-deploy-token: gldt- + 20 alnum/dash/underscore
#[test]
fn test_gitlab_deploy_token() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-deploy-token",
        "GITLAB_DEPLOY=gldt-k4Jm8nR2pX6sW9vB3fH7a",
    );
}

// gitlab-feature-flag-client-token: glffct- + 20 alnum/dash/underscore
#[test]
fn test_gitlab_feature_flag_client_token() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-feature-flag-client-token",
        "FEATURE_FLAG_TOKEN=glffct-k4Jm8nR2pX6sW9vB3fH7a",
    );
}

// gitlab-feed-token: glft- + 20 alnum/dash/underscore
#[test]
fn test_gitlab_feed_token() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-feed-token",
        "FEED_TOKEN=glft-k4Jm8nR2pX6sW9vB3fH7aT",
    );
}

// gitlab-incoming-mail-token: glimt- + 25 alnum/dash/underscore
#[test]
fn test_gitlab_incoming_mail_token() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-incoming-mail-token",
        "MAIL_TOKEN=glimt-k4Jm8nR2pX6sW9vB3fH7aT1qY",
    );
}

// gitlab-kubernetes-agent-token: glagent- + 50 alnum/dash/underscore
#[test]
fn test_gitlab_kubernetes_agent_token() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-kubernetes-agent-token",
        "AGENT_TOKEN=glagent-k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2ABpQ3rX9m2vJ51",
    );
}

// gitlab-oauth-app-secret: gloas- + 64 alnum/dash/underscore
#[test]
fn test_gitlab_oauth_app_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-oauth-app-secret",
        "OAUTH_SECRET=gloas-k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2ABpQ3rX9mZvJ5k4Jm8nR2pX6sW9v",
    );
}

// gitlab-pat-routable: glpat- + 27-300 alnum/dash/underscore + . + 9 alnum
#[test]
fn test_gitlab_pat_routable() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-pat-routable",
        "GITLAB_TOKEN=glpat-k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD.ab1234567",
    );
}

// gitlab-ptt: glptt- + 40 hex
#[test]
fn test_gitlab_ptt() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-ptt",
        "GITLAB_TRIGGER=glptt-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

// gitlab-rrt: GR1348941 + 20 alnum/dash/underscore
#[test]
fn test_gitlab_rrt() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-rrt",
        "RUNNER_TOKEN=GR1348941k4Jm8nR2pX6sW9vB3fH7a",
    );
}

// gitlab-runner-authentication-token-routable: glrt-t{digit}_ + 27+ alnum/dash/underscore + . + 9 alnum
#[test]
fn test_gitlab_runner_authentication_token_routable() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-runner-authentication-token-routable",
        "RUNNER_TOKEN=glrt-t1_k4Jm8nR2pX6sW9vB3fH7aT1qY5uE0cD8gLw2AB.ab1234567",
    );
}

// gitlab-scim-token: glsoat- + 20 alnum/dash/underscore
#[test]
fn test_gitlab_scim_token() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-scim-token",
        "SCIM_TOKEN=glsoat-k4Jm8nR2pX6sW9vB3fH7a",
    );
}

// gitlab-session-cookie: _gitlab_session= + 32 lowercase alnum
#[test]
fn test_gitlab_session_cookie() {
    let s = scanner();
    assert_detects(
        &s,
        "gitlab-session-cookie",
        "_gitlab_session=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// gitter-access-token: keyword context + 40 alnum/dash/underscore
#[test]
fn test_gitter_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "gitter-access-token",
        "gitter_token = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

// gocardless-api-token: gocardless keyword context + live_ + 40 alnum/dash/equals/underscore
#[test]
fn test_gocardless_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "gocardless-api-token",
        "gocardless_token = live_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

// grafana-api-key: eyJrIjoi + 70+ base64 chars
#[test]
fn test_grafana_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "grafana-api-key",
        "GRAFANA_KEY=eyJrIjoiA1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A7b2C3d4E5f6G7h8I9j0K1l2M3n4O5p",
    );
}

// groq-api-key: gsk_ + 52 uppercase alnum
#[test]
fn test_groq_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "groq-api-key",
        "GROQ_KEY=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",
    );
}

// hashicorp-tf-password: password context + quoted 8-20 alnum/equals/dash/underscore (path=*.tf)
#[test]
fn test_hashicorp_tf_password() {
    let s = scanner();
    let input = r#"administrator_login_password = "S3cr3t-P4ss1""#;
    let findings = s.scan_text(input, "main.tf", None);
    assert!(
        findings
            .iter()
            .any(|f| f.rule_id == "hashicorp-tf-password"),
        "rule 'hashicorp-tf-password' not detected.\ninput: {:?}\nfound: {:?}",
        &input[..input.len().min(120)],
        findings
            .iter()
            .map(|f| f.rule_id.as_str())
            .collect::<Vec<_>>()
    );
}

// huggingface-access-token: hf_ + 34 lowercase alpha
#[test]
fn test_huggingface_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "huggingface-access-token",
        "HF_TOKEN=hf_mnopqrstuvwxyzabcdefghijklmnopqrst",
    );
}

// huggingface-organization-api-token: api_org_ + 34 lowercase alpha
#[test]
fn test_huggingface_organization_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "huggingface-organization-api-token",
        "HF_ORG_TOKEN=api_org_mnopqrstuvwxyzabcdefghijklmnopqrst",
    );
}

// infracost-api-token: ico- + 32 alnum
#[test]
fn test_infracost_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "infracost-api-token",
        "INFRACOST_KEY=ico-A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6",
    );
}

// intra42-client-secret: s-s4t2ud- or s-s4t2af- + 64 hex
#[test]
fn test_intra42_client_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "intra42-client-secret",
        "INTRA_SECRET=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",
    );
}

// jfrog-identity-token: jfrog keyword context + 64 alnum
#[test]
fn test_jfrog_identity_token() {
    let s = scanner();
    assert_detects(
        &s,
        "jfrog-identity-token",
        "jfrog_token = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

// jwt: eyJ...eyJ...signature format
#[test]
fn test_jwt() {
    let s = scanner();
    assert_detects(
        &s,
        "jwt",
        "TOKEN=eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyMTIzNDU2Nzg5MCJ9.A1b2C3d4E5f6G7h8I9j0K1l2M3n4",
    );
}

// jwt-base64: ZXlK base64-encoded JWT prefix
#[test]
fn test_jwt_base64() {
    let s = scanner();
    // ZXlK is base64 for "eyJ"; followed by one of the known field markers (e.g. aGJHY2lPaU = "hbGci")
    assert_detects(
        &s,
        "jwt-base64",
        "TOKEN=ZXlKaGJHY2lPaUpIVXpJMU5pSjkuZXlKemRXSWlPaUoxYzJWeU1USXpORFUyTnpnNU1DSjkuQTFiMkMzZDRFNWY2RzdoOEk5ajBLMWwyTTNuNA==",
    );
}

// kraken-access-token: keyword context + 80-90 alnum/slash/equals/plus/dash/underscore
#[test]
fn test_kraken_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "kraken-access-token",
        "kraken_key = A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A7b2C3d4E5f6G7h8I9j0K1l2M3n4",
    );
}

// kubernetes-secret-yaml: kind: Secret + data: + base64 value (yaml path)
#[test]
fn test_kubernetes_secret_yaml() {
    let s = scanner();
    let input = "kind: Secret\ndata:\n  password: SGVsbG9Xb3JsZDEyMzQ=\n";
    let findings = s.scan_text(input, "secret.yaml", None);
    assert!(
        findings
            .iter()
            .any(|f| f.rule_id == "kubernetes-secret-yaml"),
        "rule 'kubernetes-secret-yaml' not detected.\ninput: {:?}\nfound: {:?}",
        &input[..input.len().min(120)],
        findings
            .iter()
            .map(|f| f.rule_id.as_str())
            .collect::<Vec<_>>()
    );
}

// kucoin-access-token: keyword context + 24 hex
#[test]
fn test_kucoin_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "kucoin-access-token",
        "kucoin_key = a1b2c3d4e5f6a7b8c9d0e1f2",
    );
}

// kucoin-secret-key: keyword context + UUID format
#[test]
fn test_kucoin_secret_key() {
    let s = scanner();
    assert_detects(
        &s,
        "kucoin-secret-key",
        "kucoin_secret = a1b2c3d4-e5f6-a7b8-c9d0-e1f2a3b4c5d6",
    );
}

// linear-api-key: lin_api_ + 40 alnum
#[test]
fn test_linear_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "linear-api-key",
        "LINEAR_KEY=lin_api_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0",
    );
}

// linear-client-secret: keyword context + 32 hex
#[test]
fn test_linear_client_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "linear-client-secret",
        "linear_secret = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// linkedin-client-id: keyword context + 14 alnum
#[test]
fn test_linkedin_client_id() {
    let s = scanner();
    assert_detects(
        &s,
        "linkedin-client-id",
        "linkedin_client_id = a1b2c3d4e5f6a7",
    );
}

// linkedin-client-secret: keyword context + 16 alnum
#[test]
fn test_linkedin_client_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "linkedin-client-secret",
        "linkedin_secret = a1b2c3d4e5f6a7b8",
    );
}

// lob-pub-api-key: lob keyword context + test_pub_ or live_pub_ + 31 hex
#[test]
fn test_lob_pub_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "lob-pub-api-key",
        "lob_pub = test_pub_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d",
    );
}

// looker-client-secret: keyword context + 24 alnum
#[test]
fn test_looker_client_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "looker-client-secret",
        "looker_secret = a1b2c3d4e5f6a7b8c9d0e1f2",
    );
}

// mailgun-pub-key: mailgun keyword context + pubkey- + 32 hex
#[test]
fn test_mailgun_pub_key() {
    let s = scanner();
    assert_detects(
        &s,
        "mailgun-pub-key",
        "mailgun_pub = pubkey-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// mailgun-signing-key: mailgun keyword context + 32 [a-h0-9] - 8 [a-h0-9] - 8 [a-h0-9]
#[test]
fn test_mailgun_signing_key() {
    let s = scanner();
    assert_detects(
        &s,
        "mailgun-signing-key",
        "mailgun_signing = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6-a1b2c3d4-e5f6a7b8",
    );
}

// messagebird-api-token: messagebird keyword context + 25 alnum
#[test]
fn test_messagebird_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "messagebird-api-token",
        "messagebird_token = a1b2c3d4e5f6a7b8c9d0e1f2a",
    );
}

// messagebird-client-id: messagebird keyword context + UUID
#[test]
fn test_messagebird_client_id() {
    let s = scanner();
    assert_detects(
        &s,
        "messagebird-client-id",
        "messagebird_client = a1b2c3d4-e5f6-a7b8-c9d0-e1f2a3b4c5d6",
    );
}

// new-relic-browser-api-token: new-relic keyword context + NRJS- + 19 hex
#[test]
fn test_new_relic_browser_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "new-relic-browser-api-token",
        "new-relic_key = NRJS-a1b2c3d4e5f6a7b8c9d",
    );
}

// new-relic-user-api-id: new-relic keyword context + 64 alnum
#[test]
fn test_new_relic_user_api_id() {
    let s = scanner();
    assert_detects(
        &s,
        "new-relic-user-api-id",
        "new-relic_user_id = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

// new-relic-user-api-key: new-relic keyword context + NRAK- + 27 alnum
#[test]
fn test_new_relic_user_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "new-relic-user-api-key",
        "new-relic_key = NRAK-a1b2c3d4e5f6a7b8c9d0e1f2a3b",
    );
}

// npm-access-token: npm_ + 36 alnum
#[test]
fn test_npm_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "npm-access-token",
        "NPM_TOKEN=npm_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8",
    );
}

// nuget-config-password: <add key="ClearTextPassword" value="..." /> in nuget.config
#[test]
fn test_nuget_config_password() {
    let s = scanner();
    let input = r#"<add key="ClearTextPassword" value="S3cretP4ss1234" />"#;
    let findings = s.scan_text(input, "nuget.config", None);
    assert!(
        findings
            .iter()
            .any(|f| f.rule_id == "nuget-config-password"),
        "rule 'nuget-config-password' not detected.\ninput: {:?}\nfound: {:?}",
        &input[..input.len().min(120)],
        findings
            .iter()
            .map(|f| f.rule_id.as_str())
            .collect::<Vec<_>>()
    );
}

// nytimes-access-token: nytimes keyword context + 32 alnum/equals/dash/underscore
#[test]
fn test_nytimes_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "nytimes-access-token",
        "nytimes_key = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// octopus-deploy-api-key: API- + 26 uppercase alnum
#[test]
fn test_octopus_deploy_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "octopus-deploy-api-key",
        "OCTOPUS_KEY=API-A1B2C3D4E5F6G7H8I9J0K1L2M3",
    );
}

// openai-api-key: sk- + 20 alnum + T3BlbkFJ + 20 alnum
#[test]
fn test_openai_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "openai-api-key",
        "OPENAI_KEY=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",
    );
}

// openrouter-api-key: sk-or-v1- + 64 hex
#[test]
fn test_openrouter_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "openrouter-api-key",
        "OPENROUTER_KEY=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",
    );
}

// pkcs12-file: path-only rule — no regex, just file path matching *.p12 or *.pfx
// This rule has no regex field; it fires on the file path, not the content.
// We verify it does NOT produce false positives on regular text content.
#[test]
fn test_pkcs12_file_no_false_positive() {
    let s = scanner();
    assert_no_detect(&s, "pkcs12-file", "some random content without a p12 token");
}

// plaid-api-token: plaid keyword context + access-{env}-UUID
#[test]
fn test_plaid_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "plaid-api-token",
        "plaid_key = access-sandbox-a1b2c3d4-e5f6-a7b8-c9d0-e1f2a3b4c5d6",
    );
}

// planetscale-api-token: pscale_tkn_ + 32-64 alnum/equals/dot/dash
#[test]
fn test_planetscale_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "planetscale-api-token",
        "PSCALE_TOKEN=pscale_tkn_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8",
    );
}

// planetscale-id: skipReport=true — just verify it fires on appropriate input
// (skip_report means it won't appear in scan_text which filters skipReport)
// Instead verify it doesn't cause issues by checking non-detection is safe.
#[test]
fn test_planetscale_id_skipped() {
    // planetscale-id has skipReport = true, so it's used as a capture-only rule.
    // It will not appear in scanner findings. Just validate scanner doesn't panic.
    let s = scanner();
    let _ = s.scan_text(
        "PSCALE_USER=abc123def456 pscale_token_xyz",
        "test.env",
        None,
    );
}

// planetscale-oauth-token: pscale_oauth_ + 32-64 alnum/equals/dot/dash
#[test]
fn test_planetscale_oauth_token() {
    let s = scanner();
    assert_detects(
        &s,
        "planetscale-oauth-token",
        "PSCALE_OAUTH=pscale_oauth_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8",
    );
}

// planetscale-password: pscale_pw_ + 32-64 alnum/equals/dot/dash
#[test]
fn test_planetscale_password() {
    let s = scanner();
    assert_detects(
        &s,
        "planetscale-password",
        "PSCALE_PW=pscale_pw_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8",
    );
}

// polymarket-address: skipReport=true — verify no panic
#[test]
fn test_polymarket_address_skipped() {
    let s = scanner();
    let _ = s.scan_text(
        "poly_address = 0xA1b2C3d4E5f6a7b8C9d0e1F2a3B4c5D6e7F8a9b0",
        "test.env",
        None,
    );
}

// polymarket-api-key: poly_key context + UUID
#[test]
fn test_polymarket_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "polymarket-api-key",
        "poly_api_key = a1b2c3d4-e5f6-a7b8-c9d0-e1f2a3b4c5d6",
    );
}

// polymarket-api-secret: skipReport=true — verify no panic
#[test]
fn test_polymarket_api_secret_skipped() {
    let s = scanner();
    let _ = s.scan_text(
        "poly_api_secret = A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0==",
        "test.env",
        None,
    );
}

// polymarket-passphrase: skipReport=true — verify no panic
#[test]
fn test_polymarket_passphrase_skipped() {
    let s = scanner();
    let _ = s.scan_text("poly_passphrase = MySecretPhrase123", "test.env", None);
}

// polymarket-private-key: poly_private_key context + 0x + 64 hex
#[test]
fn test_polymarket_private_key() {
    let s = scanner();
    assert_detects(
        &s,
        "polymarket-private-key",
        "poly_private_key = 0xA1b2C3d4E5f6a7b8C9d0e1F2a3B4c5D6e7F8a9b0C1d2E3f4a5B6c7D8e9F0a1b2",
    );
}

// privateai-api-token: private_ai keyword context + 32 alnum
#[test]
fn test_privateai_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "privateai-api-token",
        "private_ai_key = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
    );
}

// private-key: PEM private key block
#[test]
fn test_private_key() {
    let s = scanner();
    assert_detects(
        &s,
        "private-key",
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA0Z3VS5JJcds3xHn/ygWep4PAtQkMtLmMxCPbGnABwsWwmLQu\n-----END RSA PRIVATE KEY-----",
    );
}

// pulumi-api-token: pul- + 40 hex
#[test]
fn test_pulumi_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "pulumi-api-token",
        "PULUMI_TOKEN=pul-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

// readme-api-token: rdme_ + 70 lowercase alnum
#[test]
fn test_readme_api_token() {
    let s = scanner();
    assert_detects(
        &s,
        "readme-api-token",
        "README_KEY=rdme_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5",
    );
}

// sendbird-access-id: sendbird keyword context + UUID
#[test]
fn test_sendbird_access_id() {
    let s = scanner();
    assert_detects(
        &s,
        "sendbird-access-id",
        "sendbird_id = a1b2c3d4-e5f6-a7b8-c9d0-e1f2a3b4c5d6",
    );
}

// sendbird-access-token: sendbird keyword context + 40 hex
#[test]
fn test_sendbird_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "sendbird-access-token",
        "sendbird_token = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}

// settlemint-application-access-token: sm_aat_ + 16 alnum
#[test]
fn test_settlemint_application_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "settlemint-application-access-token",
        "SETTLEMINT_TOKEN=sm_aat_A1b2C3d4E5f6G7h8",
    );
}

// settlemint-personal-access-token: sm_pat_ + 16 alnum
#[test]
fn test_settlemint_personal_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "settlemint-personal-access-token",
        "SETTLEMINT_PAT=sm_pat_A1b2C3d4E5f6G7h8",
    );
}

// settlemint-service-access-token: sm_sat_ + 16 alnum
#[test]
fn test_settlemint_service_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "settlemint-service-access-token",
        "SETTLEMINT_SAT=sm_sat_A1b2C3d4E5f6G7h8",
    );
}

// shopify-access-token: shpat_ + 32 hex
#[test]
fn test_shopify_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "shopify-access-token",
        "SHOPIFY_TOKEN=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",
    );
}

// shopify-custom-access-token: shpca_ + 32 hex
#[test]
fn test_shopify_custom_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "shopify-custom-access-token",
        "SHOPIFY_CUSTOM=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",
    );
}

// shopify-private-app-access-token: shppa_ + 32 hex
#[test]
fn test_shopify_private_app_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "shopify-private-app-access-token",
        "SHOPIFY_PRIVATE=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",
    );
}

// shopify-shared-secret: shpss_ + 32 hex
#[test]
fn test_shopify_shared_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "shopify-shared-secret",
        "SHOPIFY_SECRET=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",
    );
}

// sidekiq-secret: BUNDLE_ENTERPRISE__CONTRIBSYS__COM context + 8hex:8hex
#[test]
fn test_sidekiq_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "sidekiq-secret",
        "BUNDLE_ENTERPRISE__CONTRIBSYS__COM=a1b2c3d4:e5f6a7b8",
    );
}

// sidekiq-sensitive-url: https with 8hex:8hex credentials at contribsys.com
#[test]
fn test_sidekiq_sensitive_url() {
    let s = scanner();
    assert_detects(
        &s,
        "sidekiq-sensitive-url",
        "SIDEKIQ_URL=https://a1b2c3d4:e5f6a7b8@gems.contribsys.com/",
    );
}

// slack-config-refresh-token: xoxe- + 1 digit + - + 146 uppercase alnum
#[test]
fn test_slack_config_refresh_token() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-config-refresh-token",
        "SLACK_REFRESH=xoxe-1-A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1",
    );
}

// slack-legacy-bot-token: xoxb- + 8-14 digits + - + 18-26 alnum
#[test]
fn test_slack_legacy_bot_token() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-legacy-bot-token",
        "SLACK_TOKEN=xoxb-12345678-A1b2C3d4E5f6G7h8I9j0K1l2",
    );
}

// slack-legacy-workspace-token: xoxa- or xoxr- + 8-48 alnum
#[test]
fn test_slack_legacy_workspace_token() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-legacy-workspace-token",
        "SLACK_WORKSPACE=xoxa-A1b2C3d4E5f6G7h8",
    );
}

// slack-session-cookie: xoxd- + 100+ alnum/slash/plus/equals/dash/underscore
#[test]
fn test_slack_session_cookie() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-session-cookie",
        "SLACK_COOKIE=xoxd-A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4",
    );
}

// slack-session-token: xoxc- + 3 groups of 9-15 digits + - + 64 hex
#[test]
fn test_slack_session_token() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-session-token",
        "SLACK_SESSION=xoxc-123456789-987654321-112233445-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    );
}

// slack-webhook-url: hooks.slack.com/services or /workflows or /triggers + 43-56 alnum
#[test]
fn test_slack_webhook_url() {
    let s = scanner();
    assert_detects(
        &s,
        "slack-webhook-url",
        "SLACK_WEBHOOK=https://hooks.slack.com/services/A1B2C3D4E/B1C2D3E4F/a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2",
    );
}

// squarespace-access-token: squarespace keyword context + UUID
#[test]
fn test_squarespace_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "squarespace-access-token",
        "squarespace_token = a1b2c3d4-e5f6-a7b8-c9d0-e1f2a3b4c5d6",
    );
}

// stability-ai-api-key: stability keyword context + sk- + 48 mixed-case alnum
#[test]
fn test_stability_ai_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "stability-ai-api-key",
        "stability_key = sk-A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4",
    );
}

// twitter-access-secret: twitter keyword context + 45 alnum
#[test]
fn test_twitter_access_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "twitter-access-secret",
        "twitter_access_secret = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e",
    );
}

// twitter-access-token: twitter keyword context + 15-25 digits + - + 20-40 alnum
#[test]
fn test_twitter_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "twitter-access-token",
        "twitter_access_token = 123456789012345-A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p",
    );
}

// twitter-api-secret: twitter keyword context + 50 alnum
#[test]
fn test_twitter_api_secret() {
    let s = scanner();
    assert_detects(
        &s,
        "twitter-api-secret",
        "twitter_api_secret = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5",
    );
}

// twitter-bearer-token: twitter keyword context + 22 A chars + 80-100 alnum/percent
#[test]
fn test_twitter_bearer_token() {
    let s = scanner();
    assert_detects(
        &s,
        "twitter-bearer-token",
        "twitter_bearer = AAAAAAAAAAAAAAAAAAAAAAA1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A1b2C3d4E5f6G7h8I9j0K1l2M3n4",
    );
}

// vercel-ai-gateway-key: vck_ + 56 mixed-case alnum/dash/underscore
#[test]
fn test_vercel_ai_gateway_key() {
    let s = scanner();
    assert_detects(
        &s,
        "vercel-ai-gateway-key",
        "VERCEL_AI_KEY=vck_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A7b2",
    );
}

// vercel-app-access-token: vca_ + 56 mixed-case alnum/dash/underscore
#[test]
fn test_vercel_app_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "vercel-app-access-token",
        "VERCEL_APP_TOKEN=vca_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A7b2",
    );
}

// vercel-app-refresh-token: vcr_ + 56 mixed-case alnum/dash/underscore
#[test]
fn test_vercel_app_refresh_token() {
    let s = scanner();
    assert_detects(
        &s,
        "vercel-app-refresh-token",
        "VERCEL_REFRESH=vcr_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A7b2",
    );
}

// vercel-integration-token: vci_ + 56 mixed-case alnum/dash/underscore
#[test]
fn test_vercel_integration_token() {
    let s = scanner();
    assert_detects(
        &s,
        "vercel-integration-token",
        "VERCEL_INT_TOKEN=vci_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A7b2",
    );
}

// weights-and-biases-api-key-v1: wandb_v1_ + 77 mixed-case alnum/underscore
#[test]
fn test_weights_and_biases_api_key_v1() {
    let s = scanner();
    assert_detects(
        &s,
        "weights-and-biases-api-key-v1",
        "WANDB_KEY=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",
    );
}

// yandex-access-token: yandex keyword context + t1. + group1 + . + 86 base64url chars
#[test]
fn test_yandex_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "yandex-access-token",
        "yandex_token = t1.A1B2C3D4.A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7",
    );
}

// yandex-api-key: yandex keyword context + AQVN + 35-38 mixed-case alnum/dash/underscore
#[test]
fn test_yandex_api_key() {
    let s = scanner();
    assert_detects(
        &s,
        "yandex-api-key",
        "yandex_api_key = AQVNA1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r",
    );
}

// yandex-aws-access-token: yandex keyword context + YC + 38 mixed-case alnum/dash/underscore
#[test]
fn test_yandex_aws_access_token() {
    let s = scanner();
    assert_detects(
        &s,
        "yandex-aws-access-token",
        "yandex_aws = YCA1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9",
    );
}

// zendesk-secret-key: zendesk keyword context + 40 alnum
#[test]
fn test_zendesk_secret_key() {
    let s = scanner();
    assert_detects(
        &s,
        "zendesk-secret-key",
        "zendesk_key = a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
    );
}
