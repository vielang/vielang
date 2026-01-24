"use client"

import { usePathname } from "next/navigation"
import { SiteFooter } from "@/components/layouts/site-footer"
import { SiteHeader } from "@/components/layouts/site-header"
import { useSecureAuth } from "@/lib/hooks/use-secure-auth"

interface LobyLayoutProps {
  children: React.ReactNode
  modal: React.ReactNode
}

export default function LobyLayout({ children, modal }: LobyLayoutProps) {
  const pathname = usePathname()
  const { user } = useSecureAuth()

  // Debug user object
  console.log("[PortalLayout] User object:", user)
  console.log("[PortalLayout] User ID:", user?.id)
  console.log("[PortalLayout] User username:", user?.username)

  // Hide header and footer on lesson/post pages for better reading experience
  const isLessonPage = pathname?.includes('/lessons/') && /\/lessons\/\d+$/.test(pathname)
  const isPostPage = pathname?.includes('/posts/') && /\/posts\/\d+$/.test(pathname)
  const hideHeaderFooter = isLessonPage || isPostPage

  return (
    <div className="relative flex min-h-screen flex-col">
      <SiteHeader hidden={hideHeaderFooter} />
      <main className="flex-1">
        {children}
        {modal}
      </main>
      <SiteFooter hidden={hideHeaderFooter} />
    </div>
  )
}
