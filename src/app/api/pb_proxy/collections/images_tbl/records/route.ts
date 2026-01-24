import { NextRequest, NextResponse } from 'next/server'

/**
 * PocketBase Proxy API Route
 * Proxies image uploads to PocketBase server
 * This avoids CORS issues and centralizes the upload logic
 */

const POCKETBASE_URL = process.env.POCKETBASE_URL || 'https://pocketbase.vielang.com'

export async function POST(req: NextRequest) {
  try {
    const formData = await req.formData()

    const response = await fetch(
      `${POCKETBASE_URL}/api/collections/images_tbl/records`,
      {
        method: 'POST',
        body: formData,
      }
    )

    if (!response.ok) {
      const errorText = await response.text()
      console.error('PocketBase upload error:', errorText)
      return NextResponse.json(
        { error: 'Upload failed', details: errorText },
        { status: response.status }
      )
    }

    const data = await response.json()
    return NextResponse.json(data)
  } catch (error) {
    console.error('Proxy error:', error)
    return NextResponse.json(
      { error: 'Internal server error', message: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    )
  }
}
