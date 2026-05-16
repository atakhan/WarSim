# Гипотезы

Проверяемые идеи. Не канон, пока `Status` не `confirmed`. Формат: [`WORKFLOWS.md`](WORKFLOWS.md).

---

## HYP-001 — FMP в Bevy-лаборатории даст нужное тактическое поведение

Status: testing  
Claim: трёхслойная FMP-симуляция в лаборатории (два отряда) воспроизведёт эмерджентные эффекты из канона: клин, фланг, паника, слом от давления.  
Why it matters: без этого ядро проекта не доказано в runtime.  
Test: минимальная сцена — две формации, параметры материала, визуализация волн/слома; сценарии «линия vs линия», «клин», «фланг», «низкая мораль», «усталость».  
Evidence: концепт и порядок систем описаны в `FMP.md`; создана Bevy 0.18 Layer 1 лаборатория с двумя формациями, полем давления, сломом, debug UI, метриками pressure/yield/fracture и сценариями `Line vs line`, `Wedge vs line`, `Flank pressure`, `Low morale defense`, `Fatigued defense`. Пользователь оценил первую Layer 1 версию как приемлемую. После этого добавлена физическая анизотропия материала: `forward_multiplier` / `lateral_multiplier` и взвешенный Лапласиан давления.  
Next step: оценить читаемость анизотропии в `Wedge vs line` и `Flank pressure`; затем добавить контрастные архетипы `Phalanx` и `Crowd` для проверки, что формации отличаются поведением, а не названием.  
Links: [`FMP.md`](../knowledge_base/FMP.md), [`modules/fmp.md`](modules/fmp.md)

---

## HYP-002 — Динамическое включение Avian-тел на границе контакта масштабируется

Status: open  
Claim: активация/деактивация `RigidBody`+`Collider` у бойцов при входе/выходе contact zone даст приемлемый FPS при сотнях бойцов в поле и узкой зоне контакта.  
Why it matters: слой 2 — узкое горлышко производительности и сложности ECS.  
Test: профилирование сцены с N бойцами, M в contact zone; измерить стоимость insert/remove и шага Avian.  
Evidence: архитектурно заложено в FMP и stack notes; не измерено.  
Next step: после слоя 1 — прототип contact zone с 20–50 активными телами.  
Links: [`FMP.md`](../knowledge_base/FMP.md) §3.2, [`modules/bevy_architecture.md`](modules/bevy_architecture.md)

---

## HYP-003 — GIC через shapecast даёт «физичные» удары без полного ragdoll

Status: open  
Claim: импульс из геометрии удара (depth/angle factors) достаточен для читаемого боя без постоянного ragdoll всех бойцов.  
Why it matters: слой 3 только на событиях — иначе O(n) взорвётся.  
Test: одиночные удары героя/бойца, переход цели в layer 3 на время реакции.  
Evidence: описано в FMP §4; не реализовано.  
Next step: после contact zone — изолированный тест GIC на одном боеце.  
Links: [`FMP.md`](../knowledge_base/FMP.md) §4
