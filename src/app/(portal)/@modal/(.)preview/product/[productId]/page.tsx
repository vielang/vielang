import { notFound } from "next/navigation"

/**
 * Product Modal Preview - Disabled
 * This modal preview feature is disabled until Vielang Portal API integration is complete
 * Users will be redirected to the full product page instead
 */

interface ProductModalPageProps {
  params: Promise<{
    productId: string
  }>
}

export default async function ProductModalPage({
  params,
}: ProductModalPageProps) {
  // Product modal preview disabled - redirect to full page
  notFound()
}
