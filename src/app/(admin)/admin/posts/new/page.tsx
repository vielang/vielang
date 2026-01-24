"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import * as z from "zod"
import type { JSONContent } from "@tiptap/core"

import { adminPostApi, adminCategoryApi } from "@/lib/api/vielang-index"
import type { ContentCategory } from "@/lib/api/vielang-types"
import { useAdminAuth } from "@/lib/hooks/use-admin-auth"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form"
import { toast } from "sonner"
import { TiptapEditorClient } from "@/components/tiptap/TiptapEditorClient"

const postFormSchema = z.object({
  title: z.string().min(1, "Title is required").max(200, "Title is too long"),
  slug: z
    .string()
    .min(1, "Slug is required")
    .max(200, "Slug is too long")
    .regex(/^[a-z0-9-]+$/, "Slug must contain only lowercase letters, numbers, and hyphens"),
  summary: z.string().max(500, "Summary is too long").optional(),
  content: z.string().min(1, "Content is required"),
  coverImage: z.string().url("Invalid URL").optional().or(z.literal("")),
  categoryId: z.string().optional(),
  status: z.enum(["0", "1", "2"]),
  visibility: z.enum(["0", "1", "2"]),
  isFeatured: z.boolean(),
  isPinned: z.boolean(),
  allowComment: z.boolean(),
})

type PostFormValues = z.infer<typeof postFormSchema>

export default function NewPostPage() {
  const router = useRouter()
  const { admin, isLoading: authLoading } = useAdminAuth()
  const [categories, setCategories] = React.useState<ContentCategory[]>([])
  const [isSubmitting, setIsSubmitting] = React.useState(false)
  const [editorContent, setEditorContent] = React.useState<string>("")

  const form = useForm<PostFormValues>({
    resolver: zodResolver(postFormSchema),
    defaultValues: {
      title: "",
      slug: "",
      summary: "",
      content: "",
      coverImage: "",
      categoryId: "",
      status: "0",
      visibility: "1",
      isFeatured: false,
      isPinned: false,
      allowComment: true,
    },
  })

  // Handle editor content changes
  const handleEditorChange = React.useCallback(
    (_json: JSONContent, html: string) => {
      setEditorContent(html)
      form.setValue("content", html, { shouldValidate: true })
    },
    [form]
  )

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
      console.log("Categories loaded:", result)
      
      // Ensure result is an array
      if (Array.isArray(result)) {
        setCategories(result)
      } else {
        console.warn("Categories result is not an array:", result)
        setCategories([])
      }
    } catch (error) {
      console.error("Error loading categories:", error)
      toast.error("Failed to load categories")
      setCategories([])
    }
  }

  // Auto-generate slug from title
  const watchTitle = form.watch("title")
  React.useEffect(() => {
    if (watchTitle && !form.formState.dirtyFields.slug) {
      const generatedSlug = watchTitle
        .toLowerCase()
        .replace(/[^a-z0-9\s-]/g, "")
        .replace(/\s+/g, "-")
        .replace(/-+/g, "-")
        .slice(0, 200)
      form.setValue("slug", generatedSlug, { shouldValidate: false })
    }
  }, [watchTitle])

  async function onSubmit(data: PostFormValues) {
    setIsSubmitting(true)

    try {
      const postData = {
        title: data.title,
        slug: data.slug,
        summary: data.summary || undefined,
        content: data.content,
        coverImage: data.coverImage || undefined,
        categoryId: data.categoryId ? Number(data.categoryId) : undefined,
        status: Number(data.status),
        visibility: Number(data.visibility),
        isFeatured: data.isFeatured ? 1 : 0,
        isPinned: data.isPinned ? 1 : 0,
        allowComment: data.allowComment ? 1 : 0,
      }

      const result = await adminPostApi.create(postData)

      toast.success("Post created successfully")
      router.push("/admin/posts")
    } catch (error: any) {
      console.error("Error creating post:", error)
      toast.error(error.message || "Failed to create post")
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
        <h1 className="text-3xl font-bold tracking-tight">Create New Post</h1>
        <p className="text-muted-foreground">
          Write and publish a new post for your community
        </p>
      </div>

      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <div className="grid gap-6 lg:grid-cols-3">
            {/* Main Content */}
            <div className="lg:col-span-2 space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle>Post Content</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="title"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Title *</FormLabel>
                        <FormControl>
                          <Input
                            placeholder="Enter post title..."
                            {...field}
                          />
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
                          <Input
                            placeholder="post-url-slug"
                            {...field}
                          />
                        </FormControl>
                        <FormDescription>
                          URL-friendly version of the title (auto-generated)
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="summary"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Summary</FormLabel>
                        <FormControl>
                          <Textarea
                            placeholder="Brief summary of the post (optional)..."
                            className="min-h-[100px] resize-none"
                            {...field}
                          />
                        </FormControl>
                        <FormDescription>
                          {field.value?.length || 0}/500 characters
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="content"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Content *</FormLabel>
                        <FormControl>
                          <TiptapEditorClient
                            initialContent={editorContent}
                            onChange={handleEditorChange}
                            placeholder="Write your post content or type '/' for commands..."
                            showLanguageSelector={false}
                            showThemeSwitcher={false}
                          />
                        </FormControl>
                        <FormDescription>
                          Full WYSIWYG editor with tables, formulas, diagrams, media, and export options
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
              {/* Publish Settings */}
              <Card>
                <CardHeader>
                  <CardTitle>Publish</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="status"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Status</FormLabel>
                        <Select
                          onValueChange={field.onChange}
                          defaultValue={field.value}
                        >
                          <FormControl>
                            <SelectTrigger>
                              <SelectValue placeholder="Select status" />
                            </SelectTrigger>
                          </FormControl>
                          <SelectContent>
                            <SelectItem value="0">Draft</SelectItem>
                            <SelectItem value="1">Published</SelectItem>
                            <SelectItem value="2">Archived</SelectItem>
                          </SelectContent>
                        </Select>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="visibility"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Visibility</FormLabel>
                        <Select
                          onValueChange={field.onChange}
                          defaultValue={field.value}
                        >
                          <FormControl>
                            <SelectTrigger>
                              <SelectValue placeholder="Select visibility" />
                            </SelectTrigger>
                          </FormControl>
                          <SelectContent>
                            <SelectItem value="0">Private</SelectItem>
                            <SelectItem value="1">Public</SelectItem>
                            <SelectItem value="2">Members Only</SelectItem>
                          </SelectContent>
                        </Select>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </CardContent>
              </Card>

              {/* Category */}
              <Card>
                <CardHeader>
                  <CardTitle>Category</CardTitle>
                </CardHeader>
                <CardContent>
                  <FormField
                    control={form.control}
                    name="categoryId"
                    render={({ field }) => (
                      <FormItem>
                        <Select
                          onValueChange={field.onChange}
                          defaultValue={field.value}
                        >
                          <FormControl>
                            <SelectTrigger>
                              <SelectValue placeholder="Select category" />
                            </SelectTrigger>
                          </FormControl>
                          <SelectContent>
                            {categories && categories.length > 0 ? (
                              categories.map((category) => (
                                <SelectItem
                                  key={category.id}
                                  value={category.id.toString()}
                                >
                                  {category.name}
                                </SelectItem>
                              ))
                            ) : (
                              <div className="px-2 py-1.5 text-sm text-muted-foreground">
                                No categories available
                              </div>
                            )}
                          </SelectContent>
                        </Select>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </CardContent>
              </Card>

              {/* Featured Image */}
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
                        <FormControl>
                          <Input
                            placeholder="https://example.com/image.jpg"
                            {...field}
                          />
                        </FormControl>
                        <FormDescription>
                          Enter the URL of the cover image
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </CardContent>
              </Card>

              {/* Post Options */}
              <Card>
                <CardHeader>
                  <CardTitle>Options</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="isFeatured"
                    render={({ field }) => (
                      <FormItem className="flex items-center space-x-2 space-y-0">
                        <FormControl>
                          <Checkbox
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                        <FormLabel className="cursor-pointer">
                          Featured Post
                        </FormLabel>
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="isPinned"
                    render={({ field }) => (
                      <FormItem className="flex items-center space-x-2 space-y-0">
                        <FormControl>
                          <Checkbox
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                        <FormLabel className="cursor-pointer">
                          Pin to Top
                        </FormLabel>
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="allowComment"
                    render={({ field }) => (
                      <FormItem className="flex items-center space-x-2 space-y-0">
                        <FormControl>
                          <Checkbox
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                        <FormLabel className="cursor-pointer">
                          Allow Comments
                        </FormLabel>
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
              {isSubmitting ? "Creating..." : "Create Post"}
            </Button>
          </div>
        </form>
      </Form>
    </div>
  )
}
