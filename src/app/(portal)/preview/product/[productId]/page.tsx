import { redirect } from "next/navigation"

interface ProductPreviewPageProps {
  params: Promise<{
    productId: string
  }>
}

export default async function ProductPreviewPage({
  params,
}: ProductPreviewPageProps) {
  const resolvedParams = await params
  const productId = Number(resolvedParams.productId)

  redirect(`/product/${productId}`)
}
