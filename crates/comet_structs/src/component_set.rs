#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentSet {
    words: Vec<u64>,
}

impl ComponentSet {
    pub fn new() -> Self {
        Self { words: Vec::new() }
    }

    pub fn from_indices(indices: Vec<usize>) -> Self {
        let mut set = Self::new();
        for index in indices {
            set.insert(index);
        }
        set
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    fn trim_trailing_zeros(&mut self) {
        while self.words.last().copied() == Some(0) {
            self.words.pop();
        }
    }

    fn ensure_word(&mut self, index: usize) {
        let word = index / 64;
        if self.words.len() <= word {
            self.words.resize(word + 1, 0);
        }
    }

    pub fn is_subset(&self, other: &ComponentSet) -> bool {
        for (i, word) in self.words.iter().enumerate() {
            let rhs = other.words.get(i).copied().unwrap_or(0);
            if (word & !rhs) != 0 {
                return false;
            }
        }
        true
    }

    pub fn is_superset(&self, other: &ComponentSet) -> bool {
        other.is_subset(self)
    }

    pub fn insert(&mut self, index: usize) {
        self.ensure_word(index);
        let bit = index % 64;
        self.words[index / 64] |= 1u64 << bit;
    }

    pub fn remove(&mut self, index: usize) {
        let word = index / 64;
        if let Some(slot) = self.words.get_mut(word) {
            let bit = index % 64;
            *slot &= !(1u64 << bit);
            self.trim_trailing_zeros();
        }
    }

    pub fn contains(&self, index: usize) -> bool {
        let word = index / 64;
        let bit = index % 64;
        self.words
            .get(word)
            .is_some_and(|slot| (*slot & (1u64 << bit)) != 0)
    }

    pub fn to_vec(&self) -> Vec<usize> {
        let mut result = Vec::new();
        for (word_idx, mut word) in self.words.iter().copied().enumerate() {
            while word != 0 {
                let bit = word.trailing_zeros() as usize;
                result.push(word_idx * 64 + bit);
                word &= word - 1;
            }
        }
        result
    }

    pub fn size(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }
}
