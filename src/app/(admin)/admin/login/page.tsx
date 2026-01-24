"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import { toast } from "sonner"
import * as z from "zod"

import { useAdminAuth } from "@/lib/hooks/use-admin-auth"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
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

const loginSchema = z.object({
  username: z
    .string()
    .min(3, "Username phải có ít nhất 3 ký tự")
    .max(50, "Username không được quá 50 ký tự"),
  password: z
    .string()
    .min(3, "Password phải có ít nhất 3 ký tự")
    .max(100, "Password không được quá 100 ký tự"),
})

type LoginFormValues = z.infer<typeof loginSchema>

export default function AdminLoginPage() {
  const router = useRouter()
  const { signIn, admin, isLoading: authLoading } = useAdminAuth()
  const [isLoading, setIsLoading] = React.useState(false)

  // Redirect if already logged in
  React.useEffect(() => {
    if (!authLoading && admin) {
      router.push("/admin")
    }
  }, [admin, authLoading, router])

  const form = useForm<LoginFormValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      username: "",
      password: "",
    },
  })

  async function onSubmit(values: LoginFormValues) {
    setIsLoading(true)

    try {
      await signIn(values.username, values.password)

      toast.success("Đăng nhập thành công!", {
        description: "Chào mừng quay trở lại quản trị viên",
      })

      // Don't redirect here - let the useEffect handle it
      // The useEffect will redirect once the admin state is fully updated
      // This prevents race condition with AdminAuthGuard
    } catch (error: any) {
      console.error("Login error:", error)

      toast.error("Đăng nhập thất bại", {
        description:
          error.message ||
          "Username hoặc password không đúng. Vui lòng thử lại.",
      })
      setIsLoading(false)
    }
  }

  // Show loading if checking auth status
  if (authLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <Icons.spinner className="text-muted-foreground h-8 w-8 animate-spin" />
      </div>
    )
  }

  // Don't show login form if already authenticated
  if (admin) {
    return null
  }

  return (
    <div className="bg-muted/40 flex min-h-screen items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="space-y-1">
          <div className="mb-4 flex items-center justify-center">
            <Icons.logo className="h-10 w-10" />
          </div>
          <CardTitle className="text-center text-2xl font-bold">
            Admin Portal
          </CardTitle>
          <CardDescription className="text-center">
            Đăng nhập vào hệ thống quản trị
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
              <FormField
                control={form.control}
                name="username"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Username</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="admin"
                        disabled={isLoading}
                        autoComplete="username"
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
                      <Input
                        type="password"
                        placeholder="••••••••"
                        disabled={isLoading}
                        autoComplete="current-password"
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <Button type="submit" className="w-full" disabled={isLoading}>
                {isLoading && (
                  <Icons.spinner className="mr-2 h-4 w-4 animate-spin" />
                )}
                Đăng nhập
              </Button>
            </form>
          </Form>
        </CardContent>
      </Card>
    </div>
  )
}
