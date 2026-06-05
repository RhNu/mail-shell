import type { JSX } from "solid-js";
import { messageDetailHref, tagInboxHref } from "./routes";
import { Dialog } from "@ark-ui/solid/dialog";
import { Portal } from "solid-js/web";
import { Menu, X, Inbox, Tag, FileText } from "lucide-solid";

type AppShellProps = {
  children?: JSX.Element;
};

export function AppShell(props: AppShellProps) {
  const NavLinks = () => (
    <nav aria-label="Primary" class="flex flex-col gap-2">
      <a
        href="#/"
        class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-zinc-600 transition-colors hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-50"
      >
        <Inbox size={18} />
        <span>Inbox</span>
      </a>
      <a
        href={`#${messageDetailHref("sample-message")}`}
        class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-zinc-600 transition-colors hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-50"
      >
        <FileText size={18} />
        <span>Message detail</span>
      </a>
      <a
        href={`#${tagInboxHref("sample-tag")}`}
        class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-zinc-600 transition-colors hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-50"
      >
        <Tag size={18} />
        <span>Tagged inbox</span>
      </a>
    </nav>
  );

  return (
    <div class="min-h-screen lg:grid lg:grid-cols-[240px_1fr] bg-zinc-50 dark:bg-zinc-950">
      {/* Mobile Top Bar */}
      <header class="flex h-14 items-center justify-between border-b border-zinc-200 px-4 lg:hidden dark:border-zinc-800">
        <div class="text-sm font-semibold uppercase tracking-widest text-zinc-900 dark:text-zinc-100">
          mail-shell
        </div>
        <Dialog.Root>
          <Dialog.Trigger class="rounded-md p-2 text-zinc-600 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-800">
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
                  <Dialog.CloseTrigger class="rounded-md p-2 text-zinc-600 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-800">
                    <X size={20} />
                    <span class="sr-only">Close Menu</span>
                  </Dialog.CloseTrigger>
                </div>
                <NavLinks />
              </Dialog.Content>
            </Dialog.Positioner>
          </Portal>
        </Dialog.Root>
      </header>

      {/* Desktop Sidebar */}
      <aside class="hidden border-r border-zinc-200 bg-zinc-50 lg:flex lg:flex-col dark:border-zinc-800 dark:bg-zinc-950">
        <div class="flex h-14 items-center border-b border-zinc-200 px-6 dark:border-zinc-800">
          <p class="text-sm font-semibold uppercase tracking-[0.2em] text-zinc-900 dark:text-zinc-100">
            mail-shell
          </p>
        </div>
        <div class="flex-1 p-4">
          <NavLinks />
        </div>
      </aside>

      {/* Main Content Area */}
      <main class="flex flex-col min-w-0">
        <div class="mx-auto w-full max-w-5xl p-6">{props.children}</div>
      </main>
    </div>
  );
}
