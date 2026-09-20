import { useEffect, useState } from "react";
import { DemoBridge, type Failure } from "./demo";
import type { Bridge } from "./types";

// This entire module and the demo engine are excluded from ordinary builds.
export default function TestingTools({
  bridge,
  setBridge,
  nativeBridge,
  busy,
}: {
  bridge: Bridge;
  setBridge: (bridge: Bridge) => void;
  nativeBridge: Bridge;
  busy: boolean;
}) {
  const [enabled, setEnabled] = useState(bridge.demo);
  const [failure, setFailure] = useState<Failure>("none");
  useEffect(() => {
    if (bridge instanceof DemoBridge) setFailure(bridge.failure);
  }, [bridge, busy]);
  return (
    <section className="testing-tools" aria-label="Testing tools">
      <label className="checkbox">
        <input
          type="checkbox"
          checked={enabled}
          disabled={busy}
          onChange={(e) => {
            setEnabled(e.target.checked);
            if (!e.target.checked && bridge.demo) setBridge(nativeBridge);
          }}
        />
        Show testing tools
      </label>
      {enabled && (
        <div className="demo-banner">
          <div>
            <strong>
              {bridge.demo ? "Demo mode" : "Testing tools enabled"}
            </strong>
            <span>
              Demo accounts and operations are fictional. Never enter real
              credentials.
            </span>
            <button
              className="text-button"
              disabled={busy}
              onClick={() =>
                setBridge(bridge.demo ? nativeBridge : new DemoBridge())
              }
            >
              {bridge.demo ? "Leave demo" : "Explore demo"}
            </button>
          </div>
          {bridge.demo && (
            <label>
              Next operation
              <select
                aria-label="Demo failure"
                disabled={busy}
                value={failure}
                onChange={(e) => {
                  const next = e.target.value as Failure;
                  setFailure(next);
                  if (bridge instanceof DemoBridge) bridge.failure = next;
                }}
              >
                <option value="none">None</option>
                <option value="lost_response">Lost creation response</option>
                <option value="offline">Offline</option>
                <option value="expired">Expired token</option>
                <option value="rate_limit">Rate limit</option>
                <option value="health">Failed sign-in check</option>
              </select>
            </label>
          )}
        </div>
      )}
    </section>
  );
}
