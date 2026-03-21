---
name: s3
description: "AWS S3 SDK for Python (boto3) - Complete guide for S3 operations in Python projects"
metadata:
  languages: "python"
  versions: "1.40.59"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "aws,s3,storage,cloud,bucket"
---

# AWS S3 SDK for Python (boto3) - Complete Guide

## Golden Rule

**ALWAYS use `boto3` for AWS S3 operations in Python projects.**

```bash
pip install boto3
```

**DO NOT use:**
- `boto` (legacy library, deprecated)
- Any unofficial S3 libraries

`boto3` is the official AWS SDK for Python. It provides low-level client access and high-level object-oriented resource access to AWS services.

**Python Version Requirements:**
- Python 3.9 or later (support for Python 3.8 ended on 2025-04-22)

---

## Installation

### Basic Installation

```bash
pip install boto3
```

### With pip (specific version)

```bash
pip install boto3==1.40.59
```

### With Poetry

```bash
poetry add boto3
```

### With uv

```bash
uv add boto3
```

### Environment Variables

Create a `.env` file:

```env
AWS_ACCESS_KEY_ID=your_access_key_id
AWS_SECRET_ACCESS_KEY=your_secret_access_key
AWS_DEFAULT_REGION=us-east-1
AWS_S3_BUCKET=your-bucket-name
```

Load environment variables in your code:

```python
import os
from dotenv import load_dotenv

load_dotenv()

REGION = os.getenv("AWS_DEFAULT_REGION")
BUCKET = os.getenv("AWS_S3_BUCKET")
```

### AWS Credentials Configuration

Boto3 looks for credentials in this order:

1. Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
2. Shared credentials file (`~/.aws/credentials`)
3. AWS config file (`~/.aws/config`)
4. IAM role (when running on EC2)

Configure credentials using AWS CLI:

```bash
aws configure
```

---

## Initialization

### Client vs Resource

boto3 provides two interfaces:

- **Client**: Low-level service access (1-to-1 mapping with AWS APIs)
- **Resource**: Higher-level object-oriented interface

```python
import boto3

# Client (low-level)
s3_client = boto3.client("s3")

# Resource (high-level)
s3_resource = boto3.resource("s3")
```

### Basic Client Setup

```python
import boto3

# Default credentials from environment or AWS config
s3 = boto3.client("s3")
```

### Client with Region

```python
import boto3

s3 = boto3.client("s3", region_name="us-east-1")
```

### Client with Explicit Credentials

```python
import boto3

s3 = boto3.client(
    "s3",
    aws_access_key_id="YOUR_ACCESS_KEY",
    aws_secret_access_key="YOUR_SECRET_KEY",
    region_name="us-east-1"
)
```

### Client with Session

```python
import boto3

session = boto3.Session(
    aws_access_key_id="YOUR_ACCESS_KEY",
    aws_secret_access_key="YOUR_SECRET_KEY",
    region_name="us-east-1"
)

s3 = session.client("s3")
```

### Resource Setup

```python
import boto3

s3 = boto3.resource("s3")

# With region
s3 = boto3.resource("s3", region_name="us-west-2")
```

### Client with Custom Endpoint (LocalStack, MinIO)

```python
import boto3

s3 = boto3.client(
    "s3",
    endpoint_url="http://localhost:4566",
    aws_access_key_id="test",
    aws_secret_access_key="test",
    region_name="us-east-1"
)
```

---

## Bucket Operations

### List All Buckets

```python
import boto3

s3 = boto3.client("s3")

def list_buckets():
    try:
        response = s3.list_buckets()

        print("Buckets:")
        for bucket in response["Buckets"]:
            print(f"  • {bucket['Name']} (Created: {bucket['CreationDate']})")
    except Exception as e:
        print(f"Error listing buckets: {e}")

# Using resource
s3 = boto3.resource("s3")

def list_buckets_resource():
    for bucket in s3.buckets.all():
        print(f"  • {bucket.name}")
```

### Create Bucket

```python
import boto3

s3 = boto3.client("s3")

def create_bucket(bucket_name):
    try:
        s3.create_bucket(Bucket=bucket_name)
        print(f"Bucket created: {bucket_name}")
    except Exception as e:
        print(f"Error creating bucket: {e}")

# Using resource
s3 = boto3.resource("s3")

def create_bucket_resource(bucket_name):
    try:
        s3.create_bucket(Bucket=bucket_name)
        print(f"Bucket created: {bucket_name}")
    except Exception as e:
        print(f"Error: {e}")
```

### Create Bucket in Specific Region

```python
import boto3

s3 = boto3.client("s3", region_name="us-west-2")

def create_bucket_in_region(bucket_name, region):
    try:
        if region == "us-east-1":
            s3.create_bucket(Bucket=bucket_name)
        else:
            s3.create_bucket(
                Bucket=bucket_name,
                CreateBucketConfiguration={"LocationConstraint": region}
            )

        print(f"Bucket created in {region}")
    except Exception as e:
        print(f"Error creating bucket: {e}")
```

### Delete Bucket

```python
import boto3

s3 = boto3.client("s3")

def delete_bucket(bucket_name):
    try:
        s3.delete_bucket(Bucket=bucket_name)
        print(f"Bucket deleted: {bucket_name}")
    except Exception as e:
        print(f"Error deleting bucket: {e}")

# Using resource
s3 = boto3.resource("s3")

def delete_bucket_resource(bucket_name):
    try:
        bucket = s3.Bucket(bucket_name)
        bucket.delete()
        print(f"Bucket deleted: {bucket_name}")
    except Exception as e:
        print(f"Error: {e}")
```

### Check if Bucket Exists

```python
import boto3
from botocore.exceptions import ClientError

s3 = boto3.client("s3")

def bucket_exists(bucket_name):
    try:
        s3.head_bucket(Bucket=bucket_name)
        return True
    except ClientError as e:
        if e.response["Error"]["Code"] == "404":
            return False
        raise
```

### Get Bucket Location

```python
import boto3

s3 = boto3.client("s3")

def get_bucket_location(bucket_name):
    try:
        response = s3.get_bucket_location(Bucket=bucket_name)
        location = response["LocationConstraint"]

        # us-east-1 returns None
        if location is None:
            location = "us-east-1"

        print(f"Bucket location: {location}")
        return location
    except Exception as e:
        print(f"Error: {e}")
```

---

## Object Upload Operations

### Upload File from Disk

```python
import boto3

s3 = boto3.client("s3")

def upload_file(file_path, bucket_name, object_key):
    try:
        s3.upload_file(file_path, bucket_name, object_key)
        print(f"Uploaded {file_path} to {bucket_name}/{object_key}")
    except Exception as e:
        print(f"Upload error: {e}")

# Using resource
s3 = boto3.resource("s3")

def upload_file_resource(file_path, bucket_name, object_key):
    try:
        bucket = s3.Bucket(bucket_name)
        bucket.upload_file(file_path, object_key)
        print(f"Uploaded {file_path}")
    except Exception as e:
        print(f"Upload error: {e}")
```

### Upload with put_object

```python
import boto3

s3 = boto3.client("s3")

def upload_object(bucket_name, key, data):
    try:
        s3.put_object(
            Bucket=bucket_name,
            Key=key,
            Body=data
        )
        print(f"Uploaded to {key}")
    except Exception as e:
        print(f"Upload error: {e}")

# Upload string
upload_object("my-bucket", "file.txt", b"Hello, S3!")

# Upload JSON
import json
upload_object("my-bucket", "data.json", json.dumps({"key": "value"}))
```

### Upload with Content Type

```python
import boto3

s3 = boto3.client("s3")

def upload_with_content_type(bucket_name, key, file_path, content_type):
    try:
        s3.upload_file(
            file_path,
            bucket_name,
            key,
            ExtraArgs={"ContentType": content_type}
        )
        print(f"Uploaded {key} as {content_type}")
    except Exception as e:
        print(f"Upload error: {e}")

# Usage
upload_with_content_type(
    "my-bucket",
    "image.jpg",
    "./photo.jpg",
    "image/jpeg"
)
```

### Upload with Metadata

```python
import boto3

s3 = boto3.client("s3")

def upload_with_metadata(bucket_name, key, file_path):
    try:
        s3.upload_file(
            file_path,
            bucket_name,
            key,
            ExtraArgs={
                "Metadata": {
                    "uploaded-by": "user123",
                    "original-name": "document.pdf",
                    "category": "reports"
                }
            }
        )
        print("Uploaded with metadata")
    except Exception as e:
        print(f"Upload error: {e}")
```

### Upload with Server-Side Encryption

```python
import boto3

s3 = boto3.client("s3")

def upload_encrypted(bucket_name, key, file_path):
    try:
        s3.upload_file(
            file_path,
            bucket_name,
            key,
            ExtraArgs={"ServerSideEncryption": "AES256"}
        )
        print("Uploaded with encryption")
    except Exception as e:
        print(f"Upload error: {e}")
```

### Upload with ACL

```python
import boto3

s3 = boto3.client("s3")

def upload_with_acl(bucket_name, key, file_path):
    try:
        s3.upload_file(
            file_path,
            bucket_name,
            key,
            ExtraArgs={"ACL": "public-read"}
        )
        print("Uploaded as public")
    except Exception as e:
        print(f"Upload error: {e}")
```

### Upload Binary Data

```python
import boto3

s3 = boto3.client("s3")

def upload_binary(bucket_name, key, data):
    try:
        s3.put_object(
            Bucket=bucket_name,
            Key=key,
            Body=data,
            ContentType="application/octet-stream"
        )
        print(f"Uploaded binary data to {key}")
    except Exception as e:
        print(f"Upload error: {e}")

# Usage
with open("image.png", "rb") as f:
    upload_binary("my-bucket", "uploads/image.png", f.read())
```

### Batch Upload Multiple Files

```python
import boto3
import os

s3 = boto3.client("s3")

def upload_directory(directory_path, bucket_name, s3_prefix=""):
    try:
        for root, dirs, files in os.walk(directory_path):
            for file in files:
                local_path = os.path.join(root, file)
                relative_path = os.path.relpath(local_path, directory_path)
                s3_key = os.path.join(s3_prefix, relative_path).replace("\\", "/")

                s3.upload_file(local_path, bucket_name, s3_key)
                print(f"Uploaded: {s3_key}")

        print(f"Uploaded directory: {directory_path}")
    except Exception as e:
        print(f"Batch upload error: {e}")
```

---

## Object Download Operations

### Download File to Disk

```python
import boto3

s3 = boto3.client("s3")

def download_file(bucket_name, key, file_path):
    try:
        s3.download_file(bucket_name, key, file_path)
        print(f"Downloaded to {file_path}")
    except Exception as e:
        print(f"Download error: {e}")

# Using resource
s3 = boto3.resource("s3")

def download_file_resource(bucket_name, key, file_path):
    try:
        bucket = s3.Bucket(bucket_name)
        bucket.download_file(key, file_path)
        print(f"Downloaded to {file_path}")
    except Exception as e:
        print(f"Download error: {e}")
```

### Download as Bytes

```python
import boto3

s3 = boto3.client("s3")

def download_as_bytes(bucket_name, key):
    try:
        response = s3.get_object(Bucket=bucket_name, Key=key)
        data = response["Body"].read()
        return data
    except Exception as e:
        print(f"Download error: {e}")
        return None

# Usage
data = download_as_bytes("my-bucket", "file.txt")
```

### Download as String

```python
import boto3

s3 = boto3.client("s3")

def download_as_string(bucket_name, key, encoding="utf-8"):
    try:
        response = s3.get_object(Bucket=bucket_name, Key=key)
        content = response["Body"].read().decode(encoding)
        return content
    except Exception as e:
        print(f"Download error: {e}")
        return None

# Download JSON
import json

def download_json(bucket_name, key):
    content = download_as_string(bucket_name, key)
    if content:
        return json.loads(content)
    return None
```

### Download with Error Handling

```python
import boto3
from botocore.exceptions import ClientError

s3 = boto3.client("s3")

def download_safe(bucket_name, key, file_path):
    try:
        s3.download_file(bucket_name, key, file_path)
        print(f"Downloaded to {file_path}")
        return True
    except ClientError as e:
        if e.response["Error"]["Code"] == "404":
            print(f"Object not found: {key}")
        else:
            print(f"Error: {e}")
        return False
```

### Download with Version

```python
import boto3

s3 = boto3.client("s3")

def download_version(bucket_name, key, file_path, version_id):
    try:
        s3.download_file(
            bucket_name,
            key,
            file_path,
            ExtraArgs={"VersionId": version_id}
        )
        print(f"Downloaded version {version_id}")
    except Exception as e:
        print(f"Download error: {e}")
```

### Download Byte Range

```python
import boto3

s3 = boto3.client("s3")

def download_range(bucket_name, key, start, end):
    try:
        response = s3.get_object(
            Bucket=bucket_name,
            Key=key,
            Range=f"bytes={start}-{end}"
        )
        data = response["Body"].read()
        return data
    except Exception as e:
        print(f"Download error: {e}")
        return None
```

### Get Object Metadata Only

```python
import boto3

s3 = boto3.client("s3")

def get_object_metadata(bucket_name, key):
    try:
        response = s3.head_object(Bucket=bucket_name, Key=key)

        print("Metadata:")
        print(f"  Content-Type: {response.get('ContentType')}")
        print(f"  Content-Length: {response.get('ContentLength')} bytes")
        print(f"  Last Modified: {response.get('LastModified')}")
        print(f"  ETag: {response.get('ETag')}")
        print(f"  Metadata: {response.get('Metadata')}")

        return response
    except Exception as e:
        print(f"Error: {e}")
```

---

## List Objects

### List All Objects

```python
import boto3

s3 = boto3.client("s3")

def list_objects(bucket_name):
    try:
        response = s3.list_objects_v2(Bucket=bucket_name)

        if "Contents" in response:
            print("Objects:")
            for obj in response["Contents"]:
                print(f"  • {obj['Key']} ({obj['Size']} bytes)")
        else:
            print("Bucket is empty")
    except Exception as e:
        print(f"List error: {e}")

# Using resource
s3 = boto3.resource("s3")

def list_objects_resource(bucket_name):
    try:
        bucket = s3.Bucket(bucket_name)

        for obj in bucket.objects.all():
            print(f"  • {obj.key} ({obj.size} bytes)")
    except Exception as e:
        print(f"List error: {e}")
```

### List Objects with Prefix

```python
import boto3

s3 = boto3.client("s3")

def list_objects_with_prefix(bucket_name, prefix):
    try:
        response = s3.list_objects_v2(Bucket=bucket_name, Prefix=prefix)

        if "Contents" in response:
            print(f"Objects with prefix '{prefix}':")
            for obj in response["Contents"]:
                print(f"  • {obj['Key']}")
        else:
            print(f"No objects with prefix '{prefix}'")
    except Exception as e:
        print(f"List error: {e}")

# Using resource
s3 = boto3.resource("s3")

def list_objects_with_prefix_resource(bucket_name, prefix):
    bucket = s3.Bucket(bucket_name)

    for obj in bucket.objects.filter(Prefix=prefix):
        print(f"  • {obj.key}")
```

### List Objects with Pagination

```python
import boto3

s3 = boto3.client("s3")

def list_objects_paginated(bucket_name):
    try:
        paginator = s3.get_paginator("list_objects_v2")
        page_iterator = paginator.paginate(Bucket=bucket_name)

        total_objects = 0

        for page in page_iterator:
            if "Contents" in page:
                for obj in page["Contents"]:
                    print(f"  • {obj['Key']}")
                    total_objects += 1

        print(f"\nTotal objects: {total_objects}")
    except Exception as e:
        print(f"List error: {e}")
```

### List Objects with Max Keys

```python
import boto3

s3 = boto3.client("s3")

def list_objects_limited(bucket_name, max_keys=10):
    try:
        response = s3.list_objects_v2(Bucket=bucket_name, MaxKeys=max_keys)

        if "Contents" in response:
            for obj in response["Contents"]:
                print(f"  • {obj['Key']}")

            print(f"\nShowing {len(response['Contents'])} objects")
            print(f"More available: {response.get('IsTruncated', False)}")
    except Exception as e:
        print(f"List error: {e}")
```

### List Objects in Folder Structure

```python
import boto3

s3 = boto3.client("s3")

def list_folder(bucket_name, prefix=""):
    try:
        response = s3.list_objects_v2(
            Bucket=bucket_name,
            Prefix=prefix,
            Delimiter="/"
        )

        # List folders (common prefixes)
        if "CommonPrefixes" in response:
            print("Folders:")
            for cp in response["CommonPrefixes"]:
                print(f"  📁 {cp['Prefix']}")

        # List files
        if "Contents" in response:
            print("\nFiles:")
            for obj in response["Contents"]:
                print(f"  📄 {obj['Key']}")
    except Exception as e:
        print(f"List error: {e}")
```

### List with Filter

```python
import boto3

s3 = boto3.resource("s3")

def list_objects_filtered(bucket_name, extension):
    try:
        bucket = s3.Bucket(bucket_name)

        for obj in bucket.objects.all():
            if obj.key.endswith(extension):
                print(f"  • {obj.key}")
    except Exception as e:
        print(f"List error: {e}")

# Usage
list_objects_filtered("my-bucket", ".jpg")
```

---

## Delete Operations

### Delete Single Object

```python
import boto3

s3 = boto3.client("s3")

def delete_object(bucket_name, key):
    try:
        s3.delete_object(Bucket=bucket_name, Key=key)
        print(f"Deleted: {key}")
    except Exception as e:
        print(f"Delete error: {e}")

# Using resource
s3 = boto3.resource("s3")

def delete_object_resource(bucket_name, key):
    try:
        obj = s3.Object(bucket_name, key)
        obj.delete()
        print(f"Deleted: {key}")
    except Exception as e:
        print(f"Delete error: {e}")
```

### Delete Multiple Objects (Batch)

```python
import boto3

s3 = boto3.client("s3")

def delete_multiple_objects(bucket_name, keys):
    try:
        response = s3.delete_objects(
            Bucket=bucket_name,
            Delete={
                "Objects": [{"Key": key} for key in keys]
            }
        )

        if "Deleted" in response:
            print(f"Deleted {len(response['Deleted'])} objects:")
            for obj in response["Deleted"]:
                print(f"  • {obj['Key']}")

        if "Errors" in response:
            print("Errors:")
            for error in response["Errors"]:
                print(f"  • {error['Key']}: {error['Message']}")
    except Exception as e:
        print(f"Batch delete error: {e}")

# Usage
delete_multiple_objects("my-bucket", ["file1.txt", "file2.txt", "file3.txt"])
```

### Delete All Objects with Prefix

```python
import boto3

s3 = boto3.client("s3")

def delete_objects_with_prefix(bucket_name, prefix):
    try:
        response = s3.list_objects_v2(Bucket=bucket_name, Prefix=prefix)

        if "Contents" not in response:
            print("No objects to delete")
            return

        keys = [obj["Key"] for obj in response["Contents"]]

        s3.delete_objects(
            Bucket=bucket_name,
            Delete={"Objects": [{"Key": key} for key in keys]}
        )

        print(f"Deleted {len(keys)} objects with prefix '{prefix}'")
    except Exception as e:
        print(f"Delete error: {e}")
```

### Delete All Objects in Bucket

```python
import boto3

s3 = boto3.resource("s3")

def empty_bucket(bucket_name):
    try:
        bucket = s3.Bucket(bucket_name)
        bucket.objects.all().delete()
        print(f"Emptied bucket: {bucket_name}")
    except Exception as e:
        print(f"Empty error: {e}")
```

### Empty and Delete Bucket

```python
import boto3

s3 = boto3.resource("s3")

def empty_and_delete_bucket(bucket_name):
    try:
        bucket = s3.Bucket(bucket_name)

        # Delete all objects
        bucket.objects.all().delete()
        print(f"Deleted all objects")

        # Delete all object versions (if versioning enabled)
        bucket.object_versions.all().delete()
        print(f"Deleted all object versions")

        # Delete bucket
        bucket.delete()
        print(f"Deleted bucket: {bucket_name}")
    except Exception as e:
        print(f"Error: {e}")
```

---

## Copy and Move Operations

### Copy Object

```python
import boto3

s3 = boto3.client("s3")

def copy_object(source_bucket, source_key, dest_bucket, dest_key):
    try:
        copy_source = {"Bucket": source_bucket, "Key": source_key}

        s3.copy_object(
            CopySource=copy_source,
            Bucket=dest_bucket,
            Key=dest_key
        )

        print(f"Copied {source_key} to {dest_key}")
    except Exception as e:
        print(f"Copy error: {e}")

# Using resource
s3 = boto3.resource("s3")

def copy_object_resource(source_bucket, source_key, dest_bucket, dest_key):
    try:
        copy_source = {"Bucket": source_bucket, "Key": source_key}

        dest_obj = s3.Object(dest_bucket, dest_key)
        dest_obj.copy_from(CopySource=copy_source)

        print(f"Copied {source_key} to {dest_key}")
    except Exception as e:
        print(f"Copy error: {e}")
```

### Copy with Metadata

```python
import boto3

s3 = boto3.client("s3")

def copy_with_metadata(source_bucket, source_key, dest_bucket, dest_key):
    try:
        copy_source = {"Bucket": source_bucket, "Key": source_key}

        s3.copy_object(
            CopySource=copy_source,
            Bucket=dest_bucket,
            Key=dest_key,
            Metadata={
                "copied-from": source_key,
                "copied-date": "2025-10-26"
            },
            MetadataDirective="REPLACE"
        )

        print("Copied with new metadata")
    except Exception as e:
        print(f"Copy error: {e}")
```

### Move Object (Copy + Delete)

```python
import boto3

s3 = boto3.client("s3")

def move_object(source_bucket, source_key, dest_bucket, dest_key):
    try:
        # Copy
        copy_source = {"Bucket": source_bucket, "Key": source_key}
        s3.copy_object(
            CopySource=copy_source,
            Bucket=dest_bucket,
            Key=dest_key
        )

        # Delete source
        s3.delete_object(Bucket=source_bucket, Key=source_key)

        print(f"Moved {source_key} to {dest_key}")
    except Exception as e:
        print(f"Move error: {e}")
```

---

## Presigned URLs

### Presigned URL for Download (GET)

```python
import boto3
from botocore.config import Config

s3 = boto3.client("s3", config=Config(signature_version="s3v4"))

def create_presigned_download_url(bucket_name, key, expiration=3600):
    try:
        url = s3.generate_presigned_url(
            ClientMethod="get_object",
            Params={"Bucket": bucket_name, "Key": key},
            ExpiresIn=expiration
        )
        return url
    except Exception as e:
        print(f"Presigned URL error: {e}")
        return None

# Usage
url = create_presigned_download_url("my-bucket", "file.pdf", 3600)
print(f"Download URL: {url}")
```

### Presigned URL for Upload (PUT)

```python
import boto3
from botocore.config import Config

s3 = boto3.client("s3", config=Config(signature_version="s3v4"))

def create_presigned_upload_url(bucket_name, key, expiration=3600):
    try:
        url = s3.generate_presigned_url(
            ClientMethod="put_object",
            Params={"Bucket": bucket_name, "Key": key},
            ExpiresIn=expiration
        )
        return url
    except Exception as e:
        print(f"Presigned URL error: {e}")
        return None

# Usage with requests
import requests

upload_url = create_presigned_upload_url("my-bucket", "upload.txt")

with open("file.txt", "rb") as f:
    response = requests.put(upload_url, data=f)
    print(f"Upload status: {response.status_code}")
```

### Presigned URL with Content Type

```python
import boto3
from botocore.config import Config

s3 = boto3.client("s3", config=Config(signature_version="s3v4"))

def create_presigned_url_with_type(bucket_name, key, content_type, expiration=3600):
    try:
        url = s3.generate_presigned_url(
            ClientMethod="put_object",
            Params={
                "Bucket": bucket_name,
                "Key": key,
                "ContentType": content_type
            },
            ExpiresIn=expiration
        )
        return url
    except Exception as e:
        print(f"Presigned URL error: {e}")
        return None
```

### Presigned POST for File Upload

```python
import boto3

s3 = boto3.client("s3")

def create_presigned_post(bucket_name, key, expiration=3600):
    try:
        response = s3.generate_presigned_post(
            Bucket=bucket_name,
            Key=key,
            ExpiresIn=expiration
        )
        return response
    except Exception as e:
        print(f"Presigned POST error: {e}")
        return None

# Usage
presigned_post = create_presigned_post("my-bucket", "upload.txt")

# Client uploads using POST with the URL and fields
import requests

with open("file.txt", "rb") as f:
    files = {"file": f}
    response = requests.post(
        presigned_post["url"],
        data=presigned_post["fields"],
        files=files
    )
    print(f"Upload status: {response.status_code}")
```

### Presigned POST with Conditions

```python
import boto3

s3 = boto3.client("s3")

def create_presigned_post_with_conditions(bucket_name, key, max_size=10485760):
    try:
        response = s3.generate_presigned_post(
            Bucket=bucket_name,
            Key=key,
            Fields={"acl": "public-read"},
            Conditions=[
                {"acl": "public-read"},
                ["content-length-range", 0, max_size]  # Max 10 MB
            ],
            ExpiresIn=3600
        )
        return response
    except Exception as e:
        print(f"Presigned POST error: {e}")
        return None
```

---

## Multipart Upload

### Automatic Multipart Upload (Recommended)

```python
import boto3
from boto3.s3.transfer import TransferConfig

s3 = boto3.client("s3")

def multipart_upload(file_path, bucket_name, key):
    try:
        # Configure multipart upload
        config = TransferConfig(
            multipart_threshold=1024 * 25,  # 25 MB
            max_concurrency=10,
            multipart_chunksize=1024 * 25,  # 25 MB
            use_threads=True
        )

        s3.upload_file(
            file_path,
            bucket_name,
            key,
            Config=config
        )

        print(f"Multipart upload complete: {key}")
    except Exception as e:
        print(f"Upload error: {e}")
```

### Multipart Upload with Progress Callback

```python
import boto3
import sys

s3 = boto3.client("s3")

class ProgressPercentage:
    def __init__(self, filename):
        self._filename = filename
        self._size = float(os.path.getsize(filename))
        self._seen_so_far = 0

    def __call__(self, bytes_amount):
        self._seen_so_far += bytes_amount
        percentage = (self._seen_so_far / self._size) * 100
        sys.stdout.write(
            f"\r{self._filename}: {self._seen_so_far} / {self._size} ({percentage:.2f}%)"
        )
        sys.stdout.flush()

def multipart_upload_with_progress(file_path, bucket_name, key):
    try:
        s3.upload_file(
            file_path,
            bucket_name,
            key,
            Callback=ProgressPercentage(file_path)
        )
        print(f"\nUpload complete: {key}")
    except Exception as e:
        print(f"Upload error: {e}")
```

### Manual Multipart Upload (Low-Level)

```python
import boto3
import os

s3 = boto3.client("s3")

def manual_multipart_upload(file_path, bucket_name, key):
    try:
        # Step 1: Initiate multipart upload
        response = s3.create_multipart_upload(
            Bucket=bucket_name,
            Key=key
        )
        upload_id = response["UploadId"]

        # Step 2: Upload parts
        parts = []
        chunk_size = 10 * 1024 * 1024  # 10 MB
        part_number = 1

        with open(file_path, "rb") as f:
            while True:
                data = f.read(chunk_size)
                if not data:
                    break

                response = s3.upload_part(
                    Bucket=bucket_name,
                    Key=key,
                    PartNumber=part_number,
                    UploadId=upload_id,
                    Body=data
                )

                parts.append({
                    "PartNumber": part_number,
                    "ETag": response["ETag"]
                })

                print(f"Uploaded part {part_number}")
                part_number += 1

        # Step 3: Complete multipart upload
        s3.complete_multipart_upload(
            Bucket=bucket_name,
            Key=key,
            UploadId=upload_id,
            MultipartUpload={"Parts": parts}
        )

        print(f"Multipart upload complete: {key}")
    except Exception as e:
        # Abort multipart upload on error
        s3.abort_multipart_upload(
            Bucket=bucket_name,
            Key=key,
            UploadId=upload_id
        )
        print(f"Upload error: {e}")
```

### List In-Progress Multipart Uploads

```python
import boto3

s3 = boto3.client("s3")

def list_multipart_uploads(bucket_name):
    try:
        response = s3.list_multipart_uploads(Bucket=bucket_name)

        if "Uploads" in response:
            print("In-progress uploads:")
            for upload in response["Uploads"]:
                print(f"  • {upload['Key']} (ID: {upload['UploadId']})")
        else:
            print("No in-progress uploads")
    except Exception as e:
        print(f"List error: {e}")
```

### Abort Multipart Upload

```python
import boto3

s3 = boto3.client("s3")

def abort_multipart_upload(bucket_name, key, upload_id):
    try:
        s3.abort_multipart_upload(
            Bucket=bucket_name,
            Key=key,
            UploadId=upload_id
        )
        print(f"Aborted upload: {key}")
    except Exception as e:
        print(f"Abort error: {e}")
```

---

## Object Tagging

### Put Object Tags

```python
import boto3

s3 = boto3.client("s3")

def tag_object(bucket_name, key, tags):
    try:
        s3.put_object_tagging(
            Bucket=bucket_name,
            Key=key,
            Tagging={
                "TagSet": [
                    {"Key": k, "Value": v} for k, v in tags.items()
                ]
            }
        )
        print(f"Tagged {key}")
    except Exception as e:
        print(f"Tagging error: {e}")

# Usage
tag_object("my-bucket", "file.txt", {
    "Environment": "Production",
    "Department": "Engineering"
})
```

### Get Object Tags

```python
import boto3

s3 = boto3.client("s3")

def get_object_tags(bucket_name, key):
    try:
        response = s3.get_object_tagging(Bucket=bucket_name, Key=key)

        print("Tags:")
        for tag in response["TagSet"]:
            print(f"  {tag['Key']}: {tag['Value']}")

        return response["TagSet"]
    except Exception as e:
        print(f"Get tags error: {e}")
        return None
```

### Delete Object Tags

```python
import boto3

s3 = boto3.client("s3")

def delete_object_tags(bucket_name, key):
    try:
        s3.delete_object_tagging(Bucket=bucket_name, Key=key)
        print(f"Deleted tags from {key}")
    except Exception as e:
        print(f"Delete tags error: {e}")
```

---

## Bucket Configuration

### Enable Versioning

```python
import boto3

s3 = boto3.client("s3")

def enable_versioning(bucket_name):
    try:
        s3.put_bucket_versioning(
            Bucket=bucket_name,
            VersioningConfiguration={"Status": "Enabled"}
        )
        print(f"Versioning enabled for {bucket_name}")
    except Exception as e:
        print(f"Versioning error: {e}")
```

### Get Versioning Status

```python
import boto3

s3 = boto3.client("s3")

def get_versioning_status(bucket_name):
    try:
        response = s3.get_bucket_versioning(Bucket=bucket_name)
        status = response.get("Status", "Disabled")
        print(f"Versioning status: {status}")
        return status
    except Exception as e:
        print(f"Error: {e}")
```

### Set Bucket CORS

```python
import boto3

s3 = boto3.client("s3")

def set_bucket_cors(bucket_name):
    try:
        cors_configuration = {
            "CORSRules": [
                {
                    "AllowedHeaders": ["*"],
                    "AllowedMethods": ["GET", "PUT", "POST", "DELETE"],
                    "AllowedOrigins": ["*"],
                    "ExposeHeaders": ["ETag"],
                    "MaxAgeSeconds": 3000
                }
            ]
        }

        s3.put_bucket_cors(
            Bucket=bucket_name,
            CORSConfiguration=cors_configuration
        )

        print("CORS configured")
    except Exception as e:
        print(f"CORS error: {e}")
```

### Get Bucket CORS

```python
import boto3

s3 = boto3.client("s3")

def get_bucket_cors(bucket_name):
    try:
        response = s3.get_bucket_cors(Bucket=bucket_name)
        print("CORS Rules:")
        print(response["CORSRules"])
        return response["CORSRules"]
    except Exception as e:
        print(f"Get CORS error: {e}")
        return None
```

### Delete Bucket CORS

```python
import boto3

s3 = boto3.client("s3")

def delete_bucket_cors(bucket_name):
    try:
        s3.delete_bucket_cors(Bucket=bucket_name)
        print("CORS configuration deleted")
    except Exception as e:
        print(f"Delete CORS error: {e}")
```

### Set Bucket Encryption

```python
import boto3

s3 = boto3.client("s3")

def set_bucket_encryption(bucket_name):
    try:
        s3.put_bucket_encryption(
            Bucket=bucket_name,
            ServerSideEncryptionConfiguration={
                "Rules": [
                    {
                        "ApplyServerSideEncryptionByDefault": {
                            "SSEAlgorithm": "AES256"
                        }
                    }
                ]
            }
        )
        print("Encryption enabled")
    except Exception as e:
        print(f"Encryption error: {e}")
```

### Set Bucket Lifecycle Policy

```python
import boto3

s3 = boto3.client("s3")

def set_lifecycle_policy(bucket_name):
    try:
        s3.put_bucket_lifecycle_configuration(
            Bucket=bucket_name,
            LifecycleConfiguration={
                "Rules": [
                    {
                        "Id": "DeleteOldObjects",
                        "Status": "Enabled",
                        "Expiration": {"Days": 90},
                        "Filter": {"Prefix": "logs/"}
                    }
                ]
            }
        )
        print("Lifecycle policy configured")
    except Exception as e:
        print(f"Lifecycle error: {e}")
```

---

## Waiters

### Wait Until Object Exists

```python
import boto3

s3 = boto3.client("s3")

def wait_for_object(bucket_name, key):
    try:
        waiter = s3.get_waiter("object_exists")
        waiter.wait(Bucket=bucket_name, Key=key)
        print(f"Object exists: {key}")
    except Exception as e:
        print(f"Wait error: {e}")
```

### Wait Until Object Not Exists

```python
import boto3

s3 = boto3.client("s3")

def wait_for_object_deletion(bucket_name, key):
    try:
        waiter = s3.get_waiter("object_not_exists")
        waiter.wait(Bucket=bucket_name, Key=key)
        print(f"Object deleted: {key}")
    except Exception as e:
        print(f"Wait error: {e}")
```

### Wait Until Bucket Exists

```python
import boto3

s3 = boto3.client("s3")

def wait_for_bucket(bucket_name):
    try:
        waiter = s3.get_waiter("bucket_exists")
        waiter.wait(Bucket=bucket_name)
        print(f"Bucket exists: {bucket_name}")
    except Exception as e:
        print(f"Wait error: {e}")
```

---

## Error Handling

### Common Error Types

```python
import boto3
from botocore.exceptions import ClientError, NoCredentialsError

s3 = boto3.client("s3")

def handle_errors(bucket_name, key):
    try:
        response = s3.get_object(Bucket=bucket_name, Key=key)
        return response["Body"].read()
    except NoCredentialsError:
        print("No AWS credentials found")
    except ClientError as e:
        error_code = e.response["Error"]["Code"]

        if error_code == "NoSuchKey":
            print("Object not found")
        elif error_code == "NoSuchBucket":
            print("Bucket not found")
        elif error_code == "AccessDenied":
            print("Access denied")
        elif error_code == "InvalidAccessKeyId":
            print("Invalid credentials")
        else:
            print(f"Error: {error_code}")
    except Exception as e:
        print(f"Unknown error: {e}")
```

### Retry Logic with Backoff

```python
import boto3
import time
from botocore.exceptions import ClientError

s3 = boto3.client("s3")

def download_with_retry(bucket_name, key, max_retries=3):
    for attempt in range(max_retries):
        try:
            response = s3.get_object(Bucket=bucket_name, Key=key)
            return response["Body"].read()
        except ClientError as e:
            if attempt < max_retries - 1:
                wait_time = 2 ** attempt  # Exponential backoff
                print(f"Attempt {attempt + 1} failed, retrying in {wait_time}s...")
                time.sleep(wait_time)
            else:
                raise
```

### Custom Retry Configuration

```python
import boto3
from botocore.config import Config

# Configure custom retry settings
config = Config(
    retries={
        "max_attempts": 10,
        "mode": "adaptive"
    }
)

s3 = boto3.client("s3", config=config)
```

---

## Complete Example: Full S3 Operations

```python
import boto3
from botocore.exceptions import ClientError
import json

def main():
    s3 = boto3.client("s3", region_name="us-east-1")
    bucket_name = f"my-test-bucket-{int(time.time())}"

    try:
        # 1. Create bucket
        print("Creating bucket...")
        s3.create_bucket(Bucket=bucket_name)

        # 2. Upload object
        print("Uploading object...")
        s3.put_object(
            Bucket=bucket_name,
            Key="test-file.txt",
            Body=b"Hello, S3!",
            ContentType="text/plain"
        )

        # 3. Generate presigned URL
        print("Generating presigned URL...")
        url = s3.generate_presigned_url(
            ClientMethod="get_object",
            Params={"Bucket": bucket_name, "Key": "test-file.txt"},
            ExpiresIn=3600
        )
        print(f"Presigned URL: {url}")

        # 4. Download object
        print("Downloading object...")
        response = s3.get_object(Bucket=bucket_name, Key="test-file.txt")
        content = response["Body"].read().decode("utf-8")
        print(f"Content: {content}")

        # 5. List objects
        print("Listing objects...")
        response = s3.list_objects_v2(Bucket=bucket_name)
        for obj in response.get("Contents", []):
            print(f"  • {obj['Key']}")

        # 6. Delete object
        print("Deleting object...")
        s3.delete_object(Bucket=bucket_name, Key="test-file.txt")

        # 7. Delete bucket
        print("Deleting bucket...")
        s3.delete_bucket(Bucket=bucket_name)

        print("Complete!")

    except ClientError as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    import time
    main()
```
