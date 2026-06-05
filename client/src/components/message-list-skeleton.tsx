import { Index } from 'solid-js';
import type { JSX } from 'solid-js';
import { Skeleton } from './ui/skeleton';

export function MessageListSkeleton(): JSX.Element {
  return (
    <div class="border-t border-zinc-200 dark:border-zinc-800">
      <Index each={Array.from({ length: 7 })}>
        {() => (
          <div class="flex items-center gap-4 border-b border-zinc-100 px-4 py-3 dark:border-zinc-800/60">
            <Skeleton class="h-4 w-32 shrink-0 rounded-sm" />
            <Skeleton class="h-4 flex-1 rounded-sm" />
            <Skeleton class="h-3 w-12 shrink-0 rounded-sm" />
          </div>
        )}
      </Index>
    </div>
  );
}
