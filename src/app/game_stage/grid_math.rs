use std::collections::HashSet;
use crate::app::editor_stage::widgets::EditableFloorData;
use crate::graphics::WallGraphicsTileGroup;

#[derive(Clone, Copy)]
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


/// Вспомогательная функция для обхода той же квадратной области,
/// что и `traverse_area_inward`, но в строго обратном порядке.
///
/// Итератор работает лениво: он не хранит координаты всей области.
/// Вместо этого он начинает с центральной
/// области и постепенно расширяется к внешним кольцам.
pub fn traverse_area_outward(width: usize) -> TraverseAreaOutwardIterator {
    let (base_offset, current_width, state) = match width {
        0 => (0, 0, TraverseAreaOutwardIteratorState::Done),
        1 => (0, 1, TraverseAreaOutwardIteratorState::YieldOne),
        _ if width % 2 == 0 => {
            let current_width = 2;
            (
                (width - current_width) / 2,
                current_width,
                TraverseAreaOutwardIteratorState::Yield2x2 { index: 0 },
            )
        },
        _ => {
            let current_width = 3;
            (
                (width - current_width) / 2,
                current_width,
                TraverseAreaOutwardIteratorState::Yield3x3 { index: 0 },
            )
        },
    };

    TraverseAreaOutwardIterator {
        base_offset_x: base_offset,
        base_offset_y: base_offset,
        width: current_width,
        max_width: width,
        state,
    }
}

/// Дуальный коллега TraverseAreaInwardIterator.
/// Написан с помощью ИИ, так что, возможно, стоит
/// провести его аудит и рефакторинг в дальнейшем
pub struct TraverseAreaOutwardIterator {
    base_offset_x: usize,
    base_offset_y: usize,
    width: usize,
    max_width: usize,
    state: TraverseAreaOutwardIteratorState,
}

#[derive(Clone, Copy)]
enum TraverseAreaOutwardIteratorState {
    Done,
    YieldOne,
    Yield2x2 { index: usize },
    Yield3x3 { index: usize },
    YieldRingEdgeCenters {
        edge_width: usize,
        half_width: usize,
        edge_index: usize,
    },
    YieldRingEdges {
        edge_width: usize,
        half_width: usize,
        edge_index: usize,
        i: usize,
    },
    YieldRingCorners { corner_index: usize },
}

impl TraverseAreaOutwardIterator {
    fn move_to_next_outer_ring(&mut self) {
        if self.width >= self.max_width {
            self.state = TraverseAreaOutwardIteratorState::Done;
            return;
        }

        self.base_offset_x -= 1;
        self.base_offset_y -= 1;
        self.width += 2;

        let edge_width = self.width - 2;
        let half_width = edge_width / 2;

        if edge_width % 2 != 0 {
            // Для нечётной внутренней ширины inward-итератор перед переходом
            // к вложенной области последними выдаёт центры рёбер. В обратном
            // порядке их нужно выдать первыми.
            self.state = TraverseAreaOutwardIteratorState::YieldRingEdgeCenters {
                edge_width,
                half_width,
                edge_index: 0,
            };
        } else {
            self.state = TraverseAreaOutwardIteratorState::YieldRingEdges {
                edge_width,
                half_width,
                edge_index: 0,
                i: half_width.max(1) - 1,
            };
        }
    }
}

impl Iterator for TraverseAreaOutwardIterator {
    type Item = [usize; 2];

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            TraverseAreaOutwardIteratorState::Done => None,
            TraverseAreaOutwardIteratorState::YieldOne => {
                let p = [self.base_offset_x, self.base_offset_y];
                self.move_to_next_outer_ring();
                Some(p)
            },
            TraverseAreaOutwardIteratorState::Yield2x2 { index } => {
                let p = match index {
                    0 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield2x2 { index: index + 1 };
                        [self.base_offset_x + 1, self.base_offset_y + 1]
                    },
                    1 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield2x2 { index: index + 1 };
                        [self.base_offset_x, self.base_offset_y + 1]
                    },
                    2 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield2x2 { index: index + 1 };
                        [self.base_offset_x + 1, self.base_offset_y]
                    },
                    _ => {
                        let p = [self.base_offset_x, self.base_offset_y];
                        self.move_to_next_outer_ring();
                        p
                    },
                };
                Some(p)
            },
            TraverseAreaOutwardIteratorState::Yield3x3 { index } => {
                let p = match index {
                    // Обратный порядок относительно Yield3x3 inward:
                    // центр, рёбра в обратном порядке, углы в обратном порядке.
                    0 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 1, self.base_offset_y + 1]
                    },
                    1 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 2, self.base_offset_y + 1]
                    },
                    2 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x, self.base_offset_y + 1]
                    },
                    3 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 1, self.base_offset_y + 2]
                    },
                    4 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 1, self.base_offset_y]
                    },
                    5 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 2, self.base_offset_y + 2]
                    },
                    6 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x, self.base_offset_y + 2]
                    },
                    7 => {
                        self.state = TraverseAreaOutwardIteratorState::Yield3x3 { index: index + 1 };
                        [self.base_offset_x + 2, self.base_offset_y]
                    },
                    _ => {
                        let p = [self.base_offset_x, self.base_offset_y];
                        self.move_to_next_outer_ring();
                        p
                    },
                };
                Some(p)
            },
            TraverseAreaOutwardIteratorState::YieldRingEdgeCenters { edge_width, half_width, edge_index } => {
                let p = match edge_index {
                    0 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdgeCenters {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                        };
                        [
                            self.base_offset_x + self.width - 1,
                            self.base_offset_y + half_width + 1,
                        ]
                    },
                    1 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdgeCenters {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                        };
                        [
                            self.base_offset_x,
                            self.base_offset_y + half_width + 1,
                        ]
                    },
                    2 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdgeCenters {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                        };
                        [
                            self.base_offset_x + half_width + 1,
                            self.base_offset_y + self.width - 1,
                        ]
                    },
                    _ => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdges {
                            edge_width,
                            half_width,
                            edge_index: 0,
                            i: half_width.max(1) - 1,
                        };
                        [
                            self.base_offset_x + half_width + 1,
                            self.base_offset_y,
                        ]
                    },
                };
                Some(p)
            },
            TraverseAreaOutwardIteratorState::YieldRingEdges { edge_width, half_width, edge_index, i } => {
                let p = match edge_index {
                    // Обратный порядок относительно блока YieldEdges inward.
                    // Сначала идём по i от центра к краям, а внутри одного i
                    // отдаём 8 точек в обратном порядке.
                    0 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x + self.width - 1,
                            self.base_offset_y + edge_width - i,
                        ]
                    },
                    1 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x + self.width - 1,
                            self.base_offset_y + 1 + i,
                        ]
                    },
                    2 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x,
                            self.base_offset_y + edge_width - i,
                        ]
                    },
                    3 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x,
                            self.base_offset_y + 1 + i,
                        ]
                    },
                    4 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x + edge_width - i,
                            self.base_offset_y + self.width - 1,
                        ]
                    },
                    5 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x + 1 + i,
                            self.base_offset_y + self.width - 1,
                        ]
                    },
                    6 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingEdges {
                            edge_width,
                            half_width,
                            edge_index: edge_index + 1,
                            i,
                        };
                        [
                            self.base_offset_x + edge_width - i,
                            self.base_offset_y,
                        ]
                    },
                    _ => {
                        let p = [
                            self.base_offset_x + 1 + i,
                            self.base_offset_y,
                        ];
                        if i > 0 {
                            self.state = TraverseAreaOutwardIteratorState::YieldRingEdges {
                                edge_width,
                                half_width,
                                edge_index: 0,
                                i: i - 1,
                            };
                        } else {
                            self.state = TraverseAreaOutwardIteratorState::YieldRingCorners {
                                corner_index: 0,
                            };
                        }
                        p
                    },
                };
                Some(p)
            },
            TraverseAreaOutwardIteratorState::YieldRingCorners { corner_index } => {
                let p = match corner_index {
                    0 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingCorners {
                            corner_index: corner_index + 1,
                        };
                        [
                            self.base_offset_x + self.width - 1,
                            self.base_offset_y + self.width - 1,
                        ]
                    },
                    1 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingCorners {
                            corner_index: corner_index + 1,
                        };
                        [
                            self.base_offset_x,
                            self.base_offset_y + self.width - 1,
                        ]
                    },
                    2 => {
                        self.state = TraverseAreaOutwardIteratorState::YieldRingCorners {
                            corner_index: corner_index + 1,
                        };
                        [
                            self.base_offset_x + self.width - 1,
                            self.base_offset_y,
                        ]
                    },
                    _ => {
                        let p = [self.base_offset_x, self.base_offset_y];
                        self.move_to_next_outer_ring();
                        p
                    },
                };
                Some(p)
            },
        }
    }
}

pub fn get_island_mapping(data: &impl EditableFloorData) ->(usize, Vec<Option<usize>>) {
    struct IslandNode {
        color: usize,
        neighbors: Vec<usize>,
        propagated: bool,
    }

    let width = data.width();
    let height = data.height();

    let mut island_nodes = Vec::new();
    let mut island_map: Vec<Option<usize>> = vec![None; width * height];
    let mut roots = HashSet::new();

    // Шаг 1: Собираем узлы островов и соединяем их с их
    // верхними соседями. Важно: связь односторонняя,
    // таким образом мы формируем Directed Acyclic Graph (DAG).
    // Мы попутно так же соберём набор корневых узлов,
    // и каждый узел по итогу станет отдельным островом.
    'rows: for y in 0..height {
        let mut x = 0;
        'series: loop {
            while x < width && !data.get_wall_data([x, y]).eq(&WallGraphicsTileGroup::None) {
                x += 1;
                if x == width {
                    continue 'rows;
                }
            }

            let color = island_nodes.len();
            let island = IslandNode {
                color,
                neighbors: Vec::with_capacity(16),
                propagated: false,
            };
            island_nodes.push(island);
            roots.insert(color);
            island_map[y * width + x] = Some(color);

            if y > 0
                && let Some(id) = island_map[(y - 1) * width + x]
                && !island_nodes[color].neighbors.contains(&id)
            {
                island_nodes[color].neighbors.push(id);
                roots.remove(&id);
            }

            loop {
                x += 1;
                if x == width {
                    continue 'rows;
                }
                if !data.get_wall_data([x, y]).eq(&WallGraphicsTileGroup::None) {
                    continue 'series;
                }

                island_map[y * width + x] = Some(color);
                if y > 0
                    && let Some(id) = island_map[(y - 1) * width + x]
                    && !island_nodes[color].neighbors.contains(&id)
                {
                    island_nodes[color].neighbors.push(id);
                    roots.remove(&id);
                }
            }
        }
    }

    // Шаг 2: Распространяем цвет узла острова на его соседей
    // по графу, тем самым соединяем острова
    let mut queue = Vec::new();
    for root in roots.iter() {
        queue.clear();
        let mut color = island_nodes[*root].color;
        queue.push(*root);

        let mut i = 0;
        while i < queue.len() {
            let neighbor = queue[i];
            if island_nodes[neighbor].propagated {
                if island_nodes[neighbor].color != color {
                    // Возможна ситуация, где один остров имеет несколько корней.
                    // Мы берём цвет первого корня и распространяем его на остальных.
                    color = island_nodes[neighbor].color;
                    for j in (0..i).rev() {
                        let neighbor = queue[j];
                        island_nodes[neighbor].color = color;
                    }
                } else {
                    i += 1;
                    continue;
                }
            }
            island_nodes[neighbor].propagated = true;
            island_nodes[neighbor].color = color;
            queue.extend(island_nodes[neighbor].neighbors.iter().copied());
            i += 1;
        }
    }

    // Шаг 3: Обновляем карту финальными цветами,
    // после чего возвращаем её, так как это именно тот результат, который ожидается.
    let mut set = HashSet::new();
    let mut result_map = vec![None; width * height];
    for (i, r) in island_map.drain(..).enumerate() {
        let Some(color) = r else { continue; };
        let color = island_nodes[color].color;
        result_map[i] = Some(color);
        set.insert(color);
    }
    (set.len(), result_map)
}


#[cfg(test)]
mod tests {
    use crate::app::game_stage::grid_math::{traverse_area_inward, traverse_area_outward};

    fn assert_next_eq<I>(iterator: &mut I, expected: [usize; 2])
    where
        I: Iterator<Item = [usize; 2]>,
    {
        assert_eq!(iterator.next(), Some(expected));
    }

    fn assert_yields_all_coords_once<I>(iterator: I, width: usize)
    where
        I: Iterator<Item = [usize; 2]>,
    {
        let mut set = std::collections::HashSet::new();
        for y in 0..width {
            for x in 0..width {
                set.insert([x, y]);
            }
        }

        let mut count = 0;
        for p in iterator {
            assert!(set.remove(&p), "unexpected or duplicate coord: {:?}", p);
            count += 1;
        }

        assert_eq!(count, width * width);
        assert!(set.is_empty());
    }

    fn assert_outward_is_inward_reversed(width: usize) {
        let inward: Vec<[usize; 2]> = traverse_area_inward(width).collect::<Vec<_>>();
        let mut outward: Vec<[usize; 2]> = traverse_area_outward(width).collect::<Vec<_>>();
        outward = outward.iter().copied().rev().collect::<Vec<_>>();
        assert_eq!(inward, outward)
    }

    #[test]
    fn test_traverse_area_inward_iterator0() {
        let mut iterator = traverse_area_inward(0);
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_traverse_area_inward_iterator1() {
        let mut iterator = traverse_area_inward(1);
        assert_next_eq(&mut iterator, [0, 0]);
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_traverse_area_inward_iterator2x2() {
        let mut iterator = traverse_area_inward(2);
        assert_next_eq(&mut iterator, [0, 0]);
        assert_next_eq(&mut iterator, [1, 0]);
        assert_next_eq(&mut iterator, [0, 1]);
        assert_next_eq(&mut iterator, [1, 1]);
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_traverse_area_inward_iterator3x3() {
        let mut iterator = traverse_area_inward(3);
        assert_next_eq(&mut iterator, [0, 0]);
        assert_next_eq(&mut iterator, [2, 0]);
        assert_next_eq(&mut iterator, [0, 2]);
        assert_next_eq(&mut iterator, [2, 2]);
        assert_next_eq(&mut iterator, [1, 0]);
        assert_next_eq(&mut iterator, [1, 2]);
        assert_next_eq(&mut iterator, [0, 1]);
        assert_next_eq(&mut iterator, [2, 1]);
        assert_next_eq(&mut iterator, [1, 1]);
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_traverse_area_inward_iterator4x4_yields_all_coords_once() {
        assert_yields_all_coords_once(traverse_area_inward(4), 4);
    }

    #[test]
    fn test_traverse_area_inward_iterator5x5_yields_all_coords_once() {
        assert_yields_all_coords_once(traverse_area_inward(5), 5);
    }

    #[test]
    fn test_traverse_area_inward_iterator6x6_yields_all_coords_once() {
        assert_yields_all_coords_once(traverse_area_inward(6), 6);
    }

    #[test]
    fn test_traverse_area_inward_iterator8x8_yields_all_coords_once() {
        assert_yields_all_coords_once(traverse_area_inward(8), 8);
    }

    #[test]
    fn test_traverse_area_inward_iterator16x16_yields_all_coords_once() {
        assert_yields_all_coords_once(traverse_area_inward(16), 16);
    }

    #[test]
    fn test_traverse_area_outward_iterator0() {
        let mut iterator = traverse_area_outward(0);
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_traverse_area_outward_iterator1() {
        let mut iterator = traverse_area_outward(1);
        assert_next_eq(&mut iterator, [0, 0]);
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_traverse_area_outward_iterator2x2() {
        let mut iterator = traverse_area_outward(2);
        assert_next_eq(&mut iterator, [1, 1]);
        assert_next_eq(&mut iterator, [0, 1]);
        assert_next_eq(&mut iterator, [1, 0]);
        assert_next_eq(&mut iterator, [0, 0]);
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_traverse_area_outward_iterator3x3() {
        let mut iterator = traverse_area_outward(3);
        assert_next_eq(&mut iterator, [1, 1]);
        assert_next_eq(&mut iterator, [2, 1]);
        assert_next_eq(&mut iterator, [0, 1]);
        assert_next_eq(&mut iterator, [1, 2]);
        assert_next_eq(&mut iterator, [1, 0]);
        assert_next_eq(&mut iterator, [2, 2]);
        assert_next_eq(&mut iterator, [0, 2]);
        assert_next_eq(&mut iterator, [2, 0]);
        assert_next_eq(&mut iterator, [0, 0]);
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_traverse_area_outward_iterator_yields_all_coords_once() {
        for width in 0..=16 {
            assert_yields_all_coords_once(traverse_area_outward(width), width);
        }
    }

    #[test]
    fn test_traverse_area_outward_iterator_is_inward_reversed() {
        for width in 0..=16 {
            assert_outward_is_inward_reversed(width);
        }
    }
}
