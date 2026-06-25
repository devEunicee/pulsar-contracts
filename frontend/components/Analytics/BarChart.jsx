import React from 'react';

const W = 500, H = 260, PAD = 50;

export default function BarChart({ data = [], title, color = '#6c63ff' }) {
  if (!data.length) return <div className="chart-empty">No data</div>;

  const max = Math.max(...data.map(d => d.value)) || 1;
  const barW = (W - PAD * 2) / data.length * 0.6;
  const gap = (W - PAD * 2) / data.length;
  const scaleY = v => (H - PAD * 2) * (v / max);

  return (
    <div className="chart-container">
      {title && <h3 className="chart-title">{title}</h3>}
      <svg viewBox={`0 0 ${W} ${H}`} className="chart-svg">
        {/* Axes */}
        <line x1={PAD} y1={PAD} x2={PAD} y2={H - PAD} stroke="#ccc" strokeWidth="1" />
        <line x1={PAD} y1={H - PAD} x2={W - PAD} y2={H - PAD} stroke="#ccc" strokeWidth="1" />

        {/* Y labels */}
        {[0, 0.5, 1].map(t => {
          const v = max * t;
          const y = H - PAD - scaleY(v);
          return (
            <g key={t}>
              <line x1={PAD - 4} y1={y} x2={PAD} y2={y} stroke="#ccc" />
              <text x={PAD - 6} y={y + 4} textAnchor="end" fontSize="10" fill="#888">
                {Math.round(v)}
              </text>
            </g>
          );
        })}

        {/* Bars */}
        {data.map((d, i) => {
          const bh = scaleY(d.value);
          const x = PAD + i * gap + (gap - barW) / 2;
          const y = H - PAD - bh;
          return (
            <g key={i}>
              <rect x={x} y={y} width={barW} height={bh} fill={color} rx="2" />
              <text x={x + barW / 2} y={H - PAD + 16} textAnchor="middle" fontSize="10" fill="#888">
                {d.label}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}
