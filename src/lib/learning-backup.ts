/**
 * Sao lưu / khôi phục dữ liệu học ra một file JSON.
 *
 * App không có tài khoản: mọi thứ nằm trên trình duyệt, nên xoá dữ liệu
 * duyệt web hay đổi máy là mất sạch lịch sử học. File sao lưu là đường lui
 * duy nhất — xuất ra, cất đi, sang máy mới thì nhập lại.
 *
 * CHỈ gồm các khoá localStorage (tiến độ, bài làm, lịch sử học, ghi chú, tuỳ
 * chọn). Ghi âm và nét vẽ nằm trong IndexedDB, nặng hàng chục MB — không
 * nhét vào một file JSON được, và My page nói rõ điều đó.
 */

/** Khoá được sao lưu. Tên khoá là khoá tra dữ liệu — xem `storage-keys.test.ts`. */
export const BACKUP_KEYS = [
  "kiip-progress-v1",
  "kiip-quiz-v1",
  "kiip-activity-v1",
  "kiip-exam-v1",
  "kiip-notes-v1",
  "kiip-reader-prefs-v1",
] as const;

const APP_ID = "vietopik-learning-backup";
const FORMAT_VERSION = 1;

export interface LearningBackup {
  app: typeof APP_ID;
  version: number;
  exportedAt: string;
  /** Nguyên văn giá trị localStorage (chuỗi JSON) của từng khoá. */
  data: Partial<Record<(typeof BACKUP_KEYS)[number], string>>;
}

export function buildBackup(storage: Storage, now = new Date()): LearningBackup {
  const data: LearningBackup["data"] = {};
  for (const key of BACKUP_KEYS) {
    const value = storage.getItem(key);
    if (value !== null) data[key] = value;
  }
  return { app: APP_ID, version: FORMAT_VERSION, exportedAt: now.toISOString(), data };
}

export class BackupError extends Error {}

/**
 * Kiểm tra file rồi mới ghi, và kiểm HẾT trước khi ghi khoá nào: file hỏng
 * giữa chừng mà đã ghi được một nửa thì tiến độ của sách này là bản mới, bài
 * làm là bản cũ — lệch nhau mà không ai biết.
 *
 * Trả về số khoá đã ghi. Khoá không có trong file thì giữ nguyên dữ liệu
 * đang có, không xoá.
 */
export function restoreBackup(storage: Storage, text: string): number {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new BackupError("File không đọc được — không phải file sao lưu của VieTopik.");
  }
  const backup = parsed as Partial<LearningBackup> | null;
  if (!backup || backup.app !== APP_ID || typeof backup.data !== "object" || !backup.data) {
    throw new BackupError("Đây không phải file sao lưu dữ liệu học của VieTopik.");
  }
  if (typeof backup.version !== "number" || backup.version > FORMAT_VERSION) {
    throw new BackupError("File được tạo bởi phiên bản app mới hơn — hãy cập nhật app rồi thử lại.");
  }

  const entries: [string, string][] = [];
  for (const key of BACKUP_KEYS) {
    const value = (backup.data as Record<string, unknown>)[key];
    if (value === undefined) continue;
    if (typeof value !== "string") throw new BackupError(`File hỏng ở phần "${key}".`);
    try {
      JSON.parse(value);
    } catch {
      throw new BackupError(`File hỏng ở phần "${key}".`);
    }
    entries.push([key, value]);
  }

  for (const [key, value] of entries) storage.setItem(key, value);
  return entries.length;
}

/** Tên file gợi ý, có ngày để cất nhiều bản không đè nhau. */
export function backupFileName(now = new Date()): string {
  const d = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(
    now.getDate()
  ).padStart(2, "0")}`;
  return `vietopik-du-lieu-hoc-${d}.json`;
}
