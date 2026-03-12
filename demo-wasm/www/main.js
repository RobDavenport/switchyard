const statusEl = document.getElementById("wasm-status");
const beatEl = document.getElementById("beat-line");
const clockEl = document.getElementById("clock-line");
const taskRowsEl = document.getElementById("task-rows");
const actionLogEl = document.getElementById("action-log");
const traceLogEl = document.getElementById("trace-log");
const snapshotBoxEl = document.getElementById("snapshot-box");
const snapshotHintEl = document.getElementById("snapshot-hint");
const tickBtn = document.getElementById("tick-btn");
const autoBtn = document.getElementById("auto-btn");
const resetBtn = document.getElementById("reset-btn");
const saveBtn = document.getElementById("save-btn");
const restoreBtn = document.getElementById("restore-btn");
const predicateToggle = document.getElementById("predicate-toggle");
const signalButtons = [...document.querySelectorAll("[data-signal]")];

let wasmApp = null;
let autoTimer = null;

const setStatus = (text, isError = false) => {
  statusEl.textContent = text;
  statusEl.style.color = isError ? "#ab3428" : "inherit";
};

const parseView = (json) => JSON.parse(json);

const renderView = (view) => {
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
      <td>${task.outcome}</td>
      <td>${task.wait}</td>
    `;
    taskRowsEl.append(row);
  }

  actionLogEl.innerHTML = "";
  const actions = view.actions.length ? view.actions : [{ label: "No actions yet. Tick the scene to start the encounter.", task_id: 0, action_id: 0 }];
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
  renderView(parseView(json));
};

const run = async (fn) => {
  try {
    refresh(await fn());
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
    await wasmModule.default();
    wasmApp = new wasmModule.ShowcaseApp();
    setStatus("WASM core ready");
    refresh(await wasmApp.view_json());
  } catch (error) {
    setStatus("WASM core unavailable. Run wasm-pack build demo-wasm --target web --release --out-dir www/pkg", true);
    console.error(error);
    return;
  }

  tickBtn.addEventListener("click", () => run(() => wasmApp.tick()));
  resetBtn.addEventListener("click", () => {
    setAutoTick(false);
    run(() => wasmApp.reset());
  });
  autoBtn.addEventListener("click", () => setAutoTick(!autoTimer));
  saveBtn.addEventListener("click", async () => {
    try {
      snapshotBoxEl.value = await wasmApp.save_snapshot();
    } catch (error) {
      setStatus(String(error), true);
    }
  });
  restoreBtn.addEventListener("click", () => run(() => wasmApp.load_snapshot(snapshotBoxEl.value)));
  predicateToggle.addEventListener("change", () => run(() => wasmApp.set_boss_vulnerable(predicateToggle.checked)));

  for (const button of signalButtons) {
    button.addEventListener("click", () => run(() => wasmApp.emit_signal(Number(button.dataset.signal))));
  }
};

void init();
