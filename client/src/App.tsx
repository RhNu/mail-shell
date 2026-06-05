export default function App() {
  return (
    <main class="min-h-screen bg-stone-950 text-stone-100">
      <section class="mx-auto flex min-h-screen max-w-5xl flex-col justify-center gap-6 px-6 py-20">
        <p class="text-sm uppercase tracking-[0.3em] text-stone-400">mail-shell</p>
        <h1 class="max-w-3xl text-4xl font-semibold tracking-tight sm:text-6xl">
          A small inbox surface for Cloudflare-routed mail.
        </h1>
        <p class="max-w-2xl text-base leading-7 text-stone-300 sm:text-lg">
          This scaffold reserves the client route space, API prefix, and system-tag model
          for recipient-address and recipient-domain classification.
        </p>
      </section>
    </main>
  );
}

