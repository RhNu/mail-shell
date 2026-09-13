import { Show, type Accessor, type JSX } from 'solid-js';
import type { Mailbox, MessageListQuery, MessageSummary } from '../features/messages/models';
import { useInboxState } from '../features/messages/use-inbox-state';
import { SearchInput, Pagination, EmptyState, ErrorBanner } from './ui';
import { MessageList } from './message-list';
import { MessageListSkeleton } from './message-list-skeleton';
import { BulkToolbar } from './bulk-toolbar';
import { useInboxKeyboardShortcuts } from '../features/messages/inbox-keyboard';

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
  selectionMode: boolean;
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
          selectionMode={props.selectionMode}
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
  selectionMode: boolean;
  hasMessages: boolean;
  actionsDisabled: boolean;
  // eslint-disable-next-line no-unused-vars
  onSearchChange: (value: string) => void;
  onToggleSelectionMode: () => void;
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
      <div class="flex w-full items-start gap-2 sm:w-auto">
        <div class="min-w-0 flex-1 sm:w-64">
          <SearchInput
            value={props.searchQuery}
            onChange={props.onSearchChange}
            placeholder="搜索邮件..."
          />
          <p class="mt-1 hidden text-right text-[11px] text-zinc-400 lg:block dark:text-zinc-500">
            J/K 浏览 · Enter 打开 · X 选择 · S 星标 · E 归档
          </p>
        </div>
        <button
          type="button"
          aria-pressed={props.selectionMode}
          disabled={!props.hasMessages || props.actionsDisabled}
          onClick={() => props.onToggleSelectionMode()}
          class="rounded-sm border border-zinc-300 px-3 py-2 text-sm hover:bg-zinc-100 disabled:cursor-not-allowed disabled:opacity-50 dark:border-zinc-700 dark:hover:bg-zinc-800"
        >
          {props.selectionMode ? '完成' : '选择'}
        </button>
      </div>
    </div>
  );
}

function MutationErrorBanner(props: { message?: string }) {
  return <>{props.message && <ErrorBanner message={props.message} />}</>;
}

type InboxState = ReturnType<typeof useInboxState>;

function InboxBulkActions(props: { state: InboxState; trashView: boolean }) {
  return (
    <BulkToolbar
      active={props.state.selectionMode()}
      count={props.state.selectedIds().size}
      allSelected={props.state.allVisibleSelected()}
      hasMessages={(props.state.messagesQuery.data?.items.length ?? 0) > 0}
      disabled={props.state.actionsDisabled()}
      trashView={props.trashView}
      onToggleAll={props.state.toggleSelectAll}
      onMarkAllRead={props.state.markAllRead}
      onRead={() => props.state.bulkUpdate({ read: true })}
      onArchive={() => props.state.bulkUpdate({ mailbox: 'archive' })}
      onTrash={() => props.state.bulkUpdate({ trashed: true })}
      onRestore={() => props.state.bulkUpdate({ trashed: false })}
    />
  );
}

export function InboxScreen(props: InboxScreenProps): JSX.Element {
  const state = useInboxState(() => props.query());
  useInboxKeyboard(state, () => props.query());
  const hasMessages = () => (state.messagesQuery.data?.items.length ?? 0) > 0;

  return (
    <section class="flex flex-col gap-4">
      <InboxToolbar
        title={props.title}
        subtitle={props.subtitle}
        tagChip={props.tagChip}
        total={state.messagesQuery.data?.total}
        searchQuery={state.searchQuery()}
        selectionMode={state.selectionMode()}
        hasMessages={hasMessages()}
        actionsDisabled={state.actionsDisabled()}
        onSearchChange={state.setSearchQuery}
        onToggleSelectionMode={state.toggleSelectionMode}
      />
      {state.messagesQuery.isError && (
        <ErrorBanner
          message={state.messagesQuery.error?.message ?? '加载邮件失败'}
          onRetry={() => state.messagesQuery.refetch()}
        />
      )}
      <MutationErrorBanner message={state.mutationErrorMessage()} />
      <InboxBulkActions state={state} trashView={Boolean(props.query()?.trashed)} />
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
        selectionMode={state.selectionMode()}
        selectedIds={state.selectedIds()}
        onSelectedChange={state.setSelected}
        actionsDisabled={state.actionsDisabled()}
        emptyDescription={props.emptyDescription}
      />
    </section>
  );
}

function useInboxKeyboard(
  state: ReturnType<typeof useInboxState>,
  query: Accessor<MessageListQuery>,
) {
  useInboxKeyboardShortcuts({
    actionsDisabled: state.actionsDisabled,
    trashView: () => Boolean(query()?.trashed),
    onEnterSelectionMode: () => state.setSelectionMode(true),
    onSelectedChange: state.setSelected,
    onUpdateState: state.updateState,
    onMoveToMailbox: state.moveToMailbox,
  });
}

function currentHashPath(): string {
  const path = window.location.hash.replace(/^#/u, '').split('?')[0];
  return path || '/';
}
