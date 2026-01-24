import { ImageResponse } from "next/og"

import { ogImageSchema } from "@/lib/validations/og"

export const runtime = "edge"

export async function GET(req: Request) {
  try {
    const calSemiBoldData = await fetch(
      new URL("../../../assets/fonts/CalSans-SemiBold.woff2", import.meta.url)
    ).then((res) => res.arrayBuffer())

    const url = new URL(req.url)
    const parsedValues = ogImageSchema.parse(
      Object.fromEntries(url.searchParams)
    )

    const { mode, title, description, type } = parsedValues

    return new ImageResponse(
      (
        <div
          tw="flex size-full flex-col items-center justify-center"
          style={{
            color: mode === "dark" ? "#fff" : "#000",
            background: mode === "dark" ? "#09090b" : "#ffffff",
          }}
        >
          <div
            tw="flex max-w-4xl flex-col items-center justify-center"
            style={{
              whiteSpace: "pre-wrap",
            }}
          >
            {type ? (
              <div tw="px-8 text-xl leading-tight font-medium tracking-tight uppercase">
                {type}
              </div>
            ) : null}
            <h1
              tw="px-8 text-6xl leading-tight font-bold tracking-tight"
              style={{
                fontFamily: "CalSans",
                color: mode === "dark" ? "#f4f4f5" : "#27272a",
              }}
            >
              {title}
            </h1>
            {description ? (
              <p
                tw="px-20 text-center text-3xl leading-tight font-normal tracking-tight"
                style={{
                  color: mode === "dark" ? "#a1a1aa" : "#71717a",
                }}
              >
                {description}
              </p>
            ) : null}
          </div>
        </div>
      ),
      {
        width: 1200,
        height: 630,
        fonts: [
          {
            name: "CalSans",
            data: calSemiBoldData,
            style: "normal",
          },
        ],
      }
    )
  } catch (error) {
    return new Response(`Failed to generate the image`, {
      status: 500,
    })
  }
}
