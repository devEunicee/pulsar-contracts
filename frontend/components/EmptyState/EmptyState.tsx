import React from 'react';

export interface EmptyStateProps {
  title: string;
  message: string;
  ctaLabel?: string;
  onCta?: () => void;
  className?: string;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  title,
  message,
  ctaLabel,
  onCta,
  className,
}) => {
  return (
    <div className={className || 'empty-state'} role="region" aria-label={title}>
      <div className="empty-state__illustration" aria-hidden="true">📭</div>
      <h2 className="empty-state__title">{title}</h2>
      <p className="empty-state__message">{message}</p>
      {ctaLabel && (
        <button className="empty-state__cta" onClick={onCta} aria-label={ctaLabel}>
          {ctaLabel}
        </button>
      )}
    </div>
  );
};

export default EmptyState;
