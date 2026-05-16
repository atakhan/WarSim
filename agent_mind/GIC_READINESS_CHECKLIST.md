# Чеклист готовности к Layer 3 (GIC)

Перед первым **Geometric Impulse Contact** в лаборатории. Канон: [`FMP.md`](../knowledge_base/FMP.md) §3.3, [`PROJECT_CONSTITUTION.md`](../knowledge_base/PROJECT_CONSTITUTION.md) §4.6, §12.

Использовать как gate: **все пункты «обязательно» — да**, иначе GIC откладываем.

---

## 1. Конституция

| # | Критерий | Обязательно |
|---|----------|-------------|
| C1 | Удар **не** задаёт исход через анимацию или таймер «окна урона» | да |
| C2 | Герой **не** доминирует над армией: импульс GIC — событие в поле/контакте, не отдельная MOBA-шкала урона | да |
| C3 | Игрок (в будущем) может понять причину: откуда импульс, куда пошла волна | да |
| C4 | Симуляция не разрушает управляемость: один прототипный удар, не хаос тел | да |

---

## 2. Аксиомы

| # | Критерий | Обязательно |
|---|----------|-------------|
| A1 | Боец как сущность: хотя бы один слот/герой с отдельным событием, не только поле | да |
| A2 | Организованность и мораль уже влияют на исход контакта (organization → yield) | да (lab) |
| A3 | Поведение читается визуально или через UI без расшифровки цифр | желательно |

---

## 3. FMP — Layer 2 достаточен

См. решение в [`DECISIONS.md`](DECISIONS.md) «Layer 2 gate для Layer 3».

| # | Критерий | Статус (2026-05-16) |
|---|----------|---------------------|
| L2-1 | Production path: `ContactRequest → ContactBoundary → FormationField` | выполнено |
| L2-2 | Avian: `row_range` / disruption от colliders, не только geometry | v1 + v2 penetration |
| L2-3 | `compression` учитывает penetration (не только gap) | v2 |
| L2-4 | `normal_pressure` масштабируется скоростью/импульсом контакта (`impact_scale`) | v2 |
| L2-5 | Regression harness стабилен (synthetic); Avian не ломает контракт | 35+ тестов |
| L2-6 | Сценарий Offset + approach читаем вживую | Guided demo |

**Не требуется до GIC:** dynamic insert/remove всех тел, полный solver на всём строе, ландшафт.

---

## 4. Архитектура GIC (первый шаг)

| # | Правило |
|---|---------|
| G1 | GIC → **граничное условие** или локальный импульс в Layer 2/1, **не** прямой `field.pressure[i] += damage` в обход boundary |
| G2 | Один shapecast / один архетип удара (например thrust) |
| G3 | Герой — маркер + источник запроса, не отдельная боевая петля |
| G4 | После удара — существующий `propagate_pressure_wave`, без нового типа «урона» |

---

## 5. Проверка перед merge GIC (v0 выполнено)

- [x] Новый тест: импульс увеличивает peak pressure относительно baseline (`gic_thrust_raises_blue_center_pressure`)
- [x] `DECISIONS.md` запись о форме GIC v0
- [x] `ContactRequest` не расширялся; `BoundaryContactInput.gic_impulse`
- [x] Прогон: `cargo test`, `cargo clippy -- -D warnings`

---

## Связанные файлы

- [`modules/avian_contact.md`](modules/avian_contact.md)
- [`modules/fmp.md`](modules/fmp.md)
- [`REFLECTION_2026-05-16.md`](REFLECTION_2026-05-16.md)
