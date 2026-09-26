import type { Metadata, Viewport } from "next";
import { Inter } from "next/font/google";
import { ThemeProvider } from "@/components/theme-provider";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ServiceWorker } from "@/components/service-worker";
import { InstallPromptCapture } from "@/components/install-prompt-capture";
import { AutoplayFollower } from "@/components/autoplay-follower";
import { Analytics } from "@vercel/analytics/next";
import { JsonLd } from "@/components/seo/json-ld";
import { SITE_DESCRIPTION, SITE_NAME, SITE_URL, absoluteUrl } from "@/lib/seo";
import "./globals.css";

const inter = Inter({
  variable: "--font-sans",
  subsets: ["latin", "vietnamese"],
  display: "swap",
});

export const metadata: Metadata = {
  // Gốc cho mọi URL tương đối trong metadata (canonical, og:image…).
  metadataBase: new URL(SITE_URL),
  title: {
    default: "VieLang – Học lập trình và tiếng Anh bằng tiếng Việt",
    template: "%s | VieLang",
  },
  description: SITE_DESCRIPTION,
  applicationName: SITE_NAME,
  // Trang không tự khai Open Graph (vd trang "không tìm thấy") vẫn có ảnh
  // xem trước khi share. Trang có nội dung thì khai lại đủ qua
  // `pageMetadata` — Next gộp nông, không trộn hai cục với nhau.
  openGraph: {
    type: "website",
    siteName: SITE_NAME,
    locale: "vi_VN",
    images: [{ url: "/og.png", width: 1200, height: 630 }],
  },
  twitter: { card: "summary_large_image", images: ["/og.png"] },
  // Mã xác minh Google Search Console / Bing Webmaster — điền env trên
  // Vercel, không cần sửa code.
  verification: {
    google: process.env.NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION,
    other: process.env.NEXT_PUBLIC_BING_SITE_VERIFICATION
      ? { "msvalidate.01": process.env.NEXT_PUBLIC_BING_SITE_VERIFICATION }
      : undefined,
  },
  // Apple bỏ qua manifest, chỉ đọc thẻ link này khi thêm vào màn hình chính.
  appleWebApp: { capable: true, title: "VieLang", statusBarStyle: "black-translucent" },
  icons: { apple: "/icons/apple-touch-icon.png" },
};

/**
 * Tên site + tổ chức cho Google: giúp kết quả tìm kiếm hiện "VieLang" thay
 * vì tên miền trần, và gắn logo cho site.
 */
const SITE_LD = [
  {
    "@context": "https://schema.org",
    "@type": "WebSite",
    name: SITE_NAME,
    alternateName: ["Vie Lang", "vielang"],
    url: absoluteUrl("/"),
    inLanguage: "vi",
    description: SITE_DESCRIPTION,
  },
  {
    "@context": "https://schema.org",
    "@type": "EducationalOrganization",
    name: SITE_NAME,
    url: absoluteUrl("/"),
    logo: absoluteUrl("/icons/icon-512.png"),
  },
];

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
      className={`${inter.variable} h-full antialiased`}
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
        <JsonLd data={SITE_LD} />
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
