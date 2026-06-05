export function InboxRoute() {
  return (
    <section aria-labelledby="inbox-heading" class="flex flex-col gap-3">
      <h2 id="inbox-heading" class="text-2xl font-semibold">
        Inbox
      </h2>
      <p class="max-w-2xl text-sm leading-6 text-stone-300">
        Default route for the message list. This screen will own list pagination, tag chips, summary
        rows, and empty or error states.
      </p>
    </section>
  );
}
