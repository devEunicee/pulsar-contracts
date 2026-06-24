import { useState } from "react";
import { getErrorMessage } from "../lib/contractErrors";

export type CallState<T> =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "success"; data: T; txHash: string }
  | { status: "error"; message: string };

export function useContractCall<T>() {
  const [state, setState] = useState<CallState<T>>({ status: "idle" });

  async function execute(fn: () => Promise<{ result: T; txHash: string }>) {
    setState({ status: "loading" });
    try {
      const { result, txHash } = await fn();
      setState({ status: "success", data: result, txHash });
    } catch (err) {
      setState({ status: "error", message: getErrorMessage(err) });
    }
  }

  function reset() {
    setState({ status: "idle" });
  }

  return { state, execute, reset };
}
