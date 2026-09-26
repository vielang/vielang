"use client";

import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { cn } from "@/lib/utils";
import {
  PLAYBACK_RATES,
  SUBTITLE_SIZES,
  useVideoPrefsStore,
  type VideoPrefs,
} from "@/lib/video-prefs-store";

/**
 * Bảng cài đặt khi xem video — mở từ nút bánh răng trên thanh điều khiển và
 * ở đầu bảng transcript. Mỗi thay đổi áp dụng ngay và được nhớ cho mọi tập
 * (`video-prefs-store`).
 *
 * `container`: đang toàn màn hình thì popover phải gắn vào chính khung
 * player, gắn vào `body` như mặc định là nằm ngoài vùng fullscreen, không
 * ai thấy.
 */
export function VideoSettingsPopover({
  children,
  container,
  hasVi,
  align = "end",
  onOpenChange,
}: {
  children: React.ReactNode;
  container?: HTMLElement | null;
  hasVi: boolean;
  align?: "start" | "center" | "end";
  onOpenChange?: (open: boolean) => void;
}) {
  return (
    <Popover onOpenChange={onOpenChange}>
      <PopoverTrigger asChild>{children}</PopoverTrigger>
      <PopoverContent
        container={container}
        align={align}
        side="top"
        collisionPadding={8}
        className="flex max-h-(--radix-popover-content-available-height) w-80 max-w-[calc(100vw-16px)] flex-col gap-1 overflow-y-auto p-3"
        onOpenAutoFocus={(e) => e.preventDefault()}
      >
        <VideoSettings hasVi={hasVi} />
      </PopoverContent>
    </Popover>
  );
}

function VideoSettings({ hasVi }: { hasVi: boolean }) {
  const prefs = useVideoPrefsStore();
  const set = prefs.set;
  const bool = (key: keyof VideoPrefs) => ({
    checked: prefs[key] as boolean,
    onChange: (v: boolean) => set({ [key]: v }),
  });

  return (
    <>
      <Section title="Phụ đề trên video">
        <SwitchRow label="Tiếng Hàn" {...bool("subKo")} />
        <SwitchRow label="Tiếng Việt" disabled={!hasVi} hint={hasVi ? undefined : "Tập này chưa có"} {...bool("subVi")} />
        <SwitchRow label="Tô sáng từng chữ theo lời thoại" {...bool("karaoke")} />
        <ChoiceRow
          label="Cỡ chữ"
          options={SUBTITLE_SIZES.map((label, i) => ({ value: i, label }))}
          value={prefs.subSize}
          onChange={(subSize) => set({ subSize })}
        />
      </Section>

      <Section title="Luyện nghe">
        <ChoiceRow
          label="Tốc độ phát"
          options={PLAYBACK_RATES.map((r) => ({ value: r, label: r === 1 ? "1×" : `${r}` }))}
          value={prefs.playbackRate}
          onChange={(playbackRate) => set({ playbackRate })}
        />
        <SwitchRow label="Tự dừng sau mỗi câu" hint="Kịp đọc lại và nói theo" {...bool("autoPause")} />
        <SwitchRow
          label="Che tiếng Việt"
          hint="Tự nghe hiểu trước, chạm vào mới hiện"
          disabled={!hasVi}
          {...bool("blurVi")}
        />
      </Section>

      <Section title="Transcript">
        <SwitchRow label="Hiện bảng transcript" {...bool("showTranscript")} />
        <SwitchRow label="Dòng tiếng Việt dưới mỗi câu" disabled={!hasVi} {...bool("transcriptVi")} />
        <SwitchRow label="Tự cuộn theo câu đang phát" {...bool("autoScroll")} />
      </Section>

      <Section title="Phím tắt">
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs text-muted-foreground">
          <Shortcut keys="Space / K" label="Phát / tạm dừng" />
          <Shortcut keys="← / →" label="Lùi / tới 5 giây" />
          <Shortcut keys="A / D" label="Câu trước / câu sau" />
          <Shortcut keys="R" label="Nghe lại câu này" />
          <Shortcut keys="L" label="Lặp lại câu này" />
          <Shortcut keys="V" label="Bật/tắt phụ đề Việt" />
          <Shortcut keys="F" label="Toàn màn hình" />
        </dl>
      </Section>

      <button
        type="button"
        onClick={prefs.reset}
        className="mt-1 self-start rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-muted hover:text-foreground"
      >
        Khôi phục mặc định
      </button>
    </>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="flex flex-col gap-0.5 border-b border-border/60 pb-2 last-of-type:border-b-0">
      <h3 className="px-1 pt-1 pb-0.5 text-xs font-medium tracking-wide text-muted-foreground uppercase">{title}</h3>
      {children}
    </section>
  );
}

function SwitchRow({
  label,
  hint,
  checked,
  onChange,
  disabled,
}: {
  label: string;
  hint?: string;
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}) {
  const on = checked && !disabled;
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className="flex items-center justify-between gap-3 rounded-md px-1 py-1.5 text-left hover:bg-muted disabled:opacity-50 disabled:hover:bg-transparent"
    >
      <span className="flex min-w-0 flex-col">
        <span>{label}</span>
        {hint && <span className="text-xs text-muted-foreground">{hint}</span>}
      </span>
      <span
        aria-hidden
        className={cn(
          "relative h-5 w-9 shrink-0 rounded-full transition-colors",
          on ? "bg-primary" : "bg-muted-foreground/30"
        )}
      >
        <span
          className={cn(
            "absolute top-0.5 left-0.5 size-4 rounded-full bg-background shadow transition-transform",
            on && "translate-x-4"
          )}
        />
      </span>
    </button>
  );
}

function ChoiceRow<T extends number>({
  label,
  options,
  value,
  onChange,
}: {
  label: string;
  options: { value: T; label: string }[];
  value: T;
  onChange: (v: T) => void;
}) {
  return (
    <div className="flex flex-col gap-1.5 px-1 py-1.5">
      <span>{label}</span>
      <div role="radiogroup" aria-label={label} className="flex flex-wrap gap-1">
        {options.map((o) => (
          <button
            key={o.value}
            type="button"
            role="radio"
            aria-checked={o.value === value}
            onClick={() => onChange(o.value)}
            className={cn(
              "min-w-10 rounded-md border px-2 py-1 text-xs tabular-nums transition-colors",
              o.value === value
                ? "border-primary bg-primary text-primary-foreground"
                : "border-border hover:bg-muted"
            )}
          >
            {o.label}
          </button>
        ))}
      </div>
    </div>
  );
}

function Shortcut({ keys, label }: { keys: string; label: string }) {
  return (
    <>
      <dt className="font-mono text-foreground">{keys}</dt>
      <dd>{label}</dd>
    </>
  );
}
