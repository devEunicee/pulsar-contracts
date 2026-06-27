import React from 'react';
import EmptyState from './EmptyState';

export interface EmptyListProps {
  resourceName: string;
  onPrimaryAction?: () => void;
}

export const EmptyList: React.FC<EmptyListProps> = ({ resourceName, onPrimaryAction }) => {
  return (
    <EmptyState
      title={`No ${resourceName} yet`}
      message={`There are no ${resourceName} to show right now.`}
      ctaLabel={`Create ${resourceName}`}
      onCta={onPrimaryAction}
    />
  );
};

export default EmptyList;
