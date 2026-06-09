import type { JSX } from 'solid-js';
import { Route, HashRouter } from '@solidjs/router';
import { QueryClientProvider } from '@tanstack/solid-query';
import { render, screen } from '@solidjs/testing-library';
import { describe, expect, it, vi } from 'vitest';
import { AppShell } from './app-shell';
import { queryClient } from '../lib/query-client';

vi.mock('../components/tag-nav', () => ({
  TagNav: () => <div data-testid="tag-nav" />,
}));

vi.mock('../features/system/pwa-update-controller', () => ({
  PwaUpdateController: () => null,
}));

vi.mock('../features/system/use-connectivity-status', () => ({
  useConnectivityStatus: () => ({ status: () => 'online' }),
}));

function renderShell(path: string, children: JSX.Element = <div>Content</div>) {
  window.location.hash = `#${path}`;
  return render(() => (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <Route path="/" component={(props) => <AppShell>{props.children}</AppShell>}>
          <Route path={path} component={() => children} />
        </Route>
      </HashRouter>
    </QueryClientProvider>
  ));
}

describe('AppShell navigation', () => {
  it('highlights archive navigation separately from inbox', async () => {
    renderShell('/archive');

    expect(await screen.findByText('Content')).toBeInTheDocument();
    for (const archiveLink of screen.getAllByRole('link', { name: '归档' })) {
      expect(archiveLink).toHaveClass('bg-zinc-200');
    }
    for (const inboxLink of screen.getAllByRole('link', { name: '收件箱' })) {
      expect(inboxLink).not.toHaveClass('bg-zinc-200');
    }
  });
});
