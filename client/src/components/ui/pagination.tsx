import { For } from 'solid-js';
import type { JSX } from 'solid-js';
import { ChevronLeft, ChevronRight } from 'lucide-solid';

export type PaginationProps = {
  page: number;
  totalPages: number;
  // eslint-disable-next-line no-unused-vars
  onPageChange: (page: number) => void;
};

const btnCls =
  'inline-flex h-8 w-8 items-center justify-center rounded-sm border border-zinc-200 text-zinc-600 transition-colors hover:bg-zinc-100 disabled:cursor-not-allowed disabled:opacity-40 dark:border-zinc-700 dark:text-zinc-400 dark:hover:bg-zinc-800';

function pageNumbers(page: number, totalPages: number): (number | string)[] {
  const pages: (number | string)[] = [];
  const maxVisible = 5;
  const half = Math.floor(maxVisible / 2);
  let start = Math.max(1, page - half);
  let end = Math.min(totalPages, start + maxVisible - 1);
  if (end - start + 1 < maxVisible) start = Math.max(1, end - maxVisible + 1);
  if (start > 1) {
    pages.push(1);
    if (start > 2) pages.push('…');
  }
  for (let i = start; i <= end; i++) pages.push(i);
  if (end < totalPages) {
    if (end < totalPages - 1) pages.push('…');
    pages.push(totalPages);
  }
  return pages;
}

export function Pagination(props: PaginationProps): JSX.Element {
  const canGoPrev = () => props.page > 1;
  const canGoNext = () => props.page < props.totalPages;
  return (
    <nav aria-label="分页" class="flex items-center justify-center gap-1">
      <button
        type="button"
        onClick={() => props.onPageChange(props.page - 1)}
        disabled={!canGoPrev()}
        class={btnCls}
        aria-label="上一页"
      >
        <ChevronLeft size={16} />
      </button>
      <For each={pageNumbers(props.page, props.totalPages)}>
        {(p) =>
          typeof p === 'string' ? (
            <span class="inline-flex h-8 w-8 items-center justify-center text-sm text-zinc-400 dark:text-zinc-500">
              {p}
            </span>
          ) : (
            <button
              type="button"
              onClick={() => props.onPageChange(p)}
              class={[
                'inline-flex h-8 min-w-[2rem] items-center justify-center rounded-sm border px-2 text-sm transition-colors',
                p === props.page
                  ? 'border-zinc-900 bg-zinc-900 text-zinc-50 dark:border-zinc-100 dark:bg-zinc-100 dark:text-zinc-900'
                  : 'border-zinc-200 text-zinc-600 hover:bg-zinc-100 dark:border-zinc-700 dark:text-zinc-400 dark:hover:bg-zinc-800',
              ].join(' ')}
              aria-current={p === props.page ? 'page' : undefined}
            >
              {p}
            </button>
          )
        }
      </For>
      <button
        type="button"
        onClick={() => props.onPageChange(props.page + 1)}
        disabled={!canGoNext()}
        class={btnCls}
        aria-label="下一页"
      >
        <ChevronRight size={16} />
      </button>
    </nav>
  );
}
