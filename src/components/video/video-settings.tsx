"use client";

import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { cn } from "@/lib/utils";
import { PLAYBACK_RATES, SUBTITLE_SIZE_COUNT, useVideoPrefsStore } from "@/lib/video-prefs-store";

/**
 * Bảng cài đặt khi xem video — mở từ nút bánh răng trên thanh điều khiển và
 * ở đầu bảng transcript. Mỗi thay đổi áp dụng ngay và được nhớ cho mọi tập
 * (`video-prefs-store`).
 *
 * Cố ý ít lựa chọn: chỉ giữ những thứ người học thật sự đổi qua lại (phụ đề
 * nào, cỡ chữ, tốc độ, tự dừng, che bản dịch). Tô sáng từng chữ, tự cuộn
 * transcript… luôn bật — không ai cần tắt, thêm công tắc chỉ làm rối.
 *
 * `container`: đang toàn màn hình thì popover phải gắn vào chính khung
 * player, gắn vào `body` như mặc định là nằm ngoài vùng fullscreen, không
 * ai thấy.
 */
export function VideoSettingsPopover({
  children,
  container,
  hasVi,
  side = "top",
  align = "end",
  onOpenChange,
}: {
  children: React.ReactNode;
  container?: HTMLElement | null;
  hasVi: boolean;
  side?: "top" | "bottom";
  align?: "start" | "center" | "end";
  onOpenChange?: (open: boolean) => void;
}) {
  return (
    <Popover onOpenChange={onOpenChange}>
      <PopoverTrigger asChild>{children}</PopoverTrigger>
      <PopoverContent
        container={container}
        align={align}
        side={side}
        collisionPadding={8}
        className="flex max-h-(--radix-popover-content-available-height) w-72 max-w-[calc(100vw-16px)] flex-col overflow-y-auto p-0"
        onOpenAutoFocus={(e) => e.preventDefault()}
      >
        <VideoSettings hasVi={hasVi} />
      </PopoverContent>
    </Popover>
  );
}

type SubtitleMode = "both" | "ko" | "vi" | "off";

function VideoSettings({ hasVi }: { hasVi: boolean }) {
  const prefs = useVideoPrefsStore();
  const set = prefs.set;

  const subVi = prefs.subVi && hasVi;
  const subtitleMode: SubtitleMode =
    prefs.subKo && subVi ? "both" : prefs.subKo ? "ko" : subVi ? "vi" : "off";
  const setSubtitleMode = (mode: SubtitleMode) =>
    set({ subKo: mode === "both" || mode === "ko", subVi: mode === "both" || mode === "vi" });

  return (
    <>
      <div className="flex flex-col gap-3 p-3">
        <Field label="Phụ đề">
          <Segmented
            label="Phụ đề"
            value={subtitleMode}
            onChange={setSubtitleMode}
            options={[
              { value: "both", label: "Hàn + Việt", disabled: !hasVi },
              { value: "ko", label: "Hàn" },
              { value: "vi", label: "Việt", disabled: !hasVi },
              { value: "off", label: "Tắt" },
            ]}
          />
        </Field>

        <Field label="Cỡ chữ">
          <Segmented
            label="Cỡ chữ"
            value={prefs.subSize}
            onChange={(subSize) => set({ subSize })}
            options={Array.from({ length: SUBTITLE_SIZE_COUNT }, (_, i) => ({
              value: i,
              label: <span style={{ fontSize: 11 + i * 2.5 }}>가</span>,
              ariaLabel: ["Nhỏ", "Vừa", "Lớn", "Rất lớn"][i],
            }))}
          />
        </Field>

        <Field label="Tốc độ">
          <Segmented
            label="Tốc độ"
            value={prefs.playbackRate}
            onChange={(playbackRate) => set({ playbackRate })}
            options={PLAYBACK_RATES.map((r) => ({ value: r, label: r === 1 ? "Chuẩn" : `${r}` }))}
          />
        </Field>
      </div>

      <div className="flex flex-col border-t border-border/60 px-1.5 py-1.5">
        <SwitchRow label="Tự dừng sau mỗi câu" checked={prefs.autoPause} onChange={(autoPause) => set({ autoPause })} />
        <SwitchRow
          label="Che bản dịch, chạm để xem"
          checked={prefs.blurVi && hasVi}
          disabled={!hasVi}
          onChange={(blurVi) => set({ blurVi })}
        />
        <SwitchRow
          label="Transcript"
          checked={prefs.showTranscript}
          onChange={(showTranscript) => set({ showTranscript })}
        />
      </div>

      <p className="hidden border-t border-border/60 px-3 py-2 text-[11px] leading-relaxed text-muted-foreground lg:block">
        <Kbd>A</Kbd>/<Kbd>D</Kbd> câu trước/sau · <Kbd>R</Kbd> nghe lại · <Kbd>L</Kbd> lặp câu · <Kbd>V</Kbd> bản dịch
      </p>
    </>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-1.5">
      <span className="text-xs text-muted-foreground">{label}</span>
      {children}
    </div>
  );
}

function Segmented<T extends string | number>({
  label,
  value,
  onChange,
  options,
}: {
  label: string;
  value: T;
  onChange: (v: T) => void;
  options: { value: T; label: React.ReactNode; ariaLabel?: string; disabled?: boolean }[];
}) {
  return (
    <div role="radiogroup" aria-label={label} className="flex rounded-lg bg-muted p-0.5">
      {options.map((o) => {
        const selected = o.value === value;
        return (
          <button
            key={String(o.value)}
            type="button"
            role="radio"
            aria-checked={selected}
            aria-label={o.ariaLabel}
            disabled={o.disabled}
            onClick={() => onChange(o.value)}
            className={cn(
              "font-korean flex h-7 min-w-0 flex-1 items-center justify-center rounded-md px-1 text-xs whitespace-nowrap tabular-nums transition-all disabled:opacity-35",
              selected
                ? "bg-background font-medium text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground disabled:hover:text-muted-foreground"
            )}
          >
            {o.label}
          </button>
        );
      })}
    </div>
  );
}

function SwitchRow({
  label,
  checked,
  onChange,
  disabled,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className="flex items-center justify-between gap-3 rounded-md px-1.5 py-2 text-left text-sm hover:bg-muted/70 disabled:opacity-40 disabled:hover:bg-transparent"
    >
      {label}
      <span
        aria-hidden
        className={cn(
          "relative h-[18px] w-8 shrink-0 rounded-full transition-colors",
          checked ? "bg-primary" : "bg-muted-foreground/25"
        )}
      >
        <span
          className={cn(
            "absolute top-0.5 left-0.5 size-3.5 rounded-full bg-background shadow-sm transition-transform",
            checked && "translate-x-3.5"
          )}
        />
      </span>
    </button>
  );
}

function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd className="rounded border border-border bg-muted px-1 font-mono text-[10px] text-foreground">{children}</kbd>
  );
}
