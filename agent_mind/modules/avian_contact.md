# Avian → ContactRequest (дизайн, не код)

Рабочая заметка перед подключением физдвижка. **Канон:** [`FMP.md`](../../knowledge_base/FMP.md) §3.2, §7.3.

## Роль Avian

Avian не заменяет Formation Field. Он поставляет **измеримые граничные условия** для Layer 2:

- геометрия фронта (`ContactFront` / позиции тел переднего ряда),
- импульс и сжатие на контакте,
- частичное перекрытие → `disruption`.

Layer 1 по-прежнему: `ContactBoundary → FormationField`.

## Поток данных (целевой)

```
Avian colliders (front row only)
    → ContactProbe (позиции, overlap width, relative velocity)
    → ContactRequest (pressure, compression, disruption, row_range)
    → ContactBoundary::resolve()
    → FormationField (pressure + organization loss)
```

Лаборатория: `ContactProbeKind` + `detect_with_probe`; **Synthetic** — geometry; **Avian** — `AvianContactCache` после `collect_avian_contacts` (`Collisions` + `CollidingEntities`), fallback в Synthetic если кэш пуст. **v2:** `compression = max(gap, penetration)`, `impact_scale` на `normal_pressure`. UI: overlap rows, disruption, penetration, gap/pen compression, impact scale. Gate Layer 3: [`GIC_READINESS_CHECKLIST.md`](../GIC_READINESS_CHECKLIST.md).

## Маппинг полей

| ContactRequest | Источник в Avian (идея) |
|----------------|-------------------------|
| `front_column`, `rows` | индексация слотов переднего ряда, без изменений |
| `row_range` | проекция overlap AABB фронтов на ось фланга |
| `normal_pressure` | сумма импульсов нормали × площадь контакта / dt, capped |
| `compression` | `compression_from_gap(front_gap, contact_distance)` или пружина контакта |
| `disruption` | `1 - overlap_ratio` (+ опционально боковой зазор) |
| `incoming_profile` | профиль атакующей формации (не из Avian) |

## Активация тел

- По умолчанию бойцы — точки поля (Layer 1).
- При `compression > 0` на слоте переднего ряда — insert kinematic/dynamic body + collider.
- При разрыве контакта / отходе — remove body, оставить след в `organization` и `fractured`.

## Не делать на первом интеграционном шаге

- Полный O(n) rigid body для всех бойцов.
- Прямой импорт давления в поле в обход `ContactBoundary`.
- Смешивание Layer 3 (GIC) с первым контактным прототипом.

## Критерий готовности к Avian

1. Сценарии Wedge / Offset / Phalanx читаемы с disruption и approach.
2. Regression harness стабилен.
3. `ContactRequest` не меняется без записи в `DECISIONS.md`.

## Реализация (2026-05-16)

- `avian3d` 0.5, `PhysicsPlugins` в `main`, `collect_avian_contacts` в `PhysicsSchedule` после `PhysicsStepSystems::Last`.
- Colliders только на `formation.front_column()`; слои `RED_LAYER` / `BLUE_LAYER`.
- `row_range` — min/max row из пар colliding front slots; `disruption` — `1 - overlap_ratio`.
- **v2:** `ContactPair::find_deepest_contact` / `normal_speed` / impulse; геометрический `geometric_penetration(red_x, blue_x)`; `merge_contact_compression`.
- **GIC:** `SpatialQuery::shape_hits` (swept cuboid от героя, фильтр `BLUE_CONTACT_LAYER`) → `GicImpulse`; геометрический cast только в `#[cfg(test)]`.
- `GuidedDemo` — фиксированный пресет, `lock_tuning`, без слайдеров.
- Production path: `simulation.rs` передаёт кэш при `contact_probe == Avian`.
