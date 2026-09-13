import { createEffect, createMemo, createSignal, type Accessor, type Setter } from 'solid-js';
import type { MessageListQuery } from './models';
import { useMessagesList } from './queries';
import { useInboxMutations } from './use-inbox-mutations';

const DEFAULT_LIMIT = 20;

function createSelectionState(visibleIds: Accessor<string[]>) {
  const [selectionMode, setSelectionMode] = createSignal(false);
  const [selectedIds, setSelectedIds] = createSignal(new Set<string>());
  const allVisibleSelected = createMemo(
    () => visibleIds().length > 0 && visibleIds().every((id) => selectedIds().has(id)),
  );
  const clearSelection = () => setSelectedIds(new Set<string>());
  const exitSelection = () => {
    clearSelection();
    setSelectionMode(false);
  };
  return {
    selectionMode,
    setSelectionMode,
    selectedIds,
    setSelectedIds,
    allVisibleSelected,
    exitSelection,
    toggleSelectionMode: () => (selectionMode() ? exitSelection() : setSelectionMode(true)),
    toggleSelectAll: () =>
      setSelectedIds(allVisibleSelected() ? new Set<string>() : new Set(visibleIds())),
  };
}

function setPageAndExit(page: number, setPage: Setter<number>, exitSelection: () => void) {
  setPage(page);
  exitSelection();
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

export function useInboxState(query: Accessor<MessageListQuery>) {
  const [page, setPage] = createSignal(1);
  const [searchQuery, setSearchQuery] = createSignal('');
  const queryKey = createMemo(() => JSON.stringify(query() ?? {}));
  const currentQuery = createMemo<MessageListQuery>(() => ({
    ...query(),
    q: searchQuery().trim() || query()?.q,
    page: page(),
    limit: DEFAULT_LIMIT,
  }));
  const messagesQuery = useMessagesList(() => currentQuery());
  const selection = createSelectionState(() =>
    (messagesQuery.data?.items ?? []).map((message) => message.id),
  );
  const mutations = useInboxMutations(
    selection.selectedIds,
    selection.setSelectedIds,
    selection.setSelectionMode,
  );
  createEffect(() => {
    queryKey();
    setPage(1);
    selection.exitSelection();
  });
  return {
    page,
    searchQuery,
    messagesQuery,
    ...selection,
    ...mutations,
    setSearchQuery: (value: string) => {
      setSearchQuery(value);
      setPage(1);
      selection.exitSelection();
    },
    markAllRead: () => mutations.markAllRead(currentQuery()),
    totalPages: () =>
      messagesQuery.data ? Math.ceil(messagesQuery.data.total / messagesQuery.data.limit) : 0,
    setPageAndScroll: (newPage: number) =>
      setPageAndExit(newPage, setPage, selection.exitSelection),
  };
}
