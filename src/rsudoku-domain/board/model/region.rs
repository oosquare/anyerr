use std::fmt::Debug;
use std::ops::Range;

use crate::board::model::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionKind {
    Row,
    Column,
    Box,
}

pub trait Region: Debug + Clone + PartialEq + Eq {
    type Iter: RegionIter;

    fn kind(&self) -> RegionKind;

    fn size(&self) -> usize {
        Position::EXCLUSIVE_MAX_VALUE
    }

    fn iter(&self) -> Self::Iter;

    fn to_positions(&self) -> Vec<Position> {
        self.iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionVariant {
    Row(RowRegion),
    Column(ColumnRegion),
    Box(BoxRegion),
}

impl RegionVariant {
    pub fn new_row(row: usize) -> Self {
        Self::Row(RowRegion::new(row))
    }

    pub fn new_column(column: usize) -> Self {
        Self::Column(ColumnRegion::new(column))
    }

    pub fn new_box(box_row: usize, box_column: usize) -> Self {
        Self::Box(BoxRegion::new(box_row, box_column))
    }

    pub fn of_position(position: &Position, kind: RegionKind) -> Self {
        match kind {
            RegionKind::Row => Self::Row(RowRegion::of_position(position)),
            RegionKind::Column => Self::Column(ColumnRegion::of_position(position)),
            RegionKind::Box => Self::Box(BoxRegion::of_position(position)),
        }
    }
}

impl Region for RegionVariant {
    type Iter = RegionIterVariant;

    fn kind(&self) -> RegionKind {
        match self {
            Self::Row(s) => s.kind(),
            Self::Column(s) => s.kind(),
            Self::Box(s) => s.kind(),
        }
    }

    fn size(&self) -> usize {
        match self {
            Self::Row(s) => s.size(),
            Self::Column(s) => s.size(),
            Self::Box(s) => s.size(),
        }
    }

    fn iter(&self) -> Self::Iter {
        match self {
            Self::Row(s) => s.iter().into(),
            Self::Column(s) => s.iter().into(),
            Self::Box(s) => s.iter().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowRegion {
    row: usize,
}

impl RowRegion {
    fn new(row: usize) -> Self {
        Self { row }
    }

    fn of_position(position: &Position) -> Self {
        Self::new(position.row())
    }
}

impl Region for RowRegion {
    type Iter = RowRegionIter;

    fn kind(&self) -> RegionKind {
        RegionKind::Row
    }

    fn iter(&self) -> Self::Iter {
        RowRegionIter::new(self.row)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnRegion {
    column: usize,
}

impl ColumnRegion {
    fn new(column: usize) -> Self {
        Self { column }
    }

    fn of_position(position: &Position) -> Self {
        Self::new(position.column())
    }
}

impl Region for ColumnRegion {
    type Iter = ColumnRegionIter;

    fn kind(&self) -> RegionKind {
        RegionKind::Column
    }

    fn iter(&self) -> Self::Iter {
        ColumnRegionIter::new(self.column)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxRegion {
    box_row: usize,
    box_column: usize,
}

impl BoxRegion {
    fn new(box_row: usize, box_column: usize) -> Self {
        Self {
            box_row,
            box_column,
        }
    }

    fn of_position(position: &Position) -> Self {
        Self::new(position.row() / 3, position.column() / 3)
    }
}

impl Region for BoxRegion {
    type Iter = BoxRegionIter;

    fn kind(&self) -> RegionKind {
        RegionKind::Box
    }

    fn iter(&self) -> Self::Iter {
        BoxRegionIter::new(self.box_row, self.box_column)
    }
}

pub trait RegionIter: Clone + Iterator<Item = Position> {}

#[derive(Debug, Clone)]
pub enum RegionIterVariant {
    Row(RowRegionIter),
    Column(ColumnRegionIter),
    Box(BoxRegionIter),
}

impl Iterator for RegionIterVariant {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Row(s) => s.next(),
            Self::Column(s) => s.next(),
            Self::Box(s) => s.next(),
        }
    }
}

impl From<RowRegionIter> for RegionIterVariant {
    fn from(value: RowRegionIter) -> Self {
        Self::Row(value)
    }
}

impl From<ColumnRegionIter> for RegionIterVariant {
    fn from(value: ColumnRegionIter) -> Self {
        Self::Column(value)
    }
}

impl From<BoxRegionIter> for RegionIterVariant {
    fn from(value: BoxRegionIter) -> Self {
        Self::Box(value)
    }
}

impl RegionIter for RegionIterVariant {}

#[derive(Debug, Clone)]
pub struct RowRegionIter {
    row: usize,
    column_range: Range<usize>,
}

impl RowRegionIter {
    fn new(row: usize) -> Self {
        Self {
            row,
            column_range: 0..Position::EXCLUSIVE_MAX_VALUE,
        }
    }
}

impl Iterator for RowRegionIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        self.column_range
            .next()
            .map(|column| Position::new(self.row, column))
    }
}

impl RegionIter for RowRegionIter {}

#[derive(Debug, Clone)]
pub struct ColumnRegionIter {
    column: usize,
    row_range: Range<usize>,
}

impl ColumnRegionIter {
    fn new(column: usize) -> Self {
        Self {
            column,
            row_range: 0..Position::EXCLUSIVE_MAX_VALUE,
        }
    }
}

impl Iterator for ColumnRegionIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        self.row_range
            .next()
            .map(|row| Position::new(row, self.column))
    }
}

impl RegionIter for ColumnRegionIter {}

#[derive(Debug, Clone)]
pub struct BoxRegionIter {
    box_row_offset: usize,
    box_column_offset: usize,
    order_range: Range<usize>,
}

impl BoxRegionIter {
    fn new(box_row: usize, box_column: usize) -> Self {
        Self {
            box_row_offset: box_row * 3,
            box_column_offset: box_column * 3,
            order_range: 0..Position::EXCLUSIVE_MAX_VALUE,
        }
    }
}

impl Iterator for BoxRegionIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        self.order_range.next().map(|order| {
            Position::new(
                self.box_row_offset + order / 3,
                self.box_column_offset + order % 3,
            )
        })
    }
}

impl RegionIter for BoxRegionIter {}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn region_kind() {
        {
            let region = RegionVariant::new_row(0);
            assert_eq!(RegionKind::Row, region.kind());
        }

        {
            let region = RegionVariant::new_column(0);
            assert_eq!(RegionKind::Column, region.kind());
        }

        {
            let region = RegionVariant::new_box(0, 0);
            assert_eq!(RegionKind::Box, region.kind());
        }
    }

    #[test]
    fn region_size() {
        {
            let region = RegionVariant::new_row(0);
            assert_eq!(9, region.size());
        }

        {
            let region = RegionVariant::new_column(0);
            assert_eq!(9, region.size());
        }

        {
            let region = RegionVariant::new_box(0, 0);
            assert_eq!(9, region.size());
        }
    }

    #[test]
    fn region_to_position() {
        {
            let region = RegionVariant::new_row(1);
            let actual: HashSet<_> = region.to_positions().into_iter().collect();
            let expected: HashSet<_> = (0..9).map(|column| Position::new(1, column)).collect();
            assert_eq!(expected, actual);
        }

        {
            let region = RegionVariant::new_column(2);
            let actual: HashSet<_> = region.to_positions().into_iter().collect();
            let expected: HashSet<_> = (0..9).map(|row| Position::new(row, 2)).collect();
            assert_eq!(expected, actual);
        }

        {
            let region = RegionVariant::new_box(1, 2);
            let actual: HashSet<_> = region.to_positions().into_iter().collect();
            let expected: HashSet<_> = [
                [(3, 6), (3, 7), (3, 8)],
                [(4, 6), (4, 7), (4, 8)],
                [(5, 6), (5, 7), (5, 8)],
            ]
            .iter()
            .flatten()
            .map(|&(row, column)| Position::new(row, column))
            .collect();
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn region_variant_of_position() {
        {
            let region = RegionVariant::of_position(&Position::new(2, 3), RegionKind::Row);
            assert_eq!(RegionVariant::new_row(2), region);
        }

        {
            let region = RegionVariant::of_position(&Position::new(2, 3), RegionKind::Column);
            assert_eq!(RegionVariant::new_column(3), region);
        }

        {
            let region = RegionVariant::of_position(&Position::new(2, 3), RegionKind::Box);
            assert_eq!(RegionVariant::new_box(0, 1), region);
        }
    }
}
