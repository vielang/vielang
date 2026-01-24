"use client"

import { useState, useEffect } from "react"
import { Heart } from "lucide-react"
import { useRouter } from "next/navigation"

import { Button } from "@/components/ui/button"
import { portalSocialApi } from "@/lib/api/vielang-index"
import { useSecureAuth } from "@/lib/hooks/use-secure-auth"
import { toast } from "sonner"

interface LikeButtonProps {
  postId: number
  initialLikesCount: number
}

export function LikeButton({ postId, initialLikesCount }: LikeButtonProps) {
  const router = useRouter()
  const { user, isAuthenticated } = useSecureAuth()
  const [isLiked, setIsLiked] = useState(false)
  const [likesCount, setLikesCount] = useState(initialLikesCount)
  const [isLoading, setIsLoading] = useState(false)
  const [isCheckingLike, setIsCheckingLike] = useState(true)

  // Check if user has liked this post
  useEffect(() => {
    const checkLikeStatus = async () => {
      if (!isAuthenticated) {
        setIsCheckingLike(false)
        return
      }

      try {
        const liked = await portalSocialApi.checkLikeStatus(postId)
        setIsLiked(liked)
      } catch (error) {
        console.error("Error checking like status:", error)
      } finally {
        setIsCheckingLike(false)
      }
    }

    checkLikeStatus()
  }, [postId, isAuthenticated])

  const handleLike = async () => {
    if (!isAuthenticated) {
      toast.error("Please login to like posts")
      router.push("/signin")
      return
    }

    setIsLoading(true)

    try {
      if (isLiked) {
        await portalSocialApi.unlikePost(postId)
        setIsLiked(false)
        setLikesCount((prev) => Math.max(0, prev - 1))
        toast.success("Post unliked")
      } else {
        await portalSocialApi.likePost(postId)
        setIsLiked(true)
        setLikesCount((prev) => prev + 1)
        toast.success("Post liked")
      }
    } catch (error) {
      console.error("Error toggling like:", error)
      toast.error("Failed to update like status")
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <Button
      variant={isLiked ? "default" : "outline"}
      size="sm"
      onClick={handleLike}
      disabled={isLoading || isCheckingLike}
      className="gap-2"
    >
      <Heart
        className={`h-4 w-4 ${isLiked ? "fill-current" : ""}`}
      />
      <span>{likesCount}</span>
    </Button>
  )
}
