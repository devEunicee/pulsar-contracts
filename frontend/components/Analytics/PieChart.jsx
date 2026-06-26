export function PieChart({ data = [], title }) {
  if (!data.length) return <div className="chart-empty">No data</div>;

  const CX = 100, CY = 100, R = 70, IR = 42; // donut inner radius
  const total = data.reduce((s, d) => s + d.value, 0) || 1;

  let angle = -Math.PI / 2; // start at top
  const slices = data.map(d => {
    const sweep = (d.value / total) * 2 * Math.PI;
    const start = angle;
    angle += sweep;
    return { ...d, start, sweep };
  });

  function arc(start, sweep) {
    const x1 = CX + R * Math.cos(start);
    const y1 = CY + R * Math.sin(start);
    const x2 = CX + R * Math.cos(start + sweep);
    const y2 = CY + R * Math.sin(start + sweep);
    const xi1 = CX + IR * Math.cos(start + sweep);
    const yi1 = CY + IR * Math.sin(start + sweep);
    const xi2 = CX + IR * Math.cos(start);
    const yi2 = CY + IR * Math.sin(start);
    const large = sweep > Math.PI ? 1 : 0;
    return `M${x1} ${y1} A${R} ${R} 0 ${large} 1 ${x2} ${y2} L${xi1} ${yi1} A${IR} ${IR} 0 ${large} 0 ${xi2} ${yi2} Z`;
  }

  return (
    <figure className="chart-wrap" aria-label={title}>
      {title && <figcaption className="chart-title">{title}</figcaption>}
      <div className="pie-layout">
        <svg viewBox="0 0 200 200" role="img" aria-label={title} className="chart-svg pie-svg">
          {slices.map((s, i) => (
            <path key={i} d={arc(s.start, s.sweep)} fill={s.color} stroke="#fff" strokeWidth="1.5">
              <title>{s.label}: {s.value} ({((s.value / total) * 100).toFixed(1)}%)</title>
            </path>
          ))}
        </svg>
        <ul className="pie-legend" aria-label="Legend">
          {slices.map((s, i) => (
            <li key={i} className="pie-legend__item">
              <span className="pie-legend__swatch" style={{ background: s.color }} aria-hidden="true" />
              <span>{s.label}</span>
              <span className="pie-legend__pct">{((s.value / total) * 100).toFixed(1)}%</span>
            </li>
          ))}
        </ul>
      </div>
    </figure>
  );
}
