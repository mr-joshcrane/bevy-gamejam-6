struct Block {
    top_left_corner: Vec2,
    normalised_block_size_x: f32,
    normalised_block_size_y: f32,
    grid_cell_size: f32,
}

impl Block {
    pub fn is_neighbour(&self, other: &Block) -> bool {
        let x_distance = (self.top_left_corner.x - other.top_left_corner.x).abs();
        let y_distance = (self.top_left_corner.y - other.top_left_corner.y).abs();

        let horizontal_neighbour = x_distance == self.normalised_block_size_x && y_distance == 0.0;
        let vertical_neighbour = y_distance == self.normalised_block_size_y && x_distance == 0.0;

        horizontal_neighbour || vertical_neighbour
    }
}

#[test]
fn are_blocks_neighbours_horizontal() {
    let block1 = Block {
        top_left_corner: Vec2::new(0.0, 1.0),
        normalised_block_size_x: 1,
        normalised_block_size_y: 1,
        grid_cell_size: 16.0,
    };
    let block2 = Block {
        top_left_corner: Vec2::new(1.0, 1.0),
        normalised_block_size_x: 1,
        normalised_block_size_y: 1,
        grid_cell_size: 16.0,
    };
    let block3 = Block {
        top_left_corner: Vec2::new(1.0, 2.0),
        normalised_block_size_x: 1,
        normalised_block_size_y: 1,
        grid_cell_size: 16.0,
    };
    assert(block1.is_neighbour(&block2));
    assert!(!block1.is_neighbour(&block3));
}

#[test]
fn are_blocks_neighbours_vertical() {
    let block1 = Block {
        top_left_corner: Vec2::new(0.0, 1.0),
        normalised_block_size_x: 1,
        normalised_block_size_y: 1,
        grid_cell_size: 16.0,
    };
    let block2 = Block {
        top_left_corner: Vec2::new(0.0, 2.0),
        normalised_block_size_x: 1,
        normalised_block_size_y: 1,
        grid_cell_size: 16.0,
    };
    let block3 = Block {
        top_left_corner: Vec2::new(0.0, 2.0),
        normalised_block_size_x: 1,
        normalised_block_size_y: 1,
        grid_cell_size: 16.0,
    };
    assert!(block1.is_neighbour(&block2));
    assert!(!block1.is_neighbour(&block3));
}
