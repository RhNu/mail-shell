import { Route, HashRouter } from '@solidjs/router';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { beforeEach, describe, expect, it, vi } from 'vitest';
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
    to_address: string;
    created_at: string;
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
}));

vi.mock('../../features/messages/queries', () => ({
  useMessageDetail: () => messageDetailQueryState.value,
}));

function renderRoute(path = '/messages/msg-1') {
  window.location.hash = `#${path}`;

  return render(() => (
    <HashRouter>
      <Route path="/messages/:messageId" component={MessageDetailRoute} />
    </HashRouter>
  ));
}

const baseMessageDetailData = {
  id: 'msg-1',
  subject: 'Release plan',
  from_address: 'alice@example.com',
  to_address: 'bob@example.com',
  created_at: '2026-06-05T10:30:00.000Z',
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

describe('MessageDetailRoute', () => {
  beforeEach(() => {
    setSuccessQuery();
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
});
