import { For, Show } from 'solid-js';
import { useLocation } from '@solidjs/router';
import { Bookmark, Compass, Settings, Tag } from 'lucide-solid';
import { useLabels, useSavedViews } from '../features/classification/queries';
import type { Label, SavedView } from '../features/classification/api';

const itemClass = (active: boolean) =>
  [
    'flex items-center gap-2.5 rounded-sm px-3 py-1.5 text-sm transition-colors',
    active
      ? 'bg-zinc-200 font-medium text-zinc-900 dark:bg-zinc-800 dark:text-zinc-100'
      : 'text-zinc-600 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-800',
  ].join(' ');

function PinnedViews(props: { views: SavedView[]; path: string; onNavigate?: () => void }) {
  return (
    <Show when={props.views.some((view) => view.pinned)}>
      <div>
        <p class="mb-1.5 px-3 text-xs font-semibold tracking-wider text-zinc-400">固定视图</p>
        <For each={props.views.filter((view) => view.pinned)}>
          {(view) => (
            <a
              href={`#/views/${view.id}`}
              onClick={props.onNavigate}
              class={itemClass(props.path === `/views/${view.id}`)}
            >
              <Bookmark size={14} />
              <span class="truncate">{view.name}</span>
            </a>
          )}
        </For>
      </div>
    </Show>
  );
}

function LabelsNav(props: { labels: Label[]; path: string; onNavigate?: () => void }) {
  return (
    <div>
      <p class="mb-1.5 px-3 text-xs font-semibold tracking-wider text-zinc-400">标签</p>
      <For each={props.labels}>
        {(label) => (
          <a
            href={`#/labels/${label.id}`}
            onClick={props.onNavigate}
            class={itemClass(props.path === `/labels/${label.id}`)}
          >
            <Tag size={14} />
            <span class="truncate">{label.name}</span>
            <span class="ml-auto text-xs text-zinc-400">{label.message_count ?? 0}</span>
          </a>
        )}
      </For>
    </div>
  );
}

export function TagNav(props: { onNavigate?: () => void } = {}) {
  const location = useLocation();
  const labels = useLabels();
  const views = useSavedViews();
  return (
    <div class="flex flex-col gap-4">
      <PinnedViews
        views={views.data ?? []}
        path={location.pathname}
        onNavigate={props.onNavigate}
      />
      <LabelsNav
        labels={labels.data ?? []}
        path={location.pathname}
        onNavigate={props.onNavigate}
      />
      <div class="flex flex-col gap-0.5 border-t border-zinc-200 pt-3 dark:border-zinc-800">
        <a
          href="#/facets"
          onClick={props.onNavigate}
          class={itemClass(location.pathname === '/facets')}
        >
          <Compass size={14} />
          浏览分类
        </a>
        <a
          href="#/classification"
          onClick={props.onNavigate}
          class={itemClass(location.pathname === '/classification')}
        >
          <Settings size={14} />
          分类设置
        </a>
      </div>
    </div>
  );
}
