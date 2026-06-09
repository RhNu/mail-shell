import { Menu as ArkMenu } from '@ark-ui/solid/menu';
import { Portal } from 'solid-js/web';
import { Archive, ArchiveRestore, Ellipsis, FileDown, FileText, Trash2 } from 'lucide-solid';
import { rawMessageDownloadUrl } from '../features/messages/api';
import type { Mailbox } from '../features/messages/models';

export type MessageActionMenuProps = {
  messageId: string;
  mailbox: Mailbox;
  onViewHeaders?: () => void;
  // eslint-disable-next-line no-unused-vars
  onMoveToMailbox?: (_mailbox: Mailbox) => void;
  onDelete?: () => void;
  disabled?: boolean;
};

function downloadEml(messageId: string) {
  const a = document.createElement('a');
  a.href = rawMessageDownloadUrl(messageId);
  a.download = '';
  a.click();
}

const DELETE_CONFIRM_MESSAGE = '永久删除这封邮件？此操作不会进入回收站，且无法撤销。';
const menuItemClass =
  'flex items-center gap-2 px-3 py-2 text-sm text-zinc-700 transition-colors data-[highlighted]:bg-zinc-200 data-[highlighted]:text-zinc-900 dark:text-zinc-300 dark:data-[highlighted]:bg-zinc-800 dark:data-[highlighted]:text-zinc-100';

function MoveMailboxItem(props: {
  mailbox: Mailbox;
  // eslint-disable-next-line no-unused-vars
  onSelectAction: (_value: string) => void;
}) {
  const moveLabel = () => (props.mailbox === 'archive' ? '移回收件箱' : '归档');

  return (
    <ArkMenu.Item
      value="move-mailbox"
      class={menuItemClass}
      onSelect={() => props.onSelectAction('move-mailbox')}
      onClick={() => props.onSelectAction('move-mailbox')}
      onKeyDown={(event) => selectValueWithKeyboard(event, 'move-mailbox', props.onSelectAction)}
    >
      {props.mailbox === 'archive' ? <ArchiveRestore size={16} /> : <Archive size={16} />}
      {moveLabel()}
    </ArkMenu.Item>
  );
}

function ViewHeadersItem(props: {
  // eslint-disable-next-line no-unused-vars
  onSelectAction: (_value: string) => void;
}) {
  return (
    <ArkMenu.Item
      value="view-headers"
      class={menuItemClass}
      onSelect={() => props.onSelectAction('view-headers')}
      onClick={() => props.onSelectAction('view-headers')}
      onKeyDown={(event) => selectValueWithKeyboard(event, 'view-headers', props.onSelectAction)}
    >
      <FileText size={16} />
      查看原始字段
    </ArkMenu.Item>
  );
}

function DownloadRawItem(props: {
  // eslint-disable-next-line no-unused-vars
  onSelectAction: (_value: string) => void;
}) {
  return (
    <ArkMenu.Item
      value="download-raw"
      class={menuItemClass}
      onSelect={() => props.onSelectAction('download-raw')}
      onClick={() => props.onSelectAction('download-raw')}
      onKeyDown={(event) => selectValueWithKeyboard(event, 'download-raw', props.onSelectAction)}
    >
      <FileDown size={16} />
      下载 EML 源文件
    </ArkMenu.Item>
  );
}

function DeleteItem(props: {
  // eslint-disable-next-line no-unused-vars
  onSelectAction: (_value: string) => void;
}) {
  return (
    <ArkMenu.Item
      value="delete"
      class="flex items-center gap-2 px-3 py-2 text-sm text-red-700 transition-colors data-[highlighted]:bg-red-50 data-[highlighted]:text-red-800 dark:text-red-400 dark:data-[highlighted]:bg-red-950/40 dark:data-[highlighted]:text-red-300"
      onSelect={() => props.onSelectAction('delete')}
      onClick={() => props.onSelectAction('delete')}
      onKeyDown={(event) => selectValueWithKeyboard(event, 'delete', props.onSelectAction)}
    >
      <Trash2 size={16} />
      永久删除
    </ArkMenu.Item>
  );
}

function MessageActionMenuContent(
  props: MessageActionMenuProps & {
    // eslint-disable-next-line no-unused-vars
    onSelectAction: (_value: string) => void;
  },
) {
  return (
    <ArkMenu.Content class="min-w-[160px] rounded-sm border border-zinc-200 bg-zinc-50 py-1 shadow-lg dark:border-zinc-800 dark:bg-zinc-950">
      {props.onMoveToMailbox && (
        <MoveMailboxItem mailbox={props.mailbox} onSelectAction={props.onSelectAction} />
      )}
      {props.onViewHeaders && <ViewHeadersItem onSelectAction={props.onSelectAction} />}
      <DownloadRawItem onSelectAction={props.onSelectAction} />
      {props.onDelete && <DeleteItem onSelectAction={props.onSelectAction} />}
    </ArkMenu.Content>
  );
}

function handleMenuSelect(props: MessageActionMenuProps, value: string) {
  if (value === 'move-mailbox') {
    props.onMoveToMailbox?.(props.mailbox === 'archive' ? 'inbox' : 'archive');
    return;
  }

  if (value === 'view-headers') {
    props.onViewHeaders?.();
    return;
  }

  if (value === 'download-raw') {
    downloadEml(props.messageId);
    return;
  }

  if (value === 'delete' && props.onDelete && window.confirm(DELETE_CONFIRM_MESSAGE)) {
    props.onDelete();
  }
}

function selectValueWithKeyboard(
  event: KeyboardEvent,
  value: string,
  // eslint-disable-next-line no-unused-vars
  onSelectAction: (_value: string) => void,
) {
  if (event.key !== 'Enter' && event.key !== ' ') return;
  event.preventDefault();
  onSelectAction(value);
}

export function MessageActionMenu(props: MessageActionMenuProps) {
  let actionQueued = false;
  const onSelectAction = (value: string) => {
    if (actionQueued) return;
    actionQueued = true;
    handleMenuSelect(props, value);
    queueMicrotask(() => {
      actionQueued = false;
    });
  };

  return (
    <ArkMenu.Root onSelect={(details) => onSelectAction(details.value)}>
      <ArkMenu.Trigger
        class="rounded-sm p-1.5 text-zinc-500 transition-colors hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-100"
        aria-label="更多操作"
        disabled={props.disabled}
      >
        <Ellipsis size={18} />
      </ArkMenu.Trigger>
      <Portal>
        <ArkMenu.Positioner class="z-50">
          <MessageActionMenuContent {...props} onSelectAction={onSelectAction} />
        </ArkMenu.Positioner>
      </Portal>
    </ArkMenu.Root>
  );
}
