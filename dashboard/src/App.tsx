import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { DashboardLayout } from './components/layout/DashboardLayout';
import Overview from './pages/Overview';
import StrategyConfig from './pages/StrategyConfig';
import BacktestRunner from './pages/BacktestRunner';
import LiveMonitor from './pages/LiveMonitor';
import RulesEditor from './pages/RulesEditor';

function App() {
  return (
    <BrowserRouter>
      <DashboardLayout>
        <Routes>
          <Route path="/" element={<Overview />} />
          <Route path="/strategy" element={<StrategyConfig />} />
          <Route path="/backtest" element={<BacktestRunner />} />
          <Route path="/live" element={<LiveMonitor />} />
          <Route path="/rules" element={<RulesEditor />} />
        </Routes>
      </DashboardLayout>
    </BrowserRouter>
  );
}

export default App;
