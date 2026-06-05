import { describe, expect, it } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { QueryClientProvider } from '@tanstack/solid-query';
import { HashRouter } from '@solidjs/router';
import App from './App';
import { queryClient } from './lib/query-client';

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

    expect(await screen.findByRole('heading', { name: 'Inbox' })).toBeInTheDocument();
  });

  it('renders the message detail route for a selected message', async () => {
    renderApp('/messages/msg-123');

    expect(await screen.findByRole('heading', { name: 'Message detail' })).toBeInTheDocument();
  });

  it('renders the tag inbox route for a selected tag', async () => {
    renderApp('/tags/42');

    expect(await screen.findByRole('heading', { name: 'Tagged inbox' })).toBeInTheDocument();
  });

  it('renders a not found route for unknown paths', async () => {
    renderApp('/missing');

    expect(await screen.findByRole('heading', { name: 'Route not found' })).toBeInTheDocument();
  });
});
