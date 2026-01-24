"use client"

import * as React from "react"
import Link from "next/link"
import { formatDistanceToNow } from "date-fns"
import {
  FileText,
  MessageCircle,
  Heart,
  Users,
  TrendingUp,
  Eye,
  PlusCircle,
  FolderTree,
  Flag,
} from "lucide-react"

import { adminPostApi, adminCommentApi } from "@/lib/api/vielang-index"
import type { ContentPost, SocialComment } from "@/lib/api/vielang-types"
import { useAdminAuth } from "@/lib/hooks/use-admin-auth"
import { formatDate } from "@/lib/utils"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"

export default function AdminDashboardPage() {
  const { admin, isLoading: authLoading } = useAdminAuth()
  const [stats, setStats] = React.useState({
    totalPosts: 0,
    publishedPosts: 0,
    draftPosts: 0,
    totalComments: 0,
    totalLikes: 0,
    totalViews: 0,
  })
  const [recentPosts, setRecentPosts] = React.useState<ContentPost[]>([])
  const [recentComments, setRecentComments] = React.useState<SocialComment[]>([])
  const [isLoading, setIsLoading] = React.useState(true)

  React.useEffect(() => {
    // Check both context state AND localStorage to handle race conditions
    const hasLocalAuth =
      typeof window !== "undefined" &&
      localStorage.getItem("admin_auth_token") &&
      localStorage.getItem("admin_user")

    // Only load data when admin is authenticated (either in context or localStorage)
    if (!authLoading && (admin || hasLocalAuth)) {
      loadDashboardData()
    }
  }, [admin, authLoading])

  async function loadDashboardData() {
    try {
      setIsLoading(true)

      // Fetch posts and comments data
      const [allPostsResult, publishedResult, draftsResult, commentsResult] =
        await Promise.all([
          adminPostApi.getList({ pageNum: 1, pageSize: 1 }),
          adminPostApi.getList({ status: 1, pageNum: 1, pageSize: 5 }),
          adminPostApi.getList({ status: 0, pageNum: 1, pageSize: 1 }),
          adminCommentApi.getList({ pageNum: 1, pageSize: 5 }),
        ])

      // Calculate total likes and views from published posts
      const totalLikes =
        publishedResult.list?.reduce((sum, post) => sum + post.likesCount, 0) || 0
      const totalViews =
        publishedResult.list?.reduce((sum, post) => sum + post.viewsCount, 0) || 0

      setStats({
        totalPosts: allPostsResult.total || 0,
        publishedPosts: publishedResult.total || 0,
        draftPosts: draftsResult.total || 0,
        totalComments: commentsResult.total || 0,
        totalLikes,
        totalViews,
      })

      setRecentPosts(publishedResult.list || [])
      setRecentComments(commentsResult.list || [])
    } catch (error) {
      console.error("Error loading dashboard data:", error)
    } finally {
      setIsLoading(false)
    }
  }

  const statCards = [
    {
      title: "Total Posts",
      value: stats.totalPosts.toString(),
      change: `${stats.publishedPosts} published, ${stats.draftPosts} drafts`,
      icon: FileText,
    },
    {
      title: "Total Comments",
      value: stats.totalComments.toString(),
      change: "All time comments",
      icon: MessageCircle,
    },
    {
      title: "Total Likes",
      value: stats.totalLikes.toString(),
      change: "Community engagement",
      icon: Heart,
    },
    {
      title: "Total Views",
      value: stats.totalViews.toString(),
      change: "Content reach",
      icon: Eye,
    },
  ]

  const getPostStatusBadge = (status?: boolean) => {
    if (status === true) {
      return <Badge variant="default">Published</Badge>
    } else if (status === false) {
      return <Badge variant="outline">Draft</Badge>
    }
    return <Badge variant="outline">Unknown</Badge>
  }

  const getCommentStatusBadge = (status?: number) => {
    switch (status) {
      case 0:
        return <Badge variant="outline">Hidden</Badge>
      case 1:
        return <Badge variant="default">Visible</Badge>
      case 2:
        return <Badge variant="destructive">Reported</Badge>
      default:
        return <Badge variant="outline">Unknown</Badge>
    }
  }

  if (authLoading || isLoading) {
    return <LoadingSkeleton />
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">
          Overview of your social platform performance
        </p>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {statCards.map((stat, index) => (
          <Card key={index}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">
                {stat.title}
              </CardTitle>
              <stat.icon className="text-muted-foreground h-4 w-4" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stat.value}</div>
              <p className="text-muted-foreground text-xs">{stat.change}</p>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Quick Actions & Recent Posts */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        <Card className="col-span-4">
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <Button asChild className="w-full">
              <Link href="/admin/posts/new">
                <PlusCircle className="mr-2 h-4 w-4" />
                Create Post
              </Link>
            </Button>
            <Button asChild variant="outline" className="w-full">
              <Link href="/admin/posts">
                <FileText className="mr-2 h-4 w-4" />
                Manage Posts
              </Link>
            </Button>
            <Button asChild variant="outline" className="w-full">
              <Link href="/admin/categories">
                <FolderTree className="mr-2 h-4 w-4" />
                Categories
              </Link>
            </Button>
            <Button asChild variant="outline" className="w-full">
              <Link href="/admin/comments">
                <Flag className="mr-2 h-4 w-4" />
                Moderate Comments
              </Link>
            </Button>
          </CardContent>
        </Card>

        {/* Recent Posts */}
        <Card className="col-span-3">
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle>Recent Posts</CardTitle>
            <Button asChild variant="ghost" size="sm">
              <Link href="/admin/posts">View All</Link>
            </Button>
          </CardHeader>
          <CardContent>
            {recentPosts.length === 0 ? (
              <div className="text-muted-foreground py-6 text-center text-sm">
                No posts yet
              </div>
            ) : (
              <div className="space-y-4">
                {recentPosts.map((post) => (
                  <div key={post.id} className="space-y-1">
                    <Link
                      href={`/admin/posts/${post.id}/edit`}
                      className="text-sm leading-none font-medium hover:underline line-clamp-1"
                    >
                      {post.title}
                    </Link>
                    <div className="flex items-center gap-2">
                      {getPostStatusBadge(post.status)}
                      <span className="text-muted-foreground text-xs">
                        {formatDistanceToNow(new Date(post.createdAt), {
                          addSuffix: true,
                        })}
                      </span>
                    </div>
                    <div className="flex items-center gap-3 text-xs text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <Eye className="h-3 w-3" />
                        {post.viewsCount}
                      </span>
                      <span className="flex items-center gap-1">
                        <Heart className="h-3 w-3" />
                        {post.likesCount}
                      </span>
                      <span className="flex items-center gap-1">
                        <MessageCircle className="h-3 w-3" />
                        {post.commentsCount}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Recent Comments */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>Recent Comments</CardTitle>
          <Button asChild variant="ghost" size="sm">
            <Link href="/admin/comments">View All</Link>
          </Button>
        </CardHeader>
        <CardContent>
          {recentComments.length === 0 ? (
            <div className="text-muted-foreground py-6 text-center text-sm">
              No comments yet
            </div>
          ) : (
            <div className="space-y-4">
              {recentComments.map((comment) => (
                <div
                  key={comment.id}
                  className="flex items-start justify-between gap-4 pb-4 border-b last:border-0 last:pb-0"
                >
                  <div className="space-y-1 flex-1 min-w-0">
                    <p className="text-sm leading-none font-medium">
                      {comment.member?.nickname ||
                        comment.member?.username ||
                        "User"}
                    </p>
                    <p className="text-sm text-muted-foreground line-clamp-2">
                      {comment.content}
                    </p>
                    <div className="flex items-center gap-2">
                      {getCommentStatusBadge(comment.status)}
                      <span className="text-muted-foreground text-xs">
                        {formatDistanceToNow(new Date(comment.createdAt), {
                          addSuffix: true,
                        })}
                      </span>
                    </div>
                  </div>
                  <Button asChild variant="outline" size="sm">
                    <Link href={`/admin/comments?id=${comment.id}`}>
                      Review
                    </Link>
                  </Button>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

function LoadingSkeleton() {
  return (
    <div className="space-y-6">
      <div>
        <Skeleton className="h-9 w-64" />
        <Skeleton className="mt-2 h-5 w-96" />
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <Card key={i}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <Skeleton className="h-4 w-24" />
              <Skeleton className="h-4 w-4" />
            </CardHeader>
            <CardContent>
              <Skeleton className="h-8 w-20" />
              <Skeleton className="mt-2 h-3 w-32" />
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        <Card className="col-span-4">
          <CardHeader>
            <Skeleton className="h-6 w-32" />
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 md:grid-cols-2">
              {Array.from({ length: 4 }).map((_, i) => (
                <Skeleton key={i} className="h-10 w-full" />
              ))}
            </div>
          </CardContent>
        </Card>

        <Card className="col-span-3">
          <CardHeader>
            <Skeleton className="h-6 w-32" />
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {Array.from({ length: 3 }).map((_, i) => (
                <Skeleton key={i} className="h-16 w-full" />
              ))}
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <Skeleton className="h-6 w-32" />
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {Array.from({ length: 3 }).map((_, i) => (
              <Skeleton key={i} className="h-20 w-full" />
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
