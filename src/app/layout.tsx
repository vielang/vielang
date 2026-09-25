import type { Metadata, Viewport } from "next";
import { Inter, Noto_Sans_KR } from "next/font/google";
import { ThemeProvider } from "@/components/theme-provider";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ServiceWorker } from "@/components/service-worker";
import { InstallPromptCapture } from "@/components/install-prompt-capture";
import { AutoplayFollower } from "@/components/autoplay-follower";
import { Analytics } from "@vercel/analytics/next";
import "./globals.css";

const inter = Inter({
  variable: "--font-sans",
  subsets: ["latin", "vietnamese"],
  display: "swap",
});

const notoSansKr = Noto_Sans_KR({
  variable: "--font-korean",
  subsets: ["latin"],
  weight: ["400", "500", "700"],
  display: "swap",
  preload: false,
});

export const metadata: Metadata = {
  title: {
    default: "VieTopik – Đọc sách tiếng Hàn",
    template: "%s | VieTopik",
  },
  description:
    "Đọc sách văn hóa – xã hội Hàn Quốc (chương trình KIIP) dành cho người Việt học tiếng Hàn.",
  // Apple bỏ qua manifest, chỉ đọc thẻ link này khi thêm vào màn hình chính.
  appleWebApp: { capable: true, title: "VieTopik", statusBarStyle: "black-translucent" },
  icons: { apple: "/icons/apple-touch-icon.png" },
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  // iOS: thiếu cái này thì `env(safe-area-inset-*)` luôn bằng 0, nên các
  // thanh dính đáy (thanh tab, thanh thao tác, phiếu trả lời) nằm lọt dưới
  // vạch Home / thanh công cụ Safari và bị che mất một phần. Android không
  // có vùng đó nên không lộ ra.
  viewportFit: "cover",
  // Không khoá pinch-zoom toàn cục (a11y) — trang đọc sách tự quản lý zoom
  // riêng cho ảnh qua touch-action, không cần chặn zoom trình duyệt ở đây.
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#ffffff" },
    { media: "(prefers-color-scheme: dark)", color: "#0a0a0a" },
  ],
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html
      lang="vi"
      suppressHydrationWarning
      className={`${inter.variable} ${notoSansKr.variable} h-full antialiased`}
    >
      <head>
        {/* Đánh dấu app đang chạy từ màn hình chính (PWA) TRƯỚC khi vẽ trang
            — xem `--standalone-safe-bottom` trong globals.css. Safari iOS
            không theo media query `display-mode`, chỉ có cờ riêng
            `navigator.standalone`, mà cờ đó CSS không đọc được. */}
        <script
          dangerouslySetInnerHTML={{
            __html: `if(navigator.standalone)document.documentElement.setAttribute("data-standalone","")`,
          }}
        />
      </head>
      <body className="min-h-full flex flex-col">
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange
        >
          <TooltipProvider delayDuration={200}>{children}</TooltipProvider>
          <ServiceWorker />
          <InstallPromptCapture />
          {/* Nghe tự động cả sách phải sống qua các lần lật trang — xem
              lib/autoplay-player.ts. */}
          <AutoplayFollower />
          {/* Vercel Analytics — số lượt xem trang, không cookie, không
              theo dấu người dùng qua các trang web khác.

              Đặt trong layout gốc để đếm được cả những lần lật trang trong
              app: lật trang sách là điều hướng phía client, không tải lại
              trang, nên nếu chỉ nhúng script một lần thì mọi trang sau đều
              không được tính.

              Service worker KHÔNG cần sửa gì cho chỗ này (đã kiểm `sw.js`):
              beacon là POST nên thoát ngay ở chốt `request.method !== "GET"`,
              còn script `/_vercel/insights/*` không khớp nhánh nào nên rơi
              hết, để trình duyệt tự lo. Mất mạng thì script hỏng lặng lẽ,
              phần đọc offline không ảnh hưởng. */}
          <Analytics />
        </ThemeProvider>
      </body>
    </html>
  );
}
