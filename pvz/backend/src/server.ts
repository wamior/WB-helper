import express from 'express';
import cors from 'cors';
import bodyParser from 'body-parser';
import bcrypt from 'bcryptjs';
import { v4 as uuidv4 } from 'uuid';
import { loadUsers, saveUsers, findUserByLogin, User } from './config';
import { authMiddleware, generateToken } from './auth';

const app = express();
const PORT = process.env.PORT || 3001;

app.use(cors());
app.use(bodyParser.json());

// Инициализация первого админа если пользователей нет
const initAdmin = async () => {
    const users = loadUsers();
    if (users.length === 0) {
        const salt = await bcrypt.genSalt(10);
        const passwordHash = await bcrypt.hash('admin', salt);
        users.push({
            id: 'admin-id',
            login: 'admin',
            passwordHash,
            fullName: 'Администратор системы',
            phone: '8000000000',
            isAdmin: true
        });
        saveUsers(users);
        console.log('Создан стандартный администратор: admin / admin');
    }
};
initAdmin();

// Логин
app.post('/auth/login', async (req, res) => {
    const { login, password } = req.body;

    if (!login || !password) {
        res.status(400).json({ error: 'Введите логин и пароль' });
        return;
    }

    const user = findUserByLogin(login);
    if (!user) {
        res.status(401).json({ error: 'Неверные данные для входа' });
        return;
    }

    const isMatch = await bcrypt.compare(password, user.passwordHash);
    if (!isMatch) {
        res.status(401).json({ error: 'Неверные данные для входа' });
        return;
    }

    if (!user.isAdmin) {
        res.status(403).json({ error: 'У вас нет прав администратора' });
        return;
    }

    const token = generateToken({ id: user.id, login: user.login, isAdmin: user.isAdmin });
    res.json({ token, user: { id: user.id, login: user.login, fullName: user.fullName, isAdmin: user.isAdmin } });
});

// Защищенные API маршруты
app.use('/api', authMiddleware);

// Список пользователей
app.get('/api/users', (req, res) => {
    const users = loadUsers().map(u => {
        const { passwordHash, ...rest } = u;
        return rest;
    });
    res.json(users);
});

// Добавление пользователя
app.post('/api/users', async (req, res) => {
    const { login, password, fullName, phone, isAdmin } = req.body;

    if (!login || !password || !fullName || !phone) {
        res.status(400).json({ error: 'Заполните все поля' });
        return;
    }

    const users = loadUsers();
    if (users.find(u => u.login === login)) {
        res.status(400).json({ error: 'Пользователь с таким логином уже существует' });
        return;
    }

    const salt = await bcrypt.genSalt(10);
    const passwordHash = await bcrypt.hash(password, salt);

    const newUser: User = {
        id: uuidv4(),
        login,
        passwordHash,
        fullName,
        phone,
        isAdmin: !!isAdmin
    };

    users.push(newUser);
    saveUsers(users);

    res.json({ success: true });
});

// Редактирование пользователя
app.put('/api/users/:id', async (req, res) => {
    const { id } = req.params;
    const { login, password, fullName, phone, isAdmin } = req.body;

    let users = loadUsers();
    const userIndex = users.findIndex(u => u.id === id);

    if (userIndex === -1) {
        res.status(404).json({ error: 'Пользователь не найден' });
        return;
    }

    if (login && login !== users[userIndex].login) {
        if (users.find(u => u.login === login)) {
            res.status(400).json({ error: 'Логин уже занят' });
            return;
        }
        users[userIndex].login = login;
    }

    if (password) {
        const salt = await bcrypt.genSalt(10);
        users[userIndex].passwordHash = await bcrypt.hash(password, salt);
    }

    if (fullName !== undefined) users[userIndex].fullName = fullName;
    if (phone !== undefined) users[userIndex].phone = phone;
    if (isAdmin !== undefined) users[userIndex].isAdmin = isAdmin;

    saveUsers(users);
    res.json({ success: true });
});

// Удаление пользователя
app.delete('/api/users/:id', (req, res) => {
    const { id } = req.params;
    let users = loadUsers();

    // Предотвращение удаления самого себя если нужно (опционально)
    // if ((req as any).user.id === id) {
    //     res.status(400).json({ error: 'Нельзя удалить самого себя' });
    //     return;
    // }

    const newUsers = users.filter(u => u.id !== id);
    saveUsers(newUsers);
    res.json({ success: true });
});

app.listen(PORT, () => {
    console.log(`Сервер запущен на порту ${PORT}`);
});
