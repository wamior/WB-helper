import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
    title: "PVZ Helper - Админ Панель",
    description: "Управление пользователями PVZ Helper",
};

export default function RootLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang="ru">
            <body className="antialiased font-sans">
                {children}
            </body>
        </html>
    );
}
