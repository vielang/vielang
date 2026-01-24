"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import { toast } from "sonner"
import type { z } from "zod"

// TODO: Implement password reset with Vielang Portal API
// import { portalAuthApi } from "@/lib/api/vielang-index"
import { showErrorToast } from "@/lib/handle-error"
import { checkEmailSchema } from "@/lib/validations/auth"
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

type Inputs = z.infer<typeof checkEmailSchema>

export function ResetPasswordForm() {
  const router = useRouter()
  const [loading, setLoading] = React.useState(false)

  // react-hook-form
  const form = useForm<Inputs>({
    resolver: zodResolver(checkEmailSchema),
    defaultValues: {
      email: "",
    },
  })

  async function onSubmit(data: Inputs) {
    setLoading(true)

    try {
      // TODO: Implement with Vielang Portal API
      // Vielang Portal uses phone + authCode for password reset
      // await portalAuthApi.updatePassword(telephone, password, authCode)

      toast.info("Feature not implemented", {
        description:
          "Password reset via email is not available yet. Please contact admin.",
      })

      // await pb.collection("users").requestPasswordReset(data.email)
      // toast.success("Password reset email sent", {
      //   description: "Check your email for password reset instructions.",
      // })
      // router.push("/signin")
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
              <FormLabel>Email</FormLabel>
              <FormControl>
                <Input placeholder="rodneymullen180@gmail.com" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button className="mt-2" disabled={loading}>
          {loading && (
            <Icons.spinner
              className="mr-2 size-4 animate-spin"
              aria-hidden="true"
            />
          )}
          Continue
          <span className="sr-only">
            Continue to reset password verification
          </span>
        </Button>
      </form>
    </Form>
  )
}
