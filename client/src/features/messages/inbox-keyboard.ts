import { onCleanup, onMount, type Accessor } from 'solid-js';
import type { Mailbox } from './models';

export type InboxShortcut =
  | 'search'
  | 'next'
  | 'previous'
  | 'open'
  | 'select'
  | 'star'
  | 'archive'
  | 'trash';

export function inboxShortcutFor(
  key: string,
  options: { modified: boolean; editing: boolean },
): InboxShortcut | undefined {
  if (options.modified || options.editing) return undefined;
  const normalized = key.toLowerCase();
  if (normalized === '/') return 'search';
  if (normalized === 'j') return 'next';
  if (normalized === 'k') return 'previous';
  if (normalized === 'enter') return 'open';
  if (normalized === 'x') return 'select';
  if (normalized === 's') return 'star';
  if (normalized === 'e') return 'archive';
  if (normalized === 'delete' || normalized === 'backspace') return 'trash';
  return undefined;
}

export function nextMessageIndex(current: number, length: number, direction: 1 | -1) {
  if (length === 0) return -1;
  if (current < 0) return direction === 1 ? 0 : length - 1;
  return Math.min(length - 1, Math.max(0, current + direction));
}

type InboxKeyboardOptions = {
  actionsDisabled: Accessor<boolean>;
  trashView: Accessor<boolean>;
  onEnterSelectionMode: () => void;
  // eslint-disable-next-line no-unused-vars
  onSelectedChange: (id: string, selected: boolean) => void;
  // eslint-disable-next-line no-unused-vars
  onUpdateState: (id: string, state: { starred?: boolean; trashed?: boolean }) => void;
  // eslint-disable-next-line no-unused-vars
  onMoveToMailbox: (id: string, mailbox: Mailbox) => void;
};

function isEditing(target: EventTarget | null) {
  return (
    target instanceof HTMLElement &&
    Boolean(target.closest('input, textarea, select, button, a, [contenteditable="true"]'))
  );
}

function messageRows() {
  return [...document.querySelectorAll<HTMLElement>('[data-message-row]')];
}

function focusedRow(rows: HTMLElement[]) {
  return rows.find((row) => row === document.activeElement || row.contains(document.activeElement));
}

function focusAdjacent(command: 'next' | 'previous') {
  const rows = messageRows();
  const current = rows.indexOf(focusedRow(rows)!);
  const next = nextMessageIndex(current, rows.length, command === 'next' ? 1 : -1);
  rows[next]?.focus();
}

function runRowShortcut(command: InboxShortcut, row: HTMLElement, options: InboxKeyboardOptions) {
  const id = row.dataset.messageId;
  if (!id || options.actionsDisabled()) return;
  if (command === 'open') row.querySelector<HTMLAnchorElement>('[data-message-link]')?.click();
  if (command === 'select') {
    options.onEnterSelectionMode();
    options.onSelectedChange(id, row.dataset.messageSelected !== 'true');
  }
  if (command === 'star')
    options.onUpdateState(id, { starred: row.dataset.messageStarred !== 'true' });
  if (command === 'archive' && !options.trashView()) options.onMoveToMailbox(id, 'archive');
  if (command === 'trash' && !options.trashView()) options.onUpdateState(id, { trashed: true });
}

export function useInboxKeyboardShortcuts(options: InboxKeyboardOptions) {
  const onKeyDown = (event: KeyboardEvent) => {
    const command = inboxShortcutFor(event.key, {
      modified: event.altKey || event.ctrlKey || event.metaKey,
      editing: isEditing(event.target),
    });
    if (!command) return;
    if (command === 'search') {
      event.preventDefault();
      document.querySelector<HTMLInputElement>('[data-mail-search]')?.focus();
      return;
    }
    if (command === 'next' || command === 'previous') {
      event.preventDefault();
      focusAdjacent(command);
      return;
    }
    const row = focusedRow(messageRows());
    if (!row) return;
    event.preventDefault();
    runRowShortcut(command, row, options);
  };

  onMount(() => window.addEventListener('keydown', onKeyDown));
  onCleanup(() => window.removeEventListener('keydown', onKeyDown));
}
