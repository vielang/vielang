import Link from "next/link"

import { siteConfig } from "@/config/site"
import { cn } from "@/lib/utils"
import { buttonVariants } from "@/components/ui/button"
import { Icons } from "@/components/icons"
import { ModeToggle } from "@/components/layouts/mode-toggle"
import { Logo } from "@/components/logo"
import { Shell } from "@/components/shell"

interface SiteFooterProps {
  hidden?: boolean
}

export function SiteFooter({ hidden = false }: SiteFooterProps) {
  if (hidden) {
    return null
  }

  return (
    <footer className="bg-background w-full border-t">
      <Shell>
        <section className="flex flex-col gap-10 lg:flex-row lg:gap-20">
          <section>
            <Link href="/" className="flex w-fit items-center space-x-2">
              <Logo className="size-6" aria-hidden="true" />
              <span className="font-bold">{siteConfig.name}</span>
              <span className="sr-only">Home</span>
            </Link>
          </section>
          <section className="xxs:grid-cols-2 grid flex-1 grid-cols-1 gap-10 sm:grid-cols-4">
            {siteConfig.footerNav.map((item) => (
              <div key={item.title} className="space-y-3">
                <h4 className="text-base font-medium">{item.title}</h4>
                <ul className="space-y-2.5">
                  {item.items.map((link) => (
                    <li key={link.title}>
                      <Link
                        href={link.href}
                        target={link?.external ? "_blank" : undefined}
                        rel={link?.external ? "noreferrer" : undefined}
                        className="text-muted-foreground hover:text-foreground text-sm transition-colors"
                      >
                        {link.title}
                        <span className="sr-only">{link.title}</span>
                      </Link>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </section>
        </section>
        <section className="flex items-center space-x-4">
          <div className="text-muted-foreground flex-1 text-left text-sm leading-loose">
            Built by{" "}
            <Link
              href="https://github.com/vielang"
              target="_blank"
              rel="noreferrer"
              className="hover:text-foreground font-semibold transition-colors"
            >
              khieu-dv
              <span className="sr-only">Twitter</span>
            </Link>
            .
          </div>
          <div className="flex items-center space-x-1">
            <Link
              href={siteConfig.links.github}
              target="_blank"
              rel="noreferrer"
              className={cn(
                buttonVariants({
                  size: "icon",
                  variant: "ghost",
                })
              )}
            >
              <Icons.gitHub className="size-4" aria-hidden="true" />
              <span className="sr-only">GitHub</span>
            </Link>
            <ModeToggle />
          </div>
        </section>
      </Shell>
    </footer>
  )
}
