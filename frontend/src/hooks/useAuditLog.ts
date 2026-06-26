import { useState, useCallback } from "react";

export interface AuditEntry {
  id: string;
  action: string;
  actor: string;
  timestamp: number;
  detail?: string;
}

export function useAuditLog() {
  const [log, setLog] = useState<AuditEntry[]>([]);

  const record = useCallback((action: string, actor: string, detail?: string) => {
    setLog(prev => [
      { id: `${Date.now()}-${Math.random()}`, action, actor, timestamp: Date.now(), detail },
      ...prev,
    ]);
  }, []);

  return { log, record };
}
