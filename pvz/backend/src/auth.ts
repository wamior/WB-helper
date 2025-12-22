import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';

const JWT_SECRET = process.env.JWT_SECRET || 'pvz-helper-secret-key';

export interface AuthRequest extends Request {
    user?: {
        id: string;
        login: string;
        isAdmin: boolean;
    };
}

export const authMiddleware = (req: AuthRequest, res: Response, next: NextFunction) => {
    if (req.method === 'OPTIONS') return next();

    const authHeader = req.headers.authorization;
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
        res.status(401).json({ error: 'Auth failed: Missing token' });
        return;
    }

    const token = authHeader.split(' ')[1];

    try {
        const decoded = jwt.verify(token, JWT_SECRET) as any;
        req.user = decoded;

        if (!decoded.isAdmin) {
            res.status(403).json({ error: 'Доступ запрещен: Требуются права администратора' });
            return;
        }

        next();
    } catch (e) {
        res.status(401).json({ error: 'Auth failed: Invalid token' });
    }
};

export const generateToken = (payload: object) => {
    return jwt.sign(payload, JWT_SECRET, { expiresIn: '24h' });
};
