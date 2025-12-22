import axios from 'axios';

// Calls go to Next.js API routes (Proxy)
const api = axios.create({
    baseURL: '/api/proxy',
});

// No interceptors needed for token, as the cookie is handled by the browser/Next.js
export default api;
