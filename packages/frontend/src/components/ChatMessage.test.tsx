import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ChatMessage } from './ChatMessage';
import type { ChatMessage as ChatMessageType } from '../types';

// Mock framer-motion to avoid animation issues in tests
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: React.HTMLAttributes<HTMLDivElement>) => (
      <div {...props}>{children}</div>
    ),
  },
}));

// Mock clipboard API
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn().mockResolvedValue(undefined),
  },
});

describe('ChatMessage', () => {
  const mockUserMessage: ChatMessageType = {
    id: 1,
    briefing_id: 1,
    card_index: 0,
    role: 'user',
    content: 'Hello, this is a test message',
    created_at: new Date().toISOString(),
  };

  const mockAssistantMessage: ChatMessageType = {
    id: 2,
    briefing_id: 1,
    card_index: 0,
    role: 'assistant',
    content: 'This is a response from the assistant',
    tokens_used: 150,
    created_at: new Date().toISOString(),
  };

  it('renders user message content', () => {
    render(<ChatMessage message={mockUserMessage} />);
    expect(screen.getByText('Hello, this is a test message')).toBeInTheDocument();
  });

  it('renders assistant message content', () => {
    render(<ChatMessage message={mockAssistantMessage} />);
    expect(screen.getByText('This is a response from the assistant')).toBeInTheDocument();
  });

  it('shows token count for assistant messages', () => {
    render(<ChatMessage message={mockAssistantMessage} />);
    expect(screen.getByText(/150 tokens/)).toBeInTheDocument();
  });

  it('does not show token count for user messages', () => {
    render(<ChatMessage message={mockUserMessage} />);
    expect(screen.queryByText(/tokens/)).not.toBeInTheDocument();
  });

  it('renders user avatar icon for user messages', () => {
    const { container } = render(<ChatMessage message={mockUserMessage} />);
    // User messages should have flex-row-reverse class
    const messageWrapper = container.querySelector('.flex-row-reverse');
    expect(messageWrapper).toBeInTheDocument();
  });

  it('renders assistant avatar icon for assistant messages', () => {
    const { container } = render(<ChatMessage message={mockAssistantMessage} />);
    // Assistant messages should have flex-row class (not reversed)
    const messageWrapper = container.querySelector('.flex-row:not(.flex-row-reverse)');
    expect(messageWrapper).toBeInTheDocument();
  });

  it('displays relative timestamp', () => {
    render(<ChatMessage message={mockUserMessage} />);
    // formatDistanceToNow will show something like "less than a minute ago"
    expect(screen.getByText(/ago/)).toBeInTheDocument();
  });

  it('has a copy button', () => {
    render(<ChatMessage message={mockAssistantMessage} />);
    const copyButton = screen.getByTitle('Copy message');
    expect(copyButton).toBeInTheDocument();
  });

  it('copies message content when copy button is clicked', async () => {
    render(<ChatMessage message={mockAssistantMessage} />);
    const copyButton = screen.getByTitle('Copy message');

    fireEvent.click(copyButton);

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith(mockAssistantMessage.content);
  });

  it('renders markdown in assistant messages', () => {
    const markdownMessage: ChatMessageType = {
      id: 3,
      briefing_id: 1,
      card_index: 0,
      role: 'assistant',
      content: '**Bold text** and *italic text*',
      created_at: new Date().toISOString(),
    };

    render(<ChatMessage message={markdownMessage} />);
    // ReactMarkdown should render the bold and italic elements
    expect(screen.getByText('Bold text')).toBeInTheDocument();
    expect(screen.getByText('italic text')).toBeInTheDocument();
  });
});
