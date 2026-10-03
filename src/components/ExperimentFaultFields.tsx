import type { DownFault, Experiment, ExperimentNode, RelayProtocol } from "../lib/api";
import { useT, type TKey } from "../lib/i18n";
import { TemplateField } from "./TemplateField";
import { ImpairProfileFields } from "./ImpairProfileFields";

const DOWN_FAULTS: DownFault[] = ["unavailable", "reset", "timeout"];

type FaultNode = Extract<ExperimentNode, { type: "impairment" | "impairment_change" | "emulator_state" }>;

/** A relay's protocol; a Change impairment of no relay reads its profile as a UDP one. */
const relayProtocol = (relay: Extract<ExperimentNode, { type: "impairment" }> | undefined): RelayProtocol => relay?.protocol ?? "udp";

/**
 * The fields of the nodes that break things on cue: an Impairment's relay and
 * profile, the profile a Change impairment switches to, and which emulator an
 * Emulator down/up takes down. The last two pick a node of `doc`.
 */
export function ExperimentFaultFields({ node, doc, patch }: { node: FaultNode; doc: Experiment; patch: (change: Partial<ExperimentNode>) => void }) {
  const t = useT();
  const relays = doc.nodes.filter((candidate): candidate is Extract<ExperimentNode, { type: "impairment" }> => candidate.type === "impairment");
  const emulators = doc.nodes.filter((candidate): candidate is Extract<ExperimentNode, { type: "emulator" }> => candidate.type === "emulator");
  switch (node.type) {
    case "impairment": return <>
      <label data-tip={t("exp.relayListenHint")}>{t("exp.relayListen")}<TemplateField primary value={node.listen} placeholder="127.0.0.1:9010" onChange={listen => patch({ listen })} /></label>
      <label data-tip={t("exp.relayTargetHint")}>{t("exp.relayTarget")}<TemplateField value={node.target} placeholder="127.0.0.1:9000" onChange={target => patch({ target })} /></label>
      <label data-tip={t("ns.protocolHint")}>{t("ns.protocol")}<select value={node.protocol ?? "udp"} onChange={event => patch({ protocol: event.target.value === "tcp" ? "tcp" : undefined })}>
        <option value="udp">UDP</option>
        <option value="tcp">TCP</option>
      </select></label>
      <ImpairProfileFields profile={node.profile} onChange={profile => patch({ profile })} protocol={node.protocol ?? "udp"} />
    </>;
    case "impairment_change": return <>
      {relays.length === 0 ? <p className="experiment-empty">{t("exp.noRelays")}</p>
        : <label data-tip={t("exp.relayHint")}>{t("exp.relay")}<select data-primary value={node.relay} onChange={event => patch({ relay: event.target.value })}>
          {!relays.some(relay => relay.id === node.relay) && <option value={node.relay}>—</option>}
          {relays.map(relay => <option key={relay.id} value={relay.id}>{relay.protocol === "tcp" ? "TCP " : ""}{relay.listen} → {relay.target}</option>)}
        </select></label>}
      <ImpairProfileFields profile={node.profile} onChange={profile => patch({ profile })} protocol={relayProtocol(relays.find(relay => relay.id === node.relay))} />
    </>;
    case "emulator_state": {
      const chosen = emulators.find(emulator => emulator.id === node.emulator);
      return <>
        {emulators.length === 0 ? <p className="experiment-empty">{t("exp.noEmulators")}</p>
          : <label data-tip={t("exp.emulatorNodeHint")}>{t("exp.emulatorNode")}<select data-primary value={node.emulator} onChange={event => patch({ emulator: event.target.value })}>
            {!chosen && <option value={node.emulator}>—</option>}
            {emulators.map(emulator => <option key={emulator.id} value={emulator.id}>{emulator.emulator.name} · {emulator.emulator.bind}</option>)}
          </select></label>}
        <label>{t("exp.emulatorDownState")}<select value={node.down ? "down" : "up"} onChange={event => patch({ down: event.target.value === "down" })}>
          <option value="down">{t("exp.emulatorGoesDown")}</option>
          <option value="up">{t("exp.emulatorComesUp")}</option>
        </select></label>
        {node.down && (!chosen || chosen.emulator.protocol === "http") && <label data-tip={t("exp.downFaultHint")}>{t("exp.downFault")}
          <select value={node.fault} onChange={event => patch({ fault: event.target.value as DownFault })}>
            {DOWN_FAULTS.map(fault => <option key={fault} value={fault}>{t(`emu.outageFault.${fault}` as TKey)}</option>)}
          </select></label>}
      </>;
    }
  }
}
