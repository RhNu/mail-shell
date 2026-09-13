import { createMemo, createSignal, For } from 'solid-js';
import { useFacets } from '../../features/classification/queries';
import { SearchInput } from '../../components/ui';

function FacetKindButtons(props: {
  // eslint-disable-next-line no-unused-vars
  onSelect: (kind: string) => void;
}) {
  const kinds = [
    ['recipient', '收件地址'],
    ['sender', '发件人'],
    ['domain', '域名'],
  ] as const;
  return (
    <div class="flex gap-2">
      <For each={kinds}>
        {([value, label]) => (
          <button
            type="button"
            onClick={() => props.onSelect(value)}
            class="rounded-sm border border-zinc-300 px-3 py-1.5 text-sm dark:border-zinc-700"
          >
            {label}
          </button>
        )}
      </For>
    </div>
  );
}

export function FacetsRoute() {
  const [kind, setKind] = createSignal('recipient');
  const [search, setSearch] = createSignal('');
  const facets = useFacets(kind);
  const visible = createMemo(() => {
    const query = search().trim().toLowerCase();
    return (facets.data ?? []).filter((facet) => !query || facet.value.includes(query));
  });
  return (
    <section class="flex flex-col gap-4">
      <div>
        <h1 class="text-xl font-semibold">浏览分类</h1>
        <p class="text-sm text-zinc-500">系统维度只显示当前仍有邮件的项目。</p>
      </div>
      <FacetKindButtons onSelect={setKind} />
      <SearchInput value={search()} onChange={setSearch} placeholder="筛选分类..." />
      <div class="divide-y divide-zinc-200 border-y border-zinc-200 dark:divide-zinc-800 dark:border-zinc-800">
        <For each={visible()}>
          {(facet) => (
            <a
              class="flex justify-between px-3 py-2 text-sm hover:bg-zinc-100 dark:hover:bg-zinc-900"
              href={`#/search?q=${encodeURIComponent(facet.value)}`}
            >
              <span>{facet.label}</span>
              <span class="text-zinc-400">{facet.message_count}</span>
            </a>
          )}
        </For>
      </div>
    </section>
  );
}
