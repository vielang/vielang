'use client';

import { useEffect, useRef, useState } from 'react';
import { Mic, MicOff } from 'lucide-react';
import { useLang } from '@/contexts';

const COPY = {
  VN: {
    checking: 'Đang kiểm tra micro…',
    ok: 'Micro đang hoạt động',
    silent: 'Chưa nghe thấy gì — thử nói vào micro',
    denied: 'Không có quyền truy cập micro',
    label: 'Mức âm thanh',
  },
  EN: {
    checking: 'Checking your microphone…',
    ok: 'Microphone is working',
    silent: 'No sound detected — try speaking into your mic',
    denied: 'Microphone access denied',
    label: 'Input level',
  },
} as const;

const BAR_COUNT = 7;
// If the user says nothing for this long we downgrade "ok" to "silent" so
// they know we can hear the mic hardware but not their voice.
const SILENT_THRESHOLD_MS = 3000;
// RMS below this is treated as room silence — anything under it never lights
// a bar. Chosen empirically on a quiet room + built-in laptop mic.
const NOISE_FLOOR = 0.02;

type Status = 'checking' | 'ok' | 'silent' | 'denied';

/**
 * Live mic level meter for the prejoin lobby.
 *
 * Runs its own getUserMedia({audio:true}) stream separate from LiveKit's
 * PreJoin prefab — LiveKit doesn't expose an internal stream we can tap
 * into, and modern browsers cheerfully hand out multiple parallel captures
 * of the same device. When the parent PreJoinPanel unmounts (phase →
 * connecting) the cleanup below tears the stream + AudioContext down so
 * LiveKit can acquire the mic without contention.
 *
 * The status downgrades to "silent" after 3s below the noise floor so a
 * user with a plugged-in-but-off mic sees a hint instead of a green "ok"
 * that lies.
 */
export function PreJoinAudioMeter() {
  const { lang } = useLang();
  const lc = COPY[lang];
  const [level, setLevel] = useState(0);
  const [status, setStatus] = useState<Status>('checking');
  const rafRef = useRef<number | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const ctxRef = useRef<AudioContext | null>(null);
  // Initialised inside the effect to keep render pure (react-hooks/purity).
  const lastLoudAtRef = useRef<number>(0);

  useEffect(() => {
    let cancelled = false;
    let analyser: AnalyserNode | null = null;
    // Uint8Array<ArrayBuffer> — TS 6 tightened getByteFrequencyData to reject
    // the SharedArrayBuffer-backed variant, so allocate a plain ArrayBuffer
    // explicitly instead of the default generic.
    let data: Uint8Array<ArrayBuffer> | null = null;

    async function setup() {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        if (cancelled) {
          stream.getTracks().forEach((t) => t.stop());
          return;
        }
        streamRef.current = stream;
        // Cast for older Safari (webkitAudioContext) — TS 6's DOM types no
        // longer include the prefix.
        const AudioCtx =
          window.AudioContext ||
          (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
        const ctx = new AudioCtx();
        // AudioContext may start suspended if the browser couldn't prove a
        // user gesture — resume once we have permission. Ignore rejection.
        void ctx.resume().catch(() => {});
        ctxRef.current = ctx;
        const source = ctx.createMediaStreamSource(stream);
        analyser = ctx.createAnalyser();
        analyser.fftSize = 256;
        analyser.smoothingTimeConstant = 0.6;
        source.connect(analyser);
        data = new Uint8Array(new ArrayBuffer(analyser.frequencyBinCount));
        setStatus('ok');
        lastLoudAtRef.current = Date.now();

        const tick = () => {
          if (!analyser || !data) return;
          analyser.getByteFrequencyData(data);
          let sum = 0;
          for (let i = 0; i < data.length; i++) sum += data[i] * data[i];
          const rms = Math.sqrt(sum / data.length) / 255;
          setLevel(rms);
          if (rms > NOISE_FLOOR) {
            lastLoudAtRef.current = Date.now();
            setStatus((s) => (s === 'denied' ? s : 'ok'));
          } else if (Date.now() - lastLoudAtRef.current > SILENT_THRESHOLD_MS) {
            setStatus((s) => (s === 'denied' || s === 'checking' ? s : 'silent'));
          }
          rafRef.current = requestAnimationFrame(tick);
        };
        rafRef.current = requestAnimationFrame(tick);
      } catch {
        // NotAllowedError, NotFoundError, or a browser without
        // getUserMedia. All present as "we can't check your mic".
        if (!cancelled) setStatus('denied');
      }
    }

    void setup();
    return () => {
      cancelled = true;
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      streamRef.current?.getTracks().forEach((t) => t.stop());
      void ctxRef.current?.close().catch(() => {});
    };
  }, []);

  // Map RMS 0..1 → number of lit bars. Slight power curve so a normal speaking
  // voice lights ~3–5 bars; the top two only ignite on loud speech.
  const lit = Math.min(BAR_COUNT, Math.round(Math.pow(level * 2.5, 0.7) * BAR_COUNT));

  const statusText =
    status === 'checking'
      ? lc.checking
      : status === 'ok'
        ? lc.ok
        : status === 'silent'
          ? lc.silent
          : lc.denied;

  return (
    <div className="rounded-lg border border-slate-200 bg-white p-3 shadow-sm dark:border-slate-800 dark:bg-slate-900">
      <div className="mb-2 flex items-center gap-2 text-xs">
        {status === 'denied' ? (
          <MicOff className="size-3.5 text-red-500" aria-hidden />
        ) : (
          <Mic
            className={`size-3.5 ${status === 'ok' ? 'text-emerald-500' : 'text-slate-400'}`}
            aria-hidden
          />
        )}
        <span className="text-slate-600 dark:text-slate-300">{statusText}</span>
      </div>
      <div
        role="meter"
        aria-label={lc.label}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(level * 100)}
        className="flex items-end gap-1"
      >
        {Array.from({ length: BAR_COUNT }, (_, i) => (
          <span
            key={i}
            className={`h-4 flex-1 rounded-sm transition-colors ${
              i < lit
                ? i >= BAR_COUNT - 2
                  ? 'bg-red-400'
                  : i >= BAR_COUNT - 4
                    ? 'bg-amber-400'
                    : 'bg-emerald-500'
                : 'bg-slate-200 dark:bg-slate-700'
            }`}
          />
        ))}
      </div>
    </div>
  );
}
