// Vitest shim for Next.js `server-only`. The real module throws when imported
// into a client bundle — in tests we run in Node, so we intentionally do
// nothing here.
export {};
