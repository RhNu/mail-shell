import type { JSX } from 'solid-js';
import { Show } from 'solid-js';

type PwaUpdateModalProps = {
  open: boolean;
  onLater: () => void;
  onUpdate: () => void;
};

export function PwaUpdateModal(props: PwaUpdateModalProps): JSX.Element {
  return (
    <Show when={props.open}>
      <div class="fixed inset-0 z-50 flex items-center justify-center bg-zinc-950/55 p-4 backdrop-blur-sm">
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="pwa-update-title"
          class="w-full max-w-md rounded-sm border border-zinc-200 bg-white p-6 shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        >
          <div class="flex flex-col gap-2">
            <h2 id="pwa-update-title" class="text-lg font-semibold text-zinc-900 dark:text-zinc-50">
              发现新版本
            </h2>
            <p class="text-sm text-zinc-600 dark:text-zinc-300">
              有新内容可用。立即更新将刷新当前页面。
            </p>
          </div>
          <div class="mt-6 flex justify-end gap-3">
            <button
              type="button"
              onClick={() => props.onLater()}
              class="rounded-sm border border-zinc-300 px-3 py-2 text-sm font-medium text-zinc-700 transition-colors hover:bg-zinc-100 dark:border-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-900"
            >
              稍后
            </button>
            <button
              type="button"
              onClick={() => props.onUpdate()}
              class="rounded-sm bg-sky-600 px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-sky-500"
            >
              立即更新
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
}
