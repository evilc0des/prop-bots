import React from 'react';
import { Sidebar } from './Sidebar';

export const DashboardLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    return (
        <div className="flex h-screen bg-gray-950 text-gray-100 overflow-hidden">
            <Sidebar />
            <div className="flex-1 flex flex-col overflow-hidden">
                <header className="h-16 border-b border-gray-800 bg-gray-900/50 flex items-center px-6">
                    <div className="text-sm font-medium text-gray-400">System Status: <span className="text-emerald-500">● Online</span></div>
                </header>
                <main className="flex-1 overflow-auto p-6">
                    {children}
                </main>
            </div>
        </div>
    );
};
