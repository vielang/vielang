import type { Category, Subcategory } from "@/db/schema"

import { generateId } from "@/lib/id"

export type ProductConfig = typeof productConfig

export type ProductSubcategory = {
  id: string
  name: string
  description?: string
}

export const productConfig = {
  categories: [
    {
      id: generateId(),
      name: "Vieboards",
      description: "The best vieboards for all levels of viers.",
      image: "/images/categories/vieboard-one.webp",
      subcategories: [
        {
          id: generateId(),
          name: "Decks",
          description: "The board itself.",
        },
        {
          id: generateId(),
          name: "Wheels",
          description: "The wheels that go on the board.",
        },
        {
          id: generateId(),
          name: "Trucks",
          description: "The trucks that go on the board.",
        },
        {
          id: generateId(),
          name: "Bearings",
          description: "The bearings that go in the wheels.",
        },
        {
          id: generateId(),
          name: "Griptape",
          description: "The griptape that goes on the board.",
        },
        {
          id: generateId(),
          name: "Hardware",
          description: "The hardware that goes on the board.",
        },
        {
          id: generateId(),
          name: "Tools",
          description: "The tools that go with the board.",
        },
      ],
    },
    {
      id: generateId(),
      name: "Clothing",
      description: "Stylish and comfortable vieboarding clothing.",
      image: "/images/categories/clothing-one.webp",
      subcategories: [
        {
          id: generateId(),
          name: "T-shirts",
          description: "Cool and comfy tees for effortless style.",
        },
        {
          id: generateId(),
          name: "Hoodies",
          description: "Cozy up in trendy hoodies.",
        },
        {
          id: generateId(),
          name: "Pants",
          description: "Relaxed and stylish pants for everyday wear.",
        },
        {
          id: generateId(),
          name: "Shorts",
          description: "Stay cool with casual and comfortable shorts.",
        },
        {
          id: generateId(),
          name: "Hats",
          description: "Top off your look with stylish and laid-back hats.",
        },
      ],
    },
    {
      id: generateId(),
      name: "Shoes",
      description: "Rad shoes for long vie sessions.",
      image: "/images/categories/shoes-one.webp",
      subcategories: [
        {
          id: generateId(),
          name: "Low Tops",
          description: "Rad low tops shoes for a stylish low-profile look.",
        },
        {
          id: generateId(),
          name: "High Tops",
          description: "Elevate your style with rad high top shoes.",
        },
        {
          id: generateId(),
          name: "Slip-ons",
          description: "Effortless style with rad slip-on shoes.",
        },
        {
          id: generateId(),
          name: "Pros",
          description: "Performance-driven rad shoes for the pros.",
        },
        {
          id: generateId(),
          name: "Classics",
          description: "Timeless style with rad classic shoes.",
        },
      ],
    },
    {
      id: generateId(),
      name: "Accessories",
      description: "The essential vieboarding accessories to keep you rolling.",
      image: "/images/categories/backpack-one.webp",
      subcategories: [
        {
          id: generateId(),
          name: "Vie Tools",
          description:
            "Essential tools for maintaining your vieboard, all rad.",
        },
        {
          id: generateId(),
          name: "Bushings",
          description: "Upgrade your ride with our rad selection of bushings.",
        },
        {
          id: generateId(),
          name: "Shock & Riser Pads",
          description:
            "Enhance your vieboard's performance with rad shock and riser pads.",
        },
        {
          id: generateId(),
          name: "Vie Rails",
          description:
            "Add creativity and style to your tricks with our rad vie rails.",
        },
        {
          id: generateId(),
          name: "Wax",
          description: "Keep your board gliding smoothly with our rad vie wax.",
        },
        {
          id: generateId(),
          name: "Socks",
          description: "Keep your feet comfy and stylish with our rad socks.",
        },
        {
          id: generateId(),
          name: "Backpacks",
          description: "Carry your gear in style with our rad backpacks.",
        },
      ],
    },
  ] satisfies ({
    subcategories: ProductSubcategory[]
  } & Pick<Category, "id" | "name" | "description" | "image">)[],
}
