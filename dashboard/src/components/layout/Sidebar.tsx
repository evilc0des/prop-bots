import React from 'react';
import { NavLink } from 'react-router-dom';
import { Activity, LayoutDashboard, Settings, PlayCircle, Shield } from 'lucide-react';

export const Sidebar: React.FC = () => {
    const navItems = [
        { to: '/', label: 'Overview', icon: LayoutDashboard },
        { to: '/strategy', label: 'Strategies', icon: Settings },
        { to: '/backtest', label: 'Backtests', icon: PlayCircle },
        { to: '/live', label: 'Live Monitor', icon: Activity },
        { to: '/rules', label: 'Prop Firm Rules', icon: Shield },
    ];

    return (
        <aside className="w-64 bg-gray-900 border-r border-gray-800 text-gray-300 h-screen flex flex-col">
            <div className="p-6">
                <h1 className="text-xl font-bold font-mono tracking-tight text-white flex items-center gap-2">
                    <Activity className="text-blue-500" />
                    PROP<span className="text-blue-500">BOTS</span>
                </h1>
            </div>

            <nav className="flex-1 py-4">
                <ul className="space-y-1">
                    {navItems.map((item) => (
                        <li key={item.to}>
                            <NavLink
                                to={item.to}
                                className={({ isActive }) =>
                                    `flex items-center gap-3 px-6 py-3 transition-colors ${isActive
                                        ? 'bg-blue-500/10 text-blue-500 border-r-4 border-blue-500'
                                        : 'hover:bg-gray-800 hover:text-white'
                                    }`
                                }
                            >
                                <item.icon size={20} />
                                <span className="font-medium">{item.label}</span>
                            </NavLink>
                        </li>
                    ))}
                </ul>
            </nav>
        </aside>
    );
};
