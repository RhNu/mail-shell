import { createEffect, createMemo, createSignal, type Accessor, type JSX } from 'solid-js';
import { useMessagesList } from '../features/messages/queries';
import type { MessageListQuery, MessageSummary } from '../features/messages/models';
import { SearchInput } from './ui/search-input';
import { Pagination } from './ui/pagination';
import { EmptyState } from './ui/empty-state';
import { ErrorBanner } from './ui/error-banner';
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
  return (
    <p class="mt-0.5 text-sm text-zinc-500 dark:text-zinc-400">
      {props.total} message{props.total === 1 ? '' : 's'}
    </p>
  );
}

function ListSection(props: {
  loading: boolean;
  data: { items: MessageSummary[]; total: number; limit: number } | undefined;
  searchQuery: string;
  page: number;
  totalPages: number;
  // eslint-disable-next-line no-unused-vars
  onPageChange: (p: number) => void;
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
          placeholder="Search in current page..."
        />
      </div>
    </div>
  );
}

export function InboxScreen(props: InboxScreenProps): JSX.Element {
  const [page, setPage] = createSignal(1);
  const [searchQuery, setSearchQuery] = createSignal('');
  const queryKey = createMemo(() => JSON.stringify(props.query() ?? {}));
  const messagesQuery = useMessagesList(() => ({
    ...props.query(),
    page: page(),
    limit: DEFAULT_LIMIT,
  }));
  const totalPages = () =>
    messagesQuery.data ? Math.ceil(messagesQuery.data.total / messagesQuery.data.limit) : 0;

  createEffect(() => {
    queryKey();
    setPage(1);
  });

  return (
    <section class="flex flex-col gap-4">
      <InboxToolbar
        title={props.title}
        subtitle={props.subtitle}
        tagChip={props.tagChip}
        total={messagesQuery.data?.total}
        searchQuery={searchQuery()}
        onSearchChange={setSearchQuery}
      />
      {messagesQuery.isError && (
        <ErrorBanner
          message={messagesQuery.error?.message ?? 'Failed to load messages'}
          onRetry={() => messagesQuery.refetch()}
        />
      )}
      <ListSection
        loading={messagesQuery.isLoading}
        data={messagesQuery.data}
        searchQuery={searchQuery()}
        page={page()}
        totalPages={totalPages()}
        onPageChange={(newPage) => {
          setPage(newPage);
          window.scrollTo({ top: 0, behavior: 'smooth' });
        }}
        emptyDescription={props.emptyDescription}
      />
    </section>
  );
}
