import type { JSX } from 'solid-js';

export type SkeletonProps = {
  class?: string;
};

export function Skeleton(props: SkeletonProps): JSX.Element {
  return (
    <div class={['animate-pulse bg-zinc-200 dark:bg-zinc-800', props.class ?? ''].join(' ')} />
  );
}
