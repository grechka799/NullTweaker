export type Lang = "ru" | "en";

const saved = typeof localStorage !== "undefined" ? localStorage.getItem("nt-lang") : null;
const initial: Lang = saved === "en" || saved === "ru" ? saved : "ru";

export const lang = $state<{ value: Lang }>({ value: initial });

export function setLang(value: Lang) {
  lang.value = value;
  try {
    localStorage.setItem("nt-lang", value);
  } catch {
  }
}

export function pick(loc: { ru: string; en: string }): string {
  return lang.value === "en" ? loc.en : loc.ru;
}
