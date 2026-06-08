import { Dialog } from '@ark-ui/solid/dialog';
import { Portal } from 'solid-js/web';
import { For } from 'solid-js';
import { X } from 'lucide-solid';
import { useMessageHeaders } from '../features/messages/queries';
import type { MessageHeadersResponse } from '../features/messages/models';

export type RawHeadersDialogProps = {
  messageId: string;
  open: boolean;
  onClose: () => void;
};

function RawHeadersDialogHeader() {
  return (
    <div class="flex items-center justify-between border-b border-zinc-200 px-4 py-3 dark:border-zinc-800">
      <Dialog.Title class="text-base font-semibold text-zinc-900 dark:text-zinc-100">
        原始字段
      </Dialog.Title>
      <Dialog.CloseTrigger
        aria-label="关闭"
        class="rounded-sm p-1 text-zinc-500 transition-colors hover:bg-zinc-200 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-100"
      >
        <X size={18} />
      </Dialog.CloseTrigger>
    </div>
  );
}

function HeaderTable(props: { headers: MessageHeadersResponse['headers'] | undefined }) {
  return (
    <table class="w-full border-collapse text-sm">
      <thead>
        <tr class="border-b border-zinc-200 dark:border-zinc-800">
          <th class="w-1/3 pb-2 text-left font-medium text-zinc-500 dark:text-zinc-400">字段</th>
          <th class="pb-2 text-left font-medium text-zinc-500 dark:text-zinc-400">值</th>
        </tr>
      </thead>
      <tbody>
        <For each={props.headers}>
          {(h) => (
            <tr class="border-b border-zinc-100 last:border-b-0 dark:border-zinc-800/50">
              <td class="py-1.5 pr-3 font-mono text-xs text-zinc-700 dark:text-zinc-300">
                {h.name}
              </td>
              <td class="py-1.5 break-all text-zinc-900 dark:text-zinc-100">{h.value}</td>
            </tr>
          )}
        </For>
      </tbody>
    </table>
  );
}

function RawHeadersDialogBody(props: { query: ReturnType<typeof useMessageHeaders> }) {
  return (
    <div class="overflow-y-auto px-4 py-3" style={{ 'max-height': 'calc(80vh - 52px)' }}>
      {props.query.isLoading ? (
        <p class="text-sm text-zinc-500 dark:text-zinc-400">加载中…</p>
      ) : props.query.isError ? (
        <p class="text-sm text-red-600 dark:text-red-400">加载失败</p>
      ) : (
        <HeaderTable headers={props.query.data?.headers} />
      )}
    </div>
  );
}

export function RawHeadersDialog(props: RawHeadersDialogProps) {
  const query = useMessageHeaders(
    () => props.messageId,
    () => props.open,
  );

  return (
    <Dialog.Root
      open={props.open}
      onOpenChange={(e) => {
        if (!e.open) props.onClose();
      }}
    >
      <Portal>
        <Dialog.Backdrop class="fixed inset-0 z-50 bg-black/50" />
        <Dialog.Positioner class="fixed inset-0 z-50 flex items-center justify-center">
          <Dialog.Content
            class="max-h-[80vh] w-full max-w-2xl overflow-hidden rounded-sm border border-zinc-200 bg-zinc-50 shadow-xl dark:border-zinc-800 dark:bg-zinc-950"
            aria-label="原始字段"
          >
            <RawHeadersDialogHeader />
            <RawHeadersDialogBody query={query} />
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  );
}
