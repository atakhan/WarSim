# Карта контекста проекта

Куда смотреть за информацией. **Канон** — в `knowledge_base/`; **память агента** — в `agent_mind/`.

## knowledge_base (канон)

| Документ | Содержание | Когда читать |
|----------|------------|--------------|
| [`PROJECT_CONSTITUTION.md`](../knowledge_base/PROJECT_CONSTITUTION.md) | Манифест игры: сущность, бой, кооп, MVP, анти-принципы | Дизайн фич, приоритеты, «можно ли так» |
| [`PROJECT_AXIOMS.md`](../knowledge_base/PROJECT_AXIOMS.md) | Аксиомы мира и армии | Поведение армии, мораль, формации, управляемость |
| [`FMP.md`](../knowledge_base/FMP.md) | Formation Material Physics — движок боя | Симуляция, слои, GIC, ECS-порядок систем |
| [`TECHNICAL_STACK_NOTES.md`](../knowledge_base/TECHNICAL_STACK_NOTES.md) | Bevy, Avian, лаборатория, стек | Реализация, выбор технологий |

## agent_mind (оперативная память)

| Документ | Содержание |
|----------|------------|
| [`PROJECT_MODEL.md`](PROJECT_MODEL.md) | Сжатая текущая модель проекта |
| [`DECISIONS.md`](DECISIONS.md) | Принятые решения |
| [`HYPOTHESES.md`](HYPOTHESES.md) | Идеи на проверке |
| [`OPEN_QUESTIONS.md`](OPEN_QUESTIONS.md) | Открытые вопросы |
| [`GLOSSARY.md`](GLOSSARY.md) | Термины |
| [`LESSONS_LEARNED.md`](LESSONS_LEARNED.md) | Уроки |
| [`modules/fmp.md`](modules/fmp.md) | FMP — рабочие заметки агента |
| [`modules/bevy_architecture.md`](modules/bevy_architecture.md) | Bevy / ECS — рабочие заметки |
| [`modules/combat_simulation.md`](modules/combat_simulation.md) | Бой и лаборатория |
| [`modules/project_docs.md`](modules/project_docs.md) | Связь канона и памяти |

## Быстрые маршруты по темам

**«Можно ли добавить механику X?»**  
→ `PROJECT_CONSTITUTION.md` (анти-принципы) → `PROJECT_AXIOMS.md` → `DECISIONS.md`

**«Как симулировать столкновение формаций?»**  
→ `FMP.md` → `modules/fmp.md` → `HYPOTHESES.md`

**«С чего начать код?»**  
→ `PROJECT_MODEL.md` → `TECHNICAL_STACK_NOTES.md` → `modules/bevy_architecture.md` → `DECISIONS.md`

**«Что уже решили по стеку?»**  
→ `DECISIONS.md` → `TECHNICAL_STACK_NOTES.md`

## Корень репозитория

| Файл | Назначение |
|------|------------|
| [`README.md`](../README.md) | Название проекта (пока минимально) |
| [`AGENT_MIND.md`](../AGENT_MIND.md) | Точка входа для агента |
