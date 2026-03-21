---
name: translation
description: "DeepL API client for JavaScript/TypeScript with text and document translation, glossary management, and language detection"
metadata:
  languages: "javascript"
  versions: "1.21.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "deepl,translation,language,localization,api"
---

# DeepL API Coding Guidelines (JavaScript/TypeScript)

You are a DeepL API coding expert. Help me with writing code using the DeepL API calling the official libraries and SDKs.

## Golden Rule: Use the Correct and Current SDK

Always use the official DeepL Node.js SDK for all DeepL API interactions. Do not use unofficial or deprecated libraries.

- **Library Name:** DeepL Node.js Client Library
- **NPM Package:** `deepl-node`
- **Current Version:** 1.21.0
- **Deprecated/Unofficial Packages:** `deepl`, `deepl-api`, `deepl-translate`

**Installation:**

```bash
# Correct
npm install deepl-node

# Incorrect - unofficial packages
npm install deepl
npm install deepl-api
```

**Import Patterns:**

```javascript
// Correct - ES6 import
import * as deepl from 'deepl-node';

// Correct - CommonJS
const deepl = require('deepl-node');

// Incorrect
import DeepL from 'deepl';
import { translate } from 'deepl-api';
```

## Initialization and API Key

The DeepL library requires creating a `DeepLClient` instance with your authentication key for all API calls.

### Getting an API Key

Create a DeepL API account to receive your authentication key. With a DeepL API Free account, you can translate up to 500,000 characters per month for free.

### Basic Initialization

```javascript
import * as deepl from 'deepl-node';

// Initialize with API key
const authKey = 'your-api-key-here'; // Replace with your actual key
const translator = new deepl.DeepLClient(authKey);
```

### Environment Variable Configuration

In production, always fetch the authentication key from environment variables:

```javascript
import * as deepl from 'deepl-node';

const authKey = process.env.DEEPL_AUTH_KEY;
const translator = new deepl.DeepLClient(authKey);
```

### CommonJS Initialization

```javascript
const deepl = require('deepl-node');

const authKey = process.env.DEEPL_AUTH_KEY;
const translator = new deepl.DeepLClient(authKey);
```

## Text Translation

### Basic Translation

Translate a single text string from one language to another:

```javascript
import * as deepl from 'deepl-node';

const authKey = process.env.DEEPL_AUTH_KEY;
const translator = new deepl.DeepLClient(authKey);

const result = await translator.translateText('Hello, world!', null, 'fr');
console.log(result.text); // 'Bonjour, le monde !'
```

The `translateText` method signature:
- First parameter: text to translate (string or array of strings)
- Second parameter: source language (null for auto-detection, or language code like 'en')
- Third parameter: target language (required, e.g., 'fr', 'de', 'es')
- Fourth parameter (optional): options object

### Auto-Detection of Source Language

Use `null` as the source language to automatically detect it:

```javascript
const result = await translator.translateText('Hello, world!', null, 'de');
console.log(result.text); // 'Hallo, Welt!'
console.log(result.detectedSourceLang); // 'EN'
```

### Specifying Source Language

```javascript
const result = await translator.translateText('Hello, world!', 'en', 'es');
console.log(result.text); // '¡Hola, mundo!'
```

### Multiple Texts Translation

Translate multiple texts in a single API call:

```javascript
const texts = [
  'Hello, world!',
  'How are you?',
  'Good morning'
];

const results = await translator.translateText(texts, null, 'ja');
results.forEach(result => {
  console.log(result.text);
});
// 'こんにちは、世界！'
// 'お元気ですか？'
// 'おはようございます'
```

### TextResult Properties

Each translation result contains:

```javascript
const result = await translator.translateText('Hello', null, 'de');

console.log(result.text);                    // Translated text: 'Hallo'
console.log(result.detectedSourceLang);      // Source language: 'EN'
console.log(result.billedCharacters);        // Characters charged
console.log(result.modelTypeUsed);           // Model type if specified
```

## Translation Options

### Formality Control

Control the formality level of translations for supported languages (DE, FR, IT, ES, NL, PL, PT-BR, PT-PT, JA, RU):

```javascript
// Informal translation
const informal = await translator.translateText(
  'How are you?',
  null,
  'de',
  { formality: 'less' }
);
console.log(informal.text); // 'Wie geht es dir?'

// Formal translation
const formal = await translator.translateText(
  'How are you?',
  null,
  'de',
  { formality: 'more' }
);
console.log(formal.text); // 'Wie geht es Ihnen?'
```

Formality options:
- `'less'` - informal language
- `'more'` - formal language
- `'default'` - standard formality
- `'prefer_less'` - informal if available, otherwise default
- `'prefer_more'` - formal if available, otherwise default

### Context Parameter

Provide additional context to improve translation accuracy (not translated, not billed):

```javascript
const result = await translator.translateText(
  'bank',
  'en',
  'de',
  { context: 'financial institution' }
);
console.log(result.text); // 'Bank' (financial context)

const result2 = await translator.translateText(
  'bank',
  'en',
  'de',
  { context: 'riverside' }
);
console.log(result2.text); // 'Ufer' (riverbank)
```

### Sentence Splitting Options

Control how text is split into sentences:

```javascript
// Split by newlines and punctuation (default)
const result1 = await translator.translateText(
  'First sentence. Second sentence.',
  null,
  'de',
  { splitSentences: 'on' }
);

// No splitting
const result2 = await translator.translateText(
  'First sentence. Second sentence.',
  null,
  'de',
  { splitSentences: 'off' }
);

// Split by punctuation only (not newlines)
const result3 = await translator.translateText(
  'First sentence.\nSecond sentence.',
  null,
  'de',
  { splitSentences: 'nonewlines' }
);
```

### Preserve Formatting

Prevent automatic format correction:

```javascript
const result = await translator.translateText(
  'Hello,    world!',
  null,
  'de',
  { preserveFormatting: true }
);
// Preserves extra spaces
```

### Model Type Selection

Choose between quality-optimized and latency-optimized models:

```javascript
// Quality-optimized (higher quality, slower)
const qualityResult = await translator.translateText(
  'Complex technical document',
  'en',
  'de',
  { modelType: 'quality_optimized' }
);

// Latency-optimized (faster, still high quality)
const fastResult = await translator.translateText(
  'Simple message',
  'en',
  'de',
  { modelType: 'latency_optimized' }
);

// Use best available
const result = await translator.translateText(
  'Text here',
  'en',
  'de',
  { modelType: 'prefer_quality_optimized' }
);
```

### HTML and XML Tag Handling

Parse and preserve markup tags:

```javascript
// HTML tag handling
const htmlResult = await translator.translateText(
  '<p>Hello, <strong>world</strong>!</p>',
  null,
  'de',
  { tagHandling: 'html' }
);
console.log(htmlResult.text); // '<p>Hallo, <strong>Welt</strong>!</p>'

// XML tag handling
const xmlResult = await translator.translateText(
  '<text>Hello, <emphasis>world</emphasis>!</text>',
  null,
  'de',
  { tagHandling: 'xml' }
);
```

### XML-Specific Options

Advanced XML handling options:

```javascript
const result = await translator.translateText(
  '<root><p>First paragraph</p><ignore>Do not translate</ignore></root>',
  'en',
  'de',
  {
    tagHandling: 'xml',
    outlineDetection: false,           // Disable automatic tag detection
    splittingTags: ['p', 'div'],       // Tags that split sentences
    nonSplittingTags: ['span'],        // Tags that don't split sentences
    ignoreTags: ['ignore', 'code']     // Tags with untranslated content
  }
);
```

## Document Translation

### Basic Document Translation

Translate entire documents while preserving formatting:

```javascript
import * as deepl from 'deepl-node';

const authKey = process.env.DEEPL_AUTH_KEY;
const translator = new deepl.DeepLClient(authKey);

// Translate document from file path to file path
await translator.translateDocument(
  'input.docx',
  'output.docx',
  'en',
  'de'
);
```

### Document Translation with Options

```javascript
await translator.translateDocument(
  'Manual.docx',
  'Anleitung.docx',
  'en',
  'de',
  {
    formality: 'more',
    glossary: 'glossary-id-here'
  }
);
```

### Supported Document Formats

DeepL supports various document formats:
- **Text documents:** docx, pptx, pdf, htm, html, txt
- **Spreadsheets:** xlsx
- **Others:** Check DeepL API documentation for complete list

### Document Translation Input/Output Formats

Documents can be provided as:
- File paths (strings)
- Streams or FileHandles
- Buffers (requires `filename` option)

```javascript
// Using file paths
await translator.translateDocument(
  'input.pdf',
  'output.pdf',
  'en',
  'fr'
);

// Using streams
import * as fs from 'fs';

const inputStream = fs.createReadStream('input.docx');
const outputStream = fs.createWriteStream('output.docx');

await translator.translateDocument(
  inputStream,
  outputStream,
  'en',
  'de',
  { filename: 'input.docx' }
);

// Using buffer
const buffer = fs.readFileSync('input.docx');
await translator.translateDocument(
  buffer,
  'output.docx',
  'en',
  'es',
  { filename: 'document.docx' }
);
```

### Document Minification

Reduce document size by compressing embedded media (supports docx, pptx):

```javascript
await translator.translateDocument(
  'large_presentation.pptx',
  'output.pptx',
  'en',
  'de',
  { enableDocumentMinification: true }
);
```

Supported media types for minification: png, jpg, jpeg, emf, bmp, tiff, wdp, svg, gif, mp4, asf, avi, m4v, mpg, mpeg, wmv, mov, aiff, au, mid, midi, mp3, m4a, wav, wma

**Note:** Requires 2x file size temporary disk space.

### Document Translation Error Handling

Handle errors and retrieve document handle for recovery:

```javascript
try {
  await translator.translateDocument(
    'Manual.docx',
    'Anleitung.docx',
    'en',
    'de'
  );
} catch (error) {
  if (error.documentHandle) {
    const { documentId, documentKey } = error.documentHandle;
    console.log(`Document ID: ${documentId}`);
    console.log(`Document Key: ${documentKey}`);
    // Use these to check status or download later
  } else {
    console.error('Translation failed:', error.message);
  }
}
```

### Lower-Level Document Translation Methods

For advanced control over the document translation process:

```javascript
// Upload document
const handle = await translator.uploadDocument(
  'input.docx',
  'en',
  'de'
);

// Check translation status
const status = await translator.getDocumentStatus(handle);
console.log(`Status: ${status.status}`);
console.log(`Billed characters: ${status.billedCharacters}`);

// Wait until translation is complete
await translator.isDocumentTranslationComplete(handle);

// Download translated document
await translator.downloadDocument(handle, 'output.docx');
```

### Polling Document Status

```javascript
const handle = await translator.uploadDocument('input.pdf', 'en', 'fr');

// Poll until complete
while (true) {
  const status = await translator.getDocumentStatus(handle);

  if (status.status === 'done') {
    await translator.downloadDocument(handle, 'output.pdf');
    break;
  } else if (status.status === 'error') {
    throw new Error('Document translation failed');
  }

  // Wait before checking again
  await new Promise(resolve => setTimeout(resolve, 5000));
}
```

## Glossaries

Glossaries allow you to specify custom translations for specific terms and phrases.

### Creating Multilingual Glossaries

```javascript
import * as deepl from 'deepl-node';

const authKey = process.env.DEEPL_AUTH_KEY;
const translator = new deepl.DeepLClient(authKey);

// Create glossary entries
const entries = new deepl.GlossaryEntries({
  entries: {
    'artist': 'Maler',
    'prize': 'Gewinn'
  }
});

// Create glossary
const glossary = await translator.createMultilingualGlossary(
  'My glossary',
  [{
    sourceLangCode: 'en',
    targetLangCode: 'de',
    entries: entries
  }]
);

console.log(`Glossary ID: ${glossary.glossaryId}`);
console.log(`Created: ${glossary.creationTime}`);
```

### Creating Glossary from CSV

```javascript
const fs = require('fs').promises;

// Read CSV file
const csvContent = await fs.readFile('glossary.csv', 'utf-8');

// CSV format:
// source,target
// artist,Maler
// prize,Gewinn

const glossary = await translator.createMultilingualGlossaryWithCsv(
  'CSV Glossary',
  'en',
  'de',
  csvContent
);
```

### Listing Glossaries

```javascript
const glossaries = await translator.listMultilingualGlossaries();

glossaries.forEach(g => {
  console.log(`Name: ${g.name}`);
  console.log(`ID: ${g.glossaryId}`);
  console.log(`Created: ${g.creationTime}`);

  g.dictionaries.forEach(dict => {
    console.log(`  ${dict.sourceLang} -> ${dict.targetLang}: ${dict.entryCount} entries`);
  });
});
```

### Finding Specific Glossary

```javascript
const glossaries = await translator.listMultilingualGlossaries();
const myGlossary = glossaries.find(g => g.name === 'My glossary');

if (myGlossary) {
  console.log(`Found glossary: ${myGlossary.glossaryId}`);
}
```

### Using Glossaries in Translation

```javascript
// Using glossary ID
const result = await translator.translateText(
  'The artist won the prize',
  'en',
  'de',
  { glossary: 'glossary-id-here' }
);

// Using glossary object
const glossaries = await translator.listMultilingualGlossaries();
const myGlossary = glossaries[0];

const result2 = await translator.translateText(
  'The artist won the prize',
  'en',
  'de',
  { glossary: myGlossary }
);

console.log(result2.text); // Uses custom translations from glossary
```

### Deleting Glossaries

```javascript
// Delete by glossary object
const glossaries = await translator.listMultilingualGlossaries();
const glossaryToDelete = glossaries.find(g => g.name === 'Old glossary');
await translator.deleteMultilingualGlossary(glossaryToDelete.glossaryId);

// Delete by ID
await translator.deleteMultilingualGlossary('glossary-id-here');
```

### Glossary Supported Languages

Check which language pairs support glossaries:

```javascript
const glossaryLanguages = await translator.getGlossaryLanguages();

glossaryLanguages.forEach(langPair => {
  console.log(`${langPair.sourceLang} -> ${langPair.targetLang}`);
});
```

## Language Detection and Information

### Detecting Language

Detect the language of a text without translating:

```javascript
const detectedLang = await translator.detectLanguage('Bonjour, comment allez-vous?');
console.log(detectedLang); // 'fr'

// Detect from array of texts
const texts = ['Hello', 'Bonjour', 'Hola'];
const languages = await translator.detectLanguage(texts);
languages.forEach((lang, i) => {
  console.log(`"${texts[i]}" is ${lang}`);
});
// "Hello" is en
// "Bonjour" is fr
// "Hola" is es
```

### Getting Supported Languages

```javascript
// Get all target languages
const targetLanguages = await translator.getTargetLanguages();
targetLanguages.forEach(lang => {
  console.log(`${lang.code}: ${lang.name}`);
  if (lang.supportsFormality) {
    console.log('  Supports formality');
  }
});

// Get all source languages
const sourceLanguages = await translator.getSourceLanguages();
sourceLanguages.forEach(lang => {
  console.log(`${lang.code}: ${lang.name}`);
});
```

Language object properties:
- `code` - ISO 639-1 language code (e.g., 'en', 'de', 'fr')
- `name` - Display name (e.g., 'English', 'German', 'French')
- `supportsFormality` - Boolean (target languages only)

## Usage Monitoring

### Checking Account Usage

Monitor your character usage and account limits:

```javascript
const usage = await translator.getUsage();

console.log(`Characters used: ${usage.character.count}`);
console.log(`Character limit: ${usage.character.limit}`);
console.log(`Document count: ${usage.document.count}`);
console.log(`Document limit: ${usage.document.limit}`);

// Calculate remaining characters
const remaining = usage.character.limit - usage.character.count;
console.log(`Remaining characters: ${remaining}`);
```

### Account Tiers

- **DeepL API Free:** 500,000 characters/month, up to 2 active API keys
- **DeepL API Pro:** Pay-as-you-go, up to 25 active API keys, cost control limits

## Error Handling

### Common Error Patterns

```javascript
try {
  const result = await translator.translateText('Hello', null, 'de');
  console.log(result.text);
} catch (error) {
  if (error.code === 403) {
    console.error('Authentication failed. Check your API key.');
  } else if (error.code === 456) {
    console.error('Quota exceeded. Monthly character limit reached.');
  } else if (error.code === 400) {
    console.error('Bad request. Check parameters.');
  } else {
    console.error('Translation error:', error.message);
  }
}
```

### Document Translation Errors

```javascript
try {
  await translator.translateDocument('in.docx', 'out.docx', 'en', 'de');
} catch (error) {
  if (error.documentHandle) {
    // Document was uploaded but translation failed
    const { documentId, documentKey } = error.documentHandle;
    console.log('Can retry with document ID:', documentId);
  } else {
    // Upload or other error
    console.error('Error:', error.message);
  }
}
```

## TypeScript Support

The deepl-node library includes full TypeScript type definitions:

```typescript
import * as deepl from 'deepl-node';

const authKey: string = process.env.DEEPL_AUTH_KEY!;
const translator: deepl.DeepLClient = new deepl.DeepLClient(authKey);

async function translateText() {
  const targetLang: deepl.TargetLanguageCode = 'de';
  const result: deepl.TextResult = await translator.translateText(
    'Hello, world!',
    null,
    targetLang
  );

  console.log(result.text);
  console.log(result.detectedSourceLang);
}

translateText();
```

### Type Definitions

```typescript
// Language codes
type SourceLanguageCode = 'en' | 'de' | 'fr' | 'es' | 'it' | 'ja' | 'ko' | ...;
type TargetLanguageCode = 'en-US' | 'en-GB' | 'de' | 'fr' | 'es' | ...;

// Formality options
type Formality = 'less' | 'more' | 'default' | 'prefer_less' | 'prefer_more';

// Model types
type ModelType = 'quality_optimized' | 'latency_optimized' | 'prefer_quality_optimized';

// Translation result
interface TextResult {
  text: string;
  detectedSourceLang: string;
  billedCharacters?: number;
  modelTypeUsed?: string;
}
```

## Advanced Configuration

### Custom Server URL

For DeepL API Pro plans with custom endpoints:

```javascript
const translator = new deepl.DeepLClient(authKey, {
  serverUrl: 'https://api.deepl.com'  // Pro endpoint
  // or 'https://api-free.deepl.com' for Free endpoint
});
```

### HTTP Options

Configure timeout and other HTTP options:

```javascript
const translator = new deepl.DeepLClient(authKey, {
  maxRetries: 5,
  minTimeout: 10000  // 10 seconds
});
```

## Promise Handling

All methods return Promises and support both async/await and .then()/.catch():

```javascript
// Async/await (recommended)
async function translate() {
  const result = await translator.translateText('Hello', null, 'fr');
  console.log(result.text);
}

// Promise chaining
translator.translateText('Hello', null, 'fr')
  .then(result => {
    console.log(result.text);
  })
  .catch(error => {
    console.error('Translation failed:', error);
  });
```

## Complete Translation Example

```javascript
import * as deepl from 'deepl-node';

const authKey = process.env.DEEPL_AUTH_KEY;
const translator = new deepl.DeepLClient(authKey);

async function completeExample() {
  try {
    // 1. Check usage
    const usage = await translator.getUsage();
    console.log(`Remaining: ${usage.character.limit - usage.character.count} characters`);

    // 2. Get supported languages
    const targetLangs = await translator.getTargetLanguages();
    const germanLang = targetLangs.find(l => l.code === 'de');
    console.log(`German supports formality: ${germanLang.supportsFormality}`);

    // 3. Translate with options
    const result = await translator.translateText(
      'How are you doing today?',
      null,
      'de',
      { formality: 'more' }
    );
    console.log(`Translation: ${result.text}`);
    console.log(`Detected source: ${result.detectedSourceLang}`);

    // 4. Translate document
    await translator.translateDocument(
      'document.pdf',
      'dokument.pdf',
      'en',
      'de',
      { formality: 'more' }
    );

    console.log('Translation complete!');
  } catch (error) {
    console.error('Error:', error.message);
  }
}

completeExample();
```

## Node.js Version Support

The deepl-node package supports Node.js versions 12, 14, 16, 17, 18, 20, 22, and 24.

## Additional Resources

- Official Documentation: https://www.deepl.com/docs-api/
- GitHub Repository: https://github.com/DeepLcom/deepl-node
- NPM Package: https://www.npmjs.com/package/deepl-node
