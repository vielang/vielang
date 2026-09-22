"use client";

import { Check } from "lucide-react";
import { cn } from "@/lib/utils";
import { addDays, dayKey, heatLevel, isStudyDay, weekStart, type DayStats } from "@/lib/activity";
import { HEAT_CLASSES } from "@/components/my/heat";

const LABELS = ["T2", "T3", "T4", "T5", "T6", "T7", "CN"];

/**
 * Bảy ngày của tuần này, mỗi ngày một vòng tròn: đậm theo số phút, có dấu ✓
 * khi đủ một ngày học, hôm nay có viền. Nhìn là biết tuần này mình đã đều
 * tay chưa — thứ người học quan tâm hơn là một con số tổng.
 */
export function WeekStrip({ days, today }: { days: Record<string, DayStats>; today: Date }) {
  const monday = weekStart(today);
  const todayKey = dayKey(today);
  const cells = LABELS.map((label, i) => {
    const key = dayKey(addDays(monday, i));
    const stats = days[key];
    return {
      label,
      key,
      minutes: Math.floor((stats?.activeMs ?? 0) / 60_000),
      studied: isStudyDay(stats),
      isToday: key === todayKey,
      future: key > todayKey,
    };
  });
  const studiedCount = cells.filter((c) => c.studied).length;

  return (
    <ol
      className="grid grid-cols-7 gap-1"
      aria-label={`Tuần này đã học ${studiedCount}/7 ngày`}
    >
      {cells.map((c) => (
        <li key={c.key} className="flex flex-col items-center gap-1.5">
          <span
            title={c.future ? undefined : `${c.label}: ${c.minutes} phút`}
            className={cn(
              "flex size-8 items-center justify-center rounded-full sm:size-9",
              c.future ? "border border-dashed border-border" : HEAT_CLASSES[heatLevel(c.minutes)],
              c.isToday && "ring-2 ring-primary ring-offset-2 ring-offset-background"
            )}
          >
            {c.studied && (
              <Check
                className={cn(
                  "size-4",
                  heatLevel(c.minutes) >= 3 ? "text-primary-foreground" : "text-foreground"
                )}
                aria-hidden
              />
            )}
          </span>
          <span
            className={cn(
              "text-[11px] leading-none",
              c.isToday ? "font-semibold text-foreground" : "text-muted-foreground"
            )}
          >
            {c.label}
          </span>
        </li>
      ))}
    </ol>
  );
}
