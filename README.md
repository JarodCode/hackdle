# HACKDLE

> Jeu de dactylographie 2D en Rust — un hacker attaqué par des virus informatiques.

**Stack :** Rust 2024 Edition + `macroquad 0.4` — pas de game engine, architecture manuelle.

---

## Présentation

Hackdle est un jeu de dactylographie dans lequel le joueur incarne un hacker positionné au centre de l'écran, attaqué par des vagues de virus informatiques. Pour éliminer chaque ennemi, il doit taper correctement et rapidement le mot affiché au-dessus de lui.

**Caractéristiques :**
- Vagues progressives avec difficulté croissante
- 4 types d'ennemis : Rapide, Classique, Lourd, Boss
- Boss multi-vie en fin de manche
- Système de monnaie virtuelle et boutique d'améliorations entre les vagues
- Mots classés par difficulté chargés depuis des fichiers JSON

---

## Stack technique

| Crate | Version | Rôle |
|-------|---------|------|
| `macroquad` | 0.4 | Rendu 2D, fenêtre, input clavier |
| `serde` + `serde_json` | 1 | Chargement des mots depuis JSON |
| `rand` | 0.8 | Génération aléatoire (spawn, sélection de mots) |
| `thiserror` | 1 | Gestion d'erreurs typées |

```toml
# Cargo.toml
[dependencies]
macroquad  = "0.4"
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
rand       = "0.8"
thiserror  = "1"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

---

## Architecture

Architecture orientée **structs + traits + game loop manuelle**. Pas d'ECS.

```
src/
├── main.rs              # Entry point, game loop macroquad::main
├── core/
│   ├── mod.rs
│   ├── game.rs          # struct Game — état global, dispatch update/draw
│   ├── player.rs        # struct Player — vie, position, stats
│   ├── virus.rs         # struct Virus, enum VirusKind — comportements ennemis
│   └── wave.rs          # Génération et gestion des vagues
├── ui/
│   ├── mod.rs
│   ├── renderer.rs      # Rendu graphique pur — aucune logique métier
│   └── input.rs         # Logique de frappe — 100% indépendante de macroquad
└── data/
    ├── mod.rs
    └── words.rs          # Chargement et sélection des mots par difficulté

assets/
├── words/               # JSON mots classés par difficulté
├── sprites/             # Textures PNG ennemis, joueur, fond
└── fonts/               # Polices .ttf
```

### Game loop

```rust
#[macroquad::main("Hackdle")]
async fn main() {
    let mut game = Game::new().await;
    loop {
        game.update(get_frame_time());
        game.draw();
        next_frame().await;
    }
}
```

### Responsabilités des modules

| Module | Responsabilité |
|--------|---------------|
| `core/game.rs` | État global, dispatch selon `GameState` |
| `core/player.rs` | Vie, position, stats du joueur |
| `core/virus.rs` | Comportements et déplacement des ennemis |
| `core/wave.rs` | Spawn des ennemis, progression de difficulté |
| `ui/input.rs` | Logique de frappe pure — matching, progression, résultat |
| `ui/renderer.rs` | Rendu graphique pur — aucune logique métier |
| `data/words.rs` | Chargement JSON, sélection mots par difficulté |

---

## Bonnes pratiques Rust

### 1. Typage strict — enums et newtypes

```rust
// États du jeu
enum GameState { MainMenu, InWave, BetweenWaves, Shop, GameOver }

// Types d'ennemis
enum VirusKind { Fast, Classic, Heavy, Boss }

// Newtypes pour les valeurs métier — évite les confusions de types
struct Health(u32);
struct Currency(u32);
struct WaveNumber(u32);
```

### 2. Séparation update / draw

Chaque struct avec une représentation visuelle expose exactement deux méthodes. Le rendu ne contient **jamais** de logique métier.

```rust
impl Virus {
    pub fn update(&mut self, dt: f32) { /* logique, déplacement */ }
    pub fn draw(&self)               { /* rendu pur, aucune mutation */ }
}
```

### 3. Delta time obligatoire

Tous les mouvements utilisent `dt` (`get_frame_time()`). Jamais de vitesse en pixels/frame.

```rust
// ❌ Interdit — dépend du framerate
self.x += 2.0;

// ✅ Correct — indépendant du framerate
self.x += self.speed * dt;
```

### 4. Gestion des erreurs avec thiserror

Pas de `unwrap()` en dehors des tests ou contextes explicitement documentés.

```rust
#[derive(thiserror::Error, Debug)]
pub enum GameError {
    #[error("Failed to load asset: {0}")]
    AssetLoad(String),
    #[error("Invalid word list: {0}")]
    WordList(String),
}

// Propager avec ?, jamais unwrap()
pub fn load_words(path: &str) -> Result<Vec<String>, GameError> { ... }
```

### 5. Logique de frappe isolée

`ui/input.rs` ne dépend pas de macroquad. La logique est pure et testable sans fenêtre.

```rust
// src/ui/input.rs — zéro import macroquad
pub struct TypingState {
    pub target: String,
    pub progress: usize,
}

impl TypingState {
    pub fn type_char(&mut self, c: char) -> TypingResult { ... }
}
```

### 6. Iterators et retain()

```rust
// Supprimer les virus morts sans borrow issues
self.viruses.retain(|v| v.is_alive());

// Préférer les iterators aux boucles impératives
let boss_count = self.viruses.iter()
    .filter(|v| v.kind == VirusKind::Boss)
    .count();
```

### 7. Performance — pas d'allocations en game loop

```rust
// Pré-allouer au chargement
let mut viruses: Vec<Virus> = Vec::with_capacity(64);

// Passer &str, pas String, pour les mots affichés
fn draw_word(word: &str, x: f32, y: f32) { ... }

// Textures chargées une seule fois, passées par référence
pub struct Assets {
    pub virus_texture: Texture2D,
    pub font: Font,
}
```

---

## Tests

La logique pure est testable sans démarrer macroquad. Chaque module de logique expose ses tests unitaires.

**Ce qui doit être testé :**
- Frappe correcte → progression du mot
- Frappe incorrecte → aucune progression
- Mot complet → ennemi éliminé
- Calcul de récompense selon le type d'ennemi
- Spawn des ennemis selon le numéro de vague
- Sélection de mots par niveau de difficulté

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_char_advances_progress() {
        let mut state = TypingState::new("rust".to_string());
        assert_eq!(state.type_char('r'), TypingResult::Correct);
        assert_eq!(state.progress, 1);
    }

    #[test]
    fn wrong_char_does_not_advance() {
        let mut state = TypingState::new("rust".to_string());
        assert_eq!(state.type_char('x'), TypingResult::Wrong);
        assert_eq!(state.progress, 0);
    }
}
```

---

## Workflow

### Commandes

```bash
cargo run                        # Lancer le jeu
cargo test                       # Tests unitaires
cargo clippy -- -D warnings      # Linter strict — zéro warning toléré
cargo fmt                        # Formater le code
cargo build --release            # Build optimisé
```

### Checklist avant chaque commit

- [ ] `cargo clippy -- -D warnings` passe sans erreur
- [ ] `cargo test` — tous les tests passent
- [ ] `cargo fmt` appliqué
- [ ] Aucun `unwrap()` non documenté ajouté
- [ ] `dt` utilisé pour tout mouvement

### Règles de collaboration avec Claude

- Toujours préciser le fichier `src/` ciblé : *"Dans `src/core/virus.rs`, ajoute..."*
- Indiquer le mode : **prototype rapide** ou **prêt pour production**
- Si une fonction dépasse 40 lignes, demander explicitement la découpe en fonctions auxiliaires
- Demander les unit tests avec chaque module de logique pure

---

## Roadmap

| Phase | Contenu |
|-------|---------|
| **Phase 1** — Fondations | Game loop, structs Game/Player/Virus, rendu de base, input clavier |
| **Phase 2** — Gameplay | Système de frappe, vagues, scoring, mort des ennemis |
| **Phase 3** — Contenu | Types d'ennemis, boss, mots JSON par difficulté |
| **Phase 4** — Progression | Monnaie, boutique, améliorations entre les vagues |
| **Phase 5** — Polish | Effets visuels, sons, menus, animations, équilibrage |

---

*Hackdle — Projet Rust pédagogique | Licence MIT*