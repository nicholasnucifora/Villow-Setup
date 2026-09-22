import { useEffect, useState } from "react";

export function CopyAddress({
  label,
  value,
  buttonLabel,
  disabled,
}: {
  label: string;
  value: string;
  buttonLabel: string;
  disabled: boolean;
}) {
  const [status, setStatus] = useState<
    "idle" | "copying" | "copied" | "failed"
  >("idle");
  useEffect(() => {
    if (status !== "copied") return;
    const timer = window.setTimeout(() => setStatus("idle"), 3000);
    return () => window.clearTimeout(timer);
  }, [status]);
  const copy = async () => {
    setStatus("copying");
    try {
      await navigator.clipboard.writeText(value);
      setStatus("copied");
    } catch {
      setStatus("failed");
    }
  };
  return (
    <div className="copy-row" role="group" aria-label={label}>
      <div>
        <small>{label}</small>
        <code>{value}</code>
      </div>
      <div className="copy-control">
        <button
          type="button"
          className="secondary"
          disabled={disabled || !value || status === "copying"}
          onClick={copy}
        >
          {status === "copied"
            ? "✓ Copied!"
            : status === "copying"
              ? "Copying…"
              : buttonLabel}
        </button>
        <small role="status">
          {status === "copied" && `${label} copied.`}
          {status === "failed" &&
            "Couldn’t copy. Select the address and press Ctrl+C."}
        </small>
      </div>
    </div>
  );
}
