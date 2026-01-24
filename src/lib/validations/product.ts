import { z } from "zod"

export const productSchema = z.object({
  // Basic Information
  name: z.string().min(1, "Product name is required").max(200, "Name too long"),
  productSn: z.string().optional(),
  brandId: z.number().optional(),
  productCategoryId: z.number().optional(),

  // Pricing
  price: z
    .string()
    .min(1, "Price is required")
    .refine(
      (val) => !isNaN(Number(val)) && Number(val) > 0,
      "Price must be a positive number"
    ),
  originalPrice: z.string().optional(),
  promotionPrice: z.string().optional(),

  // Inventory
  stock: z.number().min(0, "Stock cannot be negative"),
  lowStock: z.number().optional(),
  unit: z.string().optional(),
  weight: z.string().optional(),

  // Description
  subTitle: z.string().optional(),
  description: z.string().optional(),
  detailTitle: z.string().optional(),
  detailDesc: z.string().optional(),
  detailHtml: z.string().optional(),

  // Images
  pic: z.string().optional(),
  albumPics: z.string().optional(),

  // Status
  publishStatus: z.number().default(0),
  newStatus: z.number().default(0),
  recommandStatus: z.number().default(0),
  verifyStatus: z.number().default(1),

  // SEO
  keywords: z.string().optional(),
  note: z.string().optional(),

  // Rewards
  giftGrowth: z.number().default(0),
  giftPoint: z.number().default(0),
  usePointLimit: z.number().optional(),

  // Sorting
  sort: z.number().default(0),
})

export type ProductFormValues = z.infer<typeof productSchema>

export const productFilterSchema = z.object({
  keyword: z.string().optional(),
  productSn: z.string().optional(),
  brandId: z.number().optional(),
  productCategoryId: z.number().optional(),
  publishStatus: z.number().optional(),
  verifyStatus: z.number().optional(),
  pageNum: z.number().default(0),
  pageSize: z.number().default(10),
})

export type ProductFilterValues = z.infer<typeof productFilterSchema>
