<script lang="ts">
  import { onMount } from "svelte";
  import { fly, fade } from "svelte/transition";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import * as api from "./lib/api";
  import type { SystemInfo, TweakDto, UwpApp } from "./lib/api";
  import { lang, setLang, pick } from "./lib/i18n.svelte";

  type Tab = "home" | "personalization" | "performance" | "telemetry" | "uwp" | "recovery" | "settings";
  type ThemePref = "system" | "dark" | "light";

  const win = getCurrentWindow();

  let info: SystemInfo | null = $state(null);
  let tweaks: TweakDto[] = $state([]);
  let states: Record<string, boolean> = $state({});
  let selected: Record<string, boolean> = $state({});
  let busy = $state(false);
  let tab = $state<Tab>("home");
  let isMax = $state(false);

  let toast = $state<{ text: string; error: boolean; id: number } | null>(null);
  let toastTimer: ReturnType<typeof setTimeout>;
  function notify(text: string, error = false) {
    toast = { text, error, id: Date.now() };
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), 2500);
  }

  const glowSaved = typeof localStorage !== "undefined" ? localStorage.getItem("nt-glow") : null;
  let glowOn = $state(glowSaved !== "0");

  const themeSaved = typeof localStorage !== "undefined" ? localStorage.getItem("nt-theme") : null;
  let themePref = $state<ThemePref>(
    themeSaved === "dark" || themeSaved === "light" ? themeSaved : "system"
  );

  const mql = typeof window !== "undefined" ? window.matchMedia("(prefers-color-scheme: dark)") : null;

  function applyTheme() {
    const eff =
      themePref === "system" ? (mql?.matches ? "dark" : "light") : themePref;
    document.documentElement.dataset.theme = eff;
  }

  function setThemePref(v: ThemePref) {
    themePref = v;
    try {
      localStorage.setItem("nt-theme", v);
    } catch { /* noop */ }
    applyTheme();
  }

  function glowMove(e: MouseEvent) {
    if (!glowOn) return;
    const el = document.getElementById("glow");
    const area = document.getElementById("content");
    if (!el || !area) return;
    const r = area.getBoundingClientRect();
    el.style.left = `${e.clientX - r.left}px`;
    el.style.top = `${e.clientY - r.top}px`;
  }

  function toggleGlow() {
    glowOn = !glowOn;
    try {
      localStorage.setItem("nt-glow", glowOn ? "1" : "0");
    } catch { /* noop */ }
  }

  const tabTitles: Record<Tab, { ru: string; en: string }> = {
    home: { ru: "Главная", en: "Home" },
    personalization: { ru: "Персонализация", en: "Personalization" },
    performance: { ru: "Ускорение", en: "Performance" },
    telemetry: { ru: "Телеметрия", en: "Telemetry" },
    uwp: { ru: "UWP", en: "UWP" },
    recovery: { ru: "Восстановление", en: "Recovery" },
    settings: { ru: "Настройки", en: "Settings" },
  };

  function tabSub(key: Tab): { ru: string; en: string } {
    if (key === "home")
      return {
        ru: "Чистая Windows — без телеметрии и мусора. Всё обратимо.",
        en: "A clean Windows — no telemetry, no bloat. Everything reversible.",
      };
    if (key === "uwp")
      return {
        ru: "Предустановленный мусор Pro/Home: Cortana, Teams, Bing-набор и т.п.",
        en: "Pro/Home preinstalled bloat: Cortana, Teams, Bing apps etc.",
      };
    if (key === "recovery")
      return {
        ru: "Точка восстановления, TEMP, SFC и DISM.",
        en: "Restore point, TEMP cleanup, SFC and DISM.",
      };
    if (key === "personalization")
      return {
        ru: "Проводник, панель задач и контекстное меню.",
        en: "Explorer, taskbar and context menu.",
      };
    if (key === "performance")
      return {
        ru: "Анимации, прозрачность и отклик системы.",
        en: "Animations, transparency and system responsiveness.",
      };
    if (key === "telemetry")
      return {
        ru: "Телеметрия, реклама и отправка данных Microsoft.",
        en: "Telemetry, ads and Microsoft data collection.",
      };
    return { ru: "Оформление и сведения о системе.", en: "Appearance and system info." };
  }

  const texts = {
    greetTitle: { ru: "Добро пожаловать в NullTweaker", en: "Welcome to NullTweaker" },
    greetText: {
      ru: "Тонкая настройка Windows в один клик: отключай телеметрию, удаляй предустановленный мусор и ускоряй систему. Перед каждым применением снимается снапшот реестра — ничего не потеряешь.",
      en: "One-click Windows tuning: turn off telemetry, remove preinstalled bloat and speed up the system. A registry snapshot is taken before every apply — nothing gets lost.",
    },
    changelog: { ru: "Changelog", en: "Changelog" },
    githubBtn: { ru: "Открыть GitHub", en: "Open GitHub" },
    apply: { ru: "Применить", en: "Apply" },
    revert: { ru: "Откатить", en: "Revert" },
    all: { ru: "Всё", en: "All" },
    none: { ru: "Очистить", en: "Clear" },
    safePreset: { ru: "Безопасный", en: "Safe" },
    empty: { ru: "Ничего не выбрано.", en: "Nothing selected." },
    restartExplorer: { ru: "Перезапустить проводник", en: "Restart Explorer" },
    restorePoint: { ru: "Точка восстановления системы", en: "System restore point" },
    create: { ru: "Создать", en: "Create" },
    clearTemp: { ru: "Очистить папку TEMP", en: "Clear TEMP folder" },
    scanDisk: { ru: "Проверка системных файлов (SFC)", en: "System file check (SFC)" },
    startScan: { ru: "Проверить", en: "Run check" },
    dism: { ru: "Проверка хранилища (DISM)", en: "Component store check (DISM)" },
    recycleBin: { ru: "Очистить корзину", en: "Empty Recycle Bin" },
    clear: { ru: "Очистить", en: "Clear" },
    appearance: { ru: "Оформление", en: "Appearance" },
    theme: { ru: "Тема", en: "Theme" },
    themeSystem: { ru: "Системная", en: "System" },
    themeDark: { ru: "Тёмная", en: "Dark" },
    themeLight: { ru: "Светлая", en: "Light" },
    glow: { ru: "Свечение за курсором", en: "Cursor glow" },
    sysinfo: { ru: "О системе", en: "About system" },
    os: { ru: "Редакция", en: "Edition" },
    version: { ru: "Версия", en: "Version" },
    build: { ru: "Сборка", en: "Build" },
    arch: { ru: "Архитектура", en: "Architecture" },
    privileges: { ru: "Запуск", en: "Launch" },
    elevated: { ru: "от администратора", en: "elevated" },
    notElevated: { ru: "без повышения", en: "non-elevated" },

    uwpList: { ru: "UWP-приложения текущего пользователя", en: "UWP apps for the current user" },
    uwpSearch: { ru: "Поиск по пакетам…", en: "Search packages…" },
    uwpRemoveAll: { ru: "Удалить все", en: "Remove all" },
    uwpRemoveSel: { ru: "Удалить выбранное", en: "Remove selected" },
    uwpRemoving: { ru: "Удаление…", en: "Removing…" },
    uwpEmpty: { ru: "Мусорных пакетов не найдено.", en: "No bloat packages found." },
    uwpLoading: { ru: "Загрузка списка…", en: "Loading list…" },
    uwpDone: { ru: "Удалено:", en: "Uninstalled:" },
    uwpFail: { ru: "не удалось:", en: "failed:" },
    perfTools: { ru: "Инструменты", en: "Tools" },
    powerPlan: { ru: "Схема питания: максимальная производительность", en: "Power plan: max performance" },
    powerOn: { ru: "Схема максимальной производительности включена", en: "Max performance plan enabled" },
    powerOff: { ru: "Вернулась сбалансированная схема", en: "Balanced plan restored" },
    appliedToast: { ru: "Применено", en: "Applied" },
    appliedSome: { ru: "применено", en: "applied" },
    revertToast: { ru: "Откат", en: "Reverted" },
    restored: { ru: "Снимок восстановлен", en: "Snapshot restored" },
    noSnapshots: { ru: "Снимков ещё нет", en: "No snapshots yet" },
    restarted: { ru: "Проводник перезапущен", en: "Explorer restarted" },
    rpOk: { ru: "Точка восстановления создана", en: "Restore point created" },
    rpFail: { ru: "Не удалось создать точку восстановления", en: "Failed to create restore point" },
    tempCleared: { ru: "TEMP очищена:", en: "TEMP cleared:" },
    binCleared: { ru: "Корзина очищена", en: "Recycle bin cleared" },
    sfcRunning: { ru: "SFC: проверка идёт…", en: "SFC: running…" },
    sfcOk: { ru: "SFC: нарушений не найдено", en: "SFC: no issues found" },
    sfcFail: { ru: "SFC: код", en: "SFC: code" },
    dismRunning: { ru: "DISM: проверка идёт…", en: "DISM: running…" },
    dismOk: { ru: "DISM: хранилище цело", en: "DISM: store healthy" },
    dismFail: { ru: "DISM: код", en: "DISM: code" },
  };

  function t(key: keyof typeof texts): string {
    return pick(texts[key]);
  }

  const GITHUB_URL = "https://github.com/grechka799/NullTweaker";

  const CHANGELOG: { v: string; items: { ru: string; en: string }[] }[] = [
    {
      v: "0.2",
      items: [
        { ru: "UWP-дебloat: точечный список предустановленного мусора Pro/Home", en: "UWP debloat: targeted Pro/Home preinstalled junk list" },
        { ru: "Copilot и Recall отключаются политиками реестра", en: "Copilot and Recall disabled via registry policies" },
        { ru: "Схема питания «Максимальная производительность» в один клик", en: "One-click Ultimate Performance power plan" },
        { ru: "Тёмная / светлая / системная тема, анимированный RU/EN", en: "Dark / light / system theme, animated RU/EN switch" },
      ],
    },
    {
      v: "0.1",
      items: [
        { ru: "Первый релиз: 30+ твиков (телеметрия, проводник, скорость)", en: "First release: 30+ tweaks (telemetry, explorer, speed)" },
        { ru: "Автоснапшот реестра перед применением и полный откат", en: "Auto registry snapshot before applying, full revert" },
        { ru: "Восстановление: точка восстановления, TEMP, SFC, DISM", en: "Maintenance: restore point, TEMP, SFC, DISM" },
      ],
    },
  ];

  function mb(bytes: number): string {
    return `${(bytes / 1048576).toFixed(1)} MB`;
  }

  function applyTabFilter(list: TweakDto[], active: Tab): TweakDto[] {
    if (active === "telemetry") return list.filter((x) => x.group === "privacy");
    if (active === "personalization") return list.filter((x) => x.group === "explorer");
    if (active === "performance") return list.filter((x) => x.group === "performance");
    return [];
  }

  let isWin11 = $derived(info ? Number.parseInt(info.build_number, 10) >= 22000 : true);
  let visible = $derived(applyTabFilter(tweaks, tab).filter((x) => !x.win11_only || isWin11));
  let selectedIds = $derived(visible.filter((x) => selected[x.id]).map((x) => x.id));

  const safePresetIds = new Set([
    "disable-advertising-id",
    "disable-tailored-experiences",
    "disable-start-suggestions",
    "disable-bing-search",
    "show-file-extensions",
    "show-hidden-files",
    "classic-context-menu",
    "fast-menu-show",
    "no-delete-confirm",
  ]);

  function select(ids: Iterable<string>, value: boolean) {
    selected = { ...selected };
    for (const id of ids) selected[id] = value;
  }

  function switchTab(next: Tab) {
    tab = next;
    if (next === "uwp") void loadUwp();
  }

  async function refreshStates() {
    states = await api.getTweaksState(tweaks.map((x) => x.id));
  }

  async function doApply() {
    if (!selectedIds.length) return notify(t("empty"));
    busy = true;
    try {
      const results = await api.applyTweaks(selectedIds);
      await refreshStates();
      const ok = results.filter((r) => r.ok).length;
      const failed = results.filter((r) => !r.ok);
      if (failed.length) {
        notify(`${ok} ✓ · ${failed.length} ✗`, true);
      } else {
        notify(`${t("appliedToast")} · ${ok}`);
      }
    } catch (e) {
      notify(String(e), true);
    } finally {
      busy = false;
    }
  }

  async function doRevert(ids?: string[]) {
    const target = ids ?? selectedIds;
    if (!target.length) return notify(t("empty"));
    busy = true;
    try {
      const results = await api.revertTweaks(target);
      await refreshStates();
      const ok = results.filter((r) => r.ok).length;
      notify(`${t("revertToast")} · ${ok}`);
    } catch (e) {
      notify(String(e), true);
    } finally {
      busy = false;
    }
  }

  async function doRestore() {
    busy = true;
    try {
      const n = await api.restoreSnapshot();
      await refreshStates();
      notify(`${t("restored")}: ${n}`);
    } catch {
      notify(t("noSnapshots"), true);
    } finally {
      busy = false;
    }
  }

  async function doRestartExplorer() {
    try {
      await api.restartExplorer();
      notify(t("restarted"));
    } catch (e) {
      notify(String(e), true);
    }
  }

  async function doCreateRp() {
    busy = true;
    try {
      await api.createRestorePoint();
      notify(t("rpOk"));
    } catch (e) {
      notify(`${t("rpFail")}: ${String(e)}`, true);
    } finally {
      busy = false;
    }
  }

  async function doClearTemp() {
    busy = true;
    try {
      const n = await api.clearTemp();
      notify(`${t("tempCleared")} ${mb(n)}`);
    } catch (e) {
      notify(String(e), true);
    } finally {
      busy = false;
    }
  }

  async function doSfc() {
    busy = true;
    notify(t("sfcRunning"));
    try {
      const code = await api.runSfc();
      if (code === 0) notify(t("sfcOk"));
      else notify(`${t("sfcFail")} ${code}`, true);
    } catch (e) {
      notify(String(e), true);
    } finally {
      busy = false;
    }
  }

  async function doDism() {
    busy = true;
    notify(t("dismRunning"));
    try {
      const code = await api.runDism();
      if (code === 0) notify(t("dismOk"));
      else notify(`${t("dismFail")} ${code}`, true);
    } catch (e) {
      notify(String(e), true);
    } finally {
      busy = false;
    }
  }

  async function doBin() {
    busy = true;
    try {
      await api.emptyRecycleBin();
      notify(t("binCleared"));
    } catch (e) {
      notify(String(e), true);
    } finally {
      busy = false;
    }
  }

  let uwpList: UwpApp[] = $state([]);
  let uwpLoading = $state(false);
  let uwpFilter = $state("");
  let uwpSelected: Record<string, boolean> = $state({});

  async function loadUwp(force = false) {
    if (uwpLoading || (uwpList.length && !force)) return;
    uwpLoading = true;
    try {
      uwpList = await api.listUwp();
      uwpSelected = {};
    } catch (e) {
      notify(String(e), true);
    } finally {
      uwpLoading = false;
    }
  }

  let filteredUwp = $derived(
    uwpList.filter((x) =>
      x.name.toLowerCase().includes(uwpFilter.trim().toLowerCase())
    )
  );
  let selectedUwpCount = $derived(
    uwpList.filter((x) => uwpSelected[x.full_name]).length
  );

  async function doRemoveUwp(ids: string[]) {
    if (!ids.length) return;
    busy = true;
    try {
      const res = await api.removeUwp(ids);
      await loadUwp(true);
      const ok = res.filter((r) => r.ok).length;
      const fail = res.length - ok;
      if (fail) notify(`${t("uwpDone")} ${ok} · ${t("uwpFail")} ${fail}`, true);
      else notify(`${t("uwpDone")} ${ok}`);
    } catch (e) {
      notify(String(e), true);
    } finally {
      busy = false;
    }
  }

  function doRemoveSelected() {
    return doRemoveUwp(
      uwpList.filter((x) => uwpSelected[x.full_name]).map((x) => x.full_name)
    );
  }

  function doRemoveAll() {
    return doRemoveUwp(uwpList.map((x) => x.full_name));
  }

  let maxPower = $state(false);
  let maxPowerBusy = $state(false);

  async function onPowerToggle() {
    const target = !maxPower;
    maxPowerBusy = true;
    maxPower = target;
    try {
      await api.setMaxPower(target);
      notify(target ? t("powerOn") : t("powerOff"));
    } catch (e) {
      maxPower = !target;
      notify(String(e), true);
    } finally {
      maxPowerBusy = false;
    }
  }

  let cleanupResize: (() => void) | undefined = undefined;

  onMount(() => {
    applyTheme();
    document.addEventListener("mousemove", glowMove, { passive: true });
    if (mql) mql.addEventListener("change", applyTheme);
    api.getSystemInfo().then((v) => (info = v));
    api.listTweaks().then(async (v) => {
      tweaks = v;
      await refreshStates();
    });
    api.getMaxPowerState().then((v) => (maxPower = v));
    win.isMaximized().then((v) => (isMax = v));
    win.onResized(async () => {
      isMax = await win.isMaximized();
    }).then((un) => (cleanupResize = un));

    return () => {
      document.removeEventListener("mousemove", glowMove);
      cleanupResize?.();
      mql?.removeEventListener("change", applyTheme);
    };
  });
</script>

<div class="app">
  <!-- Кастомный титлбар -->
  <header class="titlebar" data-tauri-drag-region>
    <div class="brand">
      <img class="logo" src="/logo.png" alt="" />
      <span class="brand-name">NullTweaker</span>
    </div>
    <div class="drag-space" data-tauri-drag-region></div>
    <div class="lang" onmousedown={(e) => e.stopPropagation()}>
      <div
        class="lang-slider"
        class:right={lang.value !== "ru"}
        aria-hidden="true"
      ></div>
      <button
        type="button"
        class:active={lang.value === "ru"}
        onclick={() => setLang("ru")}
      >RU</button>
      <button
        type="button"
        class:active={lang.value === "en"}
        onclick={() => setLang("en")}
      >EN</button>
    </div>
    <div class="win-controls" onmousedown={(e) => e.stopPropagation()}>
      <button class="win-btn" onclick={() => win.minimize()} aria-label="Minimize">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M6 12h12"/></svg>
      </button>
      <button class="win-btn" onclick={() => win.toggleMaximize()} aria-label="Maximize">
        {#if isMax}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M9 9h10v10H9z"/><path d="M15 5H5v10"/></svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><rect x="6" y="6" width="12" height="12" rx="1.5"/></svg>
        {/if}
      </button>
      <button class="win-btn close" onclick={() => win.close()} aria-label="Close">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M7 7l10 10M17 7L7 17"/></svg>
      </button>
    </div>
  </header>

  <div class="body">
    <main id="content" onmousemove={glowMove}>
      {#if glowOn}
        <div class="glow" id="glow" aria-hidden="true"></div>
      {/if}
      {#key tab}
        <div class="page" in:fly={{ y: 10, duration: 200 }}>
          {#if tab === "home"}
            <h1>{pick(tabTitles.home)}</h1>
            <p class="sub-label">{pick(tabSub(tab))}</p>

            <div class="hero">
              <img class="hero-logo" src="/logo.png" alt="NullTweaker" />
              <div class="hero-body">
                <span class="hero-title">{t("greetTitle")}</span>
                <p class="hero-text">{t("greetText")}</p>
              </div>
              <button class="btn primary md" onclick={() => api.openUrl(GITHUB_URL)}>
                {t("githubBtn")}
              </button>
            </div>

            <h2 class="subhead">{t("changelog")}</h2>
            <div class="rows">
              {#each CHANGELOG as entry (entry.v)}
                <div class="log-entry">
                  <span class="log-ver mono">v{entry.v}</span>
                  <ul class="log-items">
                    {#each entry.items as item (pick(item))}
                      <li>{pick(item)}</li>
                    {/each}
                  </ul>
                </div>
              {/each}
            </div>
          {:else if tab === "recovery"}
            <h1>{pick(tabTitles.recovery)}</h1>
            <p class="sub-label">{pick(tabSub("recovery"))}</p>
            <div class="rows">
              <div class="tool">
                <span class="tool-name">{t("restorePoint")}</span>
                <button class="btn ghost" disabled={busy} onclick={doCreateRp}>{t("create")}</button>
              </div>
              <div class="tool">
                <span class="tool-name">{t("clearTemp")}</span>
                <button class="btn ghost" disabled={busy} onclick={doClearTemp}>{t("clear")}</button>
              </div>
              <div class="tool">
                <span class="tool-name">{t("recycleBin")}</span>
                <button class="btn ghost" disabled={busy} onclick={doBin}>{t("clear")}</button>
              </div>
              <div class="tool">
                <span class="tool-name">{t("scanDisk")}</span>
                <button class="btn ghost" disabled={busy} onclick={doSfc}>{t("startScan")}</button>
              </div>
              <div class="tool">
                <span class="tool-name">{t("dism")}</span>
                <button class="btn ghost" disabled={busy} onclick={doDism}>{t("startScan")}</button>
              </div>
            </div>
          {:else if tab === "uwp"}
            <h1>{pick(tabTitles.uwp)}</h1>
            <p class="sub-label">{pick(tabSub(tab))}</p>

            <input
              class="search"
              type="text"
              placeholder={t("uwpSearch")}
              bind:value={uwpFilter}
            />

            <div class="rows">
              {#if uwpLoading}
                <div class="row muted-row">{t("uwpLoading")}</div>
              {:else if !filteredUwp.length}
                <div class="row muted-row">{t("uwpEmpty")}</div>
              {:else}
                {#each filteredUwp as app, i (app.name)}
                  <div class="row" in:fly={{ y: 5, duration: 140, delay: Math.min(i * 18, 180) }}>
                    <span class="row-name">{app.name}</span>
                    <span class="row-ver mono">{app.version}</span>
                    <input
                      type="checkbox"
                      class="switch"
                      bind:checked={uwpSelected[app.full_name]}
                      aria-label={app.name}
                    />
                  </div>
                {/each}
              {/if}
            </div>

            <div class="uwp-actions">
              <button
                class="btn big primary"
                disabled={busy || uwpLoading || !uwpList.length}
                onclick={doRemoveAll}
              >{uwpLoading ? t("uwpRemoving") : t("uwpRemoveAll")}</button>
              <button
                class="btn big ghost"
                disabled={busy || uwpLoading || !selectedUwpCount}
                onclick={doRemoveSelected}
              >{t("uwpRemoveSel")}{selectedUwpCount ? ` (${selectedUwpCount})` : ""}</button>
            </div>

          {:else if tab === "settings"}
            <h1>{pick(tabTitles.settings)}</h1>
            <p class="sub-label">{pick(tabSub("settings"))}</p>

            <h2 class="subhead">{t("appearance")}</h2>
            <div class="rows">
              <div class="tool">
                <span class="tool-name">{t("theme")}</span>
                <div class="seg">
                  {#each (["system", "dark", "light"] as const) as v (v)}
                    <button
                      class="seg-btn"
                      class:on={themePref === v}
                      onclick={() => setThemePref(v)}
                    >{t(("theme" + v[0].toUpperCase() + v.slice(1)) as keyof typeof texts)}</button>
                  {/each}
                </div>
              </div>
              <div class="tool">
                <span class="tool-name">{t("glow")}</span>
                <input type="checkbox" class="switch" checked={glowOn} onchange={toggleGlow} />
              </div>
            </div>

            <h2 class="subhead">{t("sysinfo")}</h2>
            <div class="rows">
              {#if info}
                <div class="kv"><span>{t("os")}</span><b>{info.windows_product}</b></div>
                <div class="kv"><span>{t("version")}</span><b>{info.display_version}</b></div>
                <div class="kv"><span>{t("build")}</span><b class="mono">{info.build_number}</b></div>
                <div class="kv"><span>{t("arch")}</span><b class="mono">{info.arch}</b></div>
                <div class="kv"><span>{t("privileges")}</span><b>{info.is_admin ? t("elevated") : t("notElevated")}</b></div>
              {/if}
            </div>
          {:else}
            <h1>{pick(tabTitles[tab])}</h1>
            <p class="sub-label">{pick(tabSub(tab))}</p>

            <div class="toolbar">
              <button class="btn ghost sm" onclick={() => select(safePresetIds, true)}>{t("safePreset")}</button>
              <button class="btn ghost sm" onclick={() => select(visible.map((x) => x.id), true)}>{t("all")}</button>
              <button class="btn ghost sm" onclick={() => select(visible.map((x) => x.id), false)}>{t("none")}</button>
              <span class="spacer"></span>
              <span class="counter">{selectedIds.length}/{visible.length}</span>
            </div>

            <div class="rows">
              {#each visible as tweak, i (tweak.id)}
                <div
                  class="row"
                  in:fly={{ y: 6, duration: 160, delay: Math.min(i * 24, 240) }}
                >
                  <span class="row-name">{pick(tweak.title)}</span>
                  <input
                    type="checkbox"
                    class="switch"
                    bind:checked={selected[tweak.id]}
                    aria-label={pick(tweak.title)}
                  />
                </div>
              {/each}
            </div>

            {#if tab === "performance"}
              <h2 class="subhead">{t("perfTools")}</h2>
              <div class="rows">
                <div class="tool">
                  <span class="tool-name">{t("powerPlan")}</span>
                  <input
                    type="checkbox"
                    class="switch"
                    checked={maxPower}
                    disabled={maxPowerBusy}
                    onchange={onPowerToggle}
                    aria-label={t("powerPlan")}
                  />
                </div>
              </div>
            {/if}
          {/if}
        </div>
      {/key}
    </main>

    <aside class="rail">
      {#each (["home", "personalization", "performance", "telemetry", "uwp", "recovery", "settings"] as const) as key (key)}
        <button class="tab" class:active={tab === key} onclick={() => switchTab(key)}>
          <span>{pick(tabTitles[key])}</span>
        </button>
      {/each}
    </aside>
  </div>

  <footer>
    <button class="btn ghost sm" onclick={doRestartExplorer}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 12a9 9 0 0 1 15-6.7L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-15 6.7L3 16"/><path d="M3 21v-5h5"/></svg>
      {t("restartExplorer")}
    </button>
    <span class="spacer"></span>
    {#if tab !== "recovery" && tab !== "settings" && tab !== "uwp" && tab !== "home"}
      <button class="btn ghost md" disabled={busy} onclick={() => doRevert()}>{t("revert")}</button>
      <button class="btn primary md" disabled={busy || !selectedIds.length} onclick={doApply}>
        {t("apply")}
      </button>
    {:else if tab === "recovery"}
      <button class="btn ghost md" disabled={busy} onclick={() => doRevert(tweaks.map((x) => x.id))}>
        {t("revert")}
      </button>
    {/if}
  </footer>

  {#if toast}
    {#key toast.id}
      <div class="toast" class:error={toast.error} in:fly={{ y: 12, duration: 180 }} out:fade={{ duration: 150 }}>
        {toast.text}
      </div>
    {/key}
  {/if}
</div>
