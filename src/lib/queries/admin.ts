import "server-only"

import { unstable_noStore as noStore } from "next/cache"

import {
  adminCategoryApi,
  adminPostApi,
  adminCommentApi,
} from "@/lib/api/vielang-index"

/**
 * Admin queries using Vielang Admin API (Social Platform)
 */

export async function getAdminStats() {
  noStore()
  try {
    // Get post count
    const postsResult = await adminPostApi.getList({
      pageNum: 1,
      pageSize: 1,
    })

    // Get comment count
    const commentsResult = await adminCommentApi.getList({
      pageNum: 1,
      pageSize: 1,
    })

    return {
      totalPosts: postsResult.total || 0,
      totalComments: commentsResult.total || 0,
      totalLikes: 0, // TODO: Calculate from posts
      totalViews: 0, // TODO: Calculate from posts
    }
  } catch (error) {
    console.error("Error fetching admin stats:", error)
    return {
      totalPosts: 0,
      totalComments: 0,
      totalLikes: 0,
      totalViews: 0,
    }
  }
}

export async function getRecentPosts(
  pageNum: number = 1,
  pageSize: number = 10
) {
  noStore()
  try {
    const result = await adminPostApi.getList({
      pageNum,
      pageSize,
      status: 1, // Published posts
    })

    return result
  } catch (error) {
    console.error("Error fetching recent posts:", error)
    return {
      pageNum: 1,
      pageSize: 10,
      totalPage: 0,
      total: 0,
      list: [],
    }
  }
}

export async function getRecentComments(
  pageNum: number = 1,
  pageSize: number = 10
) {
  noStore()
  try {
    const result = await adminCommentApi.getList({
      pageNum,
      pageSize,
    })

    return result
  } catch (error) {
    console.error("Error fetching recent comments:", error)
    return {
      pageNum: 1,
      pageSize: 10,
      totalPage: 0,
      total: 0,
      list: [],
    }
  }
}
