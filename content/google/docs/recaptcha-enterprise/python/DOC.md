---
name: recaptcha-enterprise
description: "Google Cloud reCAPTCHA Enterprise Python client for assessments, key management, and metrics"
metadata:
  languages: "python"
  versions: "1.30.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,recaptcha-enterprise,security,fraud-detection"
---

# Google Cloud reCAPTCHA Enterprise Python Client

## Golden Rule

Use `google-cloud-recaptcha-enterprise` for Python integrations with reCAPTCHA Enterprise, create assessments on your backend, and prefer the `v1` API surface over older `v1beta1` examples.

The package version validated for this doc is `1.30.0` from PyPI. The official import path is:

```python
from google.cloud import recaptchaenterprise_v1
```

## Install

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "google-cloud-recaptcha-enterprise==1.30.0"
```

Common alternatives:

```bash
uv add "google-cloud-recaptcha-enterprise==1.30.0"
poetry add "google-cloud-recaptcha-enterprise==1.30.0"
```

## Required Setup

Before the client works, Google’s product docs say you must:

1. Select or create a Google Cloud project.
2. Enable billing.
3. Enable the reCAPTCHA Enterprise API.
4. Set up authentication.

For website assessments, the product docs also call out the `reCAPTCHA Enterprise Agent` role (`roles/recaptchaenterprise.agent`) and recommend creating assessments only on your backend.

## Authentication

### Preferred: Application Default Credentials

On local development machines, use ADC:

```bash
gcloud auth application-default login
```

At runtime, the library follows normal ADC lookup order. The most common inputs are:

- `GOOGLE_APPLICATION_CREDENTIALS=/path/to/credentials.json`
- Local ADC created by `gcloud auth application-default login`
- An attached service account when running on Google Cloud

Basic client creation:

```python
from google.cloud import recaptchaenterprise_v1

client = recaptchaenterprise_v1.RecaptchaEnterpriseServiceClient()
```

### Service Account File

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

```python
from google.cloud import recaptchaenterprise_v1

client = recaptchaenterprise_v1.RecaptchaEnterpriseServiceClient()
```

### API Key for Python Client Libraries

The reCAPTCHA product docs explicitly say Python client libraries can use API keys or Workload Identity Federation outside Google Cloud. Google API Core exposes API-key auth through `ClientOptions`:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import recaptchaenterprise_v1

client = recaptchaenterprise_v1.RecaptchaEnterpriseServiceClient(
    client_options=ClientOptions(api_key="YOUR_API_KEY")
)
```

Use ADC or Workload Identity Federation for server-side production systems when possible. If you do use an API key, restrict it.

## Create An Assessment

This is the core flow for score-based or checkbox website integrations:

```python
from google.cloud import recaptchaenterprise_v1

def create_assessment(
    project_id: str,
    site_key: str,
    token: str,
    expected_action: str,
    user_ip_address: str,
    user_agent: str,
    ja3: str | None = None,
):
    client = recaptchaenterprise_v1.RecaptchaEnterpriseServiceClient()

    event = recaptchaenterprise_v1.Event(
        site_key=site_key,
        token=token,
        expected_action=expected_action,
        user_ip_address=user_ip_address,
        user_agent=user_agent,
        ja3=ja3 or "",
    )
    assessment = recaptchaenterprise_v1.Assessment(event=event)
    request = recaptchaenterprise_v1.CreateAssessmentRequest(
        parent=f"projects/{project_id}",
        assessment=assessment,
    )

    response = client.create_assessment(request=request)

    if not response.token_properties.valid:
        invalid_reason = response.token_properties.invalid_reason.name
        raise ValueError(f"Invalid token: {invalid_reason}")

    if response.token_properties.action != expected_action:
        raise ValueError(
            f"Action mismatch: expected {expected_action}, "
            f"got {response.token_properties.action}"
        )

    return {
        "assessment_name": response.name,
        "score": response.risk_analysis.score,
        "reasons": [reason.name for reason in response.risk_analysis.reasons],
    }
```

Important fields from the response:

- `response.token_properties.valid`: whether the token was accepted
- `response.token_properties.action`: must match your expected action for action-based integrations
- `response.risk_analysis.score`: risk score used for your allow/challenge/block logic
- `response.risk_analysis.reasons`: classifier reasons that explain higher-risk outcomes
- `response.name`: assessment resource name, reused later for annotation

## Annotate Assessments

If you later learn whether an interaction was legitimate or fraudulent, send that feedback back to reCAPTCHA. Product docs say this improves site-specific model performance over time.

```python
from google.cloud import recaptchaenterprise_v1

def annotate_fraudulent_assessment(project_id: str, assessment_id: str) -> None:
    client = recaptchaenterprise_v1.RecaptchaEnterpriseServiceClient()

    request = recaptchaenterprise_v1.AnnotateAssessmentRequest(
        name=f"projects/{project_id}/assessments/{assessment_id}",
        annotation=recaptchaenterprise_v1.AnnotateAssessmentRequest.Annotation.FRAUDULENT,
        reasons=[
            recaptchaenterprise_v1.AnnotateAssessmentRequest.Reason.FAILED_TWO_FACTOR,
        ],
    )

    client.annotate_assessment(request=request)
```

If you stored the full assessment resource name from `create_assessment`, you can pass that directly as `name` instead of rebuilding it.

## Key Management

### Create a score-based website key

```python
from google.cloud import recaptchaenterprise_v1

def create_score_key(project_id: str, domain_name: str) -> str:
    client = recaptchaenterprise_v1.RecaptchaEnterpriseServiceClient()

    web_settings = recaptchaenterprise_v1.WebKeySettings(
        allowed_domains=[domain_name],
        allow_amp_traffic=False,
        integration_type=recaptchaenterprise_v1.WebKeySettings.IntegrationType.SCORE,
    )
    key = recaptchaenterprise_v1.Key(
        display_name="primary-web-key",
        web_settings=web_settings,
    )

    response = client.create_key(
        request=recaptchaenterprise_v1.CreateKeyRequest(
            parent=f"projects/{project_id}",
            key=key,
        )
    )
    return response.name
```

### Get an existing key

```python
from google.cloud import recaptchaenterprise_v1

client = recaptchaenterprise_v1.RecaptchaEnterpriseServiceClient()
key = client.get_key(name="projects/PROJECT_ID/keys/KEY_ID")
```

### Fetch metrics for a key

```python
from google.cloud import recaptchaenterprise_v1

client = recaptchaenterprise_v1.RecaptchaEnterpriseServiceClient()
metrics = client.get_metrics(name="projects/PROJECT_ID/keys/KEY_ID/metrics")

for day_metric in metrics.score_metrics:
    print(day_metric.overall_metrics.score_buckets)
```

Use `metrics.score_metrics` for score-based keys and `metrics.challenge_metrics` for checkbox-style challenge keys.

## Async Client

The generated async surface is available if your app already uses `asyncio`:

```python
import asyncio
from google.cloud import recaptchaenterprise_v1

async def main() -> None:
    client = recaptchaenterprise_v1.RecaptchaEnterpriseServiceAsyncClient()
    metrics = await client.get_metrics(
        name="projects/PROJECT_ID/keys/KEY_ID/metrics"
    )
    print(metrics.name)

asyncio.run(main())
```

## Common Pitfalls

- Create assessments only on your backend. Google’s product docs explicitly warn against creating them in the browser because attackers can forge those requests.
- Tokens are single-use and expire after two minutes. If the frontend action is retried later, obtain a fresh token.
- Always verify `response.token_properties.action == expected_action` for action-based flows. A mismatch is a fraud signal.
- Pass extra context such as `userAgent`, `userIpAddress`, and TLS fingerprints (`ja3` or `ja4`) when available. Google’s docs recommend them to improve detection quality.
- Keep and reuse the assessment resource name. You need it for later annotation.
- Enable billing before debugging library code. Assessment calls stop once you exhaust the free monthly quota and do not have billing enabled.
- Reuse clients instead of creating a new client object for every request path in hot code.
- Prefer the `v1` API. Product docs explicitly recommend migrating away from `v1beta1` for newer features.

## Version-Sensitive Notes

- PyPI shows `1.30.0` as the current release for this package, published on January 15, 2026.
- The generated Google Cloud reference site is authoritative for API surface shape, but its `latest` class pages can lag in the visible version label. Validate the installed package version against PyPI when version precision matters.
- The current package still targets Python `>=3.7`, but for new projects you should align with a currently supported Python runtime in your deployment environment.

## Official Sources

- PyPI package: https://pypi.org/project/google-cloud-recaptcha-enterprise/
- Client library reference root: https://cloud.google.com/python/docs/reference/recaptchaenterprise/latest/index.html
- Client class reference: https://cloud.google.com/python/docs/reference/recaptchaenterprise/latest/google.cloud.recaptchaenterprise_v1.services.recaptcha_enterprise_service.RecaptchaEnterpriseServiceClient
- Product docs: https://cloud.google.com/recaptcha/docs/create-assessment-website
- Product samples:
  - https://cloud.google.com/recaptcha/docs/samples/recaptcha-enterprise-create-assessment
  - https://cloud.google.com/recaptcha/docs/samples/recaptcha-enterprise-annotate-assessment
  - https://cloud.google.com/recaptcha/docs/samples/recaptcha-enterprise-create-site-key
  - https://cloud.google.com/recaptcha/docs/samples/recaptcha-enterprise-get-metrics-site-key
- Auth docs:
  - https://cloud.google.com/docs/authentication/application-default-credentials
  - https://docs.cloud.google.com/docs/authentication/provide-credentials-adc
  - https://googleapis.dev/python/google-api-core/latest/client_options.html
