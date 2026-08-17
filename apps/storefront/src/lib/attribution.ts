/**
 * Source attribution helper (P10-03d):
 * Extracts ?ref= affiliate / marketing code from incoming URL params,
 * stores it in session / cookies, and attaches to Cart metadata
 * so that it automatically propagates to Order metadata on complete-cart.
 */

export const REF_PARAM_KEY = 'ref';
export const REF_COOKIE_NAME = 'stheno_ref_attribution';

export function extractAttributionFromUrl(searchParams: URLSearchParams): string | null {
  const ref = searchParams.get(REF_PARAM_KEY);
  if (ref && ref.trim().length > 0) {
    return ref.trim();
  }
  return null;
}

export function attachAttributionToCartMetadata(
  existingMetadata: Record<string, any> = {},
  refCode?: string | null
): Record<string, any> {
  if (!refCode) {
    return existingMetadata;
  }
  return {
    ...existingMetadata,
    source: 'social',
    ref: refCode,
    attributed_at: new Date().toISOString(),
  };
}
