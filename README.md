# HACKDLE

> Jeu de dactylographie 2D en Rust — un hacker défend son système contre des vagues de virus informatiques.

## Présentation

Hackdle est un jeu de dactylographie dans lequel le joueur incarne un hacker positionné au centre de l'écran, attaqué par des vagues de virus informatiques. Pour éliminer chaque ennemi, il doit taper correctement le mot affiché au-dessus de lui.

**Caractéristiques :**
- Vagues progressives avec difficulté croissante (spawn accéléré, mots plus longs)
- 3 types d'ennemis normaux : Fast, Classic, Heavy
- 3 types de boss (toutes les 5 vagues) : Boss classique, SummonerBoss, ReverseBoss
- SummonerBoss : invoque des sbires en cercle autour de lui, invincible tant qu'ils sont en vie
- ReverseBoss : les mots s'affichent à l'envers, mais se tapent normalement
- Système de profils persistants avec classement (best wave, moyenne)
- Effets visuels : lasers, explosions, screen shake, fond Matrix animé
- Sauvegarde JSON locale via `directories`

### États du jeu (`GameState`)

```
Login ──► MainMenu ──► InWave ──► BetweenWaves ──► InWave (boucle)
                │          │
                │          └──► GameOver ──► MainMenu
                └──► (Escape) GameOver
```

| État | Description |
|------|-------------|
| `Login` | Saisie du pseudo |
| `MainMenu` | Affichage du classement, lancement de partie |
| `InWave` | Gameplay, frappe, spawn, VFX |
| `BetweenWaves` | Résumé de vague, transition |
| `Shop` | Boutique (non implémenté) |
| `GameOver` | Fin de partie, enregistrement du score |

### Responsabilités des modules

| Module | Responsabilité |
|--------|---------------|
| `core/game.rs` | Orchestration globale, transitions d'état |
| `core/player.rs` | Vie, position, dessin du joueur |
| `core/virus.rs` | Déplacement vers le joueur, collision, bounce |
| `core/wave.rs` | Spawn progressif, dispatch du typing, kills |
| `core/boss.rs` | Comportements multi-mots, invocation de sbires |
| `ui/input.rs` | Matching caractère par caractère, zéro dépendance macroquad |
| `ui/renderer.rs` | Rendu HUD, mots, scoreboard, zéro logique métier |
| `ui/vfx.rs` | Particules, lasers, screen shake |
| `ui/matrix_bg.rs` | Fond animé (streams de symboles qui tombent) |
| `data/words.rs` | Listes statiques Easy/Medium/Hard, sélection par index |
| `data/profile.rs` | Stats par utilisateur, calcul de moyenne |
| `data/storage.rs` | Sérialisation/désérialisation JSON, chemin multiplateforme |

---

## Système de boss

Un boss apparaît toutes les 5 vagues. Le type dépend de l'index de boss :

| Vague | Boss | Comportement |
|-------|------|-------------|
| 5 | `Boss` | Enchaîne 4 mots Hard l'un après l'autre |
| 10 | `SummonerBoss` | Invoque des cycles de 4 sbires Classic, invincible tant qu'ils vivent |
| 15 | `ReverseBoss` | Mots affichés à l'envers (tapés dans le bon ordre) |
| 20+ | Alternance Summoner/Reverse | Difficulté croissante |

---

## Roadmap

| Phase | Statut | Contenu |
|-------|--------|---------|
| **Phase 1** — Fondations | ✅ | Game loop, structs Game/Player/Virus, rendu de base, input clavier |
| **Phase 2** — Gameplay | ✅ | Système de frappe, vagues, scoring, mort des ennemis |
| **Phase 3** — Contenu | ✅ | 3 types d'ennemis, 3 types de boss, mots par difficulté |
| **Phase 4** — Progression | 🔲 | Monnaie, boutique, améliorations entre les vagues |
| **Phase 5** — Polish | 🔲 | Animations d'entrée, sons supplémentaires, équilibrage |

---

## Sauvegarde

Les profils sont sauvegardés en JSON dans le dossier de données applicatives du système :

| OS | Chemin |
|----|--------|
| Linux | `~/.local/share/Hackdle/save.json` |
| macOS | `~/Library/Application Support/org.hackdle.Hackdle/save.json` |
| Windows | `%APPDATA%\hackdle\Hackdle\data\save.json` |

---