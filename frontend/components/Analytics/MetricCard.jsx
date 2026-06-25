import React from 'react';

export default function MetricCard({ label, value, trend, trendValue }) {
  const isUp = trend === 'up';
  return (
    <div className="metric-card">
      <span className="metric-label">{label}</span>
      <span className="metric-value">{value}</span>
      {trendValue != null && (
        <span className={`metric-trend metric-trend--${trend}`}>
          {isUp ? '▲' : '▼'} {trendValue}%
        </span>
      )}
    </div>
  );
}
