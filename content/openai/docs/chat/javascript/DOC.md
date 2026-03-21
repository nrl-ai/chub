---
name: chat
description: "OpenAI API for text generation, chat completions, streaming, function calling, vision, embeddings, and assistants"
metadata:
  languages: "javascript"
  versions: "6.27.0"
  updated-on: "2026-03-06"
  source: maintainer
  tags: "openai,chat,llm,ai"
---

# OpenAI API Coding Guidelines (JavaScript/TypeScript)

You are an OpenAI API coding expert. Help me with writing code using the OpenAI API calling the official libraries and SDKs.

## Golden Rule: Use the Correct and Current SDK

Always use the official OpenAI Node.js SDK for all OpenAI API interactions.

- **Library Name:** OpenAI Node.js SDK
- **NPM Package:** `openai`
- **JSR Package:** `@openai/openai`

**Installation:**

```bash
# NPM
npm install openai

# JSR (Deno/Node.js)
deno add jsr:@openai/openai
npx jsr add @openai/openai
```

**Import Patterns:**

```typescript
// Correct - ES6 import
import OpenAI from 'openai';

// Correct - with additional utilities
import OpenAI, { toFile } from 'openai';

// JSR import for Deno
import OpenAI from 'jsr:@openai/openai';
```

## Initialization and API Key

The OpenAI library requires creating an `OpenAI` client instance for all API calls.

```typescript
import OpenAI from 'openai';

// Uses OPENAI_API_KEY environment variable automatically
const client = new OpenAI({
  apiKey: process.env['OPENAI_API_KEY'], // This is the default and can be omitted
});

// Or pass API key directly
const client = new OpenAI({
  apiKey: 'your-api-key-here'
});
```

## Models (as of March 2026)

Default choices:
- **General Text Tasks:** `gpt-5.4` (frontier) or `gpt-4.1` (non-reasoning)
- **Complex Reasoning Tasks:** `gpt-5.4` or `gpt-5.4-pro`
- **Fast & Cost-Efficient:** `gpt-5-mini` or `gpt-4.1-mini`
- **Cheapest / Fastest:** `gpt-5-nano` or `gpt-4.1-nano`
- **Audio Processing:** `gpt-audio` or `gpt-audio-mini`
- **Vision Tasks:** `gpt-5.4` or `gpt-4.1`
- **Agentic Coding:** `gpt-5.3-codex`
- **Search (Chat Completions):** `gpt-5-search-api`, `gpt-4o-search-preview`, or `gpt-4o-mini-search-preview`

Frontier (reasoning, configurable effort):
- `gpt-5.4`, `gpt-5.4-2026-03-05`, `gpt-5.4-pro`, `gpt-5.4-pro-2026-03-05`
- `gpt-5.2`, `gpt-5.2-2025-12-11`, `gpt-5.2-pro`
- `gpt-5.1`, `gpt-5.1-2025-11-13`, `gpt-5.1-pro`
- `gpt-5`, `gpt-5-2025-08-07`, `gpt-5-pro`
- `gpt-5-mini`, `gpt-5-mini-2025-08-07`
- `gpt-5-nano`, `gpt-5-nano-2025-08-07`

Non-reasoning:
- `gpt-4.1`, `gpt-4.1-2025-04-14`
- `gpt-4.1-mini`, `gpt-4.1-mini-2025-04-14`
- `gpt-4.1-nano`, `gpt-4.1-nano-2025-04-14`

Reasoning (o-series, succeeded by GPT-5):
- `o3`, `o3-2025-04-16`, `o3-pro`, `o3-pro-2025-06-10`
- `o4-mini`, `o4-mini-2025-04-16`
- `o3-mini`, `o3-mini-2025-01-31`
- `o1`, `o1-2024-12-17`

Deep research: `o3-deep-research`, `o4-mini-deep-research`

Codex (agentic coding, Responses API only):
- `gpt-5.3-codex`, `gpt-5.2-codex`, `gpt-5.1-codex`, `gpt-5.1-codex-max`, `gpt-5.1-codex-mini`, `gpt-5-codex`

Audio chat: `gpt-audio`, `gpt-audio-2025-08-28`, `gpt-audio-mini`
Realtime: `gpt-realtime`, `gpt-realtime-2025-08-28`, `gpt-realtime-mini`
TTS: `gpt-4o-mini-tts`, `gpt-4o-mini-tts-2025-12-15`, `tts-1`, `tts-1-hd`
STT: `gpt-4o-transcribe`, `gpt-4o-mini-transcribe`, `gpt-4o-mini-transcribe-2025-12-15`, `gpt-4o-transcribe-diarize`, `whisper-1`
Image generation: `gpt-image-1.5`, `gpt-image-1.5-2025-12-16`, `gpt-image-1`, `gpt-image-1-mini`, `chatgpt-image-latest`
Embeddings: `text-embedding-3-large`, `text-embedding-3-small`, `text-embedding-ada-002`
Moderation: `omni-moderation-latest`
Search (Chat Completions only): `gpt-5-search-api`, `gpt-4o-search-preview`, `gpt-4o-mini-search-preview`

Legacy (still available): `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo`, `gpt-3.5-turbo`

Deprecated (shutdown scheduled):
- `dall-e-3`, `dall-e-2` → May 12, 2026 (use `gpt-image-1`)
- `o1-preview`, `o1-mini` → deprecated (use `o3` or `gpt-5`)
- `codex-mini-latest` → shut down Feb 12, 2026
- `chatgpt-4o-latest` → shut down Feb 17, 2026
- `gpt-4o-realtime-preview` → Mar 24, 2026 (use `gpt-realtime`)
- `gpt-4o-mini-audio-preview` → Mar 24, 2026 (use `gpt-audio-mini`)
- `gpt-4.5-preview` → deprecated
- Assistants API → sunset Aug 26, 2026 (migrate to Responses API)

## Primary APIs

### Responses API (Recommended)

The Responses API is the primary interface for text generation.

```typescript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: process.env['OPENAI_API_KEY'],
});

const response = await client.responses.create({
  model: 'gpt-5.4',
  instructions: 'You are a coding assistant that talks like a pirate',
  input: 'Are semicolons optional in JavaScript?',
});

console.log(response.output_text);
```

### Chat Completions API (Legacy but Supported)

The Chat Completions API remains fully supported for existing applications.

```typescript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: process.env['OPENAI_API_KEY'],
});

const completion = await client.chat.completions.create({
  model: 'gpt-5.4',
  messages: [
    { role: 'developer', content: 'Talk like a pirate.' },
    { role: 'user', content: 'Are semicolons optional in JavaScript?' },
  ],
});

console.log(completion.choices[0].message.content);
```

## API Resources Structure

The OpenAI client organizes endpoints into logical resource groupings:

```typescript
// Core API resources available on client
client.completions     // Text completions
client.chat           // Chat completions
client.embeddings     // Text embeddings
client.files          // File management
client.images         // Image generation
client.audio          // Audio processing
client.moderations    // Content moderation
client.models         // Model information
client.fineTuning     // Fine-tuning jobs
client.graders        // Model evaluation
```

## Streaming Responses

Both Responses and Chat Completions APIs support streaming for real-time output.

### Responses API Streaming

```typescript
import OpenAI from 'openai';

const client = new OpenAI();

const stream = await client.responses.create({
  model: 'gpt-5.4',
  input: 'Say "Sheep sleep deep" ten times fast!',
  stream: true,
});

for await (const event of stream) {
  console.log(event);
}
```

### Chat Completions Streaming

```typescript
const stream = await client.chat.completions.create({
  model: 'gpt-5.4',
  messages: [{ role: 'user', content: 'Count to 10' }],
  stream: true,
});

for await (const chunk of stream) {
  process.stdout.write(chunk.choices[0]?.delta?.content || '');
}
```

## File Uploads

The library supports multiple file upload formats for various use cases.

```typescript
import fs from 'fs';
import OpenAI, { toFile } from 'openai';

const client = new OpenAI();

// Method 1: Node.js fs.ReadStream (recommended for Node.js)
await client.files.create({
  file: fs.createReadStream('input.jsonl'),
  purpose: 'fine-tune'
});

// Method 2: Web File API
await client.files.create({
  file: new File(['my bytes'], 'input.jsonl'),
  purpose: 'fine-tune'
});

// Method 3: Fetch Response
await client.files.create({
  file: await fetch('https://somesite/input.jsonl'),
  purpose: 'fine-tune'
});

// Method 4: toFile helper utility
await client.files.create({
  file: await toFile(Buffer.from('my bytes'), 'input.jsonl'),
  purpose: 'fine-tune',
});
```

## Advanced Configuration

### Function Calling (Tools)

```typescript
const completion = await client.chat.completions.create({
  model: 'gpt-5.4',
  messages: [{ role: 'user', content: 'What is the weather like today?' }],
  tools: [
    {
      type: 'function',
      function: {
        name: 'get_current_weather',
        description: 'Get the current weather in a given location',
        parameters: {
          type: 'object',
          properties: {
            location: {
              type: 'string',
              description: 'The city and state, e.g. San Francisco, CA',
            },
            unit: { type: 'string', enum: ['celsius', 'fahrenheit'] },
          },
          required: ['location'],
        },
      },
    },
  ],
  tool_choice: 'auto',
});
```

### Temperature and Sampling Parameters

Configure model behavior using parameters in the chat completions API:

```typescript
const completion = await client.chat.completions.create({
  model: 'gpt-5.4',
  messages: [{ role: 'user', content: 'Write a creative story' }],
  temperature: 0.8,        // Higher = more creative (0-2)
  max_tokens: 1000,        // Maximum response length
  top_p: 0.9,             // Nucleus sampling
  frequency_penalty: 0.1,  // Reduce repetition
  presence_penalty: 0.1,   // Encourage new topics
});
```

### Structured Outputs (JSON Mode)

```typescript
const completion = await client.chat.completions.create({
  model: 'gpt-5.4',
  messages: [
    { role: 'user', content: 'Extract the name and age from: "John is 30 years old"' }
  ],
  response_format: {
    type: 'json_object'
  },
});

const result = JSON.parse(completion.choices[0].message.content);
```

## Error Handling

The library provides specific error types for different failure scenarios:

```typescript
import OpenAI from 'openai';

const client = new OpenAI();

try {
  const completion = await client.chat.completions.create({
    model: 'gpt-5.4',
    messages: [{ role: 'user', content: 'Hello!' }],
  });
} catch (error) {
  if (error instanceof OpenAI.RateLimitError) {
    console.log('Rate limit exceeded');
  } else if (error instanceof OpenAI.AuthenticationError) {
    console.log('Invalid API key');
  } else if (error instanceof OpenAI.APIError) {
    console.log(error.status);  // HTTP status code
    console.log(error.name);    // Error name
    console.log(error.headers); // Response headers
  } else {
    console.log('Unexpected error:', error);
  }
}
```

## Common Patterns

### Retry Logic with Exponential Backoff

```typescript
async function createCompletionWithRetry(messages: any[], maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await client.chat.completions.create({
        model: 'gpt-5.4',
        messages,
      });
    } catch (error) {
      if (error instanceof OpenAI.RateLimitError && attempt < maxRetries) {
        const delay = Math.pow(2, attempt) * 1000; // Exponential backoff
        await new Promise(resolve => setTimeout(resolve, delay));
        continue;
      }
      throw error;
    }
  }
}
```

### Conversation Management

```typescript
class ChatSession {
  private messages: OpenAI.Chat.ChatCompletionMessageParam[] = [];

  constructor(private client: OpenAI, systemPrompt?: string) {
    if (systemPrompt) {
      this.messages.push({ role: 'system', content: systemPrompt });
    }
  }

  async sendMessage(content: string) {
    this.messages.push({ role: 'user', content });

    const completion = await this.client.chat.completions.create({
      model: 'gpt-5.4',
      messages: this.messages,
    });

    const response = completion.choices[0].message;
    this.messages.push(response);

    return response.content;
  }
}
```

## Useful Links

- **Documentation:** https://platform.openai.com/docs/api-reference
- **API Keys:** https://platform.openai.com/api-keys
- **Models:** https://platform.openai.com/docs/models
- **Pricing:** https://openai.com/pricing
- **Rate Limits:** https://platform.openai.com/docs/guides/rate-limits
