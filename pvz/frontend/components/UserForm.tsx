'use client';

import { useState, useEffect } from 'react';
import api from '@/lib/api';
import { X, RefreshCw, Eye, EyeOff } from 'lucide-react';

interface Props {
    user: any | null;
    onClose: () => void;
    onSuccess: () => void;
}

export default function UserForm({ user, onClose, onSuccess }: Props) {
    const [formData, setFormData] = useState({
        login: '',
        password: '',
        fullName: '',
        phone: '',
        isAdmin: false
    });
    const [showPassword, setShowPassword] = useState(false);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState('');

    useEffect(() => {
        if (user) {
            setFormData({
                login: user.login || '',
                password: '', // Пароль не редактируется напрямую, если пусто - не меняется
                fullName: user.fullName || '',
                phone: user.phone || '',
                isAdmin: !!user.isAdmin
            });
        }
    }, [user]);

    const generatePassword = () => {
        const chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+";
        let password = "";
        for (let i = 0; i < 12; i++) {
            password += chars.charAt(Math.floor(Math.random() * chars.length));
        }
        setFormData({ ...formData, password });
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        setError('');

        try {
            if (user) {
                await api.put(`/users/${user.id}`, formData);
            } else {
                await api.post('/users', formData);
            }
            onSuccess();
        } catch (err: any) {
            setError(err.response?.data?.error || 'Произошла ошибка');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
            <div className="card w-full max-w-lg shadow-2xl animate-in fade-in zoom-in duration-200">
                <div className="flex justify-between items-center mb-6">
                    <h2 className="text-xl font-bold">{user ? 'Редактировать пользователя' : 'Добавить пользователя'}</h2>
                    <button onClick={onClose} className="text-slate-400 hover:text-white transition-colors">
                        <X size={24} />
                    </button>
                </div>

                {error && (
                    <div className="bg-red-900/50 border border-red-700 text-red-200 p-3 rounded-lg mb-4 text-sm">
                        {error}
                    </div>
                )}

                <form onSubmit={handleSubmit} className="space-y-4">
                    <div className="grid grid-cols-1 gap-4">
                        <div>
                            <label className="block text-sm font-medium mb-1 text-slate-400">Логин</label>
                            <input
                                type="text"
                                className="input w-full"
                                value={formData.login}
                                onChange={(e) => setFormData({ ...formData, login: e.target.value })}
                                required
                            />
                        </div>

                        <div className="relative">
                            <label className="block text-sm font-medium mb-1 text-slate-400">Пароль {user && '(оставьте пустым, чтобы не менять)'}</label>
                            <div className="flex gap-2">
                                <div className="relative flex-1">
                                    <input
                                        type={showPassword ? "text" : "password"}
                                        className="input w-full pr-10"
                                        value={formData.password}
                                        onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                                        required={!user}
                                    />
                                    <button
                                        type="button"
                                        onClick={() => setShowPassword(!showPassword)}
                                        className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300"
                                    >
                                        {showPassword ? <EyeOff size={18} /> : <Eye size={18} />}
                                    </button>
                                </div>
                                <button
                                    type="button"
                                    onClick={generatePassword}
                                    className="btn btn-secondary flex items-center gap-2 shrink-0"
                                    title="Сгенерировать"
                                >
                                    <RefreshCw size={18} /> Сгенерировать
                                </button>
                            </div>
                        </div>

                        <div>
                            <label className="block text-sm font-medium mb-1 text-slate-400">ФИО полностью</label>
                            <input
                                type="text"
                                className="input w-full"
                                placeholder="Иванов Иван Иванович"
                                value={formData.fullName}
                                onChange={(e) => setFormData({ ...formData, fullName: e.target.value })}
                                required
                            />
                        </div>

                        <div>
                            <label className="block text-sm font-medium mb-1 text-slate-400">Номер телефона</label>
                            <input
                                type="text"
                                className="input w-full"
                                placeholder="79001112233"
                                value={formData.phone}
                                onChange={(e) => setFormData({ ...formData, phone: e.target.value })}
                                required
                            />
                        </div>

                        <div className="flex items-center gap-3 p-3 bg-slate-800/50 rounded-lg border border-slate-700">
                            <input
                                type="checkbox"
                                id="isAdmin"
                                className="w-5 h-5 rounded bg-slate-900 border-slate-700 text-blue-600 focus:ring-blue-500"
                                checked={formData.isAdmin}
                                onChange={(e) => setFormData({ ...formData, isAdmin: e.target.checked })}
                            />
                            <label htmlFor="isAdmin" className="flex-1 cursor-pointer">
                                <span className="block font-medium">Права администратора</span>
                                <span className="text-xs text-slate-400">Разрешить доступ к этой панели управления</span>
                            </label>
                        </div>
                    </div>

                    <div className="flex gap-3 pt-4">
                        <button
                            type="button"
                            onClick={onClose}
                            className="btn btn-secondary flex-1"
                        >
                            Отмена
                        </button>
                        <button
                            type="submit"
                            disabled={loading}
                            className="btn btn-primary flex-1"
                        >
                            {loading ? 'Сохранение...' : (user ? 'Сохранить изменения' : 'Создать пользователя')}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
