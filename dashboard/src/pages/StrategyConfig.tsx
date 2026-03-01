import React from 'react';

const StrategyConfig: React.FC = () => {
    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center">
                <h2 className="text-2xl font-bold tracking-tight">Strategy Configurator</h2>
                <button className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors">
                    New Strategy
                </button>
            </div>
            <div className="bg-gray-900 border border-gray-800 rounded-xl overflow-hidden">
                <table className="w-full text-left">
                    <thead className="bg-gray-800/50 text-gray-400 text-sm">
                        <tr>
                            <th className="px-6 py-4 font-medium">Name</th>
                            <th className="px-6 py-4 font-medium">Type</th>
                            <th className="px-6 py-4 font-medium">Status</th>
                            <th className="px-6 py-4 font-medium">Last Modified</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-800 text-sm">
                        <tr>
                            <td className="px-6 py-4 font-medium">MA Crossover v2</td>
                            <td className="px-6 py-4 text-gray-400">Rule-based</td>
                            <td className="px-6 py-4"><span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-emerald-500/10 text-emerald-500">Active</span></td>
                            <td className="px-6 py-4 text-gray-400">2 mins ago</td>
                        </tr>
                        <tr>
                            <td className="px-6 py-4 font-medium">ONNX Mean Reversion</td>
                            <td className="px-6 py-4 text-gray-400">ML (ORT)</td>
                            <td className="px-6 py-4"><span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-500/10 text-gray-500">Inactive</span></td>
                            <td className="px-6 py-4 text-gray-400">1 day ago</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    );
};

export default StrategyConfig;
