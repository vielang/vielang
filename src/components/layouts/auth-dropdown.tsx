"use client"

import * as React from "react"
import Link from "next/link"
import { DashboardIcon, ExitIcon, GearIcon } from "@radix-ui/react-icons"

import { type User } from "@/lib/api/types"
import { useSecureAuth } from "@/lib/hooks/use-secure-auth"
import { cn } from "@/lib/utils"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { Button, type ButtonProps } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Skeleton } from "@/components/ui/skeleton"
import { Icons } from "@/components/icons"

interface AuthDropdownProps
  extends React.ComponentPropsWithRef<typeof DropdownMenuTrigger>,
    ButtonProps {}

export function AuthDropdown({ className, ...props }: AuthDropdownProps) {
  const { user, signOut } = useSecureAuth()
  if (!user) {
    return (
      <Button size="sm" className={cn(className)} {...props} asChild>
        <Link href="/signin">
          Sign In
          <span className="sr-only">Sign In</span>
        </Link>
      </Button>
    )
  }

  // Extract initials from nickname or username
  const initials = user.nickname
    ? user.nickname
        .split(" ")
        .map((n) => n.charAt(0))
        .join("")
        .slice(0, 2)
        .toUpperCase()
    : user.username.charAt(0).toUpperCase()

  const contactInfo = user.phone || user.username // Use phone or username for contact display

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="secondary"
          className={cn("size-8 rounded-full", className)}
          {...props}
        >
          <Avatar className="size-8">
            <AvatarImage src={user.icon} alt={user.username} />{" "}
            {/* Use user.icon for avatar */}
            <AvatarFallback>{initials}</AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-56" align="end" forceMount>
        <DropdownMenuLabel className="font-normal">
          <div className="flex flex-col space-y-1">
            <p className="text-sm leading-none font-medium">
              {user.nickname || user.username}
            </p>
            <p className="text-muted-foreground text-xs leading-none">
              {contactInfo}
            </p>
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuItem asChild>
            <Link href="/orders">
              <Icons.orders className="mr-2 size-4" aria-hidden="true" />
              My Orders
              <DropdownMenuShortcut>⌘O</DropdownMenuShortcut>
            </Link>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <Link href="/cart">
              <Icons.cart className="mr-2 size-4" aria-hidden="true" />
              Shopping Cart
              <DropdownMenuShortcut>⌘C</DropdownMenuShortcut>
            </Link>
          </DropdownMenuItem>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => signOut()} className="cursor-pointer">
          <ExitIcon className="mr-2 size-4" aria-hidden="true" />
          Log out
          <DropdownMenuShortcut>⇧⌘Q</DropdownMenuShortcut>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
