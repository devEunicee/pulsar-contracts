import { useState, useMemo } from 'react';
import { MetricCard } from './MetricCard';
import { LineChart } from './LineChart';
import { BarChart } from './BarChart';
import { PieChart } from './PieChart';
import './Analytics.css';

const SAMPLE_PAYMENTS = [
  { date: '2024-01', merchant: 'Acme', amount: 1200, status: 'completed' },
  { date: '2024-02', merchant: 'Globex', amount: 3400, status: 'completed' },
  { date: '2024-03', merchant: 'Acme', amount: 900, status: 'refunded' },
  { date: '2024-04', merchant: 'Initech', amount: 2100, status: 'completed' },
  { date: '2024-05', merchant: 'Globex', amount: 4500, status: 'failed' },
  { date: '2024-06', merchant: 'Acme', amount: 3100, status: 'completed' },
  { date: '2024-07', merchant: 'Initech', amount: 1800, status: 'completed' },
  { date: '2024-08', merchant: 'Umbrella', amount: 2700, status: 'refunded' },
];

const STATUS_COLORS = {
  completed: '#22c55e',
  refunded: '#3b82f6',
  failed: '#ef4444',
  pending: '#f59e0b',
  disputed: '#f97316',
};

export function AnalyticsDashboard({ payments = SAMPLE_PAYMENTS, onExport }) {
  const [dateStart, setDateStart] = useState('');
  const [dateEnd, setDateEnd] = useState('');

  const filtered = useMemo(() => {
    return payments.filter(p => {
      if (dateStart && p.date < dateStart.slice(0, 7)) return false;
      if (dateEnd && p.date > dateEnd.slice(0, 7)) return false;
      return true;
    });
  }, [payments, dateStart, dateEnd]);

  // KPIs
  const totalVolume = filtered.reduce((s, p) => s + p.amount, 0);
  const avgTransaction = filtered.length ? Math.round(totalVolume / filtered.length) : 0;
  const totalPayments = filtered.length;
  const refundCount = filtered.filter(p => p.status === 'refunded').length;
  const refundRate = totalPayments ? ((refundCount / totalPayments) * 100).toFixed(1) + '%' : '0%';

  // Line chart: volume per month
  const volumeByMonth = filtered.reduce((acc, p) => {
    acc[p.date] = (acc[p.date] || 0) + p.amount;
    return acc;
  }, {});
  const lineData = Object.entries(volumeByMonth)
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([label, value]) => ({ label, value }));

  // Bar chart: volume by merchant
  const volumeByMerchant = filtered.reduce((acc, p) => {
    acc[p.merchant] = (acc[p.merchant] || 0) + p.amount;
    return acc;
  }, {});
  const barData = Object.entries(volumeByMerchant)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 8)
    .map(([label, value]) => ({ label, value }));

  // Pie chart: status distribution
  const byStatus = filtered.reduce((acc, p) => {
    acc[p.status] = (acc[p.status] || 0) + 1;
    return acc;
  }, {});
  const pieData = Object.entries(byStatus).map(([label, value]) => ({
    label, value, color: STATUS_COLORS[label] || '#9ca3af',
  }));

  function handleExport() {
    if (onExport) onExport(filtered);
    else window.print();
  }

  return (
    <div className="ad-container">
      {/* Filter bar */}
      <div className="ad-toolbar">
        <h1 className="ad-heading">Analytics Dashboard</h1>
        <div className="ad-filters">
          <label htmlFor="ad-from">From</label>
          <input id="ad-from" type="date" value={dateStart} onChange={e => setDateStart(e.target.value)} />
          <label htmlFor="ad-to">To</label>
          <input id="ad-to" type="date" value={dateEnd} onChange={e => setDateEnd(e.target.value)} />
          <button className="ad-export-btn" onClick={handleExport} aria-label="Export data">
            ↓ Export
          </button>
        </div>
      </div>

      {/* KPI cards */}
      <div className="ad-metrics">
        <MetricCard label="Total Volume" value={`$${totalVolume.toLocaleString()}`} trend="up" trendValue="12%" />
        <MetricCard label="Avg Transaction" value={`$${avgTransaction.toLocaleString()}`} />
        <MetricCard label="Total Payments" value={totalPayments} trend="up" trendValue="8%" />
        <MetricCard label="Refund Rate" value={refundRate} trend={refundCount > 1 ? 'down' : 'up'} trendValue={refundRate} />
      </div>

      {/* Charts grid */}
      <div className="ad-charts">
        <div className="ad-chart-full">
          <LineChart data={lineData} title="Payment Volume Over Time" color="#3b82f6" />
        </div>
        <div className="ad-chart-half">
          <BarChart data={barData} title="Top Merchants" color="#8b5cf6" />
        </div>
        <div className="ad-chart-half">
          <PieChart data={pieData} title="Payment Status Distribution" />
        </div>
      </div>
    </div>
  );
}
