// Temporary middleware - passes through all requests
// TODO: Implement Vielang auth middleware if needed
import type { NextRequest } from "next/server"
import { NextResponse } from "next/server"

export default function middleware(_request: NextRequest) {
  try {
    // For now, just pass through all requests
    return NextResponse.next()
  } catch (error) {
    console.error("Middleware error:", error)
    // Return a proper response even if middleware fails
    return NextResponse.next()
  }
}

// export default clerkMiddleware((auth, req) => {
//   if (isProtectedRoute(req)) {
//     const url = new URL(req.nextUrl.origin)

//     auth().protect({
//       unauthenticatedUrl: `${url.origin}/signin`,
//       unauthorizedUrl: `${url.origin}/dashboard/stores`,
//     })
//   }
// })

export const config = {
  matcher: ["/((?!.*\\..*|_next).*)", "/", "/(api|trpc)(.*)"],
}
