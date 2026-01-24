"use client"

import Image, { ImageProps } from "next/image"

export function Logo({
  className,
  ...props
}: Omit<ImageProps, "src" | "alt" | "width" | "height"> & {
  className?: string
}) {
  return (
    <Image
      src="/icon.png"
      alt="Logo"
      width={24} // You might need to adjust these based on your design
      height={24} // You might need to adjust these based on your design
      className={className}
      {...props}
    />
  )
}
