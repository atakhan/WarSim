# Журнал решений

Принятые решения с последствиями. Новые записи — **сверху**. Формат: [`WORKFLOWS.md`](WORKFLOWS.md).

---

## 2026-05-16 — GIC thrust через Avian shapecast

Status: accepted  
Context: GIC v0 использовал геометрический луч по позициям слотов; канон FMP §4 — shapecast.  
Decision: Runtime: `SpatialQuery::shape_hits` с swept cuboid от героя, фильтр `BLUE_CONTACT_LAYER`, маппинг `Entity` → `SoldierSlot` + `ContactZoneCollider`. Импульс из `hit.distance` и `normal1`. Геометрический путь остаётся в `#[cfg(test)]` для harness.  
Rationale: Colliders переднего ряда уже в Avian; единый физический стек для contact probe и GIC.  
Consequences: UI показывает `avian shapecast` vs `geometry`; thrust при промахе shapecast не даёт fallback в runtime.  
Links: `src/lab/gic.rs`, `src/lab/simulation.rs`

---

## 2026-05-16 — GIC v0: thrust героя через ContactBoundary

Status: accepted  
Context: Первый Layer 3 в lab по [`GIC_READINESS_CHECKLIST.md`](GIC_READINESS_CHECKLIST.md).  
Decision: `GicThrustParams` + геометрический cast по переднему ряду защитника → `GicImpulse` (pressure_boost). Импульс **сливается** в `ContactBoundary` (`merge_into_boundary`), затем `apply_to_field` + `propagate_pressure_wave`. Герой — `FormationHero` на центральном слоте красного фронта; авто-thrust в Guided demo (`gic_enabled`). Контракт `ContactRequest` не менялся; в `BoundaryContactInput` добавлено `gic_impulse: Option<GicImpulse>`.  
Rationale: G1–G4 чеклиста: не прямой урон, один архетип thrust, герой как источник запроса.  
Consequences: Полноценный Avian shapecast — следующий шаг; regression `gic_thrust_raises_blue_center_pressure`.  
Links: `src/lab/gic.rs`, [`GIC_READINESS_CHECKLIST.md`](GIC_READINESS_CHECKLIST.md)

---

## 2026-05-16 — Layer 2 gate для старта GIC (Layer 3)

Status: accepted  
Context: Нужен явный критерий «достаточно Contact Zone», чтобы не начать GIC поверх заглушки или обхода boundary.  
Decision: Переход к первому GIC в lab разрешён, когда выполнены пункты L2-1…L2-6 в [`GIC_READINESS_CHECKLIST.md`](GIC_READINESS_CHECKLIST.md): контракт в production, Avian даёт row_range + penetration-based compression + impact_scale, harness стабилен, Guided demo читаем. GIC v0 — один импульс через границу, без прямой записи в поле в обход `ContactBoundary`; герой не доминирует над армией.  
Rationale: Конституция §4.6 и §12; FMP §3.2–3.3.  
Consequences: Новый код Layer 3 проверяется по чеклисту; расширение `ContactRequest` только через DECISIONS.  
Links: [`GIC_READINESS_CHECKLIST.md`](GIC_READINESS_CHECKLIST.md), [`REFLECTION_2026-05-16.md`](REFLECTION_2026-05-16.md)

---

## 2026-05-16 — Avian v2: penetration + impact_scale

Status: accepted  
Context: v1 брал `compression` только из gap; Avian почти не отличался от synthetic по силе удара.  
Decision: `collect_avian_contacts` использует `Collisions` (deepest `penetration`, `normal_speed`, impulse/dt) и геометрический fallback; `compression = max(gap, penetration)`; `normal_pressure` × `impact_scale` от скорости сближения. UI показывает gap/pen/compression и impact scale.  
Rationale: Шаг к FMP §3.2 без смены `ContactRequest`.  
Consequences: Synthetic probe по-прежнему gap-only; сравнение synthetic vs avian осмысленно по силе и penetration.  
Links: `src/lab/avian.rs`, [`modules/avian_contact.md`](modules/avian_contact.md)

---

## 2026-05-16 — Guided demo без слайдеров

Status: accepted  
Context: Аксиома 16: читаемость для проверки MVP без перегруза подстройкой.  
Decision: `LabScenario::GuidedDemo` (Offset + approach + Avian preset), `LabSettings::lock_tuning`, чеклист «Смотри на сцене» в UI.  
Rationale: Отделить демонстрацию поведения от калибровки разработчика.  
Consequences: Смена сценария с Guided снимает lock; калибровка — через остальные сценарии.  
Links: `src/lab/scenario.rs`, `src/lab/settings.rs`, `src/lab/ui.rs`

---

## 2026-05-16 — Avian для row_range, geometry для compression (v1)

Status: accepted  
Context: Нужен реальный Layer 2 probe на colliders без переписывания `ContactBoundary` и без rigid body на всех слотах.  
Decision: `avian3d` 0.5: kinematic cuboid на каждом слоте переднего ряда; `collect_avian_contacts` пишет `AvianContactCache` → `detect_with_probe(Avian)`; `row_range` и `disruption` из пересечений collider, `compression` пока из `front_gap` как у synthetic. Пустой кэш — fallback в Synthetic.  
Rationale: Минимальная интеграция проверяет offset/partial overlap через физику перекрытия, контракт `ContactRequest` не меняется.  
Consequences: Synthetic и Avian могут расходиться на compression до manifolds; тесты harness остаются на Synthetic.  
Links: `src/lab/avian.rs`, [`modules/avian_contact.md`](modules/avian_contact.md)

---

## 2026-05-16 — Formation approach без Avian

Status: accepted  
Context: Compression в Contact Zone должен расти от реального зазора между фронтами, а не только от слайдера `contact_distance`.  
Decision: Добавлен lab-режим `FormationMotion::Approach`: обе формации сближаются с `approach_speed`, остановка когда `front_gap <= speed * dt`. UI показывает live `front gap` и `compression`; regression test сравнивает peak pressure со static.  
Rationale: Движение строя — минимальный шаг к динамическому контакту до Avian; detection уже использует `ContactFront.front_position` из geometry.  
Consequences: Approach в harness и runtime; Avian позже может заменить источник позиций, но не должен обходить `detect_contact_request`.  
Links: [`modules/fmp.md`](modules/fmp.md), `src/lab/motion.rs`

---

## 2026-05-16 — Layer 2 начинается с чистого contact contract

Status: accepted  
Context: После стабилизации Layer 1 возник переход к Contact Zone. Подключать Avian сразу рискованно: можно смешать физдвижок, геометрию контакта и обратную связь в Formation Field в один неотделимый слой.  
Decision: Layer 2 сначала фиксируется как чистый контракт `ContactRequest -> ContactBoundary -> FormationField`, без Avian. Production simulation и test-only scenario harness должны идти через этот контракт.  
Rationale: Контракт даёт тестируемую границу между Contact Zone и Formation Field: pressure samples по front row, compression/disruption как будущие данные контакта, и отдельное применение boundary к полю. Это позволяет позже заменить источник `ContactBoundary` на Avian/реальные тела без переписывания Layer 1.  
Consequences: До подключения Avian новые работы по Layer 2 должны сохранять этот интерфейс или явно пересматривать решение. Synthetic boundary pressure не должен обходить `ContactBoundary` в production path.  
Links: [`modules/fmp.md`](modules/fmp.md), [`FMP.md`](../knowledge_base/FMP.md) §3.2, §7.2

---

## 2026-05-16 — Анизотропия через взвешенный Лапласиан в Слое 1

Status: accepted  
Context: Разные формации (линия, клин) должны по-разному передавать давление (Аксиома 6, FMP §2), но изначальный Лапласиан был изотропным.  
Decision: Добавлены `forward_multiplier` и `lateral_multiplier` в `FormationMaterial`. Волна давления рассчитывается через взвешенный Лапласиан.  
Rationale: Это позволяет без усложнения геометрии сделать "Клин" прочным по оси атаки и хрупким с флангов, а "Линию" - равномерной. Полностью соответствует канону FMP.  
Consequences: Любая новая формация теперь требует настройки этих двух множителей для определения её физического поведения.  
Links: [`modules/combat_simulation.md`](modules/combat_simulation.md), [`FMP.md`](../knowledge_base/FMP.md)

---

## 2026-05-16 — FMP-лаборатория вместо полной игры как первый код

Status: accepted  
Context: проект в фазе доказательства боевой модели; канон требует системный бой до масштаба контента.  
Decision: первый runnable — **FMP-лаборатория** (два отряда, наблюдение симуляции), не игра с контентом.  
Rationale: цель — доказать FMP вживую, как в Python-симуляции; минимальный scope снижает риск расползания.  
Consequences: приоритет кода — слои FMP, debug UI, сценарии столкновения; мир/базы/кооп — позже.  
Links: [`TECHNICAL_STACK_NOTES.md`](../knowledge_base/TECHNICAL_STACK_NOTES.md), [`FMP.md`](../knowledge_base/FMP.md), [`PROJECT_CONSTITUTION.md`](../knowledge_base/PROJECT_CONSTITUTION.md) §11

---

## 2026-05-16 — 3D top-down с первого прототипа

Status: accepted  
Context: выбор 2D vs 3D для лаборатории; финальная игра с рельефом — 3D.  
Decision: **сразу 3D, камера сверху**; лаборатория визуально «как 2D», под капотом avian3d.  
Rationale: миграция avian2d→3d нетривиальна; рельеф и высоты — в финальном видении.  
Consequences: ECS, коллайдеры, камера — 3D с начала; не закладывать 2D-only API.  
Links: [`TECHNICAL_STACK_NOTES.md`](../knowledge_base/TECHNICAL_STACK_NOTES.md)

---

## 2026-05-16 — Bevy 0.18 + Avian, слой 1 без внешнего движка

Status: accepted  
Context: нужна ECS-нативная физика с динамической активацией тел по слоям FMP.  
Decision: **Bevy 0.18**; **Avian** для слоёв 2–3; **слой 1 — кастомная система**; не гнаться за апдейтами Bevy во время лаборатории.  
Rationale: Avian в ECS vs отдельный мир Rapier; FMP требует insert/remove физики у бойцов при смене слоя.  
Consequences: зафиксировать версии в `Cargo.toml` при старте кода; закладывать 1–3 дня на миграцию при осознанном апгрейде.  
Links: [`FMP.md`](../knowledge_base/FMP.md) §7.3, [`TECHNICAL_STACK_NOTES.md`](../knowledge_base/TECHNICAL_STACK_NOTES.md)

---

## 2026-05-16 — Разделение knowledge_base и agent_mind

Status: accepted  
Context: нужна эволюционирующая память агента без размывания канона дизайна.  
Decision: **`knowledge_base/`** — канон; **`agent_mind/`** — рабочая память ИИ; точка входа **`AGENT_MIND.md`**.  
Rationale: разная скорость изменений и разная аудитория (люди vs операционный контекст агента).  
Consequences: при конфликте приоритет у канона; гипотезы не писать в канон без подтверждения.  
Links: [`AGENT_MIND.md`](../AGENT_MIND.md), [`OPERATING_PROTOCOL.md`](OPERATING_PROTOCOL.md)
