export type ReviewPosition = {
  featureKey: string;
  where: string;
};

const PREFIX = 'review-position-';

function keyOf(id: string): string {
  return PREFIX + id;
}

export function markReviewPosition(id: string, position: ReviewPosition): void {
  try {
    window.sessionStorage.setItem(keyOf(id), JSON.stringify(position));
  } catch {
  }
}

export function readReviewPosition(id: string): ReviewPosition | null {
  try {
    const raw = window.sessionStorage.getItem(keyOf(id));
    if (raw === null) return null;
    const parsed = JSON.parse(raw) as Partial<ReviewPosition>;
    if (typeof parsed.featureKey !== 'string' || typeof parsed.where !== 'string') return null;
    return { featureKey: parsed.featureKey, where: parsed.where };
  } catch {
    return null;
  }
}

export function clearReviewPosition(id: string): void {
  try {
    window.sessionStorage.removeItem(keyOf(id));
  } catch {
  }
}
