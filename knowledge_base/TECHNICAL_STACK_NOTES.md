Хороший момент для этого вопроса. Но прежде чем рассуждать про стек — проверю актуальные версии, чтобы не советовать устаревшее.Картина актуальная. Bevy 0.18 вышел в марте 2026, Avian 0.5 — это обновление под Bevy 0.18, 0.6 с новыми фичами идёт следом. Стек сейчас стабильнее чем был год назад.

Давай разберу по трём вопросам честно.

---

## Стек — что брать

**Bevy 0.18** — фиксируем и не гонимся за обновлениями во время активной разработки лаборатории. Каждый релиз Bevy ломает что-то, и стоит закладывать 1-3 дня на миграцию каждые 3-4 месяца. Для лаборатории это некритично — мигрируем осознанно после того как FMP докажет себя.

**Avian 0.5+** для физики Слоёв 2 и 3. Ключевое отличие от Rapier: Avian использует компоненты и делает вычисления прямо внутри Bevy ECS, тогда как Rapier держит отдельное представление мира и проецирует его обратно. Для FMP это принципиально — нам нужно динамически добавлять и убирать физические компоненты у бойцов при переходе между слоями.

**leafwing-input-manager** — для управления героем. Практически обязательно для любого проекта сложнее простого прототипа.

**bevy_egui** — для debug UI лаборатории. Будем смотреть на параметры формаций в реальном времени.

**Lightyear** — для мультиплеера, но это не сейчас. Фиксируем в уме.

---

## Стоит ли двигаться к реализации

Да. И именно сейчас — пока FMP свежий и понятный.

Но с чётким scope. Не игра, не прототип с контентом — а **FMP-лаборатория**: минимальная среда где можно поставить два отряда друг против друга и наблюдать как работает материальная физика.

Цель лаборатории — одна: **доказать что FMP даёт нужное поведение вживую**, а не только в Python-симуляции.

---

## ECS-архитектура для лаборатории

Вот как это ложится на Bevy ECS конкретно:

```rust
// --- FORMATION ---
// Одна entity на формацию

#[derive(Component)]
struct FormationMaterial {
    stiffness: f32,      // организованность
    yield_strength: f32, // мораль
    viscosity: f32,      // усталость
}

#[derive(Component)]
struct PressureField {
    values: Vec<f32>,    // давление в каждом слоте
}

#[derive(Component)]
struct FormationMembers {
    soldiers: Vec<Entity>,
}

// --- SOLDIER ---
// Одна entity на бойца

#[derive(Component)]
struct SoldierStats {
    morale: f32,
    organization: f32,
    fatigue: f32,
    health: f32,
}

#[derive(Component)]
enum PhysicsLayer {
    Field,    // Layer 1 — точка выборки поля
    Contact,  // Layer 2 — полная физика
    Individual, // Layer 3 — событийная физика
}

#[derive(Component)]
struct SoldierSlot {
    formation: Entity,
    slot_index: usize,
}
```

**Ключевой момент — переход между слоями:**

```rust
// Система активации Layer 2
fn activate_contact_zone(
    mut commands: Commands,
    soldiers: Query<(Entity, &PhysicsLayer, &SoldierSlot)>,
    contact_zones: Res<ContactZones>,
) {
    for (entity, layer, slot) in &soldiers {
        if contact_zones.contains(slot) {
            if matches!(layer, PhysicsLayer::Field) {
                // Добавляем Avian компоненты — боец становится физическим телом
                commands.entity(entity).insert((
                    RigidBody::Dynamic,
                    Collider::capsule(0.3, 0.8),
                    PhysicsLayer::Contact,
                ));
            }
        } else if matches!(layer, PhysicsLayer::Contact) {
            // Убираем физику — возвращаем в поле
            commands.entity(entity).remove::<(RigidBody, Collider)>()
                .insert(PhysicsLayer::Field);
        }
    }
}
```

**Порядок систем в одном тике:**

```
1. update_formation_material      // мораль/орг → stiffness/yield_strength
2. propagate_pressure_waves       // волны внутри формаций (Layer 1)
3. detect_contact_zones           // где формации соприкасаются
4. activate_contact_zone_physics  // Layer 1 → Layer 2 где нужно
5. [Avian physics step]           // физика для Layer 2 и 3
6. feed_contact_results_to_field  // результаты Layer 2 → давление в Layer 1
7. update_field_soldier_positions // позиции Layer 1 бойцов из поля
8. propagate_panic                // паника через соседей
```

---

## Главный вопрос перед стартом

Прежде чем открывать редактор — нужно решить **2D или 3D**.

Для лаборатории 2D проще и быстрее. Но финальная игра с рельефом, высотами и формациями — это 3D. Миграция с avian2d на avian3d в Bevy нетривиальна.

Поэтому берем **сразу 3D, вид сверху**. Лаборатория выглядит как 2D, но под капотом уже правильный стек. Никакой миграции потом.
