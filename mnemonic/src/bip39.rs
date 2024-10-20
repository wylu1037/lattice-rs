use std::str::FromStr;

use bip39::Language;
use bip39::Mnemonic as Bip39Mnemonic;

#[derive(Debug)]
pub struct Mnemonic {
    words: String,
    word_count: usize,
    lang: Language,
}

impl Mnemonic {
    pub fn new(lang: Language, word_count: usize) -> Mnemonic {
        let mnemonic = Bip39Mnemonic::generate_in(lang, word_count).unwrap();
        Mnemonic {
            words: mnemonic.to_string(),
            word_count: mnemonic.word_count(),
            lang,
        }
    }

    pub fn from(words: &str) -> Mnemonic {
        let mnemonic = Bip39Mnemonic::from_str(words).unwrap();
        Mnemonic {
            words: mnemonic.to_string(),
            word_count: mnemonic.word_count(),
            lang: mnemonic.language(),
        }
    }

    pub fn to_entropy(&self) -> Vec<u8> {
        let mnemonic = Bip39Mnemonic::from_str(&self.words)
            .expect(format!("recover mnemonic from words {} failed", &self.words).as_str());
        mnemonic.to_entropy()
    }

    pub fn to_seed(&self, passphrase: &str) -> [u8; 64] {
        let mnemonic = Bip39Mnemonic::from_str(&self.words)
            .expect(format!("recover mnemonic from words {} failed", &self.words).as_str());
        mnemonic.to_seed(passphrase)
    }
}

#[cfg(test)]
mod tests {
    use bip39::Language;

    use crate::bip39::Mnemonic;

    const WORDS: &str = "potato front rug inquiry old author dose little still apart below develop";

    #[test]
    fn test_new_mnemonic() {
        let mnemonic = Mnemonic::new(Language::English, 12);
        println!("{}", mnemonic.words);
        assert_eq!(mnemonic.word_count, 12);
        assert_eq!(mnemonic.lang, Language::English)
    }

    #[test]
    fn test_from_words() {
        let words = "potato front rug inquiry old author dose little still apart below develop";
        let mnemonic = Mnemonic::from(words);
        assert_eq!(mnemonic.words, words)
    }

    #[test]
    fn test_to_entropy() {
        let entropy = Mnemonic::from(WORDS).to_entropy();
        assert_eq!(
            entropy,
            vec![168, 203, 170, 244, 58, 105, 160, 30, 208, 100, 20, 213, 193, 72, 83, 30]
        )
    }
}
