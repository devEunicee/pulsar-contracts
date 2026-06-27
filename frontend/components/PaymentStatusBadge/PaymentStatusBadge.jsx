import React, { useState } from 'react';
import './PaymentStatusBadge.css';

const STATUS_CONFIG = {
  pending: {
    label: 'Pending',
    icon: '⏳',
    ariaLabel: 'Payment pending',
    tooltip: 'Payment is awaiting processing.',
    cssVar: '--badge-pending',
  },
  completed: {
    label: 'Completed',
    icon: '✓',
    ariaLabel: 'Payment completed',
    tooltip: 'Payment was successfully processed.',
    cssVar: '--badge-completed',
  },
  failed: {
    label: 'Failed',
    icon: '✕',
    ariaLabel: 'Payment failed',
    tooltip: 'Payment could not be processed.',
    cssVar: '--badge-failed',
  },
  refunded: {
    label: 'Refunded',
    icon: '↩',
    ariaLabel: 'Payment refunded',
    tooltip: 'Payment has been refunded to the payer.',
    cssVar: '--badge-refunded',
  },
  disputed: {
    label: 'Disputed',
    icon: '⚠',
    ariaLabel: 'Payment disputed',
    tooltip: 'Payment is under dispute review.',
    cssVar: '--badge-disputed',
  },
};

export function PaymentStatusBadge({
  status,
  loading = false,
  showTooltip = false,
  size = 'md',
}) {
  const [tooltipVisible, setTooltipVisible] = useState(false);

  if (loading) {
    return (
      <span
        className={`psb psb--loading psb--${size}`}
        role="status"
        aria-label="Loading payment status"
      />
    );
  }

  const config = STATUS_CONFIG[status?.toLowerCase()] ?? {
    label: status ?? 'Unknown',
    icon: '?',
    ariaLabel: `Payment status: ${status ?? 'unknown'}`,
    tooltip: 'Status is not recognised.',
    cssVar: '--badge-unknown',
  };

  const badge = (
    <span
      className={`psb psb--${status?.toLowerCase() ?? 'unknown'} psb--${size}`}
      role="status"
      aria-label={config.ariaLabel}
      onMouseEnter={() => showTooltip && setTooltipVisible(true)}
      onMouseLeave={() => showTooltip && setTooltipVisible(false)}
      onFocus={() => showTooltip && setTooltipVisible(true)}
      onBlur={() => showTooltip && setTooltipVisible(false)}
      tabIndex={showTooltip ? 0 : undefined}
    >
      <span className="psb__icon" aria-hidden="true">{config.icon}</span>
      <span className="psb__label">{config.label}</span>
    </span>
  );

  if (!showTooltip) return badge;

  return (
    <span className="psb-wrapper">
      {badge}
      {tooltipVisible && (
        <span className="psb__tooltip" role="tooltip">
          {config.tooltip}
        </span>
      )}
    </span>
  );
}
