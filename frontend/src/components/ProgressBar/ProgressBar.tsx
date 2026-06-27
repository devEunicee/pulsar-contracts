import React from "react";
import "./ProgressBar.css";

interface ProgressBarProps {
  /** 0–100. When null the bar is hidden. */
  progress: number | null;
}

/**
 * Slim page-level progress bar fixed to the top of the viewport.
 * Renders null when progress is null (hidden).
 */
export function ProgressBar({ progress }: ProgressBarProps) {
  if (progress === null) return null;
  return (
    <div
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={progress}
      aria-label="Page loading"
      className="progress-bar"
    >
      <div className="progress-bar__fill" style={{ width: `${progress}%` }} />
    </div>
  );
}
