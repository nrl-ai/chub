---
name: llm
description: "DeepSeek API JavaScript/TypeScript SDK coding guide for LLM chat and code generation"
metadata:
  languages: "javascript"
  versions: "1.0.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "deepseek,llm,ai,chat,code"
---

# DeepSeek API JavaScript/TypeScript SDK Coding Guide

## 1. Golden Rule

**Always use OpenAI-compatible SDKs to access the DeepSeek API:**
- Primary SDK: `openai` (Official OpenAI JavaScript SDK)
- Alternative: Any OpenAI-compatible SDK

**DeepSeek does NOT provide a dedicated JavaScript SDK.** Instead, the DeepSeek API is fully OpenAI-compatible, allowing you to use the official OpenAI SDK by simply changing the base URL and API key.

**Never use:**
- Unofficial or deprecated DeepSeek-specific packages
- Legacy API implementations
- Non-standard HTTP clients without proper error handling

**Correct Installation:**
```bash
npm install openai
```

## 2. Installation

### npm
```bash
npm install openai
```

### yarn
```bash
yarn add openai
```

### pnpm
```bash
pnpm add openai
```

**Environment Variables:**
```bash
DEEPSEEK_API_KEY=your_deepseek_api_key_here
# Get your API key from: https://platform.deepseek.com/api_keys
```

**Optional Configuration:**
```bash
DEEPSEEK_BASE_URL=https://api.deepseek.com
```

## 3. Initialization

### Basic Initialization

```javascript
import OpenAI from 'openai';

// Initialize with DeepSeek API configuration
const client = new OpenAI({
  apiKey: process.env.DEEPSEEK_API_KEY,
  baseURL: 'https://api.deepseek.com',
});
```

### With Explicit Configuration

```javascript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: 'your-deepseek-api-key',
  baseURL: 'https://api.deepseek.com',
  timeout: 30000, // 30 seconds
  maxRetries: 3,
});
```

### TypeScript Initialization

```typescript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: process.env.DEEPSEEK_API_KEY!,
  baseURL: 'https://api.deepseek.com',
});
```

## 4. Core API Surfaces

### Chat Completions (Non-Reasoning Mode)

**Minimal Example:**
```javascript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: process.env.DEEPSEEK_API_KEY,
  baseURL: 'https://api.deepseek.com',
});

async function chat() {
  const completion = await client.chat.completions.create({
    model: 'deepseek-chat',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'Hello, how are you?' }
    ],
  });

  console.log(completion.choices[0].message.content);
}

chat();
```

**Advanced Example with Parameters:**
```javascript
const completion = await client.chat.completions.create({
  model: 'deepseek-chat',
  messages: [
    { role: 'system', content: 'You are a creative writing assistant.' },
    { role: 'user', content: 'Write a short story about AI.' }
  ],
  temperature: 0.7,
  max_tokens: 2000,
  top_p: 0.9,
  frequency_penalty: 0.5,
  presence_penalty: 0.5,
  stop: ['\n\n', 'END'],
});

console.log(completion.choices[0].message.content);
```

### Reasoning Mode (DeepSeek-R1)

**Minimal Example:**
```javascript
const completion = await client.chat.completions.create({
  model: 'deepseek-reasoner',
  messages: [
    { role: 'user', content: 'Solve this math problem: What is the derivative of x^2 + 3x + 2?' }
  ],
});

// Access reasoning process
console.log('Reasoning:', completion.choices[0].message.reasoning_content);
console.log('Answer:', completion.choices[0].message.content);
```

**Advanced Reasoning Example:**
```javascript
const completion = await client.chat.completions.create({
  model: 'deepseek-reasoner',
  messages: [
    {
      role: 'system',
      content: 'You are a mathematical reasoning assistant. Show your work step by step.'
    },
    {
      role: 'user',
      content: 'A train travels 120 km in 2 hours. If it increases its speed by 20%, how long will it take to travel 180 km?'
    }
  ],
  max_tokens: 4000,
  temperature: 1.0, // Reasoning models work best with temperature 1.0
});

// The reasoning process is in reasoning_content
console.log('Thinking process:\n', completion.choices[0].message.reasoning_content);
console.log('\nFinal answer:\n', completion.choices[0].message.content);
```

### Streaming Responses

**Basic Streaming:**
```javascript
async function streamChat() {
  const stream = await client.chat.completions.create({
    model: 'deepseek-chat',
    messages: [
      { role: 'user', content: 'Write a poem about coding.' }
    ],
    stream: true,
  });

  for await (const chunk of stream) {
    const content = chunk.choices[0]?.delta?.content || '';
    process.stdout.write(content);
  }
}

streamChat();
```

**Advanced Streaming with Error Handling:**
```javascript
async function streamWithErrorHandling() {
  try {
    const stream = await client.chat.completions.create({
      model: 'deepseek-chat',
      messages: [{ role: 'user', content: 'Tell me a story.' }],
      stream: true,
      max_tokens: 1000,
    });

    let fullResponse = '';

    for await (const chunk of stream) {
      const delta = chunk.choices[0]?.delta;

      if (delta?.content) {
        fullResponse += delta.content;
        process.stdout.write(delta.content);
      }

      // Check for finish reason
      if (chunk.choices[0]?.finish_reason) {
        console.log('\n\nFinish reason:', chunk.choices[0].finish_reason);
      }
    }

    return fullResponse;
  } catch (error) {
    console.error('Streaming error:', error);
    throw error;
  }
}
```

**Streaming with Reasoning Mode:**
```javascript
async function streamReasoning() {
  const stream = await client.chat.completions.create({
    model: 'deepseek-reasoner',
    messages: [
      { role: 'user', content: 'Explain quantum entanglement.' }
    ],
    stream: true,
  });

  let reasoning = '';
  let answer = '';

  for await (const chunk of stream) {
    const delta = chunk.choices[0]?.delta;

    // Reasoning content comes first
    if (delta?.reasoning_content) {
      reasoning += delta.reasoning_content;
      console.log('[THINKING]', delta.reasoning_content);
    }

    // Final answer comes after reasoning
    if (delta?.content) {
      answer += delta.content;
      console.log('[ANSWER]', delta.content);
    }
  }
}
```

### JSON Mode (Structured Output)

**Minimal Example:**
```javascript
const completion = await client.chat.completions.create({
  model: 'deepseek-chat',
  messages: [
    {
      role: 'system',
      content: 'You are a helpful assistant. Always respond with valid JSON.'
    },
    {
      role: 'user',
      content: 'Extract the following information as JSON: Name is John, age 30, city New York.'
    }
  ],
  response_format: { type: 'json_object' },
});

const result = JSON.parse(completion.choices[0].message.content);
console.log(result);
```

**Advanced JSON Mode with Schema Validation:**
```javascript
async function structuredExtraction() {
  const completion = await client.chat.completions.create({
    model: 'deepseek-chat',
    messages: [
      {
        role: 'system',
        content: `You are a data extraction assistant. Extract information and return it as JSON with this schema:
{
  "name": "string",
  "age": "number",
  "occupation": "string",
  "skills": ["string"],
  "contact": {
    "email": "string",
    "phone": "string"
  }
}`
      },
      {
        role: 'user',
        content: 'John Doe is a 35-year-old software engineer skilled in Python, JavaScript, and DevOps. Email: john@example.com, Phone: 555-1234.'
      }
    ],
    response_format: { type: 'json_object' },
    temperature: 0.1, // Lower temperature for structured output
  });

  try {
    const data = JSON.parse(completion.choices[0].message.content);
    console.log('Extracted data:', data);
    return data;
  } catch (error) {
    console.error('Failed to parse JSON:', error);
    throw error;
  }
}
```

### Function Calling (Tool Use)

**Minimal Example:**
```javascript
const tools = [
  {
    type: 'function',
    function: {
      name: 'get_weather',
      description: 'Get the current weather for a location',
      parameters: {
        type: 'object',
        properties: {
          location: {
            type: 'string',
            description: 'The city and state, e.g. San Francisco, CA',
          },
          unit: {
            type: 'string',
            enum: ['celsius', 'fahrenheit'],
            description: 'The temperature unit',
          },
        },
        required: ['location'],
      },
    },
  },
];

const completion = await client.chat.completions.create({
  model: 'deepseek-chat',
  messages: [
    { role: 'user', content: "What's the weather in San Francisco?" }
  ],
  tools: tools,
});

// Check if the model wants to call a function
const message = completion.choices[0].message;
if (message.tool_calls) {
  console.log('Function to call:', message.tool_calls[0].function.name);
  console.log('Arguments:', message.tool_calls[0].function.arguments);
}
```

**Advanced Function Calling with Execution:**
```javascript
// Define available functions
const availableFunctions = {
  get_weather: async (location, unit = 'celsius') => {
    // Simulate weather API call
    return {
      location,
      temperature: 22,
      unit,
      condition: 'sunny',
    };
  },
  calculate: async (expression) => {
    return eval(expression); // Use safely in production!
  },
};

// Define tools
const tools = [
  {
    type: 'function',
    function: {
      name: 'get_weather',
      description: 'Get current weather',
      parameters: {
        type: 'object',
        properties: {
          location: { type: 'string' },
          unit: { type: 'string', enum: ['celsius', 'fahrenheit'] },
        },
        required: ['location'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'calculate',
      description: 'Perform mathematical calculation',
      parameters: {
        type: 'object',
        properties: {
          expression: { type: 'string', description: 'Math expression to evaluate' },
        },
        required: ['expression'],
      },
    },
  },
];

async function runConversation(userMessage) {
  const messages = [{ role: 'user', content: userMessage }];

  // First API call
  let completion = await client.chat.completions.create({
    model: 'deepseek-chat',
    messages: messages,
    tools: tools,
    tool_choice: 'auto',
  });

  let message = completion.choices[0].message;

  // Handle function calls
  if (message.tool_calls) {
    messages.push(message); // Add assistant message with tool calls

    // Execute each function call
    for (const toolCall of message.tool_calls) {
      const functionName = toolCall.function.name;
      const functionArgs = JSON.parse(toolCall.function.arguments);

      console.log(`Calling function: ${functionName}`, functionArgs);

      // Execute function
      const functionResponse = await availableFunctions[functionName](...Object.values(functionArgs));

      // Add function response to messages
      messages.push({
        role: 'tool',
        tool_call_id: toolCall.id,
        content: JSON.stringify(functionResponse),
      });
    }

    // Second API call with function results
    completion = await client.chat.completions.create({
      model: 'deepseek-chat',
      messages: messages,
    });

    message = completion.choices[0].message;
  }

  return message.content;
}

// Usage
runConversation("What's the weather in Tokyo and what's 123 * 456?").then(console.log);
```

### Context Caching (Automatic)

DeepSeek automatically caches frequently referenced contexts. No code changes required.

**Example with Repeated Context:**
```javascript
const systemPrompt = `You are an expert code reviewer. Review code for:
1. Security vulnerabilities
2. Performance issues
3. Code style and best practices
4. Potential bugs
5. Maintainability concerns

Provide detailed feedback with examples.`;

// First request - cache miss
const review1 = await client.chat.completions.create({
  model: 'deepseek-chat',
  messages: [
    { role: 'system', content: systemPrompt },
    { role: 'user', content: 'Review this code: function add(a, b) { return a + b; }' }
  ],
});

console.log('Cache stats:', review1.usage);
// prompt_cache_miss_tokens: 150 (example)
// prompt_cache_hit_tokens: 0

// Second request with same system prompt - cache hit
const review2 = await client.chat.completions.create({
  model: 'deepseek-chat',
  messages: [
    { role: 'system', content: systemPrompt }, // Same system prompt
    { role: 'user', content: 'Review this code: function multiply(x, y) { return x * y; }' }
  ],
});

console.log('Cache stats:', review2.usage);
// prompt_cache_miss_tokens: 20
// prompt_cache_hit_tokens: 150 (cached system prompt)
```

**Monitoring Cache Performance:**
```javascript
function analyzeCachePerformance(response) {
  const usage = response.usage;
  const cacheHitRate = usage.prompt_cache_hit_tokens /
    (usage.prompt_cache_hit_tokens + usage.prompt_cache_miss_tokens);

  console.log('Total prompt tokens:', usage.prompt_tokens);
  console.log('Cache hits:', usage.prompt_cache_hit_tokens);
  console.log('Cache misses:', usage.prompt_cache_miss_tokens);
  console.log('Cache hit rate:', `${(cacheHitRate * 100).toFixed(2)}%`);
  console.log('Completion tokens:', usage.completion_tokens);

  // Calculate cost savings (approximate)
  const cachedCost = (usage.prompt_cache_hit_tokens / 1_000_000) * 0.014;
  const uncachedCost = (usage.prompt_cache_miss_tokens / 1_000_000) * 0.14;
  console.log(`Cost: $${(cachedCost + uncachedCost).toFixed(6)}`);
}
```

## 5. Advanced Features

### Multi-Turn Conversations

```javascript
async function conversation() {
  const messages = [
    { role: 'system', content: 'You are a helpful coding assistant.' }
  ];

  // First turn
  messages.push({ role: 'user', content: 'How do I read a file in Node.js?' });

  let completion = await client.chat.completions.create({
    model: 'deepseek-chat',
    messages: messages,
  });

  messages.push({ role: 'assistant', content: completion.choices[0].message.content });
  console.log('Assistant:', completion.choices[0].message.content);

  // Second turn
  messages.push({ role: 'user', content: 'Can you show me an async example?' });

  completion = await client.chat.completions.create({
    model: 'deepseek-chat',
    messages: messages,
  });

  messages.push({ role: 'assistant', content: completion.choices[0].message.content });
  console.log('Assistant:', completion.choices[0].message.content);

  return messages;
}
```

### Token Counting and Management

```javascript
// Approximate token counting (use tiktoken library for accuracy)
import { encoding_for_model } from 'tiktoken';

function countTokens(text, model = 'gpt-4') {
  const encoding = encoding_for_model(model);
  const tokens = encoding.encode(text);
  encoding.free();
  return tokens.length;
}

async function managedCompletion(messages, maxResponseTokens = 2000) {
  // Count input tokens
  const inputText = messages.map(m => m.content).join('\n');
  const estimatedInputTokens = countTokens(inputText);

  console.log(`Estimated input tokens: ${estimatedInputTokens}`);

  // Ensure we don't exceed context window (128K for deepseek-chat)
  if (estimatedInputTokens > 120000) {
    console.warn('Input approaching context limit!');
  }

  const completion = await client.chat.completions.create({
    model: 'deepseek-chat',
    messages: messages,
    max_tokens: maxResponseTokens,
  });

  // Actual usage from API
  console.log('Actual usage:', completion.usage);

  return completion;
}
```

### Custom Request Headers

```javascript
const client = new OpenAI({
  apiKey: process.env.DEEPSEEK_API_KEY,
  baseURL: 'https://api.deepseek.com',
  defaultHeaders: {
    'X-Custom-Header': 'value',
  },
});
```

### Timeout and Retry Configuration

```javascript
const client = new OpenAI({
  apiKey: process.env.DEEPSEEK_API_KEY,
  baseURL: 'https://api.deepseek.com',
  timeout: 60000, // 60 seconds
  maxRetries: 5,
});

// Per-request override
const completion = await client.chat.completions.create(
  {
    model: 'deepseek-chat',
    messages: [{ role: 'user', content: 'Hello' }],
  },
  {
    timeout: 30000,
    maxRetries: 2,
  }
);
```

## 6. TypeScript Usage

### Type-Safe Completions

```typescript
import OpenAI from 'openai';
import type { ChatCompletionMessageParam, ChatCompletion } from 'openai/resources/chat';

const client = new OpenAI({
  apiKey: process.env.DEEPSEEK_API_KEY!,
  baseURL: 'https://api.deepseek.com',
});

async function typedChat(): Promise<string> {
  const messages: ChatCompletionMessageParam[] = [
    { role: 'system', content: 'You are helpful.' },
    { role: 'user', content: 'Hello' },
  ];

  const completion: ChatCompletion = await client.chat.completions.create({
    model: 'deepseek-chat',
    messages: messages,
  });

  return completion.choices[0].message.content || '';
}
```

### Typed Function Calling

```typescript
import OpenAI from 'openai';
import type { ChatCompletionTool } from 'openai/resources/chat';

interface WeatherParams {
  location: string;
  unit?: 'celsius' | 'fahrenheit';
}

interface WeatherResult {
  location: string;
  temperature: number;
  unit: string;
  condition: string;
}

const tools: ChatCompletionTool[] = [
  {
    type: 'function',
    function: {
      name: 'get_weather',
      description: 'Get current weather',
      parameters: {
        type: 'object',
        properties: {
          location: { type: 'string' },
          unit: { type: 'string', enum: ['celsius', 'fahrenheit'] },
        },
        required: ['location'],
      },
    },
  },
];

async function getWeather(params: WeatherParams): Promise<WeatherResult> {
  // Implementation
  return {
    location: params.location,
    temperature: 22,
    unit: params.unit || 'celsius',
    condition: 'sunny',
  };
}
```

### Strict Type Checking for Streaming

```typescript
import type { Stream } from 'openai/streaming';
import type { ChatCompletionChunk } from 'openai/resources/chat';

async function typedStream(): Promise<void> {
  const stream: Stream<ChatCompletionChunk> = await client.chat.completions.create({
    model: 'deepseek-chat',
    messages: [{ role: 'user', content: 'Hello' }],
    stream: true,
  });

  for await (const chunk of stream) {
    const content: string | undefined = chunk.choices[0]?.delta?.content;
    if (content) {
      process.stdout.write(content);
    }
  }
}
```

## 7. Best Practices

### Error Handling

```javascript
import OpenAI from 'openai';

async function robustCompletion(messages) {
  try {
    const completion = await client.chat.completions.create({
      model: 'deepseek-chat',
      messages: messages,
    });

    return completion.choices[0].message.content;
  } catch (error) {
    if (error instanceof OpenAI.APIError) {
      console.error('API Error:', error.status, error.message);
      console.error('Request ID:', error.headers?.['x-request-id']);

      // Handle specific error codes
      if (error.status === 429) {
        console.error('Rate limit exceeded. Implementing backoff...');
        // Implement exponential backoff
        await new Promise(resolve => setTimeout(resolve, 5000));
        return robustCompletion(messages); // Retry
      } else if (error.status === 401) {
        console.error('Authentication failed. Check API key.');
        throw error;
      } else if (error.status >= 500) {
        console.error('Server error. Retrying...');
        await new Promise(resolve => setTimeout(resolve, 2000));
        return robustCompletion(messages);
      }
    } else if (error.code === 'ECONNABORTED') {
      console.error('Request timeout');
    } else {
      console.error('Unexpected error:', error);
    }

    throw error;
  }
}
```

### Rate Limiting and Throttling

```javascript
class DeepSeekClient {
  constructor(apiKey) {
    this.client = new OpenAI({
      apiKey: apiKey,
      baseURL: 'https://api.deepseek.com',
    });
    this.requestQueue = [];
    this.processing = false;
  }

  async queueRequest(params) {
    return new Promise((resolve, reject) => {
      this.requestQueue.push({ params, resolve, reject });
      this.processQueue();
    });
  }

  async processQueue() {
    if (this.processing || this.requestQueue.length === 0) return;

    this.processing = true;

    while (this.requestQueue.length > 0) {
      const { params, resolve, reject } = this.requestQueue.shift();

      try {
        const result = await this.client.chat.completions.create(params);
        resolve(result);
      } catch (error) {
        if (error.status === 429) {
          // Re-queue with delay
          await new Promise(r => setTimeout(r, 5000));
          this.requestQueue.unshift({ params, resolve, reject });
        } else {
          reject(error);
        }
      }

      // Add small delay between requests
      await new Promise(r => setTimeout(r, 100));
    }

    this.processing = false;
  }
}

// Usage
const deepseek = new DeepSeekClient(process.env.DEEPSEEK_API_KEY);
const result = await deepseek.queueRequest({
  model: 'deepseek-chat',
  messages: [{ role: 'user', content: 'Hello' }],
});
```

### Optimizing for Context Caching

```javascript
// Structure prompts to maximize cache hits
class ConversationManager {
  constructor() {
    this.systemPrompt = ''; // Static system prompt for caching
    this.conversationHistory = [];
  }

  setSystemPrompt(prompt) {
    // Set once and reuse for cache efficiency
    this.systemPrompt = prompt;
  }

  async chat(userMessage) {
    const messages = [
      { role: 'system', content: this.systemPrompt }, // Cached
      ...this.conversationHistory,
      { role: 'user', content: userMessage }
    ];

    const completion = await client.chat.completions.create({
      model: 'deepseek-chat',
      messages: messages,
    });

    const assistantMessage = completion.choices[0].message.content;

    // Update history
    this.conversationHistory.push(
      { role: 'user', content: userMessage },
      { role: 'assistant', content: assistantMessage }
    );

    // Keep history manageable (trim if too long)
    if (this.conversationHistory.length > 20) {
      this.conversationHistory = this.conversationHistory.slice(-20);
    }

    return {
      message: assistantMessage,
      usage: completion.usage,
    };
  }
}
```

### Model Selection Strategy

```javascript
function selectModel(taskType, budget = 'normal') {
  const modelMap = {
    // Use deepseek-chat for general tasks
    'chat': 'deepseek-chat',
    'coding': 'deepseek-chat',
    'writing': 'deepseek-chat',
    'translation': 'deepseek-chat',

    // Use deepseek-reasoner for complex reasoning
    'math': 'deepseek-reasoner',
    'logic': 'deepseek-reasoner',
    'analysis': 'deepseek-reasoner',
    'problem-solving': 'deepseek-reasoner',
  };

  return modelMap[taskType] || 'deepseek-chat';
}

async function smartCompletion(taskType, message) {
  const model = selectModel(taskType);

  console.log(`Using model: ${model} for task: ${taskType}`);

  const completion = await client.chat.completions.create({
    model: model,
    messages: [{ role: 'user', content: message }],
  });

  return completion.choices[0].message.content;
}
```

### Security Best Practices

```javascript
// NEVER expose API keys in client-side code
// NEVER commit API keys to version control
// ALWAYS use environment variables

// Use a proxy server for client applications
// Example Express.js proxy endpoint:
import express from 'express';

const app = express();
app.use(express.json());

app.post('/api/chat', async (req, res) => {
  try {
    // Validate and sanitize input
    const { messages } = req.body;

    if (!Array.isArray(messages) || messages.length === 0) {
      return res.status(400).json({ error: 'Invalid messages' });
    }

    // Rate limiting per user/IP (use express-rate-limit)
    // Authentication (use your auth system)

    const completion = await client.chat.completions.create({
      model: 'deepseek-chat',
      messages: messages,
      max_tokens: 2000, // Limit to control costs
    });

    res.json({ message: completion.choices[0].message.content });
  } catch (error) {
    console.error('Proxy error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});
```

## 8. Production Checklist

### Version Management
- Pin exact SDK version in package.json: `"openai": "4.73.0"`
- Test thoroughly before upgrading OpenAI SDK versions
- Monitor OpenAI SDK changelog for breaking changes
- Document which SDK version works with DeepSeek API

### Environment Configuration
```javascript
// config/deepseek.js
export const deepseekConfig = {
  apiKey: process.env.DEEPSEEK_API_KEY,
  baseURL: process.env.DEEPSEEK_BASE_URL || 'https://api.deepseek.com',
  timeout: parseInt(process.env.DEEPSEEK_TIMEOUT || '60000'),
  maxRetries: parseInt(process.env.DEEPSEEK_MAX_RETRIES || '3'),
};

// Validate configuration on startup
export function validateConfig() {
  if (!deepseekConfig.apiKey) {
    throw new Error('DEEPSEEK_API_KEY environment variable is required');
  }

  console.log('DeepSeek configuration validated');
}
```

### Error Handling Checklist
- ✅ Implement exponential backoff for rate limits (429 errors)
- ✅ Handle network timeouts gracefully
- ✅ Log all errors with request IDs for debugging
- ✅ Implement circuit breaker for repeated failures
- ✅ Provide fallback responses for critical paths
- ✅ Monitor error rates and set up alerts

### Monitoring and Logging
```javascript
import winston from 'winston';

const logger = winston.createLogger({
  level: 'info',
  format: winston.format.json(),
  transports: [
    new winston.transports.File({ filename: 'deepseek-error.log', level: 'error' }),
    new winston.transports.File({ filename: 'deepseek-combined.log' }),
  ],
});

async function monitoredCompletion(messages) {
  const startTime = Date.now();

  try {
    const completion = await client.chat.completions.create({
      model: 'deepseek-chat',
      messages: messages,
    });

    const duration = Date.now() - startTime;

    logger.info('Completion success', {
      duration,
      model: 'deepseek-chat',
      inputTokens: completion.usage.prompt_tokens,
      outputTokens: completion.usage.completion_tokens,
      cacheHits: completion.usage.prompt_cache_hit_tokens,
      cacheMisses: completion.usage.prompt_cache_miss_tokens,
    });

    return completion;
  } catch (error) {
    const duration = Date.now() - startTime;

    logger.error('Completion failed', {
      duration,
      error: error.message,
      status: error.status,
      requestId: error.headers?.['x-request-id'],
    });

    throw error;
  }
}
```

### Cost Tracking
```javascript
class CostTracker {
  constructor() {
    this.totalInputTokens = 0;
    this.totalOutputTokens = 0;
    this.totalCachedTokens = 0;
  }

  trackUsage(usage) {
    this.totalInputTokens += usage.prompt_cache_miss_tokens || 0;
    this.totalOutputTokens += usage.completion_tokens || 0;
    this.totalCachedTokens += usage.prompt_cache_hit_tokens || 0;
  }

  estimateCost() {
    // Pricing as of 2025
    const inputCost = (this.totalInputTokens / 1_000_000) * 0.14;
    const outputCost = (this.totalOutputTokens / 1_000_000) * 0.28;
    const cachedCost = (this.totalCachedTokens / 1_000_000) * 0.014;

    return {
      input: inputCost,
      output: outputCost,
      cached: cachedCost,
      total: inputCost + outputCost + cachedCost,
    };
  }

  report() {
    const cost = this.estimateCost();
    console.log('Cost Report:');
    console.log(`  Input tokens: ${this.totalInputTokens.toLocaleString()} ($${cost.input.toFixed(4)})`);
    console.log(`  Output tokens: ${this.totalOutputTokens.toLocaleString()} ($${cost.output.toFixed(4)})`);
    console.log(`  Cached tokens: ${this.totalCachedTokens.toLocaleString()} ($${cost.cached.toFixed(4)})`);
    console.log(`  Total cost: $${cost.total.toFixed(4)}`);
  }
}

// Usage
const tracker = new CostTracker();

const completion = await client.chat.completions.create({
  model: 'deepseek-chat',
  messages: [{ role: 'user', content: 'Hello' }],
});

tracker.trackUsage(completion.usage);
tracker.report();
```

### Testing Strategy
```javascript
// __tests__/deepseek.test.js
import { jest } from '@jest/globals';
import OpenAI from 'openai';

// Mock the OpenAI client for testing
jest.mock('openai');

describe('DeepSeek Integration', () => {
  let client;

  beforeEach(() => {
    client = new OpenAI({
      apiKey: 'test-key',
      baseURL: 'https://api.deepseek.com',
    });
  });

  test('should create completion', async () => {
    const mockResponse = {
      choices: [{ message: { content: 'Test response' } }],
      usage: { prompt_tokens: 10, completion_tokens: 5 },
    };

    client.chat.completions.create.mockResolvedValue(mockResponse);

    const result = await client.chat.completions.create({
      model: 'deepseek-chat',
      messages: [{ role: 'user', content: 'Test' }],
    });

    expect(result.choices[0].message.content).toBe('Test response');
  });

  test('should handle errors', async () => {
    const mockError = new Error('API Error');
    mockError.status = 429;

    client.chat.completions.create.mockRejectedValue(mockError);

    await expect(
      client.chat.completions.create({
        model: 'deepseek-chat',
        messages: [{ role: 'user', content: 'Test' }],
      })
    ).rejects.toThrow('API Error');
  });
});
```

### Performance Optimization
- Use streaming for long responses to improve perceived latency
- Implement request batching where appropriate
- Cache static system prompts for context caching benefits
- Monitor and optimize token usage
- Use appropriate max_tokens limits to control costs
- Consider using connection pooling for high-volume applications

### Security Checklist
- ✅ Store API keys in environment variables or secure vaults
- ✅ Never commit API keys to version control
- ✅ Use .gitignore for .env files
- ✅ Implement rate limiting on your API endpoints
- ✅ Validate and sanitize all user inputs
- ✅ Use HTTPS for all API communications
- ✅ Implement proper authentication for your application
- ✅ Monitor for unusual API usage patterns
- ✅ Set up alerts for cost thresholds
- ✅ Regularly rotate API keys

### Deployment Checklist
- ✅ Environment variables configured correctly
- ✅ Error logging and monitoring in place
- ✅ Rate limiting implemented
- ✅ Cost tracking configured
- ✅ Backup error handling for API failures
- ✅ Health check endpoints for API connectivity
- ✅ Documentation for API usage and limits
- ✅ Incident response plan for API outages

## Available Models

To get the current list of available models, pricing, and specifications, use the DeepSeek API's models endpoint:

### List All Models via CLI

```bash
curl https://api.deepseek.com/models \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY"
```

### List Models via JavaScript

```javascript
async function listModels() {
  const response = await fetch('https://api.deepseek.com/models', {
    headers: {
      'Authorization': `Bearer ${process.env.DEEPSEEK_API_KEY}`,
    },
  });

  const data = await response.json();
  console.log(JSON.stringify(data, null, 2));
  return data;
}
```

The response will include model IDs, context lengths, pricing, and capabilities. Primary models include `deepseek-chat` and `deepseek-reasoner`.
