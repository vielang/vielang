import * as React from "react"

import { AdminAuthProvider } from "@/lib/hooks/use-admin-auth"

export default function AdminLoginLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return <AdminAuthProvider>{children}</AdminAuthProvider>
}
