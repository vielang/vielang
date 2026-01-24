import { cookies } from "next/headers"
import { NextRequest, NextResponse } from "next/server"

// Logout Endpoint - Clear all auth cookies
export async function POST(request: NextRequest) {
  try {
    const cookieStore = await cookies()
    const token = cookieStore.get("auth_token")?.value
    const tokenHead = cookieStore.get("token_head")?.value || "Bearer "

    // Optional: Call backend logout endpoint
    if (token) {
      const backendUrl =
        process.env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL || "http://localhost:8080"
      try {
        await fetch(`${backendUrl}/api/v1/portal/sso/logout`, {
          method: "POST",
          headers: {
            Authorization: `${tokenHead}${token}`,
          },
        })
      } catch (error) {
        // Ignore backend logout errors, still clear cookies
        console.error("Backend logout error:", error)
      }
    }

    // Clear all auth cookies
    cookieStore.delete("auth_token")
    cookieStore.delete("token_head")
    cookieStore.delete("refresh_token")
    cookieStore.delete("user_info")
    cookieStore.delete("csrf_token")
    // Clear readable cookies
    cookieStore.delete("auth_token_readable")
    cookieStore.delete("token_head_readable")

    return NextResponse.json({
      success: true,
      message: "Logged out successfully",
    })
  } catch (error) {
    console.error("Logout error:", error)
    return NextResponse.json({ error: "Logout failed" }, { status: 500 })
  }
}
