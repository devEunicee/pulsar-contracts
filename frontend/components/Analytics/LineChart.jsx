import React from 'react';

const W = 500, H = 260, PAD = 50;

export default function LineChart({ data = [], title, color = '#6c63ff' }) {
  if (!data.length) return <div className="chart-empty">No data</div>;

  const values = data.map(d => d.value);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;

  const xStep = (W - PAD * 2) / (data.length - 1 || 1);
  const scaleY = v => PAD + (H - PAD * 2) * (1 - (v - min) / range);
  const scaleX = i => PAD + i * xStep;

  const points = data.map((d, i) => `${scaleX(i)},${scaleY(d.value)}`).join(' ');

  return (
    <div className="chart-container">
      {title && <h3 className="chart-title">{title}</h3>}
      <svg viewBox={`0 0 ${W} ${H}`} className="chart-svg">
        {/* Axes */}
        <line x1={PAD} y1={PAD} x2={PAD} y2={H - PAD} stroke="#ccc" strokeWidth="1" />
        <line x1={PAD} y1={H - PAD} x2={W - PAD} y2={H - PAD} stroke="#ccc" strokeWidth="1" />

        {/* Y labels */}
        {[0, 0.5, 1].map(t => {
          const v = min + range * t;
          const y = scaleY(v);
          return (
            <g key={t}>
              <line x1={PAD - 4} y1={y} x2={PAD} y2={y} stroke="#ccc" />
              <text x={PAD - 6} y={y + 4} textAnchor="end" fontSize="10" fill="#888">
                {Math.round(v)}
              </text>
            </g>
          );
        })}

        {/* X labels */}
        {data.map((d, i) => (
          <text key={i} x={scaleX(i)} y={H - PAD + 16} textAnchor="middle" fontSize="10" fill="#888">
            {d.label}
          </text>
        ))}

        {/* Line */}
        <polyline points={points} fill="none" stroke={color} strokeWidth="2" />

        {/* Dots */}
        {data.map((d, i) => (
          <circle key={i} cx={scaleX(i)} cy={scaleY(d.value)} r="4" fill={color} />
        ))}
      </svg>
    </div>
  );
}
