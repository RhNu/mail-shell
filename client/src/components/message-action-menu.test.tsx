import { createSignal } from 'solid-js';
import { fireEvent, render, screen, within } from '@solidjs/testing-library';
import { beforeEach, expect, it, vi } from 'vitest';
import { MessageActionMenu } from './message-action-menu';

beforeEach(() => {
  vi.restoreAllMocks();
});

async function openMenu() {
  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
}

async function selectMenuItem(name: string) {
  const item = await screen.findByRole('menuitem', { name });
  await fireEvent.pointerDown(item, { pointerType: 'mouse' });
  await fireEvent.click(item);
}

it('offers archive for inbox messages', async () => {
  const onMoveToMailbox = vi.fn();

  render(() => (
    <MessageActionMenu
      messageId="msg-1"
      mailbox="inbox"
      onMoveToMailbox={onMoveToMailbox}
      onDelete={vi.fn()}
    />
  ));

  await openMenu();
  await selectMenuItem('归档');

  expect(onMoveToMailbox).toHaveBeenCalledWith('archive');
});

it('selects menu actions from the Ark menu selection pipeline', async () => {
  const onMoveToMailbox = vi.fn();

  render(() => (
    <MessageActionMenu
      messageId="msg-1"
      mailbox="inbox"
      onMoveToMailbox={onMoveToMailbox}
      onDelete={vi.fn()}
    />
  ));

  await openMenu();
  const item = await screen.findByRole('menuitem', { name: '归档' });
  item.dispatchEvent(new CustomEvent('menu:select', { detail: { value: 'move-mailbox' } }));

  expect(onMoveToMailbox).toHaveBeenCalledWith('archive');
});

it('offers restore for archive messages', async () => {
  const onMoveToMailbox = vi.fn();

  render(() => (
    <MessageActionMenu
      messageId="msg-1"
      mailbox="archive"
      onMoveToMailbox={onMoveToMailbox}
      onDelete={vi.fn()}
    />
  ));

  await openMenu();
  await selectMenuItem('移回收件箱');

  expect(onMoveToMailbox).toHaveBeenCalledWith('inbox');
});

it('opens a delete confirmation dialog without using window.confirm', async () => {
  const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true);
  const onDelete = vi.fn();

  render(() => (
    <MessageActionMenu
      messageId="msg-1"
      mailbox="inbox"
      onMoveToMailbox={vi.fn()}
      onDelete={onDelete}
    />
  ));

  await openMenu();
  await selectMenuItem('永久删除');

  const dialog = await screen.findByRole('dialog', { name: '永久删除邮件' });
  expect(within(dialog).getByText('此操作不会进入回收站，且无法撤销。')).toBeInTheDocument();
  expect(confirm).not.toHaveBeenCalled();
  expect(onDelete).not.toHaveBeenCalled();
});

it('cancels permanent deletion from the confirmation dialog', async () => {
  const onDelete = vi.fn();

  render(() => (
    <MessageActionMenu
      messageId="msg-1"
      mailbox="inbox"
      onMoveToMailbox={vi.fn()}
      onDelete={onDelete}
    />
  ));

  await openMenu();
  await selectMenuItem('永久删除');
  await fireEvent.click(within(screen.getByRole('dialog')).getByRole('button', { name: '取消' }));

  expect(screen.queryByRole('dialog', { name: '永久删除邮件' })).not.toBeInTheDocument();
  expect(onDelete).not.toHaveBeenCalled();
});

it('invokes permanent deletion once after dialog confirmation', async () => {
  const onDelete = vi.fn();

  render(() => (
    <MessageActionMenu
      messageId="msg-1"
      mailbox="inbox"
      onMoveToMailbox={vi.fn()}
      onDelete={onDelete}
    />
  ));

  await openMenu();
  await selectMenuItem('永久删除');
  const confirmDelete = within(screen.getByRole('dialog', { name: '永久删除邮件' })).getByRole(
    'button',
    { name: '永久删除' },
  );
  await fireEvent.click(confirmDelete);
  await fireEvent.click(confirmDelete);

  expect(onDelete).toHaveBeenCalledOnce();
});

it('ignores menu item selection after the menu becomes disabled', async () => {
  const onMoveToMailbox = vi.fn();
  const [disabled, setDisabled] = createSignal(false);

  render(() => (
    <MessageActionMenu
      messageId="msg-1"
      mailbox="inbox"
      onMoveToMailbox={onMoveToMailbox}
      onDelete={vi.fn()}
      disabled={disabled()}
    />
  ));

  await openMenu();
  setDisabled(true);
  await selectMenuItem('归档');

  expect(onMoveToMailbox).not.toHaveBeenCalled();
});
