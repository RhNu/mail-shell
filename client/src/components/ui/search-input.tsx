import type { JSX } from 'solid-js';
import { Search } from 'lucide-solid';

export type SearchInputProps = {
  value: string;
  // eslint-disable-next-line no-unused-vars
  onChange: (value: string) => void;
  placeholder?: string;
};

export function SearchInput(props: SearchInputProps): JSX.Element {
  return (
    <div class="relative flex items-center">
      <Search
        size={16}
        class="absolute left-2.5 text-zinc-400 dark:text-zinc-500"
        aria-hidden="true"
      />
      <input
        data-mail-search
        aria-keyshortcuts="/"
        type="text"
        value={props.value}
        onInput={(e) => props.onChange(e.currentTarget.value)}
        placeholder={props.placeholder ?? '搜索...'}
        class="w-full rounded-sm border border-zinc-200 bg-white py-1.5 pr-9 pl-8 text-sm text-zinc-900 outline-none transition-[colors,box-shadow,transform] duration-200 ease-out placeholder:text-zinc-400 focus:scale-[1.01] focus:border-zinc-400 focus:ring-2 focus:ring-zinc-300/50 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100 dark:placeholder:text-zinc-500 dark:focus:border-zinc-500 dark:focus:ring-zinc-600/50"
      />
      <kbd class="pointer-events-none absolute right-2 hidden rounded border border-zinc-200 px-1.5 py-0.5 text-[10px] text-zinc-400 sm:block dark:border-zinc-700 dark:text-zinc-500">
        /
      </kbd>
    </div>
  );
}
