"use client"

import * as React from "react"
import { useRouter, useSearchParams } from "next/navigation"
import { Mail } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Icons } from "@/components/icons"

export function VerifyEmailForm() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const email = searchParams.get("email")

  return (
    <div className="flex flex-col items-center gap-6">
      {/* Mail Icon */}
      <div className="bg-primary/10 flex size-20 items-center justify-center rounded-full">
        <Mail className="text-primary size-10" />
      </div>

      {/* Main Message */}
      <div className="space-y-2 text-center">
        <h3 className="text-lg font-semibold">Check your email</h3>
        <p className="text-muted-foreground text-sm">
          We've sent a verification link to
          {email && (
            <>
              <br />
              <span className="font-medium">{email}</span>
            </>
          )}
        </p>
      </div>

      {/* Instructions */}
      <div className="bg-muted w-full rounded-lg p-4">
        <p className="text-muted-foreground text-sm">
          <span className="font-medium">Next steps:</span>
        </p>
        <ol className="text-muted-foreground mt-2 list-inside list-decimal space-y-1 text-sm">
          <li>Open your email inbox</li>
          <li>Click the verification link in the email</li>
          <li>Return here to sign in</li>
        </ol>
      </div>

      {/* Action Buttons */}
      <div className="flex w-full flex-col gap-2">
        <Button onClick={() => router.push("/signin")} className="w-full">
          Continue to Sign In
        </Button>
        <Button
          variant="outline"
          onClick={() => router.push("/signup")}
          className="w-full"
        >
          Back to Sign Up
        </Button>
      </div>

      {/* Help Text */}
      <p className="text-muted-foreground text-center text-xs">
        Didn't receive the email? Check your spam folder or contact support.
      </p>
    </div>
  )
}
