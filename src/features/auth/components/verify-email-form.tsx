"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { toast } from "sonner"

import { Button } from "@/components/ui/button"

export function VerifyEmailForm() {
  const router = useRouter()

  React.useEffect(() => {
    // PocketBase handles email verification via email links
    toast.message("Email verification handled by email", {
      description: "Please check your email for verification instructions.",
    })
    router.push("/signin")
  }, [router])

  return (
    <div className="flex flex-col items-center gap-4">
      <p className="text-muted-foreground text-center text-sm">
        Email verification is handled through email links with PocketBase.
        <br />
        Please check your email for verification instructions.
      </p>
      <Button onClick={() => router.push("/signin")} className="w-full">
        Return to Sign In
      </Button>
    </div>
  )
}
