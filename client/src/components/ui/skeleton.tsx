import type { JSX } from 'solid-js';

export type SkeletonProps = {
  class?: string;
};

export function Skeleton(props: SkeletonProps): JSX.Element {
  return (
    <div
      class={[
        'animate-shimmer bg-gradient-to-r from-zinc-200 via-zinc-100 to-zinc-200 bg-[length:200%_100%] dark:from-zinc-800 dark:via-zinc-700 dark:to-zinc-800',
        props.class ?? '',
      ].join(' ')}
    />
  );
}
