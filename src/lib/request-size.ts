import { type NextRequest, NextResponse } from 'next/server';

// Per-endpoint payload caps. Pick the smallest cap that still fits the
// largest legitimate request for that route.
//
// Why per-route instead of a global middleware?
//   1. Image upload (base64 data URLs) needs ~7MB; everything else is <100KB.
//   2. Next.js middleware can't easily inspect parsed body — we'd have to
//      read the stream there, then again in the route handler.
//   3. Per-route limits document the expected payload shape next to the
//      handler that consumes it, which catches drift during review.
export const SIZE_LIMITS = {
  /** Tiny JSON commands: filters, status updates, login tokens. */
  small: 16 * 1024, // 16KB
  /** Form submissions, search params, individual entity creates. */
  medium: 256 * 1024, // 256KB
  /** Content with embedded HTML/Markdown (news article, terms page). */
  large: 1 * 1024 * 1024, // 1MB
  /** Image-bearing payloads (course/combo with base64 gallery). */
  upload: 8 * 1024 * 1024, // 8MB
} as const;

export type SizeLimit = keyof typeof SIZE_LIMITS;

/**
 * Read the request body as text and reject if it exceeds the per-endpoint
 * cap. Returns either the parsed JSON or an early `NextResponse` (413)
 * that the route handler should return directly.
 *
 * Pattern:
 *   const parsed = await readJsonWithLimit(req, 'medium');
 *   if (parsed instanceof NextResponse) return parsed;
 *   const body = parsed;
 */
export async function readJsonWithLimit<T = unknown>(
  req: NextRequest,
  limit: SizeLimit,
): Promise<T | NextResponse> {
  const cap = SIZE_LIMITS[limit];
  const lengthHeader = req.headers.get('content-length');
  if (lengthHeader && Number(lengthHeader) > cap) {
    return NextResponse.json(
      { error: 'Payload too large', limit: cap, received: Number(lengthHeader) },
      { status: 413 },
    );
  }
  const text = await req.text();
  if (text.length > cap) {
    return NextResponse.json(
      { error: 'Payload too large', limit: cap, received: text.length },
      { status: 413 },
    );
  }
  try {
    return JSON.parse(text) as T;
  } catch {
    return NextResponse.json({ error: 'Invalid JSON' }, { status: 400 });
  }
}
