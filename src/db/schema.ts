export type Category = {
  id: string
  name: string
  slug: string
  description?: string
  image?: string
  created: string // datetime
  updated: string // datetime
}

export type Subcategory = {
  id: string
  name: string
  slug: string
  description?: string
  category: string // relation to categories
  created: string // datetime
  updated: string // datetime
}
