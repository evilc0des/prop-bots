import React from 'react';

const LiveMonitor: React.FC = () => {
    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center">
                <h2 className="text-2xl font-bold tracking-tight">Live Monitor</h2>
                <div className="flex gap-2">
                    <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-500 border border-emerald-500/20">
                        <span className="w-2 h-2 rounded-full bg-emerald-500 mr-2"></span> NT8 Connected
                    </span>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
                <div className="lg:col-span-3 border border-gray-800 bg-gray-900 rounded-xl p-6 min-h-[400px]">
                    <h3 className="text-sm font-medium text-gray-400 mb-4">Live Chart (NQ)</h3>
                    <div className="flex items-center justify-center h-[300px] border border-gray-800 border-dashed rounded-lg bg-gray-950">
                        <span className="text-gray-600 text-sm">Chart Placeholder</span>
                    </div>
                </div>

                <div className="border border-gray-800 bg-gray-900 rounded-xl p-6 flex flex-col">
                    <h3 className="text-sm font-medium text-gray-400 mb-4">Open Positions</h3>
                    <div className="flex-1">
                        <div className="bg-gray-950 border border-gray-800 rounded-lg p-4 mb-3">
                            <div className="flex justify-between items-start mb-2">
                                <span className="font-bold">NQ M4</span>
                                <span className="text-red-500 font-bold">-2</span>
                            </div>
                            <div className="flex justify-between text-sm text-gray-400">
                                <span>Entry: 18,240.50</span>
                                <span className="text-emerald-500">+$120.00</span>
                            </div>
                        </div>
                    </div>
                    <button className="w-full bg-orange-600/20 text-orange-500 hover:bg-orange-600/30 font-medium py-2 rounded-md border border-orange-500/30 transition-colors">
                        Flatten All (Emergency)
                    </button>
                </div>
            </div>
        </div>
    );
};

export default LiveMonitor;
