# VieLang Architecture Documentation

## 📁 Cấu trúc dự án

Dự án được tổ chức theo **Feature-based Architecture** để dễ bảo trì và mở rộng.

### Tổ chức thư mục

```
vvlog/
├── docs/                    # Project documentation
│   ├── api/                 # API documentation
│   ├── development/         # Development guides
│   └── architecture.md      # This file
│
├── scripts/                 # Build và utility scripts
│   └── build-search.cjs     # Search index builder
│
├── public/                  # Static assets
│   ├── locales/             # i18n translation files
│   ├── images/              # Images và media
│   └── _pagefind/           # Generated search index
│
├── src/
│   ├── app/                 # Next.js App Router
│   │   ├── (main)/          # Main pages group
│   │   ├── docs/            # Documentation pages
│   │   ├── api/             # API routes
│   │   ├── layout.tsx       # Root layout
│   │   └── globals.css      # Global styles
│   │
│   ├── components/          # React components
│   │   ├── common/          # Shared components (Header, Footer, etc.)
│   │   ├── docs/            # Docs-specific components
│   │   └── ui/              # Reusable UI components (shadcn/ui)
│   │
│   ├── features/            # Feature modules (domain-driven)
│   │   ├── auth/            # Authentication feature
│   │   │   ├── api/         # Auth API logic
│   │   │   ├── hooks/       # Auth hooks
│   │   │   ├── types/       # Auth types
│   │   │   └── services/    # Auth services
│   │   ├── comments/        # Comments feature
│   │   │   ├── services/    # Comment services
│   │   │   └── types/       # Comment types
│   │   └── docs/            # Documentation feature
│   │
│   ├── lib/                 # Shared utilities
│   │   ├── utils/           # Utility functions
│   │   ├── constants/       # App constants
│   │   └── hooks/           # Shared hooks
│   │
│   ├── styles/              # Global styles
│   │   ├── globals.css      # Main styles
│   │   └── docs.css         # Docs-specific styles
│   │
│   ├── types/               # Global TypeScript types
│   │
│   ├── config/              # App configuration
│   │
│   └── content/             # Content management
│       └── courses/         # Course content
│
├── package.json
├── tsconfig.json
├── next.config.js
└── biome.json              # Code formatter & linter
```

## 🎯 Design Principles

### 1. Feature-based Architecture

- Code được tổ chức theo domains/features
- Mỗi feature có structure tương tự: services, hooks, types, api
- Dễ dàng tìm kiếm và bảo trì code

### 2. Separation of Concerns

- **Components**: UI presentation only
- **Features**: Business logic và domain-specific code
- **Lib**: Shared utilities và helpers
- **App**: Routing và layouts

### 3. Type Safety

- TypeScript strict mode
- Centralized types trong `src/types/`
- Feature-specific types trong mỗi feature folder

### 4. Code Quality

- **Biome**: Code formatting và linting
- **TypeScript**: Type checking
- **Knip**: Unused exports detection

## 📝 Path Aliases

Dự án sử dụng `@/` alias để import:

```typescript
// ✅ Đúng
import { Button } from "@/components/ui/Button";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { cn } from "@/lib/utils";

// ❌ Sai (không còn dùng)
import { Button } from "~/components/ui/Button";
```

Config trong `tsconfig.json`:

```json
{
  "compilerOptions": {
    "baseUrl": "./src",
    "paths": {
      "@/*": ["./*"]
    }
  }
}
```

## 🔄 Feature Module Pattern

Mỗi feature module theo cấu trúc:

```
features/[feature-name]/
├── api/         # API route handlers
├── hooks/       # React hooks
├── services/    # Business logic
└── types/       # TypeScript types
```

### Example: Auth Feature

```typescript
// services/authClient.ts
export const signIn = async (credentials) => { ... }
export const signOut = async () => { ... }

// hooks/useAuth.ts
export const useAuth = () => {
  const { data: session } = useSession();
  return { session, signIn, signOut };
}

// types/auth.ts
export type User = { ... }
export type Session = { ... }
```

## 🛣️ Routing Structure

Next.js App Router với route groups:

- `(main)/` - Marketing pages (home, about)
- `docs/` - Documentation pages
- `api/` - API endpoints

## 🎨 UI Components

### Component Categories

1. **Common Components** (`components/common/`)

   - Shared across the app
   - Header, Footer, Navigation

2. **UI Components** (`components/ui/`)

   - Reusable UI primitives
   - Based on shadcn/ui
   - Button, Dialog, Dropdown, etc.

3. **Feature Components** (`components/docs/`, etc.)
   - Feature-specific
   - CommentSection, DocNavigation

## 🔧 Configuration Files

- `next.config.js` - Next.js configuration
- `tsconfig.json` - TypeScript configuration
- `biome.json` - Biome formatter & linter
- `tailwind.config.ts` - Tailwind CSS configuration
- `components.json` - shadcn/ui configuration

## 📦 Dependencies Management

### Core Dependencies

- Next.js 16 - React framework
- React 19 - UI library
- TypeScript - Type safety
- TailwindCSS - Styling
- Nextra - Documentation

### Development Tools

- Biome - Linting & Formatting
- Knip - Dead code detection

## 🚀 Development Workflow

1. **Start dev server**: `npm run dev`
2. **Build project**: `npm run build`
3. **Check code**: `npm run check`
4. **Update deps**: `npm run latest`

## 📚 Best Practices

### Imports

- Sử dụng absolute imports với `@/`
- Import types riêng biệt: `import type { ... }`
- Group imports: external → internal → relative

### Components

- One component per file
- Use TypeScript for props
- Prefer function components với hooks

### Styling

- TailwindCSS utility classes
- Use `cn()` utility cho conditional classes
- Keep styles colocated với components

### Type Safety

- Always type function parameters
- Use `type` over `interface`
- Avoid `any` - use `unknown` if needed

## 🔐 Authentication

Authentication sử dụng PocketBase:

- Service: `features/auth/services/authClient.ts`
- API routes: `app/api/auth/`
- Session management với hooks

## 💬 Comments System

- Service: `features/comments/services/comments.ts`
- API routes: `app/api/comments/`
- Real-time updates với PocketBase

## 🌐 Internationalization

- Locale files: `public/locales/`
- Support: en, vi, ko, zh-cn, zh-tw
- Using i18next

## 📖 Documentation

Nextra-based documentation system:

- Content: MDX files
- Search: Pagefind
- Layout: Custom Nextra theme
