# Уроки

Выводы после экспериментов и ошибок. Формат: [`WORKFLOWS.md`](WORKFLOWS.md).

---

## 2026-05-16 — Анизотропия превращает профиль формации в физику материала

Context: после приемлемой оценки первой Layer 1 лаборатории добавили `forward_multiplier` и `lateral_multiplier` в `FormationMaterial`, а распространение давления перевели с изотропного на взвешенный Лапласиан.  
What happened: `Wedge` перестал быть только профилем входящего давления и получил собственное поведение материала: высокая передача давления вдоль оси атаки и слабая боковая связность. `Line` осталась почти изотропным контролем.  
Lesson: формация должна задаваться не только тем, как она давит на противника, но и тем, как она внутри себя проводит, гасит и ломает давление. Это ближе к канону FMP: тип формации = анизотропия материала, а не визуальный пресет или бафф.  
Action: при добавлении новых формаций обязательно задавать пару `forward_multiplier` / `lateral_multiplier` и проверять сценарии с фланговым давлением; для Phalanx и Crowd это станет главным отличием, а не просто новым названием.  
Links: [`DECISIONS.md`](DECISIONS.md), [`modules/combat_simulation.md`](modules/combat_simulation.md), [`FMP.md`](../knowledge_base/FMP.md)

---

## 2026-05-16 — bevy_egui UI-системы не запускать в обычном Update

Context: первый запуск Bevy/FMP-лаборатории с `bevy_egui 0.39.1`.  
What happened: `cargo check` проходил, но `cargo run` падал в `lab_ui` с panic `Called available_rect() before Context::run()`.  
Lesson: для текущего `bevy_egui` UI-системы первичного контекста нужно регистрировать в `EguiPrimaryContextPass`, иначе egui-контекст может использоваться до `Context::run()`.  
Action: `lab_ui` перенесён из `Update` в `EguiPrimaryContextPass`; при добавлении новых egui-систем держать их в этом schedule.  
Links: [`modules/bevy_architecture.md`](modules/bevy_architecture.md), [`src/main.rs`](../src/main.rs)

---

<!-- Шаблон для первой записи:

## YYYY-MM-DD — Краткое название

Context: ...
What happened: ...
Lesson: ...
Action: ...
Links: ...

-->
