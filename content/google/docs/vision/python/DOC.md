---
name: vision
description: "google-cloud-vision package guide for Python Cloud Vision API clients"
metadata:
  languages: "python"
  versions: "3.12.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,vision,gcp,ocr,images,python"
---

# google-cloud-vision Python Package Guide

## Golden Rule

Use `google-cloud-vision` with Application Default Credentials (ADC), enable `vision.googleapis.com` in the same Google Cloud project, and reuse a single `ImageAnnotatorClient` for normal request traffic.

## Install

```bash
pip install google-cloud-vision==3.12.1
```

Use `pip install --upgrade google-cloud-vision` only when you explicitly want the newest release instead of the pinned version used here.

## Setup And Authentication

You need all of the following before API calls will succeed:

1. A Google Cloud project.
2. Billing enabled for that project.
3. The Cloud Vision API enabled.
4. ADC configured for the identity your code will run as.

Enable the API:

```bash
gcloud services enable vision.googleapis.com
```

For local development, the standard ADC flow is:

```bash
gcloud auth application-default login
```

If you are not running on Google Cloud and must use a service account key file:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
```

Production code on Google Cloud should usually rely on the runtime's attached service account instead of shipping key files.

## Initialize The Client

```python
from google.cloud import vision

client = vision.ImageAnnotatorClient()
```

The client uses ADC automatically. Create it once and reuse it for multiple requests instead of constructing a new client per image.

If you need a non-default endpoint or networking config, pass `client_options`:

```python
from google.cloud import vision

client = vision.ImageAnnotatorClient(
    client_options={"api_endpoint": "vision.googleapis.com"}
)
```

## Core Usage

### Detect Labels From A Cloud Storage Object

```python
from google.cloud import vision

client = vision.ImageAnnotatorClient()

image = vision.Image()
image.source.image_uri = "gs://cloud-samples-data/vision/label/wakeupcat.jpg"

response = client.label_detection(image=image)

if response.error.message:
    raise RuntimeError(response.error.message)

for label in response.label_annotations:
    print(label.description, label.score)
```

### Detect Labels From Local Image Bytes

```python
from google.cloud import vision

client = vision.ImageAnnotatorClient()

with open("image.jpg", "rb") as fh:
    image = vision.Image(content=fh.read())

response = client.label_detection(image=image)

if response.error.message:
    raise RuntimeError(response.error.message)

for label in response.label_annotations:
    print(label.description)
```

### OCR For Documents And Dense Text

Use `document_text_detection` for scanned pages, forms, and multi-block OCR. It returns the richer `full_text_annotation` tree.

```python
from google.cloud import vision

client = vision.ImageAnnotatorClient()

with open("document.png", "rb") as fh:
    image = vision.Image(content=fh.read())

response = client.document_text_detection(image=image)

if response.error.message:
    raise RuntimeError(response.error.message)

print(response.full_text_annotation.text)
```

### Combine Multiple Features In One Request

Use `batch_annotate_images` when you want multiple feature passes over the same image and a single request/response envelope.

```python
from google.cloud import vision

client = vision.ImageAnnotatorClient()

with open("image.jpg", "rb") as fh:
    image = vision.Image(content=fh.read())

request = vision.AnnotateImageRequest(
    image=image,
    features=[
        vision.Feature(type_=vision.Feature.Type.LABEL_DETECTION),
        vision.Feature(type_=vision.Feature.Type.IMAGE_PROPERTIES),
    ],
)

response = client.batch_annotate_images(requests=[request])
annotated = response.responses[0]

if annotated.error.message:
    raise RuntimeError(annotated.error.message)

for label in annotated.label_annotations:
    print(label.description)
```

### Large Async Jobs With Cloud Storage Input And Output

For many images, or when you want results written back to Cloud Storage, use `async_batch_annotate_images`.

```python
from google.cloud import vision

client = vision.ImageAnnotatorClient()

source = {"image_uri": "gs://your-bucket/input/image.jpg"}
image = {"source": source}
features = [{"type_": vision.Feature.Type.LABEL_DETECTION}]
requests = [{"image": image, "features": features}]

output_config = {
    "gcs_destination": {"uri": "gs://your-bucket/output/"},
    "batch_size": 2,
}

operation = client.async_batch_annotate_images(
    requests=requests,
    output_config=output_config,
)

result = operation.result(timeout=300)
print(result.output_config.gcs_destination.uri)
```

For PDFs and TIFFs, use file-oriented methods such as `batch_annotate_files` or `async_batch_annotate_files` instead of image-only methods.

## Configuration Notes

- ADC resolution order matters. Local shells, IDEs, CI jobs, and deployed workloads can each pick up different identities.
- If you read from or write to `gs://...` URIs, the calling identity also needs the relevant Cloud Storage IAM permissions in addition to Vision API access.
- The generated reference uses `google.cloud.vision_v1` service namespaces, but normal application imports commonly use `from google.cloud import vision`.
- Prefer typed request objects and enums for long-lived code. Ad-hoc dict payloads work, but they are easier to get wrong when fields are renamed or nested.

## Common Pitfalls

- Missing ADC is the most common failure mode. `DefaultCredentialsError` usually means you have not run `gcloud auth application-default login`, have the wrong environment variable, or are using credentials from the wrong project.
- API enablement and credentials must point at the same project. A valid credential alone is not enough if `vision.googleapis.com` is disabled there.
- `text_detection` and `document_text_detection` are not interchangeable. Prefer the document variant for dense OCR and scanned documents.
- Per-image failures are returned inside batch responses. Always inspect `response.error.message` or `annotated.error.message`, not just transport success.
- Cloud Storage input and output require bucket permissions. Vision access does not grant read or write access to `gs://` paths automatically.
- Reusing the client matters in hot paths. Creating a new client for every request adds avoidable overhead.

## Version-Sensitive Notes For `3.12.1`

- PyPI lists `3.12.1` as the package version covered by this entry.
- On `2026-03-12`, the official `latest` Python reference pages were also serving `3.12.1`, so the version used here and the current reference root were aligned.
- On `2026-03-12`, the official Vision Python changelog page still topped out at `3.12.0`. If you need the exact delta from `3.12.0` to `3.12.1`, check the PyPI release page and repository history in addition to the generated changelog.

## Official Sources

- Python reference root: `https://cloud.google.com/python/docs/reference/vision/latest`
- `ImageAnnotatorClient` reference: `https://cloud.google.com/python/docs/reference/vision/latest/google.cloud.vision_v1.services.image_annotator.ImageAnnotatorClient`
- Label detection quickstart: `https://cloud.google.com/vision/docs/detect-labels-image-client-libraries`
- OCR guide: `https://cloud.google.com/vision/docs/detecting-fulltext`
- Async batch sample: `https://cloud.google.com/vision/docs/samples/vision-async-batch-annotate-images`
- ADC overview: `https://cloud.google.com/docs/authentication/provide-credentials-adc`
- Local ADC setup: `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`
- Package registry: `https://pypi.org/project/google-cloud-vision/`
