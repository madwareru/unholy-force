/*
Данный функционал позаимствован (с некоторыми изменениями) из крейта egui_code_editor
ввиду более старой версии egui и слегка других нужд. Огромное спасибо Роману Чумаку за этот
замечательный крейт ☺
*/

use std::{iter::Peekable, str::Chars};

const ROOT_CHAR: char = ' ';

#[derive(Debug, Clone, PartialEq, Eq)]
/// Приставочное дерево для удобного поиска слов.
pub struct Trie {
    root: char,
    is_word: bool,
    leaves: Vec<Trie>,
}

impl Trie {
    /// Создаёт пустое дерево.
    pub fn new() -> Self {
        Self {
            root: ROOT_CHAR,
            is_word: false,
            leaves: vec![],
        }
    }

    /// Очищает все слова в дереве.
    pub fn clear(&mut self) {
        self.leaves.clear();
    }

    /// Добавляет новое слово в дерево.
    pub fn push(&mut self, word: &str) {
        self.push_chars(&mut word.chars());
    }

    /// Добавляет в дерево новое слово из итератора символов.
    pub fn push_chars(&mut self, word: &mut Chars) {
        let next_char = word.next();

        if next_char.is_none() {
            self.is_word = true;
            self.leaves.sort();
            self.leaves.reverse();
            return;
        }

        let Some(first) = next_char else {
            unreachable!()
        };

        let leaf_attempt = self.leaves
            .iter_mut()
            .find(|l| l.root == first);

        if leaf_attempt.is_none() {
            let mut new = Self { root: first, ..Self::new() };
            new.push_chars(word);
            self.leaves.push(new);
            return;
        }

        let Some(leaf) = leaf_attempt else {
            unreachable!()
        };

        leaf.push_chars(word);
    }

    /// Получает список слов в Trie.
    /// В случае если это было дерево для приставки,
    /// а не корневое, слова не включают эту приставку.
    pub fn words(&self) -> Vec<String> {
        let mut words = vec![];
        for child in self.leaves.iter() {
            child.words_recursive("", &mut words);
        }
        words
    }

    /// Находит список автодополнений для приставки prefix.
    /// Если автодополнений нет, возвращает пустой список.
    /// Автодополнения не включают в себя приставку
    pub fn find_completions(&self, prefix: &str) -> Vec<String> {
        self.find_by_prefix(prefix)
            .map(|t| t.words())
            .unwrap_or_default()
    }

    /// Возвращает Some(Trie) для слов, начинающихся с приставки prefix,
    /// если такой существовал, иначе возвращает None. В дальнейшем все
    /// слова данного Trie не будут включать приставку и будут выглядеть
    /// как автодополнения для неё
    pub fn find_by_prefix(&self, prefix: &str) -> Option<&Trie> {
        let mut found = None;
        let mut start = " ".to_string();
        start.push_str(prefix);
        let mut part = start.chars().peekable();
        self.find_recursive(&mut part, &mut found);
        found
    }

    fn words_recursive(&self, prefix: &str, words: &mut Vec<String>) {
        let mut prefix = prefix.to_string();
        prefix.push(self.root);

        if self.is_word {
            words.push(prefix.clone());
        }

        for child in self.leaves.iter() {
            child.words_recursive(&prefix, words);
        }
    }

    fn find_recursive<'a>(&'a self, part: &mut Peekable<Chars>, found: &mut Option<&'a Trie>) {
        let part_matches = part.next().map(|c| self.root == c).unwrap_or(false);
        if !part_matches {
            return;
        }

        if part.peek().is_none() {
            *found = Some(self);
            return;
        }

        self.leaves
            .iter()
            .for_each(|l| l.find_recursive(&mut part.clone(), found));
    }
}

impl PartialOrd for Trie {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Trie {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.root.cmp(&other.root)
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_contains<'a>(words: impl IntoIterator<Item=&'a String>, word: &'a str) {
        if words.into_iter().all(|it| !(*it).eq(word)) {
            panic!("expected to contain {}", word);
        }
    }

    #[test]
    fn test_trie() {
        let mut trie = Trie::new();
        trie.push("hello");
        trie.push("world");
        trie.push("rust");
        let words = trie.words();
        check_contains(&words, "hello");
        check_contains(&words, "world");
        check_contains(&words, "rust");

        trie.clear();
        let words = trie.words();
        assert!(words.is_empty());

        trie.push("hero");
        trie.push("tries");
        trie.push("to");
        trie.push("trust");

        let Some(sub_trie) = trie.find_by_prefix("t") else {
            panic!("expected to find sub trie for prefix 't'");
        };
        let words = sub_trie.words();
        // приставка опускается в дополнениях
        check_contains(&words, "ries");
        check_contains(&words, "o");
        check_contains(&words, "rust");
    }
}
