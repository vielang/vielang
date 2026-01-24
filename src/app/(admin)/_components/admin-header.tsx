"use client"

import * as React from "react"
import { useRouter } from "next/navigation"

import { useAdminAuth } from "@/lib/hooks/use-admin-auth"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Icons } from "@/components/icons"

export function AdminHeader() {
  const router = useRouter()
  const { admin, signOut } = useAdminAuth()

  const handleSignOut = async () => {
    await signOut()
    // signOut will automatically redirect to /admin/login
  }

  return (
    <header className="border-b">
      <div className="flex h-16 items-center justify-between px-6">
        <div className="flex items-center gap-4">
          <h2 className="text-2xl font-semibold">Admin Dashboard</h2>
        </div>

        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon">
            <Icons.bell className="h-5 w-5" />
          </Button>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                className="relative h-10 w-10 rounded-full"
              >
                <Avatar className="h-10 w-10">
                  <AvatarImage
                    src={admin?.avatar || undefined}
                    alt={admin?.username || "Admin"}
                  />
                  <AvatarFallback>
                    {admin?.username?.charAt(0).toUpperCase() || "A"}
                  </AvatarFallback>
                </Avatar>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent className="w-56" align="end" forceMount>
              <DropdownMenuLabel className="font-normal">
                <div className="flex flex-col space-y-1">
                  <p className="text-sm leading-none font-medium">
                    {admin?.username || "Admin"}
                  </p>
                  <p className="text-muted-foreground text-xs leading-none">
                    {admin?.email || admin?.nickname || "Administrator"}
                  </p>
                  {admin?.roles && admin.roles.length > 0 && (
                    <p className="text-muted-foreground text-xs leading-none">
                      Roles: {admin.roles.join(", ")}
                    </p>
                  )}
                </div>
              </DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={() => router.push("/admin/settings")}>
                <Icons.settings className="mr-2 h-4 w-4" />
                <span>Settings</span>
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => router.push("/")}>
                <Icons.home className="mr-2 h-4 w-4" />
                <span>View Store</span>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={handleSignOut}>
                <Icons.logout className="mr-2 h-4 w-4" />
                <span>Log out</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </header>
  )
}
