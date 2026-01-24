"use client"

import { useState } from "react"
import { formatDistanceToNow } from "date-fns"
import { MessageCircle, User, Trash2 } from "lucide-react"
import { useRouter } from "next/navigation"

import { portalSocialApi } from "@/lib/api/vielang-index"
import type { SocialComment } from "@/lib/api/vielang-types"
import { useSecureAuth } from "@/lib/hooks/use-secure-auth"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/card"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import { toast } from "sonner"

interface CommentSectionProps {
  postId: number
  initialComments: SocialComment[]
}

export function CommentSection({ postId, initialComments }: CommentSectionProps) {
  const router = useRouter()
  const { user, isAuthenticated } = useSecureAuth()
  const [comments, setComments] = useState<SocialComment[]>(
    Array.isArray(initialComments) ? initialComments : []
  )
  const [newComment, setNewComment] = useState("")
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmitComment = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!isAuthenticated) {
      toast.error("Please login to comment")
      router.push("/signin")
      return
    }

    if (!newComment.trim()) {
      toast.error("Please enter a comment")
      return
    }

    setIsSubmitting(true)

    try {
      const result = await portalSocialApi.addComment(postId, newComment.trim())

      // Add the new comment to the list
      if (result.code === 200 && result.data) {
        setComments((prev) => [result.data, ...prev])
        setNewComment("")
        toast.success("Comment added successfully")
      } else {
        toast.error(result.message || "Failed to add comment")
      }

      // Refresh to update comment count
      router.refresh()
    } catch (error) {
      console.error("Error adding comment:", error)
      toast.error("Failed to add comment")
    } finally {
      setIsSubmitting(false)
    }
  }

  const handleDeleteComment = async (commentId: number) => {
    try {
      await portalSocialApi.deleteComment(commentId)
      setComments((prev) => prev.filter((c) => c.id !== commentId))
      toast.success("Comment deleted")

      // Refresh to update comment count
      router.refresh()
    } catch (error) {
      console.error("Error deleting comment:", error)
      toast.error("Failed to delete comment")
    }
  }

  return (
    <div>
      <h2 className="mb-6 text-2xl font-bold">
        Comments ({comments.length})
      </h2>

      {/* Comment Form */}
      {isAuthenticated ? (
        <form onSubmit={handleSubmitComment} className="mb-8">
          <Card>
            <CardContent className="pt-6">
              <Textarea
                value={newComment}
                onChange={(e) => setNewComment(e.target.value)}
                placeholder="Write a comment..."
                className="min-h-[100px] resize-none"
                disabled={isSubmitting}
              />
            </CardContent>
            <CardFooter className="flex justify-between">
              <p className="text-sm text-muted-foreground">
                {newComment.length} characters
              </p>
              <Button type="submit" disabled={isSubmitting || !newComment.trim()}>
                {isSubmitting ? "Posting..." : "Post Comment"}
              </Button>
            </CardFooter>
          </Card>
        </form>
      ) : (
        <Card className="mb-8">
          <CardContent className="pt-6 text-center">
            <MessageCircle className="h-12 w-12 mx-auto mb-3 opacity-50" />
            <p className="text-muted-foreground mb-4">
              Please login to comment on this post
            </p>
            <Button onClick={() => router.push("/signin")}>
              Login to Comment
            </Button>
          </CardContent>
        </Card>
      )}

      {/* Comments List */}
      {comments.length === 0 ? (
        <div className="text-center py-8 text-muted-foreground">
          <MessageCircle className="h-12 w-12 mx-auto mb-2 opacity-50" />
          <p>No comments yet. Be the first to comment!</p>
        </div>
      ) : (
        <div className="space-y-4">
          {comments.map((comment) => (
            <CommentCard
              key={comment.id}
              comment={comment}
              currentUserId={user?.id}
              onDelete={handleDeleteComment}
            />
          ))}
        </div>
      )}
    </div>
  )
}

interface CommentCardProps {
  comment: SocialComment
  currentUserId?: number
  onDelete: (commentId: number) => void
}

function CommentCard({ comment, currentUserId, onDelete }: CommentCardProps) {
  const isOwner = currentUserId === comment.memberId

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <Avatar className="h-8 w-8">
              <AvatarImage src={comment.member?.avatar} />
              <AvatarFallback>
                <User className="h-4 w-4" />
              </AvatarFallback>
            </Avatar>
            <div>
              <p className="font-medium text-sm">
                {comment.member?.nickname || comment.member?.username || "User"}
              </p>
              <p className="text-xs text-muted-foreground">
                {formatDistanceToNow(new Date(comment.createdAt), {
                  addSuffix: true,
                })}
              </p>
            </div>
          </div>

          {isOwner && (
            <AlertDialog>
              <AlertDialogTrigger asChild>
                <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                  <Trash2 className="h-4 w-4" />
                </Button>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>Delete Comment</AlertDialogTitle>
                  <AlertDialogDescription>
                    Are you sure you want to delete this comment? This action cannot be undone.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel>Cancel</AlertDialogCancel>
                  <AlertDialogAction onClick={() => onDelete(comment.id)}>
                    Delete
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          )}
        </div>
      </CardHeader>
      <CardContent>
        <p className="text-sm whitespace-pre-wrap">{comment.content}</p>
      </CardContent>
    </Card>
  )
}
