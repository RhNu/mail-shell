import { useParams } from '@solidjs/router';

export function TaggedInboxRoute() {
  const params = useParams<{ tagId: string }>();

  return (
    <section aria-labelledby="tagged-inbox-heading" class="flex flex-col gap-3">
      <h2 id="tagged-inbox-heading" class="text-2xl font-semibold">
        Tagged inbox
      </h2>
      <p class="max-w-2xl text-sm leading-6 text-stone-300">
        Filtered message list route scoped to a single tag. This keeps tag drill-down shareable as a
        first-class URL instead of a purely local UI state.
      </p>
      <p class="text-sm text-stone-400">Selected tag: {params.tagId}</p>
    </section>
  );
}
