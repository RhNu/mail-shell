import { For } from 'solid-js';
import { useLocation } from '@solidjs/router';
import { Hash } from 'lucide-solid';
import { useTagsList } from '../features/tags/queries';
import { tagInboxHref } from '../app/routes';

function groupByKind<T extends { kind: string }>(items: T[]): Record<string, T[]> {
  return items.reduce(
    (acc, item) => {
      if (!acc[item.kind]) acc[item.kind] = [];
      acc[item.kind].push(item);
      return acc;
    },
    {} as Record<string, T[]>,
  );
}

function kindLabel(kind: string): string {
  switch (kind) {
    case 'recipient_address':
      return 'Recipients';
    case 'recipient_domain':
      return 'Domains';
    default:
      return kind;
  }
}

export function TagNav() {
  const location = useLocation();
  const tagsQuery = useTagsList();
  const tags = () => tagsQuery.data ?? [];
  const groups = () => groupByKind(tags());
  const kinds = () => Object.keys(groups()).toSorted();

  return (
    <>
      <For each={kinds()}>
        {(kind) => (
          <div class="mb-4">
            <p class="mb-1.5 px-3 text-xs font-semibold uppercase tracking-wider text-zinc-400 dark:text-zinc-500">
              {kindLabel(kind)}
            </p>
            <div class="flex flex-col gap-0.5">
              <For each={groups()[kind]}>
                {(tag) => {
                  const href = `#${tagInboxHref(String(tag.id))}`;
                  const active = location.pathname === `/tags/${tag.id}`;
                  return (
                    <a
                      href={href}
                      class={[
                        'flex items-center gap-2.5 rounded-sm px-3 py-1.5 text-sm transition-colors',
                        active
                          ? 'bg-zinc-200 font-medium text-zinc-900 dark:bg-zinc-800 dark:text-zinc-100'
                          : 'text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-50',
                      ].join(' ')}
                    >
                      <Hash size={14} class="shrink-0 opacity-60" />
                      <span class="truncate">{tag.label}</span>
                      {tag.message_count !== null && tag.message_count !== undefined && (
                        <span class="ml-auto text-xs text-zinc-400 dark:text-zinc-500">
                          {tag.message_count}
                        </span>
                      )}
                    </a>
                  );
                }}
              </For>
            </div>
          </div>
        )}
      </For>
    </>
  );
}
