"use client"

import * as React from "react"
import Link from "next/link"
import { useRouter, useSearchParams } from "next/navigation"
import { formatDistanceToNow } from "date-fns"
import {
  Search,
  Eye,
  EyeOff,
  Trash2,
  MoreHorizontal,
  MessageCircle,
  CheckCircle,
  XCircle,
  AlertTriangle,
} from "lucide-react"

import { adminCommentApi } from "@/lib/api/vielang-index"
import type { SocialComment } from "@/lib/api/vielang-types"
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

export default function AdminCommentsPage() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const { admin, isLoading: authLoading } = useAdminAuth()

  const [comments, setComments] = React.useState<SocialComment[]>([])
  const [totalPages, setTotalPages] = React.useState(0)
  const [total, setTotal] = React.useState(0)
  const [isLoading, setIsLoading] = React.useState(true)
  const [statusFilter, setStatusFilter] = React.useState<string>("all")
  const [currentPage, setCurrentPage] = React.useState(1)
  const [deleteDialogOpen, setDeleteDialogOpen] = React.useState(false)
  const [commentToDelete, setCommentToDelete] = React.useState<number | null>(null)

  React.useEffect(() => {
    const hasLocalAuth =
      typeof window !== "undefined" &&
      localStorage.getItem("admin_auth_token") &&
      localStorage.getItem("admin_user")

    if (!authLoading && (admin || hasLocalAuth)) {
      loadComments()
    }
  }, [admin, authLoading, statusFilter, currentPage])

  async function loadComments() {
    try {
      setIsLoading(true)
      const params: any = {
        pageNum: currentPage,
        pageSize: 10,
      }

      if (statusFilter !== "all") {
        params.status = Number(statusFilter)
      }

      const result = await adminCommentApi.getList(params)
      setComments(result.list || [])
      setTotalPages(result.totalPage || 0)
      setTotal(result.total || 0)
    } catch (error) {
      console.error("Error loading comments:", error)
      toast.error("Failed to load comments")
    } finally {
      setIsLoading(false)
    }
  }

  const handleDeleteComment = async () => {
    if (!commentToDelete) return

    try {
      await adminCommentApi.delete(commentToDelete)
      toast.success("Comment deleted successfully")
      setDeleteDialogOpen(false)
      setCommentToDelete(null)
      loadComments()
    } catch (error) {
      console.error("Error deleting comment:", error)
      toast.error("Failed to delete comment")
    }
  }

  const handleApproveComment = async (commentId: number) => {
    try {
      await adminCommentApi.approve(commentId)
      toast.success("Comment approved")
      loadComments()
    } catch (error) {
      console.error("Error approving comment:", error)
      toast.error("Failed to approve comment")
    }
  }

  const handleHideComment = async (commentId: number) => {
    try {
      await adminCommentApi.hide(commentId)
      toast.success("Comment hidden")
      loadComments()
    } catch (error) {
      console.error("Error hiding comment:", error)
      toast.error("Failed to hide comment")
    }
  }

  const handleMarkAsReported = async (commentId: number) => {
    try {
      await adminCommentApi.markAsReported(commentId)
      toast.success("Comment marked as reported")
      loadComments()
    } catch (error) {
      console.error("Error marking comment:", error)
      toast.error("Failed to mark comment")
    }
  }

  const getStatusBadge = (status?: number) => {
    switch (status) {
      case 0:
        return (
          <Badge variant="secondary" className="gap-1">
            <EyeOff className="h-3 w-3" />
            Hidden
          </Badge>
        )
      case 1:
        return (
          <Badge variant="default" className="gap-1">
            <CheckCircle className="h-3 w-3" />
            Visible
          </Badge>
        )
      case 2:
        return (
          <Badge variant="destructive" className="gap-1">
            <AlertTriangle className="h-3 w-3" />
            Reported
          </Badge>
        )
      default:
        return <Badge variant="outline">Unknown</Badge>
    }
  }

  if (authLoading || isLoading) {
    return <LoadingSkeleton />
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Comments</h1>
          <p className="text-muted-foreground">
            Moderate user comments ({total} total)
          </p>
        </div>
      </div>

      <Card>
        <CardHeader>
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <CardTitle>All Comments</CardTitle>
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-full sm:w-40">
                  <SelectValue placeholder="Filter by status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Status</SelectItem>
                  <SelectItem value="0">Hidden</SelectItem>
                  <SelectItem value="1">Visible</SelectItem>
                  <SelectItem value="2">Reported</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {comments.length === 0 ? (
            <div className="text-muted-foreground py-12 text-center">
              No comments found
            </div>
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Comment</TableHead>
                    <TableHead>Author</TableHead>
                    <TableHead>Post</TableHead>
                    <TableHead>Stats</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Date</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {comments.map((comment) => (
                    <TableRow key={comment.id}>
                      <TableCell className="max-w-md">
                        <div className="line-clamp-2">
                          {comment.content}
                        </div>
                      </TableCell>
                      <TableCell>
                        {comment.member ? (
                          <div>
                            <div className="font-medium">
                              {comment.member.nickname || comment.member.username}
                            </div>
                            <div className="text-xs text-muted-foreground">
                              {comment.member.email}
                            </div>
                          </div>
                        ) : (
                          <span className="text-muted-foreground">Unknown</span>
                        )}
                      </TableCell>
                      <TableCell>
                        <span className="text-sm text-muted-foreground">
                          Post #{comment.postId}
                        </span>
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-3 text-xs text-muted-foreground">
                          <span className="flex items-center gap-1">
                            <Eye className="h-3 w-3" />
                            {comment.likesCount || 0}
                          </span>
                          <span className="flex items-center gap-1">
                            <MessageCircle className="h-3 w-3" />
                            {comment.repliesCount || 0}
                          </span>
                        </div>
                      </TableCell>
                      <TableCell>{getStatusBadge(comment.status)}</TableCell>
                      <TableCell className="text-sm text-muted-foreground">
                        {formatDistanceToNow(new Date(comment.createdAt), {
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
                            {comment.status !== 1 && (
                              <DropdownMenuItem
                                onClick={() => handleApproveComment(comment.id)}
                              >
                                <CheckCircle className="mr-2 h-4 w-4" />
                                Approve
                              </DropdownMenuItem>
                            )}
                            {comment.status !== 0 && (
                              <DropdownMenuItem
                                onClick={() => handleHideComment(comment.id)}
                              >
                                <EyeOff className="mr-2 h-4 w-4" />
                                Hide
                              </DropdownMenuItem>
                            )}
                            {comment.status !== 2 && (
                              <DropdownMenuItem
                                onClick={() => handleMarkAsReported(comment.id)}
                              >
                                <AlertTriangle className="mr-2 h-4 w-4" />
                                Mark as Reported
                              </DropdownMenuItem>
                            )}
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              className="text-destructive"
                              onClick={() => {
                                setCommentToDelete(comment.id)
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
            <AlertDialogTitle>Delete Comment</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete this comment? This action cannot be
              undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteComment}>
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
      </div>

      <Card>
        <CardHeader>
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <Skeleton className="h-6 w-32" />
            <Skeleton className="h-10 w-40" />
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
