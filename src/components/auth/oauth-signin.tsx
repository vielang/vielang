"use client"

import * as React from "react"
import { toast } from "sonner"

import { Button } from "@/components/ui/button"
import { Icons } from "@/components/icons"

const oauthProviders = [
  { name: "Google", icon: "google" },
  { name: "Discord", icon: "discord" },
] as const

export function OAuthSignIn() {
  const handleOAuthSignIn = (providerName: string) => {
    toast.message("OAuth not available", {
      description: `${providerName} OAuth will be available in a future update with PocketBase.`,
    })
  }

  return (
    <div className="flex flex-col items-center gap-2 sm:flex-row sm:gap-4">
      {oauthProviders.map((provider) => {
        const Icon = Icons[provider.icon]

        return (
          <Button
            key={provider.name}
            variant="outline"
            className="bg-background w-full opacity-50"
            onClick={() => handleOAuthSignIn(provider.name)}
            disabled
          >
            <Icon className="mr-2 size-4" aria-hidden="true" />
            {provider.name}
            <span className="sr-only">{provider.name} (Coming soon)</span>
          </Button>
        )
      })}
    </div>
  )
}
