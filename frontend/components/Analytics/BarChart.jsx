export function BarChart({ data = [], title, color = '#8b5cf6' }) {
  if (!data.length) return <div className="chart-empty">No data</div>;

  const W = 480, H = 220, PAD = { top: 20, right: 20, bottom: 40, left: 50 };
  const innerW = W - PAD.left - PAD.right;
  const innerH = H - PAD.top - PAD.bottom;

  const maxVal = Math.max(...data.map(d => d.value), 1);
  const barW = (innerW / data.length) * 0.6;
  const gap = innerW / data.length;

  return (
    <figure className="chart-wrap" aria-label={title}>
      {title && <figcaption className="chart-title">{title}</figcaption>}
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={title} className="chart-svg">
        {/* Y-axis gridlines */}
        <line x1={PAD.left} y1={PAD.top} x2={PAD.left} y2={PAD.top + innerH} stroke="#e5e7eb" />
        {[0, 0.25, 0.5, 0.75, 1].map(t => {
          const y = PAD.top + innerH - t * innerH;
          return (
            <g key={t}>
              <line x1={PAD.left} y1={y} x2={PAD.left + innerW} y2={y} stroke="#f3f4f6" strokeDasharray="3 3" />
              <text x={PAD.left - 8} y={y + 4} textAnchor="end" fontSize="10" fill="#9ca3af">
                {Math.round(t * maxVal)}
              </text>
            </g>
          );
        })}
        {/* X-axis */}
        <line x1={PAD.left} y1={PAD.top + innerH} x2={PAD.left + innerW} y2={PAD.top + innerH} stroke="#e5e7eb" />
        {data.map((d, i) => {
          const barH = (d.value / maxVal) * innerH;
          const x = PAD.left + i * gap + (gap - barW) / 2;
          const y = PAD.top + innerH - barH;
          return (
            <g key={i}>
              <rect x={x} y={y} width={barW} height={barH} fill={color} rx="3">
                <title>{d.label}: {d.value}</title>
              </rect>
              <text x={x + barW / 2} y={PAD.top + innerH + 16} textAnchor="middle" fontSize="10" fill="#9ca3af">
                {d.label.length > 8 ? d.label.slice(0, 7) + '…' : d.label}
              </text>
            </g>
          );
        })}
      </svg>
    </figure>
  );
}
