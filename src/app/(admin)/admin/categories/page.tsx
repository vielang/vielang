"use client"

import * as React from "react"
import Link from "next/link"
import { useRouter } from "next/navigation"
import {
  PlusCircle,
  Edit,
  Trash2,
  MoreHorizontal,
  FolderTree,
  Eye,
  EyeOff,
} from "lucide-react"

import { adminCategoryApi } from "@/lib/api/vielang-index"
import type { ContentCategory } from "@/lib/api/vielang-types"
import { useAdminAuth } from "@/lib/hooks/use-admin-auth"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { toast } from "sonner"

export default function AdminCategoriesPage() {
  const router = useRouter()
  const { admin, isLoading: authLoading } = useAdminAuth()

  const [categories, setCategories] = React.useState<ContentCategory[]>([])
  const [isLoading, setIsLoading] = React.useState(true)
  const [deleteDialogOpen, setDeleteDialogOpen] = React.useState(false)
  const [categoryToDelete, setCategoryToDelete] = React.useState<number | null>(null)

  React.useEffect(() => {
    const hasLocalAuth =
      typeof window !== "undefined" &&
      localStorage.getItem("admin_auth_token") &&
      localStorage.getItem("admin_user")

    if (!authLoading && (admin || hasLocalAuth)) {
      loadCategories()
    }
  }, [admin, authLoading])

  async function loadCategories() {
    try {
      setIsLoading(true)
      const result = await adminCategoryApi.getAll()
      setCategories(result || [])
    } catch (error) {
      console.error("Error loading categories:", error)
      toast.error("Failed to load categories")
    } finally {
      setIsLoading(false)
    }
  }

  const handleDeleteCategory = async () => {
    if (!categoryToDelete) return

    try {
      await adminCategoryApi.delete(categoryToDelete)
      toast.success("Category deleted successfully")
      setDeleteDialogOpen(false)
      setCategoryToDelete(null)
      loadCategories()
    } catch (error) {
      console.error("Error deleting category:", error)
      toast.error("Failed to delete category")
    }
  }

  const handleToggleNavStatus = async (categoryId: number, currentStatus: number | boolean) => {
    try {
      const isActive = typeof currentStatus === 'boolean' ? currentStatus : currentStatus === 1
      await adminCategoryApi.updateNavStatus(
        [categoryId],
        isActive ? 0 : 1
      )
      toast.success(
        isActive
          ? "Category hidden from navigation"
          : "Category shown in navigation"
      )
      loadCategories()
    } catch (error) {
      console.error("Error toggling nav status:", error)
      toast.error("Failed to update navigation status")
    }
  }

  const handleToggleStatus = async (categoryId: number, currentStatus: number | boolean) => {
    try {
      const isActive = typeof currentStatus === 'boolean' ? currentStatus : currentStatus === 1
      await adminCategoryApi.updateStatus(
        [categoryId],
        isActive ? 0 : 1
      )
      toast.success(
        isActive
          ? "Category disabled"
          : "Category enabled"
      )
      loadCategories()
    } catch (error) {
      console.error("Error toggling status:", error)
      toast.error("Failed to update status")
    }
  }

  // Helper function to check if value is active (handles both boolean and number)
  const isActive = (value: number | boolean) => {
    return typeof value === 'boolean' ? value : value === 1
  }

  if (authLoading || isLoading) {
    return <LoadingSkeleton />
  }

  // Organize categories by hierarchy
  const rootCategories = categories.filter(cat => cat.parentId === 0)
  const childCategories = categories.filter(cat => cat.parentId !== 0)

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Categories</h1>
          <p className="text-muted-foreground">
            Manage content categories ({categories.length} total)
          </p>
        </div>
        <Button asChild>
          <Link href="/admin/categories/new">
            <PlusCircle className="mr-2 h-4 w-4" />
            Create Category
          </Link>
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>All Categories</CardTitle>
        </CardHeader>
        <CardContent>
          {categories.length === 0 ? (
            <div className="text-muted-foreground py-12 text-center">
              No categories found
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Slug</TableHead>
                  <TableHead>Level</TableHead>
                  <TableHead>Posts</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rootCategories.map((category) => (
                  <React.Fragment key={category.id}>
                    <TableRow>
                      <TableCell className="font-medium">
                        <div className="flex items-center gap-2">
                          <FolderTree className="h-4 w-4" />
                          {category.name}
                        </div>
                      </TableCell>
                      <TableCell className="font-mono text-sm">
                        {category.slug}
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline">Root</Badge>
                      </TableCell>
                      <TableCell>{category.postCount || 0}</TableCell>
                      <TableCell>
                        <div className="flex gap-1">
                          {isActive(category.status) ? (
                            <Badge variant="default">Active</Badge>
                          ) : (
                            <Badge variant="secondary">Disabled</Badge>
                          )}
                          {isActive(category.isNav) && (
                            <Badge variant="outline">In Nav</Badge>
                          )}
                        </div>
                      </TableCell>
                      <TableCell className="text-right">
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon">
                              <MoreHorizontal className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuLabel>Actions</DropdownMenuLabel>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem asChild>
                              <Link href={`/admin/categories/${category.id}/edit`}>
                                <Edit className="mr-2 h-4 w-4" />
                                Edit
                              </Link>
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onClick={() =>
                                handleToggleStatus(category.id, category.status)
                              }
                            >
                              {isActive(category.status) ? "Disable" : "Enable"}
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onClick={() =>
                                handleToggleNavStatus(category.id, category.isNav)
                              }
                            >
                              {isActive(category.isNav) ? (
                                <>
                                  <EyeOff className="mr-2 h-4 w-4" />
                                  Hide from Nav
                                </>
                              ) : (
                                <>
                                  <Eye className="mr-2 h-4 w-4" />
                                  Show in Nav
                                </>
                              )}
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              className="text-destructive"
                              onClick={() => {
                                setCategoryToDelete(category.id)
                                setDeleteDialogOpen(true)
                              }}
                            >
                              <Trash2 className="mr-2 h-4 w-4" />
                              Delete
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                    {/* Child categories */}
                    {childCategories
                      .filter((child) => child.parentId === category.id)
                      .map((child) => (
                        <TableRow key={child.id} className="bg-muted/30">
                          <TableCell className="font-medium">
                            <div className="flex items-center gap-2 pl-8">
                              <span className="text-muted-foreground">└─</span>
                              {child.name}
                            </div>
                          </TableCell>
                          <TableCell className="font-mono text-sm">
                            {child.slug}
                          </TableCell>
                          <TableCell>
                            <Badge variant="secondary">Sub</Badge>
                          </TableCell>
                          <TableCell>{child.postCount || 0}</TableCell>
                          <TableCell>
                            <div className="flex gap-1">
                              {isActive(child.status) ? (
                                <Badge variant="default">Active</Badge>
                              ) : (
                                <Badge variant="secondary">Disabled</Badge>
                              )}
                              {isActive(child.isNav) && (
                                <Badge variant="outline">In Nav</Badge>
                              )}
                            </div>
                          </TableCell>
                          <TableCell className="text-right">
                            <DropdownMenu>
                              <DropdownMenuTrigger asChild>
                                <Button variant="ghost" size="icon">
                                  <MoreHorizontal className="h-4 w-4" />
                                </Button>
                              </DropdownMenuTrigger>
                              <DropdownMenuContent align="end">
                                <DropdownMenuLabel>Actions</DropdownMenuLabel>
                                <DropdownMenuSeparator />
                                <DropdownMenuItem asChild>
                                  <Link href={`/admin/categories/${child.id}/edit`}>
                                    <Edit className="mr-2 h-4 w-4" />
                                    Edit
                                  </Link>
                                </DropdownMenuItem>
                                <DropdownMenuItem
                                  onClick={() =>
                                    handleToggleStatus(child.id, child.status)
                                  }
                                >
                                  {isActive(child.status) ? "Disable" : "Enable"}
                                </DropdownMenuItem>
                                <DropdownMenuItem
                                  onClick={() =>
                                    handleToggleNavStatus(child.id, child.isNav)
                                  }
                                >
                                  {isActive(child.isNav) ? (
                                    <>
                                      <EyeOff className="mr-2 h-4 w-4" />
                                      Hide from Nav
                                    </>
                                  ) : (
                                    <>
                                      <Eye className="mr-2 h-4 w-4" />
                                      Show in Nav
                                    </>
                                  )}
                                </DropdownMenuItem>
                                <DropdownMenuSeparator />
                                <DropdownMenuItem
                                  className="text-destructive"
                                  onClick={() => {
                                    setCategoryToDelete(child.id)
                                    setDeleteDialogOpen(true)
                                  }}
                                >
                                  <Trash2 className="mr-2 h-4 w-4" />
                                  Delete
                                </DropdownMenuItem>
                              </DropdownMenuContent>
                            </DropdownMenu>
                          </TableCell>
                        </TableRow>
                      ))}
                  </React.Fragment>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Category</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete this category? This action cannot be
              undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteCategory}>
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}

function LoadingSkeleton() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <Skeleton className="h-9 w-48" />
          <Skeleton className="mt-2 h-5 w-64" />
        </div>
        <Skeleton className="h-10 w-40" />
      </div>

      <Card>
        <CardHeader>
          <Skeleton className="h-6 w-32" />
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-16 w-full" />
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
