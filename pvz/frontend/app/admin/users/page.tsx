'use client';

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import api from '@/lib/api';
import axios from 'axios';
import { Trash2, UserPlus, Shield, ShieldAlert, LogOut, Phone, User as UserIcon, RefreshCw } from 'lucide-react';
import UserForm from '@/components/UserForm';

interface User {
    id: string;
    login: string;
    fullName: string;
    phone: string;
    isAdmin: boolean;
}

export default function UsersPage() {
    const [users, setUsers] = useState<User[]>([]);
    const [loading, setLoading] = useState(true);
    const [showModal, setShowModal] = useState(false);
    const [editingUser, setEditingUser] = useState<User | null>(null);
    const router = useRouter();

    const fetchUsers = async () => {
        try {
            const res = await api.get('/users');
            setUsers(res.data);
        } catch (err: any) {
            if (err.response?.status === 401 || err.response?.status === 403) {
                router.push('/login');
            }
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchUsers();
    }, []);

    const handleDelete = async (id: string) => {
        if (!confirm('Вы уверены, что хотите удалить этого пользователя?')) return;
        try {
            await api.delete(`/users/${id}`);
            setUsers(users.filter(u => u.id !== id));
        } catch (err) {
            alert('Ошибка при удалении');
        }
    };

    const handleLogout = async () => {
        try {
            await axios.post('/api/auth/logout');
            localStorage.clear();
            router.push('/login');
            router.refresh();
        } catch (e) {
            console.error('Logout failed', e);
        }
    };

    return (
        <div className="min-h-screen bg-slate-950 text-slate-200">
            {/* Navbar */}
            <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-md sticky top-0 z-10 px-6 py-4 flex justify-between items-center">
                <h1 className="text-xl font-bold flex items-center gap-2">
                    <Shield className="text-blue-500" /> PVZ Helper: Админка
                </h1>
                <div className="flex items-center gap-4">
                    <button onClick={fetchUsers} className="p-2 hover:bg-slate-800 rounded-full transition-colors">
                        <RefreshCw size={20} className={loading ? 'animate-spin' : ''} />
                    </button>
                    <button onClick={handleLogout} className="btn border border-slate-700 hover:bg-slate-800 flex items-center gap-2">
                        <LogOut size={18} /> Выйти
                    </button>
                </div>
            </nav>

            <main className="p-8 max-w-6xl mx-auto">
                <div className="flex justify-between items-center mb-8">
                    <div>
                        <h2 className="text-3xl font-bold">Управление пользователями</h2>
                        <p className="text-slate-400 mt-1">Всего пользователей: {users.length}</p>
                    </div>
                    <button
                        onClick={() => { setEditingUser(null); setShowModal(true); }}
                        className="btn btn-primary flex items-center gap-2"
                    >
                        <UserPlus size={20} /> Добавить пользователя
                    </button>
                </div>

                {loading ? (
                    <div className="flex justify-center p-12">
                        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
                    </div>
                ) : (
                    <div className="card overflow-hidden p-0">
                        <table className="w-full text-left">
                            <thead className="bg-slate-800/50 text-slate-400 text-sm uppercase">
                                <tr>
                                    <th className="px-6 py-4 font-semibold">Пользователь</th>
                                    <th className="px-6 py-4 font-semibold">ФИО</th>
                                    <th className="px-6 py-4 font-semibold">Телефон</th>
                                    <th className="px-6 py-4 font-semibold text-center">Роль</th>
                                    <th className="px-6 py-4 font-semibold text-right">Действия</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-slate-800">
                                {users.map((user) => (
                                    <tr key={user.id} className="hover:bg-slate-800/30 transition-colors">
                                        <td className="px-6 py-4">
                                            <div className="flex items-center gap-3">
                                                <div className="w-10 h-10 rounded-full bg-slate-800 flex items-center justify-center text-blue-400">
                                                    <UserIcon size={20} />
                                                </div>
                                                <span className="font-medium text-white">{user.login}</span>
                                            </div>
                                        </td>
                                        <td className="px-6 py-4 text-slate-300">{user.fullName}</td>
                                        <td className="px-6 py-4">
                                            <div className="flex items-center gap-2 text-slate-400">
                                                <Phone size={14} /> {user.phone}
                                            </div>
                                        </td>
                                        <td className="px-6 py-4">
                                            <div className="flex justify-center">
                                                {user.isAdmin ? (
                                                    <span className="flex items-center gap-1.5 px-3 py-1 rounded-full bg-blue-900/30 text-blue-300 text-xs font-bold border border-blue-800">
                                                        <Shield size={12} /> Админ
                                                    </span>
                                                ) : (
                                                    <span className="px-3 py-1 rounded-full bg-slate-800 text-slate-400 text-xs font-medium border border-slate-700">
                                                        Юзер
                                                    </span>
                                                )}
                                            </div>
                                        </td>
                                        <td className="px-6 py-4 text-right">
                                            <div className="flex justify-end gap-2">
                                                <button
                                                    onClick={() => { setEditingUser(user); setShowModal(true); }}
                                                    className="p-2 hover:bg-blue-900/40 text-blue-400 rounded-lg transition-colors"
                                                >
                                                    Изменить
                                                </button>
                                                <button
                                                    onClick={() => handleDelete(user.id)}
                                                    className="p-2 hover:bg-red-900/40 text-red-400 rounded-lg transition-colors"
                                                >
                                                    <Trash2 size={18} />
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </main>

            {showModal && (
                <UserForm
                    user={editingUser}
                    onClose={() => setShowModal(false)}
                    onSuccess={() => { fetchUsers(); setShowModal(false); }}
                />
            )}
        </div>
    );
}
