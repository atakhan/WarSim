# Открытые вопросы

Вопросы без срочного ответа. Формат: [`WORKFLOWS.md`](WORKFLOWS.md).

---

## Q-001 — Размер и дискретизация Formation Field

Status: open  
Raised: 2026-05-16  
Context: слой 1 хранит поле давления по слотам формации; не зафиксировано в каноне число слотов и топология сетки.  
Blocking: детальная реализация `propagate_pressure_waves`  
Notes: зависит от визуальной плотности строя и производительности; возможны 1D цепочки для прототипа vs 2D решётка.  
Answer: —

---

## Q-002 — Соответствие Python-симуляции и Rust-лаборатории

Status: open  
Raised: 2026-05-16  
Context: в stack notes упоминается доказательство FMP «не только в Python»; репозиторий Python-кода не индексирован в текущем дереве.  
Blocking: валидация HYP-001  
Notes: нужно ли портировать параметры 1:1 или достаточно качественного совпадения поведения.  
Answer: —

---

## Q-003 — Минимальный набор формаций для лаборатории

Status: partially resolved (anisotropy implemented, need more archetypes)
Raised: 2026-05-16  
Context: анизотропия материала завязана на тип формации (клин, фаланга и т.д.).  
Blocking: контент сцен лаборатории  
Notes: для MVP лаборатории возможно 2–3 архетипа (линия, клин, рыхлая масса).  
Answer: (2026-05-16) Физическая поддержка анизотропии добавлена (разные множители передачи давления `forward_multiplier` и `lateral_multiplier`). Линия и Клин уже работают с разной физикой. Осталось добавить новые профили (например, Phalanx и Crowd).

---

## Q-004 — Debug UI: какие параметры обязательны в bevy_egui

Status: partially resolved
Raised: 2026-05-16  
Context: stack notes предлагают смотреть параметры формаций в реальном времени.  
Blocking: удобство итерации дизайнером/разработчиком  
Notes: кандидаты: stiffness, yield_strength, viscosity, мораль отряда, визуализация pressure_wave.  
Answer: (2026-05-16) Базовые параметры (stiffness, yield_strength, viscosity, morale, fatigue) и множители анизотропии выведены. Метрики (average pressure, peak pressure, fracture ratio) тоже есть. Можно закрыть, когда будем уверены, что хватает.
