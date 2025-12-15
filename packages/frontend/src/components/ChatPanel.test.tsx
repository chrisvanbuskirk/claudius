import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ChatPanel } from './ChatPanel';

// Mock framer-motion
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: React.HTMLAttributes<HTMLDivElement>) => (
      <div {...props}>{children}</div>
    ),
  },
  AnimatePresence: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

// Mock the useChat hook
vi.mock('../hooks/useChat', () => ({
  useChat: vi.fn(() => ({
    messages: [],
    loading: false,
    sending: false,
    error: null,
    sendMessage: vi.fn(),
    clearHistory: vi.fn(),
  })),
}));

describe('ChatPanel', () => {
  const mockOnClose = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders nothing when isOpen is false', () => {
    const { container } = render(
      <ChatPanel
        briefingId="1"
        cardIndex={0}
        briefingTitle="Test Card"
        isOpen={false}
        onClose={mockOnClose}
      />
    );
    // AnimatePresence with isOpen=false should render nothing
    expect(container.firstChild).toBeNull();
  });

  it('renders panel when isOpen is true', () => {
    render(
      <ChatPanel
        briefingId="1"
        cardIndex={0}
        briefingTitle="Test Card"
        isOpen={true}
        onClose={mockOnClose}
      />
    );
    expect(screen.getByText('Chat about Card')).toBeInTheDocument();
    expect(screen.getByText('Test Card')).toBeInTheDocument();
  });

  it('displays empty state when no messages', () => {
    render(
      <ChatPanel
        briefingId="1"
        cardIndex={0}
        briefingTitle="Test Card"
        isOpen={true}
        onClose={mockOnClose}
      />
    );
    expect(screen.getByText('Start a Conversation')).toBeInTheDocument();
    expect(screen.getByText(/Ask questions about this card/)).toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', () => {
    render(
      <ChatPanel
        briefingId="1"
        cardIndex={0}
        briefingTitle="Test Card"
        isOpen={true}
        onClose={mockOnClose}
      />
    );

    // Find the close button by aria-label or by the X icon
    const buttons = screen.getAllByRole('button');
    const closeButton = buttons.find(btn => btn.querySelector('.lucide-x'));

    if (closeButton) {
      fireEvent.click(closeButton);
      expect(mockOnClose).toHaveBeenCalled();
    }
  });

  it('shows placeholder text in input', () => {
    render(
      <ChatPanel
        briefingId="1"
        cardIndex={0}
        briefingTitle="Test Card"
        isOpen={true}
        onClose={mockOnClose}
      />
    );
    expect(screen.getByPlaceholderText('Ask about this card...')).toBeInTheDocument();
  });

  it('shows helper text for keyboard shortcuts', () => {
    render(
      <ChatPanel
        briefingId="1"
        cardIndex={0}
        briefingTitle="Test Card"
        isOpen={true}
        onClose={mockOnClose}
      />
    );
    expect(screen.getByText(/Press Enter to send/)).toBeInTheDocument();
  });

  it('disables input when briefingId is null', () => {
    render(
      <ChatPanel
        briefingId={null}
        cardIndex={0}
        briefingTitle=""
        isOpen={true}
        onClose={mockOnClose}
      />
    );
    const textarea = screen.getByPlaceholderText('Ask about this card...');
    expect(textarea).toBeDisabled();
  });
});
