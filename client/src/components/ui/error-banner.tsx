import type { JSX } from 'solid-js';
import { AlertTriangle, RotateCcw } from 'lucide-solid';

export type ErrorBannerProps = {
  title?: string;
  message: string;
  onRetry?: () => void;
};

export function ErrorBanner(props: ErrorBannerProps): JSX.Element {
  return (
    <div class="animate-slide-down flex items-start gap-3 rounded-sm border border-red-200 bg-red-50 p-4 dark:border-red-900/30 dark:bg-red-950/20">
      <AlertTriangle
        size={18}
        class="mt-0.5 shrink-0 text-red-600 dark:text-red-400"
        aria-hidden="true"
      />
      <div class="flex-1">
        <p class="text-sm font-medium text-red-800 dark:text-red-300">
          {props.title ?? '出了点问题'}
        </p>
        <p class="mt-0.5 text-sm text-red-700 dark:text-red-400">{props.message}</p>
      </div>
      {props.onRetry && (
        <button
          type="button"
          onClick={props.onRetry}
          class="inline-flex shrink-0 items-center gap-1.5 rounded-sm bg-red-100 px-2.5 py-1.5 text-sm font-medium text-red-800 transition-colors hover:bg-red-200 dark:bg-red-900/30 dark:text-red-300 dark:hover:bg-red-900/50"
        >
          <RotateCcw size={14} />
          重试
        </button>
      )}
    </div>
  );
}
