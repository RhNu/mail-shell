import { Show } from 'solid-js';

export function BulkToolbar(props: {
  count: number;
  trashView: boolean;
  onRead: () => void;
  onArchive: () => void;
  onTrash: () => void;
  onRestore: () => void;
}) {
  const buttonClass =
    'rounded-sm border border-zinc-300 px-3 py-1.5 text-sm hover:bg-zinc-100 dark:border-zinc-700 dark:hover:bg-zinc-800';
  return (
    <Show when={props.count > 0}>
      <div class="flex flex-wrap items-center gap-2 rounded-sm bg-zinc-100 p-2 dark:bg-zinc-900">
        <span class="px-1 text-sm text-zinc-600 dark:text-zinc-300">已选择 {props.count} 封</span>
        {props.trashView ? (
          <button type="button" class={buttonClass} onClick={props.onRestore}>
            恢复
          </button>
        ) : (
          <>
            <button type="button" class={buttonClass} onClick={props.onRead}>
              标为已读
            </button>
            <button type="button" class={buttonClass} onClick={props.onArchive}>
              归档
            </button>
            <button type="button" class={buttonClass} onClick={props.onTrash}>
              移到垃圾箱
            </button>
          </>
        )}
      </div>
    </Show>
  );
}
