# Test ROMs Output Methods
Source: https://github.com/retrio/gb-test-roms
## Résumé
| Test | Serial (SB/$81) | Mémoire ($A000+) | CGB requis | Fin de test ||------|:---:|:---:|:---:|---|| `cpu_instrs` | ✅ | ✅ | Non | Serial "Passed" ou self-loop || `instr_timing` | ✅ | ✅ | Non | Serial "Passed" ou self-loop || `mem_timing` | ✅ | ✅ | Non | Serial "Passed" ou self-loop || `mem_timing-2` | ❌ | ✅ | Non | Self-loop (JP $) || `interrupt_time` | ✅ (probable) | ✅ | **Oui** | Self-loop (JP $) || `dmg_sound` | ❌ | ✅ | Non | Self-loop (JP $) || `cgb_sound` | ❌ | ✅ | **Oui** | Self-loop (JP $) || `oam_bug` | ❌ | ✅ | Non | Self-loop (JP $) || `halt_bug.gb` | ❌ | ✅ | Non | Self-loop (JR $) |
## Vérifié empiriquement sur CrabbyBoy
- `cpu_instrs` (combiné + individuels) : **serial fonctionne** ✅- `instr_timing` : **serial fonctionne** ✅- `mem_timing` (v1) : **serial fonctionne** ✅- `halt_bug.gb` : **pas de serial** ❌
## Détails
### Tests avec serial (ancien framework shell.inc)
`cpu_instrs`, `instr_timing`, `mem_timing` utilisent l'ancien `shell.inc` :1. Envoient chaque caractère sur le port série : SB = char, SC = $812. Affichent le texte à l'écran (LCD)3. Écrivent les résultats en mémoire à `$A000` avec signature `$DE, $B0, $61`4. Terminent par un self-loop (`JP $`)
Notre émulateur détecte "Passed"/"Failed" dans la sortie serial et termine immédiatement.
### Tests sans serial (nouveau framework)
`mem_timing-2`, `dmg_sound`, `oam_bug` utilisent un framework plus récent :- **Pas de sortie serial**- Résultats uniquement en mémoire ($A000+ avec signature DE B0 61)
### `halt_bug.gb` — Cas particulier
- **PAS de serial** — n'écrit jamais `$81` dans SC- Résultats uniquement en mémoire ($A000+ avec signature DE B0 61)- Status `$A000` = `0x00` pour "Passed"- Termine par `JR $-2` (self-loop)- Pas de source fournie dans le repo
### `interrupt_time` — Requiert CGB
- Utilise `shell.inc` → serial probable + mémoire- `.define REQUIRE_CGB 1` dans le source- Ne fonctionnera PAS sur un émulateur DMG-only
## Protocole mémoire (tous les tests)
```$A000     = status (0x80 = en cours, 0x00 = passé, autre = code erreur)$A001-03  = signature DE B0 61$A004+    = texte résultat (null-terminated)```
## Pour notre émulateur (CrabbyBoy)
Tests présents et exécutables :- ✅ `cpu_instrs` — PASSE (serial)- ✅ `instr_timing` — PASSE (serial)- ✅ `mem_timing` — PASSE (serial)- ✅ `halt_bug.gb` — PASSE (self-loop)- ⬜ `dmg_sound` — Requiert APU (ROMs pas présentes)- ⬜ `oam_bug` — Requiert PPU OAM (ROMs pas présentes)
Non exécutables :- ❌ `interrupt_time` — Requiert CGB- ❌ `cgb_sound` — Requiert CGB

