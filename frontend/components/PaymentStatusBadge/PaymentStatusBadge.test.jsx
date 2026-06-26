import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PaymentStatusBadge } from './PaymentStatusBadge';

describe('PaymentStatusBadge', () => {
  describe('loading state', () => {
    it('renders a loading badge when loading=true', () => {
      render(<PaymentStatusBadge loading />);
      expect(screen.getByRole('status')).toHaveAttribute(
        'aria-label',
        'Loading payment status'
      );
    });

    it('does not render a status label while loading', () => {
      render(<PaymentStatusBadge loading status="completed" />);
      expect(screen.queryByText('Completed')).toBeNull();
    });
  });

  describe('status rendering', () => {
    const cases = [
      { status: 'pending',   label: 'Pending',   icon: '⏳' },
      { status: 'completed', label: 'Completed', icon: '✓'  },
      { status: 'failed',    label: 'Failed',    icon: '✕'  },
      { status: 'refunded',  label: 'Refunded',  icon: '↩'  },
      { status: 'disputed',  label: 'Disputed',  icon: '⚠'  },
    ];

    cases.forEach(({ status, label, icon }) => {
      it(`renders ${status} badge with correct label and icon`, () => {
        render(<PaymentStatusBadge status={status} />);
        expect(screen.getByText(label)).toBeTruthy();
        expect(screen.getByText(icon)).toBeTruthy();
      });

      it(`${status} badge has correct aria-label`, () => {
        render(<PaymentStatusBadge status={status} />);
        expect(screen.getByRole('status')).toHaveAttribute(
          'aria-label',
          `Payment ${status}`
        );
      });

      it(`${status} badge has correct CSS class`, () => {
        render(<PaymentStatusBadge status={status} />);
        expect(screen.getByRole('status')).toHaveClass(`psb--${status}`);
      });
    });

    it('handles unknown status gracefully', () => {
      render(<PaymentStatusBadge status="unknown_status" />);
      expect(screen.getByRole('status')).toBeTruthy();
    });
  });

  describe('size prop', () => {
    ['sm', 'md', 'lg'].forEach((size) => {
      it(`applies psb--${size} class for size="${size}"`, () => {
        render(<PaymentStatusBadge status="completed" size={size} />);
        expect(screen.getByRole('status')).toHaveClass(`psb--${size}`);
      });
    });
  });

  describe('tooltip', () => {
    it('does not show tooltip by default', () => {
      render(<PaymentStatusBadge status="pending" />);
      expect(screen.queryByRole('tooltip')).toBeNull();
    });

    it('shows tooltip on hover when showTooltip=true', async () => {
      render(<PaymentStatusBadge status="pending" showTooltip />);
      await userEvent.hover(screen.getByRole('status'));
      expect(screen.getByRole('tooltip')).toBeTruthy();
      expect(screen.getByRole('tooltip').textContent).toBe(
        'Payment is awaiting processing.'
      );
    });

    it('hides tooltip after mouse leaves', async () => {
      render(<PaymentStatusBadge status="pending" showTooltip />);
      const badge = screen.getByRole('status');
      await userEvent.hover(badge);
      await userEvent.unhover(badge);
      expect(screen.queryByRole('tooltip')).toBeNull();
    });

    it('shows tooltip on focus for keyboard users', async () => {
      render(<PaymentStatusBadge status="completed" showTooltip />);
      await userEvent.tab();
      expect(screen.getByRole('tooltip')).toBeTruthy();
    });
  });

  describe('accessibility', () => {
    it('icon is hidden from assistive technology', () => {
      render(<PaymentStatusBadge status="completed" />);
      const icon = screen.getByText('✓');
      expect(icon).toHaveAttribute('aria-hidden', 'true');
    });

    it('badge element has role="status"', () => {
      render(<PaymentStatusBadge status="failed" />);
      expect(screen.getByRole('status')).toBeTruthy();
    });
  });
});
