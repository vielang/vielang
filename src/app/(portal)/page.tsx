import { Suspense } from "react"
import Link from "next/link"
import { formatDistanceToNow } from "date-fns"
import { BookOpen, Users, TrendingUp, Heart, MessageCircle, Eye, Layers } from "lucide-react"

import { portalPostApi, portalCategoryApi } from "@/lib/api/vielang-index"
import type { ContentPost, ContentCategory } from "@/lib/api/vielang-types"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"
import { Badge } from "@/components/ui/badge"
import { CategoryCard } from "./_components/category-card"

export const metadata = {
  title: "Vielang - Discover & Share Stories",
  description: "Join our community to discover amazing stories, share your thoughts, and connect with others",
}

async function HomeContent() {
  // Fetch data for homepage in parallel
  let featuredCategories: ContentCategory[] = []
  let latestPosts: ContentPost[] = []
  let trendingPosts: ContentPost[] = []

  try {
    const results = await Promise.all([
      portalCategoryApi.getList(6),
      portalPostApi.getList({ sortBy: "latest", pageSize: 6 }),
      portalPostApi.getTrending(),
    ])

    featuredCategories = results[0]
    latestPosts = results[1].list || []
    trendingPosts = results[2]
  } catch (error) {
    console.error("Error fetching homepage data:", error)
  }

  return (
    <div className="container py-6 sm:py-8">
      {/* Hero Section */}
      <section className="mb-12 sm:mb-16 text-center">
        <h1 className="mb-3 sm:mb-4 text-4xl sm:text-5xl lg:text-6xl font-bold tracking-tight">
          Welcome to Vielang
        </h1>
        <p className="text-muted-foreground mx-auto mb-6 sm:mb-8 max-w-2xl text-base sm:text-lg lg:text-xl px-4">
          Discover amazing stories, share your thoughts, and connect with a vibrant community
        </p>
        <div className="flex flex-col sm:flex-row flex-wrap justify-center gap-3 sm:gap-4 px-4">
          <Button asChild size="lg" className="w-full sm:w-auto">
            <Link href="/posts">Explore Posts</Link>
          </Button>
          <Button asChild variant="outline" size="lg" className="w-full sm:w-auto">
            <Link href="/signup">Join Community</Link>
          </Button>
        </div>
      </section>

      {/* Featured Categories Section */}
      {featuredCategories.length > 0 && (
        <section className="mb-12 sm:mb-16">
          <div className="mb-6 flex items-center justify-between">
            <h2 className="flex items-center gap-2 text-2xl font-bold sm:text-3xl">
              <Layers className="h-6 w-6" />
              Featured Categories
            </h2>
            <Link href="/categories" className="text-sm text-primary hover:underline">
              View All →
            </Link>
          </div>
          <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {featuredCategories.map((category) => (
              <CategoryCard key={category.id} category={category} />
            ))}
          </div>
        </section>
      )}

      {/* Latest Posts Section */}
      {latestPosts.length > 0 && (
        <section className="mb-12 sm:mb-16">
          <div className="mb-6 flex items-center justify-between">
            <h2 className="text-2xl font-bold sm:text-3xl">Latest Posts</h2>
            <Link href="/posts?sort=latest" className="text-sm text-primary hover:underline">
              View All →
            </Link>
          </div>
          <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {latestPosts.map((post) => (
              <Link key={post.id} href={`/posts/${post.id}`}>
                <Card className="h-full hover:shadow-lg transition-shadow overflow-hidden group">
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
                    {post.category && (
                      <Badge variant="secondary" className="mb-2 w-fit">
                        {post.category.name}
                      </Badge>
                    )}
                    <h3 className="text-lg font-semibold line-clamp-2 group-hover:text-primary transition-colors">
                      {post.title}
                    </h3>
                  </CardHeader>
                  {post.summary && (
                    <CardContent className="pb-3">
                      <p className="text-sm text-muted-foreground line-clamp-2">
                        {post.summary}
                      </p>
                    </CardContent>
                  )}
                  <CardFooter className="flex gap-4 text-xs text-muted-foreground">
                    <div className="flex items-center gap-1">
                      <Eye className="h-3 w-3" />
                      <span>{post.viewsCount}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Heart className="h-3 w-3" />
                      <span>{post.likesCount}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <MessageCircle className="h-3 w-3" />
                      <span>{post.commentsCount}</span>
                    </div>
                  </CardFooter>
                </Card>
              </Link>
            ))}
          </div>
        </section>
      )}

      {/* Trending Posts Section */}
      {trendingPosts.length > 0 && (
        <section className="mb-12 sm:mb-16">
          <div className="mb-6 flex items-center justify-between">
            <h2 className="flex items-center gap-2 text-2xl font-bold sm:text-3xl">
              <TrendingUp className="h-6 w-6" />
              Trending Posts
            </h2>
            <Link href="/posts?sort=trending" className="text-sm text-primary hover:underline">
              View All →
            </Link>
          </div>
          <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {trendingPosts.slice(0, 6).map((post) => (
              <Link key={post.id} href={`/posts/${post.id}`}>
                <Card className="h-full hover:shadow-lg transition-shadow overflow-hidden group">
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
                    {post.category && (
                      <Badge variant="secondary" className="mb-2 w-fit">
                        {post.category.name}
                      </Badge>
                    )}
                    <h3 className="text-lg font-semibold line-clamp-2 group-hover:text-primary transition-colors">
                      {post.title}
                    </h3>
                  </CardHeader>
                  {post.summary && (
                    <CardContent className="pb-3">
                      <p className="text-sm text-muted-foreground line-clamp-2">
                        {post.summary}
                      </p>
                    </CardContent>
                  )}
                  <CardFooter className="flex gap-4 text-xs text-muted-foreground">
                    <div className="flex items-center gap-1">
                      <Eye className="h-3 w-3" />
                      <span>{post.viewsCount}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Heart className="h-3 w-3" />
                      <span>{post.likesCount}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <MessageCircle className="h-3 w-3" />
                      <span>{post.commentsCount}</span>
                    </div>
                  </CardFooter>
                </Card>
              </Link>
            ))}
          </div>
        </section>
      )}

      {/* Features Section */}
      <section className="mb-8 sm:mb-12">
        <h2 className="mb-6 sm:mb-8 text-2xl sm:text-3xl font-bold text-center">
          Why Join Vielang
        </h2>
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          <Card>
            <CardContent className="pt-6">
              <BookOpen className="h-10 w-10 mb-4 text-primary" />
              <h3 className="mb-2 text-lg font-semibold">Quality Content</h3>
              <p className="text-muted-foreground text-sm">
                Discover carefully curated posts covering diverse topics and interests
              </p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-6">
              <Users className="h-10 w-10 mb-4 text-primary" />
              <h3 className="mb-2 text-lg font-semibold">Vibrant Community</h3>
              <p className="text-muted-foreground text-sm">
                Connect with like-minded people through comments and discussions
              </p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-6">
              <Heart className="h-10 w-10 mb-4 text-primary" />
              <h3 className="mb-2 text-lg font-semibold">Engage & Share</h3>
              <p className="text-muted-foreground text-sm">
                Like, comment, and share your favorite posts with the community
              </p>
            </CardContent>
          </Card>
        </div>
      </section>
    </div>
  )
}

function LoadingSkeleton() {
  return (
    <div className="container py-6 sm:py-8">
      {/* Hero Skeleton */}
      <div className="mb-12 sm:mb-16 text-center px-4">
        <Skeleton className="mx-auto mb-3 sm:mb-4 h-12 sm:h-14 w-full max-w-md" />
        <Skeleton className="mx-auto mb-6 sm:mb-8 h-6 w-full max-w-2xl" />
        <div className="flex flex-col sm:flex-row justify-center gap-3 sm:gap-4">
          <Skeleton className="h-11 w-full sm:w-40" />
          <Skeleton className="h-11 w-full sm:w-40" />
        </div>
      </div>

      {/* Featured Categories Skeleton */}
      <div className="mb-12 sm:mb-16">
        <div className="flex items-center justify-between mb-6">
          <Skeleton className="h-9 w-56" />
          <Skeleton className="h-5 w-24" />
        </div>
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }).map((_, i) => (
            <Card key={i} className="overflow-hidden">
              <Skeleton className="aspect-video w-full" />
              <CardHeader className="pb-3">
                <Skeleton className="h-6 w-3/4" />
              </CardHeader>
              <CardContent className="pb-3">
                <Skeleton className="h-4 w-full mb-2" />
                <Skeleton className="h-4 w-2/3" />
                <Skeleton className="h-5 w-20 mt-3" />
              </CardContent>
            </Card>
          ))}
        </div>
      </div>

      {/* Latest Posts Skeleton */}
      <div className="mb-12 sm:mb-16">
        <div className="flex items-center justify-between mb-6">
          <Skeleton className="h-9 w-48" />
          <Skeleton className="h-5 w-24" />
        </div>
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }).map((_, i) => (
            <Card key={i} className="overflow-hidden">
              <Skeleton className="aspect-video w-full" />
              <CardHeader className="pb-3">
                <Skeleton className="mb-2 h-5 w-20" />
                <Skeleton className="h-6 w-full" />
              </CardHeader>
              <CardContent className="pb-3">
                <Skeleton className="h-4 w-full mb-2" />
                <Skeleton className="h-4 w-3/4" />
              </CardContent>
              <CardFooter className="flex gap-4">
                {Array.from({ length: 3 }).map((_, j) => (
                  <Skeleton key={j} className="h-4 w-12" />
                ))}
              </CardFooter>
            </Card>
          ))}
        </div>
      </div>

      {/* Trending Posts Skeleton */}
      <div className="mb-12 sm:mb-16">
        <div className="flex items-center justify-between mb-6">
          <Skeleton className="h-9 w-48" />
          <Skeleton className="h-9 w-24" />
        </div>
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }).map((_, i) => (
            <Card key={i} className="overflow-hidden">
              <Skeleton className="aspect-video w-full" />
              <CardHeader className="pb-3">
                <Skeleton className="mb-2 h-5 w-20" />
                <Skeleton className="h-6 w-full" />
              </CardHeader>
              <CardContent className="pb-3">
                <Skeleton className="h-4 w-full mb-2" />
                <Skeleton className="h-4 w-3/4" />
              </CardContent>
              <CardFooter className="flex gap-4">
                {Array.from({ length: 3 }).map((_, j) => (
                  <Skeleton key={j} className="h-4 w-12" />
                ))}
              </CardFooter>
            </Card>
          ))}
        </div>
      </div>

      {/* Features Skeleton */}
      <div className="mb-8 sm:mb-12">
        <Skeleton className="mx-auto mb-6 sm:mb-8 h-9 w-64" />
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <Card key={i}>
              <CardContent className="pt-6">
                <Skeleton className="mb-4 h-10 w-10" />
                <Skeleton className="mb-2 h-6 w-32" />
                <Skeleton className="h-4 w-full mb-2" />
                <Skeleton className="h-4 w-3/4" />
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </div>
  )
}

export default function HomePage() {
  return (
    <Suspense fallback={<LoadingSkeleton />}>
      <HomeContent />
    </Suspense>
  )
}
