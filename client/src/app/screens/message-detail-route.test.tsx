import { Route, HashRouter } from '@solidjs/router';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { beforeEach, expect, it, vi } from 'vitest';
import { HttpResponseError, NetworkRequestError } from '../../api/core/errors';
import { MessageDetailRoute } from './message-detail-route';

type MockMessageDetailQuery = {
  isLoading: boolean;
  isError: boolean;
  error?: Error;
  data?: {
    id: string;
    subject: string;
    from_address: string;
    to_address?: string | null;
    envelope_to: string;
    created_at: string;
    mailbox: 'inbox' | 'archive';
    body_text: string;
    body_html: string;
    attachments: Array<{ id: string }>;
  };
  refetch: ReturnType<typeof vi.fn>;
};

const messageDetailQueryState = vi.hoisted(() => ({
  value: {
    isLoading: false,
    isError: false,
    error: undefined,
    data: undefined,
    refetch: vi.fn(),
  } as MockMessageDetailQuery,
  updateMailbox: vi.fn(),
  deleteMessage: vi.fn(),
  updateMailboxState: {
    isPending: false,
    isError: false,
    error: undefined as Error | undefined,
  },
  deleteMessageState: {
    isPending: false,
    isError: false,
    error: undefined as Error | undefined,
  },
}));

vi.mock('../../features/messages/queries', () => ({
  useMessageDetail: () => messageDetailQueryState.value,
  useMessageHeaders: () => ({
    isLoading: false,
    isError: false,
    error: undefined,
    data: { headers: [] },
  }),
  useUpdateMessageMailbox: () => ({
    mutate: messageDetailQueryState.updateMailbox,
    get isPending() {
      return messageDetailQueryState.updateMailboxState.isPending;
    },
    get isError() {
      return messageDetailQueryState.updateMailboxState.isError;
    },
    get error() {
      return messageDetailQueryState.updateMailboxState.error;
    },
  }),
  useDeleteMessage: () => ({
    mutate: messageDetailQueryState.deleteMessage,
    get isPending() {
      return messageDetailQueryState.deleteMessageState.isPending;
    },
    get isError() {
      return messageDetailQueryState.deleteMessageState.isError;
    },
    get error() {
      return messageDetailQueryState.deleteMessageState.error;
    },
  }),
}));

function renderRoute(path = '/messages/msg-1') {
  window.location.hash = `#${path}`;

  return render(() => (
    <HashRouter>
      <Route path="/messages/:messageId" component={MessageDetailRoute} />
    </HashRouter>
  ));
}

async function selectMenuItem(name: string) {
  const item = await screen.findByRole('menuitem', { name });
  await fireEvent.pointerDown(item, { pointerType: 'mouse' });
  await fireEvent.click(item);
}

const baseMessageDetailData = {
  id: 'msg-1',
  subject: 'Release plan',
  from_address: 'alice@example.com',
  to_address: 'bob@example.com',
  envelope_to: 'delivered@example.com',
  created_at: '2026-06-05T10:30:00.000Z',
  mailbox: 'inbox' as const,
  body_text: 'Plain fallback',
  body_html: '<p>Hello</p>',
  attachments: [],
};

function setSuccessQuery(bodyHtml = baseMessageDetailData.body_html) {
  messageDetailQueryState.value = {
    isLoading: false,
    isError: false,
    error: undefined,
    data: {
      ...baseMessageDetailData,
      body_html: bodyHtml,
    },
    refetch: vi.fn(),
  };
}

function setErrorQuery(error: Error) {
  messageDetailQueryState.value = {
    isLoading: false,
    isError: true,
    error,
    data: undefined,
    refetch: vi.fn(),
  };
}

function setMissingToHeaderQuery() {
  messageDetailQueryState.value = {
    ...messageDetailQueryState.value,
    data: {
      ...baseMessageDetailData,
      to_address: null,
    },
  };
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.stubGlobal('scrollTo', vi.fn());
  setSuccessQuery();
  messageDetailQueryState.updateMailbox.mockReset();
  messageDetailQueryState.deleteMessage.mockReset();
  messageDetailQueryState.updateMailboxState = {
    isPending: false,
    isError: false,
    error: undefined,
  };
  messageDetailQueryState.deleteMessageState = {
    isPending: false,
    isError: false,
    error: undefined,
  };
});

it('gates remote resources behind an explicit user action', async () => {
  setSuccessQuery('<p>Hello</p><img src="https://tracker.test/pixel.png" alt="Tracker pixel" />');

  const view = renderRoute();

  expect(screen.getByRole('button', { name: '加载远程资源' })).toBeInTheDocument();
  expect(view.container.querySelector('img')).toBeNull();

  await fireEvent.click(screen.getByRole('button', { name: '加载远程资源' }));

  expect(screen.queryByRole('button', { name: '加载远程资源' })).not.toBeInTheDocument();
  expect(view.container.querySelector('img')?.getAttribute('src')).toBe(
    'https://tracker.test/pixel.png',
  );
});

it('falls back to the envelope recipient when the To header is missing', () => {
  setMissingToHeaderQuery();

  renderRoute();

  expect(screen.getByText('delivered@example.com')).toBeInTheDocument();
});

it('shows the retry banner without a not-found empty state for network errors', () => {
  setErrorQuery(new NetworkRequestError('offline'));

  renderRoute();

  expect(screen.getByText('offline')).toBeInTheDocument();
  expect(screen.queryByText('邮件未找到')).not.toBeInTheDocument();
});

it('renders the not-found state for 404 responses', () => {
  setErrorQuery(new HttpResponseError(404, 'Not Found', { error: 'message missing' }));

  renderRoute();

  expect(screen.getByText('邮件未找到')).toBeInTheDocument();
  expect(screen.queryByText('message missing')).not.toBeInTheDocument();
});

it('uses returnTo search param for the back link', () => {
  renderRoute('/messages/msg-1?returnTo=%2Farchive');

  expect(screen.getByRole('link', { name: /返回/u })).toHaveAttribute('href', '#/archive');
});

it('restores archived messages from the action menu', async () => {
  messageDetailQueryState.value = {
    ...messageDetailQueryState.value,
    data: {
      ...baseMessageDetailData,
      mailbox: 'archive',
    },
  };

  renderRoute();

  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
  await selectMenuItem('移回收件箱');

  expect(messageDetailQueryState.updateMailbox).toHaveBeenCalled();
  expect(messageDetailQueryState.updateMailbox.mock.calls[0][0]).toEqual({
    id: 'msg-1',
    mailbox: 'inbox',
  });
});

it('shows detail action errors and disables actions while pending', () => {
  messageDetailQueryState.updateMailboxState = {
    isPending: true,
    isError: true,
    error: new Error('归档失败'),
  };

  renderRoute();

  expect(screen.getByText('归档失败')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: '更多操作' })).toBeDisabled();
});

it('returns to the source route after a mailbox action succeeds', async () => {
  messageDetailQueryState.updateMailbox.mockImplementation((_variables, options) => {
    options.onSuccess();
  });

  renderRoute('/messages/msg-1?returnTo=%2Farchive');

  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
  await selectMenuItem('归档');

  await waitFor(() => expect(window.location.hash).toBe('#/archive'));
});

it('deletes from detail after confirmation and returns to the source route', async () => {
  const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true);
  messageDetailQueryState.deleteMessage.mockImplementation((_variables, options) => {
    options.onSuccess();
  });

  renderRoute('/messages/msg-1?returnTo=%2Ftags%2F7');

  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
  await selectMenuItem('永久删除');
  await fireEvent.click(
    within(screen.getByRole('dialog', { name: '永久删除邮件' })).getByRole('button', {
      name: '永久删除',
    }),
  );

  expect(confirm).not.toHaveBeenCalled();
  expect(messageDetailQueryState.deleteMessage.mock.calls[0][0]).toEqual({ id: 'msg-1' });
  await waitFor(() => expect(window.location.hash).toBe('#/tags/7'));
});
