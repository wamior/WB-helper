import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export function middleware(request: NextRequest) {
    const token = request.cookies.get('token')?.value;
    const isLoginPage = request.nextUrl.pathname === '/login';
    const isAdminPage = request.nextUrl.pathname.startsWith('/admin');

    // If trying to access admin page without token, redirect to login
    if (isAdminPage && !token) {
        return NextResponse.redirect(new URL('/login', request.url));
    }

    // If already logged in and trying to access login, redirect to admin
    if (isLoginPage && token) {
        return NextResponse.redirect(new URL('/admin/users', request.url));
    }

    return NextResponse.next();
}

export const config = {
    matcher: ['/admin/:path*', '/login'],
};
