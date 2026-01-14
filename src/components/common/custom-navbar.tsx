"use client"

import { Navbar } from 'nextra-theme-docs'
import Link from 'next/link'
import { LogOut, User } from 'lucide-react'
import { signOut, useSession } from '@/features/auth/services/authClient'
import { Button } from '../custom/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu'

export function CustomNavbar() {
  const { data: session } = useSession()

  const handleSignOut = () => {
    const auth = localStorage.getItem('pocketbase_auth')
    const token = auth ? JSON.parse(auth)?.token : undefined
    void signOut(token)
  }

  return (
    <Navbar
      logo={<span className="font-bold text-primary">VieLang</span>}
      logoLink="/"
    >
      <div className="flex items-center gap-4">
        {/* <Link href="/docs" className="hover:text-primary">
          Docs
        </Link> */}
        {session ? (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="relative overflow-hidden rounded-full"
              >
                {session.user?.image ? (
                  <img
                    src={session.user.image}
                    alt={session.user.username || 'User'}
                    className="h-9 w-9 rounded-full object-cover"
                  />
                ) : (
                  <User className="h-5 w-5" />
                )}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-56">
              <div className="flex items-center justify-start gap-2 p-2">
                <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10">
                  {session.user?.image ? (
                    <img
                      src={session.user.image}
                      alt={session.user.username || 'User'}
                      className="h-7 w-7 rounded-full object-cover"
                    />
                  ) : (
                    <User className="h-4 w-4 text-primary" />
                  )}
                </div>
                <div className="flex flex-col space-y-0.5">
                  <p className="text-sm font-medium">
                    {session.user?.name || 'User'}
                  </p>
                  <p className="text-xs text-muted-foreground truncate max-w-[160px]">
                    {session.user?.email}
                  </p>
                </div>
              </div>
              <DropdownMenuSeparator />
              <DropdownMenuItem asChild>
                <Link href="/profile" className="cursor-pointer">
                  <User className="mr-2 h-4 w-4" />
                  Profile
                </Link>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={handleSignOut}
                className="cursor-pointer text-destructive focus:text-destructive"
              >
                <LogOut className="mr-2 h-4 w-4" />
                Log out
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        ) : (
          <div className="flex items-center gap-2">
            <Link href="/auth/sign-in">
              <Button
                variant="ghost"
                size="sm"
                className="text-slate-600 dark:text-muted-foreground hover:text-slate-900 dark:hover:text-foreground"
              >
                Log in
              </Button>
            </Link>
            <Link href="/auth/sign-up">
              <Button
                size="sm"
                className="bg-blue-600 hover:bg-blue-700 text-white"
              >
                Sign up
              </Button>
            </Link>
          </div>
        )}
      </div>
    </Navbar>
  )
}
