import { type MetadataRoute } from "next"

import { absoluteUrl } from "@/lib/utils"

import { pages as allPages, posts as allPosts } from ".velite"

/**
 * Sitemap for SEO
 * Note: Dynamic product/category routes are disabled until Vielang Portal API pagination is implemented
 * Currently only includes static routes and content pages
 */
export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  // Store routes disabled - vielang backend is a social platform, not e-commerce
  const storesRoutes: MetadataRoute.Sitemap = []

  // Product routes disabled - would need to paginate through Vielang API
  // TODO: Implement with Vielang Portal API product search with pagination
  const productsRoutes: MetadataRoute.Sitemap = []

  // Category routes disabled - would need Vielang API category endpoints
  // TODO: Implement with Vielang Portal API category tree
  const categoriesRoutes: MetadataRoute.Sitemap = []

  // Subcategory routes disabled
  const subcategoriesRoutes: MetadataRoute.Sitemap = []

  const pagesRoutes = allPages.map((page) => ({
    url: absoluteUrl(page.slug),
    lastModified: new Date().toISOString(),
  }))

  const postsRoutes = allPosts.map((post) => ({
    url: absoluteUrl(post.slug),
    lastModified: new Date().toISOString(),
  }))

  const routes = [
    "",
    "/products",
    "/stores",
    "/blog",
    "/cart",
    "/orders",
  ].map((route) => ({
    url: absoluteUrl(route),
    lastModified: new Date().toISOString(),
  }))

  return [
    ...routes,
    ...storesRoutes,
    ...productsRoutes,
    ...categoriesRoutes,
    ...subcategoriesRoutes,
    ...pagesRoutes,
    ...postsRoutes,
  ]
}
