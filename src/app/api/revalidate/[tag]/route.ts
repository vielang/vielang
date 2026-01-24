import { revalidateTag } from "next/cache"
import { env } from "@/env"
import { z } from "zod"

const schema = z.object({
  params: z.promise(
    z.object({
      tag: z.string(),
    })
  ),
})

export async function POST(req: Request, context: z.infer<typeof schema>) {
  if (env.NODE_ENV !== "development") {
    return new Response("Not allowed", { status: 403 })
  }

  const { params: paramsPromise } = schema.parse(context)

  const { tag } = await paramsPromise

  revalidateTag(tag, {})

  return new Response(`revalidated: ${tag}`, { status: 200 })
}
