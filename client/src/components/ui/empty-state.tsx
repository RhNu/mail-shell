import type { JSX } from 'solid-js';
import { Inbox } from 'lucide-solid';

export type EmptyStateProps = {
  title?: string;
  description?: string;
  icon?: JSX.Element;
};

export function EmptyState(props: EmptyStateProps): JSX.Element {
  return (
    <div class="flex flex-col items-center justify-center py-16 text-center">
      <div class="mb-4 text-zinc-300 dark:text-zinc-600">
        {props.icon ?? <Inbox size={40} strokeWidth={1.5} />}
      </div>
      <h3 class="mb-1 text-base font-medium text-zinc-900 dark:text-zinc-100">
        {props.title ?? '暂无邮件'}
      </h3>
      <p class="max-w-xs text-sm text-zinc-500 dark:text-zinc-400">
        {props.description ?? '收到邮件后将显示在这里。'}
      </p>
    </div>
  );
}
