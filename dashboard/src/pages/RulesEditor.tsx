import React from 'react';

const RulesEditor: React.FC = () => {
    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center">
                <h2 className="text-2xl font-bold tracking-tight">Prop Firm Rules</h2>
                <button className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors">
                    Add Profile
                </button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="bg-gray-900 border border-gray-800 rounded-xl p-6">
                    <div className="flex justify-between items-center mb-4">
                        <h3 className="text-lg font-bold">TopStep 50k</h3>
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-emerald-500/10 text-emerald-500">Active</span>
                    </div>
                    <div className="space-y-3">
                        <div className="flex justify-between border-b border-gray-800 pb-2">
                            <span className="text-gray-400">Daily Loss Limit</span>
                            <span className="font-medium">-$1,000</span>
                        </div>
                        <div className="flex justify-between border-b border-gray-800 pb-2">
                            <span className="text-gray-400">Max Drawdown</span>
                            <span className="font-medium">-$2,000 (Trailing)</span>
                        </div>
                        <div className="flex justify-between border-b border-gray-800 pb-2">
                            <span className="text-gray-400">Max Contracts</span>
                            <span className="font-medium">5</span>
                        </div>
                        <div className="flex justify-between pt-1">
                            <span className="text-gray-400">Auto-Flatten at</span>
                            <span className="font-medium text-orange-500">-$900 (90%)</span>
                        </div>
                    </div>
                    <div className="mt-6 flex gap-2">
                        <button className="flex-1 bg-gray-800 hover:bg-gray-700 text-white px-3 py-2 rounded-md text-sm font-medium">Edit</button>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default RulesEditor;
