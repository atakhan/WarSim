# Agent Mind — индекс

Рабочая память ИИ для проекта WarSim. Канон игры и движка — в [`knowledge_base/`](../knowledge_base/). Точка входа в корне: [`AGENT_MIND.md`](../AGENT_MIND.md).

## Порядок чтения (типовая задача)

1. [`PROJECT_MODEL.md`](PROJECT_MODEL.md) — текущая картина проекта (30 сек).
2. [`CONTEXT_MAP.md`](CONTEXT_MAP.md) — куда идти за деталями.
3. Релевантный журнал: `DECISIONS`, `HYPOTHESES`, `OPEN_QUESTIONS`, `LESSONS_LEARNED`.
4. Модуль в [`modules/`](modules/), если тема узкая (FMP, Bevy, бой).
5. Документы в `knowledge_base/` по ссылкам из карты.

## Файлы

| Файл | Назначение |
|------|------------|
| [`OPERATING_PROTOCOL.md`](OPERATING_PROTOCOL.md) | Как агент работает с памятью |
| [`PROJECT_MODEL.md`](PROJECT_MODEL.md) | Компактная модель проекта сейчас |
| [`DECISIONS.md`](DECISIONS.md) | Журнал решений |
| [`HYPOTHESES.md`](HYPOTHESES.md) | Проверяемые идеи |
| [`OPEN_QUESTIONS.md`](OPEN_QUESTIONS.md) | Отложенные вопросы |
| [`CONTEXT_MAP.md`](CONTEXT_MAP.md) | Карта документов проекта |
| [`WORKFLOWS.md`](WORKFLOWS.md) | Процедуры и форматы записей |
| [`GLOSSARY.md`](GLOSSARY.md) | Термины |
| [`LESSONS_LEARNED.md`](LESSONS_LEARNED.md) | Уроки после экспериментов |
| [`modules/`](modules/) | Память по подсистемам |
| [`archive/`](archive/) | Устаревшее (не читать по умолчанию) |

## Цикл мышления

```text
контекст → гипотеза → решение → эксперимент → урок → обновление модели
```
