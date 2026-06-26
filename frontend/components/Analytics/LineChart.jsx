export function LineChart({ data = [], title, color = '#3b82f6' }) {
  if (!data.length) return <div className="chart-empty">No data</div>;

  const W = 480, H = 220, PAD = { top: 20, right: 20, bottom: 40, left: 50 };
  const innerW = W - PAD.left - PAD.right;
  const innerH = H - PAD.top - PAD.bottom;

  const maxVal = Math.max(...data.map(d => d.value), 1);
  const xStep = innerW / (data.length - 1 || 1);

  const pts = data.map((d, i) => ({
    x: PAD.left + i * xStep,
    y: PAD.top + innerH - (d.value / maxVal) * innerH,
    label: d.label,
    value: d.value,
  }));

  const polyline = pts.map(p => `${p.x},${p.y}`).join(' ');
  const area = `M${pts[0].x},${PAD.top + innerH} ${pts.map(p => `L${p.x},${p.y}`).join(' ')} L${pts[pts.length - 1].x},${PAD.top + innerH} Z`;

  return (
    <figure className="chart-wrap" aria-label={title}>
      {title && <figcaption className="chart-title">{title}</figcaption>}
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={title} className="chart-svg">
        {/* Y-axis */}
        <line x1={PAD.left} y1={PAD.top} x2={PAD.left} y2={PAD.top + innerH} stroke="#e5e7eb" />
        {[0, 0.25, 0.5, 0.75, 1].map(t => {
          const y = PAD.top + innerH - t * innerH;
          return (
            <g key={t}>
              <line x1={PAD.left - 4} y1={y} x2={PAD.left + innerW} y2={y} stroke="#f3f4f6" strokeDasharray="3 3" />
              <text x={PAD.left - 8} y={y + 4} textAnchor="end" fontSize="10" fill="#9ca3af">
                {Math.round(t * maxVal)}
              </text>
            </g>
          );
        })}
        {/* X-axis */}
        <line x1={PAD.left} y1={PAD.top + innerH} x2={PAD.left + innerW} y2={PAD.top + innerH} stroke="#e5e7eb" />
        {pts.map((p, i) => (
          <text key={i} x={p.x} y={PAD.top + innerH + 16} textAnchor="middle" fontSize="10" fill="#9ca3af">
            {p.label}
          </text>
        ))}
        {/* Area fill */}
        <path d={area} fill={color} opacity="0.1" />
        {/* Line */}
        <polyline points={polyline} fill="none" stroke={color} strokeWidth="2" strokeLinejoin="round" />
        {/* Dots */}
        {pts.map((p, i) => (
          <circle key={i} cx={p.x} cy={p.y} r="4" fill={color} stroke="#fff" strokeWidth="2">
            <title>{p.label}: {p.value}</title>
          </circle>
        ))}
      </svg>
    </figure>
  );
}
