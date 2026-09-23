# Comparación de repositorios de skills

[English](../en/skill-source-review.md)

Una [comparación posterior centrada en Superpowers](superpowers-comparison.md) revisa sus 15 entradas y valida mejoras concretas; las cifras históricas de esta revisión se conservan.

Se contrastaron **66 repositorios** del catálogo actual e histórico. El inventario agrupó **7,216 candidatos** por nombre y repositorio; durante la ampliación se descargaron **536 entradas** y, entre ambas revisiones, se consultaron secciones de **201 rutas únicas**. La biblioteca resultante contiene **142 skills propias**, 92 nuevas en esta ampliación.

Las cifras de inventario no significan lectura íntegra. Se revisaron los README e inventarios de todos los repositorios, y las secciones seleccionadas quedan identificadas por ruta, commit y hash. Los catálogos de enlaces no se recorrieron hasta cada repositorio externo. Agrupar rutas con el mismo nombre tampoco demuestra que su contenido sea idéntico.

Por ejemplo, sickn33 contiene 6.697 rutas SKILL.md, agrupadas en 2.130 nombres candidatos. Fénix consolida variantes con el mismo resultado; conserva diferencias útiles entre lenguajes, migraciones, pagos, ensayos, ciencia y trabajo de oficina. No se ejecutaron scripts ni se instalaron dependencias de estas fuentes.

[Evidencia y decisiones detalladas](../skills/source-review.json) · [Inventario completo](../skills/upstream-inventory.csv) · [Catálogo de Fénix](skill-library.md)

## Comparación por repositorio

“Secciones” cuenta las entradas con secciones consultadas, no archivos leídos íntegramente. “Cobertura” expresa los resultados cubiertos en Fénix, no una integración con ese proveedor.

| Repository | SKILL.md paths | Candidate names | Sections | Fénix coverage |
| --- | ---: | ---: | ---: | --- |
| [obra/superpowers](https://github.com/obra/superpowers/tree/5bf4e78011075bcfc0dc295f0724994cd123ee71) | 15 | 15 | 3 | `code-review`, `skill-authoring`, `test-driven-repair` |
| [mattpocock/skills](https://github.com/mattpocock/skills/tree/c55ee46073ed923f86ce59a5eb3b6d895095d1b7) | 38 | 38 | 4 | `code-review`, `learning-and-tutoring`, `test-driven-repair` |
| [anthropics/skills](https://github.com/anthropics/skills/tree/34040c9c568585f6929bedeaad110ad08f079624) | 20 | 20 | 2 | `professional-writing`, `technical-documentation` |
| [nextlevelbuilder/ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/tree/de5f12b400775997d213524ef02a7c7d2746806f) | 13 | 7 | 2 | `brand-and-design-tokens`, `frontend-design-and-accessibility` |
| [VoltAgent/awesome-design-md](https://github.com/VoltAgent/awesome-design-md/tree/8147538b4226ae41e2487a9179e3bcc1f68e8554) | 0 | 0 | 0 | Sin skill adicional; decisión en el manifiesto |
| [JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman/tree/542442bab314973709f95b85b1ac0b3f6f5b5dc6) | 24 | 20 | 0 | Sin skill adicional; decisión en el manifiesto |
| [nexu-io/open-design](https://github.com/nexu-io/open-design/tree/6fd2f60802232e0fdb29121a6b3e745d2fe9ff42) | 540 | 333 | 0 | Sin skill adicional; decisión en el manifiesto |
| [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills/tree/c004a74784a08295d52749b04cda634125b9a581) | 25 | 25 | 4 | `api-contract-design`, `code-review`, `technical-documentation`, `test-driven-repair` |
| [Leonxlnx/taste-skill](https://github.com/Leonxlnx/taste-skill/tree/e79ca9ec7e071eb3a3b623c4fb752e853fc3ed58) | 13 | 13 | 1 | `frontend-design-and-accessibility` |
| [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2) | 864 | 864 | 4 | `calendar-scheduling`, `cloud-file-sync`, `google-docs`, `google-drive`, `google-sheets`, `google-slides`, `team-messaging` |
| [mvanhorn/last30days-skill](https://github.com/mvanhorn/last30days-skill/tree/beb7ed1868f034f198842174bfe2694e44b78363) | 1 | 1 | 1 | `web-research`, `competitive-research` |
| [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code/tree/e5abfc242bd92341611943eb9053675a0d7e043e) | 0 | 0 | 0 | Sin skill adicional; decisión en el manifiesto |
| [calesthio/OpenMontage](https://github.com/calesthio/OpenMontage/tree/08e2151fa02de28a5d6a312b3d575692bf147ad7) | 138 | 90 | 2 | `video-editing` |
| [VoltAgent/awesome-openclaw-skills](https://github.com/VoltAgent/awesome-openclaw-skills/tree/4367aac4da2a6202b56cf6dfdbab91076bbb8a1d) | 0 | 0 | 0 | Sin skill adicional; decisión en el manifiesto |
| [kepano/obsidian-skills](https://github.com/kepano/obsidian-skills/tree/3ccff5338ea700537839b21900aa5358a0402c98) | 6 | 6 | 1 | `knowledge-base-maintenance` |
| [coreyhaines31/marketingskills](https://github.com/coreyhaines31/marketingskills/tree/5b2c0007766c6a1cf1d53fd8fc73e979e0821022) | 50 | 50 | 4 | `content-calendar`, `marketing-content`, `seo-review` |
| [sickn33/agentic-awesome-skills](https://github.com/sickn33/agentic-awesome-skills/tree/b6299bc7505a40194bc7793d382ab805097a692f) | 6697 | 2130 | 9 | `cache-design`, `data-analysis`, `employee-onboarding`, `incident-response`, `meal-planning`, `network-diagnostics`, `privacy-redaction`, `scheduled-workflows`, `travel-planning` |
| [Imbad0202/academic-research-skills](https://github.com/Imbad0202/academic-research-skills/tree/3c546bc08c56f79e0068f1ea4f0acedf5bf69b5e) | 4 | 4 | 0 | Sin skill adicional; decisión en el manifiesto |
| [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills/tree/330c8e764435a731eff571e3efdda70b363d0792) | 166 | 166 | 19 | `bibliography-management`, `data-analysis`, `geospatial-analysis`, `grant-proposals`, `latex-authoring`, `literature-review`, `mathematical-computation`, `peer-review`, `scientific-figures`, `scientific-writing`, `simulation-modeling`, `time-series-forecasting` |
| [wshobson/agents](https://github.com/wshobson/agents/tree/4236bb91f8395b0435f1d8b8baf9e8e4c69a8620) | 183 | 183 | 22 | `api-contract-design`, `authentication-integration`, `background-jobs`, `data-pipelines`, `database-migrations`, `database-schema-design`, `incident-response`, `mobile-app-testing`, `payment-and-billing`, `shell-automation`, `task-delegation` |
| [blader/humanizer](https://github.com/blader/humanizer/tree/9862685f575c65a8247f90369951df1b3416e3d6) | 1 | 1 | 1 | `professional-writing` |
| [github/awesome-copilot](https://github.com/github/awesome-copilot/tree/4f4796f0bf30e105700f97ed8408c12b6aa95e06) | 436 | 436 | 3 | `browser-navigation`, `frontend-design-and-accessibility`, `meeting-notes`, `technical-documentation` |
| [Yuan1z0825/nature-skills](https://github.com/Yuan1z0825/nature-skills/tree/9cecfef6ac683fa59d7d15d2e22f98fa71dacaf5) | 20 | 20 | 2 | `experiment-and-benchmark-design`, `literature-review` |
| [emilkowalski/skills](https://github.com/emilkowalski/skills/tree/85e8e2363b713506e1d5b6e07a0eb2da66be1bc3) | 13 | 13 | 1 | `frontend-design-and-accessibility` |
| [VoltAgent/awesome-agent-skills](https://github.com/VoltAgent/awesome-agent-skills/tree/755521de9a19bceff41aa96275db03e40a539b36) | 0 | 0 | 0 | Sin skill adicional; decisión en el manifiesto |
| [openai/codex-plugin-cc](https://github.com/openai/codex-plugin-cc/tree/db52e28f4d9ded852ab3942cea316258ae4ef346) | 3 | 3 | 0 | Sin skill adicional; decisión en el manifiesto |
| [alchaincyf/nuwa-skill](https://github.com/alchaincyf/nuwa-skill/tree/fe0374687037c4cc51a65c1e0c145afe2981dc69) | 16 | 1 | 0 | Sin skill adicional; decisión en el manifiesto |
| [mukul975/Anthropic-Cybersecurity-Skills](https://github.com/mukul975/Anthropic-Cybersecurity-Skills/tree/54a798831d2266a3ca61ce68a7acb80b81160d57) | 818 | 818 | 5 | `backup-and-restore`, `phishing-triage`, `threat-modeling` |
| [vercel-labs/agent-skills](https://github.com/vercel-labs/agent-skills/tree/063bee94c3f4df8453406c830b0a7df0f2860278) | 9 | 9 | 2 | `frontend-design-and-accessibility`, `performance-diagnosis` |
| [zhaoxuya520/reverse-skill](https://github.com/zhaoxuya520/reverse-skill/tree/7e2097fd90d25c2f976f6eba26d6c00aa88051df) | 89 | 89 | 0 | Sin skill adicional; decisión en el manifiesto |
| [vercel-labs/skills](https://github.com/vercel-labs/skills/tree/7407f3893ad4dceab546ac002c3ef806e4000c73) | 1 | 1 | 0 | Sin skill adicional; decisión en el manifiesto |
| [jarrodwatts/claude-hud](https://github.com/jarrodwatts/claude-hud/tree/939eb66485832dead1b0a28a954f76f7aa2bdb06) | 0 | 0 | 0 | Sin skill adicional; decisión en el manifiesto |
| [Nutlope/hallmark](https://github.com/Nutlope/hallmark/tree/13ac0ec7e148655948100b6396439e481361d690) | 1 | 1 | 0 | Sin skill adicional; decisión en el manifiesto |
| [virgiliojr94/book-to-skill](https://github.com/virgiliojr94/book-to-skill/tree/526f362552562d88c1a8bbf8012d2cee93f831d5) | 1 | 1 | 0 | Sin skill adicional; decisión en el manifiesto |
| [phuryn/pm-skills](https://github.com/phuryn/pm-skills/tree/8607e3b077817f89bf4a9b623246219734ac3be0) | 69 | 69 | 18 | `ab-test-analysis`, `competitive-research`, `dashboard-design`, `forms-and-surveys`, `meeting-notes`, `pricing-analysis`, `privacy-and-compliance`, `requirements-and-planning`, `roadmap-prioritization`, `user-research` |
| [JimLiu/baoyu-skills](https://github.com/JimLiu/baoyu-skills/tree/1567581c26ec29f4216c6e6835415bf30343b0e3) | 22 | 22 | 3 | `diagram-authoring`, `image-workflows`, `professional-writing`, `translation-and-localization` |
| [openai/skills](https://github.com/openai/skills/tree/49f948faa9258a0c61caceaf225e179651397431) | 44 | 43 | 0 | Sin skill adicional; decisión en el manifiesto |
| [alirezarezvani/claude-skills](https://github.com/alirezarezvani/claude-skills/tree/19392f7a08264ed00486a251f5b2098321771f94) | 846 | 374 | 24 | `analytics-instrumentation`, `container-workflows`, `contract-review`, `crm-workflows`, `customer-support`, `data-quality-audit`, `document-comparison`, `email-workflows`, `feature-rollouts`, `financial-modeling`, `git-worktree-coordination`, `hiring-and-interviews`, `identity-access-review`, `issue-tracker-workflows`, `knowledge-workspace-connectors`, `meeting-notes`, `observability-and-slos`, `personal-organization`, `process-improvement`, `procurement-and-vendors`, `product-discovery`, `project-status-review`, `purchase-comparison`, `rag-evaluation`, `sales-opportunities` |
| [op7418/guizang-ppt-skill](https://github.com/op7418/guizang-ppt-skill/tree/c91369c449d34755d320a8b81d0734000d99d1ab) | 1 | 1 | 0 | `web-presentations`, `documents-presentations` |
| [agentskills/agentskills](https://github.com/agentskills/agentskills/tree/69ef37e9424c0a7ea9dd2293b559e43ec8176379) | 0 | 0 | 0 | `skill-authoring` |
| [tt-a1i/archify](https://github.com/tt-a1i/archify/tree/72c750bb070d95171dbb2244e5b62b1b7da69c12) | 2 | 2 | 0 | Sin skill adicional; decisión en el manifiesto |
| [KKKKhazix/khazix-skills](https://github.com/KKKKhazix/khazix-skills/tree/4f2db09802736ac8130ddf8dd6121435b5a41b55) | 6 | 6 | 1 | `storage-and-archive-work` |
| [liyupi/ai-guide](https://github.com/liyupi/ai-guide/tree/a3822511b469ef2722a327567af53eb63fca99d6) | 0 | 0 | 0 | Sin skill adicional; decisión en el manifiesto |
| [google/skills](https://github.com/google/skills/tree/18152e0d310e4d7047e9c2ec25a37b0d22d6893e) | 151 | 146 | 10 | `cloud-cost-review`, `cloud-deployment`, `google-drive`, `web-research` |
| [muratcankoylan/Agent-Skills-for-Context-Engineering](https://github.com/muratcankoylan/Agent-Skills-for-Context-Engineering/tree/6dbe1a1d868eab51a3bc9011b0f55e2891513e40) | 23 | 19 | 3 | `conversation-recap`, `experiment-and-benchmark-design`, `task-delegation` |
| [microsoft/SkillOpt](https://github.com/microsoft/SkillOpt/tree/79124b37e9a6371e13b753f8bcd7adb1e493ade1) | 5 | 2 | 1 | `experiment-and-benchmark-design`, `skill-authoring` |
| [composio-community/awesome-codex-skills](https://github.com/composio-community/awesome-codex-skills/tree/0930e1373789d2eda449039f7ac154b33031de89) | 880 | 880 | 3 | `invoice-and-expense-records`, `local-file-work`, `meeting-notes` |
| [AgriciDaniel/claude-seo](https://github.com/AgriciDaniel/claude-seo/tree/92795530b4cc92c6bf7a2435b82c15b003e71181) | 33 | 31 | 1 | `seo-review` |
| [NVIDIA/SkillSpector](https://github.com/NVIDIA/SkillSpector/tree/d162d9b343e559be13df8ebba093df3bc9d58c90) | 27 | 1 | 1 | `skill-source-review` |
| [travisvn/awesome-claude-skills](https://github.com/travisvn/awesome-claude-skills/tree/1da55aa810f206d3fe2005e7e3989b15a275d942) | 0 | 0 | 0 | Sin skill adicional; decisión en el manifiesto |
| [yusufkaraaslan/Skill_Seekers](https://github.com/yusufkaraaslan/Skill_Seekers/tree/bbe9381a3038dd83684314e0198680b669002ed5) | 27 | 3 | 1 | `skill-authoring` |
| [greensock/gsap-skills](https://github.com/greensock/gsap-skills/tree/aed9cfd3277740755f6bfc1155c7aa645403b760) | 8 | 8 | 0 | Sin skill adicional; decisión en el manifiesto |
| [earthtojake/text-to-cad](https://github.com/earthtojake/text-to-cad/tree/4ebbd2cc6172e4c5d53560a5948df214192aefac) | 12 | 12 | 2 | Sin skill adicional; decisión en el manifiesto |
| [MiniMax-AI/skills](https://github.com/MiniMax-AI/skills/tree/60aaae52bb2af8162732751a4332f62a5fef518b) | 23 | 23 | 0 | `image-workflows`, `video-editing`, `audio-transcription` |
| [Piebald-AI/claude-code-system-prompts](https://github.com/Piebald-AI/claude-code-system-prompts/tree/d268e417c239db98a309682a3032d349f595ea01) | 0 | 0 | 0 | Sin skill adicional; decisión en el manifiesto |
| [Orchestra-Research/AI-Research-SKILLs](https://github.com/Orchestra-Research/AI-Research-SKILLs/tree/773a52944ba4747a18bd4ae9ade53fff041adcbc) | 98 | 98 | 5 | `audio-transcription`, `experiment-and-benchmark-design`, `structured-llm-output` |
| [nidhinjs/prompt-master](https://github.com/nidhinjs/prompt-master/tree/2bd92518e26bf659e21e3d9ab90573fcf3ddeccb) | 1 | 1 | 0 | Sin skill adicional; decisión en el manifiesto |
| [ConardLi/garden-skills](https://github.com/ConardLi/garden-skills/tree/aaf9a82f5efd73e87cc0998edc398e75bfc35901) | 5 | 5 | 2 | `knowledge-base-maintenance`, `pdf-workflows`, `web-presentations` |
| [Jeffallan/claude-skills](https://github.com/Jeffallan/claude-skills/tree/882ef55e377dbf9a4dbe496bb41ac6ccd0e555cf) | 67 | 67 | 26 | `api-contract-design`, `api-integration`, `architecture-design`, `browser-regression-testing`, `cpp-development`, `dependency-and-secret-audit`, `dotnet-development`, `go-development`, `infrastructure-as-code`, `jvm-development`, `kubernetes-operations`, `legacy-migration`, `load-and-resilience-testing`, `mcp-integration`, `mobile-app-development`, `python-development`, `realtime-protocols`, `rust-development`, `security-code-review`, `test-strategy`, `typescript-development` |
| [huggingface/skills](https://github.com/huggingface/skills/tree/abc20ae526d8b4c0e4dff89f904adce28a4a0eb6) | 26 | 26 | 4 | `data-analysis`, `ml-dataset-preparation`, `ml-inference-deployment`, `ml-training` |
| [numman-ali/openskills](https://github.com/numman-ali/openskills/tree/57d933a4f0d5c8659bd8b285f50fb9554360f0b3) | 1 | 0 | 0 | Sin skill adicional; decisión en el manifiesto |
| [helloianneo/ian-xiaohei-illustrations](https://github.com/helloianneo/ian-xiaohei-illustrations/tree/91b560849e8f883922cc2fa8a358a668caa94105) | 1 | 1 | 0 | Sin skill adicional; decisión en el manifiesto |
| [alchaincyf/zhangxuefeng-skill](https://github.com/alchaincyf/zhangxuefeng-skill/tree/3501b1e679cf595a6af800619af757aba46d7574) | 1 | 1 | 0 | Sin skill adicional; decisión en el manifiesto |
| [BehiSecc/awesome-claude-skills](https://github.com/BehiSecc/awesome-claude-skills/tree/e3d2916d0c4f26ab450006ff2ee759d9770eed1c) | 0 | 0 | 0 | Sin skill adicional; decisión en el manifiesto |
| [diet103/claude-code-infrastructure-showcase](https://github.com/diet103/claude-code-infrastructure-showcase/tree/07f75ce3c301259e857343596e7883b8ce5a9f50) | 9 | 5 | 1 | `skill-authoring` |
| [iOfficeAI/OfficeCLI](https://github.com/iOfficeAI/OfficeCLI/tree/dced0d74ff85b1fef0b777efcdb637c2c4ef8a6e) | 12 | 12 | 3 | `documents-presentations`, `documents-spreadsheets`, `documents-word` |

## Criterios de selección

- Procedimientos originales por resultado, sin copiar cuerpos, scripts, imágenes ni estilos de autores.
- Conexiones de CRM, mensajería, trackers y archivos cloud se documentan por capacidad y requieren herramientas disponibles; no cientos de envoltorios de proveedores.
- Skills específicas cuando cambia la decisión técnica: propiedad en Rust, cancelación en Go, transacciones de base de datos, sincronización multimedia o diseño estadístico.
- Se mantienen fuera instaladores alternativos, personalidades, técnicas ofensivas, instrucciones clínicas y especialidades sin herramientas verificadas.
- No se adoptan porcentajes de cobertura, frameworks o instalaciones automáticas como reglas universales.
- La revisión de fuentes orienta la cobertura; no demuestra que un modelo complete todas estas tareas.

## Búsqueda comunitaria (2026-09-19)

La revisión de `mvanhorn/last30days-skill` orienta la selección de fuentes, fechas y duplicados de `web-research`. Fénix utiliza su rastreador, índice y navegador local; no integra scripts ajenos ni proveedores externos de búsqueda.
