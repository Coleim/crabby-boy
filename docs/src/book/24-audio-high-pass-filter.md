# 24. Un filtre passe-haut (high-pass filter) pour un son plus propre

Avec les 4 canaux passant les tests de Blargg (chapitre 20), la sortie
mixée brute peut encore présenter un effet secondaire indésirable : une
composante continue (DC offset) qui dérive lentement (la valeur moyenne
de la forme d'onde n'est pas centrée sur zéro), ce qui tend à produire un
bourdonnement audible ou un "thump" plutôt qu'un son propre. Ce petit
commit (`58e899f`) corrige cela avec un classique du traitement du
signal : un **filtre passe-haut (high-pass filter)**.

## Ce que fait un filtre passe-haut, sur le plan conceptuel

Un filtre passe-haut laisse passer le contenu du signal qui change
rapidement, tout en "oubliant"/supprimant progressivement une dérive
lente, quasi constante. En termes audio : il conserve le son réel voulu,
tout en supprimant une composante continue ou un grondement de très
basse fréquence qui ne devrait pas être là. Cette conception précise est
un simple filtre à un pôle — il n'a besoin de se souvenir que de
l'échantillon d'entrée et de sortie *précédent*, pas de tout un
historique.

## L'implémentation

```rust
// src/audio/audio_output.rs
let mut hp_x1 = 0.0; // échantillon d'entrée précédent
let mut hp_y1 = 0.0; // échantillon de sortie précédent
let hp_cutoff_hz = 20.0;
let dt = 1.0 / selected_sample_rate as f32;
let rc = 1.0 / (2.0 * std::f32::consts::PI * hp_cutoff_hz);
let hp_alpha = rc / (rc + dt);

// ... à l'intérieur du callback audio, une fois par échantillon :
hp_y1 = hp_alpha * (hp_y1 + last_sample - hp_x1);
hp_x1 = last_sample;
for out in frame.iter_mut() {
    *out = hp_y1;
}
```

`hp_cutoff_hz = 20.0` définit la "fréquence de coupure" du filtre —
grosso modo, le seuil en dessous duquel le contenu est atténué. 20 Hz se
situe tout au bord inférieur de l'audition humaine, choisi
spécifiquement pour supprimer la dérive DC et le grondement sub-audible
tout en laissant intacte chaque fréquence réellement audible. `rc` et
`hp_alpha` proviennent de la formule standard pour ce type de filtre (un
"filtre passe-haut RC du premier ordre", si vous voulez approfondir la
théorie électronique générale) — ce qui compte vraiment à retenir pour
ce projet n'est pas de redériver la formule soi-même, mais de reconnaître
*quand* on en a besoin : chaque fois que la sortie audio mixée a un
bourdonnement/thump audible que les vérifications de correction de type
ROM de test (chapitre 20) ne détecteraient pas, puisqu'elles vérifient le
comportement logique des registres, pas la qualité finale de la forme
d'onde de type analogique.

## Où se situe ce filtre

Remarquez que ceci se trouve dans `audio_output.rs` — la couche qui
communique avec `cpal` et la véritable carte son (chapitre 15) — et non
à l'intérieur de l'APU elle-même. L'APU reste concentrée sur l'émulation
fidèle de ce que calcule le vrai matériel (hardware) de la Game Boy ; le
nettoyage final du signal pour le chemin de sortie propre *à cet
émulateur en particulier* est une préoccupation séparée, de niveau
présentation, conservée à sa propre place.

## Ce que nous avons maintenant

- Une sortie audio plus propre, avec la composante continue et le
  grondement sub-audible filtrés avant d'atteindre de vrais haut-parleurs.
- Une séparation claire entre "émulation matérielle fidèle" (APU) et
  "amélioration de la qualité de sortie" (`audio_output.rs`) — un
  principe de conception général à garder à l'esprit.

## Ce qui manque encore

- Il s'agit d'une correction de couche présentation, pas d'une correction
  de fidélité matérielle — elle n'a aucune incidence sur les ROM de test
  `dmg_sound` du chapitre 20, qui continuent de réussir ou d'échouer
  indépendamment d'elle.
