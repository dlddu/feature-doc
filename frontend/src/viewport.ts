import { useEffect, useState } from 'react';

/** The first width `index.css` expands the layout at — the fold and the layout must agree. */
export const WIDE_QUERY = '(min-width: 600px)';

/** True once the viewport reaches that expansion breakpoint. */
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
