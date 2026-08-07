// Small presentational helpers shared by S02 · S03.

/** Cents → `$1.23`. The backend reports every estimate in whole cents. */
export function formatCost(cents: number): string {
  return `$${(cents / 100).toFixed(2)}`;
}

/** Bytes → a short human size (`2.3 MB`), matching the S03 mockup's density. */
export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${Math.round(kb)} KB`;
  return `${(kb / 1024).toFixed(1)} MB`;
}

/** Unix seconds → `4m ago` / `2 days ago`, as the S02 repo cards read. */
export function formatAgo(unixSeconds: number, now = Date.now()): string {
  const seconds = Math.max(0, Math.floor(now / 1000) - unixSeconds);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return days === 1 ? '1 day ago' : `${days} days ago`;
}
