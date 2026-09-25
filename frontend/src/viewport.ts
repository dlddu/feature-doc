import { useEffect, useState } from 'react';

/** Compact (< 600px) is the base design; this is the first width the same screen expands at. */
export const WIDE_QUERY = '(min-width: 600px)';

/**
 * True once the viewport reaches the first expansion breakpoint. Screens use it to
 * decide whether a long result starts summarised (AC4.4) or already unfolded — the
 * breakpoint is the one `index.css` already expands the layout at, so the fold and
 * the layout never disagree about what "compact" means.
 */
export function useWideViewport(): boolean {
  const [wide, setWide] = useState(() => window.matchMedia(WIDE_QUERY).matches);
  useEffect(() => {
    const query = window.matchMedia(WIDE_QUERY);
    const onChange = (e: MediaQueryListEvent) => setWide(e.matches);
    query.addEventListener('change', onChange);
    // The query can already have flipped between first render and this effect.
    setWide(query.matches);
    return () => query.removeEventListener('change', onChange);
  }, []);
  return wide;
}
