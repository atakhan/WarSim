# Журнал решений

Принятые решения с последствиями. Новые записи — **сверху**. Формат: [`WORKFLOWS.md`](WORKFLOWS.md).

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
