import { createEffect, createMemo, createSignal, Show, type Accessor, type JSX } from 'solid-js';
import {
  useDeleteMessage,
  useMessagesList,
  useUpdateMessageState,
  useUpdateMessagesState,
  useUpdateMessageMailbox,
} from '../features/messages/queries';
import type { Mailbox, MessageListQuery, MessageSummary } from '../features/messages/models';
import { SearchInput, Pagination, EmptyState, ErrorBanner } from './ui';
import { MessageList } from './message-list';
import { MessageListSkeleton } from './message-list-skeleton';
import { BulkToolbar } from './bulk-toolbar';

const DEFAULT_LIMIT = 20;

type InboxScreenProps = {
  title: JSX.Element;
  subtitle?: string;
  tagChip?: JSX.Element;
  query: Accessor<MessageListQuery>;
  emptyDescription?: string;
};

function MessageCount(props: { total: number }) {
  return <p class="mt-0.5 text-sm text-zinc-500 dark:text-zinc-400">{props.total} 封邮件</p>;
}

type ListSectionProps = {
  loading: boolean;
  data: { items: MessageSummary[]; total: number; limit: number } | undefined;
  page: number;
  totalPages: number;
  returnTo: string;
  // eslint-disable-next-line no-unused-vars
  onPageChange: (p: number) => void;
  // eslint-disable-next-line no-unused-vars
  onMoveToMailbox: (id: string, mailbox: Mailbox) => void;
  // eslint-disable-next-line no-unused-vars
  onDelete?: (id: string) => void;
  onUpdateState: (
    // eslint-disable-next-line no-unused-vars
    id: string,
    // eslint-disable-next-line no-unused-vars
    state: { read?: boolean; starred?: boolean; trashed?: boolean },
  ) => void;
  trashView: boolean;
  selectedIds: Set<string>;
  // eslint-disable-next-line no-unused-vars
  onSelectedChange: (id: string, selected: boolean) => void;
  actionsDisabled: boolean;
  emptyDescription?: string;
};

function MessageResults(props: ListSectionProps) {
  return (
    <Show when={!props.loading} fallback={<MessageListSkeleton />}>
      <Show
        when={props.data && props.data.items.length > 0}
        fallback={<EmptyState description={props.emptyDescription} />}
      >
        <MessageList
          messages={props.data!.items}
          attachmentCounts={new Map()}
          returnTo={props.returnTo}
          onMoveToMailbox={props.onMoveToMailbox}
          onDelete={props.onDelete}
          onUpdateState={props.onUpdateState}
          trashView={props.trashView}
          selectedIds={props.selectedIds}
          onSelectedChange={props.onSelectedChange}
          actionsDisabled={props.actionsDisabled}
        />
      </Show>
    </Show>
  );
}

function ListSection(props: ListSectionProps) {
  return (
    <>
      <MessageResults {...props} />
      {props.totalPages > 1 && (
        <div class="pt-2">
          <Pagination
            page={props.page}
            totalPages={props.totalPages}
            onPageChange={props.onPageChange}
          />
        </div>
      )}
    </>
  );
}

function InboxToolbar(props: {
  title: JSX.Element;
  subtitle?: string;
  tagChip?: JSX.Element;
  total?: number;
  searchQuery: string;
  // eslint-disable-next-line no-unused-vars
  onSearchChange: (value: string) => void;
}) {
  return (
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        {props.title}
        {props.subtitle === undefined ? (
          props.total !== undefined && <MessageCount total={props.total} />
        ) : (
          <p class="mt-0.5 text-sm text-zinc-500 dark:text-zinc-400">{props.subtitle}</p>
        )}
        {props.tagChip && <div class="mt-2">{props.tagChip}</div>}
      </div>
      <div class="w-full sm:w-64">
        <SearchInput
          value={props.searchQuery}
          onChange={props.onSearchChange}
          placeholder="搜索邮件..."
        />
      </div>
    </div>
  );
}

function MutationErrorBanner(props: { message?: string }) {
  return <>{props.message && <ErrorBanner message={props.message} />}</>;
}

function scrollToTop() {
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

export function InboxScreen(props: InboxScreenProps): JSX.Element {
  const state = useInboxState(() => props.query());

  return (
    <section class="flex flex-col gap-4">
      <InboxToolbar
        title={props.title}
        subtitle={props.subtitle}
        tagChip={props.tagChip}
        total={state.messagesQuery.data?.total}
        searchQuery={state.searchQuery()}
        onSearchChange={state.setSearchQuery}
      />
      {state.messagesQuery.isError && (
        <ErrorBanner
          message={state.messagesQuery.error?.message ?? '加载邮件失败'}
          onRetry={() => state.messagesQuery.refetch()}
        />
      )}
      <MutationErrorBanner message={state.mutationErrorMessage()} />
      <BulkToolbar
        count={state.selectedIds().size}
        trashView={Boolean(props.query()?.trashed)}
        onRead={() => state.bulkUpdate({ read: true })}
        onArchive={() => state.bulkUpdate({ mailbox: 'archive' })}
        onTrash={() => state.bulkUpdate({ trashed: true })}
        onRestore={() => state.bulkUpdate({ trashed: false })}
      />
      <ListSection
        loading={state.messagesQuery.isLoading}
        data={state.messagesQuery.data}
        page={state.page()}
        totalPages={state.totalPages()}
        returnTo={currentHashPath()}
        onPageChange={state.setPageAndScroll}
        onMoveToMailbox={state.moveToMailbox}
        onDelete={props.query()?.trashed ? state.deleteMessage : undefined}
        onUpdateState={state.updateState}
        trashView={Boolean(props.query()?.trashed)}
        selectedIds={state.selectedIds()}
        onSelectedChange={state.setSelected}
        actionsDisabled={state.actionsDisabled()}
        emptyDescription={props.emptyDescription}
      />
    </section>
  );
}

function useInboxMutations(
  selectedIds: Accessor<Set<string>>,
  // eslint-disable-next-line no-unused-vars
  setSelectedIds: (value: Set<string>) => void,
) {
  const updateMailboxMutation = useUpdateMessageMailbox();
  const updateStateMutation = useUpdateMessageState();
  const updateStatesMutation = useUpdateMessagesState();
  const deleteMessageMutation = useDeleteMessage();
  const mutationErrorMessage = () =>
    updateMailboxMutation.isError ||
    updateStateMutation.isError ||
    updateStatesMutation.isError ||
    deleteMessageMutation.isError
      ? (updateMailboxMutation.error?.message ??
        updateStateMutation.error?.message ??
        updateStatesMutation.error?.message ??
        deleteMessageMutation.error?.message ??
        '更新邮件失败')
      : undefined;

  return {
    mutationErrorMessage,
    actionsDisabled: () =>
      updateMailboxMutation.isPending ||
      updateStateMutation.isPending ||
      updateStatesMutation.isPending ||
      deleteMessageMutation.isPending,
    setSelected: (id: string, selected: boolean) => {
      const next = new Set(selectedIds());
      if (selected) next.add(id);
      else next.delete(id);
      setSelectedIds(next);
    },
    bulkUpdate: (state: { mailbox?: Mailbox; read?: boolean; trashed?: boolean }) => {
      const ids = [...selectedIds()];
      if (ids.length === 0) return;
      updateStatesMutation.mutate(
        { ids, state },
        { onSuccess: () => setSelectedIds(new Set<string>()) },
      );
    },
    moveToMailbox: (id: string, mailbox: Mailbox) => updateMailboxMutation.mutate({ id, mailbox }),
    updateState: (id: string, state: { read?: boolean; starred?: boolean; trashed?: boolean }) =>
      updateStateMutation.mutate({ id, state }),
    deleteMessage: (id: string) => deleteMessageMutation.mutate({ id }),
  };
}

function useInboxState(query: Accessor<MessageListQuery>) {
  const [page, setPage] = createSignal(1);
  const [searchQuery, setSearchQuery] = createSignal('');
  const [selectedIds, setSelectedIds] = createSignal(new Set<string>());
  const queryKey = createMemo(() => JSON.stringify(query() ?? {}));
  const messagesQuery = useMessagesList(() => ({
    ...query(),
    q: searchQuery().trim() || query()?.q,
    page: page(),
    limit: DEFAULT_LIMIT,
  }));
  const mutations = useInboxMutations(selectedIds, setSelectedIds);
  createEffect(() => {
    queryKey();
    setPage(1);
    setSelectedIds(new Set<string>());
  });
  return {
    page,
    selectedIds,
    searchQuery,
    setSearchQuery,
    messagesQuery,
    totalPages: () =>
      messagesQuery.data ? Math.ceil(messagesQuery.data.total / messagesQuery.data.limit) : 0,
    setPageAndScroll: (newPage: number) => {
      setPage(newPage);
      scrollToTop();
    },
    ...mutations,
  };
}

function currentHashPath(): string {
  const path = window.location.hash.replace(/^#/u, '').split('?')[0];
  return path || '/';
}
