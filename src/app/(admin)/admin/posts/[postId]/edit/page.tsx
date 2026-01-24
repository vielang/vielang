"use client"

import * as React from "react"
import { useRouter, useParams } from "next/navigation"
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import * as z from "zod"
import type { JSONContent } from "@tiptap/core"

import { adminPostApi, adminCategoryApi } from "@/lib/api/vielang-index"
import type { ContentCategory, ContentPost } from "@/lib/api/vielang-types"
import { useAdminAuth } from "@/lib/hooks/use-admin-auth"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
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
import { Skeleton } from "@/components/ui/skeleton"
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
  status: z.boolean(),
  visibility: z.boolean(),
  isFeatured: z.boolean(),
  isPinned: z.boolean(),
  allowComment: z.boolean(),
})

type PostFormValues = z.infer<typeof postFormSchema>

export default function EditPostPage() {
  const router = useRouter()
  const params = useParams()
  const postId = params.postId as string
  const { admin, isLoading: authLoading } = useAdminAuth()

  const [categories, setCategories] = React.useState<ContentCategory[]>([])
  const [isSubmitting, setIsSubmitting] = React.useState(false)
  const [isLoadingPost, setIsLoadingPost] = React.useState(true)
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
      status: false,
      visibility: true,
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
      loadPost()
    }
  }, [admin, authLoading, postId])

  async function loadCategories() {
    try {
      const result = await adminCategoryApi.getAll()
      if (Array.isArray(result)) {
        setCategories(result)
      } else {
        setCategories([])
      }
    } catch (error) {
      console.error("Error loading categories:", error)
      toast.error("Failed to load categories")
      setCategories([])
    }
  }

  async function loadPost() {
    try {
      setIsLoadingPost(true)
      const post = await adminPostApi.getById(Number(postId))

      // Set editor content
      setEditorContent(post.content || "")

      form.reset({
        title: post.title,
        slug: post.slug,
        summary: post.summary || "",
        content: post.content,
        coverImage: post.coverImage || "",
        categoryId: post.categoryId?.toString() || "",
        status: post.status,
        visibility: post.visibility,
        isFeatured: post.isFeatured,
        isPinned: post.isPinned,
        allowComment: post.allowComment,
      })
    } catch (error) {
      console.error("Error loading post:", error)
      toast.error("Failed to load post")
      router.push("/admin/posts")
    } finally {
      setIsLoadingPost(false)
    }
  }

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
        status: data.status ? 1 : 0,
        visibility: data.visibility ? 1 : 0,
        isFeatured: data.isFeatured ? 1 : 0,
        isPinned: data.isPinned ? 1 : 0,
        allowComment: data.allowComment ? 1 : 0,
      }

      await adminPostApi.update(Number(postId), postData)

      toast.success("Post updated successfully")
      router.push("/admin/posts")
    } catch (error: any) {
      console.error("Error updating post:", error)
      toast.error(error.message || "Failed to update post")
    } finally {
      setIsSubmitting(false)
    }
  }

  if (authLoading || isLoadingPost) {
    return (
      <div className="space-y-6">
        <div>
          <Skeleton className="h-9 w-64" />
          <Skeleton className="mt-2 h-5 w-96" />
        </div>
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-32" />
          </CardHeader>
          <CardContent className="space-y-4">
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-32 w-full" />
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Edit Post</h1>
        <p className="text-muted-foreground">
          Update your post content and settings
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
                          <Input placeholder="Post title" {...field} />
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
                          <Input placeholder="post-slug" {...field} />
                        </FormControl>
                        <FormDescription>
                          URL-friendly version of the title
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
                            placeholder="Brief summary..."
                            rows={3}
                            {...field}
                          />
                        </FormControl>
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
                  <CardTitle>Publish Settings</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="status"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-3">
                        <div className="space-y-0.5">
                          <FormLabel>Published</FormLabel>
                          <FormDescription>
                            Make this post public
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Checkbox
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="visibility"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-3">
                        <div className="space-y-0.5">
                          <FormLabel>Public</FormLabel>
                          <FormDescription>
                            Visible to everyone
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Checkbox
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
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
                          value={field.value}
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
                  <CardTitle>Featured Image</CardTitle>
                </CardHeader>
                <CardContent>
                  <FormField
                    control={form.control}
                    name="coverImage"
                    render={({ field }) => (
                      <FormItem>
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

              {/* Post Options */}
              <Card>
                <CardHeader>
                  <CardTitle>Post Options</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="isFeatured"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-3">
                        <div className="space-y-0.5">
                          <FormLabel>Featured</FormLabel>
                          <FormDescription>
                            Highlight this post
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Checkbox
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="isPinned"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-3">
                        <div className="space-y-0.5">
                          <FormLabel>Pinned</FormLabel>
                          <FormDescription>
                            Pin to top
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Checkbox
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="allowComment"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-3">
                        <div className="space-y-0.5">
                          <FormLabel>Allow Comments</FormLabel>
                          <FormDescription>
                            Enable comments
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Checkbox
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
              {isSubmitting ? "Updating..." : "Update Post"}
            </Button>
          </div>
        </form>
      </Form>
    </div>
  )
}
