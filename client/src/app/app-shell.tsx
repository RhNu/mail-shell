import { createSignal, type JSX } from 'solid-js';
import { useLocation } from '@solidjs/router';
import { Dialog } from '@ark-ui/solid/dialog';
import { Portal } from 'solid-js/web';
import { Menu, X, Inbox } from 'lucide-solid';
import { ConnectivityBanner } from '../components/connectivity-banner';
import { TagNav } from '../components/tag-nav';
import { PwaUpdateController } from '../features/system/pwa-update-controller';
import { useConnectivityStatus } from '../features/system/use-connectivity-status';

function navCls(active: boolean) {
  return [
    'flex items-center gap-3 rounded-sm px-3 py-2 text-sm font-medium transition-colors',
    active
      ? 'bg-zinc-200 text-zinc-900 dark:bg-zinc-800 dark:text-zinc-100'
      : 'text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-50',
  ].join(' ');
}

function MobileHeader() {
  const location = useLocation();
  const [open, setOpen] = createSignal(false);
  const isInboxActive = () => location.pathname === '/';
  return (
    <header class="flex h-14 items-center justify-between border-b border-zinc-200 px-4 lg:hidden dark:border-zinc-800">
      <div class="text-sm font-semibold uppercase tracking-widest text-zinc-900 dark:text-zinc-100">
        Mail Shell
      </div>
      <Dialog.Root open={open()} onOpenChange={(details) => setOpen(details.open)}>
        <Dialog.Trigger class="rounded-sm p-2 text-zinc-600 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-800">
          <Menu size={20} />
          <span class="sr-only">Open Menu</span>
        </Dialog.Trigger>
        <Portal>
          <Dialog.Backdrop class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
          <Dialog.Positioner class="fixed inset-0 z-50 flex justify-end">
            <Dialog.Content class="h-full w-3/4 max-w-sm border-l border-zinc-200 bg-zinc-50 p-6 shadow-xl data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right sm:w-80 dark:border-zinc-800 dark:bg-zinc-950">
              <div class="mb-6 flex items-center justify-between">
                <Dialog.Title class="text-sm font-semibold uppercase tracking-widest text-zinc-900 dark:text-zinc-100">
                  Menu
                </Dialog.Title>
                <Dialog.CloseTrigger class="rounded-sm p-2 text-zinc-600 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-800">
                  <X size={20} />
                  <span class="sr-only">Close Menu</span>
                </Dialog.CloseTrigger>
              </div>
              <nav aria-label="Primary" class="flex flex-col gap-1">
                <a href="#/" class={navCls(isInboxActive())} onClick={() => setOpen(false)}>
                  <Inbox size={18} />
                  <span>Inbox</span>
                </a>
              </nav>
              <div class="my-4 border-t border-zinc-200 dark:border-zinc-800" />
              <nav aria-label="Tags">
                <TagNav onNavigate={() => setOpen(false)} />
              </nav>
            </Dialog.Content>
          </Dialog.Positioner>
        </Portal>
      </Dialog.Root>
    </header>
  );
}

function DesktopSidebar() {
  const location = useLocation();
  const isInboxActive = () => location.pathname === '/';
  return (
    <aside class="hidden border-r border-zinc-200 bg-zinc-50 lg:flex lg:flex-col dark:border-zinc-800 dark:bg-zinc-950">
      <div class="flex h-14 items-center border-b border-zinc-200 px-6 dark:border-zinc-800">
        <p class="text-sm font-semibold uppercase tracking-[0.2em] text-zinc-900 dark:text-zinc-100">
          Mail Shell
        </p>
      </div>
      <div class="flex-1 overflow-y-auto p-4">
        <nav aria-label="Primary" class="mb-4 flex flex-col gap-0.5">
          <a href="#/" class={navCls(isInboxActive())}>
            <Inbox size={18} />
            <span>Inbox</span>
          </a>
        </nav>
        <div class="mb-4 border-t border-zinc-200 dark:border-zinc-800" />
        <nav aria-label="Tags">
          <TagNav />
        </nav>
      </div>
    </aside>
  );
}

export function AppShell(props: { children?: JSX.Element }): JSX.Element {
  const connectivity = useConnectivityStatus();

  return (
    <div class="min-h-screen bg-zinc-50 lg:grid lg:grid-cols-[240px_1fr] dark:bg-zinc-950">
      <PwaUpdateController />
      <MobileHeader />
      <DesktopSidebar />
      <main class="flex min-w-0 flex-col">
        <ConnectivityBanner status={connectivity.status()} />
        <div class="mx-auto w-full max-w-5xl p-4 sm:p-6">{props.children}</div>
      </main>
    </div>
  );
}
