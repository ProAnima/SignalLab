import { describeError, type Failure } from "../lib/errors";
import { useT } from "../lib/i18n";

/**
 * Where, what and why of a failure, with the system's own wording folded
 * underneath for diagnosis. `onShow` adds a button that brings the node the
 * failure is about into view.
 */
export function ErrorMessage({ error, nodeLabel, onShow, className = "" }: {
  error: Failure;
  nodeLabel?: (id: string) => string | null | undefined;
  onShow?: (nodeId: string) => void;
  className?: string;
}) {
  const t = useT();
  const described = describeError(error, t, nodeLabel);
  const nodeId = described.nodeId && nodeLabel?.(described.nodeId) ? described.nodeId : undefined;
  return <div className={`error-message ${className}`} role="alert">
    <p>{described.where && <b className="error-where">{described.where}</b>}<span>{described.message}</span>
      {onShow && nodeId && <button className="ghost sm error-show" onClick={() => onShow(nodeId)}>{t("err.show")}</button>}</p>
    {described.detail && <details className="error-detail"><summary>{t("err.details")}</summary><code>{described.detail}</code></details>}
  </div>;
}
