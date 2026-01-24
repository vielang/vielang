import Link from "next/link"
import { Card, CardContent, CardHeader } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { FileText } from "lucide-react"
import type { ContentCategory } from "@/lib/api/vielang-types"

interface CategoryCardProps {
  category: ContentCategory
}

export function CategoryCard({ category }: CategoryCardProps) {
  return (
    <Link href={`/posts?category=${category.id}`}>
      <Card className="h-full transition-all hover:shadow-lg">
        {category.coverImage && (
          <div className="aspect-video overflow-hidden">
            <img
              src={category.coverImage}
              alt={category.name}
              className="h-full w-full object-cover transition-transform hover:scale-105"
            />
          </div>
        )}
        <CardHeader>
          <div className="flex items-center gap-2">
            {category.icon && <span className="text-2xl">{category.icon}</span>}
            <h3 className="font-semibold">{category.name}</h3>
          </div>
        </CardHeader>
        <CardContent>
          <p className="mb-3 line-clamp-2 text-sm text-muted-foreground">
            {category.description || "Explore posts in this category"}
          </p>
          <Badge variant="secondary" className="gap-1">
            <FileText className="h-3 w-3" />
            {category.postCount} posts
          </Badge>
        </CardContent>
      </Card>
    </Link>
  )
}
