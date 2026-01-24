// import { type Store } from "@/db/schema"
// import { type SQL } from "drizzle-orm"
// Removed Stripe - will design payment gateway later

// Removed uploadthing dependency

import type { Icons } from "@/components/icons"

export interface NavItem {
  title: string
  href?: string
  active?: boolean
  disabled?: boolean
  external?: boolean
  icon?: keyof typeof Icons
  label?: string
  description?: string
}

export interface NavItemWithChildren extends NavItem {
  items?: NavItemWithChildren[]
}

export interface FooterItem {
  title: string
  items: {
    title: string
    href: string
    external?: boolean
  }[]
}

export type MainNavItem = NavItemWithChildren

export type SidebarNavItem = NavItemWithChildren

export interface SearchParams {
  [key: string]: string | string[] | undefined
}

// For Next.js 15 compatibility
export type AsyncSearchParams = Promise<SearchParams>
export type AsyncParams = Promise<{ [key: string]: string }>

// Store type
export interface Store {
  id: string
  name: string
  slug: string
  description?: string
  user: string
  plan: "free" | "standard" | "pro"
  plan_ends_at?: string
  cancel_plan_at_end: boolean
  product_limit: number
  tag_limit: number
  variant_limit: number
  created: string
  updated: string
  // Optional computed properties
  orderCount?: number
  customerCount?: number
  productCount?: number
}

// Product type
export interface Product {
  id: string
  name: string
  description?: string
  images: string[]
  category: string
  subcategory?: string
  price: string
  inventory: number
  rating: number
  store: string
  active: boolean
  created: string
  updated: string
}

export interface CheckoutItemSchema {
  productId: string
  name: string
  price: string
  quantity: number
}

export interface Order {
  id: string
  user?: string
  store: string
  items: CheckoutItemSchema[]
  quantity?: number
  amount: string
  status: "pending" | "processing" | "shipped" | "delivered" | "cancelled"
  name: string
  email: string
  address: string
  created: string
  updated: string
  expand?: { store: Store }
}

export interface Category {
  id: string
  name: string
  slug: string
  description?: string
  image?: string
  created: string
  updated: string
}

export interface Subcategory {
  id: string
  name: string
  slug: string
  description?: string
  category: string
  created: string
  updated: string
}

// Removed uploadthing UploadedFile type - now using StoredFile

export interface StoredFile {
  id: string
  name: string
  url: string
}

export interface Option {
  label: string
  value: string
  icon?: React.ComponentType<{ className?: string }>
  withCount?: boolean
}

export interface DataTableFilterField<TData> {
  label: string
  value: keyof TData
  placeholder?: string
  options?: Option[]
}

// Removed Stripe payment status - will implement custom payment later
export type PaymentStatus =
  | "pending"
  | "processing"
  | "succeeded"
  | "failed"
  | "canceled"

export interface Plan {
  id: Store["plan"]
  title: string
  description: string
  features: string[]
  priceId: string // Removed stripe prefix
  limits: {
    stores: number
    products: number
    tags: number
    variants: number
  }
}

export interface PlanWithPrice extends Plan {
  price: string
}

export interface UserPlan extends Plan {
  subscriptionId?: string | null
  currentPeriodEnd?: string | null
  customerId?: string | null
  isSubscribed: boolean
  isCanceled: boolean
  isActive: boolean
}
