import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import axios from 'axios';

export async function POST(request: Request) {
    try {
        const body = await request.json();
        const { login, password } = body;

        // Call external backend
        const backendRes = await axios.post('http://localhost:3001/auth/login', {
            login,
            password
        });

        const { token, user } = backendRes.data;

        // Set cookie
        const cookieStore = await cookies();
        cookieStore.set('token', token, {
            httpOnly: true,
            secure: process.env.NODE_ENV === 'production',
            sameSite: 'strict',
            maxAge: 60 * 60 * 24, // 1 day
            path: '/',
        });

        return NextResponse.json({ user });
    } catch (error: any) {
        const status = error.response?.status || 500;
        const message = error.response?.data?.error || 'Internal Server Error';
        return NextResponse.json({ error: message }, { status });
    }
}
