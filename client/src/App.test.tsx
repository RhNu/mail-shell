import type { JSX } from 'solid-js';
import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { QueryClientProvider } from '@tanstack/solid-query';
import { HashRouter } from '@solidjs/router';
import App from './App';
import { queryClient } from './lib/query-client';

vi.mock('./app/screens', () => ({
  InboxRoute: () => <h1>Inbox screen</h1>,
  ArchiveRoute: () => <h1>Archive screen</h1>,
  MessageDetailRoute: () => <h1>Message detail screen</h1>,
  LabelInboxRoute: () => <h1>Label inbox screen</h1>,
  NotFoundRoute: () => <h1>Not found screen</h1>,
  ClassificationRoute: () => null,
  FacetsRoute: () => null,
  SavedViewRoute: () => null,
  SearchRoute: () => null,
  StarredRoute: () => null,
  TrashRoute: () => null,
  UnreadRoute: () => null,
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

  it('renders the archive route', async () => {
    renderApp('/archive');

    expect(await screen.findByRole('heading', { name: 'Archive screen' })).toBeInTheDocument();
  });

  it('renders the label inbox route for a selected label', async () => {
    renderApp('/labels/42');

    expect(await screen.findByRole('heading', { name: 'Label inbox screen' })).toBeInTheDocument();
  });

  it('renders a not found route for unknown paths', async () => {
    renderApp('/missing');

    expect(await screen.findByRole('heading', { name: 'Not found screen' })).toBeInTheDocument();
  });
});
