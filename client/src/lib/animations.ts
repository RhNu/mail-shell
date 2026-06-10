/**
 * Return an `animation-delay` value for staggered list items.
 * Falls back to `0ms` when reduced motion is preferred.
 */
export function staggerDelay(index: number, baseMs = 40, maxMs = 400): string {
  const delay = Math.min(index * baseMs, maxMs);
  return `${delay}ms`;
}

/**
 * Detect whether the user prefers reduced motion.
 */
export function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}
