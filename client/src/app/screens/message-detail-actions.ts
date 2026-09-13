import { useNavigate, useSearchParams } from '@solidjs/router';
import {
  useDeleteMessage,
  useUpdateMessageMailbox,
  useUpdateMessageState,
} from '../../features/messages/queries';
import type { MessageStateUpdateRequest } from '../../features/messages/models';
import type { Mailbox } from '../../features/messages/models';

export type { Mailbox };

export function useDetailReturn() {
  const [searchParams] = useSearchParams<{ returnTo?: string }>();
  const navigate = useNavigate();
  const returnTo = () => normalizeReturnTo(searchParams.returnTo);

  return {
    returnTo,
    navigateBack: () => navigate(returnTo(), { replace: true }),
  };
}

export function useDetailActions(messageId: () => string, onSuccess: () => void) {
  const updateMailboxMutation = useUpdateMessageMailbox();
  const deleteMessageMutation = useDeleteMessage();
  const updateStateMutation = useUpdateMessageState();
  const isPending = () =>
    updateMailboxMutation.isPending ||
    updateStateMutation.isPending ||
    deleteMessageMutation.isPending;
  const errorMessage = () =>
    updateMailboxMutation.isError || updateStateMutation.isError || deleteMessageMutation.isError
      ? (updateMailboxMutation.error?.message ??
        updateStateMutation.error?.message ??
        deleteMessageMutation.error?.message ??
        '更新邮件失败')
      : undefined;

  return {
    isPending,
    errorMessage,
    moveToMailbox: (mailbox: Mailbox) =>
      updateMailboxMutation.mutate({ id: messageId(), mailbox }, { onSuccess }),
    deleteMessage: () => deleteMessageMutation.mutate({ id: messageId() }, { onSuccess }),
    updateState: (state: MessageStateUpdateRequest, navigateAfter = false) =>
      updateStateMutation.mutate(
        { id: messageId(), state },
        navigateAfter ? { onSuccess } : undefined,
      ),
  };
}

function normalizeReturnTo(value: string | undefined): string {
  if (['/', '/archive', '/unread', '/starred', '/trash'].includes(value ?? '')) {
    return value!;
  }

  if (value && /^\/tags\/\d+$/u.test(value)) {
    return value;
  }

  return '/';
}

export function backLabel(path: string): string {
  if (path === '/archive') return '返回归档';
  if (path === '/unread') return '返回未读';
  if (path === '/starred') return '返回星标';
  if (path === '/trash') return '返回垃圾箱';
  if (path.startsWith('/tags/')) return '返回标签';
  return '返回收件箱';
}
