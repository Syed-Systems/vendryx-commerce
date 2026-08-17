/**
 * Stheno Admin API Client (P18-03):
 * Communicates with stheno-server at /admin endpoints.
 */

export interface ApiResponse<T> {
  data?: T;
  count?: number;
  offset?: number;
  limit?: number;
  [key: string]: any;
}

const BASE_URL = import.meta.env.VITE_MEDUSA_BACKEND_URL || 'http://localhost:9000';

export async function adminFetch<T = any>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${BASE_URL}${path.startsWith('/') ? path : `/${path}`}`;
  const headers = new Headers(options.headers || {});

  if (!headers.has('Content-Type') && options.body && typeof options.body === 'string') {
    headers.set('Content-Type', 'application/json');
  }

  const token = localStorage.getItem('stheno_admin_token');
  if (token && !headers.has('Authorization')) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  const response = await fetch(url, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const errorText = await response.text();
    let errorJson;
    try {
      errorJson = JSON.parse(errorText);
    } catch {
      errorJson = { message: errorText || `HTTP ${response.status}` };
    }
    throw new Error(errorJson.message || `Request failed with status ${response.status}`);
  }

  const contentType = response.headers.get('content-type');
  if (contentType && contentType.includes('application/json')) {
    return response.json();
  }
  return (await response.text()) as any;
}
