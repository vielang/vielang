// Neutral mid-tone shimmer that works on both light and dark backgrounds.
// Base64 data URLs cannot use CSS vars, so we pick a tone (~slate-400/500) that's
// acceptable in light (slightly dark) and in dark (slightly light) without glare.
const shimmer = (w: number, h: number) => `
<svg width="${w}" height="${h}" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
  <defs>
    <linearGradient id="g">
      <stop stop-color="#94a3b8" offset="20%" />
      <stop stop-color="#cbd5e1" offset="50%" />
      <stop stop-color="#94a3b8" offset="70%" />
    </linearGradient>
  </defs>
  <rect width="${w}" height="${h}" fill="#94a3b8" />
  <rect id="r" width="${w}" height="${h}" fill="url(#g)" />
  <animate xlink:href="#r" attributeName="x" from="-${w}" to="${w}" dur="1.2s" repeatCount="indefinite"  />
</svg>`;

const toBase64 = (str: string) =>
  typeof window === 'undefined' ? Buffer.from(str).toString('base64') : window.btoa(str);

export const shimmerDataUrl = (w: number = 640, h: number = 400) =>
  `data:image/svg+xml;base64,${toBase64(shimmer(w, h))}`;

export const SHIMMER_PLACEHOLDER = shimmerDataUrl();

const DEFAULT_FALLBACK = '/images/1.jpeg';

export function safeImageSrc(src: unknown, fallback: string = DEFAULT_FALLBACK): string {
  if (typeof src !== 'string') return fallback;
  const trimmed = src.trim();
  if (!trimmed) return fallback;
  if (trimmed.startsWith('/')) return trimmed;
  if (trimmed.startsWith('data:image/')) return trimmed;
  if (/^https?:\/\//i.test(trimmed)) {
    try {
      new URL(trimmed);
      return trimmed;
    } catch {
      return fallback;
    }
  }
  return fallback;
}
