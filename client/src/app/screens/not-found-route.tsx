export function NotFoundRoute() {
  return (
    <section aria-labelledby="not-found-heading" class="flex flex-col gap-3">
      <h2 id="not-found-heading" class="text-2xl font-semibold">
        Route not found
      </h2>
      <p class="max-w-2xl text-sm leading-6 text-stone-300">
        Unknown client route. Keep this separate from server 404 handling because the app uses hash
        routing.
      </p>
    </section>
  );
}
