#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct WordList;

impl WordList {
    pub fn get(difficulty: Difficulty) -> &'static [&'static str] {
        match difficulty {
            Difficulty::Easy => EASY_WORDS,
            Difficulty::Medium => MEDIUM_WORDS,
            Difficulty::Hard => HARD_WORDS,
        }
    }

    pub fn pick(difficulty: Difficulty, index: usize) -> &'static str {
        let list = Self::get(difficulty);
        list[index % list.len()]
    }
}

// Mots courts 2-3 lettres
static EASY_WORDS: &[&str] = &[
    "bit", "bug", "cpu", "ram", "log", "hex", "key", "net",
    "run", "ssh", "tcp", "udp", "vim", "www", "zip", "api",
    "bot", "cmd", "dns", "elf", "ftp", "git", "hub", "ip",
    "jar", "lib", "map", "nul", "obj", "php", "sql", "url",
    "var", "xml", "yes", "zsh", "awk", "cat", "cut", "dig",
];

// Mots moyens 5-7 lettres
static MEDIUM_WORDS: &[&str] = &[
    "array", "cache", "class", "clone", "crash", "debug", "event",
    "fetch", "frame", "index", "input", "layer", "linux", "login",
    "macro", "mutex", "nginx", "patch", "pixel", "proxy", "query",
    "queue", "quota", "regex", "route", "scope", "shell", "stack",
    "stdin", "stdio", "token", "trait", "tuple", "virus", "watch",
    "yield", "async", "await", "build", "bytes", "close", "codec",
    "delta", "emacs", "errno", "forge", "grant", "guard", "hooks",
];

// Mots longs 8+ lettres
static HARD_WORDS: &[&str] = &[
    "assembly", "callback", "checksum", "compiler", "database",
    "deadlock", "encoding", "ethernet", "firewall", "firmware",
    "frontend", "function", "graphics", "hashbrown", "iterator",
    "keyboard", "lifetime", "loopback", "manifest", "markdown",
    "metadata", "overflow", "pipeline", "platform", "priority",
    "protocol", "refactor", "renderer", "resource", "rollback",
    "segments", "shutdown", "sideband", "snapshot", "syscall",
    "template", "terminal", "throttle", "topology", "unittest",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_wraps_around() {
        let list = WordList::get(Difficulty::Easy);
        let len = list.len();
        assert_eq!(WordList::pick(Difficulty::Easy, 0), list[0]);
        assert_eq!(WordList::pick(Difficulty::Easy, len), list[0]);
        assert_eq!(WordList::pick(Difficulty::Easy, len + 1), list[1]);
    }

    #[test]
    fn all_difficulties_have_words() {
        assert!(!WordList::get(Difficulty::Easy).is_empty());
        assert!(!WordList::get(Difficulty::Medium).is_empty());
        assert!(!WordList::get(Difficulty::Hard).is_empty());
    }

    #[test]
    fn easy_words_are_short() {
        for word in WordList::get(Difficulty::Easy) {
            assert!(word.len() <= 4, "mot trop long en Easy : {}", word);
        }
    }

    #[test]
    fn hard_words_are_long() {
        for word in WordList::get(Difficulty::Hard) {
            assert!(word.len() >= 8, "mot trop court en Hard : {}", word);
        }
    }
}