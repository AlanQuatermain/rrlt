use crate::prelude::*;

pub enum RoomSort {
    Leftmost,
    Rightmost,
    Topmost,
    Bottommost,
    Central,
}

pub struct RoomSorter {
    sort_by: RoomSort,
}

impl MetaMapBuilder for RoomSorter {
    #[allow(dead_code)]
    fn build_map(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.sorter(build_data);
    }
}

impl Default for RoomSorter {
    fn default() -> Self {
        RoomSorter {
            sort_by: RoomSort::Leftmost,
        }
    }
}

impl RoomSorter {
    #[allow(dead_code)]
    pub fn new(sort_by: RoomSort) -> Box<RoomSorter> {
        Box::new(RoomSorter { sort_by })
    }

    fn sorter(&mut self, build_data: &mut BuilderMap) {
        let rooms: &mut Vec<Rect>;
        if let Some(rooms_builder) = build_data.rooms.as_mut() {
            rooms = rooms_builder;
        } else {
            panic!("Room Sorter requires a builder that creates rooms");
        }

        match self.sort_by {
            RoomSort::Leftmost => rooms.sort_by(|a, b| a.x1.cmp(&b.x1)),
            RoomSort::Rightmost => rooms.sort_by(|a, b| b.x2.cmp(&a.x2)),
            RoomSort::Topmost => rooms.sort_by(|a, b| a.y1.cmp(&b.y1)),
            RoomSort::Bottommost => rooms.sort_by(|a, b| b.y2.cmp(&a.y2)),
            RoomSort::Central => {
                let map_center = Point::new(build_data.map.width / 2, build_data.map.height / 2);
                let center_sort = |a: &Rect, b: &Rect| {
                    let a_center = a.center();
                    let b_center = b.center();
                    let distance_a = DistanceAlg::Pythagoras.distance2d(a_center, map_center);
                    let distance_b = DistanceAlg::Pythagoras.distance2d(b_center, map_center);
                    distance_a.partial_cmp(&distance_b).unwrap()
                };
                rooms.sort_by(center_sort);
            }
        }
    }
}
