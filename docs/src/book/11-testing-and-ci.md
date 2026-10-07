# 11. Infrastructure de test & Intégration Continue

Avec un véritable harnais de test automatisé en place (Chapitre 9) et une
pile croissante de ROMs de test (Chapitres 4, 5, 10), deux choses se
produisent en parallèle dans cette partie de l'histoire du projet : les
conventions de test sont consignées correctement, et un collaborateur
rejoint le projet pour mettre en place l'Intégration Continue
(CI) — construisant et testant automatiquement le projet à chaque push.

## Consigner les conventions des ROMs de test

Le Chapitre 10 a déjà introduit les deux conventions de résultat de
style Blargg (texte série vs. signature mémoire). Cela est désormais
formalisé dans `TEST_ROM_SPECS.MD`, résumé en un tableau de référence :

| Test | Série (SB/$81) | Mémoire ($A000+) | CGB requis | Fin de test |
|---|:---:|:---:|:---:|---|
| `cpu_instrs` | ✅ | ❌ | Non | Série "Passed" ou boucle infinie |
| `instr_timing` | ✅ | ❌ | Non | Série "Passed" ou boucle infinie |
| `mem_timing` | ✅ | ❌ | Non | Série "Passed" ou boucle infinie |
| `mem_timing-2` | ❌ | ✅ | Non | Boucle infinie (`JP $`) |
| `dmg_sound` | ❌ | ✅ | Non | Boucle infinie (`JP $`) |
| `halt_bug.gb` | ❌ | ✅ | Non | Boucle infinie (`JR $`) |

Écrire un tableau comme celui-ci n'est pas du travail inutile — c'est ce
qui permet d'ajouter plus tard de nouvelles ROMs de test (tests son, tests
OAM) en vérifiant simplement "quelle ligne correspond à celle-ci" plutôt
que de refaire la rétro-ingénierie de son comportement à chaque fois.

## Mise en place de la CI : un workflow GitHub Actions

```yaml
# .github/workflows/rust.yml
name: Rust

on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Build
      run: cargo build --verbose
    - name: Run tests
      run: cargo test --verbose
```

C'est à peu près aussi simple que possible pour une CI : à chaque push
ou pull request, une machine Ubuntu fraîche est démarrée, le code est
récupéré, construit, puis `cargo test` est lancé. Comme le Chapitre 9 a
déjà transformé les ROMs de test en véritables fonctions `#[test]`, cela
"fonctionne directement" — la CI n'a pas besoin de connaître quoi que ce
soit sur les Game Boy, les cartouches ou les opcodes ; elle a seulement
besoin de savoir comment exécuter une suite de tests Rust.

## Une vraie leçon : les tests peuvent être *trop* lents pour la CI

La toute première version de ce workflow avait son étape `cargo test`
**mise en commentaire**, avec une note indiquant que la simple
construction suffisait pour l'instant. Pourquoi ? Exécuter l'*intégralité*
de la ROM `cpu_instrs.gb` (par opposition à ses 11 sous-tests individuels
plus petits) nécessite un très grand nombre d'étapes CPU émulées pour se
terminer — c'est bien d'exécuter cela localement et d'attendre, mais
beaucoup moins pratique de le faire à chaque push CI si cela ralentit
significativement le retour d'information. La correction finale
(`cea7aca`, "Activate all tests but too long all_cpu_instrs") a consisté
à activer *toutes* les ROMs de test individuelles rapides, tout en
laissant explicitement en commentaire le seul test de ROM combinée
réellement lent :

```rust
// Too long
// cpu_instr_test!(test_all_cpu_instrs, "./tests/cpu_instrs.gb");
```

C'est une leçon réutilisable et générale pour tout projet ayant un test
lent mais précieux : ne désactivez pas toute votre suite de tests à cause
d'un seul membre lent — isolez-le, et gardez la majorité rapide active à
chaque commit.

## Ce que nous avons maintenant

- `TEST_ROM_SPECS.MD`, documentant exactement comment interpréter chaque
  famille de ROM de test.
- Un pipeline GitHub Actions fonctionnel, construisant et testant à
  chaque push et pull request.
- Une exception délibérée et documentée pour la seule ROM de test trop
  lente pour tourner à chaque push CI.

## Ce qui manque encore

- Pas encore de ROMs de test spécifiques au PPU (comme `dmg-acid2`, un
  test de correction du rendu PPU bien connu) — il n'y a pas encore de
  véritable rendu PPU à tester. Cela commence dans le tout prochain
  chapitre.
- Pas encore de ROMs de test son qui passent (`dmg_sound` est documenté,
  mais pas encore exécutable) — ce sera la Partie VI.
