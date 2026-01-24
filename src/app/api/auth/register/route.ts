import { NextRequest, NextResponse } from "next/server"

interface RegisterRequest {
  username: string
  password: string
  telephone: string
  authCode: string
}

// Register Endpoint
export async function POST(request: NextRequest) {
  try {
    const body = (await request.json()) as RegisterRequest
    const { username, password, telephone, authCode } = body

    if (!username || !password || !telephone || !authCode) {
      return NextResponse.json(
        { error: "All fields are required" },
        { status: 400 }
      )
    }

    // Call backend API
    const params = new URLSearchParams()
    params.append("username", username)
    params.append("password", password)
    params.append("telephone", telephone)
    params.append("authCode", authCode)

    const backendUrl =
      process.env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL || "http://localhost:8080"
    const response = await fetch(`${backendUrl}/api/v1/portal/sso/register`, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: params,
    })

    if (!response.ok) {
      const errorData: any = await response.json()
      return NextResponse.json(
        { error: errorData.message || "Registration failed" },
        { status: response.status }
      )
    }

    const data: any = await response.json()

    if (data.code !== 200) {
      return NextResponse.json(
        { error: data.message || "Registration failed" },
        { status: 400 }
      )
    }

    return NextResponse.json({
      success: true,
      message: "Registration successful. Please login.",
    })
  } catch (error) {
    console.error("Registration error:", error)
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    )
  }
}
