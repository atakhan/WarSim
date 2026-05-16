# Проектная память ИИ (Agent Mind)

Краткая точка входа для агента. Полная структура — в папке [`agent_mind/`](agent_mind/).

В Cursor это правило подхватывается автоматически: [`.cursor/rules/agent-mind.mdc`](.cursor/rules/agent-mind.mdc).

## Что это

**`agent_mind/`** — рабочая память помощника: как думать о проекте, что уже решено, что проверяется, какие уроки извлечены.

**`knowledge_base/`** — канон проекта (дизайн, FMP, аксиомы). При конфликте канон важнее памяти агента.

## Перед серьёзной задачей

1. Прочитать [`agent_mind/README.md`](agent_mind/README.md) — индекс и порядок чтения.
2. Открыть только релевантные файлы из `agent_mind/` (см. `CONTEXT_MAP.md`).
3. Свериться с нужными документами в `knowledge_base/`.

## После задачи (если было что зафиксировать)

| Ситуация | Куда писать |
|----------|-------------|
| Принято решение с последствиями | [`agent_mind/DECISIONS.md`](agent_mind/DECISIONS.md) |
| Новая проверяемая идея | [`agent_mind/HYPOTHESES.md`](agent_mind/HYPOTHESES.md) |
| Вопрос отложен | [`agent_mind/OPEN_QUESTIONS.md`](agent_mind/OPEN_QUESTIONS.md) |
| Урок после эксперимента / ошибки | [`agent_mind/LESSONS_LEARNED.md`](agent_mind/LESSONS_LEARNED.md) |
| Сменилась общая картина проекта | [`agent_mind/PROJECT_MODEL.md`](agent_mind/PROJECT_MODEL.md) |
| Гипотеза стала частью проекта | Кратко в `knowledge_base/` + статус в `HYPOTHESES.md` |

Подробный протокол: [`agent_mind/OPERATING_PROTOCOL.md`](agent_mind/OPERATING_PROTOCOL.md).  
Форматы записей и процедуры: [`agent_mind/WORKFLOWS.md`](agent_mind/WORKFLOWS.md).

## Правила в двух строках

- Память короткая и операционная; канон не дублировать — ссылаться.
- Старое не удалять молча: `superseded` или `archive/`.
