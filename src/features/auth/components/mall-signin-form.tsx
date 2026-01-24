"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import { toast } from "sonner"
import { z } from "zod"

import { showErrorToast } from "@/lib/handle-error"
import { useSecureAuth } from "@/lib/hooks/use-secure-auth"
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

// Vielang backend signin schema
const mallSignInSchema = z.object({
  username: z.string().min(1, "Username is required"),
  password: z.string().min(1, "Password is required"),
})

type Inputs = z.infer<typeof mallSignInSchema>

export function MallSignInForm() {
  const router = useRouter()
  const { signIn } = useSecureAuth()
  const [loading, setLoading] = React.useState(false)

  // react-hook-form
  const form = useForm<Inputs>({
    resolver: zodResolver(mallSignInSchema),
    defaultValues: {
      username: "",
      password: "",
    },
  })

  async function onSubmit(data: Inputs) {
    setLoading(true)

    try {
      // Use secure auth (BFF endpoints with httpOnly cookies)
      await signIn(data.username, data.password)
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
          name="username"
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
            Username: <span className="font-mono">admin</span>
          </p>
          <p className="text-muted-foreground text-sm">
            Password: <span className="font-mono">admin123</span>
          </p>
        </div>
      </form>
    </Form>
  )
}
