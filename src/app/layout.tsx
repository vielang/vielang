import type { Metadata, Viewport } from "next";
import { Inter, Noto_Sans_KR } from "next/font/google";
import { ThemeProvider } from "@/components/theme-provider";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ServiceWorker } from "@/components/service-worker";
import { InstallPromptCapture } from "@/components/install-prompt-capture";
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
        </ThemeProvider>
      </body>
    </html>
  );
}
