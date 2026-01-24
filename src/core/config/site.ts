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
    "Your social platform for connecting and sharing.",
  url: "https://vielang.com",
  ogImage: "https://vielang.com/opengraph-image.png",
  links,
  mainNav: [
    {
      title: "Learning",
      items: [
        {
          title: "Home",
          href: "/",
          description: "Korean Learning home page.",
          items: [],
        },
        {
          title: "Dashboard",
          href: "/korean/dashboard",
          description: "Your learning progress and statistics.",
          items: [],
        },
        {
          title: "Courses",
          href: "/korean/courses",
          description: "Browse and enroll in Korean courses.",
          items: [],
        },
        {
          title: "Vocabulary",
          href: "/korean/vocabulary",
          description: "Learn and practice Korean vocabulary.",
          items: [],
        },
        {
          title: "Tests",
          href: "/korean/tests",
          description: "Take practice tests and assessments.",
          items: [],
        },
        {
          title: "TOPIK Exams",
          href: "/topik",
          description: "Practice with official TOPIK exam papers.",
          items: [],
        },
      ],
    },
    {
      title: "Community",
      items: [
        {
          title: "Leaderboard",
          href: "/korean/leaderboard",
          description: "See top learners and your ranking.",
          items: [],
        },
        {
          title: "Achievements",
          href: "/korean/achievements",
          description: "View your certificates and achievements.",
          items: [],
        },
        {
          title: "Blog",
          href: "/blog",
          description: "Read our latest blog posts and tips.",
          items: [],
        },
      ],
    },
  ] satisfies MainNavItem[],
  footerNav: [
    {
      title: "Help",
      items: [
        {
          title: "About",
          href: "/about",
          external: false,
        },
        {
          title: "Contact",
          href: "/contact",
          external: false,
        },
        {
          title: "Terms",
          href: "/terms",
          external: false,
        },
        {
          title: "Privacy",
          href: "/privacy",
          external: false,
        },
      ],
    },
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
