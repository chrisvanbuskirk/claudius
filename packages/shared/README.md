# @claudius/shared

Shared database layer and types for Claudius - an AI-powered daily briefing system.

## Overview

This package provides a SQLite database layer for storing and managing:
- Daily briefings with AI-generated cards
- User feedback on briefings and cards
- Research logs from MCP servers
- Feedback patterns for improving future briefings

## Installation

```bash
npm install
```

## Usage

### Initialize Database

```typescript
import { initDatabase } from '@claudius/shared';

const db = initDatabase('/path/to/claudius.db');
```

### Create a Briefing

```typescript
import { createBriefing } from '@claudius/shared';

const briefing = createBriefing({
  date: '2025-12-06',
  title: 'Daily Tech Briefing',
  cards: [
    {
      title: 'AI Developments',
      summary: 'Latest advancements in AI...',
      sources: [
        { url: 'https://example.com', title: 'Source 1' }
      ],
      relevance_score: 0.95
    }
  ],
  research_time_ms: 5000,
  model_used: 'claude-opus-4-5',
  total_tokens: 10000
});
```

### Query Briefings

```typescript
import { getBriefing, getBriefingsByDate, searchBriefings } from '@claudius/shared';

// Get single briefing
const briefing = getBriefing(1);

// Get briefings by date range
const briefings = getBriefingsByDate('2025-12-01', '2025-12-07');

// Search briefings
const results = searchBriefings('AI developments');
```

### Add Feedback

```typescript
import { addFeedback, updateFeedbackPattern } from '@claudius/shared';

// Add feedback for a card
addFeedback({
  briefing_id: 1,
  card_index: 0,
  rating: 1,  // 1 = like, 0 = neutral, -1 = dislike
  reason: 'Very informative'
});

// Update feedback pattern
updateFeedbackPattern('AI', true);  // true = positive
```

### Get Feedback Patterns

```typescript
import { getFeedbackPatterns, getFeedbackStats } from '@claudius/shared';

// Get all patterns
const patterns = getFeedbackPatterns();

// Get statistics
const stats = getFeedbackStats();
console.log(`Positive: ${stats.positive}, Negative: ${stats.negative}`);
```

## Database Schema

### Tables

- **briefings** - Main briefing records
- **briefing_cards** - Individual cards within briefings
- **feedback** - User feedback on briefings/cards
- **research_logs** - Research operation logs
- **feedback_patterns** - Aggregated feedback patterns by topic

## API Reference

### Database Initialization

- `initDatabase(dbPath: string)` - Initialize database with schema
- `getDatabase()` - Get current database instance
- `closeDatabase()` - Close database connection

### Briefing Operations

- `createBriefing(data)` - Create new briefing
- `getBriefing(id)` - Get briefing by ID
- `getBriefingsByDate(startDate, endDate)` - Get briefings in date range
- `searchBriefings(query)` - Search briefings by text
- `deleteBriefing(id)` - Delete briefing
- `getAllBriefings(limit?, offset?)` - Get all briefings with pagination

### Feedback Operations

- `addFeedback(data)` - Add user feedback
- `getFeedbackForBriefing(briefingId)` - Get all feedback for a briefing
- `getFeedbackPatterns()` - Get all feedback patterns
- `getFeedbackPattern(topic)` - Get pattern for specific topic
- `updateFeedbackPattern(topic, isPositive)` - Update pattern
- `getFeedbackStats()` - Get overall feedback statistics
- `deleteFeedbackForBriefing(briefingId)` - Delete all feedback for a briefing

## TypeScript Types

All types are exported from the package:

```typescript
import type {
  Briefing,
  BriefingCard,
  Feedback,
  FeedbackPattern,
  ResearchLog,
  CreateBriefingData,
  AddFeedbackData
} from '@claudius/shared';
```

## Development

```bash
# Build
npm run build

# Watch mode
npm run dev

# Clean
npm run clean
```

## License

MIT
