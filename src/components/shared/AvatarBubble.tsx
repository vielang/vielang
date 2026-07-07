import Image from 'next/image';
import { initials } from '@/lib/format';
import { cn } from '@/lib/utils';

interface Props {
  src?: string | null;
  name?: string | null;
  size?: number;
  className?: string;
  ringClassName?: string;
  alt?: string;
}

/**
 * Circular avatar that falls back to two-letter initials on the brand color
 * when no image is available. Every place across the app that shows a user
 * photo goes through here so the sizing/ring/fallback style stays in sync.
 */
export function AvatarBubble({ src, name, size = 40, className, ringClassName, alt = '' }: Props) {
  const style = { width: size, height: size } as const;
  const ring = ringClassName ?? '';

  if (src) {
    return (
      <Image
        src={src}
        alt={alt}
        width={size}
        height={size}
        className={cn('rounded-full object-cover', ring, className)}
        style={style}
      />
    );
  }

  return (
    <div
      aria-hidden={alt === '' ? true : undefined}
      className={cn(
        'bg-brand flex items-center justify-center rounded-full font-semibold text-white',
        ring,
        className,
      )}
      style={{
        ...style,
        // Scale the initials with the bubble so a 24px avatar reads as clearly
        // as a 64px one instead of being drowned by a fixed font-size.
        fontSize: Math.round(size * 0.4),
      }}
    >
      {initials(name)}
    </div>
  );
}
