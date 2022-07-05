#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn rev(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
    pub fn rotate_cw(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }
    pub fn rotate_ccw(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }
    pub fn all() -> [Direction; 4] {
        [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ]
    }
    pub fn angle(&self) -> f64 {
        match self {
            Direction::Up => 0.0,
            Direction::Down => std::f64::consts::PI,
            Direction::Left => std::f64::consts::PI * -0.5,
            Direction::Right => std::f64::consts::PI * 0.5,
        }
    }
}

impl Into<(i32, i32)> for Direction {
    fn into(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Point { active: bool, marked: u32 },
    Cable { active: bool, direction: Direction },
    And { active: bool, direction: Direction },
    Not { active: bool, direction: Direction },
    Tee { active: bool, direction: Direction },
}

impl Cell {
    pub fn is_active(&self) -> bool {
        match self {
            Cell::Empty => false,
            Cell::Cable { active, .. }
            | Cell::And { active, .. }
            | Cell::Not { active, .. }
            | Cell::Tee { active, .. }
            | Cell::Point { active, .. } => *active,
        }
    }
    pub fn direction(&self) -> Option<Direction> {
        match self {
            Self::Empty | Self::Point { .. } => None,
            Self::Cable { direction, .. }
            | Self::And { direction, .. }
            | Self::Not { direction, .. }
            | Self::Tee { direction, .. } => Some(*direction),
        }
    }
    pub fn direction_mut(&mut self) -> Option<&mut Direction> {
        match self {
            Self::Empty | Self::Point { .. } => None,
            Self::Cable { direction, .. }
            | Self::And { direction, .. }
            | Self::Not { direction, .. }
            | Self::Tee { direction, .. } => Some(direction),
        }
    }
    fn signal_in_direction(&self, dir: Direction) -> bool {
        self.is_active() && {
            match self {
                Cell::Empty => unreachable!(),
                Cell::Point { .. } => true,
                Cell::Tee { direction, .. } => {
                    // is orthogonal
                    *direction != dir && direction.rev() != dir
                }
                Cell::Cable { direction, .. }
                | Cell::And { direction, .. }
                | Cell::Not { direction, .. } => *direction == dir,
            }
        }
    }
    pub fn set(&mut self, signal: bool) {
        match self {
            Cell::Empty => {}
            Cell::Cable { active, .. }
            | Cell::And { active, .. }
            | Cell::Not { active, .. }
            | Cell::Tee { active, .. }
            | Cell::Point { active, .. } => *active = signal,
        }
    }
    pub fn rotate(&mut self) {
        match self {
            Cell::Empty | Cell::Point { .. } => {}
            Cell::Cable { direction, .. }
            | Cell::And { direction, .. }
            | Cell::Not { direction, .. }
            | Cell::Tee { direction, .. } => *direction = direction.rotate_cw(),
        }
    }
}

impl From<Cell> for u8 {
    fn from(cell: Cell) -> Self {
        let kind: u8 = match cell {
            Cell::Empty => 0,
            Cell::Cable { .. } => 1,
            Cell::And { .. } => 2,
            Cell::Not { .. } => 3,
            Cell::Tee { .. } => 4,
            Cell::Point { .. } => 5,
        };
        let dir: u8 = match cell.direction().unwrap_or(Direction::Up) {
            Direction::Up => 0,
            Direction::Right => 1,
            Direction::Down => 2,
            Direction::Left => 3,
        };
        let active = cell.is_active() as u8;
        kind << 3 | dir << 1 | active
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CellParseError {
    DirectionInvalid(u8, u8),
    KindInvalid(u8, u8),
}

impl TryFrom<u8> for Cell {
    type Error = CellParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let kind = (value & 0b111000) >> 3;
        let active = (value & 0b1) == 1;
        if kind == 5 {
            return Ok(Cell::Point { active, marked: 0 });
        }
        let direction = (value & 0b110) >> 1;
        let direction = match direction {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            n => return Err(CellParseError::DirectionInvalid(value, n)),
        };
        Ok(match kind {
            0 => Cell::Empty,
            1 => Cell::Cable { active, direction },
            2 => Cell::And { active, direction },
            3 => Cell::Not { active, direction },
            4 => Cell::Tee { active, direction },
            5 => unreachable!(),
            n => return Err(CellParseError::KindInvalid(value, n)),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    next: Vec<Cell>,
    pub marker: u32,
}

pub struct GridIterator<'a> {
    inner: <&'a Vec<Cell> as IntoIterator>::IntoIter,
    i: usize,
    width: usize,
}

impl<'a> Iterator for GridIterator<'a> {
    type Item = ([usize; 2], &'a Cell);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|cell| {
            let res = ([self.i % self.width, self.i / self.width], cell);
            self.i += 1;
            res
        })
    }
}

impl<'a> IntoIterator for &'a Grid {
    type Item = <GridIterator<'a> as Iterator>::Item;
    type IntoIter = GridIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        GridIterator {
            width: self.width,
            inner: (&self.cells).into_iter(),
            i: 0,
        }
    }
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells: Vec<Cell> = (0..width * height).map(|_| Cell::Empty).collect();
        Self {
            width,
            next: cells.clone(),
            cells,
            height,
            marker: 1,
        }
    }
    fn mark(
        &mut self,
        marker: u32,
        x: i32,
        y: i32,
        f: &mut Option<&mut impl FnMut(&mut Cell)>,
    ) -> bool {
        match &mut self[[x, y]] {
            Cell::Point { marked, .. } => {
                if *marked == marker {
                    return false;
                }
                *marked = marker;
                if x > 0 {
                    self.mark(marker, x - 1, y, f);
                }
                if x + 1 < self.width as i32 {
                    self.mark(marker, x + 1, y, f);
                }
                if y > 0 {
                    self.mark(marker, x, y - 1, f);
                }
                if y + 1 < self.height as i32 {
                    self.mark(marker, x, y + 1, f);
                }
                if let Some(f) = f {
                    f(&mut self[[x, y]]);
                }
                true
            }
            _ => false,
        }
    }
    pub fn simulate(self: &mut Box<Self>) {
        for x in 0..self.width as i32 {
            for y in 0..self.height as i32 {
                if self.mark(self.marker, x, y, &mut Some(&mut |c| c.set(false))) {
                    self.marker += 1;
                }
            }
        }
        for x in 0..self.width as i32 {
            for y in 0..self.height as i32 {
                match self[[x, y]] {
                    Cell::Point { .. } => {}
                    _ => continue,
                };
                if self[[x, y]].is_active() {
                    continue;
                }
                for dir in Direction::all() {
                    let (ox, oy): (i32, i32) = dir.into();
                    let nx = x + ox;
                    let ny = y + oy;
                    if nx < 0 || nx >= self.width as i32 || ny < 0 || ny >= self.height as i32 {
                        continue;
                    }
                    if self[[nx, ny]].signal_in_direction(dir.rev()) {
                        if self.mark(self.marker, x, y, &mut Some(&mut |c| c.set(true))) {
                            self.marker += 1;
                        }
                        break;
                    }
                }
            }
        }
        self.next.as_mut_slice().copy_from_slice(&self.cells);
        for x in 0..self.width {
            for y in 0..self.height {
                match self.cells[x + y * self.width] {
                    Cell::Empty | Cell::Point { .. } => {}
                    Cell::Cable { direction, .. } => {
                        let active = {
                            let (ox, oy): (i32, i32) = direction.rev().into();
                            let x = x as i32 + ox;
                            let y = y as i32 + oy;
                            if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
                                false
                            } else {
                                self[[x, y]].signal_in_direction(direction)
                            }
                        };
                        self.next[x + y * self.width].set(active);
                    }
                    Cell::And { direction, .. } => {
                        let active = [direction.rotate_cw(), direction.rotate_ccw()]
                            .into_iter()
                            .all(|dir| {
                                let (ox, oy): (i32, i32) = dir.into();
                                let x = x as i32 + ox;
                                let y = y as i32 + oy;
                                if x < 0
                                    || x >= self.width as i32
                                    || y < 0
                                    || y >= self.height as i32
                                {
                                    return false;
                                }
                                let neighbour = &self[[x, y]];
                                neighbour.signal_in_direction(dir.rev())
                            });
                        self.next[x + y * self.width].set(active);
                    }
                    Cell::Not { direction, .. } => {
                        let (ox, oy): (i32, i32) = direction.rev().into();
                        let nx = x as i32 + ox;
                        let ny = y as i32 + oy;
                        let signal = nx >= 0
                            && nx < self.width as i32
                            && ny >= 0
                            && ny < self.height as i32
                            && self[[nx, ny]].signal_in_direction(direction);
                        self.next[x + y * self.width].set(!signal)
                    }
                    Cell::Tee { direction, .. } => {
                        let (ox, oy): (i32, i32) = direction.rev().into();
                        let nx = x as i32 + ox;
                        let ny = y as i32 + oy;
                        let signal = nx >= 0
                            && nx < self.width as i32
                            && ny >= 0
                            && ny < self.height as i32
                            && self[[nx, ny]].signal_in_direction(direction);
                        self.next[x + y * self.width].set(signal)
                    }
                }
            }
        }
        std::mem::swap(&mut self.cells, &mut self.next)
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x > self.width {
            return None;
        }
        self.cells.get(x + y * self.width)
    }
    pub fn serialize(&self) -> impl Iterator<Item = u8> + '_ {
        (self.height as u32)
            .to_le_bytes()
            .into_iter()
            .chain((self.width as u32).to_le_bytes())
            .chain(self.cells.iter().map(|c| u8::from(*c)))
    }
    pub fn deserialize(mut bytes: impl Iterator<Item = u8>) -> Result<Self, GridParseError> {
        let mut f = || -> Option<(u32, u32)> {
            let width =
                u32::from_le_bytes([bytes.next()?, bytes.next()?, bytes.next()?, bytes.next()?]);
            let height =
                u32::from_le_bytes([bytes.next()?, bytes.next()?, bytes.next()?, bytes.next()?]);
            Some((width, height))
        };
        let (width, height) = f().ok_or(GridParseError::InputTooShort)?;
        let width = width as usize;
        let height = height as usize;
        let cells: Vec<Cell> = bytes
            .take(width * height)
            .map(Cell::try_from)
            .collect::<Result<Vec<Cell>, CellParseError>>()?;
        if cells.len() != width * height {
            return Err(GridParseError::InputTooShort);
        }
        Ok(Self {
            width,
            height,
            next: cells.clone(),
            cells,
            marker: 1,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GridParseError {
    InputTooShort,
    CellParseError(CellParseError),
    MoreCellsExpected { expected: u32 },
}

impl From<CellParseError> for GridParseError {
    fn from(e: CellParseError) -> Self {
        Self::CellParseError(e)
    }
}

impl std::ops::Index<[usize; 2]> for Grid {
    type Output = Cell;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        &self.cells[index[0] + index[1] * self.width]
    }
}

impl std::ops::IndexMut<[usize; 2]> for Grid {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        &mut self.cells[index[0] + index[1] * self.width]
    }
}

impl std::ops::Index<[i32; 2]> for Grid {
    type Output = Cell;

    fn index(&self, index: [i32; 2]) -> &Self::Output {
        &self.cells[index[0] as usize + index[1] as usize * self.width]
    }
}

impl std::ops::IndexMut<[i32; 2]> for Grid {
    fn index_mut(&mut self, index: [i32; 2]) -> &mut Self::Output {
        &mut self.cells[index[0] as usize + index[1] as usize * self.width]
    }
}

#[test]
fn grid_serialize() {
    let mut grid = Grid::new(10, 10);
    grid[[0, 5]] = Cell::And {
        active: true,
        direction: Direction::Left,
    };
    assert_eq!(grid, Grid::deserialize(grid.serialize()).unwrap());
}
