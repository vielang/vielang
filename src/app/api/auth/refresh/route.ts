import { cookies } from "next/headers"
import { NextRequest, NextResponse } from "next/server"

// Token Refresh Endpoint
export async function POST(request: NextRequest) {
  try {
    const cookieStore = await cookies()
    const refreshToken = cookieStore.get("refresh_token")?.value
    const tokenHead = cookieStore.get("token_head")?.value || "Bearer "

    if (!refreshToken) {
      return NextResponse.json(
        { error: "No refresh token found" },
        { status: 401 }
      )
    }

    // Call backend refresh endpoint
    const backendUrl =
      process.env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL || "http://localhost:8080"
    const response = await fetch(`${backendUrl}/api/v1/portal/sso/refreshToken`, {
      method: "GET",
      headers: {
        Authorization: `${tokenHead}${refreshToken}`,
      },
    })

    if (!response.ok) {
      // Refresh token expired or invalid
      // Clear all cookies and return 401
      cookieStore.delete("auth_token")
      cookieStore.delete("token_head")
      cookieStore.delete("refresh_token")
      cookieStore.delete("user_info")
      cookieStore.delete("auth_token_readable")
      cookieStore.delete("token_head_readable")

      return NextResponse.json(
        { error: "Refresh token expired" },
        { status: 401 }
      )
    }

    const data: any = await response.json()

    if (data.code !== 200) {
      return NextResponse.json(
        { error: data.message || "Token refresh failed" },
        { status: 401 }
      )
    }

    const { token, tokenHead: newTokenHead } = data.data

    // Set new access token (15 minutes)
    cookieStore.set("auth_token", token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "strict",
      maxAge: 900, // 15 minutes
      path: "/",
    })

    // Also set readable cookie for axios interceptor
    cookieStore.set("auth_token_readable", token, {
      httpOnly: false,
      secure: process.env.NODE_ENV === "production",
      sameSite: "strict",
      maxAge: 900,
      path: "/",
    })

    if (newTokenHead) {
      cookieStore.set("token_head", newTokenHead, {
        httpOnly: true,
        secure: process.env.NODE_ENV === "production",
        sameSite: "strict",
        maxAge: 900,
        path: "/",
      })

      cookieStore.set("token_head_readable", newTokenHead, {
        httpOnly: false,
        secure: process.env.NODE_ENV === "production",
        sameSite: "strict",
        maxAge: 900,
        path: "/",
      })
    }

    // Refresh user info cookie expiry
    const userInfo = cookieStore.get("user_info")?.value
    if (userInfo) {
      cookieStore.set("user_info", userInfo, {
        httpOnly: false,
        secure: process.env.NODE_ENV === "production",
        sameSite: "strict",
        maxAge: 900,
        path: "/",
      })
    }

    return NextResponse.json({
      success: true,
      message: "Token refreshed successfully",
    })
  } catch (error) {
    console.error("Token refresh error:", error)
    return NextResponse.json({ error: "Token refresh failed" }, { status: 500 })
  }
}
