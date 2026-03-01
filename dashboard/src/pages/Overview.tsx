import React from 'react';

const Overview: React.FC = () => {
    return (
        <div className="space-y-6">
            <h2 className="text-2xl font-bold tracking-tight">Overview</h2>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="bg-gray-900 border border-gray-800 rounded-xl p-6">
                    <h3 className="text-sm font-medium text-gray-400 mb-2">Total Equity</h3>
                    <p className="text-3xl font-bold">$150,240.50</p>
                    <p className="text-emerald-500 text-sm mt-2">+2.4% today</p>
                </div>
                <div className="bg-gray-900 border border-gray-800 rounded-xl p-6">
                    <h3 className="text-sm font-medium text-gray-400 mb-2">Active Bots</h3>
                    <p className="text-3xl font-bold">4</p>
                    <p className="text-gray-500 text-sm mt-2">2 running, 2 stopped</p>
                </div>
                <div className="bg-gray-900 border border-gray-800 rounded-xl p-6">
                    <h3 className="text-sm font-medium text-gray-400 mb-2">Prop Firm Status</h3>
                    <p className="text-3xl font-bold text-emerald-500">Passing</p>
                    <p className="text-gray-500 text-sm mt-2">No violations</p>
                </div>
            </div>
        </div>
    );
};

export default Overview;
