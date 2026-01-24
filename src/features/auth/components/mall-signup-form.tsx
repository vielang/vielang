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
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form"
import { Input } from "@/components/ui/input"
import { Icons } from "@/components/icons"
import { PasswordInput } from "@/components/password-input"

// Vielang backend signup schema
const mallSignUpSchema = z
  .object({
    username: z.string().min(3, "Username must be at least 3 characters"),
    password: z.string().min(6, "Password must be at least 6 characters"),
    confirmPassword: z.string(),
    telephone: z.string().regex(/^[0-9]{10,11}$/, "Invalid phone number"),
    authCode: z.string().min(4, "Auth code must be at least 4 characters"),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: "Passwords don't match",
    path: ["confirmPassword"],
  })

type Inputs = z.infer<typeof mallSignUpSchema>

export function MallSignUpForm() {
  const router = useRouter()
  const { signUp, getAuthCode } = useSecureAuth()
  const [loading, setLoading] = React.useState(false)
  const [sendingCode, setSendingCode] = React.useState(false)
  const [codeSent, setCodeSent] = React.useState(false)

  // react-hook-form
  const form = useForm<Inputs>({
    resolver: zodResolver(mallSignUpSchema),
    defaultValues: {
      username: "",
      password: "",
      confirmPassword: "",
      telephone: "",
      authCode: "",
    },
  })

  async function handleGetAuthCode() {
    const telephone = form.getValues("telephone")

    if (!telephone || !/^[0-9]{10,11}$/.test(telephone)) {
      toast.error("Please enter a valid phone number first")
      return
    }

    setSendingCode(true)

    try {
      const code = await getAuthCode(telephone)
      toast.success(`Verification code sent: ${code}`)
      setCodeSent(true)
    } catch (err) {
      showErrorToast(err)
    } finally {
      setSendingCode(false)
    }
  }

  async function onSubmit(data: Inputs) {
    setLoading(true)

    try {
      // Use secure auth (BFF endpoints with httpOnly cookies)
      await signUp(data.username, data.password, data.telephone, data.authCode)
      toast.success("Account created successfully! Please sign in.")
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
                <Input type="text" placeholder="Choose a username" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="telephone"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Phone Number</FormLabel>
              <FormControl>
                <div className="flex gap-2">
                  <Input
                    type="tel"
                    placeholder="Enter phone number"
                    {...field}
                  />
                  <Button
                    type="button"
                    variant="outline"
                    onClick={handleGetAuthCode}
                    disabled={sendingCode || codeSent}
                  >
                    {sendingCode && (
                      <Icons.spinner
                        className="mr-2 size-4 animate-spin"
                        aria-hidden="true"
                      />
                    )}
                    {codeSent ? "Code Sent" : "Get Code"}
                  </Button>
                </div>
              </FormControl>
              <FormDescription>
                Enter your phone number to receive verification code
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="authCode"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Verification Code</FormLabel>
              <FormControl>
                <Input
                  type="text"
                  placeholder="Enter verification code"
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

        <FormField
          control={form.control}
          name="confirmPassword"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Confirm Password</FormLabel>
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
          Create Account
          <span className="sr-only">Create account</span>
        </Button>
      </form>
    </Form>
  )
}
