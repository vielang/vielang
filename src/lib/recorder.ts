"use client";

/**
 * Bọc `MediaRecorder` cho gọn: mở micro, thu, rồi trả về một Blob.
 *
 * Tách khỏi component vì đây là chỗ mỗi trình duyệt một kiểu, và vì việc
 * NHẢ MICRO sau khi thu xong rất dễ quên — quên là đèn "đang ghi âm" của
 * trình duyệt sáng mãi, người dùng tưởng app nghe lén.
 */

/**
 * Kiểu tệp thu, xếp theo thứ tự ưu tiên.
 *
 * Opus trong WebM là thứ Chrome/Firefox/Edge cho ra, nhẹ và chất lượng tốt
 * cho giọng nói. Safari không nhận WebM mà chỉ có MP4/AAC — nên phải hỏi
 * `isTypeSupported` chứ không đoán, và phải NHỚ LẠI kiểu thật sự dùng để sau
 * này phát đúng.
 */
const PREFERRED_TYPES = [
  "audio/webm;codecs=opus",
  "audio/webm",
  "audio/mp4",
  "audio/ogg;codecs=opus",
];

export function pickMimeType(
  isSupported: (type: string) => boolean = (t) =>
    typeof MediaRecorder !== "undefined" && MediaRecorder.isTypeSupported(t)
): string | undefined {
  // `undefined` = để trình duyệt tự chọn. Vẫn tốt hơn ép một kiểu nó không
  // nhận, vì ép sai thì `new MediaRecorder` ném lỗi luôn.
  return PREFERRED_TYPES.find(isSupported);
}

/**
 * Ràng buộc lúc MỞ micro. Đây mới là chỗ quyết định chất lượng — xử lý sau
 * khi đã nén thì chỉ làm hỏng thêm, vì mọi lần nén lại một luồng đã nén là
 * một lần mất mát nữa.
 *
 * Giọng đọc là một người, một micro: thu 1 kênh là đủ, thu 2 kênh chỉ nhân
 * đôi dung lượng cho hai bản giống hệt nhau.
 *
 * KHỬ ỒN TẮT, khác mặc định của Chrome lẫn Safari. Bộ khử ồn nhận dạng
 * tiếng nói rồi gọt phần còn lại, mà các âm xát tiếng Hàn (ㅅ, ㅆ, ㅊ, ㅎ)
 * về mặt phổ âm thì giống tiếng ồn — nên chúng bị mài mất. Đúng mấy âm
 * người học cần nghe kỹ nhất khi so lại cách phát âm của mình, nên ở đây
 * thu trung thực quan trọng hơn thu sạch.
 *
 * Khử vọng và tự chỉnh âm lượng vẫn bật: chúng không đụng vào phổ âm theo
 * kiểu đó, mà lại giúp bản ghi bằng điện thoại nghe đều tiếng hơn.
 *
 * Dùng `ideal` để máy nào không làm được thì vẫn thu, không bật lỗi.
 */
const AUDIO_CONSTRAINTS: MediaTrackConstraints = {
  channelCount: { ideal: 1 },
  echoCancellation: { ideal: true },
  noiseSuppression: { ideal: false },
  autoGainControl: { ideal: true },
};

/**
 * Opus ở 32 kbps đã quá đủ cho một giọng nói — nó vốn được thiết kế cho
 * đúng việc đó. Không đặt thì Chrome dùng 128 kbps, tức tốn gấp 4 lần chỗ
 * mà tai không nghe ra khác biệt.
 */
const AUDIO_BITS_PER_SECOND = 32_000;

export type RecorderError = "denied" | "unsupported" | "failed";

/** Dịch lỗi của `getUserMedia` sang lý do mà UI nói lại được cho người dùng. */
export function toRecorderError(err: unknown): RecorderError {
  const name = (err as { name?: string } | null)?.name;
  if (name === "NotAllowedError" || name === "SecurityError") return "denied";
  if (name === "NotFoundError" || name === "NotSupportedError") return "unsupported";
  return "failed";
}

export const RECORDER_ERROR_MESSAGE: Record<RecorderError, string> = {
  denied: "Trình duyệt chưa cho phép dùng micro. Hãy bật quyền rồi thử lại.",
  unsupported: "Không tìm thấy micro nào trên thiết bị này.",
  failed: "Không bật được micro. Thử tải lại trang.",
};

export interface ActiveRecording {
  /** Dừng thu và trả về tiếng đã ghi. Gọi lần thứ hai thì trả về cùng kết quả. */
  stop: () => Promise<{ blob: Blob; mimeType: string }>;
  /** Bỏ hẳn, không lấy kết quả — vẫn nhả micro. */
  cancel: () => void;
}

/**
 * Bật micro và bắt đầu thu. Ném lỗi nếu người dùng từ chối quyền — bên gọi
 * bắt rồi dịch qua `toRecorderError`.
 */
export async function startRecording(): Promise<ActiveRecording> {
  const stream = await navigator.mediaDevices.getUserMedia({ audio: AUDIO_CONSTRAINTS });
  const mimeType = pickMimeType();
  const recorder = new MediaRecorder(stream, {
    ...(mimeType ? { mimeType } : {}),
    audioBitsPerSecond: AUDIO_BITS_PER_SECOND,
  });
  const chunks: Blob[] = [];
  recorder.addEventListener("dataavailable", (e) => {
    if (e.data.size > 0) chunks.push(e.data);
  });
  recorder.start();

  /** Nhả micro. Bỏ qua là đèn "đang ghi âm" của trình duyệt sáng mãi. */
  function release() {
    for (const track of stream.getTracks()) track.stop();
  }

  let result: Promise<{ blob: Blob; mimeType: string }> | null = null;

  return {
    stop() {
      // Nhớ lại lời hứa: bấm "dừng" hai lần (hoặc dừng rồi component unmount
      // gọi lại) thì vẫn ra cùng một bản ghi, không treo mãi ở lần thứ hai.
      result ??= new Promise((resolve) => {
        recorder.addEventListener(
          "stop",
          () => {
            release();
            // `recorder.mimeType` là kiểu THẬT trình duyệt dùng, có thể khác
            // cái mình xin — lấy nó để sau này phát lại cho đúng.
            const type = recorder.mimeType || mimeType || "audio/webm";
            resolve({ blob: new Blob(chunks, { type }), mimeType: type });
          },
          { once: true }
        );
        if (recorder.state !== "inactive") recorder.stop();
        else release();
      });
      return result;
    },
    cancel() {
      if (recorder.state !== "inactive") recorder.stop();
      release();
    },
  };
}
