'use client'

import dynamic from 'next/dynamic'
import type { TiptapEditorProps } from './TiptapEditor'
import { Skeleton } from '@/components/ui/skeleton'

/**
 * Client-side wrapper for TiptapEditor to handle SSR
 *
 * This component uses dynamic import with ssr: false to ensure
 * the Tiptap editor only loads on the client side, avoiding
 * SSR-related issues with Next.js.
 */
const TiptapEditor = dynamic<TiptapEditorProps>(
  () => import('./TiptapEditor').then((mod) => mod.TiptapEditor),
  {
    ssr: false,
    loading: () => (
      <div className="space-y-3">
        <Skeleton className="h-12 w-full" />
        <Skeleton className="h-[500px] w-full" />
      </div>
    ),
  }
)

/**
 * Client wrapper component that passes all props to the dynamically loaded editor
 */
export function TiptapEditorClient(props: TiptapEditorProps) {
  return <TiptapEditor {...props} />
}

export default TiptapEditorClient
