# Модуль: FMP

Рабочие заметки агента. **Канон:** [`FMP.md`](../../knowledge_base/FMP.md).

## Суть (одним абзацем)

Формация = физический материал. Мораль, организованность, усталость — не множители урона, а **yield / stiffness / пластическая деформация**. Контакт формаций = столкновение материалов + волны давления. Производительность: не O(n²) full rigid body, не «фейковая» raycast-физика — **три слоя** с активацией по необходимости.

## Порядок систем за тик (эталон)

1. `update_formation_material_properties`
2. `propagate_pressure_waves`
3. `detect_contact_zones`
4. `activate_contact_zone_physics`
5. `process_individual_events` (удары, сломы)
6. `feed_results_to_formation_field`
7. `update_soldier_positions_from_field`
8. `propagate_panic`

Avian — только шаг между 4 и 6 (слои 2–3).

## Эмерджентность (что должно «само» получиться)

- Клин бьёт точечно по yield
- Фланг — интерференция волн
- Смерть героя — падение yield соседей
- Усталость — накопленная «деформация», тот же удар слабее держится

## Layer 1: анизотропия материала

В лаборатории анизотропия реализована как взвешенное распространение давления по сетке формации:

- `forward_multiplier` — насколько хорошо материал проводит давление вдоль оси формации;
- `lateral_multiplier` — насколько хорошо материал связан поперёк / к флангам.

Это важная граница модели: профиль формации задаёт не только то, как она давит на противника, но и то, как она сама проводит и ломает давление. `Line` сейчас служит почти изотропным контролем; `Wedge` — жёсткий по оси атаки и слабее связан сбоку. Для следующих архетипов (Phalanx, Crowd) эти множители должны быть главным отличием поведения.

## Layer 2: Contact contract перед Avian

Переход к Contact Zone начинается не с физдвижка, а с устойчивой границы:

`ContactRequest -> ContactBoundary -> FormationField`

Текущий смысл:

- `ContactRequest` описывает запрос контакта: front column, число рядов, входящий pressure profile, normal pressure, compression, disruption.
- `ContactBoundary` — результат Layer 2 для Layer 1: pressure samples по front row.
- `FormationField` принимает boundary и дальше распространяет давление уже своими Layer 1 правилами.

Добавлен первый pure contact detection:

- `ContactFront` описывает фронт формации без Bevy/Avian: front column, rows, row spacing, lateral center, front position.
- `ContactDetection` задаёт contact distance и base pressure.
- `detect_contact_request` возвращает `None`, если фронты не сжаты или не перекрываются по флангу; иначе считает compression, normal pressure и активный `ContactRowRange`.

`ContactFront` теперь строится из реальной `Formation` geometry (`origin`, `forward`, rows, slot spacing), а не из условных нулей. Добавлен сценарий `OffsetContact`: Red смещён по флангу, поэтому `ContactRowRange` давит только на перекрытую часть Blue front row; regression test закрепляет flank pressure asymmetry.

Важно: production `simulation.rs` и test-only `experiment.rs` уже идут через этот контракт. Avian позже должен стать источником `ContactBoundary` или более точных `ContactRequest`, а не заменить Layer 1 напрямую.

## Открыто в памяти

- Дискретизация поля: [`OPEN_QUESTIONS.md`](../OPEN_QUESTIONS.md) Q-001
- Гипотезы: HYP-001, HYP-002, HYP-003

## Не путать

| Явление | Механизм |
|---------|----------|
| Бегство | Моральный слом, yield |
| Выбит из строя | Организованность / смещение, может остаться в бою |
