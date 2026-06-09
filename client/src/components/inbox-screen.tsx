import { createEffect, createMemo, createSignal, type Accessor, type JSX } from 'solid-js';
import {
  useDeleteMessage,
  useMessagesList,
  useUpdateMessageMailbox,
} from '../features/messages/queries';
import type { Mailbox, MessageListQuery, MessageSummary } from '../features/messages/models';
import { SearchInput, Pagination, EmptyState, ErrorBanner } from './ui';
import { MessageList } from './message-list';
import { MessageListSkeleton } from './message-list-skeleton';

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

function ListSection(props: {
  loading: boolean;
  data: { items: MessageSummary[]; total: number; limit: number } | undefined;
  searchQuery: string;
  page: number;
  totalPages: number;
  returnTo: string;
  // eslint-disable-next-line no-unused-vars
  onPageChange: (p: number) => void;
  // eslint-disable-next-line no-unused-vars
  onMoveToMailbox: (id: string, mailbox: Mailbox) => void;
  // eslint-disable-next-line no-unused-vars
  onDelete: (id: string) => void;
  actionsDisabled: boolean;
  emptyDescription?: string;
}) {
  return (
    <>
      {props.loading ? (
        <MessageListSkeleton />
      ) : props.data && props.data.items.length > 0 ? (
        <>
          <MessageList
            messages={props.data.items}
            tagsMap={new Map()}
            attachmentCounts={new Map()}
            searchQuery={props.searchQuery}
            returnTo={props.returnTo}
            onMoveToMailbox={props.onMoveToMailbox}
            onDelete={props.onDelete}
            actionsDisabled={props.actionsDisabled}
          />
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
      ) : (
        <EmptyState description={props.emptyDescription} />
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
          placeholder="在当前页面搜索..."
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
      <ListSection
        loading={state.messagesQuery.isLoading}
        data={state.messagesQuery.data}
        searchQuery={state.searchQuery()}
        page={state.page()}
        totalPages={state.totalPages()}
        returnTo={currentHashPath()}
        onPageChange={state.setPageAndScroll}
        onMoveToMailbox={state.moveToMailbox}
        onDelete={state.deleteMessage}
        actionsDisabled={state.actionsDisabled()}
        emptyDescription={props.emptyDescription}
      />
    </section>
  );
}

function useInboxState(query: Accessor<MessageListQuery>) {
  const [page, setPage] = createSignal(1);
  const [searchQuery, setSearchQuery] = createSignal('');
  const queryKey = createMemo(() => JSON.stringify(query() ?? {}));
  const updateMailboxMutation = useUpdateMessageMailbox();
  const deleteMessageMutation = useDeleteMessage();
  const messagesQuery = useMessagesList(() => ({
    ...query(),
    page: page(),
    limit: DEFAULT_LIMIT,
  }));
  const totalPages = () =>
    messagesQuery.data ? Math.ceil(messagesQuery.data.total / messagesQuery.data.limit) : 0;
  const mutationErrorMessage = () =>
    updateMailboxMutation.isError || deleteMessageMutation.isError
      ? (updateMailboxMutation.error?.message ??
        deleteMessageMutation.error?.message ??
        '更新邮件失败')
      : undefined;

  createEffect(() => {
    queryKey();
    setPage(1);
  });

  return {
    page,
    searchQuery,
    setSearchQuery,
    messagesQuery,
    totalPages,
    mutationErrorMessage,
    actionsDisabled: () => updateMailboxMutation.isPending || deleteMessageMutation.isPending,
    setPageAndScroll: (newPage: number) => {
      setPage(newPage);
      scrollToTop();
    },
    moveToMailbox: (id: string, mailbox: Mailbox) => updateMailboxMutation.mutate({ id, mailbox }),
    deleteMessage: (id: string) => deleteMessageMutation.mutate({ id }),
  };
}

function currentHashPath(): string {
  const path = window.location.hash.replace(/^#/u, '').split('?')[0];
  return path || '/';
}
