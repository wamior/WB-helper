import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import axios from 'axios';

const BACKEND_URL = 'http://localhost:3001/api';

async function proxyRequest(request: Request, { params }: { params: { path: string[] } }) {
    const path = params.path.join('/');
    const url = `${BACKEND_URL}/${path}`;

    const cookieStore = await cookies();
    const token = cookieStore.get('token')?.value;

    const headers: any = {
        'Content-Type': 'application/json',
    };

    if (token) {
        headers['Authorization'] = `Bearer ${token}`;
    }

    try {
        const body = request.method !== 'GET' && request.method !== 'DELETE' ? await request.json() : undefined;

        const response = await axios({
            method: request.method,
            url,
            data: body,
            headers,
            validateStatus: () => true, // Handle errors manually
        });

        return NextResponse.json(response.data, { status: response.status });
    } catch (error: any) {
        console.error('Proxy Error:', error.message);
        return NextResponse.json({ error: 'Proxy Error' }, { status: 500 });
    }
}

export async function GET(req: Request, ctx: any) { return proxyRequest(req, ctx); }
export async function POST(req: Request, ctx: any) { return proxyRequest(req, ctx); }
export async function PUT(req: Request, ctx: any) { return proxyRequest(req, ctx); }
export async function DELETE(req: Request, ctx: any) { return proxyRequest(req, ctx); }
