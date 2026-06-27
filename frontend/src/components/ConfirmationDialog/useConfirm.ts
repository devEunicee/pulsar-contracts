import React from "react";

interface UseConfirmOptions {
  title: string;
  description: string;
  consequences?: string[];
  warning?: string;
  confirmLabel?: string;
  cancelLabel?: string;
}

interface UseConfirmReturn {
  isOpen: boolean;
  dialogProps: UseConfirmOptions & { open: boolean; onConfirm: () => void; onCancel: () => void };
  confirm: () => Promise<boolean>;
}

/**
 * Hook that imperatively opens a confirmation dialog.
 *
 * ```tsx
 * const { confirm, dialogProps } = useConfirm({
 *   title: "Deactivate merchant?",
 *   description: "This will prevent the merchant from processing payments.",
 * });
 *
 * const handleDeactivate = async () => {
 *   if (await confirm()) deactivateMerchant();
 * };
 *
 * return <ConfirmationDialog {...dialogProps} />;
 * ```
 */
export function useConfirm(options: UseConfirmOptions): UseConfirmReturn {
  const [isOpen, setIsOpen] = React.useState(false);
  const resolveRef = React.useRef<(value: boolean) => void>(null);

  const confirm = React.useCallback((): Promise<boolean> => {
    setIsOpen(true);
    return new Promise((resolve) => {
      (resolveRef as React.MutableRefObject<(v: boolean) => void>).current = resolve;
    });
  }, []);

  const handleConfirm = React.useCallback(() => {
    setIsOpen(false);
    resolveRef.current?.(true);
  }, []);

  const handleCancel = React.useCallback(() => {
    setIsOpen(false);
    resolveRef.current?.(false);
  }, []);

  return {
    isOpen,
    dialogProps: { ...options, open: isOpen, onConfirm: handleConfirm, onCancel: handleCancel },
    confirm,
  };
}
