"use client"

import * as React from "react"

/**
 * Sign Up Form - Under Construction
 * 
 * This component needs to be refactored to support Vielang API authentication.
 * Vielang API requires:
 * - username
 * - password
 * - telephone (phone number)
 * - authCode (verification code from SMS)
 * 
 * Previous PocketBase implementation used email-based signup.
 * New implementation requires phone verification flow.
 */
export function SignUpForm() {
  return (
    <div className="text-center p-8 border rounded-lg">
      <h3 className="text-lg font-semibold mb-2">Sign Up - Under Construction</h3>
      <p className="text-sm text-muted-foreground mb-4">
        This feature is being updated to support Vielang API authentication.
      </p>
      <p className="text-xs text-muted-foreground">
        New signup will require phone number verification instead of email.
      </p>
    </div>
  )
}
