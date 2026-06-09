import { createSignal } from 'solid-js';
import { fireEvent, render, screen, within } from '@solidjs/testing-library';
import { beforeEach, expect, it, vi } from 'vitest';
import { InboxScreen } from './inbox-screen';

const messagesListHookState = vi.hoisted(() => ({
  refetch: vi.fn(),
  updateMailbox: vi.fn(),
  deleteMessage: vi.fn(),
  updateMailboxPending: false,
  deleteMessagePending: false,
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
  useUpdateMessageMailbox: () => ({
    mutate: messagesListHookState.updateMailbox,
    get isPending() {
      return messagesListHookState.updateMailboxPending;
    },
  }),
  useDeleteMessage: () => ({
    mutate: messagesListHookState.deleteMessage,
    get isPending() {
      return messagesListHookState.deleteMessagePending;
    },
  }),
}));

function buildMessage(id: string, subject: string) {
  return {
    id,
    subject,
    from_address: 'sender@example.com',
    to_address: 'recipient@example.com',
    envelope_to: 'recipient@example.com',
    mailbox: 'inbox',
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

async function selectMenuItem(name: string) {
  const item = await screen.findByRole('menuitem', { name });
  await fireEvent.pointerDown(item, { pointerType: 'mouse' });
  await fireEvent.click(item);
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.stubGlobal('scrollTo', vi.fn());
  messagesListHookState.updateMailbox.mockReset();
  messagesListHookState.deleteMessage.mockReset();
  messagesListHookState.updateMailboxPending = false;
  messagesListHookState.deleteMessagePending = false;
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

it('archives an inbox message from the list action menu', async () => {
  render(() => <InboxScreen title={<h1>Inbox</h1>} query={() => ({ mailbox: 'inbox' })} />);

  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
  await selectMenuItem('归档');

  expect(messagesListHookState.updateMailbox).toHaveBeenCalledWith({
    id: 'msg-page-1',
    mailbox: 'archive',
  });
});

it('permanently deletes a message from the list action menu after confirmation', async () => {
  const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true);

  render(() => <InboxScreen title={<h1>Inbox</h1>} query={() => ({ mailbox: 'inbox' })} />);

  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
  await selectMenuItem('永久删除');
  await fireEvent.click(
    within(screen.getByRole('dialog', { name: '永久删除邮件' })).getByRole('button', {
      name: '永久删除',
    }),
  );

  expect(confirm).not.toHaveBeenCalled();
  expect(messagesListHookState.deleteMessage).toHaveBeenCalledWith({
    id: 'msg-page-1',
  });
});

it('disables list action menus while a message mutation is pending', () => {
  messagesListHookState.deleteMessagePending = true;

  render(() => <InboxScreen title={<h1>Inbox</h1>} query={() => ({ mailbox: 'inbox' })} />);

  expect(screen.getByRole('button', { name: '更多操作' })).toBeDisabled();
});
