import { createSignal } from 'solid-js';
import { fireEvent, render, screen, within } from '@solidjs/testing-library';
import { beforeEach, expect, it, vi } from 'vitest';
import { InboxScreen } from './inbox-screen';

const messagesListHookState = vi.hoisted(() => ({
  refetch: vi.fn(),
  updateMailbox: vi.fn(),
  deleteMessage: vi.fn(),
  updateState: vi.fn(),
  updateStates: vi.fn(),
  markAllRead: vi.fn(),
  updateMailboxPending: false,
  deleteMessagePending: false,
}));

// oxlint-disable-next-line max-lines-per-function
vi.mock('../features/messages/queries', () => ({
  useMessagesList: (query: () => { label?: number; page?: number; limit?: number }) => ({
    get data() {
      const current = query();
      const page = current.page ?? 1;

      if (current.label === 2) {
        return {
          items: page === 1 ? [buildMessage('msg-label-2', 'Label 2 first page')] : [],
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
  useUpdateMessageState: () => ({
    mutate: messagesListHookState.updateState,
    isPending: false,
    isError: false,
    error: undefined,
  }),
  useUpdateMessagesState: () => ({
    mutate: messagesListHookState.updateStates,
    isPending: false,
    isError: false,
    error: undefined,
  }),
  useMarkAllMessagesRead: () => ({
    mutate: messagesListHookState.markAllRead,
    isPending: false,
    isError: false,
    error: undefined,
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
    is_read: true,
    is_starred: false,
    attachment_count: 0,
    labels: [],
    created_at: '2026-06-05T10:30:00.000Z',
  };
}

function TestHarness() {
  const [label, setLabel] = createSignal<number | undefined>(1);

  return (
    <>
      <button type="button" onClick={() => setLabel(2)}>
        Switch label
      </button>
      <InboxScreen title={<h1>Inbox</h1>} query={() => (label() ? { label: label() } : {})} />
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
  messagesListHookState.updateState.mockReset();
  messagesListHookState.updateStates.mockReset();
  messagesListHookState.markAllRead.mockReset();
  messagesListHookState.updateMailboxPending = false;
  messagesListHookState.deleteMessagePending = false;
});

it('resets pagination when the backing query changes', async () => {
  render(() => <TestHarness />);

  expect(screen.getByText('General page 1')).toBeInTheDocument();

  await fireEvent.click(screen.getByRole('button', { name: '3' }));
  expect(screen.getByText('General page 3')).toBeInTheDocument();

  await fireEvent.click(screen.getByRole('button', { name: 'Switch label' }));
  expect(screen.getByText('Label 2 first page')).toBeInTheDocument();
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

it('only shows selection controls after entering selection mode', async () => {
  render(() => <InboxScreen title={<h1>Inbox</h1>} query={() => ({ mailbox: 'inbox' })} />);

  expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: '全选' })).not.toBeInTheDocument();

  await fireEvent.click(screen.getByRole('button', { name: '选择' }));

  expect(screen.getByRole('checkbox')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: '全选' })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: '全部已读' })).toBeInTheDocument();
});

it('selects every visible message and can clear the selection', async () => {
  render(() => <InboxScreen title={<h1>Inbox</h1>} query={() => ({ mailbox: 'inbox' })} />);
  await fireEvent.click(screen.getByRole('button', { name: '选择' }));

  await fireEvent.click(screen.getByRole('button', { name: '全选' }));
  expect(screen.getByRole('checkbox')).toBeChecked();
  expect(screen.getByText('已选择 1 封')).toBeInTheDocument();

  await fireEvent.click(screen.getByRole('button', { name: '取消全选' }));
  expect(screen.getByRole('checkbox')).not.toBeChecked();
});

it('marks every message in the current filtered view as read', async () => {
  render(() => (
    <InboxScreen
      title={<h1>Unread</h1>}
      query={() => ({ mailbox: 'inbox', read: false, label: 3 })}
    />
  ));
  await fireEvent.click(screen.getByRole('button', { name: '选择' }));
  await fireEvent.click(screen.getByRole('button', { name: '全部已读' }));

  expect(messagesListHookState.markAllRead).toHaveBeenCalledWith(
    expect.objectContaining({ mailbox: 'inbox', read: false, label: 3 }),
    expect.any(Object),
  );
});

it('permanently deletes a trashed message from the list action menu after confirmation', async () => {
  const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true);

  render(() => (
    <InboxScreen title={<h1>Trash</h1>} query={() => ({ mailbox: 'inbox', trashed: true })} />
  ));

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
