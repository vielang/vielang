"use client"

import * as React from "react"
import Link from "next/link"
import { useRouter, useSearchParams } from "next/navigation"
import { formatDistanceToNow } from "date-fns"
import {
  PlusCircle,
  Search,
  Eye,
  Heart,
  MessageCircle,
  Edit,
  Trash2,
  MoreHorizontal,
} from "lucide-react"

import { adminPostApi } from "@/lib/api/vielang-index"
import type { ContentPost } from "@/lib/api/vielang-types"
import { useAdminAuth } from "@/lib/hooks/use-admin-auth"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { toast } from "sonner"

export default function AdminPostsPage() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const { admin, isLoading: authLoading } = useAdminAuth()

  const [posts, setPosts] = React.useState<ContentPost[]>([])
  const [totalPages, setTotalPages] = React.useState(0)
  const [total, setTotal] = React.useState(0)
  const [isLoading, setIsLoading] = React.useState(true)
  const [searchQuery, setSearchQuery] = React.useState("")
  const [statusFilter, setStatusFilter] = React.useState<string>("all")
  const [currentPage, setCurrentPage] = React.useState(1)
  const [deleteDialogOpen, setDeleteDialogOpen] = React.useState(false)
  const [postToDelete, setPostToDelete] = React.useState<number | null>(null)

  React.useEffect(() => {
    const hasLocalAuth =
      typeof window !== "undefined" &&
      localStorage.getItem("admin_auth_token") &&
      localStorage.getItem("admin_user")

    console.log("useEffect triggered:", { authLoading, admin: !!admin, hasLocalAuth })

    if (!authLoading && (admin || hasLocalAuth)) {
      console.log("Calling loadPosts()")
      loadPosts()
    } else {
      console.log("Not calling loadPosts - auth check failed or loading")
    }
  }, [admin, authLoading, statusFilter, currentPage])

  async function loadPosts() {
    try {
      setIsLoading(true)
      const params: any = {
        pageNum: currentPage,
        pageSize: 10,
      }

      if (searchQuery) {
        params.keyword = searchQuery
      }

      if (statusFilter !== "all") {
        // Backend expects boolean: "1" -> true (published), "0" -> false (draft)
        params.status = statusFilter === "1"
      }

      console.log("Loading posts with params:", params)
      const result = await adminPostApi.getList(params)
      console.log("Posts loaded:", result)
      setPosts(result.list || [])
      setTotalPages(result.totalPage || 0)
      setTotal(result.total || 0)
    } catch (error) {
      console.error("Error loading posts:", error)
      toast.error("Failed to load posts")
    } finally {
      setIsLoading(false)
    }
  }

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    setCurrentPage(1)
    loadPosts()
  }

  const handleDeletePost = async () => {
    if (!postToDelete) return

    try {
      await adminPostApi.delete(postToDelete)
      toast.success("Post deleted successfully")
      setDeleteDialogOpen(false)
      setPostToDelete(null)
      loadPosts()
    } catch (error) {
      console.error("Error deleting post:", error)
      toast.error("Failed to delete post")
    }
  }

  const handlePublishPost = async (postId: number) => {
    try {
      await adminPostApi.publish(postId)
      toast.success("Post published successfully")
      loadPosts()
    } catch (error) {
      console.error("Error publishing post:", error)
      toast.error("Failed to publish post")
    }
  }

  const handleToggleFeatured = async (postId: number, currentStatus: boolean) => {
    try {
      await adminPostApi.updateFeatured(
        [postId],
        currentStatus ? 0 : 1
      )
      toast.success(
        currentStatus
          ? "Post removed from featured"
          : "Post marked as featured"
      )
      loadPosts()
    } catch (error) {
      console.error("Error toggling featured status:", error)
      toast.error("Failed to update featured status")
    }
  }

  const handleTogglePinned = async (postId: number, currentStatus: boolean) => {
    try {
      await adminPostApi.updatePinned(
        [postId],
        currentStatus ? 0 : 1
      )
      toast.success(
        currentStatus ? "Post unpinned" : "Post pinned to top"
      )
      loadPosts()
    } catch (error) {
      console.error("Error toggling pinned status:", error)
      toast.error("Failed to update pinned status")
    }
  }

  const getStatusBadge = (status?: boolean) => {
    if (status) {
      return <Badge variant="default">Published</Badge>
    } else {
      return <Badge variant="outline">Draft</Badge>
    }
  }

  console.log("Render state:", { authLoading, isLoading, postsCount: posts.length, admin: !!admin })

  if (authLoading || isLoading) {
    return <LoadingSkeleton />
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Posts</h1>
          <p className="text-muted-foreground">
            Manage all posts ({total} total)
          </p>
        </div>
        <Button asChild>
          <Link href="/admin/posts/new">
            <PlusCircle className="mr-2 h-4 w-4" />
            Create Post
          </Link>
        </Button>
      </div>

      <Card>
        <CardHeader>
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <CardTitle>All Posts</CardTitle>
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
              <form onSubmit={handleSearch} className="flex gap-2">
                <Input
                  type="text"
                  placeholder="Search posts..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full sm:w-64"
                />
                <Button type="submit" variant="outline" size="icon">
                  <Search className="h-4 w-4" />
                </Button>
              </form>
              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-full sm:w-40">
                  <SelectValue placeholder="Filter by status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Status</SelectItem>
                  <SelectItem value="0">Draft</SelectItem>
                  <SelectItem value="1">Published</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {posts.length === 0 ? (
            <div className="text-muted-foreground py-12 text-center">
              No posts found
            </div>
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Title</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Stats</TableHead>
                    <TableHead>Created</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {posts.map((post) => (
                    <TableRow key={post.id}>
                      <TableCell className="font-medium">
                        <div className="space-y-1">
                          <Link
                            href={`/admin/posts/${post.id}/edit`}
                            className="hover:underline line-clamp-1"
                          >
                            {post.title}
                          </Link>
                          <div className="flex gap-1">
                            {post.isFeatured && (
                              <Badge variant="outline" className="text-xs">
                                Featured
                              </Badge>
                            )}
                            {post.isPinned && (
                              <Badge variant="outline" className="text-xs">
                                Pinned
                              </Badge>
                            )}
                          </div>
                        </div>
                      </TableCell>
                      <TableCell>{getStatusBadge(post.status)}</TableCell>
                      <TableCell>
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
                      </TableCell>
                      <TableCell className="text-sm text-muted-foreground">
                        {formatDistanceToNow(new Date(post.createdAt), {
                          addSuffix: true,
                        })}
                      </TableCell>
                      <TableCell className="text-right">
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon">
                              <MoreHorizontal className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuLabel>Actions</DropdownMenuLabel>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem asChild>
                              <Link href={`/admin/posts/${post.id}/edit`}>
                                <Edit className="mr-2 h-4 w-4" />
                                Edit
                              </Link>
                            </DropdownMenuItem>
                            {!post.status && (
                              <DropdownMenuItem
                                onClick={() => handlePublishPost(post.id)}
                              >
                                Publish
                              </DropdownMenuItem>
                            )}
                            <DropdownMenuItem
                              onClick={() =>
                                handleToggleFeatured(post.id, post.isFeatured)
                              }
                            >
                              {post.isFeatured
                                ? "Remove Featured"
                                : "Mark as Featured"}
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onClick={() =>
                                handleTogglePinned(post.id, post.isPinned)
                              }
                            >
                              {post.isPinned ? "Unpin" : "Pin to Top"}
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              className="text-destructive"
                              onClick={() => {
                                setPostToDelete(post.id)
                                setDeleteDialogOpen(true)
                              }}
                            >
                              <Trash2 className="mr-2 h-4 w-4" />
                              Delete
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>

              {/* Pagination */}
              {totalPages > 1 && (
                <div className="mt-4 flex items-center justify-between">
                  <div className="text-sm text-muted-foreground">
                    Page {currentPage} of {totalPages}
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                      disabled={currentPage === 1}
                    >
                      Previous
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() =>
                        setCurrentPage((p) => Math.min(totalPages, p + 1))
                      }
                      disabled={currentPage === totalPages}
                    >
                      Next
                    </Button>
                  </div>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Post</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete this post? This action cannot be
              undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeletePost}>
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}

function LoadingSkeleton() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <Skeleton className="h-9 w-32" />
          <Skeleton className="mt-2 h-5 w-48" />
        </div>
        <Skeleton className="h-10 w-32" />
      </div>

      <Card>
        <CardHeader>
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <Skeleton className="h-6 w-24" />
            <div className="flex gap-2">
              <Skeleton className="h-10 w-64" />
              <Skeleton className="h-10 w-40" />
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-16 w-full" />
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
