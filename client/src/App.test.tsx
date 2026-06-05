import type { JSX } from 'solid-js';
import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { QueryClientProvider } from '@tanstack/solid-query';
import { HashRouter } from '@solidjs/router';
import App from './App';
import { queryClient } from './lib/query-client';

vi.mock('./app/screens/inbox-route', () => ({
  InboxRoute: () => <h1>Inbox screen</h1>,
}));

vi.mock('./app/screens/message-detail-route', () => ({
  MessageDetailRoute: () => <h1>Message detail screen</h1>,
}));

vi.mock('./app/screens/tagged-inbox-route', () => ({
  TaggedInboxRoute: () => <h1>Tagged inbox screen</h1>,
}));

vi.mock('./app/screens/not-found-route', () => ({
  NotFoundRoute: () => <h1>Not found screen</h1>,
}));

vi.mock('./app/app-shell', () => ({
  AppShell: (props: { children?: JSX.Element }) => <div>{props.children}</div>,
}));

function renderApp(location: string) {
  window.location.hash = `#${location}`;
  return render(() => (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <App />
      </HashRouter>
    </QueryClientProvider>
  ));
}

describe('App routes', () => {
  it('renders the inbox route at the root location', async () => {
    renderApp('/');

    expect(await screen.findByRole('heading', { name: 'Inbox screen' })).toBeInTheDocument();
  });

  it('renders the message detail route for a selected message', async () => {
    renderApp('/messages/msg-123');

    expect(
      await screen.findByRole('heading', { name: 'Message detail screen' }),
    ).toBeInTheDocument();
  });

  it('renders the tag inbox route for a selected tag', async () => {
    renderApp('/tags/42');

    expect(await screen.findByRole('heading', { name: 'Tagged inbox screen' })).toBeInTheDocument();
  });

  it('renders a not found route for unknown paths', async () => {
    renderApp('/missing');

    expect(await screen.findByRole('heading', { name: 'Not found screen' })).toBeInTheDocument();
  });
});
