use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use directories::ProjectDirs; // Crate externe pour trouver les dossiers système
use serde::{Deserialize, Serialize};
use crate::data::UserProfile;

// Debug : affiche {:?} dans le println! ou dbg! (pour le développement)
// Clone :  Permet de dupliquer une valeure
// Serialize : Permet de convertir la struc en Json (serde)
// Deserialize : Permet de reconstruire le struct depuis le Json (serde)
// Default : Permet de créer une valeure vide (ici liste vide)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveData {
    pub profiles: Vec<UserProfile>, // Liste des profils utilisateur sauvegardés
}

// Structure utilitaire sans état, sert uniquement de namespace pour les fonctions de stockage
pub struct Storage;

impl Storage {
    // Charge les données depuis le fichier de sauvegarde (Pas dans le dossier, norme moderne)
    // Si le fichier est absent ou corrompu, retourne une valeur par défaut (SaveData vide)
    pub fn load() -> SaveData {
        let path = Self::data_path();
        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(), // Désérialise le JSON, ou valeur par défaut si échec
            Err(_) => SaveData::default(), // Fichier absent ou illisible (données vides)
        }
    }

    // Sauvegarde les données dans le fichier JSON
    // Crée les dossiers intermédiaires si nécessaire.
    // Retourne une erreur I/O en cas de problème d'écriture ou de sérialisation
    pub fn save(data: &SaveData) -> io::Result<()> {
        let path = Self::data_path();

        // Crée récursivement les répertoires parents si ils n'existent pas encore
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Sérialise les données en JSON lisible (indenté)
        let serialized = serde_json::to_string_pretty(data)
            .map_err(|err| io::Error::new(ErrorKind::Other, err))?; // Convertit l'erreur serde en io::Error

        fs::write(path, serialized)?; // Écrit le contenu dans le fichier
        Ok(())
    }

    // Détermine le chemin du fichier de sauvegarde selon le système d'exploitation
    //   - Linux   : ~/.local/share/hackdle/save.json
    //   - macOS   : ~/Library/Application Support/org.hackdle.Hackdle/save.json
    //   - Windows : C:\Users\<user>\AppData\Roaming\hackdle\Hackdle\data\save.json
    // Si le dossier système est introuvable, repli sur un fichier local hackdle_save.json
    fn data_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("org", "hackdle", "Hackdle") {
            proj_dirs.data_dir().join("save.json")
        } else {
            PathBuf::from("hackdle_save.json") // Chemin de secours dans le répertoire courant
        }
    }
}