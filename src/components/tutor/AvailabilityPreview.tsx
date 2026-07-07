import { getAvailabilityForTutor } from '@/lib/supabase';
import { Clock } from 'lucide-react';

const WEEKDAYS_EN = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

// Trim seconds so "08:00:00" reads as "08:00" — Postgres TIME column returns
// the full precision but the UI has no need for it.
const trimSeconds = (t: string) => t.slice(0, 5);

// Server Component: fetches directly via the service_role helper. Kept small
// enough to inline into the detail page's grid without a client boundary.
export async function AvailabilityPreview({ tutorId }: { tutorId: string }) {
  const rows = await getAvailabilityForTutor(tutorId).catch(() => []);
  if (rows.length === 0) {
    return (
      <p className="text-sm text-slate-500 dark:text-slate-400">
        No recurring availability set yet.
      </p>
    );
  }
  // Group by weekday so each day shows all of its ranges on one line.
  const byDay: Record<number, string[]> = {};
  for (const r of rows) {
    if (!byDay[r.weekday]) byDay[r.weekday] = [];
    byDay[r.weekday].push(`${trimSeconds(r.start_time)}–${trimSeconds(r.end_time)}`);
  }

  return (
    <ul className="space-y-2">
      {WEEKDAYS_EN.map((label, i) => {
        const slots = byDay[i];
        if (!slots) return null;
        return (
          <li
            key={i}
            className="flex items-start gap-3 rounded-lg border border-slate-100 bg-slate-50 p-2.5 dark:border-slate-800 dark:bg-slate-800/40"
          >
            <span className="bg-brand mt-0.5 inline-flex size-6 items-center justify-center rounded-md text-[10px] font-semibold text-white uppercase">
              {label.slice(0, 3)}
            </span>
            <div className="flex min-w-0 flex-1 flex-wrap gap-1.5">
              {slots.map((s) => (
                <span
                  key={s}
                  className="inline-flex h-6 items-center gap-1 rounded-md border border-slate-200 bg-white px-2 text-[11px] font-medium text-slate-700 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200"
                >
                  <Clock className="size-3 text-slate-400" />
                  {s}
                </span>
              ))}
            </div>
          </li>
        );
      })}
    </ul>
  );
}
