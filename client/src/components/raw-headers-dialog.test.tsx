import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { RawHeadersDialog } from './raw-headers-dialog';

const headersQueryState = vi.hoisted(() => ({
  value: {
    isLoading: false,
    isError: false,
    error: undefined as Error | undefined,
    data: undefined as { headers: Array<{ name: string; value: string }> } | undefined,
  },
}));

vi.mock('../features/messages/queries', () => ({
  useMessageHeaders: () => headersQueryState.value,
}));

describe('RawHeadersDialog', () => {
  beforeEach(() => {
    headersQueryState.value = {
      isLoading: false,
      isError: false,
      error: undefined,
      data: undefined,
    };
  });

  afterEach(() => {
    cleanup();
  });

  it('renders nothing when closed', () => {
    render(() => (
      <RawHeadersDialog messageId="msg-1" open={false} onClose={() => {}} />
    ));

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('renders headers in a table when open with data', () => {
    headersQueryState.value = {
      isLoading: false,
      isError: false,
      error: undefined,
      data: {
        headers: [
          { name: 'From', value: 'sender@example.com' },
          { name: 'Subject', value: 'Hello' },
        ],
      },
    };

    render(() => (
      <RawHeadersDialog messageId="msg-1" open={true} onClose={() => {}} />
    ));

    expect(screen.getByText('From')).toBeInTheDocument();
    expect(screen.getByText('sender@example.com')).toBeInTheDocument();
    expect(screen.getByText('Subject')).toBeInTheDocument();
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });

  it('shows a loading indicator while fetching', () => {
    headersQueryState.value = {
      isLoading: true,
      isError: false,
      error: undefined,
      data: undefined,
    };

    render(() => (
      <RawHeadersDialog messageId="msg-1" open={true} onClose={() => {}} />
    ));

    expect(screen.getByText('加载中…')).toBeInTheDocument();
  });

  it('shows an error message on failure', () => {
    headersQueryState.value = {
      isLoading: false,
      isError: true,
      error: new Error('network error'),
      data: undefined,
    };

    render(() => (
      <RawHeadersDialog messageId="msg-1" open={true} onClose={() => {}} />
    ));

    expect(screen.getByText('加载失败')).toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', async () => {
    const onClose = vi.fn();
    headersQueryState.value = {
      isLoading: false,
      isError: false,
      error: undefined,
      data: { headers: [{ name: 'From', value: 'a@b.com' }] },
    };

    render(() => <RawHeadersDialog messageId="msg-1" open={true} onClose={onClose} />);

    await fireEvent.click(screen.getByRole('button', { name: '关闭' }));
    expect(onClose).toHaveBeenCalled();
  });
});
