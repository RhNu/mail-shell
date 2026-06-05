import { createSignal } from 'solid-js';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { InboxScreen } from './inbox-screen';

const messagesListHookState = vi.hoisted(() => ({
  refetch: vi.fn(),
}));

vi.mock('../features/messages/queries', () => ({
  useMessagesList: (query: () => { tag?: number; page?: number; limit?: number }) => ({
    get data() {
      const current = query();
      const page = current.page ?? 1;

      if (current.tag === 2) {
        return {
          items: page === 1 ? [buildMessage('msg-tag-2', 'Tag 2 first page')] : [],
          total: 1,
          limit: current.limit ?? 20,
        };
      }

      return {
        items:
          page === 3
            ? [buildMessage('msg-page-3', 'General page 3')]
            : [buildMessage('msg-page-1', 'General page 1')],
        total: 41,
        limit: current.limit ?? 20,
      };
    },
    get isLoading() {
      return false;
    },
    get isError() {
      return false;
    },
    error: undefined,
    refetch: messagesListHookState.refetch,
  }),
}));

function buildMessage(id: string, subject: string) {
  return {
    id,
    subject,
    from_address: 'sender@example.com',
    to_address: 'recipient@example.com',
    created_at: '2026-06-05T10:30:00.000Z',
  };
}

function TestHarness() {
  const [tag, setTag] = createSignal<number | undefined>(1);

  return (
    <>
      <button type="button" onClick={() => setTag(2)}>
        Switch tag
      </button>
      <InboxScreen title={<h1>Inbox</h1>} query={() => (tag() ? { tag: tag() } : {})} />
    </>
  );
}

describe('InboxScreen', () => {
  beforeEach(() => {
    vi.stubGlobal('scrollTo', vi.fn());
  });

  it('resets pagination when the backing query changes', async () => {
    render(() => <TestHarness />);

    expect(screen.getByText('General page 1')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: '3' }));
    expect(screen.getByText('General page 3')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: 'Switch tag' }));
    expect(screen.getByText('Tag 2 first page')).toBeInTheDocument();
    expect(screen.queryByText('No messages yet')).not.toBeInTheDocument();
  });
});
