"use client"

import * as React from "react"
import { usePathname, useRouter } from "next/navigation"

import { AdminAuthProvider, useAdminAuth } from "@/lib/hooks/use-admin-auth"
import { Icons } from "@/components/icons"

import { AdminHeader } from "./_components/admin-header"
import { AdminSidebar } from "./_components/admin-sidebar"

function AdminAuthGuard({ children }: { children: React.ReactNode }) {
  const { admin, isLoading } = useAdminAuth()
  const router = useRouter()
  const pathname = usePathname()

  // For login page, always render without layout
  if (pathname === "/admin/login") {
    return <>{children}</>
  }

  // Show loading while checking auth
  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="text-center">
          <Icons.spinner className="text-muted-foreground mx-auto h-8 w-8 animate-spin" />
          <p className="text-muted-foreground mt-2 text-sm">
            Đang kiểm tra phiên đăng nhập...
          </p>
        </div>
      </div>
    )
  }

  // Check both context state AND localStorage to handle race conditions
  // This ensures that immediately after login, we show the admin content
  const hasLocalAuth =
    typeof window !== "undefined" &&
    localStorage.getItem("admin_auth_token") &&
    localStorage.getItem("admin_user")

  // If not authenticated (neither in context nor localStorage), show login prompt
  if (!admin && !hasLocalAuth) {
    return (
      <div className="bg-muted/10 flex min-h-screen items-center justify-center">
        <div className="bg-card mx-auto w-full max-w-md space-y-6 rounded-lg border p-8 text-center shadow-lg">
          <div className="space-y-2">
            <Icons.lock className="text-muted-foreground mx-auto h-12 w-12" />
            <h2 className="text-2xl font-bold">Yêu cầu đăng nhập</h2>
            <p className="text-muted-foreground text-sm">
              Bạn cần đăng nhập để truy cập trang quản trị
            </p>
          </div>
          <button
            onClick={() => router.push("/admin/login")}
            className="bg-primary text-primary-foreground ring-offset-background hover:bg-primary/90 focus-visible:ring-ring inline-flex h-10 w-full items-center justify-center rounded-md px-4 py-2 text-sm font-medium transition-colors focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none"
          >
            Đăng nhập ngay
          </button>
        </div>
      </div>
    )
  }

  // Authenticated (either in context or localStorage) - render with admin layout
  // Special pages that need full-height layout without scroll
  const fullHeightPages = ["/admin/support-chat"]
  const needsFullHeight = fullHeightPages.includes(pathname)

  return (
    <div className="flex h-screen overflow-hidden">
      <AdminSidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <AdminHeader />
        <main
          className={`flex-1 p-6 ${needsFullHeight ? "overflow-hidden" : "overflow-y-auto"}`}
        >
          {children}
        </main>
      </div>
    </div>
  )
}

export default function AdminLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <AdminAuthProvider>
      <AdminAuthGuard>{children}</AdminAuthGuard>
    </AdminAuthProvider>
  )
}
