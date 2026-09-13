import type { Accessor, Setter } from 'solid-js';
import {
  useDeleteMessage,
  useMarkAllMessagesRead,
  useUpdateMessageMailbox,
  useUpdateMessagesState,
  useUpdateMessageState,
} from './queries';
import type { Mailbox, MessageListQuery } from './models';

export function useInboxMutations(
  selectedIds: Accessor<Set<string>>,
  setSelectedIds: Setter<Set<string>>,
  setSelectionMode: Setter<boolean>,
) {
  const updateMailbox = useUpdateMessageMailbox();
  const updateState = useUpdateMessageState();
  const updateStates = useUpdateMessagesState();
  const markAllRead = useMarkAllMessagesRead();
  const deleteMessage = useDeleteMessage();
  const mutations = [updateMailbox, updateState, updateStates, markAllRead, deleteMessage];
  const finishSelection = () => {
    setSelectedIds(new Set<string>());
    setSelectionMode(false);
  };
  const mutationErrorMessage = () => {
    const failed = mutations.find((mutation) => mutation.isError);
    return failed ? (failed.error?.message ?? '更新邮件失败') : undefined;
  };

  return {
    mutationErrorMessage,
    actionsDisabled: () => mutations.some((mutation) => mutation.isPending),
    setSelected: (id: string, selected: boolean) => {
      const next = new Set(selectedIds());
      if (selected) next.add(id);
      else next.delete(id);
      setSelectedIds(next);
    },
    bulkUpdate: (state: { mailbox?: Mailbox; read?: boolean; trashed?: boolean }) => {
      const ids = [...selectedIds()];
      if (ids.length > 0) updateStates.mutate({ ids, state }, { onSuccess: finishSelection });
    },
    markAllRead: (query: MessageListQuery) =>
      markAllRead.mutate(query, { onSuccess: finishSelection }),
    moveToMailbox: (id: string, mailbox: Mailbox) => updateMailbox.mutate({ id, mailbox }),
    updateState: (id: string, state: { read?: boolean; starred?: boolean; trashed?: boolean }) =>
      updateState.mutate({ id, state }),
    deleteMessage: (id: string) => deleteMessage.mutate({ id }),
  };
}
