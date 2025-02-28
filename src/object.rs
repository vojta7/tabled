//! This module contains a list of primitives that implement a [Object] trait.
//! They help to locate a nessesary segment on a [Table].
//!
//! [Table]: crate::Table

use std::{
    collections::BTreeSet,
    ops::{Add, Bound, RangeBounds, RangeFull, Sub},
};

/// Object helps to locate a nessesary part of a [Table].
///
/// [Table]: crate::Table
pub trait Object: Sized {
    /// Cells returns a set of cordinates of cells
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)>;

    /// Combines cells.
    /// It doesn't repeat cells.
    fn and<O: Object>(self, rhs: O) -> Combination<Self, O> {
        Combination {
            lhs: self,
            rhs,
            combinator: combine_cells,
        }
    }

    /// Excludes rhs cells from this cells.
    fn not<O: Object>(self, rhs: O) -> Combination<Self, O> {
        Combination {
            lhs: self,
            rhs,
            combinator: remove_cells,
        }
    }
}

/// Segment represents a sub table of [Table].
///
/// [Table]: crate::Table
pub struct Segment<C, R> {
    columns: C,
    rows: R,
}

impl Segment<RangeFull, RangeFull> {
    /// Returns a full table segment.
    ///
    /// The same as [Full].
    pub fn all() -> Self {
        Self::new(.., ..)
    }
}

impl<C, R> Segment<C, R>
where
    C: RangeBounds<usize>,
    R: RangeBounds<usize>,
{
    /// This function builds a [Segment].
    pub fn new(rows: R, columns: C) -> Self {
        Self { columns, rows }
    }
}

impl<C, R> Object for Segment<C, R>
where
    C: RangeBounds<usize>,
    R: RangeBounds<usize>,
{
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        let (rows_start, rows_end) =
            bounds_to_usize(self.rows.start_bound(), self.rows.end_bound(), count_rows);

        let (columns_start, columns_end) = bounds_to_usize(
            self.columns.start_bound(),
            self.columns.end_bound(),
            count_columns,
        );

        let mut cells = Vec::new();
        (rows_start..rows_end)
            .for_each(|row| (columns_start..columns_end).for_each(|col| cells.push((row, col))));

        cells
    }
}

/// Frame includes cells which are on the edges of each side.
/// Therefore it's [Object] implementation returns a subset of cells which are present in frame.
pub struct Frame;

impl Object for Frame {
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();

        if count_rows > 0 {
            (0..count_columns).for_each(|col| {
                cells.push((0, col));
            });

            (0..count_columns).for_each(|col| {
                cells.push((count_rows - 1, col));
            });
        }

        if count_columns > 0 {
            (0..count_rows).for_each(|row| {
                cells.push((row, 0));
            });

            (0..count_rows).for_each(|row| {
                cells.push((row, count_columns - 1));
            });
        }

        cells
    }
}

/// FirstRow represents the first row of a [Table].
/// It's often contains headers data.
///
/// [Table]: crate::Table
pub struct FirstRow;

impl Object for FirstRow {
    fn cells(&self, _: usize, count_columns: usize) -> Vec<(usize, usize)> {
        (0..count_columns).map(|column| (0, column)).collect()
    }
}

impl Add<usize> for FirstRow {
    type Output = Row;

    fn add(self, rhs: usize) -> Self::Output {
        Row { index: rhs }
    }
}

/// LastRow represents the last row of a [Table].
///
/// [Table]: crate::Table
pub struct LastRow;

impl Object for LastRow {
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        let row = if count_rows == 0 { 0 } else { count_rows - 1 };
        (0..count_columns).map(|column| (row, column)).collect()
    }
}

impl Sub<usize> for LastRow {
    type Output = LastRowOffset;

    fn sub(self, rhs: usize) -> Self::Output {
        LastRowOffset { offset: rhs }
    }
}

/// A row which is located by an offset from the first row.
pub struct Row {
    index: usize,
}

impl Object for Row {
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        if self.index >= count_rows {
            return Vec::new();
        }

        (0..count_columns)
            .map(|column| (self.index, column))
            .collect()
    }
}

/// A row which is located by an offset from the last row.
pub struct LastRowOffset {
    offset: usize,
}

impl Object for LastRowOffset {
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        let row = if count_rows == 0 { 0 } else { count_rows - 1 };
        if self.offset > row {
            return Vec::new();
        }

        let row = row - self.offset;
        (0..count_columns).map(|column| (row, column)).collect()
    }
}

/// Full represents all cells on a [Table].
///
/// [Table]: crate::Table
#[deprecated(
    note = "It was replaced because the name is not descriptive. Use Segment::all() instead."
)]
pub struct Full;

#[allow(deprecated)]
impl Object for Full {
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        Segment::new(.., ..).cells(count_rows, count_columns)
    }
}

/// Row denotes a set of cells on given rows on a [Table].
///
/// [Table]: crate::Table
pub struct Rows<R> {
    range: R,
}

impl<R> Rows<R>
where
    R: RangeBounds<usize>,
{
    /// Returns a new instance of [Rows] for a range of rows.
    ///
    /// If the boundries are exeeded it may panic.
    pub fn new(range: R) -> Self {
        Self { range }
    }
}

impl Rows<()> {
    /// Returns a new instance of [Rows] with a single row.
    ///
    /// If the boundries are exeeded it may panic.
    pub fn single(index: usize) -> Row {
        Row { index }
    }

    /// Returns a first row [Object].
    ///
    /// If the table has 0 rows returns an empty set of cells.
    pub fn first() -> FirstRow {
        FirstRow
    }

    /// Returns a last row [Object].
    ///
    /// If the table has 0 rows returns an empty set of cells.
    pub fn last() -> LastRow {
        LastRow
    }
}

impl<R> Object for Rows<R>
where
    R: RangeBounds<usize>,
{
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        let (x, y) = bounds_to_usize(self.range.start_bound(), self.range.end_bound(), count_rows);

        (x..y)
            .map(|row| (0..count_columns).map(|column| (row, column)).collect())
            .collect::<Vec<Vec<_>>>()
            .concat()
    }
}

/// Column denotes a set of cells on given columns on a [Table].
///
/// [Table]: crate::Table
pub struct Columns<R> {
    range: R,
}

impl<R> Columns<R>
where
    R: RangeBounds<usize>,
{
    /// Returns a new instance of [Columns] for a range of columns.
    ///
    /// If the boundries are exeeded it may panic.
    pub fn new(range: R) -> Self {
        Self { range }
    }
}

impl Columns<()> {
    /// Returns a new instance of [Columns] for a single column.
    ///
    /// If the boundries are exeeded it may panic.
    pub fn single(index: usize) -> Column {
        Column(index)
    }

    /// Returns a new instance of [Columns] for a first column.
    ///
    /// If the boundries are exeeded the object will produce no cells.
    pub fn first() -> FirstColumn {
        FirstColumn
    }

    /// Returns a new instance of [Columns] for a last column.
    ///
    /// If the boundries are exeeded the object will produce no cells.
    pub fn last() -> LastColumn {
        LastColumn
    }
}

impl<R> Object for Columns<R>
where
    R: RangeBounds<usize>,
{
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        let (x, y) = bounds_to_usize(
            self.range.start_bound(),
            self.range.end_bound(),
            count_columns,
        );

        (x..y)
            .map(|column| (0..count_rows).map(|row| (row, column)).collect())
            .collect::<Vec<Vec<_>>>()
            .concat()
    }
}

/// FirstColumn represents the first column on a grid.
pub struct FirstColumn;

impl Object for FirstColumn {
    fn cells(&self, count_rows: usize, _: usize) -> Vec<(usize, usize)> {
        (0..count_rows).map(|row| (row, 0)).collect()
    }
}

impl Add<usize> for FirstColumn {
    type Output = Column;

    fn add(self, rhs: usize) -> Self::Output {
        Column(rhs)
    }
}

/// LastColumn represents the last column on a grid.
pub struct LastColumn;

impl Object for LastColumn {
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        let col = count_columns.saturating_sub(1);
        (0..count_rows).map(|row| (row, col)).collect()
    }
}

impl Sub<usize> for LastColumn {
    type Output = LastColumnOffset;

    fn sub(self, rhs: usize) -> Self::Output {
        LastColumnOffset { offset: rhs }
    }
}

/// Column represents a single column on a grid.
pub struct Column(usize);

impl Object for Column {
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        let col = self.0;
        if col >= count_columns {
            return Vec::new();
        }

        (0..count_rows).map(|row| (row, col)).collect()
    }
}

/// LastColumnOffset represents a single column on a grid indexed via offset from the last column.
pub struct LastColumnOffset {
    offset: usize,
}

impl Object for LastColumnOffset {
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        let col = count_columns.saturating_sub(1);
        if self.offset > col {
            return Vec::new();
        }

        let col = col - self.offset;
        (0..count_rows).map(|row| (row, col)).collect()
    }
}

/// Cell denotes a particular cell on a [Table].
///
/// [Table]: crate::Table
pub struct Cell(pub usize, pub usize);

impl Object for Cell {
    fn cells(&self, _: usize, _: usize) -> Vec<(usize, usize)> {
        vec![(self.0, self.1)]
    }
}

/// Combinator is a transformation function
type Combinator = fn(Vec<(usize, usize)>, Vec<(usize, usize)>) -> Vec<(usize, usize)>;

/// Combination struct used for chaning [Object]'s.
pub struct Combination<L, R> {
    lhs: L,
    rhs: R,
    combinator: Combinator,
}

impl<L, R> Object for Combination<L, R>
where
    L: Object,
    R: Object,
{
    fn cells(&self, count_rows: usize, count_columns: usize) -> Vec<(usize, usize)> {
        let l = self.lhs.cells(count_rows, count_columns);
        let r = self.rhs.cells(count_rows, count_columns);
        (self.combinator)(l, r)
    }
}

/// Combines 2 sets of cells into one.
///
/// Dublicates are removed from the output set.
fn combine_cells(lhs: Vec<(usize, usize)>, rhs: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    lhs.into_iter()
        .chain(rhs.into_iter())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Removes cells from fist set which are present in a second set.
fn remove_cells(lhs: Vec<(usize, usize)>, rhs: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    lhs.into_iter().filter(|l| !rhs.contains(l)).collect()
}

/// Converts a range bound to its indexes.
pub(crate) fn bounds_to_usize(
    left: Bound<&usize>,
    right: Bound<&usize>,
    count_elements: usize,
) -> (usize, usize) {
    match (left, right) {
        (Bound::Included(x), Bound::Included(y)) => (*x, y + 1),
        (Bound::Included(x), Bound::Excluded(y)) => (*x, *y),
        (Bound::Included(x), Bound::Unbounded) => (*x, count_elements),
        (Bound::Unbounded, Bound::Unbounded) => (0, count_elements),
        (Bound::Unbounded, Bound::Included(y)) => (0, y + 1),
        (Bound::Unbounded, Bound::Excluded(y)) => (0, *y),
        (Bound::Excluded(_), Bound::Unbounded)
        | (Bound::Excluded(_), Bound::Included(_))
        | (Bound::Excluded(_), Bound::Excluded(_)) => {
            unreachable!("A start bound can't be excluded")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_column_test() {
        assert_eq!((Columns::first()).cells(0, 0), vec![]);
        // is this correct?
        assert_eq!(
            (Columns::first()).cells(10, 0),
            vec![
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (4, 0),
                (5, 0),
                (6, 0),
                (7, 0),
                (8, 0),
                (9, 0)
            ]
        );
        assert_eq!((Columns::first()).cells(0, 10), vec![]);
        assert_eq!(
            (Columns::first()).cells(5, 2),
            vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        );
    }

    #[test]
    fn last_column_test() {
        assert_eq!((Columns::last()).cells(0, 0), vec![]);
        // is this correct?
        assert_eq!(
            (Columns::last()).cells(10, 0),
            vec![
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (4, 0),
                (5, 0),
                (6, 0),
                (7, 0),
                (8, 0),
                (9, 0)
            ]
        );
        assert_eq!((Columns::last()).cells(0, 10), vec![]);
        assert_eq!(
            (Columns::last()).cells(5, 2),
            vec![(0, 1), (1, 1), (2, 1), (3, 1), (4, 1)]
        );
    }

    #[test]
    fn last_column_sub_test() {
        assert_eq!(
            (Columns::last()).cells(5, 2),
            vec![(0, 1), (1, 1), (2, 1), (3, 1), (4, 1)]
        );
        assert_eq!(
            (Columns::last() - 0).cells(5, 2),
            vec![(0, 1), (1, 1), (2, 1), (3, 1), (4, 1)]
        );
        assert_eq!(
            (Columns::last() - 1).cells(5, 2),
            vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        );
        assert_eq!((Columns::last() - 2).cells(5, 2), vec![]);
        assert_eq!((Columns::last() - 100).cells(5, 2), vec![]);
    }

    #[test]
    fn first_column_sub_test() {
        assert_eq!(
            (Columns::first()).cells(5, 2),
            [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        );
        assert_eq!(
            (Columns::first() + 0).cells(5, 2),
            [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        );
        assert_eq!(
            (Columns::first() + 1).cells(5, 2),
            [(0, 1), (1, 1), (2, 1), (3, 1), (4, 1)]
        );
        assert_eq!((Columns::first() + 2).cells(5, 2), []);
        assert_eq!((Columns::first() + 100).cells(5, 2), []);
    }

    #[test]
    fn last_row_sub_test() {
        assert_eq!((Rows::last()).cells(5, 2), vec![(4, 0), (4, 1)]);
        assert_eq!((Rows::last() - 0).cells(5, 2), vec![(4, 0), (4, 1)]);
        assert_eq!((Rows::last() - 1).cells(5, 2), vec![(3, 0), (3, 1)]);
        assert_eq!((Rows::last() - 2).cells(5, 2), vec![(2, 0), (2, 1)]);
        assert_eq!((Rows::last() - 3).cells(5, 2), vec![(1, 0), (1, 1)]);
        assert_eq!((Rows::last() - 4).cells(5, 2), vec![(0, 0), (0, 1)]);
        assert_eq!((Rows::last() - 5).cells(5, 2), vec![]);
        assert_eq!((Rows::last() - 100).cells(5, 2), vec![]);
    }

    #[test]
    fn first_row_sub_test() {
        assert_eq!((Rows::first()).cells(5, 2), vec![(0, 0), (0, 1)]);
        assert_eq!((Rows::first() + 0).cells(5, 2), vec![(0, 0), (0, 1)]);
        assert_eq!((Rows::first() + 1).cells(5, 2), vec![(1, 0), (1, 1)]);
        assert_eq!((Rows::first() + 2).cells(5, 2), vec![(2, 0), (2, 1)]);
        assert_eq!((Rows::first() + 3).cells(5, 2), vec![(3, 0), (3, 1)]);
        assert_eq!((Rows::first() + 4).cells(5, 2), vec![(4, 0), (4, 1)]);
        assert_eq!((Rows::first() + 5).cells(5, 2), vec![]);
        assert_eq!((Rows::first() + 100).cells(5, 2), vec![]);
    }

    #[test]
    fn rows_test() {
        assert_eq!(
            (Rows::new(..)).cells(2, 3),
            vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2)]
        );
        assert_eq!((Rows::new(1..)).cells(2, 3), vec![(1, 0), (1, 1), (1, 2)]);
        assert_eq!((Rows::new(2..)).cells(2, 3), vec![]);
        assert_eq!((Rows::new(1..2)).cells(2, 3), vec![(1, 0), (1, 1), (1, 2)]);
    }

    #[test]
    fn columns_test() {
        assert_eq!(
            (Columns::new(..)).cells(2, 3),
            vec![(0, 0), (1, 0), (0, 1), (1, 1), (0, 2), (1, 2)]
        );
        assert_eq!(
            (Columns::new(1..)).cells(2, 3),
            vec![(0, 1), (1, 1), (0, 2), (1, 2)]
        );
        assert_eq!((Columns::new(2..)).cells(2, 3), vec![(0, 2), (1, 2)]);
        assert_eq!((Columns::new(3..)).cells(2, 3), vec![]);
        assert_eq!((Columns::new(1..2)).cells(2, 3), vec![(0, 1), (1, 1)]);
    }

    #[test]
    fn segment_test() {
        assert_eq!(
            (Segment::new(.., ..)).cells(2, 3),
            vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2)]
        );
        assert_eq!(
            (Segment::new(1.., ..)).cells(2, 3),
            vec![(1, 0), (1, 1), (1, 2)]
        );
        assert_eq!((Segment::new(2.., ..)).cells(2, 3), vec![]);

        assert_eq!(
            (Segment::new(.., 1..)).cells(2, 3),
            vec![(0, 1), (0, 2), (1, 1), (1, 2)]
        );
        assert_eq!((Segment::new(.., 2..)).cells(2, 3), vec![(0, 2), (1, 2)]);
        assert_eq!((Segment::new(.., 3..)).cells(2, 3), vec![]);

        assert_eq!((Segment::new(1.., 1..)).cells(2, 3), vec![(1, 1), (1, 2)]);
        assert_eq!((Segment::new(1..2, 1..2)).cells(2, 3), vec![(1, 1)]);

        assert_eq!((Segment::new(5.., 5..)).cells(2, 3), vec![]);
    }

    #[test]
    fn object_and_test() {
        assert_eq!(Cell(0, 0).and(Cell(0, 0)).cells(2, 3), vec![(0, 0)]);
        assert_eq!(Cell(0, 0).and(Cell(1, 2)).cells(2, 3), vec![(0, 0), (1, 2)]);
        assert_eq!(Cell(0, 0).and(Cell(1, 2)).cells(0, 0), vec![(0, 0), (1, 2)]);
    }

    #[test]
    fn object_not_test() {
        assert_eq!(Cell(0, 0).not(Cell(0, 0)).cells(2, 3), vec![]);
        assert_eq!(
            Rows::first().not(Cell(0, 0)).cells(2, 3),
            vec![(0, 1), (0, 2)]
        );
        assert_eq!(Rows::first().not(Cell(0, 0)).cells(0, 0), vec![]);
    }
}
