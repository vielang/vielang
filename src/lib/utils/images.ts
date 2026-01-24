import { env } from "@/env"

/**
 * Helper functions for image URLs (Vielang API returns direct URLs)
 */

export function getImageUrl(filename: string): string {
  if (!filename) return ""

  // Vielang API returns full URLs directly
  if (filename.startsWith("http://") || filename.startsWith("https://")) {
    return filename
  }

  // If relative path, prepend Vielang API URL
  return `${env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL}${filename}`
}

export function getImageUrls(images: string[]): string[] {
  if (!images || !Array.isArray(images)) return []

  return images.map((img) => getImageUrl(img))
}

export function getProductImageUrl(
  product: {
    images?: string[]
  },
  index = 0
): string | null {
  if (
    !product.images ||
    !Array.isArray(product.images) ||
    product.images.length === 0
  ) {
    return null
  }

  const filename = product.images[index]
  if (!filename) return null

  return getImageUrl(filename)
}

export function getProductImageUrls(product: {
  images?: string[]
}): string[] {
  if (!product.images || !Array.isArray(product.images)) return []

  return getImageUrls(product.images)
}
