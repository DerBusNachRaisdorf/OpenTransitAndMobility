use std::{
    cmp::min,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone)]
struct Matrix<T: Clone> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T: Clone> Matrix<T> {
    pub fn new(rows: usize, cols: usize, fill: T) -> Self {
        Self {
            data: vec![fill; rows * cols],
            rows,
            cols,
        }
    }

    pub fn get(&self, row: usize, col: usize) -> &T {
        &self.data[row * self.cols + col]
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut T {
        &mut self.data[row * self.cols + col]
    }

    #[allow(dead_code)]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[allow(dead_code)]
    pub fn cols(&self) -> usize {
        self.cols
    }
}

impl<T: Clone> Index<(usize, usize)> for Matrix<T> {
    type Output = T;
    fn index(&self, (row, col): (usize, usize)) -> &T {
        self.get(row, col)
    }
}

impl<T: Clone> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut T {
        self.get_mut(row, col)
    }
}

fn min3<T: Ord>(v1: T, v2: T, v3: T) -> T {
    min(v1, min(v2, v3))
}

pub fn edit_distance(word1: &str, word2: &str) -> usize {
    let mut cache =
        Matrix::<usize>::new(word1.len() + 1, word2.len() + 1, usize::MAX);

    for j in 0..=word1.len() {
        cache[(j, word2.len())] = word1.len() - j;
    }
    for i in 0..=word2.len() {
        cache[(word1.len(), i)] = word2.len() - i;
    }

    for i in (0..word1.len()).rev() {
        for j in (0..word2.len()).rev() {
            if word1.chars().nth(i) == word2.chars().nth(j) {
                cache[(i, j)] = cache[(i + 1, j + 1)];
            } else {
                cache[(i, j)] = 1 + min3(
                    cache[(i + 1, j)],
                    cache[(i, j + 1)],
                    cache[(i + 1, j + 1)],
                );
            }
        }
    }

    cache[(0, 0)]
}

pub fn edit_distance_shorten(word1: &str, word2: &str) -> usize {
    if word1.len() <= word2.len() {
        edit_distance(word1, &word2[..word1.len()])
    } else {
        edit_distance(&word1[..word2.len()], word2)
    }
}
