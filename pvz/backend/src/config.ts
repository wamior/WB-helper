import fs from 'fs-extra';
import path from 'path';

const USERS_FILE = path.resolve(__dirname, '../../users.json');

export interface User {
    id: string;
    login: string;
    passwordHash: string;
    fullName: string;
    phone: string;
    isAdmin: boolean;
}

export const loadUsers = (): User[] => {
    if (fs.existsSync(USERS_FILE)) {
        return fs.readJSONSync(USERS_FILE);
    }
    return [];
};

export const saveUsers = (users: User[]) => {
    fs.writeJSONSync(USERS_FILE, users, { spaces: 2 });
};

export const findUserByLogin = (login: string): User | undefined => {
    return loadUsers().find(u => u.login === login);
};
