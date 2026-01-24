import { cookies } from "next/headers"
import { NextRequest, NextResponse } from "next/server"

// Get Current User Info
export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies()
    const token = cookieStore.get("auth_token")?.value
    const tokenHead = cookieStore.get("token_head")?.value || "Bearer "

    if (!token) {
      return NextResponse.json({ error: "Not authenticated" }, { status: 401 })
    }

    // Call backend to get user info
    const backendUrl =
      process.env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL || "http://localhost:8080"
    const response = await fetch(`${backendUrl}/api/v1/portal/sso/info`, {
      headers: {
        Authorization: `${tokenHead}${token}`,
      },
    })

    if (!response.ok) {
      return NextResponse.json(
        { error: "Failed to fetch user info" },
        { status: response.status }
      )
    }

    const data: any = await response.json()

    if (data.code !== 200) {
      return NextResponse.json(
        { error: data.message || "Failed to fetch user info" },
        { status: 401 }
      )
    }

    const user = data.data

    // Update user info cookie
    cookieStore.set(
      "user_info",
      JSON.stringify({
        id: user.id,
        username: user.username,
        nickname: user.nickname,
        icon: user.icon,
      }),
      {
        httpOnly: false,
        secure: process.env.NODE_ENV === "production",
        sameSite: "strict",
        maxAge: 900,
        path: "/",
      }
    )

    return NextResponse.json({
      user,
    })
  } catch (error) {
    console.error("Get user info error:", error)
    return NextResponse.json(
      { error: "Failed to fetch user info" },
      { status: 500 }
    )
  }
}
