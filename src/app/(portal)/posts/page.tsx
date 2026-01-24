import { Suspense } from "react"
import Link from "next/link"
import { notFound } from "next/navigation"

import { portalPostApi } from "@/lib/api/vielang-index"
import type { ContentPost } from "@/lib/api/vielang-types"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"
import { Badge } from "@/components/ui/badge"
import { formatDistanceToNow } from "date-fns"
import { Calendar, Eye, Heart, MessageCircle } from "lucide-react"

export const metadata = {
  title: "Posts - Vielang",
  description: "Browse all posts on Vielang social platform",
}

interface PostsPageProps {
  searchParams: Promise<{
    category?: string
    q?: string
    sort?: string
    page?: string
  }>
}

async function PostsContent({ searchParams }: PostsPageProps) {
  const params = await searchParams
  const categoryId = params.category ? Number(params.category) : undefined
  const keyword = params.q
  const sortBy = params.sort || "latest"
  const pageNum = params.page ? Number(params.page) : 1
  const pageSize = 20

  try {
    const postsData = await portalPostApi.getList({
      categoryId,
      keyword,
      sortBy,
      pageNum,
      pageSize,
    })

    if (!postsData || !postsData.list) {
      return (
        <div className="container py-8">
          <div className="text-center py-12">
            <p className="text-muted-foreground">No posts found</p>
          </div>
        </div>
      )
    }

    const { list: posts, pageNum: currentPage, totalPage, total } = postsData

    return (
      <div className="container py-6 sm:py-8">
        {/* Header */}
        <div className="mb-6 sm:mb-8">
          <h1 className="mb-2 text-3xl sm:text-4xl font-bold tracking-tight">
            All Posts
          </h1>
          <p className="text-muted-foreground text-sm sm:text-base">
            Discover {total} posts from our community
          </p>
        </div>

        {/* Filters */}
        <div className="mb-6 flex flex-wrap gap-2">
          <Link href="/posts?sort=latest">
            <Button
              variant={sortBy === "latest" ? "default" : "outline"}
              size="sm"
            >
              Latest
            </Button>
          </Link>
          <Link href="/posts?sort=trending">
            <Button
              variant={sortBy === "trending" ? "default" : "outline"}
              size="sm"
            >
              Trending
            </Button>
          </Link>
          <Link href="/posts?sort=popular">
            <Button
              variant={sortBy === "popular" ? "default" : "outline"}
              size="sm"
            >
              Popular
            </Button>
          </Link>
        </div>

        {/* Posts Grid */}
        {posts.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-muted-foreground">
              No posts found matching your criteria
            </p>
          </div>
        ) : (
          <>
            <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
              {posts.map((post) => (
                <PostCard key={post.id} post={post} />
              ))}
            </div>

            {/* Pagination */}
            {totalPage > 1 && (
              <div className="mt-8 flex justify-center gap-2">
                {currentPage > 1 && (
                  <Link
                    href={`/posts?${new URLSearchParams({
                      ...(categoryId && { category: categoryId.toString() }),
                      ...(keyword && { q: keyword }),
                      ...(sortBy && { sort: sortBy }),
                      page: (currentPage - 1).toString(),
                    })}`}
                  >
                    <Button variant="outline" size="sm">
                      Previous
                    </Button>
                  </Link>
                )}

                <div className="flex items-center gap-2">
                  <span className="text-sm text-muted-foreground">
                    Page {currentPage} of {totalPage}
                  </span>
                </div>

                {currentPage < totalPage && (
                  <Link
                    href={`/posts?${new URLSearchParams({
                      ...(categoryId && { category: categoryId.toString() }),
                      ...(keyword && { q: keyword }),
                      ...(sortBy && { sort: sortBy }),
                      page: (currentPage + 1).toString(),
                    })}`}
                  >
                    <Button variant="outline" size="sm">
                      Next
                    </Button>
                  </Link>
                )}
              </div>
            )}
          </>
        )}
      </div>
    )
  } catch (error) {
    console.error("Error loading posts:", error)
    return (
      <div className="container py-8">
        <div className="text-center py-12">
          <p className="text-muted-foreground">
            Error loading posts. Please try again later.
          </p>
        </div>
      </div>
    )
  }
}

function PostCard({ post }: { post: ContentPost }) {
  return (
    <Link href={`/posts/${post.id}`}>
      <Card className="h-full hover:shadow-lg transition-shadow overflow-hidden group">
        {/* Cover Image */}
        {post.coverImage && (
          <div className="aspect-video w-full overflow-hidden bg-muted">
            <img
              src={post.coverImage}
              alt={post.title}
              className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
            />
          </div>
        )}

        <CardHeader className="pb-3">
          {/* Category Badge */}
          {post.category && (
            <Badge variant="secondary" className="mb-2 w-fit">
              {post.category.name}
            </Badge>
          )}

          {/* Title */}
          <h3 className="text-lg font-semibold line-clamp-2 group-hover:text-primary transition-colors">
            {post.title}
          </h3>
        </CardHeader>

        <CardContent className="pb-3">
          {/* Summary */}
          {post.summary && (
            <p className="text-sm text-muted-foreground line-clamp-3">
              {post.summary}
            </p>
          )}
        </CardContent>

        <CardFooter className="flex flex-wrap gap-4 text-xs text-muted-foreground">
          {/* Date */}
          <div className="flex items-center gap-1">
            <Calendar className="h-3 w-3" />
            <span>
              {formatDistanceToNow(new Date(post.createdAt), {
                addSuffix: true,
              })}
            </span>
          </div>

          {/* Views */}
          <div className="flex items-center gap-1">
            <Eye className="h-3 w-3" />
            <span>{post.viewsCount}</span>
          </div>

          {/* Likes */}
          <div className="flex items-center gap-1">
            <Heart className="h-3 w-3" />
            <span>{post.likesCount}</span>
          </div>

          {/* Comments */}
          <div className="flex items-center gap-1">
            <MessageCircle className="h-3 w-3" />
            <span>{post.commentsCount}</span>
          </div>
        </CardFooter>
      </Card>
    </Link>
  )
}

function LoadingSkeleton() {
  return (
    <div className="container py-6 sm:py-8">
      {/* Header Skeleton */}
      <div className="mb-6 sm:mb-8">
        <Skeleton className="mb-2 h-10 w-48" />
        <Skeleton className="h-5 w-64" />
      </div>

      {/* Filters Skeleton */}
      <div className="mb-6 flex flex-wrap gap-2">
        {Array.from({ length: 3 }).map((_, i) => (
          <Skeleton key={i} className="h-9 w-20" />
        ))}
      </div>

      {/* Posts Grid Skeleton */}
      <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {Array.from({ length: 6 }).map((_, i) => (
          <Card key={i} className="h-full overflow-hidden">
            <Skeleton className="aspect-video w-full" />
            <CardHeader className="pb-3">
              <Skeleton className="mb-2 h-5 w-20" />
              <Skeleton className="h-6 w-full" />
            </CardHeader>
            <CardContent className="pb-3">
              <Skeleton className="h-4 w-full mb-2" />
              <Skeleton className="h-4 w-full mb-2" />
              <Skeleton className="h-4 w-3/4" />
            </CardContent>
            <CardFooter className="flex gap-4">
              {Array.from({ length: 4 }).map((_, j) => (
                <Skeleton key={j} className="h-4 w-12" />
              ))}
            </CardFooter>
          </Card>
        ))}
      </div>
    </div>
  )
}

export default function PostsPage(props: PostsPageProps) {
  return (
    <Suspense fallback={<LoadingSkeleton />}>
      <PostsContent {...props} />
    </Suspense>
  )
}
