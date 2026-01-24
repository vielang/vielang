/**
 * Application Routes Constants
 */

// Portal (Customer) routes
export const PORTAL_ROUTES = {
  HOME: "/",
  PRODUCTS: "/products",
  PRODUCT_DETAIL: (id: string | number) => `/products/${id}`,
  CART: "/cart",
  CHECKOUT: "/checkout",
  ORDERS: "/orders",
  ORDER_DETAIL: (id: string | number) => `/orders/${id}`,
  ACCOUNT: "/account",
  ACCOUNT_PROFILE: "/account/profile",
  ACCOUNT_ADDRESSES: "/account/addresses",
} as const

// Admin routes
export const ADMIN_ROUTES = {
  DASHBOARD: "/admin",
  LOGIN: "/admin/login",

  // Products
  PRODUCTS: "/admin/products",
  PRODUCTS_NEW: "/admin/products/new",
  PRODUCTS_EDIT: (id: string | number) => `/admin/products/${id}`,

  // Orders
  ORDERS: "/admin/orders",
  ORDERS_DETAIL: (id: string | number) => `/admin/orders/${id}`,

  // Members
  MEMBERS: "/admin/members",
  MEMBERS_DETAIL: (id: string | number) => `/admin/members/${id}`,

  // Categories
  CATEGORIES: "/admin/categories",
  CATEGORIES_NEW: "/admin/categories/new",
  CATEGORIES_EDIT: (id: string | number) => `/admin/categories/${id}`,

  // Brands
  BRANDS: "/admin/brands",
  BRANDS_NEW: "/admin/brands/new",
  BRANDS_EDIT: (id: string | number) => `/admin/brands/${id}`,

  // Settings
  ANALYTICS: "/admin/analytics",
  SETTINGS: "/admin/settings",
} as const

// Auth routes
export const AUTH_ROUTES = {
  SIGNIN: "/signin",
  SIGNUP: "/signup",
  ADMIN_LOGIN: "/admin/login",
} as const
