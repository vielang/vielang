/**
 * API Endpoints Constants
 * Centralized API endpoint definitions
 */

// Portal API endpoints
export const PORTAL_ENDPOINTS = {
  // Products
  PRODUCTS_SEARCH: "/product/search",
  PRODUCTS_DETAIL: "/product/detail",
  PRODUCTS_CATEGORY_TREE: "/product/categoryTreeList",

  // Cart
  CART_LIST: "/cart/list",
  CART_ADD: "/cart/add",
  CART_UPDATE_QUANTITY: "/cart/update/quantity",
  CART_DELETE: "/cart/delete",
  CART_CLEAR: "/cart/clear",

  // Orders
  ORDERS_GENERATE: "/order/generateOrder",
  ORDERS_LIST: "/order/list",
  ORDERS_DETAIL: "/order/detail",
  ORDERS_CANCEL: "/order/cancelUserOrder",
  ORDERS_CONFIRM_RECEIVE: "/order/confirmReceiveOrder",
  ORDERS_DELETE: "/order/deleteOrder",

  // Member
  MEMBER_INFO: "/sso/info",
  MEMBER_UPDATE: "/member/update",

  // Address
  ADDRESS_LIST: "/memberReceiveAddress/list",
  ADDRESS_ADD: "/memberReceiveAddress/add",
  ADDRESS_UPDATE: "/memberReceiveAddress/update",
  ADDRESS_DELETE: "/memberReceiveAddress/delete",

  // Auth
  AUTH_LOGIN: "/sso/login",
  AUTH_REGISTER: "/sso/register",
  AUTH_REFRESH: "/sso/refreshToken",
  AUTH_LOGOUT: "/sso/logout",
  AUTH_CODE: "/sso/getAuthCode",

  // Brand
  BRAND_LIST: "/home/brandList",
  BRAND_RECOMMEND: "/brand/recommend",
} as const

// Admin API endpoints
export const ADMIN_ENDPOINTS = {
  // Auth
  AUTH_LOGIN: "/admin/login",
  AUTH_LOGOUT: "/admin/logout",
  AUTH_INFO: "/admin/info",

  // Products
  PRODUCTS_LIST: "/product/list",
  PRODUCTS_CREATE: "/product/create",
  PRODUCTS_UPDATE: "/product/update",
  PRODUCTS_DELETE: "/product/delete",
  PRODUCTS_UPDATE_STATUS: "/product/update/publishStatus",

  // Categories
  CATEGORIES_LIST: "/productCategory/list",
  CATEGORIES_CREATE: "/productCategory/create",
  CATEGORIES_UPDATE: "/productCategory/update",
  CATEGORIES_DELETE: "/productCategory/delete",

  // Brands
  BRANDS_LIST: "/brand/list",
  BRANDS_CREATE: "/brand/create",
  BRANDS_UPDATE: "/brand/update",
  BRANDS_DELETE: "/brand/delete",

  // Orders
  ORDERS_LIST: "/order/list",
  ORDERS_DETAIL: "/order/detail",
  ORDERS_UPDATE_STATUS: "/order/update/status",
  ORDERS_UPDATE_NOTE: "/order/update/note",
  ORDERS_DELETE: "/order/delete",

  // Members
  MEMBERS_LIST: "/member/list",
  MEMBERS_DETAIL: "/member/detail",
} as const
