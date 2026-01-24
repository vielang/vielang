import { randomBytes } from "crypto"
import { cookies } from "next/headers"
import { NextRequest, NextResponse } from "next/server"

// Generate CSRF Token
export async function GET(request: NextRequest) {
  try {
    // Generate random CSRF token
    const csrfToken = randomBytes(32).toString("hex")

    const cookieStore = await cookies()

    // Set CSRF token in cookie (readable by JavaScript for Double Submit pattern)
    cookieStore.set("csrf_token", csrfToken, {
      httpOnly: false, // Must be readable by JavaScript
      secure: process.env.NODE_ENV === "production",
      sameSite: "strict",
      maxAge: 3600, // 1 hour
      path: "/",
    })

    return NextResponse.json({
      csrfToken,
    })
  } catch (error) {
    console.error("CSRF token generation error:", error)
    return NextResponse.json(
      { error: "Failed to generate CSRF token" },
      { status: 500 }
    )
  }
}

// Validate CSRF Token (middleware helper)
export async function validateCsrfToken(
  request: NextRequest
): Promise<boolean> {
  const csrfHeader = request.headers.get("X-CSRF-Token")
  const cookieStore = await cookies()
  const csrfCookie = cookieStore.get("csrf_token")?.value

  return !!(csrfHeader && csrfCookie && csrfHeader === csrfCookie)
}
