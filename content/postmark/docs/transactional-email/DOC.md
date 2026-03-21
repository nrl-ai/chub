---
name: transactional-email
description: "Postmark API coding guidelines for transactional email sending using the official Node.js library"
metadata:
  languages: "javascript"
  versions: "4.0.5"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "postmark,email,transactional,delivery,smtp"
---

# Postmark API Coding Guidelines (JavaScript/TypeScript)

You are a Postmark API coding expert. Help me with writing code using the official Postmark Node.js library for transactional email sending and API interactions.

You can find the official documentation and code samples here: https://postmarkapp.com/developer

## Golden Rule: Use the Correct and Current Package

Always use the official Postmark Node.js library, which provides complete support for the entire Postmark REST API.

- **Library Name:** Postmark Node.js Library
- **NPM Package:** `postmark`
- **Minimum Node.js Version:** v14.0.0
- **Repository:** https://github.com/ActiveCampaign/postmark.js
- **Documentation:** https://activecampaign.github.io/postmark.js/

**Installation:**

- **Correct:** `npm install postmark`

**APIs and Usage:**

- **Correct:** `const postmark = require('postmark')`
- **Correct:** `const client = new postmark.ServerClient(serverToken)`
- **Correct:** `const accountClient = new postmark.AccountClient(accountToken)`
- **Correct:** `await client.sendEmail(...)`
- **Correct:** `await client.sendEmailWithTemplate(...)`
- **Correct:** `await client.sendEmailBatch(...)`
- **Incorrect:** Using unofficial packages or legacy libraries

## API Keys and Authentication

Postmark uses two types of API tokens for different purposes:

### Server API Token

Used for sending emails and accessing server-specific endpoints. Set the `X-Postmark-Server-Token` header for API requests.

```javascript
const postmark = require('postmark');
const serverToken = process.env.POSTMARK_SERVER_TOKEN;
const client = new postmark.ServerClient(serverToken);
```

### Account API Token

Used for managing servers and account-level operations. Set the `X-Postmark-Account-Token` header for API requests.

```javascript
const postmark = require('postmark');
const accountToken = process.env.POSTMARK_ACCOUNT_TOKEN;
const accountClient = new postmark.AccountClient(accountToken);
```

### Environment Variable Configuration

Never hardcode API keys. Always use environment variables:

```bash
echo "export POSTMARK_SERVER_TOKEN='YOUR_SERVER_TOKEN'" > postmark.env
echo "export POSTMARK_ACCOUNT_TOKEN='YOUR_ACCOUNT_TOKEN'" >> postmark.env
echo "postmark.env" >> .gitignore
source ./postmark.env
```

```javascript
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);
```

## Prerequisites

Before sending emails with Postmark:

- Sign up for a Postmark account (free tier: 100 emails/month)
- Create a server in your Postmark account
- Verify your sender signature or domain
- Obtain your Server API token from the API Tokens tab

## Basic Email Sending

### Simple Email

Send a basic email with HTML and text body:

```javascript
const postmark = require('postmark');
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);

client.sendEmail({
    From: 'sender@example.com',
    To: 'recipient@example.com',
    Subject: 'Hello from Postmark',
    HtmlBody: '<strong>Hello!</strong> This is a test email.',
    TextBody: 'Hello! This is a test email.'
}).then(response => {
    console.log('Email sent successfully');
    console.log('Message ID:', response.MessageID);
    console.log('To:', response.To);
    console.log('Submitted at:', response.SubmittedAt);
});
```

### Using async/await

```javascript
const postmark = require('postmark');
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);

async function sendEmail() {
    try {
        const response = await client.sendEmail({
            From: 'sender@example.com',
            To: 'recipient@example.com',
            Subject: 'Hello from Postmark',
            HtmlBody: '<h1>Welcome!</h1><p>This is a test email.</p>',
            TextBody: 'Welcome! This is a test email.'
        });

        console.log('Email sent:', response.MessageID);
    } catch (error) {
        console.error('Error sending email:', error);
    }
}

sendEmail();
```

### Multiple Recipients

Send to multiple recipients in the To, Cc, or Bcc fields:

```javascript
client.sendEmail({
    From: 'sender@example.com',
    To: 'recipient1@example.com, recipient2@example.com',
    Cc: 'cc1@example.com, cc2@example.com',
    Bcc: 'bcc1@example.com, bcc2@example.com',
    Subject: 'Multiple recipients',
    HtmlBody: '<p>This email goes to multiple people.</p>',
    TextBody: 'This email goes to multiple people.'
});
```

**Note:** Maximum 50 recipients per field (To, Cc, Bcc).

### With Reply-To Address

Override the reply address:

```javascript
client.sendEmail({
    From: 'noreply@example.com',
    To: 'customer@example.com',
    ReplyTo: 'support@example.com',
    Subject: 'Customer Support',
    HtmlBody: '<p>Reply to this email for support.</p>',
    TextBody: 'Reply to this email for support.'
});
```

## Batch Email Sending

Send up to 500 emails in a single API call (maximum 50 MB payload):

```javascript
const postmark = require('postmark');
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);

client.sendEmailBatch([
    {
        From: 'sender@example.com',
        To: 'user1@example.com',
        Subject: 'Welcome User 1',
        HtmlBody: '<p>Welcome to our service!</p>',
        TextBody: 'Welcome to our service!'
    },
    {
        From: 'sender@example.com',
        To: 'user2@example.com',
        Subject: 'Welcome User 2',
        HtmlBody: '<p>Welcome to our service!</p>',
        TextBody: 'Welcome to our service!'
    },
    {
        From: 'sender@example.com',
        To: 'user3@example.com',
        Subject: 'Welcome User 3',
        HtmlBody: '<p>Welcome to our service!</p>',
        TextBody: 'Welcome to our service!'
    }
]).then(responses => {
    responses.forEach((response, index) => {
        console.log(`Email ${index + 1}:`, response.Message);
        console.log('Message ID:', response.MessageID);
    });
});
```

**Important:** Batch requests return HTTP 200 even if individual messages fail validation. Always check each response for error codes.

## Template Emails

### Send Email with Template ID

Send an email using a pre-created template:

```javascript
const postmark = require('postmark');
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);

client.sendEmailWithTemplate({
    From: 'sender@example.com',
    To: 'recipient@example.com',
    TemplateId: 123456,
    TemplateModel: {
        name: 'John Doe',
        product_name: 'Awesome App',
        action_url: 'https://example.com/verify',
        company_name: 'My Company'
    }
}).then(response => {
    console.log('Template email sent:', response.MessageID);
});
```

### Send Email with Template Alias

Use a template alias instead of numeric ID:

```javascript
client.sendEmailWithTemplate({
    From: 'sender@example.com',
    To: 'recipient@example.com',
    TemplateAlias: 'welcome-email',
    TemplateModel: {
        user_name: 'Jane Smith',
        login_url: 'https://example.com/login'
    }
});
```

### Batch Template Emails

Send multiple template-based emails:

```javascript
client.sendEmailBatchWithTemplates([
    {
        From: 'sender@example.com',
        To: 'user1@example.com',
        TemplateId: 123456,
        TemplateModel: {
            name: 'User One',
            code: 'ABC123'
        }
    },
    {
        From: 'sender@example.com',
        To: 'user2@example.com',
        TemplateAlias: 'welcome-email',
        TemplateModel: {
            name: 'User Two',
            code: 'DEF456'
        }
    }
]).then(responses => {
    responses.forEach((response, index) => {
        console.log(`Template email ${index + 1}:`, response.Message);
    });
});
```

## Advanced Email Features

### Tags and Metadata

Add tags for categorization and metadata for custom tracking:

```javascript
client.sendEmail({
    From: 'sender@example.com',
    To: 'recipient@example.com',
    Subject: 'Order Confirmation',
    HtmlBody: '<p>Your order has been confirmed.</p>',
    TextBody: 'Your order has been confirmed.',
    Tag: 'order-confirmation',
    Metadata: {
        order_id: '12345',
        customer_id: '67890',
        total_amount: '99.99'
    }
});
```

**Metadata notes:**
- Maximum 1000 characters per tag
- Metadata appears in webhooks and API responses
- Useful for tracking and filtering messages

### Tracking Opens

Enable open tracking to know when recipients open emails:

```javascript
client.sendEmail({
    From: 'sender@example.com',
    To: 'recipient@example.com',
    Subject: 'Track opens',
    HtmlBody: '<p>This email tracks opens.</p>',
    TextBody: 'This email tracks opens.',
    TrackOpens: true
});
```

### Tracking Links

Track clicks on links in your emails:

```javascript
client.sendEmail({
    From: 'sender@example.com',
    To: 'recipient@example.com',
    Subject: 'Track links',
    HtmlBody: '<p>Click <a href="https://example.com">here</a>.</p>',
    TextBody: 'Click here: https://example.com',
    TrackLinks: 'HtmlAndText'
});
```

**TrackLinks options:**
- `'None'` - No link tracking
- `'HtmlOnly'` - Track links in HTML body only
- `'TextOnly'` - Track links in text body only
- `'HtmlAndText'` - Track links in both bodies

### Message Streams

Organize emails by type using message streams:

```javascript
client.sendEmail({
    From: 'sender@example.com',
    To: 'recipient@example.com',
    Subject: 'Newsletter',
    HtmlBody: '<p>Monthly newsletter content.</p>',
    TextBody: 'Monthly newsletter content.',
    MessageStream: 'broadcasts'
});
```

**Default streams:**
- `'outbound'` - Transactional emails (default)
- `'broadcasts'` - Marketing/bulk emails
- Custom streams can be created in your Postmark account

## Attachments

### Basic Attachments

Attach files using Base64 encoding:

```javascript
const fs = require('fs');
const postmark = require('postmark');
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);

const fileContent = fs.readFileSync('/path/to/document.pdf');
const base64Content = fileContent.toString('base64');

client.sendEmail({
    From: 'sender@example.com',
    To: 'recipient@example.com',
    Subject: 'Document attached',
    HtmlBody: '<p>Please see the attached document.</p>',
    TextBody: 'Please see the attached document.',
    Attachments: [
        {
            Name: 'document.pdf',
            Content: base64Content,
            ContentType: 'application/pdf'
        }
    ]
});
```

### Using Postmark Models

Use the built-in `Attachment` model for cleaner code:

```javascript
const fs = require('fs');
const postmark = require('postmark');
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);

const message = new postmark.Models.Message(
    'sender@example.com',
    'Document attached',
    '<p>Please see the attached files.</p>',
    'Please see the attached files.',
    'recipient@example.com'
);

const attachment1 = new postmark.Models.Attachment(
    'report.txt',
    Buffer.from('Report content here').toString('base64'),
    'text/plain'
);

const attachment2 = new postmark.Models.Attachment(
    'invoice.pdf',
    fs.readFileSync('/path/to/invoice.pdf').toString('base64'),
    'application/pdf'
);

message.Attachments = [attachment1, attachment2];

client.sendEmail(message);
```

### Multiple Attachments

```javascript
const attachments = [
    {
        Name: 'image.jpg',
        Content: fs.readFileSync('/path/to/image.jpg').toString('base64'),
        ContentType: 'image/jpeg'
    },
    {
        Name: 'spreadsheet.xlsx',
        Content: fs.readFileSync('/path/to/data.xlsx').toString('base64'),
        ContentType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
    },
    {
        Name: 'notes.txt',
        Content: Buffer.from('Meeting notes...').toString('base64'),
        ContentType: 'text/plain'
    }
];

client.sendEmail({
    From: 'sender@example.com',
    To: 'recipient@example.com',
    Subject: 'Multiple attachments',
    HtmlBody: '<p>Three files attached.</p>',
    TextBody: 'Three files attached.',
    Attachments: attachments
});
```

### Inline Images

Embed images directly in HTML emails using Content ID (CID):

```javascript
const fs = require('fs');
const postmark = require('postmark');
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);

const message = new postmark.Models.Message(
    'sender@example.com',
    'Inline image example',
    '<!DOCTYPE html><html><body><h1>Logo</h1><img src="cid:logo.png"/></body></html>',
    'Logo image included.',
    'recipient@example.com'
);

const inlineImage = new postmark.Models.Attachment(
    'logo.png',
    fs.readFileSync('/path/to/logo.png').toString('base64'),
    'image/png',
    'cid:logo.png'
);

message.Attachments = [inlineImage];

client.sendEmail(message);
```

## Custom Headers

Add custom email headers:

```javascript
const postmark = require('postmark');
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);

const message = new postmark.Models.Message(
    'sender@example.com',
    'Custom headers',
    '<p>Email with custom headers.</p>',
    'Email with custom headers.',
    'recipient@example.com'
);

message.Headers = [
    { Name: 'X-Custom-Header', Value: 'custom-value' },
    { Name: 'X-Priority', Value: '1' },
    { Name: 'X-Campaign-ID', Value: 'summer-2025' }
];

client.sendEmail(message);
```

## Template Management

### Get Template

Retrieve a template by ID or alias:

```javascript
// By template ID
client.getTemplate(123456).then(template => {
    console.log('Template name:', template.Name);
    console.log('Template alias:', template.Alias);
    console.log('HTML body:', template.HtmlBody);
});

// By template alias
client.getTemplate('welcome-email').then(template => {
    console.log('Template:', template);
});
```

### Create Template

```javascript
client.createTemplate({
    Name: 'Welcome Email',
    Alias: 'welcome-email',
    Subject: 'Welcome to {{company_name}}!',
    HtmlBody: '<h1>Welcome {{user_name}}!</h1><p>Thanks for joining.</p>',
    TextBody: 'Welcome {{user_name}}! Thanks for joining.'
}).then(template => {
    console.log('Created template ID:', template.TemplateId);
});
```

### Edit Template

```javascript
client.editTemplate(123456, {
    Name: 'Updated Welcome Email',
    Subject: 'Welcome to {{company_name}}, {{user_name}}!',
    HtmlBody: '<h1>Hi {{user_name}}!</h1><p>Welcome to {{company_name}}.</p>',
    TextBody: 'Hi {{user_name}}! Welcome to {{company_name}}.'
}).then(template => {
    console.log('Updated template:', template.Name);
});
```

### Delete Template

```javascript
// By template ID
client.deleteTemplate(123456).then(response => {
    console.log('Template deleted:', response.Message);
});

// By template alias
client.deleteTemplate('old-template').then(response => {
    console.log('Template deleted:', response.Message);
});
```

### List Templates

```javascript
client.getTemplates({
    Count: 10,
    Offset: 0
}).then(result => {
    console.log('Total templates:', result.TotalCount);
    result.Templates.forEach(template => {
        console.log('Template:', template.Name, '(ID:', template.TemplateId + ')');
    });
});
```

### Validate Template

Test template rendering before sending:

```javascript
client.validateTemplate({
    Subject: 'Hello {{name}}',
    HtmlBody: '<p>Welcome {{name}} to {{company}}!</p>',
    TextBody: 'Welcome {{name}} to {{company}}!',
    TestRenderModel: {
        name: 'John',
        company: 'Acme Corp'
    }
}).then(result => {
    console.log('Rendered subject:', result.Subject.RenderedContent);
    console.log('Rendered HTML:', result.HtmlBody.RenderedContent);
    console.log('Suggested template model:', result.SuggestedTemplateModel);
});
```

## Messages API

### Get Outbound Messages

Retrieve sent messages with filtering:

```javascript
client.getOutboundMessages({
    Count: 50,
    Offset: 0,
    Recipient: 'user@example.com',
    FromEmail: 'sender@example.com',
    Tag: 'order-confirmation',
    Status: 'sent'
}).then(result => {
    console.log('Total messages:', result.TotalCount);
    result.Messages.forEach(message => {
        console.log('Message ID:', message.MessageID);
        console.log('Subject:', message.Subject);
        console.log('Status:', message.Status);
    });
});
```

### Get Message Details

```javascript
client.getOutboundMessageDetails('message-id-here').then(details => {
    console.log('From:', details.From);
    console.log('To:', details.To);
    console.log('Subject:', details.Subject);
    console.log('Status:', details.Status);
    console.log('Received at:', details.ReceivedAt);
    console.log('Message events:', details.MessageEvents);
});
```

### Get Message Dump

Retrieve full message content including headers:

```javascript
client.getOutboundMessageDump('message-id-here').then(dump => {
    console.log('Raw message:', dump.Body);
});
```

### Get Message Clicks

Track which links were clicked:

```javascript
client.getMessageClicks('message-id-here').then(result => {
    result.Clicks.forEach(click => {
        console.log('Link:', click.OriginalLink);
        console.log('Clicked at:', click.RecordedAt);
        console.log('Platform:', click.Platform);
    });
});
```

### Get Message Opens

Track when messages were opened:

```javascript
client.getMessageOpens('message-id-here').then(result => {
    result.Opens.forEach(open => {
        console.log('Opened at:', open.RecordedAt);
        console.log('Platform:', open.Platform);
        console.log('Client:', open.Client);
    });
});
```

## Inbound Email Processing

### Get Inbound Messages

Retrieve inbound emails sent to your Postmark inbox:

```javascript
client.getInboundMessages({
    Count: 50,
    Offset: 0,
    Recipient: 'inbox@inbound.example.com',
    FromEmail: 'sender@example.com'
}).then(result => {
    console.log('Total inbound messages:', result.TotalCount);
    result.InboundMessages.forEach(message => {
        console.log('From:', message.From);
        console.log('Subject:', message.Subject);
        console.log('Received:', message.ReceivedAt);
    });
});
```

### Get Inbound Message Details

```javascript
client.getInboundMessageDetails('inbound-message-id').then(message => {
    console.log('From:', message.From);
    console.log('To:', message.To);
    console.log('Subject:', message.Subject);
    console.log('HTML body:', message.HtmlBody);
    console.log('Text body:', message.TextBody);
    console.log('Attachments:', message.Attachments);
});
```

### Bypass Blocked Inbound Message

Allow specific blocked inbound messages:

```javascript
client.bypassBlockedInboundMessage('inbound-message-id').then(response => {
    console.log('Message unblocked:', response.Message);
});
```

### Retry Inbound Hook

Manually retry webhook delivery for an inbound message:

```javascript
client.retryInboundHookForMessage('inbound-message-id').then(response => {
    console.log('Hook retried:', response.Message);
});
```

## Bounce Management

### Get Bounces

Retrieve bounced emails:

```javascript
client.getBounces({
    Count: 50,
    Offset: 0,
    Type: 'HardBounce',
    EmailFilter: 'user@example.com',
    Tag: 'newsletter'
}).then(result => {
    console.log('Total bounces:', result.TotalCount);
    result.Bounces.forEach(bounce => {
        console.log('Email:', bounce.Email);
        console.log('Type:', bounce.Type);
        console.log('Bounced at:', bounce.BouncedAt);
    });
});
```

### Get Bounce Details

```javascript
client.getBounce('bounce-id').then(bounce => {
    console.log('Email:', bounce.Email);
    console.log('Type:', bounce.Type);
    console.log('Description:', bounce.Description);
    console.log('Details:', bounce.Details);
});
```

### Get Bounce Dump

Retrieve raw bounce message:

```javascript
client.getBounceDump('bounce-id').then(dump => {
    console.log('Raw bounce:', dump.Body);
});
```

### Activate Bounced Email

Reactivate a bounced email address:

```javascript
client.activateBounce('bounce-id').then(response => {
    console.log('Bounce activated:', response.Message);
});
```

## Suppressions Management

### Get Suppressions

Retrieve suppressed email addresses:

```javascript
client.getSuppressions('outbound', {
    Count: 50,
    Offset: 0,
    EmailFilter: 'user@example.com'
}).then(result => {
    console.log('Total suppressions:', result.Suppressions.length);
    result.Suppressions.forEach(suppression => {
        console.log('Email:', suppression.EmailAddress);
        console.log('Reason:', suppression.SuppressionReason);
        console.log('Created at:', suppression.CreatedAt);
    });
});
```

### Create Suppressions

Manually suppress email addresses:

```javascript
client.createSuppressions('outbound', [
    { EmailAddress: 'user1@example.com' },
    { EmailAddress: 'user2@example.com' }
]).then(result => {
    console.log('Suppressions created:', result.Suppressions.length);
});
```

### Delete Suppressions

Remove email addresses from suppression list:

```javascript
client.deleteSuppressions('outbound', [
    { EmailAddress: 'user1@example.com' },
    { EmailAddress: 'user2@example.com' }
]).then(result => {
    console.log('Suppressions deleted:', result.Suppressions.length);
});
```

## Webhooks

### Get Webhooks

Retrieve configured webhooks:

```javascript
client.getWebhooks().then(webhooks => {
    webhooks.Webhooks.forEach(webhook => {
        console.log('Webhook ID:', webhook.ID);
        console.log('URL:', webhook.Url);
        console.log('Message stream:', webhook.MessageStream);
        console.log('Triggers:', webhook.Triggers);
    });
});
```

### Create Webhook

```javascript
client.createWebhook({
    Url: 'https://example.com/webhooks/postmark',
    MessageStream: 'outbound',
    HttpAuth: {
        Username: 'webhook_user',
        Password: 'webhook_pass'
    },
    HttpHeaders: [
        { Name: 'X-Custom-Header', Value: 'value' }
    ],
    Triggers: {
        Open: { Enabled: true },
        Click: { Enabled: true },
        Delivery: { Enabled: true },
        Bounce: { Enabled: true },
        SpamComplaint: { Enabled: true }
    }
}).then(webhook => {
    console.log('Webhook created:', webhook.ID);
});
```

### Edit Webhook

```javascript
client.editWebhook(123456, {
    Url: 'https://example.com/webhooks/postmark-updated',
    Triggers: {
        Open: { Enabled: true },
        Click: { Enabled: true },
        Delivery: { Enabled: false }
    }
}).then(webhook => {
    console.log('Webhook updated:', webhook.ID);
});
```

### Delete Webhook

```javascript
client.deleteWebhook(123456).then(response => {
    console.log('Webhook deleted:', response.Message);
});
```

## Server Management (Account Client)

### Get Server

```javascript
const postmark = require('postmark');
const accountClient = new postmark.AccountClient(process.env.POSTMARK_ACCOUNT_TOKEN);

accountClient.getServer(123456).then(server => {
    console.log('Server name:', server.Name);
    console.log('Server ID:', server.ID);
    console.log('Color:', server.Color);
});
```

### Create Server

```javascript
accountClient.createServer({
    Name: 'Production Server',
    Color: 'blue'
}).then(server => {
    console.log('Server created:', server.ID);
    console.log('Server token:', server.ApiTokens);
});
```

### Edit Server

```javascript
accountClient.editServer(123456, {
    Name: 'Updated Server Name',
    Color: 'green'
}).then(server => {
    console.log('Server updated:', server.Name);
});
```

### List Servers

```javascript
accountClient.getServers({
    Count: 50,
    Offset: 0,
    Name: 'Production'
}).then(result => {
    console.log('Total servers:', result.TotalCount);
    result.Servers.forEach(server => {
        console.log('Server:', server.Name, '(ID:', server.ID + ')');
    });
});
```

## Client Configuration

### Timeout Configuration

Set custom timeout for API requests:

```javascript
const postmark = require('postmark');
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);

// Set timeout to 30 seconds (default is 30000ms)
client.setClientOptions({ timeout: 60000 });
```

### Custom Headers

Add default headers to all requests:

```javascript
client.setClientOptions({
    headers: {
        'X-Custom-Header': 'custom-value'
    }
});
```

### Use Test Token

For testing without sending actual emails:

```javascript
const client = new postmark.ServerClient('POSTMARK_API_TEST');

// This will validate the API call but not actually send the email
client.sendEmail({
    From: 'sender@example.com',
    To: 'recipient@example.com',
    Subject: 'Test email',
    HtmlBody: '<p>This is a test.</p>'
});
```

## Error Handling

Always handle errors properly when making API calls:

```javascript
const postmark = require('postmark');
const client = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN);

async function sendEmailWithErrorHandling() {
    try {
        const response = await client.sendEmail({
            From: 'sender@example.com',
            To: 'recipient@example.com',
            Subject: 'Test',
            HtmlBody: '<p>Test email</p>'
        });

        console.log('Success:', response.MessageID);
        return response;

    } catch (error) {
        // Handle specific error codes
        if (error.statusCode === 401) {
            console.error('Invalid API token');
        } else if (error.statusCode === 422) {
            console.error('Validation error:', error.message);
        } else if (error.statusCode === 429) {
            console.error('Rate limit exceeded');
        } else if (error.statusCode >= 500) {
            console.error('Server error:', error.message);
        } else {
            console.error('API error:', error.message);
        }

        console.error('Error code:', error.code);
        console.error('Error details:', error.body);

        throw error;
    }
}

sendEmailWithErrorHandling();
```

### Common Error Codes

| Status Code | Error Type | Description |
|-------------|------------|-------------|
| 401 | Unauthorized | Invalid or missing API token |
| 403 | Forbidden | Sender signature not verified |
| 422 | Unprocessable Entity | Validation error (invalid parameters) |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Postmark server error |
| 503 | Service Unavailable | Postmark temporarily unavailable |

### Batch Error Handling

When sending batch emails, check each response individually:

```javascript
const responses = await client.sendEmailBatch([
    { From: 'sender@example.com', To: 'user1@example.com', Subject: 'Test 1', HtmlBody: '<p>Test</p>' },
    { From: 'sender@example.com', To: 'invalid-email', Subject: 'Test 2', HtmlBody: '<p>Test</p>' },
    { From: 'sender@example.com', To: 'user3@example.com', Subject: 'Test 3', HtmlBody: '<p>Test</p>' }
]);

responses.forEach((response, index) => {
    if (response.ErrorCode === 0) {
        console.log(`Email ${index + 1} sent successfully:`, response.MessageID);
    } else {
        console.error(`Email ${index + 1} failed:`, response.Message);
    }
});
```

## TypeScript Support

The Postmark library includes full TypeScript definitions:

```typescript
import * as postmark from 'postmark';
import { ServerClient, Message, TemplatedMessage } from 'postmark';

const client: ServerClient = new postmark.ServerClient(process.env.POSTMARK_SERVER_TOKEN!);

interface EmailResponse {
    To: string;
    SubmittedAt: string;
    MessageID: string;
    ErrorCode: number;
    Message: string;
}

async function sendTypedEmail(): Promise<EmailResponse> {
    const message: Message = new postmark.Models.Message(
        'sender@example.com',
        'Test email',
        '<p>HTML body</p>',
        'Text body',
        'recipient@example.com'
    );

    const response = await client.sendEmail(message);
    return response;
}

async function sendTemplateEmail(): Promise<EmailResponse> {
    const templateMessage: TemplatedMessage = {
        From: 'sender@example.com',
        To: 'recipient@example.com',
        TemplateId: 123456,
        TemplateModel: {
            name: 'John Doe',
            code: 'ABC123'
        }
    };

    const response = await client.sendEmailWithTemplate(templateMessage);
    return response;
}
```

## ES6 Module Support

Use ES6 imports:

```javascript
import * as postmark from 'postmark';

const serverToken = process.env.POSTMARK_SERVER_TOKEN;
const client = new postmark.ServerClient(serverToken);

export async function sendWelcomeEmail(recipientEmail, userName) {
    return await client.sendEmailWithTemplate({
        From: 'welcome@example.com',
        To: recipientEmail,
        TemplateAlias: 'welcome-email',
        TemplateModel: {
            user_name: userName
        }
    });
}
```

## Useful Links

- **Official Documentation:** https://postmarkapp.com/developer
- **API Reference:** https://postmarkapp.com/developer/api/overview
- **Node.js Library Docs:** https://activecampaign.github.io/postmark.js/
- **GitHub Repository:** https://github.com/ActiveCampaign/postmark.js
- **NPM Package:** https://www.npmjs.com/package/postmark
- **Email API:** https://postmarkapp.com/developer/api/email-api
- **Templates API:** https://postmarkapp.com/developer/api/templates-api
- **Webhooks:** https://postmarkapp.com/developer/webhooks/webhooks-overview
- **Support:** https://postmarkapp.com/support

## Notes

- Always verify sender signatures or domains before sending emails
- Use the test token `POSTMARK_API_TEST` for development
- Batch requests can include up to 500 messages (50 MB max)
- Open tracking requires HTML body
- Link tracking works in both HTML and text bodies
- Metadata is included in webhooks and API responses
- Messages are stored for 45 days by default (configurable up to 365 days)
- Rate limits apply based on your account plan
- Always check individual responses in batch operations
