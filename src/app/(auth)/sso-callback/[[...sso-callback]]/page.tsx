"use client"

import { useEffect } from "react"
import { useRouter } from "next/navigation"
import { toast } from "sonner"

import { Icons } from "@/components/icons"
import { Shell } from "@/components/shell"

export default function SSOCallbackPage() {
  const router = useRouter()

  useEffect(() => {
    // Since we're not using SSO with PocketBase in the basic setup,
    // redirect users back to signin with a message
    toast.message("SSO not available", {
      description:
        "Single Sign-On is not available with PocketBase in this setup.",
    })
    router.push("/signin")
  }, [router])

  return (
    <Shell className="max-w-lg place-items-center">
      <Icons.spinner className="size-16 animate-spin" aria-hidden="true" />
      <p className="text-muted-foreground text-sm">Redirecting...</p>
    </Shell>
  )
}
