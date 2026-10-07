# 22. Un détour qui n'a pas tenu

Ce chapitre est un peu différent : il parle d'une conception qui a été
essayée (`d7c25fd`) puis abandonnée un seul commit plus tard
(chapitre 23). Cela vaut quand même la peine d'en parler — reconnaître
rapidement une impasse fait partie normale, voire saine, de la
construction d'un projet comme celui-ci, et cette tentative a quand même
appris des choses utiles qui ont été conservées par la suite.

## L'idée : rendre l'écran de la Game Boy sous forme d'image, dans le terminal

```rust
// src/display_interface.rs (de courte durée)
pub struct DisplayInterface {
    pub running: bool,
    image_state: RefCell<StatefulProtocol>,
}

impl DisplayInterface {
    pub fn new() -> Self {
        let picker = Picker::from_query_stdio().expect("impossible de détecter le terminal");
        let img = generate_buffer();
        let protocol = picker.new_resize_protocol(img);
        DisplayInterface { running: true, image_state: RefCell::new(protocol) }
    }
}
```

Les crates utilisées ici — [`ratatui`](https://ratatui.rs/) pour
construire des interfaces terminal, et
[`ratatui-image`](https://docs.rs/ratatui-image/) spécifiquement pour
afficher de vraies images bitmap à l'intérieur d'un terminal (de nombreux
terminaux modernes prennent en charge des protocoles comme Sixel ou le
protocole d'image de Kitty qui rendent cela possible) — se sont avérées
être exactement le bon choix. L'image en dégradé 160×144 utilisée ici
comme placeholder remplace ce qui sera finalement la vraie sortie pixel
du PPU (le guide associé
[PPU Background Rendering Guide](../ppu-background.md) est ce qui produit
finalement du vrai contenu pour un widget exactement comme celui-ci).

## Un petit motif Rust intéressant : `RefCell` pour une méthode de rendu `&self`

```rust
impl Widget for &DisplayInterface {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let image_widget = StatefulImage::default();
        let mut protocol = self.image_state.borrow_mut();
        image_widget.render(main, buf, &mut *protocol);
    }
}
```

La méthode `Widget::render` de `ratatui` ne donne accès qu'à `&self` (une
référence immuable), mais le widget d'image à état de `ratatui-image` a
besoin de muter son état interne de protocole à chaque rendu (par exemple
pour mettre en cache une version encodée de l'image). `RefCell` est la
manière de Rust de permettre une mutation *contrôlée, vérifiée à
l'exécution* via une référence partagée — exactement l'outil adapté à
cette inadéquation. Ce détail survit dans l'architecture finale même si
la structure environnante, elle, ne survit pas.

## Pourquoi cette conception précise a été abandonnée

Le commit suivant immédiat (`a3ec152`, chapitre 23) réécrit entièrement
cela — pas parce que `ratatui`/`ratatui-image` étaient de mauvais outils
(ils ne l'étaient pas ; ils restent utilisés), mais à cause d'un décalage
temporel que cette conception ne prenait pas encore en compte :
`DisplayInterface::update` était censé être piloté une fois par étape
émulée, sans séparation claire entre "à quelle vitesse tourne l'émulation
du CPU" et "à quelle vitesse l'écran se redessine". Une interface
terminal se redessine en réalité à un taux bien plus bas que celui
auquel le CPU exécute des instructions (au plus quelques dizaines de fois
par seconde, contre des millions d'étapes CPU par seconde) — intégrer
directement l'affichage dans la même boucle que l'exécution du CPU,
comme le faisait implicitement cette première tentative, ne passe pas à
l'échelle dès qu'on veut réellement une interface réactive en même temps
qu'une vitesse d'émulation fidèle.

## Ce que nous avons maintenant

- La confirmation que `ratatui` + `ratatui-image` peuvent afficher une
  véritable image bitmap à l'intérieur d'un terminal — un constat
  vraiment utile, conservé par la suite.
- Un exemple concret de couplage excessif (affichage étroitement lié à
  la boucle d'émulation) à reconnaître et à éviter dans la prochaine
  conception.

## Ce qui manque encore (et est sur le point d'être repensé)

- Aucune séparation encore entre "à quelle vitesse l'émulateur avance
  (tick)" et "à quelle vitesse l'écran se redessine" — c'est exactement
  ce que le chapitre 23 introduit.
