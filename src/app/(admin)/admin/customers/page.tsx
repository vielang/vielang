"use client"

import { useEffect, useState } from "react"
import { useRouter } from "next/navigation"
import {
  Eye,
  Mail,
  MoreHorizontal,
  Phone,
  Search,
  Shield,
  ShieldOff,
  User,
} from "lucide-react"
import { toast } from "sonner"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

interface Customer {
  id: number
  username: string
  nickname: string
  email: string
  phone: string
  status: number
  memberLevelId: number
  memberLevelName: string
  totalPurchase: number
  orderCount: number
  createTime: string
}

export default function CustomersPage() {
  const router = useRouter()
  const [customers, setCustomers] = useState<Customer[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState("")
  const [selectedIds, setSelectedIds] = useState<number[]>([])
  const [statusFilter, setStatusFilter] = useState<string>("all")

  useEffect(() => {
    loadCustomers()
  }, [statusFilter])

  async function loadCustomers() {
    try {
      setIsLoading(true)
      // Mock data for demonstration
      const mockCustomers: Customer[] = [
        {
          id: 1,
          username: "user001",
          nickname: "Nguyễn Văn A",
          email: "nguyenvana@gmail.com",
          phone: "0901234567",
          status: 1,
          memberLevelId: 4,
          memberLevelName: "Platinum",
          totalPurchase: 25000000,
          orderCount: 15,
          createTime: "2024-01-15",
        },
        {
          id: 2,
          username: "user002",
          nickname: "Trần Thị B",
          email: "tranthib@gmail.com",
          phone: "0902345678",
          status: 1,
          memberLevelId: 3,
          memberLevelName: "Gold",
          totalPurchase: 15000000,
          orderCount: 10,
          createTime: "2024-02-10",
        },
        {
          id: 3,
          username: "user003",
          nickname: "Lê Văn C",
          email: "levanc@gmail.com",
          phone: "0903456789",
          status: 0,
          memberLevelId: 2,
          memberLevelName: "Silver",
          totalPurchase: 8000000,
          orderCount: 5,
          createTime: "2024-03-05",
        },
        {
          id: 4,
          username: "user004",
          nickname: "Phạm Thị D",
          email: "phamthid@gmail.com",
          phone: "0904567890",
          status: 1,
          memberLevelId: 3,
          memberLevelName: "Gold",
          totalPurchase: 18000000,
          orderCount: 12,
          createTime: "2024-01-20",
        },
        {
          id: 5,
          username: "user005",
          nickname: "Hoàng Văn E",
          email: "hoangvane@gmail.com",
          phone: "0905678901",
          status: 1,
          memberLevelId: 1,
          memberLevelName: "Bronze",
          totalPurchase: 3500000,
          orderCount: 2,
          createTime: "2024-04-12",
        },
      ]
      setCustomers(mockCustomers)
    } catch (error: any) {
      console.error("Error loading customers:", error)
      toast.error("Không thể tải danh sách khách hàng")
      setCustomers([])
    } finally {
      setIsLoading(false)
    }
  }

  async function handleToggleStatus(id: number, currentStatus: number) {
    try {
      // TODO: Implement API call
      toast.success(
        currentStatus === 1
          ? "Đã vô hiệu hóa tài khoản"
          : "Đã kích hoạt tài khoản"
      )
      await loadCustomers()
    } catch (error: any) {
      toast.error(error.message || "Lỗi khi cập nhật trạng thái")
    }
  }

  const filteredCustomers = customers.filter((customer) => {
    const matchesSearch =
      customer.username.toLowerCase().includes(searchQuery.toLowerCase()) ||
      customer.nickname.toLowerCase().includes(searchQuery.toLowerCase()) ||
      customer.email.toLowerCase().includes(searchQuery.toLowerCase()) ||
      customer.phone.includes(searchQuery)

    const matchesStatus =
      statusFilter === "all" ||
      (statusFilter === "active" && customer.status === 1) ||
      (statusFilter === "inactive" && customer.status === 0)

    return matchesSearch && matchesStatus
  })

  const toggleSelectAll = () => {
    if (selectedIds.length === filteredCustomers.length) {
      setSelectedIds([])
    } else {
      setSelectedIds(filteredCustomers.map((c) => c.id))
    }
  }

  const toggleSelect = (id: number) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id]
    )
  }

  const getMemberLevelColor = (levelName: string) => {
    switch (levelName.toLowerCase()) {
      case "platinum":
        return "bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400"
      case "gold":
        return "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400"
      case "silver":
        return "bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400"
      case "bronze":
        return "bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400"
      default:
        return "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400"
    }
  }

  if (isLoading) {
    return <LoadingSkeleton />
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">
            Quản lý khách hàng
          </h1>
          <p className="text-muted-foreground">
            Quản lý thông tin và tài khoản khách hàng
          </p>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Tổng khách hàng</CardDescription>
            <CardTitle className="text-3xl">{customers.length}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Đang hoạt động</CardDescription>
            <CardTitle className="text-3xl text-green-600">
              {customers.filter((c) => c.status === 1).length}
            </CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Bị khóa</CardDescription>
            <CardTitle className="text-3xl text-red-600">
              {customers.filter((c) => c.status === 0).length}
            </CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Khách hàng VIP</CardDescription>
            <CardTitle className="text-3xl text-purple-600">
              {customers.filter((c) => c.memberLevelId >= 3).length}
            </CardTitle>
          </CardHeader>
        </Card>
      </div>

      {/* Main Content */}
      <Card>
        <CardHeader>
          <div className="space-y-4">
            {/* Search */}
            <div className="flex items-center gap-4">
              <div className="relative max-w-sm flex-1">
                <Search className="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
                <Input
                  placeholder="Tìm kiếm theo tên, email, SĐT..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-9"
                />
              </div>
              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-[180px]">
                  <SelectValue placeholder="Trạng thái" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">Tất cả</SelectItem>
                  <SelectItem value="active">Đang hoạt động</SelectItem>
                  <SelectItem value="inactive">Bị khóa</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-12">
                    <Checkbox
                      checked={
                        selectedIds.length === filteredCustomers.length &&
                        filteredCustomers.length > 0
                      }
                      onCheckedChange={toggleSelectAll}
                    />
                  </TableHead>
                  <TableHead>Khách hàng</TableHead>
                  <TableHead>Liên hệ</TableHead>
                  <TableHead>Cấp độ</TableHead>
                  <TableHead>Tổng mua</TableHead>
                  <TableHead>Đơn hàng</TableHead>
                  <TableHead>Trạng thái</TableHead>
                  <TableHead>Ngày tham gia</TableHead>
                  <TableHead className="text-right">Thao tác</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredCustomers.length > 0 ? (
                  filteredCustomers.map((customer) => (
                    <TableRow key={customer.id}>
                      <TableCell>
                        <Checkbox
                          checked={selectedIds.includes(customer.id)}
                          onCheckedChange={() => toggleSelect(customer.id)}
                        />
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-3">
                          <div className="bg-muted flex h-10 w-10 items-center justify-center rounded-full">
                            <User className="h-5 w-5" />
                          </div>
                          <div>
                            <p className="font-medium">{customer.nickname}</p>
                            <p className="text-muted-foreground text-sm">
                              @{customer.username}
                            </p>
                          </div>
                        </div>
                      </TableCell>
                      <TableCell>
                        <div className="space-y-1">
                          <div className="flex items-center gap-2 text-sm">
                            <Mail className="text-muted-foreground h-4 w-4" />
                            <span>{customer.email}</span>
                          </div>
                          <div className="flex items-center gap-2 text-sm">
                            <Phone className="text-muted-foreground h-4 w-4" />
                            <span>{customer.phone}</span>
                          </div>
                        </div>
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant="secondary"
                          className={getMemberLevelColor(
                            customer.memberLevelName
                          )}
                        >
                          {customer.memberLevelName}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <span className="font-medium">
                          {customer.totalPurchase.toLocaleString("vi-VN")} ₫
                        </span>
                      </TableCell>
                      <TableCell>
                        <span className="font-medium">
                          {customer.orderCount}
                        </span>
                      </TableCell>
                      <TableCell>
                        {customer.status === 1 ? (
                          <Badge variant="default" className="bg-green-600">
                            <Shield className="mr-1 h-3 w-3" />
                            Hoạt động
                          </Badge>
                        ) : (
                          <Badge variant="destructive">
                            <ShieldOff className="mr-1 h-3 w-3" />
                            Bị khóa
                          </Badge>
                        )}
                      </TableCell>
                      <TableCell className="text-muted-foreground">
                        {customer.createTime}
                      </TableCell>
                      <TableCell className="text-right">
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon">
                              <MoreHorizontal className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuLabel>Thao tác</DropdownMenuLabel>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              onClick={() =>
                                router.push(`/admin/customers/${customer.id}`)
                              }
                            >
                              <Eye className="mr-2 h-4 w-4" />
                              Xem chi tiết
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onClick={() =>
                                handleToggleStatus(customer.id, customer.status)
                              }
                            >
                              {customer.status === 1 ? (
                                <>
                                  <ShieldOff className="mr-2 h-4 w-4" />
                                  Vô hiệu hóa
                                </>
                              ) : (
                                <>
                                  <Shield className="mr-2 h-4 w-4" />
                                  Kích hoạt
                                </>
                              )}
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell colSpan={9} className="h-24 text-center">
                      {searchQuery || statusFilter !== "all"
                        ? "Không tìm thấy khách hàng nào"
                        : "Chưa có khách hàng nào"}
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function LoadingSkeleton() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="space-y-2">
          <Skeleton className="h-9 w-64" />
          <Skeleton className="h-5 w-96" />
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <Card key={i}>
            <CardHeader className="pb-3">
              <Skeleton className="h-4 w-24" />
              <Skeleton className="h-8 w-16" />
            </CardHeader>
          </Card>
        ))}
      </div>

      <Card>
        <CardHeader>
          <div className="space-y-4">
            <Skeleton className="h-10 w-full max-w-sm" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-16 w-full" />
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
