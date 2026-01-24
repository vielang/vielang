// Vielang Backend API Types
// Based on Social Media Platform Schema (social_platform database)
// Database Schema: /mall/document/sql/db_schema.sql

// ============================================
// 1. AUTHENTICATION & USER MANAGEMENT
// ============================================

// Admin Users (Content Creators & Platform Managers)
export interface SysAdmin {
  id: number
  username: string
  password?: string // Only for requests
  email: string
  nickname?: string
  avatar?: string
  phone?: string
  status: number // 0: Disabled, 1: Active
  lastLoginTime?: string
  lastLoginIp?: string
  createdAt: string
  updatedAt: string
}

// Admin Info Response (from /info endpoint)
export interface SysAdminInfo {
  username: string
  icon?: string // avatar
  roles?: string[]
  menus?: SysPermission[]
}

// Members (Platform Users - Read & Interact Only)
export interface SysMember {
  id: number
  username: string
  password?: string // Only for requests
  email: string
  nickname?: string
  avatar?: string
  phone?: string
  bio?: string
  gender?: number // 0: Unknown, 1: Male, 2: Female, 3: Other
  birthday?: string
  location?: string
  status: number // 0: Disabled, 1: Active
  lastLoginTime?: string
  lastLoginIp?: string
  createdAt: string
  updatedAt: string
}

// Roles (For RBAC System)
export interface SysRole {
  id: number
  name: string // e.g., Super Admin, Content Manager
  code: string // e.g., ROLE_SUPER_ADMIN
  description?: string
  status: number // 0: Disabled, 1: Active
  sort: number
  createdAt: string
  updatedAt: string
}

// Permissions/Resources
export interface SysPermission {
  id: number
  name: string
  code: string // e.g., post:create
  type: number // 1: Menu, 2: Button, 3: API
  parentId?: number
  path?: string // URL path or route
  method?: string // HTTP method (GET, POST, etc.)
  icon?: string
  sort: number
  status: number // 0: Disabled, 1: Active
  createdAt: string
  updatedAt: string
}

// ============================================
// 2. CONTENT MANAGEMENT
// ============================================

// Post Categories
export interface ContentCategory {
  id: number
  parentId?: number // 0 for root
  name: string
  slug: string
  description?: string
  icon?: string
  coverImage?: string
  level: number // 0: Root, 1: Sub-category
  sort: number
  postCount: number
  isNav: number | boolean // 0: Hide from nav, 1: Show in nav (or boolean from backend)
  status: number | boolean // 0: Disabled, 1: Active (or boolean from backend)
  createdAt: string
  updatedAt: string
  children?: ContentCategory[] // For nested categories
}

// Posts (Created by Admins Only)
export interface ContentPost {
  id: number
  adminId: number // Author (Admin user ID)
  categoryId?: number
  title: string
  slug: string
  summary?: string
  content: string // HTML/Markdown
  coverImage?: string
  status: boolean // Backend returns boolean: true = Published, false = Draft
  visibility: boolean // Backend returns boolean: true = Public, false = Private
  isFeatured: boolean // Backend returns boolean: true = Featured, false = Normal
  isPinned: boolean // Backend returns boolean: true = Pinned, false = Normal
  allowComment: boolean // Backend returns boolean: true = Enabled, false = Disabled
  viewsCount: number
  likesCount: number
  commentsCount: number
  publishedAt?: string
  createdAt: string
  updatedAt: string
  // Extended fields
  author?: SysAdmin
  category?: ContentCategory
  tags?: ContentTag[]
  media?: ContentPostMedia[]
}

// Post Media Attachments
export interface ContentPostMedia {
  id: number
  postId: number
  type: number // 1: Image, 2: Video, 3: Audio, 4: Document
  url: string
  thumbnailUrl?: string
  title?: string
  size?: number // File size in bytes
  duration?: number // Duration in seconds (for video/audio)
  sort: number
  createdAt: string
}

// Tags/Hashtags
export interface ContentTag {
  id: number
  name: string
  slug: string
  usageCount: number
  createdAt: string
  updatedAt: string
}

// ============================================
// 3. SOCIAL INTERACTIONS
// ============================================

// Comments (Members can comment on posts)
export interface SocialComment {
  id: number
  postId: number
  memberId: number
  parentId?: number // 0 for root comments
  content: string
  likesCount: number
  repliesCount: number
  status: number // 0: Hidden, 1: Visible, 2: Reported
  createdAt: string
  updatedAt: string
  // Extended fields
  member?: SysMember
  replies?: SocialComment[]
}

// Likes (Members can like posts & comments)
export interface SocialLike {
  id: number
  memberId: number
  targetType: number // 1: Post, 2: Comment
  targetId: number // post_id or comment_id
  createdAt: string
}

// ============================================
// 4. NOTIFICATIONS
// ============================================

export interface SysNotification {
  id: number
  memberId: number
  senderType?: number // 0: System, 1: Member, 2: Admin
  senderId?: number
  type: number // 1: Like, 2: Comment, 3: Mention, 4: System
  title?: string
  content?: string
  targetType?: number // 1: Post, 2: Comment
  targetId?: number
  isRead: number // 0: Unread, 1: Read
  createdAt: string
}

// ============================================
// 5. SYSTEM LOGS
// ============================================

export interface SysAdminLog {
  id: number
  adminId: number
  action: string
  module: string
  description?: string
  ip?: string
  userAgent?: string
  createdAt: string
}

export interface SysMemberLog {
  id: number
  memberId: number
  action: string // login, logout
  ip?: string
  location?: string
  device?: string
  userAgent?: string
  createdAt: string
}

// ============================================
// 6. SYSTEM SETTINGS
// ============================================

export interface SysConfig {
  id: number
  key: string
  value?: string
  group: string // e.g., general, content
  type: string // string, number, boolean, json
  description?: string
  isPublic: number // 0: Backend only, 1: Public API accessible
  sort: number
  createdAt: string
  updatedAt: string
}

// ============================================
// REQUEST/RESPONSE TYPES
// ============================================

// Authentication
export interface LoginRequest {
  username: string
  password: string
}

export interface RegisterRequest {
  username: string
  password: string
  email: string
  phone?: string
}

export interface LoginResponse {
  token: string
  tokenHead: string // Usually "Bearer"
  user?: SysMember | SysAdmin
}

// Post Creation/Update
export interface CreatePostRequest {
  categoryId?: number
  title: string
  slug: string
  summary?: string
  content: string
  coverImage?: string
  status?: number
  visibility?: number
  isFeatured?: number
  isPinned?: number
  allowComment?: number
  tags?: number[] // Tag IDs
}

export interface UpdatePostRequest extends Partial<CreatePostRequest> {
  id: number
}

// Comment Creation
export interface CreateCommentRequest {
  postId: number
  parentId?: number
  content: string
}

// Search/Filter Params
export interface PostSearchParams {
  keyword?: string
  categoryId?: number
  tagId?: number
  status?: number
  visibility?: number
  isFeatured?: number
  pageNum?: number
  pageSize?: number
  sortBy?: string // latest, trending, popular (default: latest)
}

export interface CommentSearchParams {
  postId?: number
  memberId?: number
  status?: number
  pageNum?: number
  pageSize?: number
}

// ============================================
// COMMON RESPONSE WRAPPER
// ============================================

export interface CommonResult<T = any> {
  code: number // 200: success, 404: not found, 500: error
  message: string | null
  data: T
}

export interface CommonPage<T = any> {
  pageNum: number
  pageSize: number
  totalPage: number
  total: number
  list: T[]
}

// Helper types
export type ApiResponse<T> = CommonResult<T>
export type PagedApiResponse<T> = CommonResult<CommonPage<T>>

// Query Params (Admin)
export interface AdminQueryParam {
  keyword?: string
  status?: number
  pageNum?: number
  pageSize?: number
}

export interface RoleQueryParam {
  keyword?: string
  status?: number
  pageNum?: number
  pageSize?: number
}

export interface CategoryQueryParam {
  keyword?: string
  parentId?: number
  status?: number
  pageNum?: number
  pageSize?: number
}
