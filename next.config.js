import nextra from 'nextra'

const withNextra = nextra({
  latex: true,
  search: {
    codeblocks: false
  },
  contentDirBasePath: '/docs',
  mdxOptions: {
    remarkPlugins: [],
    rehypePlugins: []
  }
})

const nextConfig = {
  pageExtensions: ['js', 'jsx', 'mdx', 'ts', 'tsx'],
  images: {
    formats: ["image/avif", "image/webp"],
    remotePatterns: [
      { protocol: "https", hostname: "**.githubassets.com" },
      { protocol: "https", hostname: "**.githubusercontent.com" },
      { protocol: "https", hostname: "**.googleusercontent.com" },
      { protocol: "https", hostname: "**.unsplash.com" },
      { protocol: "https", hostname: "**.digitaloceanspaces.com" },
      { protocol: "https", hostname: "api.github.com" },
      { protocol: "https", hostname: "**.vielang.com" },
    ],
  },
  eslint: { ignoreDuringBuilds: true },
  async rewrites() {
    return [
      {
        source: '/games/unhaunter/pkg/:path*',
        destination: '/games/unhaunter/pkg/:path*',
      },
    ];
  },
  experimental: {
    turbo: {
      resolveAlias: {
        "@": "./src",
      },
    },
  },
};

export default withNextra(nextConfig);
