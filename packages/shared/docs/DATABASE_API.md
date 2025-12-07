# Claudius Database API Reference

Complete reference for the SQLite database layer in `@claudius/shared`.

## Table of Contents

- [Database Initialization](#database-initialization)
- [Briefing Operations](#briefing-operations)
- [Feedback Operations](#feedback-operations)
- [TypeScript Types](#typescript-types)
- [Database Schema](#database-schema)

---

## Database Initialization

### `initDatabase(dbPath: string): Database`

Initializes the SQLite database with the schema. Creates the database file if it doesn't exist.

```typescript
import { initDatabase } from '@claudius/shared';

const db = initDatabase('/path/to/claudius.db');
```

**Parameters:**
- `dbPath` - Full path to the SQLite database file

**Returns:** Database instance

**Notes:**
- Automatically enables foreign keys
- Creates all tables and indexes if they don't exist
- Safe to call multiple times (idempotent)

---

### `getDatabase(): Database`

Gets the current database instance.

```typescript
import { getDatabase } from '@claudius/shared';

const db = getDatabase();
```

**Returns:** Database instance

**Throws:** Error if database hasn't been initialized

---

### `closeDatabase(): void`

Closes the database connection.

```typescript
import { closeDatabase } from '@claudius/shared';

closeDatabase();
```

---

## Briefing Operations

### `createBriefing(data: CreateBriefingData): Briefing`

Creates a new briefing with cards.

```typescript
import { createBriefing } from '@claudius/shared';

const briefing = createBriefing({
  date: '2025-12-06',
  title: 'Daily Tech Briefing',
  cards: [
    {
      title: 'AI Developments',
      summary: 'Latest in AI...',
      sources: [{ url: 'https://example.com', title: 'Source' }],
      relevance_score: 0.95
    }
  ],
  research_time_ms: 5000,
  model_used: 'claude-opus-4-5',
  total_tokens: 10000
});
```

**Parameters:**
- `data.date` - Date in YYYY-MM-DD format (required)
- `data.title` - Briefing title (optional)
- `data.cards` - Array of briefing cards (required)
- `data.research_time_ms` - Research time in milliseconds (optional)
- `data.model_used` - AI model identifier (optional)
- `data.total_tokens` - Total tokens used (optional)

**Returns:** Created briefing with ID

---

### `getBriefing(id: number): Briefing | null`

Retrieves a single briefing by ID.

```typescript
import { getBriefing } from '@claudius/shared';

const briefing = getBriefing(1);
if (briefing) {
  console.log(briefing.title);
}
```

**Parameters:**
- `id` - Briefing ID

**Returns:** Briefing object or null if not found

---

### `getBriefingsByDate(startDate: string, endDate: string): Briefing[]`

Gets all briefings within a date range.

```typescript
import { getBriefingsByDate } from '@claudius/shared';

const briefings = getBriefingsByDate('2025-12-01', '2025-12-07');
```

**Parameters:**
- `startDate` - Start date (YYYY-MM-DD, inclusive)
- `endDate` - End date (YYYY-MM-DD, inclusive)

**Returns:** Array of briefings, sorted by date (newest first)

---

### `searchBriefings(query: string): Briefing[]`

Searches briefings by text in title, card titles, and summaries.

```typescript
import { searchBriefings } from '@claudius/shared';

const results = searchBriefings('artificial intelligence');
```

**Parameters:**
- `query` - Search query string

**Returns:** Array of matching briefings

**Notes:**
- Case-insensitive search
- Searches in: briefing title, card titles, card summaries
- Uses SQL LIKE with wildcards

---

### `getAllBriefings(limit?: number, offset?: number): Briefing[]`

Gets all briefings with pagination.

```typescript
import { getAllBriefings } from '@claudius/shared';

// Get first 10 briefings
const briefings = getAllBriefings(10, 0);

// Get next 10 briefings
const moreBriefings = getAllBriefings(10, 10);
```

**Parameters:**
- `limit` - Maximum number of results (default: 50)
- `offset` - Number of results to skip (default: 0)

**Returns:** Array of briefings, sorted by date (newest first)

---

### `deleteBriefing(id: number): boolean`

Deletes a briefing and all associated data.

```typescript
import { deleteBriefing } from '@claudius/shared';

const deleted = deleteBriefing(1);
if (deleted) {
  console.log('Briefing deleted');
}
```

**Parameters:**
- `id` - Briefing ID

**Returns:** `true` if deleted, `false` if not found

**Notes:**
- Cascade deletes: cards, feedback, and research logs are also deleted

---

## Feedback Operations

### `addFeedback(data: AddFeedbackData): Feedback`

Adds user feedback for a briefing or specific card.

```typescript
import { addFeedback } from '@claudius/shared';

const feedback = addFeedback({
  briefing_id: 1,
  card_index: 0,
  rating: 1,  // 1 = like, 0 = neutral, -1 = dislike
  reason: 'Very informative'
});
```

**Parameters:**
- `data.briefing_id` - Briefing ID (required)
- `data.card_index` - Card index (optional, omit for whole briefing)
- `data.rating` - Rating: 1 (like), 0 (neutral), -1 (dislike) (required)
- `data.reason` - Feedback reason/comment (optional)

**Returns:** Created feedback with ID

---

### `getFeedbackForBriefing(briefingId: number): Feedback[]`

Gets all feedback for a specific briefing.

```typescript
import { getFeedbackForBriefing } from '@claudius/shared';

const feedback = getFeedbackForBriefing(1);
```

**Parameters:**
- `briefingId` - Briefing ID

**Returns:** Array of feedback entries

---

### `getFeedbackPatterns(): FeedbackPattern[]`

Gets all feedback patterns, sorted by total feedback count.

```typescript
import { getFeedbackPatterns } from '@claudius/shared';

const patterns = getFeedbackPatterns();
patterns.forEach(pattern => {
  console.log(`${pattern.topic}: +${pattern.positive_count} / -${pattern.negative_count}`);
});
```

**Returns:** Array of feedback patterns

---

### `getFeedbackPattern(topic: string): FeedbackPattern | null`

Gets feedback pattern for a specific topic.

```typescript
import { getFeedbackPattern } from '@claudius/shared';

const pattern = getFeedbackPattern('AI & Machine Learning');
```

**Parameters:**
- `topic` - Topic name

**Returns:** Feedback pattern or null if not found

---

### `updateFeedbackPattern(topic: string, isPositive: boolean): FeedbackPattern`

Updates or creates a feedback pattern for a topic.

```typescript
import { updateFeedbackPattern } from '@claudius/shared';

// Increment positive count
updateFeedbackPattern('TypeScript', true);

// Increment negative count
updateFeedbackPattern('Politics', false);
```

**Parameters:**
- `topic` - Topic name
- `isPositive` - `true` to increment positive count, `false` for negative

**Returns:** Updated feedback pattern

**Notes:**
- Creates new pattern if topic doesn't exist
- Updates `last_updated` timestamp

---

### `getFeedbackStats(): Object`

Gets overall feedback statistics.

```typescript
import { getFeedbackStats } from '@claudius/shared';

const stats = getFeedbackStats();
console.log(`Total: ${stats.total}`);
console.log(`Positive: ${stats.positive}`);
console.log(`Negative: ${stats.negative}`);
console.log(`Neutral: ${stats.neutral}`);
```

**Returns:**
```typescript
{
  total: number;
  positive: number;
  negative: number;
  neutral: number;
}
```

---

### `deleteFeedbackForBriefing(briefingId: number): number`

Deletes all feedback for a briefing.

```typescript
import { deleteFeedbackForBriefing } from '@claudius/shared';

const deleted = deleteFeedbackForBriefing(1);
console.log(`Deleted ${deleted} feedback entries`);
```

**Parameters:**
- `briefingId` - Briefing ID

**Returns:** Number of feedback entries deleted

---

## TypeScript Types

### `Briefing`

```typescript
interface Briefing {
  id: number;
  date: string;
  title: string | null;
  cards: BriefingCard[];
  research_time_ms: number | null;
  model_used: string | null;
  total_tokens: number | null;
  created_at: string;
}
```

### `BriefingCard`

```typescript
interface BriefingCard {
  title: string;
  summary: string;
  sources: Source[];
  relevance_score?: number;
}
```

### `Source`

```typescript
interface Source {
  url: string;
  title?: string;
  snippet?: string;
}
```

### `Feedback`

```typescript
interface Feedback {
  id: number;
  briefing_id: number;
  card_index: number | null;
  rating: number;  // -1, 0, or 1
  reason: string | null;
  created_at: string;
}
```

### `FeedbackPattern`

```typescript
interface FeedbackPattern {
  id: number;
  topic: string;
  positive_count: number;
  negative_count: number;
  last_updated: string;
}
```

### `ResearchLog`

```typescript
interface ResearchLog {
  id: number;
  briefing_id: number;
  mcp_server: string | null;
  query: string | null;
  result_tokens: number | null;
  duration_ms: number | null;
  error_message: string | null;
  created_at: string;
}
```

---

## Database Schema

### Tables

#### `briefings`
Stores main briefing records.

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PRIMARY KEY | Auto-increment ID |
| date | DATE NOT NULL | Briefing date (YYYY-MM-DD) |
| title | TEXT | Briefing title |
| cards | JSON NOT NULL | Complete cards array |
| research_time_ms | INTEGER | Research duration |
| model_used | TEXT | AI model identifier |
| total_tokens | INTEGER | Total tokens used |
| created_at | TIMESTAMP | Creation timestamp |

**Indexes:** `idx_briefings_date` on `date`

---

#### `briefing_cards`
Normalized storage for individual cards (for efficient searching).

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PRIMARY KEY | Auto-increment ID |
| briefing_id | INTEGER NOT NULL | Foreign key to briefings |
| card_index | INTEGER | Card position in array |
| title | TEXT | Card title |
| summary | TEXT | Card summary |
| sources | JSON | Sources array |
| relevance_score | FLOAT | Relevance score |

**Foreign Keys:** `briefing_id` → `briefings(id)` ON DELETE CASCADE

**Indexes:** `idx_briefing_cards_briefing_id` on `briefing_id`

---

#### `feedback`
User feedback on briefings and cards.

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PRIMARY KEY | Auto-increment ID |
| briefing_id | INTEGER NOT NULL | Foreign key to briefings |
| card_index | INTEGER | Card index (null = whole briefing) |
| rating | INTEGER | -1 (dislike), 0 (neutral), 1 (like) |
| reason | TEXT | Feedback comment |
| created_at | TIMESTAMP | Creation timestamp |

**Foreign Keys:** `briefing_id` → `briefings(id)` ON DELETE CASCADE

**Indexes:**
- `idx_feedback_briefing_id` on `briefing_id`
- `idx_feedback_rating` on `rating`

---

#### `research_logs`
Logs of research operations.

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PRIMARY KEY | Auto-increment ID |
| briefing_id | INTEGER NOT NULL | Foreign key to briefings |
| mcp_server | TEXT | MCP server name |
| query | TEXT | Search query |
| result_tokens | INTEGER | Tokens in result |
| duration_ms | INTEGER | Operation duration |
| error_message | TEXT | Error (if any) |
| created_at | TIMESTAMP | Creation timestamp |

**Foreign Keys:** `briefing_id` → `briefings(id)` ON DELETE CASCADE

**Indexes:** `idx_research_logs_briefing_id` on `briefing_id`

---

#### `feedback_patterns`
Aggregated feedback patterns by topic.

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PRIMARY KEY | Auto-increment ID |
| topic | TEXT UNIQUE NOT NULL | Topic name |
| positive_count | INTEGER | Positive feedback count |
| negative_count | INTEGER | Negative feedback count |
| last_updated | TIMESTAMP | Last update timestamp |

**Indexes:** `idx_feedback_patterns_topic` on `topic`

---

## Best Practices

1. **Always initialize the database before use:**
   ```typescript
   import { initDatabase } from '@claudius/shared';
   initDatabase('/path/to/db.db');
   ```

2. **Use transactions for multiple operations:**
   ```typescript
   import { getDatabase } from '@claudius/shared';
   const db = getDatabase();

   const transaction = db.transaction(() => {
     // Multiple operations here
   });
   transaction();
   ```

3. **Close database when done:**
   ```typescript
   import { closeDatabase } from '@claudius/shared';
   closeDatabase();
   ```

4. **Use proper date formatting:**
   - Always use YYYY-MM-DD format for dates
   - Use `new Date().toISOString().split('T')[0]` for current date

5. **Handle null values:**
   - Many fields are nullable
   - Always check for null before using optional fields

6. **Cascade deletes:**
   - Deleting a briefing automatically deletes associated cards, feedback, and logs
   - No need to manually clean up child records
