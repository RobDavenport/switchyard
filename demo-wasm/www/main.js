const statusEl = document.getElementById("wasm-status");
const beatEl = document.getElementById("beat-line");
const clockEl = document.getElementById("clock-line");
const taskRowsEl = document.getElementById("task-rows");
const actionLogEl = document.getElementById("action-log");
const traceLogEl = document.getElementById("trace-log");
const scriptProgramsEl = document.getElementById("script-programs");
const scriptBoxEl = document.getElementById("script-box");
const snapshotBoxEl = document.getElementById("snapshot-box");
const cliExportBoxEl = document.getElementById("cli-export-box");
const cliCommandBoxEl = document.getElementById("cli-command-box");
const cliExportHintEl = document.getElementById("cli-export-hint");
const snapshotHintEl = document.getElementById("snapshot-hint");
const mindLineEl = document.getElementById("mind-line");
const mindButtonsEl = document.getElementById("mind-buttons");
const presetLineEl = document.getElementById("preset-line");
const presetCopyEl = document.getElementById("preset-copy");
const signalCopyEl = document.getElementById("signal-copy");
const predicateLabelEl = document.getElementById("predicate-label");
const tickBtn = document.getElementById("tick-btn");
const autoBtn = document.getElementById("auto-btn");
const resetBtn = document.getElementById("reset-btn");
const presetEncounterBtn = document.getElementById("preset-encounter-btn");
const presetShootemupBtn = document.getElementById("preset-shootemup-btn");
const presetMultimindBtn = document.getElementById("preset-multimind-btn");
const addProgramBtn = document.getElementById("add-program-btn");
const applyScriptBtn = document.getElementById("apply-script-btn");
const resetScriptBtn = document.getElementById("reset-script-btn");
const saveBtn = document.getElementById("save-btn");
const restoreBtn = document.getElementById("restore-btn");
const copyCatalogCliBtn = document.getElementById("copy-catalog-cli-btn");
const downloadCatalogCliBtn = document.getElementById("download-catalog-cli-btn");
const copySnapshotCliBtn = document.getElementById("copy-snapshot-cli-btn");
const downloadSnapshotCliBtn = document.getElementById("download-snapshot-cli-btn");
const predicateToggle = document.getElementById("predicate-toggle");
const signalButtons = [...document.querySelectorAll("[data-signal]")];

const PRESET_CONFIGS = {
  encounter: {
    label: "Death of Tick",
    copy: "Start with the article-style encounter and branch between boss reveal or collapse escape.",
    predicate: "Boss vulnerable predicate",
    signalCopy:
      "Signal labels adapt to the active preset so the same runtime surface can drive encounter beats or boss-phase interrupts.",
    minds: [
      { id: 1, label: "Encounter Mind" },
      { id: 2, label: "Collapse Mind" },
    ],
    signals: {
      1: "Player committed",
      2: "Scouts ready",
      3: "Boss spotted",
      4: "Collapse triggered",
    },
  },
  shootemup: {
    label: "Shootemup Boss",
    copy: "Switch to the boss-phase preset to watch projectile host calls, phase windows, and interrupts resolve in real time.",
    predicate: "Core exposed predicate",
    signalCopy:
      "The same four signal slots now drive a boss-phase timeline: wave start, add clear, core break, and bomb.",
    minds: [
      { id: 1, label: "Boss Logic" },
      { id: 2, label: "Pattern Cleanup" },
    ],
    signals: {
      1: "Wave started",
      2: "Wing drones cleared",
      3: "Core broken",
      4: "Bomb triggered",
    },
  },
  multimind: {
    label: "Mind the Gap",
    copy: "A director script parks on the gameplay mind, waits for a host handoff, then returns control to the director lane for rollout.",
    predicate: "Predicate toggle unused in this preset",
    signalCopy:
      "This preset is about parked work. Fire the director cue, watch the task park on the gameplay mind, then switch the active mind to resume it deterministically.",
    minds: [
      { id: 1, label: "Director Mind" },
      { id: 2, label: "Gameplay Mind" },
    ],
    signals: {
      1: "Director cue",
      2: "Gameplay beat locked",
      3: "Cleanup cue",
      4: "Final blackout",
    },
  },
};

const OP_KINDS = [
  { kind: "action", label: "Action", fields: [{ key: "action", label: "Action", min: 1 }] },
  {
    kind: "call",
    label: "Host Call",
    fields: [
      { key: "call", label: "Call", min: 1 },
      { key: "arg0", label: "Arg 0", min: -999999 },
      { key: "arg1", label: "Arg 1", min: -999999 },
      { key: "arg2", label: "Arg 2", min: -999999 },
      { key: "arg3", label: "Arg 3", min: -999999 },
    ],
  },
  { kind: "wait_ticks", label: "Wait Ticks", fields: [{ key: "ticks", label: "Ticks", min: 0 }] },
  {
    kind: "wait_until_tick",
    label: "Wait Until Tick",
    fields: [{ key: "until_tick", label: "Clock", min: 0 }],
  },
  { kind: "wait_signal", label: "Wait Signal", fields: [{ key: "signal", label: "Signal", min: 1 }] },
  {
    kind: "wait_signal_or_ticks",
    label: "Wait Signal Or Timeout",
    fields: [
      { key: "signal", label: "Signal", min: 1 },
      { key: "ticks", label: "Ticks", min: 0 },
    ],
  },
  {
    kind: "wait_signal_until_tick",
    label: "Wait Signal Until Tick",
    fields: [
      { key: "signal", label: "Signal", min: 1 },
      { key: "until_tick", label: "Clock", min: 0 },
    ],
  },
  {
    kind: "race_children_or_ticks",
    label: "Race Or Timeout",
    fields: [
      { key: "left", label: "Left", min: 1 },
      { key: "right", label: "Right", min: 1 },
      { key: "ticks", label: "Ticks", min: 0 },
    ],
  },
  {
    kind: "race_children_until_tick",
    label: "Race Until Tick",
    fields: [
      { key: "left", label: "Left", min: 1 },
      { key: "right", label: "Right", min: 1 },
      { key: "until_tick", label: "Clock", min: 0 },
    ],
  },
  {
    kind: "timeout_ticks",
    label: "Timeout Child",
    fields: [
      { key: "ticks", label: "Ticks", min: 0 },
      { key: "program", label: "Program", min: 1 },
    ],
  },
  {
    kind: "timeout_until_tick",
    label: "Timeout Child Until Tick",
    fields: [
      { key: "until_tick", label: "Clock", min: 0 },
      { key: "program", label: "Program", min: 1 },
    ],
  },
  {
    kind: "wait_predicate",
    label: "Wait Predicate",
    fields: [{ key: "predicate", label: "Predicate", min: 1 }],
  },
  {
    kind: "change_mind",
    label: "Change Mind",
    fields: [{ key: "mind", label: "Mind", min: 1 }],
  },
  {
    kind: "branch_predicate",
    label: "Branch Predicate",
    fields: [
      { key: "predicate", label: "Predicate", min: 1 },
      { key: "if_true", label: "If True", min: 1 },
      { key: "if_false", label: "If False", min: 1 },
    ],
  },
  {
    kind: "repeat_count",
    label: "Repeat",
    fields: [
      { key: "program", label: "Program", min: 1 },
      { key: "count", label: "Times", min: 0 },
    ],
  },
  {
    kind: "repeat_until_predicate",
    label: "Repeat Until Predicate",
    fields: [
      { key: "program", label: "Program", min: 1 },
      { key: "predicate", label: "Predicate", min: 1 },
    ],
  },
  { kind: "spawn", label: "Spawn", fields: [{ key: "program", label: "Program", min: 1 }] },
  { kind: "join_children", label: "Join Children", fields: [] },
  { kind: "join_any_children", label: "Join Any", fields: [] },
  {
    kind: "sync_children",
    label: "Sync Children",
    fields: [
      { key: "left", label: "Left", min: 1 },
      { key: "right", label: "Right", min: 1 },
    ],
  },
  {
    kind: "race_children",
    label: "Race",
    fields: [
      { key: "left", label: "Left", min: 1 },
      { key: "right", label: "Right", min: 1 },
    ],
  },
  { kind: "succeed", label: "Succeed", fields: [] },
  { kind: "fail", label: "Fail", fields: [] },
];

let wasmApp = null;
let autoTimer = null;
let draftScript = { programs: [] };
let currentPreset = "encounter";
let currentActiveMind = 1;

const setStatus = (text, isError = false) => {
  statusEl.textContent = text;
  statusEl.style.color = isError ? "#ab3428" : "inherit";
};

const parseView = (json) => JSON.parse(json);

const cliCatalogCommands = `switchyard-cli catalog-check <catalog.json>
switchyard-cli catalog-summary <catalog.json>
switchyard-cli snapshot-check <catalog.json> <runtime-snapshot.json>`;

const cliSnapshotCommands = `switchyard-cli snapshot-check <catalog.json> <runtime-snapshot.json>
switchyard-cli snapshot-summary <runtime-snapshot.json>`;

const catalogExportMethod = () => {
  if (wasmApp && typeof wasmApp.export_cli_catalog_json === "function") {
    return "export_cli_catalog_json";
  }
  if (wasmApp && typeof wasmApp.script_json === "function") {
    return "script_json";
  }
  return null;
};

const snapshotExportMethod = () => {
  if (wasmApp && typeof wasmApp.export_cli_runtime_snapshot_json === "function") {
    return "export_cli_runtime_snapshot_json";
  }
  return null;
};

const refreshCliHandoffUi = () => {
  const hasCatalogExport = catalogExportMethod() !== null;
  const hasSnapshotExport = snapshotExportMethod() !== null;
  copyCatalogCliBtn.disabled = !hasCatalogExport;
  downloadCatalogCliBtn.disabled = !hasCatalogExport;
  copySnapshotCliBtn.disabled = !hasSnapshotExport;
  downloadSnapshotCliBtn.disabled = !hasSnapshotExport;
  cliCommandBoxEl.textContent = cliCatalogCommands;
  cliExportHintEl.textContent = hasSnapshotExport
    ? "Catalog and runtime snapshot export are ready for switchyard-cli, including snapshot-check against the paired catalog."
    : "Catalog export is ready for switchyard-cli today. Runtime snapshot export remains pending until the WASM build exposes export_cli_runtime_snapshot_json(), which also unlocks snapshot-check <catalog> <snapshot>.";
};

const writeClipboard = async (text) => {
  if (!navigator.clipboard || typeof navigator.clipboard.writeText !== "function") {
    throw new Error("Clipboard API unavailable in this browser.");
  }
  await navigator.clipboard.writeText(text);
};

const downloadJson = (filename, text) => {
  const blob = new Blob([text], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.append(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
};

const previewCliExport = (kind, text) => {
  cliExportBoxEl.value = text;
  cliCommandBoxEl.textContent = kind === "snapshot" ? cliSnapshotCommands : cliCatalogCommands;
};

const loadCatalogCliExport = async () => {
  const method = catalogExportMethod();
  if (!method) {
    throw new Error("WASM build is missing script_json() / export_cli_catalog().");
  }
  return wasmApp[method]();
};

const loadSnapshotCliExport = async () => {
  const method = snapshotExportMethod();
  if (!method) {
    throw new Error(
      "WASM build is missing export_cli_runtime_snapshot_json(). Showcase save snapshots are not CLI-compatible yet, so snapshot-check <catalog> <snapshot> is unavailable from this panel.",
    );
  }
  return wasmApp[method]();
};

const mindLabelFor = (preset, mindId) => {
  const config = PRESET_CONFIGS[preset] ?? PRESET_CONFIGS.encounter;
  const match = config.minds.find((mind) => mind.id === mindId);
  return match ? `${match.label} (Mind ${match.id})` : `Mind ${mindId}`;
};

const renderMindButtons = (config) => {
  mindButtonsEl.innerHTML = "";
  for (const mind of config.minds) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = mind.label;
    button.classList.toggle("active", currentActiveMind === mind.id);
    button.addEventListener("click", () => run(() => wasmApp.set_active_mind(mind.id)));
    mindButtonsEl.append(button);
  }
};

const applyPresetUi = (preset) => {
  const config = PRESET_CONFIGS[preset] ?? PRESET_CONFIGS.encounter;
  currentPreset = preset in PRESET_CONFIGS ? preset : "encounter";
  presetLineEl.textContent = config.label;
  presetCopyEl.textContent = config.copy;
  signalCopyEl.textContent = config.signalCopy;
  predicateLabelEl.textContent = config.predicate;
  mindLineEl.textContent = mindLabelFor(currentPreset, currentActiveMind);
  presetEncounterBtn.classList.toggle("active", currentPreset === "encounter");
  presetShootemupBtn.classList.toggle("active", currentPreset === "shootemup");
  presetMultimindBtn.classList.toggle("active", currentPreset === "multimind");
  for (const button of signalButtons) {
    const signalId = Number(button.dataset.signal);
    button.textContent = config.signals[signalId] ?? `Signal ${signalId}`;
  }
  renderMindButtons(config);
};

const parseScript = (json) => {
  const parsed = JSON.parse(json);
  return {
    programs: Array.isArray(parsed.programs)
      ? parsed.programs.map((program) => ({
          id: Number(program.id ?? 1),
          ops: Array.isArray(program.ops)
            ? program.ops.map((op) => sanitizeOp(op))
            : [],
        }))
      : [],
  };
};

const sanitizeOp = (op) => {
  const normalized = defaultOpForKind(typeof op?.op === "string" ? op.op : "action");
  for (const key of Object.keys(normalized)) {
    if (key === "op") {
      continue;
    }
    normalized[key] = Number(op?.[key] ?? normalized[key]);
  }
  return normalized;
};

const defaultOpForKind = (kind) => {
  switch (kind) {
    case "action":
      return { op: "action", action: 1 };
    case "call":
      return { op: "call", call: 1, arg0: 0, arg1: 0, arg2: 0, arg3: 0 };
    case "wait_ticks":
      return { op: "wait_ticks", ticks: 1 };
    case "wait_until_tick":
      return { op: "wait_until_tick", until_tick: 1 };
    case "wait_signal":
      return { op: "wait_signal", signal: 1 };
    case "wait_signal_or_ticks":
      return { op: "wait_signal_or_ticks", signal: 1, ticks: 1 };
    case "wait_signal_until_tick":
      return { op: "wait_signal_until_tick", signal: 1, until_tick: 1 };
    case "race_children_or_ticks":
      return { op: "race_children_or_ticks", left: 1, right: 2, ticks: 2 };
    case "race_children_until_tick":
      return { op: "race_children_until_tick", left: 1, right: 2, until_tick: 2 };
    case "timeout_ticks":
      return { op: "timeout_ticks", ticks: 1, program: 1 };
    case "timeout_until_tick":
      return { op: "timeout_until_tick", until_tick: 1, program: 1 };
    case "wait_predicate":
      return { op: "wait_predicate", predicate: 1 };
    case "change_mind":
      return { op: "change_mind", mind: 2 };
    case "branch_predicate":
      return { op: "branch_predicate", predicate: 1, if_true: 1, if_false: 2 };
    case "repeat_count":
      return { op: "repeat_count", program: 1, count: 1 };
    case "repeat_until_predicate":
      return { op: "repeat_until_predicate", program: 1, predicate: 1 };
    case "spawn":
      return { op: "spawn", program: 1 };
    case "join_children":
      return { op: "join_children" };
    case "join_any_children":
      return { op: "join_any_children" };
    case "sync_children":
      return { op: "sync_children", left: 1, right: 2 };
    case "race_children":
    case "race2":
      return { op: "race_children", left: 1, right: 2 };
    case "succeed":
      return { op: "succeed" };
    case "fail":
      return { op: "fail" };
    default:
      return { op: "action", action: 1 };
  }
};

const opConfig = (kind) => OP_KINDS.find((entry) => entry.kind === kind) ?? OP_KINDS[0];

const updateDraftJsonBox = () => {
  scriptBoxEl.value = JSON.stringify(draftScript, null, 2);
};

const rerenderDraftScript = () => {
  scriptProgramsEl.innerHTML = "";

  if (!draftScript.programs.length) {
    const empty = document.createElement("p");
    empty.className = "small";
    empty.textContent = "No programs in the draft. Add one to author a scene.";
    scriptProgramsEl.append(empty);
    updateDraftJsonBox();
    return;
  }

  draftScript.programs.forEach((program, programIndex) => {
    const card = document.createElement("article");
    card.className = "program-card";

    const header = document.createElement("div");
    header.className = "program-header";
    header.innerHTML = `
      <div>
        <span class="label">Program ${programIndex + 1}</span>
        <strong class="program-title">Program ${program.id}</strong>
      </div>
    `;

    const headerActions = document.createElement("div");
    headerActions.className = "row-actions";
    headerActions.append(
      actionButton("Up", () => moveProgram(programIndex, -1), programIndex === 0),
      actionButton("Down", () => moveProgram(programIndex, 1), programIndex === draftScript.programs.length - 1),
      actionButton("Remove", () => {
        draftScript.programs.splice(programIndex, 1);
        rerenderDraftScript();
      }),
    );
    header.append(headerActions);
    card.append(header);

    const idRow = document.createElement("label");
    idRow.className = "field-row";
    idRow.innerHTML = `<span class="field-label">Program ID</span>`;
    const idInput = numericInput(program.id, 1, (value) => {
      program.id = value;
      card.querySelector(".program-title").textContent = `Program ${program.id}`;
      updateDraftJsonBox();
    });
    idRow.append(idInput);
    card.append(idRow);

    const opsList = document.createElement("div");
    opsList.className = "op-list";

    if (!program.ops.length) {
      const emptyOps = document.createElement("p");
      emptyOps.className = "small";
      emptyOps.textContent = "This program has no ops yet.";
      opsList.append(emptyOps);
    }

    program.ops.forEach((op, opIndex) => {
      const row = document.createElement("div");
      row.className = "op-row";

      const selectWrap = document.createElement("label");
      selectWrap.className = "field-row op-kind";
      selectWrap.innerHTML = `<span class="field-label">Op</span>`;
      const select = document.createElement("select");
      for (const entry of OP_KINDS) {
        const option = document.createElement("option");
        option.value = entry.kind;
        option.textContent = entry.label;
        option.selected = entry.kind === op.op;
        select.append(option);
      }
      select.addEventListener("change", () => {
        program.ops[opIndex] = defaultOpForKind(select.value);
        rerenderDraftScript();
      });
      selectWrap.append(select);
      row.append(selectWrap);

      const config = opConfig(op.op);
      const operandFields = document.createElement("div");
      operandFields.className = "operand-fields";
      for (const field of config.fields) {
        const operandRow = document.createElement("label");
        operandRow.className = "field-row operand-row";
        operandRow.innerHTML = `<span class="field-label">${field.label}</span>`;
        operandRow.append(
          numericInput(op[field.key], field.min, (value) => {
            op[field.key] = value;
            updateDraftJsonBox();
          }),
        );
        operandFields.append(operandRow);
      }
      if (!config.fields.length) {
        const badge = document.createElement("span");
        badge.className = "operand-badge";
        badge.textContent = "No operands";
        operandFields.append(badge);
      }
      row.append(operandFields);

      const opActions = document.createElement("div");
      opActions.className = "row-actions";
      opActions.append(
        actionButton("Up", () => moveOp(programIndex, opIndex, -1), opIndex === 0),
        actionButton("Down", () => moveOp(programIndex, opIndex, 1), opIndex === program.ops.length - 1),
        actionButton("Remove", () => {
          program.ops.splice(opIndex, 1);
          rerenderDraftScript();
        }),
      );
      row.append(opActions);
      opsList.append(row);
    });

    card.append(opsList);

    const footer = document.createElement("div");
    footer.className = "program-footer";
    footer.append(actionButton("Add Op", () => {
      program.ops.push(defaultOpForKind("action"));
      rerenderDraftScript();
    }));
    card.append(footer);

    scriptProgramsEl.append(card);
  });

  updateDraftJsonBox();
};

const actionButton = (label, onClick, disabled = false) => {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "micro-btn";
  button.textContent = label;
  button.disabled = disabled;
  button.addEventListener("click", onClick);
  return button;
};

const numericInput = (value, min, onChange) => {
  const input = document.createElement("input");
  input.type = "number";
  input.min = String(min);
  input.value = String(value);
  input.addEventListener("change", () => {
    const parsed = Number(input.value);
    onChange(Number.isFinite(parsed) ? Math.max(min, parsed) : min);
    input.value = String(Number.isFinite(parsed) ? Math.max(min, parsed) : min);
  });
  return input;
};

const moveItem = (items, fromIndex, delta) => {
  const toIndex = fromIndex + delta;
  if (toIndex < 0 || toIndex >= items.length) {
    return;
  }
  const [item] = items.splice(fromIndex, 1);
  items.splice(toIndex, 0, item);
};

const moveProgram = (programIndex, delta) => {
  moveItem(draftScript.programs, programIndex, delta);
  rerenderDraftScript();
};

const moveOp = (programIndex, opIndex, delta) => {
  moveItem(draftScript.programs[programIndex].ops, opIndex, delta);
  rerenderDraftScript();
};

const nextProgramId = () => {
  if (!draftScript.programs.length) {
    return 1;
  }
  return Math.max(...draftScript.programs.map((program) => Number(program.id) || 0)) + 1;
};

const renderView = (view) => {
  currentActiveMind = Number(view.active_mind ?? 1);
  applyPresetUi(view.preset);
  beatEl.textContent = view.beat;
  clockEl.textContent = String(view.clock);
  predicateToggle.checked = view.boss_vulnerable;
  snapshotHintEl.textContent = view.snapshot_hint;

  taskRowsEl.innerHTML = "";
  for (const task of view.tasks) {
    const row = document.createElement("tr");
    row.className = task.outcome;
    row.innerHTML = `
      <td>#${task.task_id}</td>
      <td>
        <strong>${task.program_label}</strong><br />
        <span class="small">ip=${task.ip}${task.parent ? ` parent=#${task.parent}` : " root"}</span>
      </td>
      <td>${mindLabelFor(currentPreset, Number(task.mind ?? 1))}</td>
      <td>${task.outcome}</td>
      <td>${task.wait}</td>
    `;
    taskRowsEl.append(row);
  }

  actionLogEl.innerHTML = "";
  const actions = view.actions.length
    ? view.actions
    : [{ label: "No actions yet. Tick the scene to start the encounter.", task_id: 0, action_id: 0 }];
  for (const action of actions.slice().reverse()) {
    const card = document.createElement("article");
    card.className = "beat-card";
    card.innerHTML = `
      <strong>${action.label}</strong>
      <span class="small">task #${action.task_id} action ${action.action_id}</span>
    `;
    actionLogEl.append(card);
  }

  traceLogEl.textContent = view.trace || "Trace log is empty.";
};

const refresh = (json) => {
  const view = parseView(json);
  renderView(view);
};

const syncDraftFromWasm = async () => {
  if (!wasmApp || typeof wasmApp.script_json !== "function") {
    draftScript = { programs: [] };
  } else {
    draftScript = parseScript(await wasmApp.script_json());
  }
  rerenderDraftScript();
};

const run = async (fn, { syncDraft = false } = {}) => {
  try {
    refresh(await fn());
    if (syncDraft) {
      await syncDraftFromWasm();
    }
    setStatus("WASM core ready");
  } catch (error) {
    setStatus(String(error), true);
    console.error(error);
  }
};

const requireScriptMethod = (name) => {
  if (!wasmApp || typeof wasmApp[name] !== "function") {
    throw new Error(`WASM build is missing ${name}(). Rebuild the showcase package.`);
  }
};

const loadPreset = async (preset) => {
  try {
    requireScriptMethod("load_preset");
    const json = await wasmApp.load_preset(preset);
    applyPresetUi(preset);
    refresh(json);
    await syncDraftFromWasm();
    setStatus("WASM core ready");
  } catch (error) {
    setStatus(String(error), true);
    console.error(error);
  }
};

const setAutoTick = (enabled) => {
  if (autoTimer) {
    clearInterval(autoTimer);
    autoTimer = null;
  }
  autoBtn.classList.toggle("active", enabled);
  autoBtn.textContent = enabled ? "Stop Auto" : "Auto Tick";
  if (enabled) {
    autoTimer = setInterval(() => {
      if (wasmApp) {
        run(() => wasmApp.tick());
      }
    }, 650);
  }
};

const init = async () => {
  try {
    const wasmModule = await import("./pkg/switchyard_demo_wasm.js");
    const wasmUrl = new URL("./pkg/switchyard_demo_wasm_bg.wasm", import.meta.url);
    await wasmModule.default({ module_or_path: wasmUrl });
    wasmApp = new wasmModule.ShowcaseApp();
    applyPresetUi(currentPreset);
    refreshCliHandoffUi();
    refresh(await wasmApp.view_json());
    await syncDraftFromWasm();
    setStatus("WASM core ready");
  } catch (error) {
    setStatus(`WASM core unavailable: ${error instanceof Error ? error.message : String(error)}`, true);
    console.error(error);
    return;
  }

  tickBtn.addEventListener("click", () => run(() => wasmApp.tick()));
  resetBtn.addEventListener("click", () => {
    setAutoTick(false);
    run(() => wasmApp.reset());
  });
  autoBtn.addEventListener("click", () => setAutoTick(!autoTimer));
  addProgramBtn.addEventListener("click", () => {
    draftScript.programs.push({ id: nextProgramId(), ops: [defaultOpForKind("succeed")] });
    rerenderDraftScript();
  });
  applyScriptBtn.addEventListener("click", () => {
    setAutoTick(false);
    run(
      () => {
        requireScriptMethod("load_script");
        return wasmApp.load_script(JSON.stringify(draftScript));
      },
      { syncDraft: true },
    );
  });
  resetScriptBtn.addEventListener("click", () => {
    setAutoTick(false);
    run(
      () => {
        requireScriptMethod("reset_script");
        return wasmApp.reset_script();
      },
      { syncDraft: true },
    );
  });
  presetEncounterBtn.addEventListener("click", () => {
    setAutoTick(false);
    void loadPreset("encounter");
  });
  presetShootemupBtn.addEventListener("click", () => {
    setAutoTick(false);
    void loadPreset("shootemup");
  });
  presetMultimindBtn.addEventListener("click", () => {
    setAutoTick(false);
    void loadPreset("multimind");
  });
  saveBtn.addEventListener("click", async () => {
    try {
      snapshotBoxEl.value = await wasmApp.save_snapshot();
      setStatus("Snapshot captured");
    } catch (error) {
      setStatus(String(error), true);
    }
  });
  restoreBtn.addEventListener("click", () => run(() => wasmApp.load_snapshot(snapshotBoxEl.value)));
  copyCatalogCliBtn.addEventListener("click", async () => {
    try {
      const json = await loadCatalogCliExport();
      previewCliExport("catalog", json);
      await writeClipboard(json);
      setStatus("Catalog export copied for switchyard-cli");
    } catch (error) {
      setStatus(String(error), true);
      console.error(error);
    }
  });
  downloadCatalogCliBtn.addEventListener("click", async () => {
    try {
      const json = await loadCatalogCliExport();
      previewCliExport("catalog", json);
      downloadJson("switchyard-catalog.json", json);
      setStatus("Catalog export downloaded");
    } catch (error) {
      setStatus(String(error), true);
      console.error(error);
    }
  });
  copySnapshotCliBtn.addEventListener("click", async () => {
    try {
      const json = await loadSnapshotCliExport();
      previewCliExport("snapshot", json);
      await writeClipboard(json);
      setStatus("Runtime snapshot export copied for switchyard-cli");
    } catch (error) {
      setStatus(String(error), true);
      console.error(error);
    }
  });
  downloadSnapshotCliBtn.addEventListener("click", async () => {
    try {
      const json = await loadSnapshotCliExport();
      previewCliExport("snapshot", json);
      downloadJson("switchyard-runtime-snapshot.json", json);
      setStatus("Runtime snapshot export downloaded");
    } catch (error) {
      setStatus(String(error), true);
      console.error(error);
    }
  });
  predicateToggle.addEventListener("change", () => run(() => wasmApp.set_boss_vulnerable(predicateToggle.checked)));

  for (const button of signalButtons) {
    button.addEventListener("click", () => run(() => wasmApp.emit_signal(Number(button.dataset.signal))));
  }
};

void init();
