import { toast } from 'sonner';
import { getAuthHeaders } from '@/contexts';

// Owner CMS image constraints. Files flow through Supabase Storage via
// /api/upload (not base64 in JSON), so the previous row-payload concern is
// gone — cap aligned with the bucket's server-side limit.
const MAX_FILE_SIZE_BYTES = 10 * 1024 * 1024; // 10MB
const MAX_FILE_SIZE_LABEL = '10MB';

type UiLang = 'VN' | 'EN';

/**
 * Validate a single file is an image within the size cap. Returns true if
 * acceptable; otherwise toasts an i18n-aware message and returns false.
 * Caller can `if (!isValidImageFile(...)) continue;` inside upload loops.
 */
export function isValidImageFile(file: File, lang: UiLang = 'EN'): boolean {
  if (!file.type.startsWith('image/')) {
    toast.error(lang === 'VN' ? 'Chỉ chấp nhận file ảnh.' : 'Only image files are allowed.');
    return false;
  }
  if (file.size > MAX_FILE_SIZE_BYTES) {
    toast.error(
      lang === 'VN'
        ? `File không được vượt quá ${MAX_FILE_SIZE_LABEL}.`
        : `File must be under ${MAX_FILE_SIZE_LABEL}.`,
    );
    return false;
  }
  return true;
}

/**
 * Upload a validated image to Supabase Storage via /api/upload. Returns the
 * public URL on success, or null on failure (already toasts the error).
 * Folder scopes the storage path so we can audit per surface — pass e.g.
 * "courses", "combos", "news", "banners".
 */
export async function uploadImageToStorage(
  file: File,
  opts: { folder?: string; lang?: UiLang } = {},
): Promise<string | null> {
  const { folder = 'misc', lang = 'EN' } = opts;
  try {
    const fd = new FormData();
    fd.append('file', file);
    fd.append('folder', folder);
    const headers = await getAuthHeaders();
    const res = await fetch('/api/upload', {
      method: 'POST',
      body: fd,
      headers, // intentionally no Content-Type — fetch sets it with boundary
      credentials: 'include',
    });
    if (!res.ok) {
      let msg = `HTTP ${res.status}`;
      try {
        const j = await res.json();
        msg = j.error || msg;
      } catch {
        /* ignore */
      }
      toast.error(lang === 'VN' ? `Upload thất bại: ${msg}` : `Upload failed: ${msg}`);
      return null;
    }
    const data = await res.json();
    return data.url || null;
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : '';
    toast.error(lang === 'VN' ? `Lỗi mạng: ${message}` : `Network error: ${message}`);
    return null;
  }
}
