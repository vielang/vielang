// Cookie management utilities for auth
export const setCookie = (name: string, value: string, days: number = 7) => {
  if (typeof window !== "undefined") {
    const isSecure = window.location.protocol === "https:"
    const sameSite = "strict"
    const maxAge = days * 24 * 60 * 60 // Convert days to seconds

    document.cookie = `${name}=${encodeURIComponent(value)}; path=/; max-age=${maxAge}; samesite=${sameSite}${isSecure ? "; secure" : ""}`
  }
}

export const getCookie = (name: string): string | null => {
  if (typeof window !== "undefined") {
    const cookies = document.cookie.split(";")

    for (const cookie of cookies) {
      const [cookieName, cookieValue] = cookie.trim().split("=")
      if (cookieName === name && cookieValue) {
        return decodeURIComponent(cookieValue)
      }
    }
  }
  return null
}

export const deleteCookie = (name: string) => {
  if (typeof window !== "undefined") {
    document.cookie = `${name}=; path=/; max-age=0`
  }
}
