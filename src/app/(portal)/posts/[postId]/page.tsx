"use client"

import { useEffect, useState } from "react"
import { useParams, useRouter } from "next/navigation"
import { ChevronLeft } from "lucide-react"

import { portalPostApi } from "@/lib/api/vielang-index"
import { Button } from "@/components/ui/button"
import { Skeleton } from "@/components/ui/skeleton"
import { TiptapEditorClient } from "@/components/tiptap/TiptapEditorClient"

export default function PostDetailPage() {
  const params = useParams()
  const router = useRouter()
  const postId = Number(params.postId)

  const [post, setPost] = useState<any>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (isNaN(postId)) {
      setError("Invalid post ID")
      setIsLoading(false)
      return
    }

    loadPost()
  }, [postId])

  async function loadPost() {
    try {
      setIsLoading(true)
      setError(null)

      const postData = await portalPostApi.getDetail(postId)

      if (!postData) {
        setError("Post not found")
        return
      }

      setPost(postData)
    } catch (error) {
      console.error("Error loading post:", error)
      setError("Failed to load post")
    } finally {
      setIsLoading(false)
    }
  }

  if (isLoading) {
    return <LoadingSkeleton />
  }

  if (error || !post) {
    return (
      <div className="flex h-screen items-center justify-center">
        <p className="text-muted-foreground">{error || "Post not found"}</p>
      </div>
    )
  }

  return (
    <div className="flex h-screen flex-col">
      {/* Header with Back button + Title */}
      <header className="border-b bg-background">
        <div className="container flex h-14 items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => router.back()}
          >
            <ChevronLeft className="h-5 w-5" />
          </Button>
          <div className="flex items-center gap-2 min-w-0 flex-1">
            <h1 className="text-base font-semibold truncate">
              {post.title}
            </h1>
          </div>
        </div>
      </header>

      {/* Content */}
      <main className="flex-1 overflow-hidden">
        <div className="h-full overflow-y-auto">
          <div className="py-4 px-2 sm:px-6 lg:px-8">
            <div className="prose prose-slate dark:prose-invert max-w-none lesson-content">
              <TiptapEditorClient
                initialContent={post.content || ""}
                editable={false}
              />
            </div>
          </div>
        </div>
      </main>
    </div>
  )
}

function LoadingSkeleton() {
  return (
    <div className="flex h-screen flex-col">
      <header className="border-b bg-background">
        <div className="container flex h-14 items-center gap-4">
          <Skeleton className="h-9 w-9" />
          <Skeleton className="h-5 w-64" />
        </div>
      </header>
      <main className="flex-1 p-6">
        <div className="space-y-4">
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-4 w-3/4" />
          <Skeleton className="h-64 w-full" />
        </div>
      </main>
    </div>
  )
}
