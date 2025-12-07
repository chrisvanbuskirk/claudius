/**
 * Example usage of the Claudius database layer
 * This file demonstrates how to use the SQLite database layer
 */

import {
  initDatabase,
  createBriefing,
  getBriefing,
  getBriefingsByDate,
  searchBriefings,
  addFeedback,
  getFeedbackPatterns,
  updateFeedbackPattern,
  getFeedbackStats,
} from '@claudius/shared';

// Initialize the database
const dbPath = './claudius-example.db';
const db = initDatabase(dbPath);

console.log('Database initialized successfully!\n');

// Example 1: Create a briefing
console.log('=== Creating a new briefing ===');
const briefing = createBriefing({
  date: '2025-12-06',
  title: 'Daily Tech Briefing - AI & Machine Learning',
  cards: [
    {
      title: 'Claude Opus 4.5 Released',
      summary: 'Anthropic announces Claude Opus 4.5 with enhanced reasoning capabilities and improved performance across benchmarks.',
      sources: [
        {
          url: 'https://www.anthropic.com/news/claude-opus-4-5',
          title: 'Anthropic Blog',
          snippet: 'Claude Opus 4.5 represents our most capable model to date...'
        }
      ],
      relevance_score: 0.95
    },
    {
      title: 'New SQLite Features in 2025',
      summary: 'SQLite 3.45 introduces JSON improvements and better performance for embedded databases.',
      sources: [
        {
          url: 'https://www.sqlite.org/changes.html',
          title: 'SQLite Release Notes'
        }
      ],
      relevance_score: 0.88
    },
    {
      title: 'TypeScript 5.7 Beta',
      summary: 'Microsoft releases TypeScript 5.7 beta with new type system improvements and better inference.',
      sources: [
        {
          url: 'https://devblogs.microsoft.com/typescript',
          title: 'TypeScript Blog'
        }
      ],
      relevance_score: 0.82
    }
  ],
  research_time_ms: 5500,
  model_used: 'claude-opus-4-5-20251101',
  total_tokens: 12500
});

console.log(`Created briefing ID: ${briefing.id}`);
console.log(`Title: ${briefing.title}`);
console.log(`Cards: ${briefing.cards.length}\n`);

// Example 2: Get a specific briefing
console.log('=== Retrieving briefing ===');
const retrieved = getBriefing(briefing.id);
if (retrieved) {
  console.log(`Retrieved: ${retrieved.title}`);
  console.log(`Date: ${retrieved.date}`);
  console.log(`Model used: ${retrieved.model_used}\n`);
}

// Example 3: Search briefings
console.log('=== Searching for "AI" in briefings ===');
const searchResults = searchBriefings('AI');
console.log(`Found ${searchResults.length} matching briefings`);
searchResults.forEach((result) => {
  console.log(`- ${result.title} (${result.date})`);
});
console.log();

// Example 4: Get briefings by date range
console.log('=== Getting briefings from last 7 days ===');
const endDate = new Date().toISOString().split('T')[0];
const startDate = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000)
  .toISOString()
  .split('T')[0];
const recentBriefings = getBriefingsByDate(startDate, endDate);
console.log(`Found ${recentBriefings.length} briefings in date range\n`);

// Example 5: Add feedback
console.log('=== Adding user feedback ===');
const feedback1 = addFeedback({
  briefing_id: briefing.id,
  card_index: 0,
  rating: 1, // Like
  reason: 'Very informative and relevant to my interests'
});
console.log(`Added positive feedback (ID: ${feedback1.id})`);

const feedback2 = addFeedback({
  briefing_id: briefing.id,
  card_index: 1,
  rating: 0, // Neutral
  reason: 'Interesting but not directly relevant to my work'
});
console.log(`Added neutral feedback (ID: ${feedback2.id})`);

const feedback3 = addFeedback({
  briefing_id: briefing.id,
  card_index: 2,
  rating: 1, // Like
  reason: 'Great summary of TypeScript updates'
});
console.log(`Added positive feedback (ID: ${feedback3.id})\n`);

// Example 6: Update feedback patterns
console.log('=== Updating feedback patterns ===');
updateFeedbackPattern('AI & Machine Learning', true);
updateFeedbackPattern('Database Technologies', true);
updateFeedbackPattern('TypeScript', true);
updateFeedbackPattern('Politics', false); // Example of negative feedback
console.log('Updated feedback patterns for various topics\n');

// Example 7: Get feedback statistics
console.log('=== Feedback statistics ===');
const stats = getFeedbackStats();
console.log(`Total feedback entries: ${stats.total}`);
console.log(`Positive: ${stats.positive} (${((stats.positive / stats.total) * 100).toFixed(1)}%)`);
console.log(`Negative: ${stats.negative} (${((stats.negative / stats.total) * 100).toFixed(1)}%)`);
console.log(`Neutral: ${stats.neutral} (${((stats.neutral / stats.total) * 100).toFixed(1)}%)\n`);

// Example 8: Get feedback patterns
console.log('=== Feedback patterns ===');
const patterns = getFeedbackPatterns();
patterns.forEach((pattern) => {
  const total = pattern.positive_count + pattern.negative_count;
  const sentiment = pattern.positive_count > pattern.negative_count ? '👍' : '👎';
  console.log(
    `${sentiment} ${pattern.topic}: +${pattern.positive_count} / -${pattern.negative_count} (total: ${total})`
  );
});

console.log('\n=== Example completed successfully! ===');
console.log(`Database location: ${dbPath}`);
