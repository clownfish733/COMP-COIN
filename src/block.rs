pub struct Block{
    height: usize,
}

impl Block{
    pub fn new(height: usize) -> Self{
        Self{
            height
        }
    }

    pub fn get_height(&self) -> usize{
        self.height
    }
}