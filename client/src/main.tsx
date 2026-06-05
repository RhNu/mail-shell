import { render } from 'solid-js/web';
import { HashRouter } from '@solidjs/router';
import { QueryClientProvider } from '@tanstack/solid-query';
import App from './App';
import { queryClient } from './lib/query-client';
import './index.css';

const root = document.querySelector<HTMLElement>('#root');

if (root === null) {
  throw new Error('Root container not found');
}

render(
  () => (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <App />
      </HashRouter>
    </QueryClientProvider>
  ),
  root,
);
