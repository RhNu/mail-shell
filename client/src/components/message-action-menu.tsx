import { Menu as ArkMenu } from '@ark-ui/solid/menu';
import { Dialog } from '@ark-ui/solid/dialog';
import { createSignal, Show } from 'solid-js';
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

const DELETE_CONFIRM_TITLE = '永久删除邮件';
const DELETE_CONFIRM_DESCRIPTION = '此操作不会进入回收站，且无法撤销。';
const menuItemClass =
  'flex items-center gap-2 px-3 py-2 text-sm text-zinc-700 transition-colors data-[highlighted]:bg-zinc-200 data-[highlighted]:text-zinc-900 dark:text-zinc-300 dark:data-[highlighted]:bg-zinc-800 dark:data-[highlighted]:text-zinc-100';

function MoveMailboxItem(props: {
  mailbox: Mailbox;
  disabled?: boolean;
  // eslint-disable-next-line no-unused-vars
  onSelectAction: (_value: string) => void;
}) {
  const moveLabel = () => (props.mailbox === 'archive' ? '移回收件箱' : '归档');

  return (
    <ArkMenu.Item
      value="move-mailbox"
      class={menuItemClass}
      disabled={props.disabled}
      onSelect={() => props.onSelectAction('move-mailbox')}
    >
      {props.mailbox === 'archive' ? <ArchiveRestore size={16} /> : <Archive size={16} />}
      {moveLabel()}
    </ArkMenu.Item>
  );
}

function ViewHeadersItem(props: {
  disabled?: boolean;
  // eslint-disable-next-line no-unused-vars
  onSelectAction: (_value: string) => void;
}) {
  return (
    <ArkMenu.Item
      value="view-headers"
      class={menuItemClass}
      disabled={props.disabled}
      onSelect={() => props.onSelectAction('view-headers')}
    >
      <FileText size={16} />
      查看原始字段
    </ArkMenu.Item>
  );
}

function DownloadRawItem(props: {
  disabled?: boolean;
  // eslint-disable-next-line no-unused-vars
  onSelectAction: (_value: string) => void;
}) {
  return (
    <ArkMenu.Item
      value="download-raw"
      class={menuItemClass}
      disabled={props.disabled}
      onSelect={() => props.onSelectAction('download-raw')}
    >
      <FileDown size={16} />
      下载 EML 源文件
    </ArkMenu.Item>
  );
}

function DeleteItem(props: {
  disabled?: boolean;
  // eslint-disable-next-line no-unused-vars
  onSelectAction: (_value: string) => void;
}) {
  return (
    <ArkMenu.Item
      value="delete"
      class="flex items-center gap-2 px-3 py-2 text-sm text-red-700 transition-colors data-[highlighted]:bg-red-50 data-[highlighted]:text-red-800 dark:text-red-400 dark:data-[highlighted]:bg-red-950/40 dark:data-[highlighted]:text-red-300"
      disabled={props.disabled}
      onSelect={() => props.onSelectAction('delete')}
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
        <MoveMailboxItem
          mailbox={props.mailbox}
          disabled={props.disabled}
          onSelectAction={props.onSelectAction}
        />
      )}
      {props.onViewHeaders && (
        <ViewHeadersItem disabled={props.disabled} onSelectAction={props.onSelectAction} />
      )}
      <DownloadRawItem disabled={props.disabled} onSelectAction={props.onSelectAction} />
      {props.onDelete && (
        <DeleteItem disabled={props.disabled} onSelectAction={props.onSelectAction} />
      )}
    </ArkMenu.Content>
  );
}

function handleMenuSelect(
  props: MessageActionMenuProps,
  value: string,
  openDeleteDialog: () => void,
) {
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

  if (value === 'delete' && props.onDelete) {
    openDeleteDialog();
  }
}

function DeleteConfirmationDialog(props: {
  open: boolean;
  disabled?: boolean;
  // eslint-disable-next-line no-unused-vars
  onOpenChange: (_open: boolean) => void;
  onConfirm: () => void;
}) {
  return (
    <Show when={props.open}>
      <Dialog.Root open={props.open} onOpenChange={(details) => props.onOpenChange(details.open)}>
        <Portal>
          <Dialog.Backdrop class="fixed inset-0 z-50 bg-black/50" />
          <Dialog.Positioner class="fixed inset-0 z-50 flex items-center justify-center p-4">
            <Dialog.Content class="w-full max-w-md rounded-sm border border-zinc-200 bg-zinc-50 p-5 shadow-xl dark:border-zinc-800 dark:bg-zinc-950">
              <div class="flex flex-col gap-2">
                <Dialog.Title class="text-base font-semibold text-zinc-900 dark:text-zinc-100">
                  {DELETE_CONFIRM_TITLE}
                </Dialog.Title>
                <Dialog.Description class="text-sm text-zinc-600 dark:text-zinc-300">
                  {DELETE_CONFIRM_DESCRIPTION}
                </Dialog.Description>
              </div>
              <div class="mt-5 flex justify-end gap-3">
                <Dialog.CloseTrigger class="rounded-sm border border-zinc-300 px-3 py-2 text-sm font-medium text-zinc-700 transition-colors hover:bg-zinc-100 dark:border-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-900">
                  取消
                </Dialog.CloseTrigger>
                <button
                  type="button"
                  onClick={() => props.onConfirm()}
                  disabled={props.disabled}
                  class="rounded-sm bg-red-700 px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-red-600 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-red-600 dark:hover:bg-red-500"
                >
                  永久删除
                </button>
              </div>
            </Dialog.Content>
          </Dialog.Positioner>
        </Portal>
      </Dialog.Root>
    </Show>
  );
}

export function MessageActionMenu(props: MessageActionMenuProps) {
  const [deleteDialogOpen, setDeleteDialogOpen] = createSignal(false);
  const [deleteSubmitted, setDeleteSubmitted] = createSignal(false);
  const openDeleteDialog = () => {
    if (props.disabled) return;
    setDeleteSubmitted(false);
    setDeleteDialogOpen(true);
  };
  const confirmDelete = () => {
    if (deleteSubmitted() || props.disabled) return;
    setDeleteSubmitted(true);
    props.onDelete?.();
    setDeleteDialogOpen(false);
  };
  const onSelectAction = (value: string) => {
    if (props.disabled) return;
    handleMenuSelect(props, value, openDeleteDialog);
  };

  return (
    <>
      <ArkMenu.Root>
        <ArkMenu.Trigger
          class="rounded-sm p-1.5 text-zinc-500 transition-colors hover:bg-zinc-100 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-50 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-100"
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
      <DeleteConfirmationDialog
        open={deleteDialogOpen()}
        disabled={props.disabled || deleteSubmitted()}
        onOpenChange={setDeleteDialogOpen}
        onConfirm={confirmDelete}
      />
    </>
  );
}
