use serde::{Deserialize, Serialize};

// Debug : affiche {:?} dans le println! ou dbg! (pour le développement)
// Clone :  Permet de dupliquer une valeure
// Serialize : Permet de convertir la struc en Json (serde)
// Deserialize : Permet de reconstruire le struct depuis le Json (serde)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub best_wave: u32, // Meilleure vague atteinte sur toutes les parties
    pub games_played: u32, // Nombre total de parties jouées
    pub total_waves: u64, // Somme de toutes les vagues atteintes (sert à calculer la moyenne)
}

impl UserProfile {
    // Crée un nouveau profil
    // Tous les compteurs démarrent à 0
    pub fn new(username: String) -> Self {
        Self {
            username,
            best_wave: 0,
            games_played: 0,
            total_waves: 0,
        }
    }

    // Enregistre le résultat d'une partie terminée.
    pub fn register_run(&mut self, wave_reached: u32) { // Prend &mut self car elle modifie le profil
        self.games_played += 1;
        self.total_waves += wave_reached as u64;
        self.best_wave = self.best_wave.max(wave_reached);
    }

    // Calcule et retourne la vague moyenne sur toutes les parties
    // Retourne 0.0 si aucune partie n'a été jouée (évite une division par zéro)
    pub fn average_wave(&self) -> f32 { // Prend `&self` car elle lit seulement le profil sans le modifier
        if self.games_played == 0 {
            0.0
        } else {
            self.total_waves as f32 / self.games_played as f32
        }
    }
}