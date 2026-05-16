# WarSim

Документация проекта: [`knowledge_base/`](knowledge_base/) (канон).  
Память ИИ-агента: [`AGENT_MIND.md`](AGENT_MIND.md) → [`agent_mind/`](agent_mind/).

## Текущий baseline

Сейчас runnable-часть проекта — первая FMP Layer 1 лаборатория:

- две формации в Bevy 0.18;
- поле давления Formation Field;
- локальный fracture строя;
- сценарии Line/Wedge/Flank/Low Morale/Fatigue;
- debug UI с метриками pressure/yield/fracture и snapshot-разрезами center/edge/front/rear/flank;
- unit- и regression-тесты для базовых инвариантов Layer 1.

Это ещё не полноценная игра и ещё не Contact Zone / Individual Physics. Цель текущего baseline — измеряемый стенд для калибровки Formation Material Physics.

## Требования

Проект использует Rust 2024 edition и Bevy 0.18. Toolchain зафиксирован в [`rust-toolchain.toml`](rust-toolchain.toml):

- stable Rust;
- `rustfmt`;
- `clippy`.

## Запуск лаборатории

```powershell
cargo run
```

## Проверки

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Эти же проверки запускаются в GitHub Actions на pull request и push в `main`/`master`.