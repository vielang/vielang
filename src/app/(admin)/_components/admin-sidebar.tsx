"use client"

import * as React from "react"
import Link from "next/link"
import { usePathname } from "next/navigation"

import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Icons } from "@/components/icons"

const routes = [
  {
    label: "Dashboard",
    icon: Icons.dashboard,
    href: "/admin",
    color: "text-sky-500",
  },
  {
    label: "Content",
    icon: Icons.book,
    href: "/admin/content",
    color: "text-violet-600",
    children: [
      {
        label: "Posts",
        href: "/admin/posts",
      },
      {
        label: "Categories",
        href: "/admin/categories",
      },
      {
        label: "Comments",
        href: "/admin/comments",
      },
      {
        label: "Users",
        href: "/admin/customers",
      },
      {
        label: "App Version",
        href: "/admin/app-version",
      },
    ],
  },
  {
    label: "Analytics",
    icon: Icons.analytics,
    href: "/admin/analytics",
    color: "text-yellow-500",
  },
  {
    label: "Settings",
    icon: Icons.settings,
    href: "/admin/settings",
  },
]

export function AdminSidebar() {
  const pathname = usePathname()
  const [expandedRoutes, setExpandedRoutes] = React.useState<string[]>([])

  // Auto-expand parent if child is active
  React.useEffect(() => {
    routes.forEach((route) => {
      if (route.children) {
        const hasActiveChild = route.children.some((child) =>
          pathname.startsWith(child.href)
        )
        if (hasActiveChild && !expandedRoutes.includes(route.href)) {
          setExpandedRoutes([...expandedRoutes, route.href])
        }
      }
    })
  }, [pathname])

  const toggleExpand = (href: string) => {
    setExpandedRoutes((prev) =>
      prev.includes(href)
        ? prev.filter((item) => item !== href)
        : [...prev, href]
    )
  }

  return (
    <div className="bg-background flex h-screen w-64 flex-col space-y-4 border-r py-4">
      <div className="px-3 py-2">
        <Link href="/admin" className="mb-14 flex items-center pl-3">
          <h1 className="text-2xl font-bold">Vielang Admin</h1>
        </Link>
        <ScrollArea className="h-[calc(100vh-8rem)]">
          <div className="space-y-1">
            {routes.map((route) => (
              <div key={route.href}>
                {route.children ? (
                  <>
                    <button
                      onClick={() => toggleExpand(route.href)}
                      className={cn(
                        "group hover:bg-accent hover:text-accent-foreground flex w-full cursor-pointer justify-start rounded-lg p-3 text-sm font-medium transition",
                        pathname.startsWith(route.href)
                          ? "bg-accent text-accent-foreground"
                          : "text-muted-foreground"
                      )}
                    >
                      <div className="flex flex-1 items-center">
                        <route.icon
                          className={cn("mr-3 h-5 w-5", route.color)}
                        />
                        {route.label}
                      </div>
                      <Icons.chevronRight
                        className={cn(
                          "h-4 w-4 transition-transform",
                          expandedRoutes.includes(route.href) && "rotate-90"
                        )}
                      />
                    </button>
                    {expandedRoutes.includes(route.href) && (
                      <div className="mt-1 ml-6 space-y-1">
                        {route.children.map((child) => (
                          <Link
                            key={child.href}
                            href={child.href}
                            className={cn(
                              "hover:bg-accent hover:text-accent-foreground flex w-full cursor-pointer justify-start rounded-lg p-2 text-sm font-medium transition",
                              pathname === child.href
                                ? "bg-accent text-accent-foreground"
                                : "text-muted-foreground"
                            )}
                          >
                            {child.label}
                          </Link>
                        ))}
                      </div>
                    )}
                  </>
                ) : (
                  <Link
                    href={route.href}
                    className={cn(
                      "group hover:bg-accent hover:text-accent-foreground flex w-full cursor-pointer justify-start rounded-lg p-3 text-sm font-medium transition",
                      pathname === route.href
                        ? "bg-accent text-accent-foreground"
                        : "text-muted-foreground"
                    )}
                  >
                    <div className="flex flex-1 items-center">
                      <route.icon className={cn("mr-3 h-5 w-5", route.color)} />
                      {route.label}
                    </div>
                  </Link>
                )}
              </div>
            ))}
          </div>
        </ScrollArea>
      </div>
    </div>
  )
}
