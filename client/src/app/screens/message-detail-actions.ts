import { useNavigate, useSearchParams } from '@solidjs/router';
import { useDeleteMessage, useUpdateMessageMailbox } from '../../features/messages/queries';
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
  const isPending = () => updateMailboxMutation.isPending || deleteMessageMutation.isPending;
  const errorMessage = () =>
    updateMailboxMutation.isError || deleteMessageMutation.isError
      ? (updateMailboxMutation.error?.message ??
        deleteMessageMutation.error?.message ??
        '更新邮件失败')
      : undefined;

  return {
    isPending,
    errorMessage,
    moveToMailbox: (mailbox: Mailbox) =>
      updateMailboxMutation.mutate({ id: messageId(), mailbox }, { onSuccess }),
    deleteMessage: () => deleteMessageMutation.mutate({ id: messageId() }, { onSuccess }),
  };
}

function normalizeReturnTo(value: string | undefined): string {
  if (value === '/' || value === '/archive') {
    return value;
  }

  if (value && /^\/tags\/\d+$/u.test(value)) {
    return value;
  }

  return '/';
}

export function backLabel(path: string): string {
  if (path === '/archive') return '返回归档';
  if (path.startsWith('/tags/')) return '返回标签';
  return '返回收件箱';
}
