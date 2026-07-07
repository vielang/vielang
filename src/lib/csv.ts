/**
 * Tiny CSV parser. We avoid papaparse because:
 *   1. We control the input (admin-uploaded CSV from a known template).
 *   2. The full RFC 4180 grammar fits in ~50 lines if we accept that
 *      embedded newlines inside quoted fields are the only edge case.
 *   3. Skipping the dep keeps the admin bundle ~30KB smaller.
 *
 * Handles: comma-separated, RFC-4180 double-quote escaping (""), quoted
 * fields containing commas/newlines, CRLF or LF line endings, leading BOM.
 * Does NOT handle: custom delimiters, multi-line headers, comments.
 */
export function parseCSV(input: string): string[][] {
  // Strip optional UTF-8 BOM that Excel adds to CSV exports.
  if (input.charCodeAt(0) === 0xfeff) input = input.slice(1);

  const rows: string[][] = [];
  let row: string[] = [];
  let field = '';
  let inQuotes = false;

  for (let i = 0; i < input.length; i++) {
    const c = input[i];
    if (inQuotes) {
      if (c === '"') {
        if (input[i + 1] === '"') {
          field += '"';
          i++;
        } else {
          inQuotes = false;
        }
      } else {
        field += c;
      }
    } else if (c === '"') {
      inQuotes = true;
    } else if (c === ',') {
      row.push(field);
      field = '';
    } else if (c === '\n' || c === '\r') {
      // Push row only if it's non-empty (skip blank trailing lines).
      row.push(field);
      field = '';
      if (row.length > 1 || row[0] !== '') rows.push(row);
      row = [];
      if (c === '\r' && input[i + 1] === '\n') i++;
    } else {
      field += c;
    }
  }
  // Trailing field with no final newline.
  if (field !== '' || row.length > 0) {
    row.push(field);
    rows.push(row);
  }
  return rows;
}

/**
 * Parse CSV into objects keyed by the header row. Header names are trimmed.
 * Returns `{ rows, errors }` where errors lists row indices with mismatched
 * column counts (silently dropped to avoid corrupting partial imports).
 */
export function parseCSVAsObjects(input: string): {
  rows: Record<string, string>[];
  errors: Array<{ line: number; reason: string }>;
} {
  const raw = parseCSV(input);
  if (raw.length === 0) return { rows: [], errors: [{ line: 0, reason: 'empty CSV' }] };
  const headers = raw[0].map((h) => h.trim());
  const errors: Array<{ line: number; reason: string }> = [];
  const rows: Record<string, string>[] = [];
  for (let i = 1; i < raw.length; i++) {
    const r = raw[i];
    if (r.length !== headers.length) {
      errors.push({ line: i + 1, reason: `expected ${headers.length} columns, got ${r.length}` });
      continue;
    }
    const obj: Record<string, string> = {};
    for (let j = 0; j < headers.length; j++) obj[headers[j]] = r[j];
    rows.push(obj);
  }
  return { rows, errors };
}
