import { createSignal, For } from 'solid-js';
import { useClassificationMutations, useSavedViews } from '../../features/classification/queries';

// oxlint-disable-next-line max-lines-per-function
export function SavedViewManager() {
  const views = useSavedViews();
  const mutations = useClassificationMutations();
  const [name, setName] = createSignal('');
  const [query, setQuery] = createSignal('');
  const submit = (event: SubmitEvent) => {
    event.preventDefault();
    if (!name().trim()) return;
    mutations.createView.mutate(
      {
        name: name().trim(),
        pinned: true,
        sort_order: 0,
        query: { mailbox: 'inbox', q: query().trim() || undefined },
      },
      {
        onSuccess: () => {
          setName('');
          setQuery('');
        },
      },
    );
  };
  return (
    <section class="flex flex-col gap-3 rounded-sm border border-zinc-200 p-4 dark:border-zinc-800">
      <h2 class="font-semibold">固定智能视图</h2>
      <form class="flex flex-col gap-2 sm:flex-row" onSubmit={submit}>
        <input
          class="rounded-sm border p-2 text-sm dark:bg-zinc-950"
          value={name()}
          onInput={(event) => setName(event.currentTarget.value)}
          placeholder="视图名称"
        />
        <input
          class="min-w-0 flex-1 rounded-sm border p-2 text-sm dark:bg-zinc-950"
          value={query()}
          onInput={(event) => setQuery(event.currentTarget.value)}
          placeholder="搜索条件"
        />
        <button
          type="submit"
          class="rounded-sm bg-zinc-900 px-3 py-2 text-sm text-white dark:bg-zinc-100 dark:text-zinc-900"
        >
          保存
        </button>
      </form>
      <For each={views.data ?? []}>
        {(view) => (
          <div class="flex items-center justify-between text-sm">
            <span>{view.name}</span>
            <button
              type="button"
              class="text-xs text-red-600"
              onClick={() => mutations.deleteView.mutate(view.id)}
            >
              删除
            </button>
          </div>
        )}
      </For>
    </section>
  );
}
