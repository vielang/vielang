# Vielang Web - Social Media Platform Frontend Template

A modern, production-ready Next.js template for building social media platforms with Spring Boot backend integration.

> **Note:** This is a template project adapted from mall-web for social media platform use cases.

## 📋 Overview

**Vielang Web** is a comprehensive frontend template designed to work seamlessly with the **Vielang Social Media Platform Backend** (Spring Boot). This template provides a complete foundation for building social media applications with features like:

- 👥 **Dual User System**: Admins (content creators) and Members (readers/interactors)
- 📝 **Content Management**: Posts, categories, tags, and media attachments
- 💬 **Social Interactions**: Comments, likes, and notifications
- 🔐 **RBAC**: Role-based access control with permissions
- 📱 **Responsive Design**: Mobile-first approach with Tailwind CSS
- ⚡ **Modern Stack**: Next.js 16, React 19, TypeScript, shadcn/ui

## 🗄️ Database Schema

The backend uses a clean social media platform schema with **17 tables**:

### 1. Authentication & User Management (6 tables)
- `sys_admin` - Admin users (content creators)
- `sys_member` - Platform users (readers/interactors)
- `sys_role` - User roles
- `sys_permission` - Permissions/resources
- `sys_admin_role` - Admin-role relationship
- `sys_role_permission` - Role-permission relationship

### 2. Content Management (5 tables)
- `content_category` - Post categories (hierarchical)
- `content_post` - Posts (admin-created content)
- `content_post_media` - Post media attachments
- `content_tag` - Tags/hashtags
- `content_post_tag` - Post-tag relationship

### 3. Social Interactions (2 tables)
- `social_comment` - Comments on posts
- `social_like` - Likes on posts & comments

### 4. Notifications (1 table)
- `sys_notification` - User notifications

### 5. System Logs (2 tables)
- `sys_admin_log` - Admin activity logs
- `sys_member_log` - Member login logs

### 6. System Settings (1 table)
- `sys_config` - System configuration

📄 **Full Schema**: `/mall/document/sql/db_schema.sql`

## 🛠️ Tech Stack

### Frontend Framework
- **Next.js 16.1.1** - App Router with Server Components
- **React 19.2.3** - Latest React with new features
- **TypeScript 5.9.3** - Type safety

### UI/Styling
- **Tailwind CSS 4.1.18** - Utility-first CSS
- **shadcn/ui** - High-quality React components
- **Radix UI** - Headless UI primitives
- **Lucide React** - Beautiful icons
- **Framer Motion** - Smooth animations

### State & Data Management
- **Zustand 5.0.9** - Lightweight state management
- **React Hook Form 7.69.0** - Form handling
- **Zod 3.24.1** - Schema validation
- **Axios 1.13.2** - HTTP client

### Development Tools
- **ESLint** - Code linting
- **Prettier** - Code formatting
- **Playwright** - E2E testing
- **pnpm 9.5.0** - Package manager

## 📁 Project Structure

```
vielang-web/
├── src/
│   ├── app/                          # Next.js App Router
│   │   ├── (portal)/                # User-facing pages
│   │   │   ├── page.tsx             # Home page
│   │   │   ├── posts/               # Post listing
│   │   │   ├── post/[postId]/       # Post detail
│   │   │   └── profile/             # User profile
│   │   ├── (admin)/                 # Admin dashboard
│   │   │   ├── admin/               # Dashboard pages
│   │   │   ├── admin/posts/         # Post management
│   │   │   ├── admin/categories/    # Category management
│   │   │   └── admin/members/       # Member management
│   │   ├── (auth)/                  # Authentication pages
│   │   └── api/                     # API routes
│   ├── components/                   # Reusable components
│   │   ├── ui/                      # shadcn/ui components
│   │   ├── vielang/                 # Custom components
│   │   └── layouts/                 # Layout components
│   ├── lib/                         # Utilities & API clients
│   │   ├── api/                     # API client implementations
│   │   │   ├── vielang-index.ts    # Main API export
│   │   │   ├── vielang-portal.ts   # Portal API client
│   │   │   ├── vielang-admin.ts    # Admin API client
│   │   │   ├── vielang-auth.ts     # Auth API
│   │   │   └── vielang-types.ts    # TypeScript types
│   │   ├── hooks/                   # Custom React hooks
│   │   └── utils/                   # Helper functions
│   ├── features/                    # Feature modules
│   ├── types/                       # Global TypeScript types
│   └── env.js                       # Environment validation
├── public/                          # Static assets
├── .env                            # Environment variables
├── .env.example                    # Environment template
├── package.json                    # Dependencies
├── tsconfig.json                   # TypeScript config
├── next.config.js                  # Next.js config
└── tailwind.config.ts              # Tailwind config
```

## 🚀 Getting Started

### Prerequisites

- Node.js 18+ and pnpm 9.5.0+
- MySQL database
- Vielang Backend running (Spring Boot)

### Installation

1. **Navigate to template directory**
   ```bash
   cd spring-boot-template/vielang-web
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Set up environment variables**
   ```bash
   cp .env.example .env
   ```

   Update `.env` with your backend URLs:
   ```env
   NEXT_PUBLIC_APP_URL="http://localhost:3000"

   # Backend API URLs
   NEXT_PUBLIC_VIELANG_PORTAL_API_URL="http://localhost:8085"
   NEXT_PUBLIC_VIELANG_ADMIN_API_URL="http://localhost:8080"

   # Optional services
   RESEND_API_KEY="your_resend_api_key"
   EMAIL_FROM_ADDRESS="noreply@yourdomain.com"
   UPLOADTHING_SECRET="your_uploadthing_secret"
   UPLOADTHING_APP_ID="your_uploadthing_app_id"
   ```

4. **Start the development server**
   ```bash
   pnpm dev
   ```

   Open [http://localhost:3000](http://localhost:3000)

### Backend Setup

Ensure your Spring Boot backend is running:

- **Portal API**: `http://localhost:8085/api/v1/portal`
- **Admin API**: `http://localhost:8080/api/v1/admin`

Database schema is located at: `/mall/document/sql/db_schema.sql`

## 📝 API Integration

### Portal API (User-Facing)

```typescript
import { portalApi } from '@/lib/api/vielang-index'

// Get posts
const posts = await portalApi.post.getList({ pageNum: 0, pageSize: 10 })

// Get post detail
const post = await portalApi.post.getDetail(postId)

// Add comment
const comment = await portalApi.comment.create({
  postId: 1,
  content: 'Great post!'
})

// Like a post
await portalApi.like.toggle({ targetType: 1, targetId: postId })
```

### Admin API (Dashboard)

```typescript
import { adminApi } from '@/lib/api/vielang-index'

// Create post
const newPost = await adminApi.post.create({
  title: 'My First Post',
  content: 'Post content here...',
  categoryId: 1,
  status: 1 // Published
})

// Update post
await adminApi.post.update(postId, { title: 'Updated Title' })

// Delete post
await adminApi.post.delete(postId)
```

### TypeScript Types

All types are available from `vielang-types.ts`:

```typescript
import type {
  SysMember,
  SysAdmin,
  ContentPost,
  ContentCategory,
  SocialComment,
  SocialLike,
  SysNotification
} from '@/lib/api/vielang-index'
```

## 🎨 Customization Guide

### 1. Update Project Name

Replace `vielang` with your project name:

```bash
# 1. Update package.json name field
# 2. Rename API files: vielang-* to yourname-*
# 3. Update imports: vielang-index to yourname-index
# 4. Update environment variable prefixes
# 5. Search and replace "vielang" throughout codebase
```

### 2. Customize Theme

Edit `tailwind.config.ts`:

```typescript
export default {
  theme: {
    extend: {
      colors: {
        primary: { /* your colors */ },
        secondary: { /* your colors */ }
      }
    }
  }
}
```

### 3. Add New Features

Follow the modular structure:

```
src/features/your-feature/
├── components/
├── hooks/
├── types/
└── utils/
```

## 📦 Available Scripts

### Development

```bash
pnpm dev          # Start dev server
pnpm lint         # Run ESLint
pnpm typecheck    # Type checking
pnpm format:write # Format code
pnpm check        # Run all checks
```

### Production Build

```bash
pnpm build        # Build for production
pnpm start        # Start production server
```

### Testing

```bash
pnpm test              # Run all tests
pnpm test:ui           # Open Playwright UI
pnpm test:chromium     # Test in Chromium
pnpm test:firefox      # Test in Firefox
pnpm test:webkit       # Test in WebKit
```

## 🌐 Deploy to Vercel

1. Push your code to GitHub
2. Import project to Vercel
3. Set environment variables in Vercel Dashboard:
   - `NEXT_PUBLIC_VIELANG_PORTAL_API_URL`
   - `NEXT_PUBLIC_VIELANG_ADMIN_API_URL`
   - `RESEND_API_KEY`
   - etc.
4. Deploy

## 📚 Key Features

### Authentication
- JWT token-based authentication
- Separate auth for Portal (members) and Admin
- Token refresh mechanism
- Protected routes with middleware

### Content Management
- Rich text editor (TipTap)
- Image/media upload support
- Category management (hierarchical)
- Tag/hashtag system
- Draft/Published/Archived states
- Featured & Pinned posts

### Social Features
- Comment system with nested replies
- Like system for posts & comments
- Real-time notifications
- User profiles with bio
- Activity tracking

### Admin Dashboard
- Post CRUD operations
- Category management
- Member management
- Role & Permission management
- Activity logs & analytics

## 🔒 Security

- Input validation with Zod schemas
- XSS protection via React
- CSRF token support
- Secure headers (CSP, HSTS, X-Frame-Options)
- Environment variable validation at build time
- JWT token management

## 📖 Documentation Links

- **Database Schema**: `/mall/document/sql/db_schema.sql`
- **API Types**: `src/lib/api/vielang-types.ts`
- **Next.js Docs**: https://nextjs.org/docs
- **shadcn/ui**: https://ui.shadcn.com
- **Tailwind CSS**: https://tailwindcss.com/docs

## 🔄 Migration from E-commerce

This template was adapted from an e-commerce platform (mall-web) to a social media platform. Key changes:

- ✅ Renamed from `mall` to `vielang`
- ✅ Updated types from Products/Cart/Orders to Posts/Comments/Likes
- ✅ Changed business logic to match social platform schema
- ✅ Updated API clients and endpoints
- ✅ Modified components for social features

## 🆘 Support & Issues

This is a template project provided as-is. For:

- **Backend integration**: Refer to Spring Boot backend documentation
- **Frontend issues**: Check Next.js, shadcn/ui, and Tailwind documentation
- **Type issues**: Review `vielang-types.ts` for available types

## 📄 License

This template is provided for use in your projects. Customize as needed!

---

**Built with ❤️ using Next.js 16, React 19, TypeScript, and Tailwind CSS**

**Database Schema**: Social Media Platform (17 tables)
**Backend**: Spring Boot + MyBatis
**Frontend**: Next.js App Router + Server Components
