import Image from "next/image"
import Link from "next/link"

import { siteConfig } from "@/config/site"
import { Logo } from "@/components/logo"

export default function AuthLayout({ children }: React.PropsWithChildren) {
  return (
    <div className="relative grid min-h-screen grid-cols-1 overflow-hidden lg:grid-cols-2">
      <Link
        href="/"
        className="text-foreground/80 hover:text-foreground absolute top-6 left-8 z-20 flex items-center text-lg font-bold tracking-tight transition-colors"
      >
        <Logo className="mr-2 size-6" aria-hidden="true" />
        <span>{siteConfig.name}</span>
      </Link>
      <main className="flex w-full items-center justify-center">
        {children}
      </main>
      <div className="relative aspect-video size-full">
        <Image
          src="/images/auth-layout.webp"
          alt="A vieboarder dropping into a bowl"
          fill
          className="absolute inset-0 object-cover"
          priority
          sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw"
        />
        <div className="from-background absolute inset-0 bg-gradient-to-t to-black/80 lg:to-black/40" />
        <div className="bg-muted text-muted-foreground absolute right-4 bottom-4 z-20 line-clamp-1 rounded-md px-3 py-1.5 text-sm">
          Photo by{" "}
          <a
            href="https://www.youtube.com/@vie-vlogs"
            className="hover:text-foreground underline transition-colors"
          >
            KhieuDV
          </a>
          {" on "}
          <a
            href="https://www.youtube.com/@vie-vlogs"
            className="hover:text-foreground underline transition-colors"
          >
            YouTube
          </a>
        </div>
      </div>
    </div>
  )
}
