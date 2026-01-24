import * as z from "zod"

// User order schemas
export const getOrderLineItemsSchema = z.object({
  storeId: z.string(),
  items: z.string().optional(),
})

export const verifyOrderSchema = z.object({
  deliveryPostalCode: z.string().min(1, {
    message: "Please enter a valid postal code",
  }),
})

// Admin order filter schema
export const adminOrderFilterSchema = z.object({
  orderSn: z.string().optional(),
  receiverKeyword: z.string().optional(),
  status: z.number().optional(),
  orderType: z.number().optional(),
  sourceType: z.number().optional(),
  createTime: z.string().optional(),
  pageNum: z.number().default(1),
  pageSize: z.number().default(10),
})

export type AdminOrderFilterValues = z.infer<typeof adminOrderFilterSchema>

// Delivery info schema
export const deliverySchema = z.object({
  orderId: z.number(),
  deliveryCompany: z.string().min(1, "Delivery company is required"),
  deliverySn: z.string().min(1, "Tracking number is required"),
})

export type DeliveryValues = z.infer<typeof deliverySchema>

// Order note schema
export const orderNoteSchema = z.object({
  note: z.string().min(1, "Note is required"),
  status: z.number(),
})

export type OrderNoteValues = z.infer<typeof orderNoteSchema>

// Receiver info update schema
export const receiverInfoSchema = z.object({
  receiverName: z.string().min(1, "Receiver name is required"),
  receiverPhone: z.string().min(1, "Phone number is required"),
  receiverPostCode: z.string().optional(),
  receiverDetailAddress: z.string().min(1, "Address is required"),
  receiverProvince: z.string().optional(),
  receiverCity: z.string().optional(),
  receiverRegion: z.string().optional(),
})

export type ReceiverInfoValues = z.infer<typeof receiverInfoSchema>

// Order amount update schema
export const orderAmountSchema = z.object({
  freightAmount: z
    .string()
    .refine(
      (val) => !isNaN(Number(val)) && Number(val) >= 0,
      "Freight amount must be a positive number"
    ),
  discountAmount: z.string().optional(),
})

export type OrderAmountValues = z.infer<typeof orderAmountSchema>

// Order status constants for display
export const ORDER_STATUS = {
  0: {
    label: "Pending Payment",
    variant: "outline" as const,
    color: "text-yellow-600",
  },
  1: { label: "Paid", variant: "default" as const, color: "text-blue-600" },
  2: {
    label: "Shipped",
    variant: "secondary" as const,
    color: "text-purple-600",
  },
  3: {
    label: "Completed",
    variant: "default" as const,
    color: "text-green-600",
  },
  4: {
    label: "Closed",
    variant: "destructive" as const,
    color: "text-gray-600",
  },
} as const

// Order type constants
export const ORDER_TYPE = {
  0: "Normal Order",
  1: "Flash Sale Order",
} as const

// Order source constants
export const ORDER_SOURCE = {
  0: "PC",
  1: "App",
} as const

// Payment type constants
export const PAYMENT_TYPE = {
  0: "Not Paid",
  1: "Alipay",
  2: "WeChat Pay",
} as const
