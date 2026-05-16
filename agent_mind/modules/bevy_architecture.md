# Модуль: Bevy / ECS

Рабочие заметки. **Канон по стеку:** [`TECHNICAL_STACK_NOTES.md`](../../knowledge_base/TECHNICAL_STACK_NOTES.md). **FMP ECS:** [`FMP.md`](../../knowledge_base/FMP.md) §7.

## Стек (зафиксировано)

| Компонент | Выбор |
|-----------|--------|
| Движок | Bevy **0.18** |
| Физика L2–L3 | **Avian** |
| Физика L1 | Кастом, без внешнего движка |
| Ввод | leafwing-input-manager |
| Debug UI | bevy_egui |
| Сеть | Lightyear — позже |
| Пространство | **3D**, камера сверху |

## Ключевые компоненты (скетч)

- **Formation:** `FormationField`, `FormationState` (pressure, fractures), `FormationMembers`
- **Soldier:** `SoldierState`, `PhysicsLayer` (Field | Contact | Individual), slot в формации
- **Hero:** всегда `PhysicsLayer::Individual`, `HeroPresence` (радиусы морали/команд)

## Паттерн смены слоя

При входе в contact zone: `insert(RigidBody, Collider)` + `PhysicsLayer::Contact`.  
При выходе: `remove` физики + `PhysicsLayer::Field`.  
Критично для HYP-002 — профилировать churn компонентов.

## Лаборатория vs игра

Сейчас только лаборатория: два отряда, egui-параметры, без мира/баз/сети.

Первый scaffold создан в корне проекта: Bevy 0.18 + `bevy_egui`, кастомный Layer 1 без Avian.

Текущая структура кода:

- `src/main.rs` — сборка Bevy app и регистрация систем;
- `src/lab/model.rs` — ECS-компоненты и доменная модель формаций;
- `src/lab/scenario.rs` — сценарии и пресеты материалов;
- `src/lab/setup.rs` — spawn камеры, света, земли и стартовых формаций;
- `src/lab/simulation.rs` — расчёт FMP Layer 1;
- `src/lab/ui.rs` — `bevy_egui` окна лаборатории;
- `src/lab/visuals.rs` — отображение давления/fracture на кубах.

Следующая архитектурная граница — перед добавлением Avian contact zone отделить лабораторные сценарии от будущего runtime-слоя боя, если появится дублирование.

## Решения

См. [`DECISIONS.md`](../DECISIONS.md): 3D top-down, Bevy+Avian, scope лаборатории.
