import { NextResponse, type NextRequest } from 'next/server';
import { updateSession } from '@/lib/supabase/middleware';

/**
 * Auth middleware. Two jobs:
 *
 *  1. Refresh the Supabase auth cookies so the server-side session never expires
 *     under the user while they browse.
 *  2. Cheap session-presence guard on /admin/* and /tutor/* — redirect to
 *     /login when no Supabase session is present. Role-level checks happen in
 *     the page Server Component (needs a DB lookup to resolve role).
 */
export async function middleware(request: NextRequest) {
  const { response, user } = await updateSession(request);
  const { pathname } = request.nextUrl;

  const isProtected =
    pathname === '/admin' ||
    pathname.startsWith('/admin/') ||
    pathname === '/tutor' ||
    pathname.startsWith('/tutor/');
  if (isProtected && !user) {
    const demoAllowed =
      process.env.NODE_ENV !== 'production' || process.env.ALLOW_DEMO_USER === 'true';
    if (demoAllowed && request.cookies.get('vielang_demo_user')?.value) {
      return response;
    }
    const url = request.nextUrl.clone();
    url.pathname = '/login';
    url.searchParams.set('redirect', pathname);
    return NextResponse.redirect(url);
  }

  return response;
}

export const config = {
  // Skip static assets and image optimization — only run on app routes.
  matcher: [
    '/((?!_next/static|_next/image|favicon.ico|images/|fonts/|.*\\.(?:svg|png|jpg|jpeg|gif|webp|ico|js|css)$).*)',
  ],
};
