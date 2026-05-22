# Test ROMs — Output Methods
**Source :** https://github.com/retrio/gb-test-roms

---

## Résumé

| Test | Serial (SB/$81) | Mémoire ($A000+) | CGB requis | Fin de test |
|------|:---:|:---:|:---:|---|
| `cpu_instrs` | ✅ | ❌ | Non | Serial "Passed" ou self-loop |
| `instr_timing` | ✅ | ❌ | Non | Serial "Passed" ou self-loop |
| `mem_timing` | ✅ | ❌ | Non | Serial "Passed" ou self-loop |
| `mem_timing-2` | ❌ | ✅ | Non | Self-loop (`JP $`) |
| `interrupt_time` | ✅ (probable) | ❌ | **Oui** | Self-loop (`JP $`) |
| `dmg_sound` | ❌ | ✅ | Non | Self-loop (`JP $`) |
| `cgb_sound` | ❌ | ✅ | **Oui** | Self-loop (`JP $`) |
| `oam_bug` | ❌ | ✅ | Non | Self-loop (`JP $`) |
| `halt_bug.gb` | ❌ | ✅ | Non | Self-loop (`JR $`) |

---

## Détails

### Tests avec serial (ancien framework `shell.inc`)

`cpu_instrs`, `instr_timing`, `mem_timing` utilisent l'ancien `shell.inc` :

1. Envoient chaque caractère sur le port série : `SB = char`, `SC = $81`
2. Affichent le texte à l'écran (LCD)
3. Terminent par un self-loop (`JP $`)

Notre émulateur détecte `"Passed"`/`"Failed"` dans la sortie serial et termine immédiatement.

> ⚠️ Ces tests **n'écrivent pas** en mémoire (`$A000`).

### Tests sans serial (nouveau framework)

`mem_timing-2`, `halt_bug`, `dmg_sound`, `oam_bug` utilisent un framework plus récent :

- **Pas de sortie serial**
- Résultats uniquement en mémoire (`$A000+` avec signature `DE B0 61`)

---

## Protocole mémoire (nouveau framework uniquement)

S'applique à : `mem_timing-2`, `dmg_sound`, `cgb_sound`, `oam_bug`, `halt_bug`

```asm
$A000     = status (0x80 = en cours, 0x00 = passé, autre = code erreur)
$A001-03  = signature DE B0 61
$A004+    = texte résultat (null-terminated)
```

---

## Pour CrabbyBoy

**Tests présents et exécutables :**

| Statut | Test | Méthode détection |
|:---:|---|---|
| ✅ | `cpu_instrs` | Serial |
| ✅ | `instr_timing` | Serial |
| ✅ | `mem_timing` | Serial |
| ✅ | `halt_bug.gb` | Mémoire + self-loop |
| ⬜ | `dmg_sound` | Requiert APU (ROMs absentes) |
| ⬜ | `oam_bug` | Requiert PPU OAM (ROMs absentes) |

