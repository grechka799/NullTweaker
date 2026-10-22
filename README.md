# NullTweaker

Деблоат и твики для Windows 10/11. Tauri 2 + Rust + Svelte 5.

Отключает телеметрию, рекламу и предустановленный мусор (Cortana, Teams,
Bing-набор и т.п.), умеет расчищать UWP-пакеты, гонять SFC/DISM, чистить TEMP
и ставить схему питания «Максимальная производительность».

Главная фишка — ничего не рвётся «навсегда»: перед каждым применением твиков
в `%APPDATA%\io.nulltweaker.app\snapshots` пишется снапшот реестра, и откат
может быть хоть по кнопке, хоть щелчком по снимку.

## Сборка из исходников

Нужны Node ≥ 20, pnpm, Rust (stable-msvc) и WebView2.

```bash
cd app
pnpm install
pnpm tauri dev    # разработка
pnpm tauri build  # релиз
```

Релиз-артефакты:

- портативный exe — `app/src-tauri/target/release/nulltweaker.exe`
- установщик — `app/src-tauri/target/release/bundle/nsis/`

## Как устроено

- `app/src-tauri/src/tweaks.rs` — каталог твиков. Каждый твик это просто список
  изменений реестра (`Set`/`Delete`), поэтому новый твик = один блок в `vec![]`.
- `app/src-tauri/src/registry.rs` — тонкая обёртка над `winreg`
  (HKLM читается с `KEY_WOW64_64KEY`).
- `app/src-tauri/src/snapshot.rs` — снимок значений перед применением и откат.
- `app/src-tauri/src/lib.rs` — все Tauri-команды. Оформление окна (титлбар,
  темы, свечение) целиком живёт в `app/src/App.svelte` + `app.css`.

## Что дальше

Возможные направления, если кому-то интересно докручивать: управление службами
и запланированными задачами, экспорт/импорт профиля твиков (`*.ntprofile`),
исключения для доверенных приложений.

## Лицензия

MIT, см. [LICENSE](LICENSE).
