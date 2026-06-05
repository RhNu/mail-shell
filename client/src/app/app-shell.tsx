import type { JSX } from 'solid-js';
import { messageDetailHref, tagInboxHref } from './routes';

type AppShellProps = {
  children?: JSX.Element;
};

export function AppShell(props: AppShellProps) {
  return (
    <main class="min-h-screen bg-stone-950 text-stone-100">
      <div class="mx-auto flex min-h-screen max-w-5xl flex-col gap-10 px-6 py-10">
        <header class="flex flex-col gap-4 border-b border-stone-800 pb-6">
          <div>
            <p class="text-sm uppercase tracking-[0.3em] text-stone-400">mail-shell</p>
            <h1 class="mt-2 text-3xl font-semibold tracking-tight">Client route skeleton</h1>
          </div>
          <nav aria-label="Primary" class="flex flex-wrap gap-4 text-sm text-stone-300">
            <a href="#/" class="hover:text-white">
              Inbox
            </a>
            <a href={`#${messageDetailHref('sample-message')}`} class="hover:text-white">
              Message detail
            </a>
            <a href={`#${tagInboxHref('sample-tag')}`} class="hover:text-white">
              Tagged inbox
            </a>
          </nav>
        </header>
        {props.children}
      </div>
    </main>
  );
}
