import React from 'react';

const BacktestRunner: React.FC = () => {
    return (
        <div className="space-y-6">
            <h2 className="text-2xl font-bold tracking-tight">Backtest Runner</h2>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div className="lg:col-span-1 border border-gray-800 bg-gray-900 rounded-xl p-6">
                    <h3 className="text-lg font-medium mb-4">Configuration</h3>
                    <form className="space-y-4">
                        <div>
                            <label className="block text-sm font-medium text-gray-400 mb-1">Strategy</label>
                            <select className="w-full bg-gray-950 border border-gray-800 rounded-md py-2 px-3 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500">
                                <option>MA Crossover v2</option>
                                <option>ONNX Mean Reversion</option>
                            </select>
                        </div>
                        <div>
                            <label className="block text-sm font-medium text-gray-400 mb-1">Data Source</label>
                            <select className="w-full bg-gray-950 border border-gray-800 rounded-md py-2 px-3 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500">
                                <option>NQ 1 Min (2023-2024)</option>
                                <option>ES Tick (Jan 2024)</option>
                            </select>
                        </div>
                        <button type="button" className="w-full bg-blue-600 hover:bg-blue-700 text-white py-2 rounded-md text-sm font-medium mt-4">
                            Run Backtest
                        </button>
                    </form>
                </div>

                <div className="lg:col-span-2 border border-gray-800 bg-gray-900 rounded-xl p-6 flex flex-col items-center justify-center min-h-[400px]">
                    <p className="text-gray-500">Run a backtest to view results.</p>
                </div>
            </div>
        </div>
    );
};

export default BacktestRunner;
