import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// pino writes JSON to stdout — we intercept the underlying logger.error to
// assert scope + message instead of trying to capture stdout.
const loggerErrorMock = vi.fn();
vi.mock('./logger', () => ({
  logger: {
    error: (...args: unknown[]) => loggerErrorMock(...args),
    child: () => ({ error: (...args: unknown[]) => loggerErrorMock(...args) }),
  },
  childLogger: () => ({ error: (...args: unknown[]) => loggerErrorMock(...args) }),
}));

// Sentry SDK is a heavy import — stub with a no-op so tests don't dial home.
vi.mock('@sentry/nextjs', () => ({
  captureException: vi.fn(),
  setUser: vi.fn(),
}));

import { apiError } from './api-errors';

beforeEach(() => {
  loggerErrorMock.mockClear();
});

afterEach(() => {
  vi.unstubAllEnvs();
});

async function readJson(res: Response) {
  return (await res.json()) as { error: string; debug?: string };
}

describe('apiError', () => {
  it('returns a 500 with a generic error message', async () => {
    const res = apiError(new Error('boom'));
    expect(res.status).toBe(500);
    const body = await readJson(res);
    expect(body.error).toBe('Internal server error');
  });

  it('includes debug field in non-production', async () => {
    vi.stubEnv('NODE_ENV', 'development');
    const res = apiError(new Error('boom'), 'sessions');
    const body = await readJson(res);
    expect(body.debug).toBe('boom');
  });

  it('OMITS debug field in production', async () => {
    vi.stubEnv('NODE_ENV', 'production');
    const res = apiError(new Error('boom'), 'sessions');
    const body = await readJson(res);
    expect(body.debug).toBeUndefined();
  });

  it('describes a Supabase-shaped error with code/details/hint', async () => {
    vi.stubEnv('NODE_ENV', 'development');
    const supabaseErr = {
      message: 'duplicate key',
      code: '23505',
      details: 'Key (id)=(1) already exists.',
      hint: 'use ON CONFLICT',
    };
    const res = apiError(supabaseErr, 'reviews');
    const body = await readJson(res);
    expect(body.debug).toContain('duplicate key');
    expect(body.debug).toContain('code=23505');
    expect(body.debug).toContain('details=Key (id)=(1) already exists.');
    expect(body.debug).toContain('hint=use ON CONFLICT');
  });

  it('handles a plain string error', async () => {
    vi.stubEnv('NODE_ENV', 'development');
    const res = apiError('something failed');
    const body = await readJson(res);
    expect(body.debug).toBe('something failed');
  });

  it('logs the scope tag when provided', () => {
    apiError(new Error('boom'), 'availability');
    expect(loggerErrorMock).toHaveBeenCalledWith(
      expect.objectContaining({ scope: 'availability' }),
      'boom',
    );
  });

  it('logs a default scope when none is provided', () => {
    apiError(new Error('boom'));
    expect(loggerErrorMock).toHaveBeenCalledWith(expect.objectContaining({ scope: 'api' }), 'boom');
  });

  it('falls back to String(err) for a non-serializable primitive', async () => {
    vi.stubEnv('NODE_ENV', 'development');
    const res = apiError(42);
    const body = await readJson(res);
    expect(body.debug).toBe('42');
  });

  it('does NOT leak stack traces even in dev', async () => {
    vi.stubEnv('NODE_ENV', 'development');
    const err = new Error('boom');
    // `.stack` present but describeError uses only .message.
    err.stack = 'Error: boom\n    at secret/path/deep/inside/prod';
    const res = apiError(err);
    const body = await readJson(res);
    expect(body.debug).toBe('boom');
    expect(body.debug).not.toContain('secret/path');
  });
});
