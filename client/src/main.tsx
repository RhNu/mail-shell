import { render } from 'solid-js/web';
import { HashRouter, Route } from '@solidjs/router';
import App from './App';
import './index.css';

const root = document.querySelector<HTMLElement>('#root');

if (root === null) {
  throw new Error('Root container not found');
}

render(
  () => (
    <HashRouter>
      <Route path="/" component={App} />
    </HashRouter>
  ),
  root,
);
