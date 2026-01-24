import Link from "next/link"

import { type getUserPlanMetrics } from "@/lib/queries/user"
import { cn } from "@/lib/utils"
import { buttonVariants } from "@/components/ui/button"

interface RateLimitAlertProps extends React.HTMLAttributes<HTMLDivElement> {
  planMetrics: Awaited<ReturnType<typeof getUserPlanMetrics>>
}

export function RateLimitAlert({
  planMetrics,
  className,
  ...props
}: RateLimitAlertProps) {
  const {
    storeLimit,
    productLimit,
    storeLimitExceeded,
    productLimitExceeded,
    subscriptionPlan,
  } = planMetrics

  return (
    <div className={cn("space-y-4", className)} {...props}>
      {storeLimitExceeded && (
        <div className="text-muted-foreground text-sm">
          You&apos;ve reached the limit of{" "}
          <span className="font-bold">{storeLimit}</span> stores for the{" "}
          <span className="font-bold">{subscriptionPlan}</span> plan.
        </div>
      )}
      {productLimitExceeded && (
        <div className="text-muted-foreground text-sm">
          You&apos;ve reached the limit of{" "}
          <span className="font-bold">{productLimit}</span> products for the{" "}
          <span className="font-bold">{subscriptionPlan}</span> plan.
        </div>
      )}
      {subscriptionPlan ? (
        subscriptionPlan === "pro" ? (
          <Link
            href="https://cal.com/khieu-dv/15min"
            target="_blank"
            rel="noopener noreferrer"
            className={buttonVariants({ className: "w-full" })}
          >
            Contact us
          </Link>
        ) : (
          <div className="text-muted-foreground text-sm">
            Plan management temporarily disabled
          </div>
        )
      ) : null}
    </div>
  )
}
