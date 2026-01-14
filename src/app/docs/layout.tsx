import { Footer, Layout } from 'nextra-theme-docs'
import { getPageMap } from 'nextra/page-map'
import type { Metadata } from 'next'
import Link from 'next/link'
import '@/styles/docs.css'
import { CustomNavbar } from '../../components/common/custom-navbar';

export const metadata: Metadata = {
  metadataBase: new URL('https://vielang.com'),
  title: {
    template: '%s - VieLang Docs',
    default: 'VieLang Documentation'
  },
  description: 'VieLang Documentation with Nextra',
  applicationName: 'VieLang Docs'
}

interface DocsLayoutProps {
  children: React.ReactNode
}

const CURRENT_YEAR = new Date().getFullYear()

export default async function DocsLayout({ children }: DocsLayoutProps) {
  const pageMap = await getPageMap()

  const excludePages = ['posts', 'auth', 'profile', 'games', 'image-editor', 'video-generator', 'korean']

  const filteredPageMap = pageMap
    .filter((item: any) => !excludePages.includes(item.name.toLowerCase()))
    .map((item: any, index: number) => ({
      ...item,
      key: item.route || item.name || `page-${index}`
    }))

  return (
    <>
      <Layout
        navbar={<CustomNavbar />}
        footer={<Footer>MIT {CURRENT_YEAR} © VieLang.</Footer>}
        sidebar={{ defaultMenuCollapseLevel: 1 }}
        pageMap={filteredPageMap}
        feedback={{ content: null }}
        editLink={null}
      >
        <div data-pagefind-body>
          {children}
        </div>
      </Layout>
    </>
  )
}
