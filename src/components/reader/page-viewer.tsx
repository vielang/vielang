"use client";

import Image from "next/image";
import { ImageOff, RotateCw } from "lucide-react";
import {
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { Button } from "@/components/ui/button";
import {
  useRetryingMedia,
  type MediaLoadStatus,
} from "@/lib/use-retrying-media";
import {
  TransformWrapper,
  TransformComponent,
  type ReactZoomPanPinchContentRef,
} from "react-zoom-pan-pinch";
import { getPageUrl, getPageAspectRatio, type Book } from "@/lib/books";
import { getPageTranslations } from "@/lib/page-translation";
import { getPageGrammar } from "@/lib/page-grammar";
import { getPageAnswers } from "@/lib/page-answers";
import { answersHidden, useReaderPrefsStore } from "@/lib/reader-prefs-store";
import { TranslationOverlay } from "@/components/reader/translation-overlay";
import { GrammarOverlay } from "@/components/reader/grammar-overlay";
import { AnswerOverlay } from "@/components/reader/answer-overlay";
import { AnnotationLayer } from "@/components/reader/annotation-layer";
import { useAnnotationStore } from "@/lib/annotation-store";
import {
  DOUBLE_TAP_STEP_NARROW,
  DOUBLE_TAP_STEP_WIDE,
  MAX_ZOOM,
  MIN_ZOOM,
  useZoomStore,
} from "@/lib/zoom-store";

export interface PageViewerHandle {
  resetZoom: () => void;
  /** Phóng tới đúng mức `scale` (1–4), giữ nguyên tâm vùng đang xem. */
  zoomTo: (scale: number) => void;
  /**
   * Lật sang spread liền kề kèm hiệu ứng trượt.
   *
   * Trả về `false` khi không lật được (hết sách, hoặc khung chưa đo xong)
   * để bên gọi còn tự chuyển trang theo cách cũ thay vì đứng im.
   */
  turn: (direction: 1 | -1) => boolean;
}

/** Khoảng hở giữa 2 trang ở chế độ xem 2 trang, mô phỏng gáy sách. */
const SPREAD_GAP = 4;

/**
 * Thời gian trang trượt hẳn sang bên, tính bằng ms.
 *
 * Quanh 250ms là khoảng các trình đọc lớn đều rơi vào: đủ chậm để mắt
 * theo kịp hướng lật, đủ nhanh để lật liền vài trang không thấy phải chờ.
 */
const TURN_MS = 260;

/** Nhanh nhất / chậm nhất cho cú trượt nốt sau khi thả tay, ms. */
const TURN_MIN_MS = 130;
const TURN_MAX_MS = 280;

/**
 * Tốc độ tối thiểu dùng để tính (px mỗi ms). Kéo chậm rồi thả khi trang gần
 * sang hẳn mà vẫn lấy đúng tốc độ tay thì quãng cuối ngắn tí lại bò mất
 * gần 300ms — nhìn như bị treo.
 */
const TURN_MIN_SPEED = 0.8;

/**
 * Trượt nốt quãng còn lại với TỐC ĐỘ NGÓN TAY vừa rời ra, thay vì luôn 260ms.
 *
 * Vẩy nhanh mà vẫn chờ đủ 260ms thì thấy ì; kéo chậm gần hết màn rồi thả mà
 * cũng 260ms thì trang giật lên một đoạn ngắn. Lấy quãng còn lại chia tốc độ
 * là nối tiếp đúng đà tay đang có.
 */
function turnDuration(remaining: number, speed: number): number {
  const v = Math.max(Math.abs(speed), TURN_MIN_SPEED);
  return Math.min(TURN_MAX_MS, Math.max(TURN_MIN_MS, remaining / v));
}

/**
 * Chạm một cái thì chờ chừng này mới bật/tắt thanh công cụ, để phân biệt với
 * bấm đúp (phóng to).
 *
 * Trước đây bật/tắt ngay ở mỗi lần nhấc tay, nên bấm đúp là thanh công cụ
 * nháy hai lần — và nếu thư viện phóng to nuốt mất lần thứ hai thì dừng ở
 * trạng thái ẩn, kéo theo thanh phóng to biến mất đúng lúc vừa phóng to.
 * 250ms: đủ cho một cú bấm đúp bình thường, mà chạm đơn vẫn không thấy trễ.
 */
const TAP_DELAY_MS = 250;

/** Kéo quá bấy nhiêu phần bề ngang màn hình thì thả tay ra là lật. */
const COMMIT_RATIO = 0.22;

/**
 * Vẩy nhanh hơn ngần này (px mỗi ms) thì lật luôn, không cần đủ quãng.
 *
 * Thiếu nhánh này thì một cái vẩy nhanh và ngắn sẽ bật ngược lại, và cảm
 * giác là ứng dụng không nghe lời.
 */
const FLICK_SPEED = 0.5;

/**
 * Lực cản khi kéo về phía không còn trang nào.
 *
 * Vẫn cho nhúc nhích một chút chứ không cứng đờ: nhúc nhích là câu trả
 * lời rằng đã nghe thấy nhưng hết sách rồi, còn cứng đờ thì không phân
 * biệt được với treo máy.
 */
const EDGE_RESISTANCE = 0.32;

interface PageViewerProps {
  book: Book;
  /** 1 trang (chế độ 1 trang) hoặc 2 trang [trái, phải] (chế độ 2 trang). */
  pages: number[];
  /** Spread liền trước, `null` khi đang ở đầu sách. Chỉ để vẽ lúc lật. */
  prevPages: number[] | null;
  /** Spread liền sau, `null` khi đang ở cuối sách. */
  nextPages: number[] | null;
  onTap: () => void;
  onSwipePrev: () => void;
  onSwipeNext: () => void;
  /**
   * Màn rộng (máy tính) hay hẹp (điện thoại) — quyết định bấm đúp phóng lên
   * bao nhiêu (xem `DOUBLE_TAP_STEP_*`). `ReaderView` vốn đã đo sẵn.
   */
  wide?: boolean;
}

/**
 * Lớp phủ lúc ảnh chưa hiện: đang tải thì là nền giấy mờ, hỏng hẳn thì là
 * lời nhắn kèm nút thử lại.
 *
 * Nằm DƯỚI ảnh trong cây DOM nhưng cùng vị trí, nên ảnh hiện ra là tự che
 * đi — không cần gỡ ra theo nhịp nào cả.
 *
 * Nền sáng chứ không phải nền tối: khung đọc nền đen, mà trang sách thì
 * luôn là giấy trắng. Để ô chờ màu đen thì lúc ảnh hiện ra sẽ chớp một cái
 * rất chói.
 */
function PageImageFallback({
  page,
  status,
  retrying,
  onRetry,
}: {
  page: number;
  status: MediaLoadStatus;
  retrying: boolean;
  onRetry: () => void;
}) {
  if (status !== "failed") {
    return (
      <div
        className="absolute inset-0 animate-pulse bg-neutral-200"
        aria-hidden
      />
    );
  }

  return (
    <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-neutral-200 px-6 text-center">
      <ImageOff className="size-8 text-neutral-500" aria-hidden />
      <p className="text-sm text-neutral-700">
        Không tải được trang {page}.
        <br />
        Kiểm tra lại mạng giúp mình nhé.
      </p>
      <Button size="sm" variant="secondary" onClick={onRetry} disabled={retrying}>
        <RotateCw className={retrying ? "size-4 animate-spin" : "size-4"} aria-hidden />
        Tải lại
      </Button>
    </div>
  );
}

/** 1 ảnh trang + vùng dịch + nét vẽ tay của nó, trong khung đã tính đúng cỡ. */
function PageImage({
  bookId,
  page,
  box,
  aspectRatio,
}: {
  bookId: string;
  page: number;
  box: { width: number; height: number };
  aspectRatio: number;
}) {
  const regions = getPageTranslations(bookId, page);
  const grammar = getPageGrammar(bookId, page);
  // Người dùng tắt chấm đáp án của cuốn này ở trang chi tiết sách (muốn tự
  // làm bài trước rồi mới dò) — xem `AnswersToggle`.
  const hideAnswers = useReaderPrefsStore((s) => answersHidden(s, bookId));
  const answerKeys = hideAnswers ? [] : getPageAnswers(bookId, page);
  const { attempt, status, retrying, onLoad, onError, retry } = useRetryingMedia();

  return (
    <div className="relative" style={{ width: box.width, height: box.height }}>
      {status !== "ok" && (
        <PageImageFallback page={page} status={status} retrying={retrying} onRetry={retry} />
      )}
      <Image
        // `key` đổi theo số lần thử: phần tử <img> đang ở trạng thái lỗi có
        // thể bị dùng lại mà không xin lại gì cả, dựng phần tử mới mới chắc
        // chắn có lượt xin mới. URL thì giữ nguyên — xem chú thích trong
        // `use-retrying-media.ts` về lý do không phá cache bằng query.
        key={attempt}
        src={getPageUrl(bookId, page)}
        alt={`Trang ${page}`}
        fill
        sizes="100vw"
        // Ảnh nguồn đã là webp tối ưu sẵn (~130KB) và đi qua rewrite cùng
        // origin, không qua bộ tối ưu của Vercel — xem `next.config.ts`.
        unoptimized
        priority
        draggable={false}
        className="object-contain"
        onLoad={onLoad}
        onError={onError}
      />
      <TranslationOverlay regions={regions} />
      <GrammarOverlay points={grammar} />
      <AnswerOverlay answerKeys={answerKeys} bookId={bookId} page={page} />
      {/* Nằm SAU vùng dịch: đang bật chế độ vẽ thì nét vẽ phải nhận được
          chạm trước, nếu không bấm trúng vùng dịch là bật bản dịch thay vì
          vẽ. Tắt chế độ vẽ thì lớp này `pointer-events: none` nên vùng dịch
          bên dưới lại nhận chạm như cũ. */}
      <AnnotationLayer
        bookId={bookId}
        page={page}
        aspectRatio={aspectRatio}
        boxWidth={box.width}
      />
    </div>
  );
}


/**
 * Spread liền kề, chỉ để nhìn thấy trong lúc lật.
 *
 * Cố tình KHÔNG có vùng dịch, chấm ngữ pháp hay lớp vẽ: mấy thứ đó chỉ có
 * nghĩa với trang đang đọc, và gắn thêm vào đây là nhân đôi số lớp phủ phải
 * dựng cho một thứ hiện ra chưa tới nửa giây.
 *
 * Ảnh đã nằm sẵn trong cache nhờ `AdjacentPreload`, nên dựng thêm ở đây
 * không tốn thêm lượt tải nào.
 */
function NeighbourSpread({
  bookId,
  pages,
  box,
}: {
  bookId: string;
  pages: number[];
  box: { width: number; height: number };
}) {
  return (
    <div
      className="flex h-full w-full items-center justify-center"
      style={pages.length > 1 ? { gap: SPREAD_GAP } : undefined}
      aria-hidden
    >
      {pages.map((p) => (
        <div
          key={p}
          className="relative bg-neutral-200"
          style={{ width: box.width, height: box.height }}
        >
          <Image
            src={getPageUrl(bookId, p)}
            alt=""
            fill
            sizes="100vw"
            unoptimized
            draggable={false}
            className="object-contain"
          />
        </div>
      ))}
    </div>
  );
}

/** Người dùng đã bảo hệ điều hành là bớt hiệu ứng thì đừng trượt. */
function prefersReducedMotion(): boolean {
  return (
    typeof window !== "undefined" &&
    window.matchMedia?.("(prefers-reduced-motion: reduce)").matches === true
  );
}

export const PageViewer = forwardRef<PageViewerHandle, PageViewerProps>(function PageViewer(
  { book, pages, prevPages, nextPages, onTap, onSwipePrev, onSwipeNext, wide = false },
  ref
) {
  const transformRef = useRef<ReactZoomPanPinchContentRef>(null);
  const scaleRef = useRef(1);
  const tapTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(() => () => {
    if (tapTimer.current) clearTimeout(tapTimer.current);
  }, []);
  const setZoomScale = useZoomStore((s) => s.setScale);
  // Chỉ đổi khi vượt qua ngưỡng "đang phóng" — không phải mỗi khung hình.
  const [zoomed, setZoomed] = useState(false);
  // Lật trang là dựng lại khung phóng từ 100% (xem `key` của TransformWrapper)
  // mà không bắn `onTransform` nào — tự đưa con số trên thanh phóng to về
  // 100%. `zoomed` thì không cần: cả trang đọc được dựng lại khi đổi trang.
  const pagesKey = pages.join("-");
  useEffect(() => {
    setZoomScale(1);
  }, [pagesKey, setZoomScale]);
  const aspectRatio = getPageAspectRatio(book);
  // Đang vẽ lên trang: một ngón/chuột thuộc về cây bút, không còn là chạm để
  // ẩn thanh công cụ, vuốt lật trang hay kéo di chuyển trang nữa.
  const drawing = useAnnotationStore((s) => s.active);

  const boxRef = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState<{ width: number; height: number } | null>(null);
  const count = pages.length;

  /**
   * Bề ngang khung, để quy quãng kéo ra tỉ lệ màn hình.
   *
   * Giữ trong ref chứ không trong state: nó được đọc giữa lúc kéo, và kéo
   * thì không được phép làm dựng lại cây component.
   */
  const widthRef = useRef(0);

  useLayoutEffect(() => {
    const el = boxRef.current;
    if (!el) return;

    function measure(target: HTMLElement) {
      widthRef.current = target.offsetWidth;
      const gapTotal = SPREAD_GAP * (count - 1);
      const maxWidthPerPage = (target.offsetWidth - gapTotal) / count;
      const width = Math.min(maxWidthPerPage, target.offsetHeight * aspectRatio);
      setBox({ width, height: width / aspectRatio });
    }

    measure(el);
    const observer = new ResizeObserver(() => measure(el));
    observer.observe(el);
    return () => observer.disconnect();
  }, [count, aspectRatio]);

  /**
   * Dải ba trang được đẩy bằng cách ghi thẳng vào `style`, không qua state.
   *
   * Mỗi lần ngón tay nhích là một lần đổi vị trí; để state lo thì mỗi nhích
   * lại dựng lại cả TransformWrapper và mấy lớp phủ, và cái kéo sẽ giật.
   */
  const trackRef = useRef<HTMLDivElement>(null);
  const place = useCallback((x: number, animate: boolean | number) => {
    const el = trackRef.current;
    if (!el) return;
    const ms = animate === true ? TURN_MS : animate === false ? 0 : animate;
    el.style.transition = ms ? `transform ${ms}ms cubic-bezier(0.22, 0.61, 0.36, 1)` : "none";
    el.style.transform = `translate3d(${x}px, 0, 0)`;
  }, []);

  // Đã chốt lật rồi thì mọi cử chỉ sau đó bị bỏ qua cho tới khi route đổi và
  // component dựng lại — nếu không, vẩy hai cái liên tiếp sẽ bắn ra hai lượt
  // chuyển trang mà chỉ trang thứ nhất kịp có hiệu ứng.
  const committed = useRef(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    []
  );

  const commit = useCallback(
    (direction: 1 | -1, gesture?: { moved: number; speed: number }) => {
      if (committed.current) return false;
      if ((direction === 1 ? nextPages : prevPages) === null) return false;
      const width = widthRef.current;
      if (width <= 0) return false;

      committed.current = true;
      const go = () => (direction === 1 ? onSwipeNext() : onSwipePrev());

      if (prefersReducedMotion()) {
        go();
        return true;
      }
      const remaining = Math.max(0, width - Math.abs(gesture?.moved ?? 0));
      const ms = gesture ? turnDuration(remaining, gesture.speed) : TURN_MS;
      place(direction === 1 ? -width : width, ms);
      // Đổi route ngay khi trang vừa trượt xong — bắt `transitionend` chứ
      // không hẹn giờ song song, để không còn quãng chờ thừa ở cuối (lúc đó
      // trang đích đã nằm đúng giữa màn hình, dựng lại chỉ là thay nội dung
      // y hệt chỗ cũ). Hẹn giờ giữ làm phương án dự phòng nếu trình duyệt
      // nuốt mất sự kiện (vd tab bị ẩn, hay hiệu ứng bị bỏ qua).
      const el = trackRef.current;
      let done = false;
      const finish = () => {
        if (done) return;
        done = true;
        el?.removeEventListener("transitionend", finish);
        go();
      };
      el?.addEventListener("transitionend", finish, { once: true });
      timer.current = setTimeout(finish, ms);
      return true;
    },
    [nextPages, prevPages, onSwipeNext, onSwipePrev, place]
  );

  useImperativeHandle(
    ref,
    () => ({
      resetZoom: () => transformRef.current?.resetTransform(),
      zoomTo: (target) => {
        const t = transformRef.current;
        if (!t) return;
        const clamped = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, target));
        const delta = clamped - t.instance.state.scale;
        if (Math.abs(delta) < 0.001) return;
        // zoomIn/zoomOut phóng quanh TÂM khung nhìn và tự giữ trong mép ảnh;
        // `step` ở bản thư viện này là số CỘNG thêm, nên đưa đúng hiệu số.
        if (clamped <= MIN_ZOOM) t.resetTransform(150);
        else if (delta > 0) t.zoomIn(delta, 150);
        else t.zoomOut(-delta, 150);
      },
      turn: commit,
    }),
    [commit]
  );

  /**
   * Cử chỉ một ngón. Ba ngả: chạm (bật/tắt thanh công cụ), kéo ngang (lật
   * trang), hoặc bỏ qua (đang zoom, đang vẽ, hai ngón, hay kéo dọc).
   *
   * Quyết hướng MỘT LẦN rồi giữ nguyên: đổi ngả giữa chừng thì trang vừa
   * trôi theo tay vừa giật về, và không cử chỉ nào ra hồn cả.
   */
  /**
   * Đang phóng to mà đã kéo tới sát mép trái/phải của ảnh chưa? Tới mép rồi
   * thì kéo tiếp cũng không di chuyển được nữa — lúc đó cử chỉ ấy chuyển
   * thành lật trang, giống lật sang trang kế khi đọc báo giấy đã phóng to.
   */
  const atPanEdge = useCallback((direction: 1 | -1) => {
    const instance = transformRef.current?.instance;
    const bounds = instance?.bounds;
    if (!instance || !bounds) return false;
    const x = instance.state.positionX;
    // direction 1 = kéo sang TRÁI (xem trang sau) → đang ở mép PHẢI của ảnh.
    return direction === 1 ? x <= bounds.minPositionX + 1 : x >= bounds.maxPositionX - 1;
  }, []);

  const drag = useRef<{
    id: number;
    startX: number;
    startY: number;
    /**
     * Mốc để tính quãng kéo: đặt lúc QUYẾT là kéo ngang, không phải lúc
     * đặt ngón xuống.
     *
     * Mười px đầu là vùng chết để phân biệt kéo với chạm. Tính quãng từ
     * lúc đặt ngón thì đúng lúc quyết xong trang sẽ nhảy cái rụp đúng
     * mười px đó.
     */
    originX: number;
    lastX: number;
    lastT: number;
    speed: number;
    axis: "undecided" | "x" | "off";
    /**
     * Cử chỉ này bị chuyển sang "off" vì đang phóng to (kéo = di chuyển vùng
     * xem). Vẫn theo dõi tiếp: kéo tới lúc chạm mép ảnh thì đổi sang lật
     * trang NGAY TRONG cùng cú vuốt — không thì người dùng phải nhấc tay
     * vuốt thêm lần nữa, cảm giác như bị khựng.
     */
    panning: boolean;
    /** Toạ độ x lúc vùng xem vừa chạm mép ảnh. */
    edgeX: number | null;
    onControl: boolean;
  } | null>(null);
  const pointers = useRef(new Set<number>());

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      if (drawing) return;
      pointers.current.add(e.pointerId);
      if (pointers.current.size > 1) {
        // Ngón thứ hai là pinch — trả lại cho thư viện zoom.
        drag.current = null;
        return;
      }
      const el = e.target as HTMLElement;
      drag.current = {
        id: e.pointerId,
        startX: e.clientX,
        startY: e.clientY,
        originX: e.clientX,
        lastX: e.clientX,
        lastT: performance.now(),
        speed: 0,
        axis: "undecided",
        panning: false,
        edgeX: null,
        // Chạm trúng chấm dịch, chấm ngữ pháp hay chấm đáp án thì để nút đó
        // tự xử: tính thêm là tap nữa thì vừa mở bong bóng vừa ẩn thanh công cụ.
        onControl: Boolean(
          el.closest("[data-translate-region]") ||
            el.closest("[data-grammar-point]") ||
            el.closest("[data-answer-key]")
        ),
      };
    },
    [drawing]
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent) => {
      const g = drag.current;
      if (!g || g.id !== e.pointerId || committed.current) return;

      const dx = e.clientX - g.startX;
      const dy = e.clientY - g.startY;

      // Đo bằng `performance.now()` chứ không bằng `e.timeStamp`: gốc
      // thời gian của `timeStamp` mỗi trình duyệt một kiểu, và trong
      // jsdom thì nó đứng yên ở 0.
      const now = performance.now();
      const dt = now - g.lastT;
      if (dt > 0) {
        g.speed = (e.clientX - g.lastX) / dt;
        g.lastX = e.clientX;
        g.lastT = now;
      }

      if (g.axis === "undecided") {
        if (Math.abs(dx) > 10 && Math.abs(dx) > Math.abs(dy) * 1.2) {
          // Đã phóng to thì kéo ngang là di chuyển vùng xem — trừ khi vùng
          // xem đã chạm mép ảnh theo đúng hướng đang kéo, lúc đó lật trang.
          if (scaleRef.current > 1.02 && !atPanEdge(dx < 0 ? 1 : -1)) {
            g.axis = "off";
            g.panning = true;
            return;
          }
          g.axis = "x";
          g.originX = e.clientX;
          e.currentTarget.setPointerCapture?.(e.pointerId);
        } else if (Math.abs(dy) > 10) {
          g.axis = "off";
        }
        return;
      }
      // Đang di chuyển vùng xem: tới mép ảnh rồi mà tay vẫn kéo tiếp theo
      // hướng đó thì chuyển thành lật trang, tính quãng từ đúng lúc chạm mép.
      if (g.axis === "off" && g.panning) {
        const dir: 1 | -1 = dx < 0 ? 1 : -1;
        if (!atPanEdge(dir)) {
          g.edgeX = null;
          return;
        }
        if (g.edgeX === null) {
          g.edgeX = e.clientX;
          return;
        }
        const past = e.clientX - g.edgeX;
        if (Math.abs(past) < 12 || Math.sign(past) !== (dir === 1 ? -1 : 1)) return;
        g.axis = "x";
        g.panning = false;
        g.originX = g.edgeX;
        e.currentTarget.setPointerCapture?.(e.pointerId);
      }
      if (g.axis !== "x") return;

      const moved = e.clientX - g.originX;
      const blocked = (moved < 0 && !nextPages) || (moved > 0 && !prevPages);
      place(blocked ? moved * EDGE_RESISTANCE : moved, false);
    },
    [nextPages, prevPages, place, atPanEdge]
  );

  const onPointerUp = useCallback(
    (e: React.PointerEvent) => {
      const g = drag.current;
      pointers.current.delete(e.pointerId);
      if (!g || g.id !== e.pointerId) return;
      drag.current = null;
      if (committed.current) return;

      const dx = e.clientX - g.startX;
      const dy = e.clientY - g.startY;

      if (g.axis === "x") {
        const moved = e.clientX - g.originX;
        const direction: 1 | -1 = moved < 0 ? 1 : -1;
        const width = widthRef.current || 1;
        const farEnough = Math.abs(moved) > width * COMMIT_RATIO;
        // Vẩy chỉ tính khi còn đang đi đúng hướng đã kéo — kéo sang trái rồi
        // hất ngược lại là người ta đổi ý, phải trả trang về chỗ cũ.
        const flicked =
          Math.abs(g.speed) > FLICK_SPEED && Math.sign(g.speed) === Math.sign(moved);
        if ((farEnough || flicked) && commit(direction, { moved, speed: g.speed })) return;
        place(0, true);
        return;
      }
      if (g.axis === "off") return;

      if (Math.abs(dx) < 10 && Math.abs(dy) < 10 && !g.onControl) {
        // Chạm lần hai trong lúc còn chờ = bấm đúp để phóng to — huỷ lần
        // bật/tắt thanh công cụ đang chờ, không bật/tắt gì cả.
        if (tapTimer.current) {
          clearTimeout(tapTimer.current);
          tapTimer.current = null;
          return;
        }
        tapTimer.current = setTimeout(() => {
          tapTimer.current = null;
          onTap();
        }, TAP_DELAY_MS);
      }
    },
    [commit, onTap, place]
  );

  const onPointerCancel = useCallback(
    (e: React.PointerEvent) => {
      const g = drag.current;
      pointers.current.delete(e.pointerId);
      if (!g || g.id !== e.pointerId) return;
      drag.current = null;
      if (g.axis === "x" && !committed.current) place(0, true);
    },
    [place]
  );

  return (
    <div
      ref={boxRef}
      className="relative h-full w-full touch-none overflow-hidden select-none"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerCancel}
    >
      <div
        ref={trackRef}
        data-page-track
        className="absolute inset-0"
        style={{ transform: "translate3d(0px, 0, 0)" }}
      >
        {box && prevPages && (
          <div className="absolute inset-0 -translate-x-full">
            <NeighbourSpread bookId={book.id} pages={prevPages} box={box} />
          </div>
        )}

        <div className="absolute inset-0">
          <TransformWrapper
            ref={transformRef}
            key={pages.join("-")}
            initialScale={1}
            minScale={MIN_ZOOM}
            maxScale={MAX_ZOOM}
            limitToBounds
            centerOnInit
            // Chế độ vẽ: khoá kéo-thả và bấm đúp (chúng nuốt mất nét bút),
            // nhưng CỐ Ý để nguyên pinch 2 ngón và cuộn trackpad — phóng to
            // rồi khoanh chú thích vào chữ nhỏ là chuyện thường xuyên nhất.
            panning={{ disabled: drawing }}
            // Bấm đúp: đang ở 100% thì phóng lên (máy tính 150%, điện thoại
            // 250% — xem `DOUBLE_TAP_STEP_*`); đang phóng ở MỨC NÀO thì cũng về
            // 100%. Chế độ "toggle" của thư viện chỉ trừ đi đúng `step`, nên
            // chụm tay lên 350% rồi bấm đúp chỉ về 300% — không phải thứ người
            // ta chờ đợi khi bấm đúp để "thoát phóng to".
            doubleClick={{
              mode: zoomed ? "reset" : "zoomIn",
              step: wide ? DOUBLE_TAP_STEP_WIDE : DOUBLE_TAP_STEP_NARROW,
              disabled: drawing,
            }}
            // Trackpad 2 ngón vuốt (không giữ Ctrl) = di chuyển vùng xem khi
            // đã phóng to, giống các trình đọc ảnh/PDF thông thường —
            // Ctrl+vuốt (pinch thật) vẫn zoom như cũ.
            wheel={{ wheelDisabled: true }}
            trackPadPanning={{ disabled: false }}
            onTransform={(_ref, state) => {
              scaleRef.current = state.scale;
              setZoomScale(state.scale);
              const nowZoomed = state.scale > 1.02;
              setZoomed((was) => (was === nowZoomed ? was : nowZoomed));
            }}
          >
            <TransformComponent
              wrapperClass="!h-full !w-full"
              contentClass="!h-full !w-full"
            >
              <div
                className="flex h-full w-full items-center justify-center"
                style={count > 1 ? { gap: SPREAD_GAP } : undefined}
              >
                {box &&
                  pages.map((p) => (
                    <PageImage
                      key={p}
                      bookId={book.id}
                      page={p}
                      box={box}
                      aspectRatio={aspectRatio}
                    />
                  ))}
              </div>
            </TransformComponent>
          </TransformWrapper>
        </div>

        {box && nextPages && (
          <div className="absolute inset-0 translate-x-full">
            <NeighbourSpread bookId={book.id} pages={nextPages} box={box} />
          </div>
        )}
      </div>
    </div>
  );
});
