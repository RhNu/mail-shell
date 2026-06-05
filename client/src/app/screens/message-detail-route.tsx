import { useParams } from '@solidjs/router';

export function MessageDetailRoute() {
  const params = useParams<{ messageId: string }>();

  return (
    <section aria-labelledby="message-detail-heading" class="flex flex-col gap-3">
      <h2 id="message-detail-heading" class="text-2xl font-semibold">
        Message detail
      </h2>
      <p class="max-w-2xl text-sm leading-6 text-stone-300">
        Message route for a single mail item, including body, metadata, and attachment actions.
      </p>
      <p class="text-sm text-stone-400">Selected message: {params.messageId}</p>
    </section>
  );
}
