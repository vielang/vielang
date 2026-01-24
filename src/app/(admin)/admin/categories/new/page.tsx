"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import * as z from "zod"

import { adminCategoryApi } from "@/lib/api/vielang-index"
import type { ContentCategory } from "@/lib/api/vielang-types"
import { useAdminAuth } from "@/lib/hooks/use-admin-auth"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
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
import { Textarea } from "@/components/ui/textarea"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import { toast } from "sonner"

const categoryFormSchema = z.object({
  name: z.string().min(1, "Name is required").max(100, "Name is too long"),
  slug: z
    .string()
    .min(1, "Slug is required")
    .max(100, "Slug is too long")
    .regex(/^[a-z0-9-]+$/, "Slug must contain only lowercase letters, numbers, and hyphens"),
  parentId: z.string().optional(),
  description: z.string().max(500, "Description is too long").optional(),
  icon: z.string().optional(),
  coverImage: z.string().url("Invalid URL").optional().or(z.literal("")),
  sort: z.number().min(0).max(9999).optional(),
  isNav: z.boolean(),
  status: z.boolean(),
})

type CategoryFormValues = z.infer<typeof categoryFormSchema>

export default function NewCategoryPage() {
  const router = useRouter()
  const { admin, isLoading: authLoading } = useAdminAuth()
  const [categories, setCategories] = React.useState<ContentCategory[]>([])
  const [isSubmitting, setIsSubmitting] = React.useState(false)

  const form = useForm<CategoryFormValues>({
    resolver: zodResolver(categoryFormSchema),
    defaultValues: {
      name: "",
      slug: "",
      parentId: "0",
      description: "",
      icon: "",
      coverImage: "",
      sort: 0,
      isNav: true,
      status: true,
    },
  })

  React.useEffect(() => {
    const hasLocalAuth =
      typeof window !== "undefined" &&
      localStorage.getItem("admin_auth_token") &&
      localStorage.getItem("admin_user")

    if (!authLoading && (admin || hasLocalAuth)) {
      loadCategories()
    }
  }, [admin, authLoading])

  async function loadCategories() {
    try {
      const result = await adminCategoryApi.getAll()
      // Only show root categories as parent options
      setCategories(result.filter(cat => cat.parentId === 0) || [])
    } catch (error) {
      console.error("Error loading categories:", error)
      toast.error("Failed to load categories")
    }
  }

  // Auto-generate slug from name
  const watchName = form.watch("name")
  React.useEffect(() => {
    if (watchName && !form.formState.dirtyFields.slug) {
      const generatedSlug = watchName
        .toLowerCase()
        .replace(/[^a-z0-9\s-]/g, "")
        .replace(/\s+/g, "-")
        .replace(/-+/g, "-")
        .slice(0, 100)
      form.setValue("slug", generatedSlug, { shouldValidate: false })
    }
  }, [watchName])

  async function onSubmit(data: CategoryFormValues) {
    setIsSubmitting(true)

    try {
      const categoryData = {
        name: data.name,
        slug: data.slug,
        parentId: data.parentId ? Number(data.parentId) : 0,
        description: data.description || undefined,
        icon: data.icon || undefined,
        coverImage: data.coverImage || undefined,
        sort: data.sort || 0,
      }

      const result = await adminCategoryApi.create(categoryData)

      // Update status and nav after creation
      if (result.code === 200 && result.data) {
        const categoryId = result.data
        
        // Update status
        await adminCategoryApi.updateStatus([categoryId], data.status ? 1 : 0)
        
        // Update nav status
        await adminCategoryApi.updateNavStatus([categoryId], data.isNav ? 1 : 0)
      }

      toast.success("Category created successfully")
      router.push("/admin/categories")
    } catch (error: any) {
      console.error("Error creating category:", error)
      toast.error(error.message || "Failed to create category")
    } finally {
      setIsSubmitting(false)
    }
  }

  if (authLoading) {
    return <div className="container py-6">Loading...</div>
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Create New Category</h1>
        <p className="text-muted-foreground">
          Add a new category to organize your content
        </p>
      </div>

      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <div className="grid gap-6 lg:grid-cols-3">
            {/* Main Content */}
            <div className="lg:col-span-2 space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle>Category Information</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="name"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Name *</FormLabel>
                        <FormControl>
                          <Input placeholder="Technology, Lifestyle, etc." {...field} />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="slug"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Slug *</FormLabel>
                        <FormControl>
                          <Input placeholder="technology" {...field} />
                        </FormControl>
                        <FormDescription>
                          URL-friendly version of the name (auto-generated)
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="description"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Description</FormLabel>
                        <FormControl>
                          <Textarea
                            placeholder="Category description..."
                            rows={4}
                            {...field}
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="icon"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Icon</FormLabel>
                        <FormControl>
                          <Input placeholder="🚀 or icon-name" {...field} />
                        </FormControl>
                        <FormDescription>
                          Emoji or icon identifier
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </CardContent>
              </Card>
            </div>

            {/* Sidebar */}
            <div className="space-y-6">
              {/* Hierarchy */}
              <Card>
                <CardHeader>
                  <CardTitle>Hierarchy</CardTitle>
                </CardHeader>
                <CardContent>
                  <FormField
                    control={form.control}
                    name="parentId"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Parent Category</FormLabel>
                        <Select
                          onValueChange={field.onChange}
                          defaultValue={field.value}
                        >
                          <FormControl>
                            <SelectTrigger>
                              <SelectValue placeholder="Select parent category" />
                            </SelectTrigger>
                          </FormControl>
                          <SelectContent>
                            <SelectItem value="0">Root Category</SelectItem>
                            {categories.map((category) => (
                              <SelectItem
                                key={category.id}
                                value={category.id.toString()}
                              >
                                {category.name}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                        <FormDescription>
                          Leave as root or nest under another category
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </CardContent>
              </Card>

              {/* Cover Image */}
              <Card>
                <CardHeader>
                  <CardTitle>Cover Image</CardTitle>
                </CardHeader>
                <CardContent>
                  <FormField
                    control={form.control}
                    name="coverImage"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Image URL</FormLabel>
                        <FormControl>
                          <Input
                            type="url"
                            placeholder="https://example.com/image.jpg"
                            {...field}
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </CardContent>
              </Card>

              {/* Settings */}
              <Card>
                <CardHeader>
                  <CardTitle>Settings</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="sort"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Sort Order</FormLabel>
                        <FormControl>
                          <Input
                            type="number"
                            min="0"
                            {...field}
                            onChange={(e) => field.onChange(Number(e.target.value))}
                          />
                        </FormControl>
                        <FormDescription>
                          Lower numbers appear first
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="isNav"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-3">
                        <div className="space-y-0.5">
                          <FormLabel>Show in Navigation</FormLabel>
                          <FormDescription>
                            Display in main navigation menu
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="status"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-3">
                        <div className="space-y-0.5">
                          <FormLabel>Active</FormLabel>
                          <FormDescription>
                            Enable this category
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />
                </CardContent>
              </Card>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex justify-end gap-4">
            <Button
              type="button"
              variant="outline"
              onClick={() => router.back()}
              disabled={isSubmitting}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Creating..." : "Create Category"}
            </Button>
          </div>
        </form>
      </Form>
    </div>
  )
}
