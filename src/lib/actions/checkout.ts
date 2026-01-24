"use server"

import { revalidatePath } from "next/cache"

import { createServerApi } from "@/lib/api/server"
import { getCachedUser } from "@/lib/queries/user"

export interface CheckoutData {
  name: string
  email: string
  phone: string
  address: string
  city: string
  state: string
  postalCode: string
  country: string
  notes?: string
}

export interface CartItem {
  id: string
  cart: string
  product: string
  quantity: number
  expand?: {
    product?: {
      id: string
      name: string
      price: string
    }
  }
}

export async function processOrder(
  checkoutData: CheckoutData,
  cartItems: CartItem[]
) {
  try {
    const user = await getCachedUser()
    const api = await createServerApi()

    if (!cartItems.length) {
      return {
        success: false,
        error: "Cart is empty",
      }
    }

    // Calculate total amount
    const totalAmount = cartItems.reduce((total, item) => {
      const price = item.expand?.product?.price
        ? parseFloat(item.expand.product.price)
        : 0
      return total + price * item.quantity
    }, 0)

    // Create address record first
    const addressData = {
      line1: checkoutData.address,
      line2: "", // Optional second line
      city: checkoutData.city,
      state: checkoutData.state,
      postal_code: checkoutData.postalCode,
      country: checkoutData.country,
      user: user?.id || "", // Can be empty for guest orders
    }

    const addressResponse = await api.post(
      "/collections/addresses/records",
      addressData
    )
    const addressRecord = addressResponse.data

    // Get store ID from the first product in the cart
    const firstProduct = cartItems[0]
    if (!firstProduct) {
      return {
        success: false,
        error: "No products in cart",
      }
    }

    const storeId = await getProductStore(firstProduct.product)

    if (!storeId) {
      return {
        success: false,
        error: "Unable to determine store for order",
      }
    }

    // Prepare order items data
    const orderItems = cartItems.map((item) => ({
      productId: item.product,
      productName: item.expand?.product?.name || "",
      price: item.expand?.product?.price || "0",
      quantity: item.quantity,
    }))

    // Create order
    const orderData = {
      user: user?.id || "", // Can be empty for guest orders
      store: storeId,
      items: orderItems, // Store as JSON
      quantity: cartItems.reduce((total, item) => total + item.quantity, 0),
      amount: totalAmount.toString(),
      status: "pending",
      name: checkoutData.name,
      email: checkoutData.email,
      address: addressRecord.id,
      // Add notes to the order data if provided
      ...(checkoutData.notes && { notes: checkoutData.notes }),
    }

    const orderResponse = await api.post(
      "/collections/orders/records",
      orderData
    )
    const orderRecord = orderResponse.data

    // Clear the user's cart after successful order
    if (user) {
      await clearUserCart(user.id.toString())
    }

    revalidatePath("/dashboard/purchases")

    return {
      success: true,
      orderId: orderRecord.id,
      orderNumber: `ORD-${orderRecord.id.slice(-8).toUpperCase()}`,
    }
  } catch (error) {
    console.error("Order processing error:", error)
    return {
      success: false,
      error: error instanceof Error ? error.message : "Failed to process order",
    }
  }
}

// Helper function to get the store ID for a product
async function getProductStore(productId: string): Promise<string | null> {
  try {
    const api = await createServerApi()
    const response = await api.get(`/collections/products/records/${productId}`)
    const product = response.data
    return product.store || null
  } catch (error) {
    console.error("Error getting product store:", error)
    return null
  }
}

// Helper function to clear user's cart
async function clearUserCart(userId: string): Promise<void> {
  try {
    const api = await createServerApi()

    // Get user's cart
    const cartsResponse = await api.get("/collections/carts/records", {
      params: {
        filter: `user = "${userId}"`,
      },
    })
    const carts = cartsResponse.data.items || []

    // Delete all cart items for each cart
    for (const cart of carts) {
      const cartItemsResponse = await api.get(
        "/collections/cart_items/records",
        {
          params: {
            filter: `cart = "${cart.id}"`,
          },
        }
      )
      const cartItems = cartItemsResponse.data.items || []

      for (const item of cartItems) {
        await api.delete(`/collections/cart_items/records/${item.id}`)
      }
    }
  } catch (error) {
    console.error("Error clearing user cart:", error)
    // Don't throw error here as order was successful
  }
}
