import { Show, type JSX } from 'solid-js';

const buttonClass =
  'rounded-sm border border-zinc-300 px-3 py-1.5 text-sm hover:bg-zinc-100 disabled:cursor-not-allowed disabled:opacity-50 dark:border-zinc-700 dark:hover:bg-zinc-800';

function ToolbarButton(props: {
  children: JSX.Element;
  disabled: boolean;
  class?: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      class={`${buttonClass} ${props.class ?? ''}`}
      disabled={props.disabled}
      onClick={() => props.onClick()}
    >
      {props.children}
    </button>
  );
}

type BulkToolbarProps = {
  active: boolean;
  count: number;
  allSelected: boolean;
  hasMessages: boolean;
  disabled: boolean;
  trashView: boolean;
  onToggleAll: () => void;
  onMarkAllRead: () => void;
  onRead: () => void;
  onArchive: () => void;
  onTrash: () => void;
  onRestore: () => void;
};

function SelectedActions(props: BulkToolbarProps) {
  const disabled = () => props.count === 0 || props.disabled;
  return (
    <Show
      when={!props.trashView}
      fallback={
        <ToolbarButton disabled={disabled()} onClick={props.onRestore}>
          恢复
        </ToolbarButton>
      }
    >
      <ToolbarButton disabled={disabled()} onClick={props.onRead}>
        标为已读
      </ToolbarButton>
      <ToolbarButton disabled={disabled()} onClick={props.onArchive}>
        归档
      </ToolbarButton>
      <ToolbarButton disabled={disabled()} onClick={props.onTrash}>
        移到垃圾箱
      </ToolbarButton>
    </Show>
  );
}

export function BulkToolbar(props: BulkToolbarProps) {
  return (
    <Show when={props.active}>
      <div class="flex flex-wrap items-center gap-2 rounded-sm bg-zinc-100 p-2 dark:bg-zinc-900">
        <ToolbarButton disabled={!props.hasMessages || props.disabled} onClick={props.onToggleAll}>
          {props.allSelected ? '取消全选' : '全选'}
        </ToolbarButton>
        <span class="px-1 text-sm text-zinc-600 dark:text-zinc-300">已选择 {props.count} 封</span>
        <SelectedActions {...props} />
        <ToolbarButton
          class="ml-auto"
          disabled={!props.hasMessages || props.disabled}
          onClick={props.onMarkAllRead}
        >
          全部已读
        </ToolbarButton>
      </div>
    </Show>
  );
}
