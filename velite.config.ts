import rehypeAutolinkHeadings from "rehype-autolink-headings"
import rehypeCodeTitles from "rehype-code-titles"
import rehypePrettyCode from "rehype-pretty-code"
import rehypeSlug from "rehype-slug"
import { codeImport } from "remark-code-import"
import { defineCollection, defineConfig, s } from "velite"

const posts = defineCollection({
  name: "Post",
  pattern: "blog/**/*.mdx",
  schema: s
    .object({
      title: s.string(),
      description: s.string().optional(),
      date: s.isodate(),
      published: s.boolean().default(true),
      image: s.string(),
      authors: s.array(s.string()),
      body: s.mdx(),
      path: s.path(),
      // Define slug, slugAsParams, and readingTime as schema fields as optional
      slug: s.string().optional(),
      slugAsParams: s.string().optional(),
      readingTime: s.number().optional(),
    })
    .transform((data) => {
      const wordsPerMinute = 200
      const numberOfWords = data.body ? data.body.split(/\s/g).length : 0
      const minutes = numberOfWords / wordsPerMinute
      const slug = data.path.replace(/^blog\//, "")
      return {
        ...data,
        slug: `/blog/${slug}`,
        slugAsParams: slug,
        readingTime: Math.ceil(minutes),
      }
    }),
})

const authors = defineCollection({
  name: "Author",
  pattern: "authors/**/*.mdx",
  schema: s
    .object({
      title: s.string(),
      description: s.string().optional(),
      avatar: s.string(),
      twitter: s.string(),
      body: s.mdx(),
      path: s.path(),
      // Define slug and slugAsParams as schema fields as optional
      slug: s.string().optional(),
      slugAsParams: s.string().optional(),
    })
    .transform((data) => ({
      ...data,
      slug: data.path.replace(/^authors\//, ""),
      slugAsParams: data.path
        .replace(/^authors\//, "")
        .split("/")
        .slice(1)
        .join("/"),
    })),
})

const pages = defineCollection({
  name: "Page",
  pattern: "pages/**/*.mdx",
  schema: s
    .object({
      title: s.string(),
      description: s.string().optional(),
      body: s.mdx(),
      path: s.path(),
      // Define slug and slugAsParams as schema fields as optional
      slug: s.string().optional(),
      slugAsParams: s.string().optional(),
    })
    .transform((data) => ({
      ...data,
      slug: data.path.replace(/^pages\//, ""),
      slugAsParams: data.path
        .replace(/^pages\//, "")
        .split("/")
        .slice(1)
        .join("/"),
    })),
})

export default defineConfig({
  root: "src/content",
  collections: { posts, authors, pages },
  mdx: {
    remarkPlugins: [codeImport],
    rehypePlugins: [
      rehypeSlug,
      [rehypePrettyCode, { theme: "github-dark" }],
      [
        rehypeAutolinkHeadings,
        {
          properties: {
            className: ["subheading-anchor"],
            ariaLabel: "Link to section",
          },
        },
      ],
      rehypeCodeTitles,
    ],
  },
})
