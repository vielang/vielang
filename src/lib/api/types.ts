// Base record interface
export interface BaseRecord {
  id: string
  created: string
  updated: string
  createdAt: string
  updatedAt: string
  collectionId: string
  collectionName: string
}

// User interface
export interface User extends BaseRecord {
  email: string
  emailVerified: boolean
  emailVisibility: boolean
  username: string
  verified: boolean
  name?: string
  avatar?: string
}

// Auth state interface
export interface AuthState {
  user: User | null
  token: string | null
  isLoading: boolean
}

// Collection types based on schema
export interface Category extends BaseRecord {
  name: string
  slug: string
  description?: string
  image?: string
}

export interface Subcategory extends BaseRecord {
  name: string
  slug: string
  description?: string
  category: string // relation to Category (PocketBase format)
  categoryId: string // relation to Category
}

export interface Store extends BaseRecord {
  name: string
  slug: string
  description?: string
  user: string // relation to User ID
  userId: string // relation to User
  plan: "free" | "standard" | "pro"
  plan_ends_at?: string
  planEndsAt?: string
  cancel_plan_at_end: boolean
  cancelPlanAtEnd: boolean
  product_limit: number
  productLimit: number
  tag_limit: number
  tagLimit: number
  variant_limit: number
  variantLimit: number
  active: boolean
}

export interface Product extends BaseRecord {
  name: string
  description?: string
  images?: string[] // array of image URLs
  category: string // relation to Category ID
  categoryId: string // relation to Category
  subcategory?: string // relation to Subcategory ID
  subcategoryId?: string // relation to Subcategory
  price: string // decimal as string for precision
  inventory: number
  rating: number
  store: string // relation to Store ID
  storeId: string // relation to Store
  active: boolean
}

export interface Cart {
  id: string
  userId?: string
  sessionId?: string
  created: string
  updated: string
}

export interface CartItem {
  id: string
  cart: string
  product: string
  quantity: number
  created: string
  updated: string
  expand?: {
    product?: {
      id: string
      name: string
      price: string
      images: string[]
    }
  }
}

export interface Address extends BaseRecord {
  line1: string
  line2?: string
  city: string
  state: string
  postalCode: string
  country: string
  userId: string // relation to User
}

export interface Order extends BaseRecord {
  userId?: string // relation to User, optional for guest orders
  storeId: string // relation to Store
  items: CheckoutItem[] // JSON array of order items
  quantity?: number
  amount: string // decimal as string
  status: OrderStatus
  name: string // customer name
  email: string // customer email
  addressId: string // relation to Address
}

export interface Customer extends BaseRecord {
  name?: string
  email: string
  store: string // relation to Store ID
  storeId: string // relation to Store
  total_orders: number
  totalOrders: number
  total_spent: string // decimal as string
  totalSpent: string // decimal as string
}

export interface Notification extends BaseRecord {
  email: string
  token: string
  userId?: string // relation to User
  communication: boolean
  newsletter: boolean
  marketing: boolean
}

// Order related types
export type OrderStatus =
  | "pending"
  | "processing"
  | "shipped"
  | "delivered"
  | "cancelled"

export interface CheckoutItem {
  productId: string
  name: string
  price: string
  quantity: number
  subcategoryId?: string
}

// Response types with expanded relations
export interface ProductWithRelations
  extends Omit<Product, "category" | "subcategory" | "store"> {
  category?: Category | string
  subcategory?: Subcategory | string
  store?: Store | string
  expand?: {
    category?: Category
    subcategory?: Subcategory
    store?: Store
  }
}

export interface OrderWithRelations extends Order {
  user?: User
  store?: Store
  address?: Address
  expand?: {
    user?: User
    store?: Store
    address?: Address
  }
}

export interface StoreWithRelations extends Omit<Store, "user"> {
  user?: User | string
  expand?: {
    user?: User
  }
}

// API Query parameters
export interface GetProductsParams {
  page?: number
  perPage?: number
  sort?: string
  subcategories?: string
  priceRange?: string
  storeIds?: string
  active?: boolean
  search?: string
}

export interface AuthCredentials {
  email: string
  password: string
}

export interface RegisterCredentials extends AuthCredentials {
  username: string
  name?: string
}

export interface AuthResponse {
  user: User
  token: string
}
