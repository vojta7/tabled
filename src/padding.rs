//! This module contains a [Padding] setting of a cell on a [Table].
//!
//! # Example
//!
//! ```
//! use tabled::{TableIteratorExt, Padding, Style, Modify, object::Cell};
//!
//! let data = "2022".chars();
//!
//! let table = data.table()
//!     .with(Style::modern())
//!     .with(Modify::new(Cell(2, 0)).with(Padding::new(1, 1, 2, 2)))
//!     .to_string();
//!
//! assert_eq!(
//!     table,
//!     concat!(
//!         "┌──────┐\n",
//!         "│ char │\n",
//!         "├──────┤\n",
//!         "│  2   │\n",
//!         "├──────┤\n",
//!         "│      │\n",
//!         "│      │\n",
//!         "│  0   │\n",
//!         "│      │\n",
//!         "│      │\n",
//!         "├──────┤\n",
//!         "│  2   │\n",
//!         "├──────┤\n",
//!         "│  2   │\n",
//!         "└──────┘\n",
//!     ),
//! );
//! ```
//!
//! [Table]: crate::Table

use crate::CellOption;
use papergrid::{Entity, Grid, Indent, Settings};

/// Padding is responsible for a left/right/top/bottom inner indent of a particular cell.
///
/// ```rust,no_run
///   # use tabled::{Style, Padding, object::Rows, Table, Modify};
///   # let data: Vec<&'static str> = Vec::new();
///     let table = Table::new(&data).with(Modify::new(Rows::single(0)).with(Padding::new(0, 0, 1, 1).set_fill('>', '<', '^', 'V')));
/// ```
pub struct Padding(papergrid::Padding);

impl Padding {
    /// Construct's an Padding object.
    ///
    /// It uses space(' ') as a default fill character.
    /// To set a custom character you can use [Self::set_fill] function.
    pub fn new(left: usize, right: usize, top: usize, bottom: usize) -> Self {
        Self(papergrid::Padding {
            top: Indent::spaced(top),
            bottom: Indent::spaced(bottom),
            left: Indent::spaced(left),
            right: Indent::spaced(right),
        })
    }

    /// Construct's an Padding object with all sides set to 0.
    ///
    /// It uses space(' ') as a default fill character.
    /// To set a custom character you can use [Self::set_fill] function.
    pub fn zero() -> Self {
        let indent = Indent::spaced(0);
        Self(papergrid::Padding {
            top: indent,
            bottom: indent,
            left: indent,
            right: indent,
        })
    }

    /// The function, sets a characters for the padding on an each side.
    pub fn set_fill(mut self, left: char, right: char, top: char, bottom: char) -> Self {
        self.0.left.fill = left;
        self.0.right.fill = right;
        self.0.top.fill = top;
        self.0.bottom.fill = bottom;
        self
    }
}

impl CellOption for Padding {
    fn change_cell(&mut self, grid: &mut Grid, row: usize, column: usize) {
        grid.set(
            Entity::Cell(row, column),
            Settings::new().padding(self.0.left, self.0.right, self.0.top, self.0.bottom),
        )
    }
}
