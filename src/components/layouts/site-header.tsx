"use client"

import { siteConfig } from "@/config/site"
import { useSecureAuth } from "@/lib/hooks/use-secure-auth"
import { AuthDropdown } from "@/components/layouts/auth-dropdown"
import { MainNav } from "@/components/layouts/main-nav"
import { MobileNav } from "@/components/layouts/mobile-nav"

interface SiteHeaderProps {
  hidden?: boolean
}

export function SiteHeader({ hidden = false }: SiteHeaderProps) {
  const { user } = useSecureAuth()

  if (hidden) {
    return null
  }

  return (
    <header className="bg-background sticky top-0 z-50 w-full border-b">
      <div className="container flex h-16 items-center">
        <MainNav items={siteConfig.mainNav} />
        <MobileNav items={siteConfig.mainNav} />
        <div className="flex flex-1 items-center justify-end space-x-4">
          <nav className="flex items-center space-x-2">
            <AuthDropdown />
          </nav>
        </div>
      </div>
    </header>
  )
}
