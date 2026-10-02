import type { ReactNode } from "react";
import type { HttpAuth } from "../lib/api";
import { useT } from "../lib/i18n";
import { useFieldIds } from "../lib/hooks";
import { TemplateField } from "./TemplateField";
import { AUTH_SCHEMES as SCHEMES, withScheme } from "../lib/httpAuth";


/**
 * How a request authenticates: none, Basic, Bearer or Digest, and the
 * credentials each needs. `templates`: fields take `{{…}}` (an experiment's
 * node — a password is then `{{secret.NAME}}`); otherwise plain fields (the
 * HTTP screen, where the password is typed and never stored).
 */
export function HttpAuthFields({ auth, onChange, templates = false, onKeyDown }: {
  auth: HttpAuth;
  onChange: (auth: HttpAuth) => void;
  templates?: boolean;
  onKeyDown?: (event: React.KeyboardEvent) => void;
}) {
  const t = useT();
  const fid = useFieldIds();
  const text = (id: string, label: string, value: string, set: (value: string) => void, secret = false): ReactNode => templates
    ? <label key={id}>{label}<TemplateField value={value} onChange={set} /></label>
    : <div className="field" key={id}>
      <label htmlFor={fid(id)}>{label}</label>
      <input id={fid(id)} type={secret ? "password" : "text"} autoComplete="off" spellCheck={false} value={value} onKeyDown={onKeyDown} onChange={(event) => set(event.target.value)} />
    </div>;
  const select = <select id={templates ? undefined : fid("scheme")} value={auth.scheme} onChange={(event) => onChange(withScheme(auth, event.target.value as HttpAuth["scheme"]))}>
    {SCHEMES.map((scheme) => <option key={scheme} value={scheme}>{t(`http.auth.${scheme}`)}</option>)}
  </select>;
  return <>
    {templates
      ? <label data-tip={t("http.authHint")}>{t("field.auth")}{select}</label>
      : <div className="field"><label htmlFor={fid("scheme")} data-tip={t("http.authHint")}>{t("field.auth")}</label>{select}</div>}
    {(auth.scheme === "basic" || auth.scheme === "digest") && <div className={templates ? undefined : "row"}>
      {text("username", t("field.username"), auth.username, (username) => onChange({ ...auth, username }))}
      {text("password", t("field.password"), auth.password, (password) => onChange({ ...auth, password }), true)}
    </div>}
    {auth.scheme === "bearer" && text("token", t("field.token"), auth.token, (token) => onChange({ ...auth, token }), true)}
  </>;
}
