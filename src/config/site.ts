import type { FooterItem, MainNavItem } from "@/types"

import { productConfig } from "@/config/product"
import { slugify } from "@/lib/utils"

export type SiteConfig = typeof siteConfig

const links = {
  x: "https://github.com/vielang",
  github: "https://github.com/vielang/vielang",
  githubAccount: "https://github.com/vielang",
  discord: "https://github.com/vielang",
  calDotCom: "https://github.com/vielang",
}

export const siteConfig = {
  name: "Vielang",
  description:
    "Your social platform for connecting and sharing",
  url: "https://vielang.com",
  ogImage: "https://vielang.com/opengraph-image.png",
  links,
  mainNav: [
    {
      title: "Explore",
      items: [
        {
          title: "Home",
          href: "/",
          description: "Vielang home page.",
          items: [],
        },
        {
          title: "Posts",
          href: "/posts",
          description: "Browse all posts.",
          items: [],
        },
        {
          title: "Trending",
          href: "/posts?sort=trending",
          description: "See what's trending.",
          items: [],
        },
        {
          title: "Categories",
          href: "/categories",
          description: "Explore by category.",
          items: [],
        },
      ],
    },
    {
      title: "Account",
      items: [
        {
          title: "Profile",
          href: "/profile",
          description: "View your profile.",
          items: [],
        },
        {
          title: "Settings",
          href: "/settings",
          description: "Manage your settings.",
          items: [],
        },
        {
          title: "Notifications",
          href: "/notifications",
          description: "View your notifications.",
          items: [],
        },
      ],
    },
  ] satisfies MainNavItem[],
  footerNav: [
    {
      title: "Social",
      items: [
        {
          title: "X",
          href: links.x,
          external: true,
        },
        {
          title: "GitHub",
          href: links.githubAccount,
          external: true,
        },
        {
          title: "Discord",
          href: links.discord,
          external: true,
        },
        {
          title: "cal.com",
          href: links.calDotCom,
          external: true,
        },
      ],
    },
  ] satisfies FooterItem[],
}
