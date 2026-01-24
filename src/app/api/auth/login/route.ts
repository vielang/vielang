import { NextRequest, NextResponse } from "next/server"

interface LoginRequest {
  username: string
  password: string
}

// Portal Login Endpoint - Returns token to client for localStorage
export async function POST(request: NextRequest) {
  try {
    const body = (await request.json()) as LoginRequest
    const { username, password } = body

    if (!username || !password) {
      return NextResponse.json(
        { error: "Username and password are required" },
        { status: 400 }
      )
    }

    // Call backend API
    const params = new URLSearchParams()
    params.append("username", username)
    params.append("password", password)

    const backendUrl =
      process.env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL || "http://localhost:8080"
    const response = await fetch(`${backendUrl}/api/v1/portal/sso/login`, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: params,
    })

    if (!response.ok) {
      const errorData: any = await response.json()
      return NextResponse.json(
        { error: errorData.message || "Login failed" },
        { status: response.status }
      )
    }

    const data: any = await response.json()

    // Check response format
    if (data.code !== 200) {
      return NextResponse.json(
        { error: data.message || "Login failed" },
        { status: 401 }
      )
    }

    const { token, tokenHead } = data.data

    // Fetch user info
    const userInfoResponse = await fetch(`${backendUrl}/api/v1/portal/sso/info`, {
      headers: {
        Authorization: `${tokenHead}${token}`,
      },
    })

    let user = null
    if (userInfoResponse.ok) {
      const userInfoData: any = await userInfoResponse.json()
      if (userInfoData.code === 200) {
        user = userInfoData.data
      }
    }

    // Return token, tokenHead, and user to client
    // Client will store in localStorage
    return NextResponse.json({
      success: true,
      token,
      tokenHead,
      user,
      message: "Login successful",
    })
  } catch (error) {
    console.error("Login error:", error)
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    )
  }
}
