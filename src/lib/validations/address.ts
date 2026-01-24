import * as z from "zod"

// Address form schema for add/edit
export const addressSchema = z.object({
  name: z
    .string()
    .min(1, "Name is required")
    .max(50, "Name must be less than 50 characters"),
  phoneNumber: z
    .string()
    .min(1, "Phone number is required")
    .regex(
      /^(\+84|0)[0-9]{9,10}$/,
      "Phone number must be a valid Vietnamese phone number"
    ),
  province: z.string().min(1, "Province is required"),
  city: z.string().min(1, "City is required"),
  region: z.string().optional(),
  detailAddress: z
    .string()
    .min(1, "Detail address is required")
    .max(200, "Address must be less than 200 characters"),
  postCode: z.string().optional(),
  defaultStatus: z.number().optional(),
})

export type AddressFormValues = z.infer<typeof addressSchema>

// Simplified schema for quick add in checkout
export const quickAddressSchema = addressSchema.pick({
  name: true,
  phoneNumber: true,
  province: true,
  city: true,
  detailAddress: true,
})

export type QuickAddressFormValues = z.infer<typeof quickAddressSchema>

// Constants for address management
export const DEFAULT_STATUS = {
  0: "Normal",
  1: "Default",
} as const

// Vietnamese provinces (major cities)
export const VIETNAM_PROVINCES = [
  "Hà Nội",
  "Hồ Chí Minh",
  "Đà Nẵng",
  "Hải Phòng",
  "Cần Thơ",
  "An Giang",
  "Bà Rịa - Vũng Tàu",
  "Bắc Giang",
  "Bắc Kạn",
  "Bạc Liêu",
  "Bắc Ninh",
  "Bến Tre",
  "Bình Định",
  "Bình Dương",
  "Bình Phước",
  "Bình Thuận",
  "Cà Mau",
  "Cao Bằng",
  "Đắk Lắk",
  "Đắk Nông",
  "Điện Biên",
  "Đồng Nai",
  "Đồng Tháp",
  "Gia Lai",
  "Hà Giang",
  "Hà Nam",
  "Hà Tĩnh",
  "Hải Dương",
  "Hậu Giang",
  "Hòa Bình",
  "Hưng Yên",
  "Khánh Hòa",
  "Kiên Giang",
  "Kon Tum",
  "Lai Châu",
  "Lâm Đồng",
  "Lạng Sơn",
  "Lào Cai",
  "Long An",
  "Nam Định",
  "Nghệ An",
  "Ninh Bình",
  "Ninh Thuận",
  "Phú Thọ",
  "Phú Yên",
  "Quảng Bình",
  "Quảng Nam",
  "Quảng Ngãi",
  "Quảng Ninh",
  "Quảng Trị",
  "Sóc Trăng",
  "Sơn La",
  "Tây Ninh",
  "Thái Bình",
  "Thái Nguyên",
  "Thanh Hóa",
  "Thừa Thiên Huế",
  "Tiền Giang",
  "Trà Vinh",
  "Tuyên Quang",
  "Vĩnh Long",
  "Vĩnh Phúc",
  "Yên Bái",
] as const
