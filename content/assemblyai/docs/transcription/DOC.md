---
name: transcription
description: "AssemblyAI JavaScript SDK coding guide for speech-to-text transcription"
metadata:
  languages: "javascript"
  versions: "4.18.4"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "assemblyai,transcription,speech-to-text,audio,ai"
---

# AssemblyAI JavaScript SDK Coding Guide

## 1. Golden Rule

**Always use the official AssemblyAI JavaScript SDK:**

Package name: `assemblyai`

To check the latest version, run:
```bash
npm view assemblyai version
```

**Never use deprecated, unofficial, or direct HTTP clients.** The official `assemblyai` package is the only supported JavaScript/TypeScript SDK maintained by AssemblyAI. It provides type-safe interfaces, automatic polling, streaming support, and simplified error handling.

## 2. Installation

### npm
```bash
npm install assemblyai
```

### yarn
```bash
yarn add assemblyai
```

### pnpm
```bash
pnpm add assemblyai
```

**Environment Variables (Required):**
```bash
ASSEMBLYAI_API_KEY=your_api_key_here
```

**Get your API key:**

Sign up at https://www.assemblyai.com, navigate to the AssemblyAI Dashboard, and copy your API key from the "Your API Key" section.

## 3. Initialization

### Basic Client Initialization
```javascript
import { AssemblyAI } from "assemblyai";

const client = new AssemblyAI({
  apiKey: process.env.ASSEMBLYAI_API_KEY,
});
```

### With Custom Configuration
```javascript
const client = new AssemblyAI({
  apiKey: process.env.ASSEMBLYAI_API_KEY,
  // Optional: Custom base URL (for testing or proxy)
  baseUrl: "https://api.assemblyai.com",
});
```

**Authentication Best Practice:**
The SDK reads from the `ASSEMBLYAI_API_KEY` environment variable by default. Always store API keys in environment variables, never hardcode them in source code.

## 4. Core API Surfaces

### 4.1 Async Transcription

Async transcription processes pre-recorded audio files. The SDK automatically polls the transcription status until completion.

**Minimal Example (Transcribe from URL):**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/audio.mp3",
});

console.log(transcript.text);
```

**Minimal Example (Transcribe from Local File):**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "./path/to/local/audio.mp3",
});

console.log(transcript.text);
```

**Advanced Example with Audio Intelligence:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/meeting.mp3",
  // Speaker identification
  speaker_labels: true,
  speakers_expected: 2,

  // Audio intelligence features
  sentiment_analysis: true,
  entity_detection: true,
  auto_highlights: true,
  content_safety: true,
  iab_categories: true,

  // Language detection and translation
  language_code: "en",
  language_detection: false,

  // Formatting options
  format_text: true,
  punctuate: true,
  disfluencies: false,

  // Custom vocabulary
  word_boost: ["AssemblyAI", "JavaScript", "Node.js"],
  boost_param: "high",
});

// Access results
console.log(transcript.text);
console.log(transcript.sentiment_analysis_results);
console.log(transcript.entities);
console.log(transcript.auto_highlights_result);
```

**Transcribe Without Waiting (Webhook or Manual Polling):**
```javascript
// Submit transcription without waiting
const transcript = await client.transcripts.submit({
  audio: "https://example.com/audio.mp3",
  webhook_url: "https://your-domain.com/webhook",
});

console.log(transcript.id); // Use this ID to check status later

// Later, retrieve the transcript
const result = await client.transcripts.get(transcript.id);
if (result.status === "completed") {
  console.log(result.text);
} else if (result.status === "error") {
  console.error(result.error);
}
```

### 4.2 Real-Time Streaming Transcription

Real-time transcription provides live speech-to-text with low latency (300ms P50 on final transcripts).

**Minimal Example:**
```javascript
import { RealtimeTranscriber } from "assemblyai";

const transcriber = new RealtimeTranscriber({
  apiKey: process.env.ASSEMBLYAI_API_KEY,
  sampleRate: 16000,
});

transcriber.on("open", ({ sessionId }) => {
  console.log("Session opened:", sessionId);
});

transcriber.on("transcript", (transcript) => {
  if (!transcript.text) return;

  if (transcript.message_type === "FinalTranscript") {
    console.log("Final:", transcript.text);
  } else {
    console.log("Partial:", transcript.text);
  }
});

transcriber.on("error", (error) => {
  console.error("Error:", error);
});

transcriber.on("close", (code, reason) => {
  console.log("Session closed:", code, reason);
});

// Connect to the streaming service
await transcriber.connect();

// Send audio data (16-bit PCM audio)
transcriber.sendAudio(audioChunk);

// Close when done
await transcriber.close();
```

**Advanced Example with Audio Intelligence:**
```javascript
const transcriber = new RealtimeTranscriber({
  apiKey: process.env.ASSEMBLYAI_API_KEY,
  sampleRate: 16000,
  encoding: "pcm_s16le",

  // Enable word-level timestamps
  word_boost: ["AssemblyAI", "JavaScript"],

  // Disable automatic punctuation
  disable_partial_transcripts: false,

  // End utterance silence threshold
  end_utterance_silence_threshold: 500,
});

transcriber.on("transcript", (transcript) => {
  if (transcript.message_type === "FinalTranscript") {
    console.log("Final transcript:", transcript.text);
    console.log("Confidence:", transcript.confidence);
    console.log("Words:", transcript.words);
  } else if (transcript.message_type === "PartialTranscript") {
    console.log("Partial:", transcript.text);
  }
});

await transcriber.connect();

// Stream from microphone or file
const stream = getMicrophoneStream(); // Your audio source
stream.on("data", (chunk) => {
  transcriber.sendAudio(chunk);
});
```

**Streaming with Temporary Token (Client-Side Security):**
```javascript
// Server-side: Generate temporary token
const token = await client.realtime.createTemporaryToken({
  expires_in: 3600, // 1 hour
});
// Send token to client

// Client-side: Use token instead of API key
import { RealtimeTranscriber } from "assemblyai";

const transcriber = new RealtimeTranscriber({
  token: temporaryToken, // Use token instead of apiKey
  sampleRate: 16000,
});
```

### 4.3 PII Redaction

Automatically detect and redact 23 categories of Personally Identifiable Information (PII).

**Minimal Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/audio.mp3",
  redact_pii: true,
  redact_pii_policies: ["us_social_security_number", "credit_card_number"],
});

console.log(transcript.text); // PII replaced with [REDACTED]
```

**Advanced Example with Audio Redaction:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/sensitive-call.mp3",

  // Text redaction
  redact_pii: true,
  redact_pii_policies: [
    "us_social_security_number",
    "credit_card_number",
    "credit_card_cvv",
    "date_of_birth",
    "drivers_license",
    "email_address",
    "phone_number",
    "medical_condition",
    "medication",
    "person_name",
  ],
  redact_pii_sub: "hash", // Options: "hash" or "entity_name"

  // Audio redaction
  redact_pii_audio: true,
  redact_pii_audio_quality: "mp3", // Options: "mp3" or "wav"
});

console.log(transcript.text); // Text with PII redacted
console.log(transcript.redacted_audio_url); // URL to redacted audio file

// Download redacted audio
if (transcript.redacted_audio_url) {
  const redactedAudio = await fetch(transcript.redacted_audio_url);
  // Process redacted audio
}
```

**Available PII Categories:**

To see the complete list of 23+ available PII categories, check the TypeScript types or run:
```bash
npm info assemblyai | grep -A 50 "redact_pii_policies"
```

Or view the official documentation at https://www.assemblyai.com/docs/audio-intelligence/pii-redaction for the most up-to-date list of supported PII categories including SSN, credit cards, medical information, and more.

### 4.4 Speaker Diarization

Identify and label different speakers in audio (up to 10 speakers).

**Minimal Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/meeting.mp3",
  speaker_labels: true,
});

// Access speaker-labeled words
for (const utterance of transcript.utterances) {
  console.log(`Speaker ${utterance.speaker}: ${utterance.text}`);
}
```

**Advanced Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/conference-call.mp3",
  speaker_labels: true,
  speakers_expected: 4, // Hint for better accuracy

  // Combine with other features
  sentiment_analysis: true,
  entity_detection: true,
});

// Analyze per-speaker sentiment
const speakerSentiments = {};
for (const result of transcript.sentiment_analysis_results) {
  const speaker = result.speaker;
  if (!speakerSentiments[speaker]) {
    speakerSentiments[speaker] = [];
  }
  speakerSentiments[speaker].push(result.sentiment);
}

console.log("Speaker sentiments:", speakerSentiments);

// Get full utterances with timestamps
for (const utterance of transcript.utterances) {
  console.log({
    speaker: utterance.speaker,
    text: utterance.text,
    start: utterance.start,
    end: utterance.end,
    confidence: utterance.confidence,
  });
}
```

### 4.5 LeMUR - Apply LLMs to Audio

LeMUR (Leveraging Large Language Models to Understand Recordings) applies powerful LLMs to transcribed speech for summarization, Q&A, action items, and custom tasks.

**Question & Answer:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/meeting.mp3",
});

const { response } = await client.lemur.question({
  transcript_ids: [transcript.id],
  questions: [
    {
      question: "What were the main action items discussed?",
      answer_format: "bullet points",
    },
    {
      question: "Who was assigned to work on the new feature?",
    },
  ],
});

console.log(response);
```

**Summarization:**
```javascript
const { response } = await client.lemur.summary({
  transcript_ids: [transcript.id],
  answer_format: "one sentence",
  context: "This is a sales call between a sales rep and a potential customer",
});

console.log("Summary:", response);
```

**Action Items:**
```javascript
const { response } = await client.lemur.actionItems({
  transcript_ids: [transcript.id],
});

console.log("Action items:", response);
```

**Custom Task:**
```javascript
const { response } = await client.lemur.task({
  transcript_ids: [transcript.id],
  prompt: "Identify the key risks discussed in this meeting and categorize them as technical, financial, or operational. Return as JSON with structure: {technical: [], financial: [], operational: []}",
  final_model: "anthropic/claude-3-5-sonnet",
});

console.log(response);
```

**Advanced LeMUR with Multiple Transcripts:**
```javascript
// Transcribe multiple files
const transcript1 = await client.transcripts.transcribe({
  audio: "https://example.com/day1.mp3",
});
const transcript2 = await client.transcripts.transcribe({
  audio: "https://example.com/day2.mp3",
});

// Analyze all transcripts together
const { response } = await client.lemur.task({
  transcript_ids: [transcript1.id, transcript2.id],
  prompt: `Analyze these two conference sessions and:
    1. Identify common themes
    2. List key speakers and their main points
    3. Suggest follow-up topics for the next conference

    Return as structured markdown.`,
  final_model: "anthropic/claude-3-5-sonnet",
  max_output_size: 4000,
  temperature: 0.3,
});

console.log(response);
```

**Available LeMUR Models:**

To see the current list of supported LLM models, check the official documentation at https://www.assemblyai.com/docs/api-reference/lemur or inspect the TypeScript types. As of 2025, Claude 3.5 Sonnet is recommended for most tasks.

### 4.6 Content Moderation

Detect sensitive or inappropriate content across multiple categories.

**Minimal Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/content.mp3",
  content_safety: true,
});

for (const result of transcript.content_safety_labels.results) {
  console.log(`${result.text}: ${result.labels.map(l => l.label).join(", ")}`);
}
```

**Advanced Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/user-generated-content.mp3",
  content_safety: true,

  // Set confidence threshold
  content_safety_confidence: 75,
});

// Filter high-severity content
const severityThreshold = 0.8;
const flaggedContent = transcript.content_safety_labels.results.filter(
  (result) => result.labels.some(label => label.confidence > severityThreshold)
);

if (flaggedContent.length > 0) {
  console.log("Flagged content detected:");
  for (const item of flaggedContent) {
    console.log({
      text: item.text,
      timestamp: `${item.timestamp.start}ms - ${item.timestamp.end}ms`,
      labels: item.labels.map(l => ({
        category: l.label,
        confidence: l.confidence,
        severity: l.severity,
      })),
    });
  }
}

// Summary of content safety
console.log("Summary:", transcript.content_safety_labels.summary);
```

**Content Safety Categories:**

To see the complete list of content moderation categories, check the official documentation at https://www.assemblyai.com/docs/audio-intelligence/content-moderation or inspect the SDK TypeScript types for `ContentSafetyLabel`. Categories include violence, profanity, NSFW, hate speech, and more.

### 4.7 Topic Detection (IAB Classification)

Automatically classify content using IAB (Interactive Advertising Bureau) taxonomy.

**Minimal Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/podcast.mp3",
  iab_categories: true,
});

console.log("Detected topics:", transcript.iab_categories_result.summary);
```

**Advanced Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/podcast.mp3",
  iab_categories: true,
});

// Get overall summary
const topTopics = Object.entries(transcript.iab_categories_result.summary)
  .sort((a, b) => b[1] - a[1])
  .slice(0, 5);

console.log("Top 5 topics:");
for (const [topic, relevance] of topTopics) {
  console.log(`${topic}: ${(relevance * 100).toFixed(1)}%`);
}

// Get segment-level topics
for (const result of transcript.iab_categories_result.results) {
  console.log({
    text: result.text,
    timestamp: `${result.timestamp.start}ms - ${result.timestamp.end}ms`,
    topics: result.labels.map(l => ({
      category: l.label,
      relevance: l.relevance,
    })),
  });
}
```

### 4.8 Sentiment Analysis

Analyze emotional tone throughout the audio.

**Minimal Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/customer-call.mp3",
  sentiment_analysis: true,
});

for (const result of transcript.sentiment_analysis_results) {
  console.log(`"${result.text}": ${result.sentiment}`);
}
```

**Advanced Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/customer-call.mp3",
  sentiment_analysis: true,
  speaker_labels: true,
});

// Calculate sentiment distribution
const sentimentCounts = { POSITIVE: 0, NEUTRAL: 0, NEGATIVE: 0 };
for (const result of transcript.sentiment_analysis_results) {
  sentimentCounts[result.sentiment]++;
}

const total = transcript.sentiment_analysis_results.length;
console.log("Sentiment distribution:");
console.log(`Positive: ${(sentimentCounts.POSITIVE / total * 100).toFixed(1)}%`);
console.log(`Neutral: ${(sentimentCounts.NEUTRAL / total * 100).toFixed(1)}%`);
console.log(`Negative: ${(sentimentCounts.NEGATIVE / total * 100).toFixed(1)}%`);

// Analyze sentiment by speaker
const speakerSentiments = {};
for (const result of transcript.sentiment_analysis_results) {
  if (!result.speaker) continue;

  if (!speakerSentiments[result.speaker]) {
    speakerSentiments[result.speaker] = { POSITIVE: 0, NEUTRAL: 0, NEGATIVE: 0 };
  }
  speakerSentiments[result.speaker][result.sentiment]++;
}

console.log("\nSentiment by speaker:", speakerSentiments);
```

### 4.9 Entity Detection

Extract and classify named entities from transcripts.

**Minimal Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/news.mp3",
  entity_detection: true,
});

for (const entity of transcript.entities) {
  console.log(`${entity.text} (${entity.entity_type})`);
}
```

**Advanced Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/business-meeting.mp3",
  entity_detection: true,
});

// Group entities by type
const entitiesByType = {};
for (const entity of transcript.entities) {
  if (!entitiesByType[entity.entity_type]) {
    entitiesByType[entity.entity_type] = [];
  }
  entitiesByType[entity.entity_type].push({
    text: entity.text,
    start: entity.start,
    end: entity.end,
  });
}

console.log("Entities by type:");
for (const [type, entities] of Object.entries(entitiesByType)) {
  console.log(`\n${type}:`);
  const uniqueEntities = [...new Set(entities.map(e => e.text))];
  console.log(uniqueEntities.join(", "));
}
```

**Entity Types:**

To see the complete list of detectable entity types, check the official documentation at https://www.assemblyai.com/docs/audio-intelligence/entity-detection or inspect the SDK TypeScript types for `EntityType`. Supported types include person names, locations, organizations, dates, medical terms, and more.

### 4.10 Auto Highlights

Automatically identify and extract key phrases and important moments.

**Minimal Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/meeting.mp3",
  auto_highlights: true,
});

for (const highlight of transcript.auto_highlights_result.results) {
  console.log(`${highlight.text} (rank: ${highlight.rank})`);
}
```

**Advanced Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/podcast.mp3",
  auto_highlights: true,
});

// Get top highlights
const topHighlights = transcript.auto_highlights_result.results
  .sort((a, b) => b.rank - a.rank)
  .slice(0, 10);

console.log("Top 10 highlights:");
for (const highlight of topHighlights) {
  console.log({
    text: highlight.text,
    rank: highlight.rank,
    count: highlight.count,
    timestamps: highlight.timestamps.map(t => ({
      start: t.start,
      end: t.end,
    })),
  });
}
```

### 4.11 Export Subtitles (SRT/VTT)

Export completed transcripts as subtitle files for video.

**Export as SRT:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/video-audio.mp3",
});

const srt = await client.transcripts.subtitles(transcript.id, "srt");
console.log(srt);

// Or with custom chars per caption
const srtCustom = await client.transcripts.subtitles(
  transcript.id,
  "srt",
  40 // chars_per_caption
);
```

**Export as VTT:**
```javascript
const vtt = await client.transcripts.subtitles(transcript.id, "vtt");
console.log(vtt);

// Save to file
import fs from "fs";
fs.writeFileSync("subtitles.vtt", vtt);
```

### 4.12 Paragraphs and Sentences

Retrieve transcripts automatically segmented into paragraphs or sentences.

**Get Paragraphs:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/lecture.mp3",
});

const paragraphs = await client.transcripts.paragraphs(transcript.id);

for (const para of paragraphs.paragraphs) {
  console.log(`[${para.start}ms - ${para.end}ms]`);
  console.log(para.text);
  console.log("---");
}
```

**Get Sentences:**
```javascript
const sentences = await client.transcripts.sentences(transcript.id);

for (const sentence of sentences.sentences) {
  console.log({
    text: sentence.text,
    start: sentence.start,
    end: sentence.end,
    confidence: sentence.confidence,
    speaker: sentence.speaker,
  });
}
```

### 4.13 Word-Level Timestamps

Access precise word-level timing information.

**Example:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/speech.mp3",
});

for (const word of transcript.words) {
  console.log({
    text: word.text,
    start: word.start, // milliseconds
    end: word.end,
    confidence: word.confidence,
    speaker: word.speaker, // if speaker_labels enabled
  });
}

// Find specific words
const targetWord = "important";
const occurrences = transcript.words.filter(w =>
  w.text.toLowerCase() === targetWord.toLowerCase()
);

console.log(`"${targetWord}" appears ${occurrences.length} times at:`);
for (const word of occurrences) {
  console.log(`  ${word.start}ms - ${word.end}ms`);
}
```

## 5. Advanced Features

### 5.1 Language Detection and Multilingual Support

**Automatic Language Detection:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/multilingual.mp3",
  language_detection: true,
});

console.log("Detected language:", transcript.language_code);
```

**Specify Language:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/spanish-audio.mp3",
  language_code: "es", // Spanish
});
```

**Supported Languages:**

AssemblyAI supports 100+ languages. To see the complete list of supported language codes, check the official documentation at https://www.assemblyai.com/docs/concepts/supported-languages or run:
```bash
npm info assemblyai | grep -A 20 "language_code"
```

Common language codes include `en` (English), `es` (Spanish), `fr` (French), `de` (German), `zh` (Chinese), `ja` (Japanese), and many more.

**Multilingual Streaming (Beta):**
```javascript
const transcriber = new RealtimeTranscriber({
  apiKey: process.env.ASSEMBLYAI_API_KEY,
  sampleRate: 16000,
  // Auto-detect between supported languages
  language_code: "multi",
});
```

### 5.2 Custom Vocabulary and Word Boost

Improve accuracy for domain-specific terms.

**Word Boost:**
```javascript
const transcript = await client.transcripts.transcribe({
  audio: "https://example.com/tech-talk.mp3",
  word_boost: [
    "AssemblyAI",
    "Kubernetes",
    "GraphQL",
    "TypeScript",
    "microservices",
  ],
  boost_param: "high", // Options: "low", "default", "high"
});
```

### 5.3 Audio Format Support

AssemblyAI automatically detects and processes various audio formats.

**Supported Formats:**

AssemblyAI automatically detects and supports most common audio/video formats including MP3, MP4, WAV, FLAC, AAC, and more. For the complete list of supported formats, see https://www.assemblyai.com/docs/concepts/audio-formats

**Optimal Settings:**
For best transcription quality, use audio with 16kHz+ sample rate, mono or stereo channels, and 16-bit+ depth.

### 5.4 Webhook Configuration

Receive notifications when transcription completes.

**Setup Webhook:**
```javascript
const transcript = await client.transcripts.submit({
  audio: "https://example.com/audio.mp3",
  webhook_url: "https://your-domain.com/webhook/assemblyai",
  webhook_auth_header_name: "X-Webhook-Secret",
  webhook_auth_header_value: process.env.WEBHOOK_SECRET,
});
```

**Webhook Handler (Express Example):**
```javascript
import express from "express";

const app = express();
app.use(express.json());

app.post("/webhook/assemblyai", (req, res) => {
  // Verify webhook secret
  if (req.headers["x-webhook-secret"] !== process.env.WEBHOOK_SECRET) {
    return res.status(401).send("Unauthorized");
  }

  const { transcript_id, status } = req.body;

  if (status === "completed") {
    console.log("Transcription completed:", transcript_id);
    // Retrieve full transcript
    client.transcripts.get(transcript_id).then(transcript => {
      console.log(transcript.text);
    });
  } else if (status === "error") {
    console.error("Transcription failed:", req.body.error);
  }

  res.sendStatus(200);
});
```

### 5.5 Polling Status Manually

```javascript
const transcript = await client.transcripts.submit({
  audio: "https://example.com/audio.mp3",
});

// Poll every 3 seconds
const checkStatus = async () => {
  const result = await client.transcripts.get(transcript.id);

  if (result.status === "completed") {
    console.log("Done:", result.text);
    return result;
  } else if (result.status === "error") {
    throw new Error(result.error);
  } else {
    console.log("Status:", result.status);
    await new Promise(resolve => setTimeout(resolve, 3000));
    return checkStatus();
  }
};

const finalTranscript = await checkStatus();
```

### 5.6 Delete Transcripts

Permanently delete transcripts from AssemblyAI servers.

```javascript
// Delete a specific transcript
await client.transcripts.delete(transcript.id);
console.log("Transcript deleted");
```

### 5.7 List Transcripts

Retrieve a list of previously submitted transcripts.

```javascript
const page = await client.transcripts.list({
  limit: 10,
  status: "completed",
});

for (const transcript of page.transcripts) {
  console.log({
    id: transcript.id,
    status: transcript.status,
    created: transcript.created,
    audio_duration: transcript.audio_duration,
  });
}

// Handle pagination
if (page.page_details.next_url) {
  const nextPage = await client.transcripts.list({
    limit: 10,
    before_id: page.transcripts[page.transcripts.length - 1].id,
  });
}
```

## 6. TypeScript Usage

The AssemblyAI SDK is written in TypeScript and provides comprehensive type definitions.

### Import Types
```typescript
import {
  AssemblyAI,
  TranscribeParams,
  Transcript,
  RealtimeTranscript,
  TranscriptStatus,
  SentimentAnalysisResult,
  Entity,
  ContentSafetyLabel,
  LemurTaskParams,
  LemurResponse,
} from "assemblyai";
```

### Type-Safe Transcription
```typescript
const params: TranscribeParams = {
  audio: "https://example.com/audio.mp3",
  speaker_labels: true,
  sentiment_analysis: true,
  entity_detection: true,
};

const transcript: Transcript = await client.transcripts.transcribe(params);

// TypeScript knows the available properties
if (transcript.status === "completed") {
  const text: string = transcript.text;
  const sentiments: SentimentAnalysisResult[] | undefined =
    transcript.sentiment_analysis_results;
  const entities: Entity[] | undefined = transcript.entities;
}
```

### Type-Safe LeMUR
```typescript
import type { LemurQuestionAnswerParams } from "assemblyai";

const params: LemurQuestionAnswerParams = {
  transcript_ids: [transcript.id],
  questions: [
    {
      question: "What were the key takeaways?",
      answer_format: "bullet points",
    },
  ],
  final_model: "anthropic/claude-3-5-sonnet",
};

const result = await client.lemur.question(params);
```

### Type Guards for Real-Time Transcripts
```typescript
import { RealtimeTranscript } from "assemblyai";

transcriber.on("transcript", (transcript: RealtimeTranscript) => {
  if (transcript.message_type === "FinalTranscript") {
    // TypeScript knows this is a final transcript
    const text: string = transcript.text;
    const confidence: number = transcript.confidence;
    const words: Array<{ text: string; start: number; end: number }> =
      transcript.words;
  } else if (transcript.message_type === "PartialTranscript") {
    // Partial transcript
    const partialText: string = transcript.text;
  }
});
```

