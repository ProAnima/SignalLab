import empty from "../../experiments/templates/empty.json";
import http from "../../experiments/templates/http-check.json";
import branch from "../../experiments/templates/status-branch.json";
import parallel from "../../experiments/templates/parallel-flows.json";
import pingReply from "../../experiments/templates/osc-ping-reply.json";
import pollUntilReady from "../../experiments/templates/poll-until-ready.json";
import flakyApi from "../../experiments/templates/flaky-api.json";
import type { Experiment } from "./api";
import type { TKey } from "./i18n";

export const experimentTemplates: { id: string; title: TKey; description: TKey; document: Experiment }[] = [
  { id: "empty", title: "exp.templateEmpty", description: "exp.templateEmptyHint", document: empty as Experiment },
  { id: "http", title: "exp.templateHttp", description: "exp.templateHttpHint", document: http as Experiment },
  { id: "branch", title: "exp.templateBranch", description: "exp.templateBranchHint", document: branch as Experiment },
  { id: "parallel", title: "exp.templateParallel", description: "exp.templateParallelHint", document: parallel as Experiment },
  { id: "ping-reply", title: "exp.templatePingReply", description: "exp.templatePingReplyHint", document: pingReply as Experiment },
  { id: "poll", title: "exp.templatePoll", description: "exp.templatePollHint", document: pollUntilReady as Experiment },
  { id: "flaky-api", title: "exp.templateFlaky", description: "exp.templateFlakyHint", document: flakyApi as Experiment },
];
