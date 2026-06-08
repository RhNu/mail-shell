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

function resetHeadersQuery() {
  headersQueryState.value = {
    isLoading: false,
    isError: false,
    error: undefined,
    data: undefined,
  };
}

function renderDialog(open = true, onClose = () => {}) {
  render(() => <RawHeadersDialog messageId="msg-1" open={open} onClose={onClose} />);
}

beforeEach(() => {
  resetHeadersQuery();
});

afterEach(() => {
  cleanup();
});

describe('RawHeadersDialog visibility', () => {
  it('renders nothing when closed', () => {
    renderDialog(false);

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

    renderDialog();

    expect(screen.getByText('From')).toBeInTheDocument();
    expect(screen.getByText('sender@example.com')).toBeInTheDocument();
    expect(screen.getByText('Subject')).toBeInTheDocument();
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });
});

describe('RawHeadersDialog query states', () => {
  it('shows a loading indicator while fetching', () => {
    headersQueryState.value = {
      isLoading: true,
      isError: false,
      error: undefined,
      data: undefined,
    };

    renderDialog();

    expect(screen.getByText('加载中…')).toBeInTheDocument();
  });

  it('shows an error message on failure', () => {
    headersQueryState.value = {
      isLoading: false,
      isError: true,
      error: new Error('network error'),
      data: undefined,
    };

    renderDialog();

    expect(screen.getByText('加载失败')).toBeInTheDocument();
  });
});

describe('RawHeadersDialog actions', () => {
  it('calls onClose when close button is clicked', async () => {
    const onClose = vi.fn();
    headersQueryState.value = {
      isLoading: false,
      isError: false,
      error: undefined,
      data: { headers: [{ name: 'From', value: 'a@b.com' }] },
    };

    renderDialog(true, onClose);

    await fireEvent.click(screen.getByRole('button', { name: '关闭' }));
    expect(onClose).toHaveBeenCalled();
  });
});
