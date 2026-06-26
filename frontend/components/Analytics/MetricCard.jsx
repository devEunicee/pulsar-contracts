import './Analytics.css';

export function MetricCard({ label, value, trend, trendValue }) {
  const up = trend === 'up';
  const down = trend === 'down';
  return (
    <div className="mc-card">
      <span className="mc-label">{label}</span>
      <span className="mc-value">{value}</span>
      {trend && (
        <span className={`mc-trend mc-trend--${trend}`} aria-label={`${trend} ${trendValue}`}>
          {up ? '▲' : '▼'} {trendValue}
        </span>
      )}
    </div>
  );
}
