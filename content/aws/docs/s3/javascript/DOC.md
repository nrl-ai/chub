---
name: s3
description: "AWS S3 SDK for JavaScript (v3) - Complete guide for S3 operations in JavaScript/Node.js projects"
metadata:
  languages: "javascript"
  versions: "3.917.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "aws,s3,storage,cloud,bucket"
---

# AWS S3 SDK for JavaScript (v3) - Complete Guide

## Golden Rule

**ALWAYS use `@aws-sdk/client-s3` for AWS S3 operations in JavaScript/Node.js projects.**

```bash
npm install @aws-sdk/client-s3
```

**DO NOT use:**
- `aws-sdk` (v2) - End-of-support on September 8, 2025
- Any unofficial S3 libraries

The AWS SDK for JavaScript v3 (`@aws-sdk/client-s3`) is the official, maintained SDK. It uses a modular architecture that reduces bundle size and improves performance.

---

## Installation

### Basic Installation

```bash
npm install @aws-sdk/client-s3
```

### Additional Packages for Advanced Features

```bash
# For presigned URLs
npm install @aws-sdk/s3-request-presigner

# For multipart uploads (large files)
npm install @aws-sdk/lib-storage

# For credential providers
npm install @aws-sdk/credential-providers
```

### Environment Variables

Create a `.env` file:

```env
AWS_ACCESS_KEY_ID=your_access_key_id
AWS_SECRET_ACCESS_KEY=your_secret_access_key
AWS_REGION=us-east-1
AWS_S3_BUCKET=your-bucket-name
```

Load environment variables in your code:

```javascript
import dotenv from 'dotenv';
dotenv.config();

const REGION = process.env.AWS_REGION;
const BUCKET = process.env.AWS_S3_BUCKET;
```

---

## Initialization

### Basic Client Setup

```javascript
import { S3Client } from "@aws-sdk/client-s3";

// Default credentials from environment variables or AWS config files
const client = new S3Client({});
```

### Client with Region

```javascript
import { S3Client } from "@aws-sdk/client-s3";

const client = new S3Client({
  region: "us-east-1"
});
```

### Client with Explicit Credentials

```javascript
import { S3Client } from "@aws-sdk/client-s3";

const client = new S3Client({
  region: "us-east-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});
```

### Client with Cognito (Browser/Frontend)

```javascript
import { S3Client } from "@aws-sdk/client-s3";
import { fromCognitoIdentityPool } from "@aws-sdk/credential-providers";

const client = new S3Client({
  region: "us-east-1",
  credentials: fromCognitoIdentityPool({
    clientConfig: { region: "us-east-1" },
    identityPoolId: "us-east-1:your-identity-pool-id",
  }),
});
```

### Client with Custom Endpoint (LocalStack, MinIO)

```javascript
import { S3Client } from "@aws-sdk/client-s3";

const client = new S3Client({
  region: "us-east-1",
  endpoint: "http://localhost:4566",
  forcePathStyle: true,
  credentials: {
    accessKeyId: "test",
    secretAccessKey: "test",
  },
});
```

---

## Bucket Operations

### List All Buckets

```javascript
import { S3Client, ListBucketsCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function listBuckets() {
  try {
    const response = await client.send(new ListBucketsCommand({}));
    console.log("Buckets:");
    response.Buckets.forEach((bucket) => {
      console.log(`  • ${bucket.Name} (Created: ${bucket.CreationDate})`);
    });
  } catch (error) {
    console.error("Error listing buckets:", error);
  }
}
```

### Create Bucket

```javascript
import {
  S3Client,
  CreateBucketCommand,
  waitUntilBucketExists
} from "@aws-sdk/client-s3";

const client = new S3Client({ region: "us-east-1" });

async function createBucket(bucketName) {
  try {
    const response = await client.send(
      new CreateBucketCommand({
        Bucket: bucketName,
      })
    );

    // Wait until bucket exists
    await waitUntilBucketExists(
      { client, maxWaitTime: 60 },
      { Bucket: bucketName }
    );

    console.log(`Bucket created: ${response.Location}`);
  } catch (error) {
    console.error("Error creating bucket:", error);
  }
}
```

### Create Bucket in Specific Region

```javascript
import { S3Client, CreateBucketCommand } from "@aws-sdk/client-s3";

const client = new S3Client({ region: "us-west-2" });

async function createBucketInRegion(bucketName, region) {
  try {
    const params = {
      Bucket: bucketName,
    };

    // us-east-1 doesn't require LocationConstraint
    if (region !== "us-east-1") {
      params.CreateBucketConfiguration = {
        LocationConstraint: region,
      };
    }

    await client.send(new CreateBucketCommand(params));
    console.log(`Bucket created in ${region}`);
  } catch (error) {
    console.error("Error creating bucket:", error);
  }
}
```

### Delete Bucket

```javascript
import {
  S3Client,
  DeleteBucketCommand,
  waitUntilBucketNotExists
} from "@aws-sdk/client-s3";

const client = new S3Client({});

async function deleteBucket(bucketName) {
  try {
    await client.send(new DeleteBucketCommand({ Bucket: bucketName }));

    await waitUntilBucketNotExists(
      { client, maxWaitTime: 60 },
      { Bucket: bucketName }
    );

    console.log(`Bucket deleted: ${bucketName}`);
  } catch (error) {
    console.error("Error deleting bucket:", error);
  }
}
```

### Check if Bucket Exists

```javascript
import { S3Client, HeadBucketCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function bucketExists(bucketName) {
  try {
    await client.send(new HeadBucketCommand({ Bucket: bucketName }));
    return true;
  } catch (error) {
    if (error.name === "NotFound") {
      return false;
    }
    throw error;
  }
}
```

---

## Object Upload Operations

### Upload File from Disk

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";
import { readFile } from "node:fs/promises";

const client = new S3Client({});

async function uploadFile(bucketName, key, filePath) {
  try {
    const fileContent = await readFile(filePath);

    const response = await client.send(
      new PutObjectCommand({
        Bucket: bucketName,
        Key: key,
        Body: fileContent,
      })
    );

    console.log("Upload successful:", response.ETag);
  } catch (error) {
    console.error("Upload error:", error);
  }
}
```

### Upload with Content Type

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";
import { readFile } from "node:fs/promises";

const client = new S3Client({});

async function uploadWithContentType(bucketName, key, filePath, contentType) {
  try {
    const fileContent = await readFile(filePath);

    await client.send(
      new PutObjectCommand({
        Bucket: bucketName,
        Key: key,
        Body: fileContent,
        ContentType: contentType,
      })
    );

    console.log(`Uploaded ${key} as ${contentType}`);
  } catch (error) {
    console.error("Upload error:", error);
  }
}

// Usage
await uploadWithContentType(
  "my-bucket",
  "images/photo.jpg",
  "./photo.jpg",
  "image/jpeg"
);
```

### Upload String/Buffer

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function uploadString(bucketName, key, content) {
  try {
    await client.send(
      new PutObjectCommand({
        Bucket: bucketName,
        Key: key,
        Body: content,
        ContentType: "text/plain",
      })
    );

    console.log(`Uploaded text to ${key}`);
  } catch (error) {
    console.error("Upload error:", error);
  }
}

// Upload JSON
async function uploadJSON(bucketName, key, data) {
  try {
    await client.send(
      new PutObjectCommand({
        Bucket: bucketName,
        Key: key,
        Body: JSON.stringify(data),
        ContentType: "application/json",
      })
    );

    console.log(`Uploaded JSON to ${key}`);
  } catch (error) {
    console.error("Upload error:", error);
  }
}
```

### Upload with Metadata

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function uploadWithMetadata(bucketName, key, body) {
  try {
    await client.send(
      new PutObjectCommand({
        Bucket: bucketName,
        Key: key,
        Body: body,
        Metadata: {
          "uploaded-by": "user123",
          "original-name": "document.pdf",
          "category": "reports",
        },
      })
    );

    console.log("Uploaded with metadata");
  } catch (error) {
    console.error("Upload error:", error);
  }
}
```

### Upload with Server-Side Encryption

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function uploadEncrypted(bucketName, key, body) {
  try {
    await client.send(
      new PutObjectCommand({
        Bucket: bucketName,
        Key: key,
        Body: body,
        ServerSideEncryption: "AES256",
      })
    );

    console.log("Uploaded with encryption");
  } catch (error) {
    console.error("Upload error:", error);
  }
}
```

### Batch Upload Multiple Files

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const client = new S3Client({});

async function uploadDirectory(bucketName, folderPath, s3Prefix = "") {
  try {
    const files = readdirSync(folderPath);

    for (const file of files) {
      const filePath = join(folderPath, file);
      const fileContent = readFileSync(filePath);
      const s3Key = s3Prefix ? `${s3Prefix}/${file}` : file;

      await client.send(
        new PutObjectCommand({
          Bucket: bucketName,
          Key: s3Key,
          Body: fileContent,
        })
      );

      console.log(`Uploaded: ${s3Key}`);
    }

    console.log(`Uploaded ${files.length} files`);
  } catch (error) {
    console.error("Batch upload error:", error);
  }
}
```

---

## Object Download Operations

### Download File to Disk

```javascript
import { S3Client, GetObjectCommand } from "@aws-sdk/client-s3";
import { writeFile } from "node:fs/promises";

const client = new S3Client({});

async function downloadFile(bucketName, key, destinationPath) {
  try {
    const response = await client.send(
      new GetObjectCommand({
        Bucket: bucketName,
        Key: key,
      })
    );

    const fileContent = await response.Body.transformToByteArray();
    await writeFile(destinationPath, fileContent);

    console.log(`Downloaded to ${destinationPath}`);
  } catch (error) {
    console.error("Download error:", error);
  }
}
```

### Download as String

```javascript
import { S3Client, GetObjectCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function downloadAsString(bucketName, key) {
  try {
    const response = await client.send(
      new GetObjectCommand({
        Bucket: bucketName,
        Key: key,
      })
    );

    const content = await response.Body.transformToString();
    return content;
  } catch (error) {
    console.error("Download error:", error);
    throw error;
  }
}

// Download JSON
async function downloadJSON(bucketName, key) {
  const content = await downloadAsString(bucketName, key);
  return JSON.parse(content);
}
```

### Download with Error Handling

```javascript
import {
  S3Client,
  GetObjectCommand,
  NoSuchKey
} from "@aws-sdk/client-s3";

const client = new S3Client({});

async function downloadSafe(bucketName, key) {
  try {
    const response = await client.send(
      new GetObjectCommand({
        Bucket: bucketName,
        Key: key,
      })
    );

    return await response.Body.transformToString();
  } catch (error) {
    if (error instanceof NoSuchKey) {
      console.error(`Key does not exist: ${key}`);
      return null;
    }
    throw error;
  }
}
```

### Download Byte Range

```javascript
import { S3Client, GetObjectCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function downloadRange(bucketName, key, start, end) {
  try {
    const response = await client.send(
      new GetObjectCommand({
        Bucket: bucketName,
        Key: key,
        Range: `bytes=${start}-${end}`,
      })
    );

    return await response.Body.transformToByteArray();
  } catch (error) {
    console.error("Download error:", error);
  }
}
```

### Get Object Metadata Only

```javascript
import { S3Client, HeadObjectCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function getObjectMetadata(bucketName, key) {
  try {
    const response = await client.send(
      new HeadObjectCommand({
        Bucket: bucketName,
        Key: key,
      })
    );

    console.log("Metadata:", {
      ContentType: response.ContentType,
      ContentLength: response.ContentLength,
      LastModified: response.LastModified,
      ETag: response.ETag,
      Metadata: response.Metadata,
    });

    return response;
  } catch (error) {
    console.error("Metadata error:", error);
  }
}
```

---

## List Objects

### List All Objects (Paginated)

```javascript
import { S3Client, paginateListObjectsV2 } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function listAllObjects(bucketName) {
  try {
    const paginator = paginateListObjectsV2(
      { client },
      { Bucket: bucketName }
    );

    let totalObjects = 0;

    for await (const page of paginator) {
      if (page.Contents) {
        page.Contents.forEach((obj) => {
          console.log(`  • ${obj.Key} (${obj.Size} bytes)`);
          totalObjects++;
        });
      }
    }

    console.log(`\nTotal objects: ${totalObjects}`);
  } catch (error) {
    console.error("List error:", error);
  }
}
```

### List Objects with Prefix

```javascript
import { S3Client, paginateListObjectsV2 } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function listObjectsWithPrefix(bucketName, prefix) {
  try {
    const paginator = paginateListObjectsV2(
      { client },
      {
        Bucket: bucketName,
        Prefix: prefix,
      }
    );

    for await (const page of paginator) {
      if (page.Contents) {
        page.Contents.forEach((obj) => {
          console.log(`  • ${obj.Key}`);
        });
      }
    }
  } catch (error) {
    console.error("List error:", error);
  }
}
```

### List Objects with Custom Page Size

```javascript
import { S3Client, paginateListObjectsV2 } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function listObjectsPaged(bucketName, pageSize = 10) {
  try {
    const paginator = paginateListObjectsV2(
      { client, pageSize },
      { Bucket: bucketName }
    );

    let pageNumber = 1;

    for await (const page of paginator) {
      console.log(`\nPage ${pageNumber}:`);

      if (page.Contents) {
        page.Contents.forEach((obj) => {
          console.log(`  • ${obj.Key}`);
        });
      }

      pageNumber++;
    }
  } catch (error) {
    console.error("List error:", error);
  }
}
```

### List Objects (Non-Paginated)

```javascript
import { S3Client, ListObjectsV2Command } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function listObjectsSimple(bucketName, maxKeys = 1000) {
  try {
    const response = await client.send(
      new ListObjectsV2Command({
        Bucket: bucketName,
        MaxKeys: maxKeys,
      })
    );

    if (response.Contents) {
      response.Contents.forEach((obj) => {
        console.log(`  • ${obj.Key}`);
      });
    }

    console.log(`Listed ${response.KeyCount} objects`);
    console.log(`Truncated: ${response.IsTruncated}`);
  } catch (error) {
    console.error("List error:", error);
  }
}
```

### List Objects in Folder Structure

```javascript
import { S3Client, ListObjectsV2Command } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function listFolder(bucketName, prefix) {
  try {
    const response = await client.send(
      new ListObjectsV2Command({
        Bucket: bucketName,
        Prefix: prefix,
        Delimiter: "/",
      })
    );

    console.log("Folders:");
    if (response.CommonPrefixes) {
      response.CommonPrefixes.forEach((cp) => {
        console.log(`  📁 ${cp.Prefix}`);
      });
    }

    console.log("\nFiles:");
    if (response.Contents) {
      response.Contents.forEach((obj) => {
        console.log(`  📄 ${obj.Key}`);
      });
    }
  } catch (error) {
    console.error("List error:", error);
  }
}
```

---

## Delete Operations

### Delete Single Object

```javascript
import {
  S3Client,
  DeleteObjectCommand,
  waitUntilObjectNotExists
} from "@aws-sdk/client-s3";

const client = new S3Client({});

async function deleteObject(bucketName, key) {
  try {
    await client.send(
      new DeleteObjectCommand({
        Bucket: bucketName,
        Key: key,
      })
    );

    await waitUntilObjectNotExists(
      { client, maxWaitTime: 60 },
      { Bucket: bucketName, Key: key }
    );

    console.log(`Deleted: ${key}`);
  } catch (error) {
    console.error("Delete error:", error);
  }
}
```

### Delete Multiple Objects (Batch)

```javascript
import { S3Client, DeleteObjectsCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function deleteMultipleObjects(bucketName, keys) {
  try {
    const response = await client.send(
      new DeleteObjectsCommand({
        Bucket: bucketName,
        Delete: {
          Objects: keys.map((key) => ({ Key: key })),
        },
      })
    );

    if (response.Deleted) {
      console.log(`Deleted ${response.Deleted.length} objects:`);
      response.Deleted.forEach((obj) => {
        console.log(`  • ${obj.Key}`);
      });
    }

    if (response.Errors) {
      console.error("Errors:", response.Errors);
    }
  } catch (error) {
    console.error("Batch delete error:", error);
  }
}

// Usage
await deleteMultipleObjects("my-bucket", [
  "file1.txt",
  "file2.txt",
  "folder/file3.txt",
]);
```

### Delete All Objects with Prefix

```javascript
import {
  S3Client,
  ListObjectsV2Command,
  DeleteObjectsCommand
} from "@aws-sdk/client-s3";

const client = new S3Client({});

async function deleteObjectsWithPrefix(bucketName, prefix) {
  try {
    const listResponse = await client.send(
      new ListObjectsV2Command({
        Bucket: bucketName,
        Prefix: prefix,
      })
    );

    if (!listResponse.Contents || listResponse.Contents.length === 0) {
      console.log("No objects to delete");
      return;
    }

    const deleteResponse = await client.send(
      new DeleteObjectsCommand({
        Bucket: bucketName,
        Delete: {
          Objects: listResponse.Contents.map((obj) => ({ Key: obj.Key })),
        },
      })
    );

    console.log(`Deleted ${deleteResponse.Deleted.length} objects`);
  } catch (error) {
    console.error("Delete error:", error);
  }
}
```

### Empty and Delete Bucket

```javascript
import {
  S3Client,
  ListObjectsV2Command,
  DeleteObjectsCommand,
  DeleteBucketCommand
} from "@aws-sdk/client-s3";

const client = new S3Client({});

async function emptyAndDeleteBucket(bucketName) {
  try {
    // List all objects
    const listResponse = await client.send(
      new ListObjectsV2Command({ Bucket: bucketName })
    );

    // Delete all objects if any exist
    if (listResponse.Contents && listResponse.Contents.length > 0) {
      await client.send(
        new DeleteObjectsCommand({
          Bucket: bucketName,
          Delete: {
            Objects: listResponse.Contents.map((obj) => ({ Key: obj.Key })),
          },
        })
      );

      console.log(`Deleted ${listResponse.Contents.length} objects`);
    }

    // Delete the bucket
    await client.send(new DeleteBucketCommand({ Bucket: bucketName }));
    console.log(`Bucket deleted: ${bucketName}`);
  } catch (error) {
    console.error("Error:", error);
  }
}
```

---

## Copy and Move Operations

### Copy Object

```javascript
import { S3Client, CopyObjectCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function copyObject(sourceBucket, sourceKey, destBucket, destKey) {
  try {
    await client.send(
      new CopyObjectCommand({
        CopySource: `${sourceBucket}/${sourceKey}`,
        Bucket: destBucket,
        Key: destKey,
      })
    );

    console.log(`Copied ${sourceKey} to ${destKey}`);
  } catch (error) {
    console.error("Copy error:", error);
  }
}
```

### Move Object (Copy + Delete)

```javascript
import {
  S3Client,
  CopyObjectCommand,
  DeleteObjectCommand
} from "@aws-sdk/client-s3";

const client = new S3Client({});

async function moveObject(sourceBucket, sourceKey, destBucket, destKey) {
  try {
    // Copy
    await client.send(
      new CopyObjectCommand({
        CopySource: `${sourceBucket}/${sourceKey}`,
        Bucket: destBucket,
        Key: destKey,
      })
    );

    // Delete source
    await client.send(
      new DeleteObjectCommand({
        Bucket: sourceBucket,
        Key: sourceKey,
      })
    );

    console.log(`Moved ${sourceKey} to ${destKey}`);
  } catch (error) {
    console.error("Move error:", error);
  }
}
```

---

## Presigned URLs

### Install Required Package

```bash
npm install @aws-sdk/s3-request-presigner
```

### Presigned URL for Download (GET)

```javascript
import { S3Client, GetObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

const client = new S3Client({ region: "us-east-1" });

async function createPresignedDownloadUrl(bucketName, key, expiresIn = 3600) {
  try {
    const command = new GetObjectCommand({
      Bucket: bucketName,
      Key: key,
    });

    const url = await getSignedUrl(client, command, { expiresIn });
    return url;
  } catch (error) {
    console.error("Presigned URL error:", error);
    throw error;
  }
}

// Usage
const url = await createPresignedDownloadUrl("my-bucket", "file.pdf", 3600);
console.log("Download URL:", url);
```

### Presigned URL for Upload (PUT)

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

const client = new S3Client({ region: "us-east-1" });

async function createPresignedUploadUrl(bucketName, key, expiresIn = 3600) {
  try {
    const command = new PutObjectCommand({
      Bucket: bucketName,
      Key: key,
    });

    const url = await getSignedUrl(client, command, { expiresIn });
    return url;
  } catch (error) {
    console.error("Presigned URL error:", error);
    throw error;
  }
}

// Usage with fetch
const uploadUrl = await createPresignedUploadUrl("my-bucket", "upload.txt");

// Client can now upload using PUT request
await fetch(uploadUrl, {
  method: "PUT",
  body: fileContent,
  headers: {
    "Content-Type": "text/plain",
  },
});
```

### Presigned URL with Content Type

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

const client = new S3Client({ region: "us-east-1" });

async function createPresignedUploadUrlWithType(
  bucketName,
  key,
  contentType,
  expiresIn = 3600
) {
  try {
    const command = new PutObjectCommand({
      Bucket: bucketName,
      Key: key,
      ContentType: contentType,
    });

    const url = await getSignedUrl(client, command, { expiresIn });
    return url;
  } catch (error) {
    console.error("Presigned URL error:", error);
    throw error;
  }
}
```

### Presigned URL with Metadata

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

const client = new S3Client({ region: "us-east-1" });

async function createPresignedUrlWithMetadata(bucketName, key) {
  try {
    const command = new PutObjectCommand({
      Bucket: bucketName,
      Key: key,
      Metadata: {
        "uploaded-by": "user123",
        "original-name": "document.pdf",
      },
    });

    const url = await getSignedUrl(client, command, { expiresIn: 3600 });
    return url;
  } catch (error) {
    console.error("Presigned URL error:", error);
    throw error;
  }
}
```

### Complete Presigned Upload Example

```javascript
import { S3Client, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

const client = new S3Client({ region: "us-east-1" });

// Server-side: Generate presigned URL
async function generateUploadUrl(fileName, fileType) {
  const command = new PutObjectCommand({
    Bucket: "my-bucket",
    Key: `uploads/${fileName}`,
    ContentType: fileType,
  });

  return await getSignedUrl(client, command, { expiresIn: 300 }); // 5 minutes
}

// Client-side: Upload using presigned URL
async function uploadUsingPresignedUrl(presignedUrl, file) {
  const response = await fetch(presignedUrl, {
    method: "PUT",
    body: file,
    headers: {
      "Content-Type": file.type,
    },
  });

  if (response.ok) {
    console.log("Upload successful");
  } else {
    console.error("Upload failed:", response.statusText);
  }
}
```

---

## Multipart Upload

### Install Required Package

```bash
npm install @aws-sdk/lib-storage
```

### Basic Multipart Upload

```javascript
import { S3Client } from "@aws-sdk/client-s3";
import { Upload } from "@aws-sdk/lib-storage";
import { createReadStream } from "node:fs";

const client = new S3Client({});

async function multipartUpload(bucketName, key, filePath) {
  try {
    const fileStream = createReadStream(filePath);

    const upload = new Upload({
      client,
      params: {
        Bucket: bucketName,
        Key: key,
        Body: fileStream,
      },
    });

    const result = await upload.done();
    console.log("Upload complete:", result.Location);
  } catch (error) {
    console.error("Multipart upload error:", error);
  }
}
```

### Multipart Upload with Progress Tracking

```javascript
import { S3Client } from "@aws-sdk/client-s3";
import { Upload } from "@aws-sdk/lib-storage";
import { createReadStream, statSync } from "node:fs";

const client = new S3Client({});

async function multipartUploadWithProgress(bucketName, key, filePath) {
  try {
    const fileStream = createReadStream(filePath);
    const fileSize = statSync(filePath).size;

    const upload = new Upload({
      client,
      params: {
        Bucket: bucketName,
        Key: key,
        Body: fileStream,
      },
    });

    upload.on("httpUploadProgress", (progress) => {
      const percentage = Math.round((progress.loaded / fileSize) * 100);
      console.log(`Upload progress: ${percentage}%`);
    });

    const result = await upload.done();
    console.log("Upload complete:", result.Location);
  } catch (error) {
    console.error("Multipart upload error:", error);
  }
}
```

### Multipart Upload with Custom Configuration

```javascript
import { S3Client } from "@aws-sdk/client-s3";
import { Upload } from "@aws-sdk/lib-storage";
import { createReadStream } from "node:fs";

const client = new S3Client({});

async function multipartUploadConfigured(bucketName, key, filePath) {
  try {
    const fileStream = createReadStream(filePath);

    const upload = new Upload({
      client,
      params: {
        Bucket: bucketName,
        Key: key,
        Body: fileStream,
      },
      queueSize: 4, // Number of concurrent uploads
      partSize: 1024 * 1024 * 5, // 5 MB parts (minimum is 5 MB)
      leavePartsOnError: false, // Clean up on error
    });

    const result = await upload.done();
    console.log("Upload complete:", result.Location);
  } catch (error) {
    console.error("Multipart upload error:", error);
  }
}
```

### Multipart Upload with Tags and Metadata

```javascript
import { S3Client } from "@aws-sdk/client-s3";
import { Upload } from "@aws-sdk/lib-storage";
import { readFileSync } from "node:fs";

const client = new S3Client({});

async function multipartUploadWithTags(bucketName, key, filePath) {
  try {
    const fileContent = readFileSync(filePath);

    const upload = new Upload({
      client,
      params: {
        Bucket: bucketName,
        Key: key,
        Body: fileContent,
        Metadata: {
          "uploaded-by": "system",
          "file-category": "large-files",
        },
      },
      tags: [
        { Key: "Environment", Value: "Production" },
        { Key: "Department", Value: "Engineering" },
      ],
    });

    const result = await upload.done();
    console.log("Upload complete:", result.Location);
  } catch (error) {
    console.error("Multipart upload error:", error);
  }
}
```

### Abort Multipart Upload

```javascript
import { S3Client } from "@aws-sdk/client-s3";
import { Upload } from "@aws-sdk/lib-storage";
import { createReadStream } from "node:fs";

const client = new S3Client({});

async function abortableMultipartUpload(bucketName, key, filePath) {
  const fileStream = createReadStream(filePath);

  const upload = new Upload({
    client,
    params: {
      Bucket: bucketName,
      Key: key,
      Body: fileStream,
    },
  });

  // Abort after 5 seconds (example)
  setTimeout(() => {
    upload.abort();
    console.log("Upload aborted");
  }, 5000);

  try {
    await upload.done();
  } catch (error) {
    if (error.name === "AbortError") {
      console.log("Upload was aborted");
    } else {
      console.error("Upload error:", error);
    }
  }
}
```

---

## Streaming Operations

### Stream Download

```javascript
import { S3Client, GetObjectCommand } from "@aws-sdk/client-s3";
import { pipeline } from "node:stream/promises";
import { createWriteStream } from "node:fs";

const client = new S3Client({});

async function streamDownload(bucketName, key, destinationPath) {
  try {
    const response = await client.send(
      new GetObjectCommand({
        Bucket: bucketName,
        Key: key,
      })
    );

    await pipeline(
      response.Body,
      createWriteStream(destinationPath)
    );

    console.log(`Streamed download to ${destinationPath}`);
  } catch (error) {
    console.error("Stream download error:", error);
  }
}
```

### Stream Upload

```javascript
import { S3Client } from "@aws-sdk/client-s3";
import { Upload } from "@aws-sdk/lib-storage";
import { createReadStream } from "node:fs";

const client = new S3Client({});

async function streamUpload(bucketName, key, filePath) {
  try {
    const fileStream = createReadStream(filePath);

    const upload = new Upload({
      client,
      params: {
        Bucket: bucketName,
        Key: key,
        Body: fileStream,
      },
    });

    await upload.done();
    console.log("Stream upload complete");
  } catch (error) {
    console.error("Stream upload error:", error);
  }
}
```

---

## Object Tagging

### Put Object Tags

```javascript
import { S3Client, PutObjectTaggingCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function tagObject(bucketName, key, tags) {
  try {
    await client.send(
      new PutObjectTaggingCommand({
        Bucket: bucketName,
        Key: key,
        Tagging: {
          TagSet: tags.map(({ Key, Value }) => ({ Key, Value })),
        },
      })
    );

    console.log(`Tagged ${key}`);
  } catch (error) {
    console.error("Tagging error:", error);
  }
}

// Usage
await tagObject("my-bucket", "file.txt", [
  { Key: "Environment", Value: "Production" },
  { Key: "Department", Value: "Engineering" },
]);
```

### Get Object Tags

```javascript
import { S3Client, GetObjectTaggingCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function getObjectTags(bucketName, key) {
  try {
    const response = await client.send(
      new GetObjectTaggingCommand({
        Bucket: bucketName,
        Key: key,
      })
    );

    console.log("Tags:");
    response.TagSet.forEach((tag) => {
      console.log(`  ${tag.Key}: ${tag.Value}`);
    });

    return response.TagSet;
  } catch (error) {
    console.error("Get tags error:", error);
  }
}
```

---

## Bucket Configuration

### Enable Versioning

```javascript
import { S3Client, PutBucketVersioningCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function enableVersioning(bucketName) {
  try {
    await client.send(
      new PutBucketVersioningCommand({
        Bucket: bucketName,
        VersioningConfiguration: {
          Status: "Enabled",
        },
      })
    );

    console.log(`Versioning enabled for ${bucketName}`);
  } catch (error) {
    console.error("Versioning error:", error);
  }
}
```

### Set Bucket CORS

```javascript
import { S3Client, PutBucketCorsCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function setBucketCORS(bucketName) {
  try {
    await client.send(
      new PutBucketCorsCommand({
        Bucket: bucketName,
        CORSConfiguration: {
          CORSRules: [
            {
              AllowedHeaders: ["*"],
              AllowedMethods: ["GET", "PUT", "POST", "DELETE"],
              AllowedOrigins: ["*"],
              ExposeHeaders: ["ETag"],
              MaxAgeSeconds: 3000,
            },
          ],
        },
      })
    );

    console.log("CORS configured");
  } catch (error) {
    console.error("CORS error:", error);
  }
}
```

### Get Bucket CORS

```javascript
import { S3Client, GetBucketCorsCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function getBucketCORS(bucketName) {
  try {
    const response = await client.send(
      new GetBucketCorsCommand({
        Bucket: bucketName,
      })
    );

    console.log("CORS Rules:", JSON.stringify(response.CORSRules, null, 2));
    return response.CORSRules;
  } catch (error) {
    console.error("Get CORS error:", error);
  }
}
```

### Set Bucket Encryption

```javascript
import { S3Client, PutBucketEncryptionCommand } from "@aws-sdk/client-s3";

const client = new S3Client({});

async function setBucketEncryption(bucketName) {
  try {
    await client.send(
      new PutBucketEncryptionCommand({
        Bucket: bucketName,
        ServerSideEncryptionConfiguration: {
          Rules: [
            {
              ApplyServerSideEncryptionByDefault: {
                SSEAlgorithm: "AES256",
              },
            },
          ],
        },
      })
    );

    console.log("Encryption enabled");
  } catch (error) {
    console.error("Encryption error:", error);
  }
}
```

---

## Error Handling

### Common Error Types

```javascript
import {
  S3Client,
  GetObjectCommand,
  NoSuchKey,
  NoSuchBucket,
} from "@aws-sdk/client-s3";

const client = new S3Client({});

async function handleErrors(bucketName, key) {
  try {
    const response = await client.send(
      new GetObjectCommand({
        Bucket: bucketName,
        Key: key,
      })
    );

    return await response.Body.transformToString();
  } catch (error) {
    if (error instanceof NoSuchKey) {
      console.error("Object not found");
    } else if (error instanceof NoSuchBucket) {
      console.error("Bucket not found");
    } else if (error.name === "AccessDenied") {
      console.error("Access denied");
    } else if (error.name === "InvalidAccessKeyId") {
      console.error("Invalid credentials");
    } else {
      console.error("Unknown error:", error);
    }

    throw error;
  }
}
```

### Retry Logic

```javascript
import { S3Client, GetObjectCommand } from "@aws-sdk/client-s3";

const client = new S3Client({
  maxAttempts: 3, // Built-in retry configuration
});

async function downloadWithRetry(bucketName, key, maxRetries = 3) {
  let lastError;

  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await client.send(
        new GetObjectCommand({
          Bucket: bucketName,
          Key: key,
        })
      );

      return await response.Body.transformToString();
    } catch (error) {
      lastError = error;
      console.log(`Attempt ${i + 1} failed, retrying...`);
      await new Promise((resolve) => setTimeout(resolve, 1000 * (i + 1)));
    }
  }

  throw lastError;
}
```

---

## Complete Example: Full S3 Operations

```javascript
import {
  S3Client,
  CreateBucketCommand,
  PutObjectCommand,
  GetObjectCommand,
  ListObjectsV2Command,
  DeleteObjectCommand,
  DeleteBucketCommand,
} from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

const client = new S3Client({ region: "us-east-1" });
const bucketName = `my-test-bucket-${Date.now()}`;

async function completeS3Example() {
  try {
    // 1. Create bucket
    console.log("Creating bucket...");
    await client.send(new CreateBucketCommand({ Bucket: bucketName }));

    // 2. Upload object
    console.log("Uploading object...");
    await client.send(
      new PutObjectCommand({
        Bucket: bucketName,
        Key: "test-file.txt",
        Body: "Hello, S3!",
        ContentType: "text/plain",
      })
    );

    // 3. Generate presigned URL
    console.log("Generating presigned URL...");
    const url = await getSignedUrl(
      client,
      new GetObjectCommand({
        Bucket: bucketName,
        Key: "test-file.txt",
      }),
      { expiresIn: 3600 }
    );
    console.log("Presigned URL:", url);

    // 4. Download object
    console.log("Downloading object...");
    const downloadResponse = await client.send(
      new GetObjectCommand({
        Bucket: bucketName,
        Key: "test-file.txt",
      })
    );
    const content = await downloadResponse.Body.transformToString();
    console.log("Content:", content);

    // 5. List objects
    console.log("Listing objects...");
    const listResponse = await client.send(
      new ListObjectsV2Command({ Bucket: bucketName })
    );
    listResponse.Contents.forEach((obj) => {
      console.log(`  • ${obj.Key}`);
    });

    // 6. Delete object
    console.log("Deleting object...");
    await client.send(
      new DeleteObjectCommand({
        Bucket: bucketName,
        Key: "test-file.txt",
      })
    );

    // 7. Delete bucket
    console.log("Deleting bucket...");
    await client.send(new DeleteBucketCommand({ Bucket: bucketName }));

    console.log("Complete!");
  } catch (error) {
    console.error("Error:", error);
  }
}

completeS3Example();
```
