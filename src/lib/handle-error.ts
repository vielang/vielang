import { toast } from "sonner"
import * as z from "zod"

import { unknownError } from "@/lib/constants"

export function getErrorMessage(err: unknown) {
  if (err instanceof z.ZodError) {
    return err.errors[0]?.message ?? unknownError
  } else if (err instanceof Error) {
    return err.message
  } else if (typeof err === "string") {
    return err
  } else if (err && typeof err === "object" && "message" in err) {
    return String((err as any).message)
  } else {
    return unknownError
  }
}

export function showErrorToast(err: unknown) {
  const errorMessage = getErrorMessage(err)
  console.log({ errorMessage })

  return toast.error(errorMessage)
}
