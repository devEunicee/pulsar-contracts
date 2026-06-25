import React from 'react';

const R = 80, r = 45, CX = 120, CY = 110;

function arc(cx, cy, R, startAngle, endAngle) {
  const a1 = (startAngle - 90) * (Math.PI / 180);
  const a2 = (endAngle - 90) * (Math.PI / 180);
  const x1 = cx + R * Math.cos(a1), y1 = cy + R * Math.sin(a1);
  const x2 = cx + R * Math.cos(a2), y2 = cy + R * Math.sin(a2);
  const large = endAngle - startAngle > 180 ? 1 : 0;
  return `M ${cx} ${cy} L ${x1} ${y1} A ${R} ${R} 0 ${large} 1 ${x2} ${y2} Z`;
}

export default function PieChart({ data = [], title }) {
  if (!data.length) return <div className="chart-empty">No data</div>;

  const total = data.reduce((s, d) => s + d.value, 0) || 1;
  let angle = 0;

  return (
    <div className="chart-container">
      {title && <h3 className="chart-title">{title}</h3>}
      <svg viewBox="0 0 280 220" className="chart-svg">
        {data.map((d, i) => {
          const slice = (d.value / total) * 360;
          const path = arc(CX, CY, R, angle, angle + slice);
          const start = angle;
          angle += slice;
          return <path key={i} d={path} fill={d.color} stroke="#fff" strokeWidth="2" />;
        })}
        {/* Donut hole */}
        <circle cx={CX} cy={CY} r={r} fill="#fff" />

        {/* Legend */}
        {data.map((d, i) => (
          <g key={i} transform={`translate(210, ${20 + i * 22})`}>
            <rect width="12" height="12" fill={d.color} rx="2" />
            <text x="16" y="10" fontSize="11" fill="#555">{d.label}</text>
          </g>
        ))}
      </svg>
    </div>
  );
}
