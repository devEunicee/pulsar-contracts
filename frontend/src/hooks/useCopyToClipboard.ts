import { useState, useCallback } from "react";

type CopyStatus = "idle" | "copied" | "error";

export function useCopyToClipboard(resetDelay = 2000) {
  const [status, setStatus] = useState<CopyStatus>("idle");

  const copy = useCallback(
    async (text: string) => {
      try {
        if (navigator.clipboard?.writeText) {
          await navigator.clipboard.writeText(text);
        } else {
          // Fallback for browsers without Clipboard API
          const textarea = document.createElement("textarea");
          textarea.value = text;
          textarea.style.cssText =
            "position:fixed;top:-9999px;left:-9999px;opacity:0";
          document.body.appendChild(textarea);
          textarea.focus();
          textarea.select();
          const ok = document.execCommand("copy");
          document.body.removeChild(textarea);
          if (!ok) throw new Error("execCommand copy failed");
        }
        setStatus("copied");
      } catch {
        setStatus("error");
      } finally {
        setTimeout(() => setStatus("idle"), resetDelay);
      }
    },
    [resetDelay],
  );

  return { copy, status };
}
