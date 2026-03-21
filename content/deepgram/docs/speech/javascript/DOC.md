---
name: speech
description: "Deepgram JavaScript SDK coding guidelines for speech recognition, text-to-speech, and audio intelligence"
metadata:
  languages: "javascript"
  versions: "4.11.2"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "deepgram,speech,transcription,tts,audio"
---

# Deepgram JavaScript SDK Coding Guidelines

You are a Deepgram API coding expert. Help me with writing code using the Deepgram JavaScript SDK for speech recognition, text-to-speech, voice agents, and text intelligence.

Please follow the following guidelines when generating code.

You can find the official SDK documentation and code samples here:
https://developers.deepgram.com/

## Golden Rule: Use the Current Official SDK

Always use the official Deepgram JavaScript SDK, which is the standard library for all Deepgram API interactions.

- **Library Name:** Deepgram JavaScript SDK
- **NPM Package:** `@deepgram/sdk`
- **Minimum Node.js Version:** 18.0.0

**Installation:**

- **Correct:** `npm install @deepgram/sdk` 

**APIs and Usage:**

- **Correct:** `import { createClient } from "@deepgram/sdk"`
- **Correct:** `const deepgram = createClient(DEEPGRAM_API_KEY)`
- **Incorrect:** Using any other import patterns or client creation methods

## Initialization and API Key

The `@deepgram/sdk` library requires creating a client instance for all API calls.

- Always use `createClient()` to create an instance
- Set your Deepgram API key as the first parameter
- Get your free API key from: https://console.deepgram.com/signup?jump=keys

```javascript
import { createClient } from "@deepgram/sdk";

const deepgram = createClient("your-deepgram-api-key");
```

## Models

### Speech Recognition Models

By default, use the following models for speech recognition:

- **General Purpose (Recommended):** `nova-3` or `nova-3-general`
- **Medical Applications:** `nova-3-medical` or `nova-2-medical`
- **Meeting Transcription:** `nova-2-meeting` or `enhanced-meeting`
- **Phone Call Audio:** `nova-2-phonecall` or `enhanced-phonecall`
- **Cost-Effective Option:** `nova` or `enhanced`
- **Highest Accuracy:** `nova-3` (latest generation)

### Text-to-Speech Models

For text-to-speech, use the Aura model series:

- **Recommended Default:** `aura-2-thalia-en`
- **Available Voices:** All `aura-2-*` models for various voice characteristics
- **Legacy Support:** `aura-*` models (first generation)

## Basic Speech Recognition (Transcription)

### Synchronous Transcription - Remote Files

```javascript
import { createClient } from "@deepgram/sdk";

const deepgram = createClient("your-api-key");

const { result, error } = await deepgram.listen.prerecorded.transcribeUrl(
  { url: "https://example.com/audio.wav" },
  { model: "nova-3" }
);

if (error) {
  console.error("Transcription error:", error);
} else {
  console.log("Transcript:", result);
}
```

### Synchronous Transcription - Local Files

```javascript
import fs from "fs";

// Using file stream
const { result, error } = await deepgram.listen.prerecorded.transcribeFile(
  fs.createReadStream("./audio.wav"),
  { model: "nova-3" }
);

// Using file buffer
const { result, error } = await deepgram.listen.prerecorded.transcribeFile(
  fs.readFileSync("./audio.wav"),
  { model: "nova-3" }
);
```

### Live Streaming Transcription

```javascript
const dgConnection = deepgram.listen.live({ model: "nova-3" });

dgConnection.on(LiveTranscriptionEvents.Open, () => {
  dgConnection.on(LiveTranscriptionEvents.Transcript, (data) => {
    console.log(data);
  });

  // Send audio data
  dgConnection.send(audioData);
});
```

## Text-to-Speech

### REST API (One-off requests)

```javascript
const { result, error } = await deepgram.speak.request(
  { text: "Hello, how are you today?" }, 
  { model: "aura-2-thalia-en" }
);
```

### WebSocket (Streaming)

```javascript
const dgConnection = deepgram.speak.live({ model: "aura-2-thalia-en" });

dgConnection.on(LiveTTSEvents.Open, () => {
  dgConnection.sendText("Hello world");
  dgConnection.flush(); // Important: flush after sending text
});
```

## Voice Agent

```javascript
const agent = deepgram.agent();

agent.on(AgentEvents.Open, () => {
  agent.configure({
    audio: {
      input: { encoding: "linear16", sample_rate: 16000 },
      output: { encoding: "linear16", container: "wav", sample_rate: 24000 }
    },
    agent: {
      listen: { model: "nova-3" },
      speak: { model: "aura-2-thalia-en" },
      think: {
        provider: { type: "anthropic" },
        model: "claude-3-haiku-20240307",
        instructions: "You are a helpful AI assistant."
      }
    }
  });
});

agent.on(AgentEvents.Audio, (audio) => {
  // Handle audio response
});
```

## Error Handling

The SDK uses a consistent error handling pattern with `DeepgramResponse`:

**Error Types:**

- `DeepgramError`: Base error class
- `DeepgramApiError`: API-related errors with HTTP status codes
- `DeepgramUnknownError`: Unexpected errors
- `DeepgramVersionError`: SDK version compatibility errors

```javascript
const { result, error } = await deepgram.listen.prerecorded.transcribeUrl(
  { url: "invalid-url" },
  { model: "nova-3" }
);

if (error) {
  if (error instanceof DeepgramApiError) {
    console.error(`API Error (${error.status}): ${error.message}`);
  } else {
    console.error("Unexpected error:", error.message);
  }
  return;
}

// Process successful result
console.log(result);
```

## Configuration and Advanced Features

### Scoped Configuration

The SDK supports namespace-specific configurations:

```javascript
// Global configuration
const deepgram = createClient("api-key", {
  global: { 
    fetch: { options: { url: "https://api.beta.deepgram.com" } } 
  }
});

// Transcription-specific configuration
const deepgram = createClient("api-key", {
  listen: { 
    fetch: { options: { url: "http://localhost:8080" } } 
  }
});
```

### Important Transcription Options

- `model`: Speech recognition model to use
- `language`: Language code (e.g., "en", "es", "fr")
- `punctuate`: Add punctuation to transcript
- `diarize`: Speaker identification
- `smart_format`: Automatic formatting improvements
- `keyterm`: Keyword detection (Nova 3 only)

### Keyterm Detection (Nova 3 Only)

```javascript
const { result, error } = await deepgram.listen.prerecorded.transcribeUrl(
  { url: "audio.wav" },
  { 
    model: "nova-3",
    keyterm: ["urgent", "deadline", "meeting"]
  }
);
```

## Common Mistakes to Avoid

- **Don't** use deprecated models like older versions without explicit need
- **Don't** forget to handle both `result` and `error` in responses
- **Don't** use keyterm detection with non-Nova-3 models
- **Don't** forget to call `flush()` when using live text-to-speech
- **Don't** hardcode API keys in your source code - use environment variables
- **Always** check for errors before processing results

## Browser Support

The SDK works in browsers with UMD and ESM support:

```html
<!-- UMD -->
<script src="https://cdn.jsdelivr.net/npm/@deepgram/sdk"></script>
<script>
  const { createClient } = deepgram;
  const client = createClient("api-key");
</script>

<!-- ESM -->
<script type="module">
  import { createClient } from "https://cdn.jsdelivr.net/npm/@deepgram/sdk/+esm";
  const client = createClient("api-key");
</script>
```

## Additional Features

### Text Intelligence

```javascript
const { result, error } = await deepgram.read.analyzeText(
  { text: "Your text content here" },
  { language: "en", topics: true, sentiment: true }
);
```

### Captions Generation

```javascript
import { webvtt, srt } from "@deepgram/captions";

// After getting transcription result
const vttOutput = webvtt(result);
const srtOutput = srt(result);
```

## Useful Links

- Documentation: https://developers.deepgram.com/
- API Reference: https://developers.deepgram.com/reference/
- Models Guide: https://developers.deepgram.com/docs/model
- Getting API Keys: https://console.deepgram.com/signup?jump=keys

## Notes

- The SDK strictly follows semantic versioning
- Always use documented interfaces to ensure compatibility
- The SDK supports both Node.js (18+) and browser environments
- For production applications, implement proper error handling and logging
- Consider using environment variables for API key management

### Citations

```json
  "name": "@deepgram/sdk",
  "engines": {
    "node": ">=18.0.0"
  },
npm install @deepgram/sdk
```


### UMD

You can now use plain `<script>`s to import deepgram from CDNs, like:

<script src="https://cdn.jsdelivr.net/npm/@deepgram/sdk"></script>


or even:

```html
<script src="https://unpkg.com/@deepgram/sdk"></script>
```

Then you can use it from a global deepgram variable:

```html
<script>
  const { createClient } = deepgram;
  const _deepgram = createClient("deepgram-api-key");

  console.log("Deepgram Instance: ", _deepgram);
  // ...
</script>
```

### ESM

You can now use type="module" `<script>`s to import deepgram from CDNs, like:

```html
<script type="module">
  import { createClient } from "https://cdn.jsdelivr.net/npm/@deepgram/sdk/+esm";
  const deepgram = createClient("deepgram-api-key");

  console.log("Deepgram Instance: ", deepgram);
  // ...
</script>
```

```typescript
import { createClient } from "@deepgram/sdk";
// - or -
const { createClient } = require("@deepgram/sdk");
```

```typescript
const deepgram = createClient(DEEPGRAM_API_KEY);
```

 To access the Deepgram API you will need a [free Deepgram API Key](https://console.deepgram.com/signup?jump=keys).


The SDK supports scoped configuration. You'll be able to configure various aspects of each namespace of the SDK from the initialization. Below outlines a flexible and customizable configuration system for the Deepgram SDK. Here's how the namespace configuration works:

### 1. Global Defaults

- The `global` namespace serves as the foundational configuration applicable across all other namespaces unless overridden.
- Includes general settings like URL and headers applicable for all API calls.
- If no specific configurations are provided for other namespaces, the `global` defaults are used.

### 2. Namespace-specific Configurations

- Each namespace (`listen`, `manage`, `onprem`, `read`, `speak`) can have its specific configurations which override the `global` settings within their respective scopes.
- Allows for detailed control over different parts of the application interacting with various Deepgram API endpoints.

### 3. Transport Options

- Configurations for both `fetch` and `websocket` can be specified under each namespace, allowing different transport mechanisms for different operations.
- For example, the `fetch` configuration can have its own URL and proxy settings distinct from the `websocket`.
- The generic interfaces define a structure for transport options which include a client (like a `fetch` or `WebSocket` instance) and associated options (like headers, URL, proxy settings).

This configuration system enables robust customization where defaults provide a foundation, but every aspect of the client's interaction with the API can be finely controlled and tailored to specific needs through namespace-specific settings. This enhances the maintainability and scalability of the application by localizing configurations to their relevant contexts.




```js
const { result, error } = await deepgram.listen.prerecorded.transcribeUrl(
  {
    url: "https://dpgr.am/spacewalk.wav",
  },
  {
    model: "nova",
  }
);

const { result, error } = await deepgram.listen.prerecorded.transcribeFile(
  fs.createReadStream("./examples/spacewalk.wav"),
  {
    model: "nova",
  }
);
```

or

```js
const { result, error } = await deepgram.listen.prerecorded.transcribeFile(
  fs.readFileSync("./examples/spacewalk.wav"),
  {
    model: "nova",
  }
);

const dgConnection = deepgram.listen.live({ model: "nova" });

dgConnection.on(LiveTranscriptionEvents.Open, () => {
  dgConnection.on(LiveTranscriptionEvents.Transcript, (data) => {
    console.log(data);
  });

  source.addListener("got-some-audio", async (event) => {
    dgConnection.send(event.raw_audio_data);
  });
});

```
```js
import { webvtt /* , srt */ } from "@deepgram/captions";

const { result, error } = await deepgram.listen.prerecorded.transcribeUrl(
  {
    url: "https://dpgr.am/spacewalk.wav",
  },
  {
    model: "nova",
  }
);

const vttOutput = webvtt(result);
// const srtOutput = srt(result);

```
```js
import { createClient } from "@deepgram/sdk";
import { AgentEvents } from "@deepgram/sdk";

const deepgram = createClient(DEEPGRAM_API_KEY);

// Create an agent connection
const agent = deepgram.agent();

// Set up event handlers
agent.on(AgentEvents.Open, () => {
  console.log("Connection opened");

  // Configure the agent once connection is established
  agent.configure({
    audio: {
      input: {
        encoding: "linear16",
        sampleRate: 16000,
      },
      output: {
        encoding: "linear16",
        container: "wav",
        sampleRate: 24000,
      },
    },
    agent: {
      listen: {
        model: "nova-3",
      },
      speak: {
        model: "aura-2-thalia-en",
      },
      think: {
        provider: {
          type: "anthropic",
        },
        model: "claude-3-haiku-20240307",
        instructions: "You are a helpful AI assistant. Keep responses brief and friendly.",
      },
    },
  });
});

// Handle agent responses
agent.on(AgentEvents.AgentStartedSpeaking, (data) => {
  console.log("Agent started speaking:", data["total_latency"]);
});

agent.on(AgentEvents.ConversationText, (message) => {
  console.log(`${message.role} said: ${message.content}`);
});

agent.on(AgentEvents.Audio, (audio) => {
  // Handle audio data from the agent
  playAudio(audio); // Your audio playback implementation
});

agent.on(AgentEvents.Error, (error) => {
  console.error("Error:", error);
});

agent.on(AgentEvents.Close, () => {
  console.log("Connection closed");
});

// Send audio data
function sendAudioData(audioData) {
  agent.send(audioData);
}

// Keep the connection alive
setInterval(() => {
  agent.keepAlive();
}, 8000);


const { result } = await deepgram.speak.request({ text }, { model: "aura-2-thalia-en" });



const dgConnection = deepgram.speak.live({ model: "aura-2-thalia-en" });

dgConnection.on(LiveTTSEvents.Open, () => {
  console.log("Connection opened");

  // Send text data for TTS synthesis
  dgConnection.sendText(text);

  // Send Flush message to the server after sending the text
  dgConnection.flush();

  dgConnection.on(LiveTTSEvents.Close, () => {
    console.log("Connection closed");
  });
});



const { result, error } = await deepgram.read.analyzeText(
  { text },
  { language: "en", topics: true, sentiment: true }
);
```



Older SDK versions will receive Priority 1 (P1) bug support only. Security issues, both in our code and dependencies, are promptly addressed. Significant bugs without clear workarounds are also given priority attention.

We strictly follow semver, and will not introduce breaking changes to the publicly documented interfaces of the SDK. **Use internal and undocumented interfaces without pinning your version, at your own risk.**



```typescript
type ListenModel =
  | "nova-3"
  | "nova-3-general"
  | "nova-3-medical"
  | "nova-2"
  | "nova-2-meeting"
  | "nova-2-phonecall"
  | "nova-2-voicemail"
  | "nova-2-finance"
  | "nova-2-conversational"
  | "nova-2-video"
  | "nova-2-medical"
  | "nova-2-drivethru"
  | "nova-2-automotive"
  | "nova-2-atc"
  | "nova"
  | "nova-phonecall"
  | "enhanced"
  | "enhanced-meeting"
  | "enhanced-phonecall"
  | "enhanced-finance"
  | "base"
  | "base-meeting"
  | "base-phonecall"
  | "base-voicemail"
  | "base-finance"
  | "base-conversational"
  | "base-video"
  | "whisper-tiny"
  | "whisper"
  | "whisper-small"
  | "whisper-medium"
  | "whisper-large"
  | string;
```

```typescript
type SpeakModel =
  | "aura-asteria-en"
  | "aura-luna-en"
  | "aura-stella-en"
  | "aura-athena-en"
  | "aura-hera-en"
  | "aura-orion-en"
  | "aura-arcas-en"
  | "aura-perseus-en"
  | "aura-angus-en"
  | "aura-orpheus-en"
  | "aura-helios-en"
  | "aura-zeus-en"
  | "aura-2-amalthea-en"
  | "aura-2-andromeda-en"
  | "aura-2-apollo-en"
  | "aura-2-arcas-en"
  | "aura-2-aries-en"
  | "aura-2-asteria-en"
  | "aura-2-athena-en"
  | "aura-2-atlas-en"
  | "aura-2-aurora-en"
  | "aura-2-callista-en"
  | "aura-2-cordelia-en"
  | "aura-2-cora-en"
  | "aura-2-cressida-en"
  | "aura-2-delia-en"
  | "aura-2-draco-en"
  | "aura-2-electra-en"
  | "aura-2-harmonia-en"
  | "aura-2-helena-en"
  | "aura-2-hera-en"
  | "aura-2-hermes-en"
  | "aura-2-hyperion-en"
  | "aura-2-iris-en"
  | "aura-2-janus-en"
  | "aura-2-juno-en"
  | "aura-2-jupiter-en"
  | "aura-2-luna-en"
  | "aura-2-mars-en"
  | "aura-2-minerva-en"
  | "aura-2-neptune-en"
  | "aura-2-odysseus-en"
  | "aura-2-ophelia-en"
  | "aura-2-orion-en"
  | "aura-2-orpheus-en"
  | "aura-2-pandora-en"
  | "aura-2-phoebe-en"
  | "aura-2-pluto-en"
  | "aura-2-saturn-en"
  | "aura-2-selene-en"
  | "aura-2-thalia-en"
  | "aura-2-theia-en"
  | "aura-2-vesta-en"
  | "aura-2-zeus-en"
  | string;
        /**
         * Only available for Nova 3.
         * @see https://developers.deepgram.com/docs/keyterm
         */
        keyterms?: string[];
      };
```

```typescript
export type DeepgramResponse<T> = SuccessResponse<T> | ErrorResponse;

interface SuccessResponse<T> {
  result: T;
  error: null;
}

interface ErrorResponse {
  result: null;
  error: DeepgramError;
}
```

```typescript
export class DeepgramError extends Error {
  protected __dgError = true;

  constructor(message: string) {
    super(message);
    this.name = "DeepgramError";
  }
}

export function isDeepgramError(error: unknown): error is DeepgramError {
  return typeof error === "object" && error !== null && "__dgError" in error;
}

export class DeepgramApiError extends DeepgramError {
  status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "DeepgramApiError";
    this.status = status;
  }

  toJSON() {
    return {
      name: this.name,
      message: this.message,
      status: this.status,
    };
  }
}

export class DeepgramUnknownError extends DeepgramError {
  originalError: unknown;

  constructor(message: string, originalError: unknown) {
    super(message);
    this.name = "DeepgramUnknownError";
    this.originalError = originalError;
  }
}

export class DeepgramVersionError extends DeepgramError {
  constructor() {
    super(
      `You are attempting to use an old format for a newer SDK version. Read more here: https://dpgr.am/js-v3`
    );

    this.name = "DeepgramVersionError";
  }
}
```

```typescript
interface TranscriptionSchema extends Record<string, unknown> {
  /**
   * @see https://developers.deepgram.com/docs/model
   */
  model?: string;

  /**
   * @deprecated
   * @see https://developers.deepgram.com/docs/tier
   */
  tier?: string;

  /**
   * @see https://developers.deepgram.com/docs/version
   */
  version?: string;

  /**
   * @see https://developers.deepgram.com/docs/language
   */
  language?: string;

  /**
   * @see https://developers.deepgram.com/docs/punctuation
   */
  punctuate?: boolean;

  /**
   * @see https://developers.deepgram.com/docs/profanity-filter
   */
  profanity_filter?: boolean;

  /**
   * @see https://developers.deepgram.com/docs/redaction
   */
  redact?: string[] | string | boolean;

  /**
   * @see https://developers.deepgram.com/docs/diarization
   */
  diarize?: boolean;

  /**
   * @see https://developers.deepgram.com/docs/diarization
   */
  diarize_version?: string;

  /**
   * @see https://developers.deepgram.com/docs/smart-format
   */
  smart_format?: boolean;

  /**
   * @see https://developers.deepgram.com/docs/filler-words
   */
  filler_words?: boolean;

  /**
   * @see https://developers.deepgram.com/docs/multichannel
   */
  multichannel?: boolean;

  /**
   * @see https://developers.deepgram.com/docs/numerals
   * @deprecated
   */
  numerals?: boolean;

  /**
   * @see https://developers.deepgram.com/docs/search
   */
  search?: string[] | string;

  /**
   * @see https://developers.deepgram.com/docs/find-and-replace
   */
  replace?: string[] | string;

  /**
   * @see https://developers.deepgram.com/docs/callback
   */
  callback?: string;

  /**
   * @see https://developers.deepgram.com/docs/callback#results
   */
  callback_method?: "put" | "post";

  /**
   * @see https://developers.deepgram.com/docs/keywords
   */
  keywords?: string[] | string;

  /**
   * @see https://developers.deepgram.com/docs/keyterm
   */
  keyterm?: string[] | string;

  /**
   * @see https://developers.deepgram.com/docs/tagging
   */
  tag?: string[];

  /**
   * As yet unreleased.
   */
  sentiment?: boolean;

  /**
   * As yet unreleased.
   */
  intents?: boolean;

  /**
   * As yet unreleased.
   */
  custom_intent?: string[] | string;

  /**
   * As yet unreleased.
   */
  custom_intent_mode?: "strict" | "extended";

  /**
   * As yet unreleased.
   */
  topics?: boolean;

  /**
   * As yet unreleased.
   */
  custom_topic?: string[] | string;

  /**
   * As yet unreleased.
   */
  custom_topic_mode?: "strict" | "extended";

  /**
   * @see https://developers.deepgram.com/docs/extra-metadata
   */
  extra?: string[] | string;
}
```
