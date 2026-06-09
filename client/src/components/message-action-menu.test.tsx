import { fireEvent, render, screen } from '@solidjs/testing-library';
import { beforeEach, expect, it, vi } from 'vitest';
import { MessageActionMenu } from './message-action-menu';

beforeEach(() => {
  vi.restoreAllMocks();
});

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

  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
  await fireEvent.click(await screen.findByRole('menuitem', { name: '归档' }));

  expect(onMoveToMailbox).toHaveBeenCalledWith('archive');
});

it('selects menu actions with the keyboard', async () => {
  const onMoveToMailbox = vi.fn();

  render(() => (
    <MessageActionMenu
      messageId="msg-1"
      mailbox="inbox"
      onMoveToMailbox={onMoveToMailbox}
      onDelete={vi.fn()}
    />
  ));

  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
  await fireEvent.keyDown(await screen.findByRole('menuitem', { name: '归档' }), { key: 'Enter' });

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

  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
  await fireEvent.click(await screen.findByRole('menuitem', { name: '移回收件箱' }));

  expect(onMoveToMailbox).toHaveBeenCalledWith('inbox');
});

it('confirms permanent deletion before invoking delete', async () => {
  const confirm = vi.spyOn(window, 'confirm').mockReturnValue(false);
  const onDelete = vi.fn();

  render(() => (
    <MessageActionMenu
      messageId="msg-1"
      mailbox="inbox"
      onMoveToMailbox={vi.fn()}
      onDelete={onDelete}
    />
  ));

  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
  await fireEvent.click(await screen.findByRole('menuitem', { name: '永久删除' }));

  expect(confirm).toHaveBeenCalledWith('永久删除这封邮件？此操作不会进入回收站，且无法撤销。');
  expect(onDelete).not.toHaveBeenCalled();
});

it('invokes permanent deletion after confirmation', async () => {
  vi.spyOn(window, 'confirm').mockReturnValue(true);
  const onDelete = vi.fn();

  render(() => (
    <MessageActionMenu
      messageId="msg-1"
      mailbox="inbox"
      onMoveToMailbox={vi.fn()}
      onDelete={onDelete}
    />
  ));

  await fireEvent.click(screen.getByRole('button', { name: '更多操作' }));
  await fireEvent.click(await screen.findByRole('menuitem', { name: '永久删除' }));

  expect(onDelete).toHaveBeenCalledOnce();
});
