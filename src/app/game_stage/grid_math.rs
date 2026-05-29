use egui::Key::P;

enum TraverseAreaIteratorState {
    Done,
    YieldOne,
    Yield2x2 { index: usize },
    Yield3x3 { index: usize },
    YieldCorners { corner_index: usize },
    YieldEdges {
        edge_width: usize,
        half_width: usize,
        edge_index: usize,
        i: usize,
    },
    YieldEdgeCenters {
        edge_width: usize,
        half_width: usize,
        edge_index: usize,
    }
}

pub struct TraverseAreaInwardIterator {
    base_offset_x: usize,
    base_offset_y: usize,
    width: usize,
    state: TraverseAreaIteratorState,
}

pub struct TraverseAreaOutwardIterator {
    base_offset_x: usize,
    base_offset_y: usize,
    width: usize,
    target_width: usize,
    state: TraverseAreaIteratorState,
}

/// Вспомогательная функция для осуществления "обратной заливки"
/// квадратной области начиная с внешних углов в сторону центра рёбер,
/// после чего процесс итеративно повторяется для вложенных квадратных
/// областей.
pub fn traverse_area_inward(width: usize) -> TraverseAreaInwardIterator {
    TraverseAreaInwardIterator {
        base_offset_x: 0,
        base_offset_y: 0,
        width,
        state: match width {
            0 => TraverseAreaIteratorState::Done,
            1 => TraverseAreaIteratorState::YieldOne,
            2 => TraverseAreaIteratorState::Yield2x2 { index: 0 },
            3 => TraverseAreaIteratorState::Yield3x3 { index: 0 },
            _ => TraverseAreaIteratorState::YieldCorners { corner_index: 0 },
        },
    }
}

impl Iterator for TraverseAreaInwardIterator {
    type Item = [usize; 2];

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            TraverseAreaIteratorState::Done => None,
            TraverseAreaIteratorState::YieldOne => {
                self.state = TraverseAreaIteratorState::Done;
                Some([self.base_offset_x, self.base_offset_y])
            },
            TraverseAreaIteratorState::Yield2x2 { index } => {
                let [x, y] = match index {
                    0 => {
                        self.state = TraverseAreaIteratorState::Yield2x2 { index: index + 1 };
                        [self.base_offset_x, self.base_offset_y]
                    },
                    1 => {
                        self.state = TraverseAreaIteratorState::Yield2x2 { index: index + 1 };
                        [self.base_offset_x + 1, self.base_offset_y]
                    },
                    2 => {
                        self.state = TraverseAreaIteratorState::Yield2x2 { index: index + 1 };
                        [self.base_offset_x, self.base_offset_y + 1]
                    },
                    _ => {
                        self.state = TraverseAreaIteratorState::Done;
                        [self.base_offset_x + 1, self.base_offset_y + 1]
                    },
                };
                Some([x, y])
            },
            TraverseAreaIteratorState::Yield3x3 { index } => {
                let [x, y] = match index {
                    // Сначала углы:
                    0 => {
                        self.state = TraverseAreaIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x, self.base_offset_y]
                    },
                    1 => {
                        self.state = TraverseAreaIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 2, self.base_offset_y]
                    },
                    2 => {
                        self.state = TraverseAreaIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x, self.base_offset_y + 2]
                    },
                    3 => {
                        self.state = TraverseAreaIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 2, self.base_offset_y + 2]
                    },
                    // Затем рёбра:
                    4 => {
                        self.state = TraverseAreaIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 1, self.base_offset_y]
                    },
                    5 => {
                        self.state = TraverseAreaIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 1, self.base_offset_y + 2]
                    },
                    6 => {
                        self.state = TraverseAreaIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x, self.base_offset_y + 1]
                    },
                    7 => {
                        self.state = TraverseAreaIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 2, self.base_offset_y + 1]
                    },
                    // Затем центр:
                    _ => {
                        self.state = TraverseAreaIteratorState::Done;
                        [self.base_offset_x + 1, self.base_offset_y + 1]
                    },
                };
                Some([x, y])
            },
            TraverseAreaIteratorState::YieldCorners { corner_index } => {
                let [x, y] = match corner_index {
                    0 => {
                        self.state = TraverseAreaIteratorState::YieldCorners {
                            corner_index: corner_index + 1
                        };
                        [self.base_offset_x, self.base_offset_y]
                    },
                    1 => {
                        self.state = TraverseAreaIteratorState::YieldCorners {
                            corner_index: corner_index + 1
                        };
                        [self.base_offset_x + self.width - 1, self.base_offset_y]
                    },
                    2 => {
                        self.state = TraverseAreaIteratorState::YieldCorners {
                            corner_index: corner_index + 1
                        };
                        [self.base_offset_x, self.base_offset_y + self.width - 1]
                    },
                    _ => {
                        let edge_width = self.width - 2;
                        let half_width = edge_width / 2;
                        self.state = TraverseAreaIteratorState::YieldEdges {
                            edge_width,
                            half_width,
                            edge_index: 0,
                            i: 0,
                        };
                        [
                            self.base_offset_x + self.width - 1,
                            self.base_offset_y + self.width - 1
                        ]
                    },
                };
                Some([x, y])
            },
            TraverseAreaIteratorState::YieldEdges { edge_width, half_width, edge_index, i } => {
                let [x, y] = match edge_index {
                    // Рёбра посещаются от краёв к центру:
                    0 => {
                        self.state = TraverseAreaIteratorState::YieldEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x + 1 + i,
                            self.base_offset_y
                        ]
                    },
                    1 => {
                        self.state = TraverseAreaIteratorState::YieldEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x + edge_width - i,
                            self.base_offset_y
                        ]
                    },
                    2 => {
                        self.state = TraverseAreaIteratorState::YieldEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x + 1 + i,
                            self.base_offset_y + self.width - 1
                        ]
                    },
                    3 => {
                        self.state = TraverseAreaIteratorState::YieldEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x + edge_width - i,
                            self.base_offset_y + self.width - 1
                        ]
                    },
                    4 => {
                        self.state = TraverseAreaIteratorState::YieldEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x,
                            self.base_offset_y + 1 + i
                        ]
                    },
                    5 => {
                        self.state = TraverseAreaIteratorState::YieldEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x,
                            self.base_offset_y + edge_width - i
                        ]
                    },
                    6 => {
                        self.state = TraverseAreaIteratorState::YieldEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x + self.width - 1,
                            self.base_offset_y + 1 + i
                        ]
                    },
                    _ => {
                        let p = [
                            self.base_offset_x + self.width - 1,
                            self.base_offset_y + edge_width - i
                        ];
                        if i < half_width.max(1) - 1 {
                            self.state = TraverseAreaIteratorState::YieldEdges {
                                edge_width,
                                half_width,
                                edge_index: 0,
                                i: i + 1,
                            };
                            p
                        } else {
                            if edge_width % 2 != 0 {
                                self.state = TraverseAreaIteratorState::YieldEdgeCenters {
                                    edge_width,
                                    half_width,
                                    edge_index: 0,
                                };
                                p
                            } else {
                                self.base_offset_x += 1;
                                self.base_offset_y += 1;
                                self.width = edge_width;
                                self.state = match edge_width {
                                    0 => TraverseAreaIteratorState::Done,
                                    1 => TraverseAreaIteratorState::YieldOne,
                                    2 => TraverseAreaIteratorState::Yield2x2 { index: 0 },
                                    3 => TraverseAreaIteratorState::Yield3x3 { index: 0 },
                                    _ => TraverseAreaIteratorState::YieldCorners { corner_index: 0 },
                                };
                                p
                            }
                        }
                    },
                };
                Some([x, y])
            }
            TraverseAreaIteratorState::YieldEdgeCenters { edge_width, half_width, edge_index } => {
                let [x, y] = match edge_index {
                    0 => {
                        self.state = TraverseAreaIteratorState::YieldEdgeCenters {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                        };
                        [
                            self.base_offset_x + half_width + 1,
                            self.base_offset_y
                        ]
                    },
                    1 => {
                        self.state = TraverseAreaIteratorState::YieldEdgeCenters {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                        };
                        [
                            self.base_offset_x + half_width + 1,
                            self.base_offset_y + self.width - 1
                        ]
                    },
                    2 => {
                        self.state = TraverseAreaIteratorState::YieldEdgeCenters {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                        };
                        [
                            self.base_offset_x,
                            self.base_offset_y + half_width + 1
                        ]
                    },
                    _ => {
                        let p = [
                            self.base_offset_x + self.width - 1,
                            self.base_offset_y + half_width + 1
                        ];
                        self.base_offset_x += 1;
                        self.base_offset_y += 1;
                        self.width = edge_width;
                        self.state = match edge_width {
                            0 => TraverseAreaIteratorState::Done,
                            1 => TraverseAreaIteratorState::YieldOne,
                            2 => TraverseAreaIteratorState::Yield2x2 { index: 0 },
                            3 => TraverseAreaIteratorState::Yield3x3 { index: 0 },
                            _ => TraverseAreaIteratorState::YieldCorners { corner_index: 0 },
                        };
                        p
                    }
                };
                Some([x, y])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::game_stage::grid_math::{traverse_area_inward};

    #[test]
    fn test_traverse_area_inward_iterator0() {
        let iterator = traverse_area_inward(0);
        let vec: Vec<[usize; 2]> = iterator.collect::<Vec<_>>();
        assert!(vec.is_empty());
    }

    #[test]
    fn test_traverse_area_inward_iterator1() {
        let iterator = traverse_area_inward(1);
        let vec: Vec<[usize; 2]> = iterator.collect::<Vec<_>>();
        assert_eq!(vec, vec![[0, 0]]);
    }

    #[test]
    fn test_traverse_area_inward_iterator2x2() {
        let iterator = traverse_area_inward(2);
        let vec = iterator.collect::<Vec<_>>();
        assert_eq!(vec, vec![[0, 0], [1, 0], [0, 1], [1, 1]]);
    }

    #[test]
    fn test_traverse_area_inward_iterator3x3() {
        let iterator = traverse_area_inward(3);
        let vec = iterator.collect::<Vec<_>>();
        assert_eq!(vec, vec![
            [0, 0], [2, 0], [0, 2], [2, 2],
            [1, 0], [1, 2], [0, 1], [2, 1],
            [1, 1]
        ]);
    }

    #[test]
    fn test_traverse_area_inward_iterator4x4_yields_all_coords_once() {
        let mut set = std::collections::HashSet::new();
        for j in 0..4 {
            for i in 0..4 {
                set.insert([i, j]);
            }
        }
        let iterator = traverse_area_inward(4);
        let vec = iterator.collect::<Vec<_>>();
        for &[x, y] in vec.iter() {
            assert!(set.remove(&[x, y]));
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_traverse_area_inward_iterator5x5_yields_all_coords_once() {
        let mut set = std::collections::HashSet::new();
        for j in 0..5 {
            for i in 0..5 {
                set.insert([i, j]);
            }
        }
        let iterator = traverse_area_inward(5);
        let vec = iterator.collect::<Vec<_>>();
        for &[x, y] in vec.iter() {
            assert!(set.remove(&[x, y]));
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_traverse_area_inward_iterator6x6_yields_all_coords_once() {
        let mut set = std::collections::HashSet::new();
        for j in 0..6 {
            for i in 0..6 {
                set.insert([i, j]);
            }
        }
        let iterator = traverse_area_inward(6);
        let vec = iterator.collect::<Vec<_>>();
        for &[x, y] in vec.iter() {
            assert!(set.remove(&[x, y]));
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_traverse_area_inward_iterator8x8_yields_all_coords_once() {
        let mut set = std::collections::HashSet::new();
        for j in 0..8 {
            for i in 0..8 {
                set.insert([i, j]);
            }
        }
        let iterator = traverse_area_inward(8);
        let vec = iterator.collect::<Vec<_>>();
        for &[x, y] in vec.iter() {
            assert!(set.remove(&[x, y]));
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_traverse_area_inward_iterator16x16_yields_all_coords_once() {
        let mut set = std::collections::HashSet::new();
        for j in 0..16 {
            for i in 0..16 {
                set.insert([i, j]);
            }
        }
        let iterator = traverse_area_inward(16);
        let vec = iterator.collect::<Vec<_>>();
        for &[x, y] in vec.iter() {
            assert!(set.remove(&[x, y]));
        }
        assert!(set.is_empty());
    }
}