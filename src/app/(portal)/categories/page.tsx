import { Suspense } from "react"
import { portalCategoryApi } from "@/lib/api/vielang-index"
import type { ContentCategory } from "@/lib/api/vielang-types"
import { CategoryCard } from "../_components/category-card"
import { Skeleton } from "@/components/ui/skeleton"
import { Card, CardContent, CardHeader } from "@/components/ui/card"

export const metadata = {
  title: "Categories - Vielang",
  description: "Browse all categories and discover interesting content",
}

async function CategoriesContent() {
  let categories: ContentCategory[] = []
  try {
    categories = await portalCategoryApi.getList() // Get all categories (default limit: 20)
  } catch (error) {
    console.error("Error fetching categories:", error)
  }

  if (categories.length === 0) {
    return (
      <div className="py-12 text-center">
        <p className="text-muted-foreground">No categories available.</p>
      </div>
    )
  }

  return (
    <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {categories.map((category) => (
        <CategoryCard key={category.id} category={category} />
      ))}
    </div>
  )
}

function LoadingSkeleton() {
  return (
    <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {Array.from({ length: 8 }).map((_, i) => (
        <Card key={i} className="overflow-hidden">
          <Skeleton className="aspect-video w-full" />
          <CardHeader className="pb-3">
            <Skeleton className="h-6 w-3/4" />
          </CardHeader>
          <CardContent className="pb-3">
            <Skeleton className="h-4 w-full mb-2" />
            <Skeleton className="h-4 w-2/3" />
            <Skeleton className="h-5 w-20 mt-3" />
          </CardContent>
        </Card>
      ))}
    </div>
  )
}

export default function CategoriesPage() {
  return (
    <div className="container py-8 sm:py-12">
      <div className="mb-8">
        <h1 className="text-3xl font-bold sm:text-4xl">All Categories</h1>
        <p className="mt-2 text-muted-foreground">
          Browse posts by category
        </p>
      </div>
      <Suspense fallback={<LoadingSkeleton />}>
        <CategoriesContent />
      </Suspense>
    </div>
  )
}
