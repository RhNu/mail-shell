import { Show } from 'solid-js';
import type { JSX } from 'solid-js';

export type TagChipProps = {
  label: string;
  href?: string;
  active?: boolean;
};

export function TagChip(props: TagChipProps): JSX.Element {
  const classes = () =>
    [
      'inline-flex items-center rounded-sm px-2 py-0.5 text-xs font-medium transition-colors',
      props.active
        ? 'bg-zinc-900 text-zinc-50 dark:bg-zinc-100 dark:text-zinc-900'
        : 'bg-zinc-100 text-zinc-700 dark:bg-zinc-800 dark:text-zinc-300',
    ].join(' ');

  return (
    <Show when={props.href} fallback={<span class={classes()}>{props.label}</span>}>
      <a href={props.href} class={classes()}>
        {props.label}
      </a>
    </Show>
  );
}
