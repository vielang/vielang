"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import { toast } from "sonner"
import type { z } from "zod"

import { showErrorToast } from "@/lib/handle-error"
import { useSecureAuth } from "@/lib/hooks/use-secure-auth"
import { authSchema } from "@/lib/validations/auth"
import { Button } from "@/components/ui/button"
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form"
import { Input } from "@/components/ui/input"
import { Icons } from "@/components/icons"
import { PasswordInput } from "@/components/password-input"

type Inputs = z.infer<typeof authSchema>

export function SignInForm() {
  const router = useRouter()
  const { signIn } = useSecureAuth()
  const [loading, setLoading] = React.useState(false)

  // react-hook-form
  const form = useForm<Inputs>({
    resolver: zodResolver(authSchema),
    defaultValues: {
      email: "",
      password: "",
    },
  })

  async function onSubmit(data: Inputs) {
    setLoading(true)

    try {
      // Vielang API uses username (not email) for authentication
      // authSchema.email field accepts username or email
      await signIn(data.email, data.password)
      toast.success("Signed in successfully")
      // Router redirect is handled by useSecureAuth hook
    } catch (err) {
      showErrorToast(err)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Form {...form}>
      <form className="grid gap-4" onSubmit={form.handleSubmit(onSubmit)}>
        <FormField
          control={form.control}
          name="email"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Username</FormLabel>
              <FormControl>
                <Input
                  type="text"
                  placeholder="Enter your username"
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Password</FormLabel>
              <FormControl>
                <PasswordInput placeholder="**********" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit" className="mt-2" disabled={loading}>
          {loading && (
            <Icons.spinner
              className="mr-2 size-4 animate-spin"
              aria-hidden="true"
            />
          )}
          Sign in
          <span className="sr-only">Sign in</span>
        </Button>
        <div className="bg-muted/50 mt-4 rounded-md border p-3">
          <p className="mb-2 text-sm font-medium">Demo Account:</p>
          <p className="text-muted-foreground text-sm">
            Username: <span className="font-mono">test</span>
          </p>
          <p className="text-muted-foreground text-sm">
            Password: <span className="font-mono">123456</span>
          </p>
          <p className="text-muted-foreground mt-2 text-xs">
            ⚠️ If login fails with "Incorrect username or password", the
            vielang-portal backend needs to be restarted to apply security
            whitelist configuration.
          </p>
        </div>
      </form>
    </Form>
  )
}
