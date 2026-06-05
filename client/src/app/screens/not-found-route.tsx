import { ArrowLeft } from 'lucide-solid';

export function NotFoundRoute() {
  return (
    <section class="flex flex-col items-center justify-center py-20 text-center">
      <h1 class="text-6xl font-bold tracking-tight text-zinc-200 dark:text-zinc-800">404</h1>
      <h2 class="mt-4 text-xl font-semibold text-zinc-900 dark:text-zinc-100">Page not found</h2>
      <p class="mt-2 max-w-xs text-sm text-zinc-500 dark:text-zinc-400">
        The page you are looking for does not exist.
      </p>
      <a
        href="#/"
        class="mt-6 inline-flex items-center gap-1.5 rounded-sm bg-zinc-900 px-4 py-2 text-sm font-medium text-zinc-50 transition-colors hover:bg-zinc-800 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200"
      >
        <ArrowLeft size={16} />
        Back to inbox
      </a>
    </section>
  );
}
