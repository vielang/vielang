"use client";

import { useRef, useState } from "react";
import { Download, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  BackupError,
  backupFileName,
  buildBackup,
  restoreBackup,
} from "@/lib/learning-backup";

/**
 * Xuất / nhập file sao lưu dữ liệu học.
 *
 * Nhập là GHI ĐÈ dữ liệu đang có, nên phải hỏi lại trước — bấm nhầm một file
 * cũ là mất mấy tuần tiến độ. Sau khi ghi thì tải lại trang: các store đã
 * nạp dữ liệu cũ vào bộ nhớ lúc mở trang, không tải lại thì lần ghi kế tiếp
 * của chúng sẽ đè mất bản vừa nhập.
 */
export function BackupPanel() {
  const fileInput = useRef<HTMLInputElement>(null);
  const [pending, setPending] = useState<{ name: string; text: string } | null>(null);
  const [error, setError] = useState<string | null>(null);

  function exportFile() {
    const blob = new Blob([JSON.stringify(buildBackup(localStorage), null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = backupFileName();
    a.click();
    URL.revokeObjectURL(url);
  }

  async function pickFile(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    e.target.value = ""; // chọn lại đúng file đó lần nữa vẫn bắn change
    if (!file) return;
    setError(null);
    setPending({ name: file.name, text: await file.text() });
  }

  function confirmRestore() {
    if (!pending) return;
    try {
      restoreBackup(localStorage, pending.text);
      window.location.reload();
    } catch (err) {
      setPending(null);
      setError(err instanceof BackupError ? err.message : "Không nhập được file này.");
    }
  }

  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-muted-foreground">
        Dữ liệu học chỉ lưu trên trình duyệt này. Xoá dữ liệu duyệt web hay đổi máy là
        mất — hãy xuất file sao lưu định kỳ và nhập lại khi cần. File gồm tiến độ, bài
        làm, lịch sử học, ghi chú và tuỳ chọn; <strong>không</strong> gồm bản ghi âm và
        nét vẽ.
      </p>
      <div className="flex flex-wrap gap-2">
        <Button size="sm" variant="outline" onClick={exportFile}>
          <Download className="size-4" aria-hidden />
          Xuất file sao lưu
        </Button>
        <Button size="sm" variant="outline" onClick={() => fileInput.current?.click()}>
          <Upload className="size-4" aria-hidden />
          Nhập từ file
        </Button>
        <input
          ref={fileInput}
          type="file"
          accept="application/json,.json"
          className="hidden"
          onChange={pickFile}
          data-testid="backup-file-input"
        />
      </div>
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}

      <Dialog open={pending !== null} onOpenChange={(open) => !open && setPending(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Nhập dữ liệu học từ file?</DialogTitle>
            <DialogDescription>
              Dữ liệu trong file <span className="font-medium">{pending?.name}</span> sẽ
              thay thế tiến độ, bài làm, lịch sử học, ghi chú và tuỳ chọn đang có trên
              trình duyệt này. Không hoàn tác được.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setPending(null)}>
              Huỷ
            </Button>
            <Button onClick={confirmRestore}>Nhập và thay thế</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
