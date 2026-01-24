"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { toast } from "sonner"

import { Button } from "@/components/ui/button"

export function ResetPasswordConfirmForm() {
  const router = useRouter()

  React.useEffect(() => {
    // PocketBase handles password reset via email links, not OTP codes
    toast.message("Password reset handled by email", {
      description: "Please check your email for password reset instructions.",
    })
    router.push("/signin")
  }, [router])

  return (
    <div className="flex flex-col items-center gap-4">
      <p className="text-muted-foreground text-center text-sm">
        Password reset is handled through email links with PocketBase.
        <br />
        Please check your email for password reset instructions.
      </p>
      <Button onClick={() => router.push("/signin")} className="w-full">
        Return to Sign In
      </Button>
    </div>
  )
}
