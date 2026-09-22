"use client";

import { cn } from "@/lib/utils";
import { heatLevel, heatmapWeeks, type DayStats } from "@/lib/activity";
import { HEAT_CLASSES } from "@/components/my/heat";

const WEEKDAY_LABELS = ["T2", "", "T4", "", "T6", "", "CN"];

function formatDay(key: string): string {
  const [y, m, d] = key.split("-");
  return `${d}/${m}/${y}`;
}

/**
 * Lịch học 12 tuần: mỗi ô một ngày, đậm theo số phút học thật.
 *
 * Rê chuột vào ô thấy ngày và số phút (`title`). Trình đọc màn hình không đi
 * qua 84 ô mà nhận một câu tóm tắt — đọc từng ô "0 phút" thì vô dụng.
 */
export function StudyHeatmap({
  days,
  today,
}: {
  days: Record<string, DayStats>;
  today: Date;
}) {
  const weeks = heatmapWeeks(days, today, 12);
  const cells = weeks.flat().filter((c) => !c.future);
  const activeDays = cells.filter((c) => c.minutes >= 1).length;
  const totalMinutes = cells.reduce((n, c) => n + c.minutes, 0);

  return (
    <div className="flex flex-col gap-2">
      <div
        role="img"
        aria-label={`Lịch học 12 tuần gần nhất: học ${activeDays} ngày, tổng ${totalMinutes} phút.`}
        className="flex gap-1 overflow-x-auto pb-1"
      >
        <div className="grid shrink-0 grid-rows-7 gap-1 pr-1" aria-hidden>
          {WEEKDAY_LABELS.map((label, i) => (
            <span
              key={i}
              className="flex h-4 items-center text-[10px] leading-none text-muted-foreground sm:h-5"
            >
              {label}
            </span>
          ))}
        </div>
        {weeks.map((week) => (
          <div key={week[0].key} className="grid shrink-0 grid-rows-7 gap-1" aria-hidden>
            {week.map((cell) =>
              cell.future ? (
                <span key={cell.key} className="size-4 sm:size-5" />
              ) : (
                <span
                  key={cell.key}
                  title={`${formatDay(cell.key)}: ${cell.minutes} phút`}
                  className={cn(
                    "size-4 rounded-[3px] sm:size-5",
                    HEAT_CLASSES[heatLevel(cell.minutes)]
                  )}
                />
              )
            )}
          </div>
        ))}
      </div>

      <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground" aria-hidden>
        <span>Ít</span>
        {HEAT_CLASSES.map((cls) => (
          <span key={cls} className={cn("size-3 rounded-[2px]", cls)} />
        ))}
        <span>Nhiều</span>
        <span className="ml-2">(theo số phút học mỗi ngày)</span>
      </div>
    </div>
  );
}
