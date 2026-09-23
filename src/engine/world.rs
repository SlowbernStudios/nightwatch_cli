use super::{content, rng::Seeded};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn distance(self, other: Self) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }
    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
    /// Frozen north, south, west, east order also defines supervisor tie breaks.
    pub fn cardinal(self) -> [Self; 4] {
        [
            self.offset(0, -1),
            self.offset(0, 1),
            self.offset(-1, 0),
            self.offset(1, 0),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Void,
    Wall,
    Floor,
    Corridor,
    Door,
    Fixture,
}

#[derive(Clone, Debug)]
struct Tile {
    terrain: Terrain,
    glyph: char,
    room: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Room {
    pub id: &'static str,
    pub kind: &'static str,
    pub title: &'static str,
    pub center: Point,
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Station {
    #[serde(flatten)]
    pub position: Point,
    pub id: &'static str,
    pub label: &'static str,
    pub glyph: char,
    pub room: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SupplyKind {
    Coffee,
    Biscuit,
}

#[derive(Clone, Debug, Serialize)]
pub struct Supply {
    #[serde(flatten)]
    pub position: Point,
    pub id: String,
    pub kind: SupplyKind,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldView {
    pub seed: u32,
    pub width: i32,
    pub height: i32,
    pub rooms: Vec<Room>,
    pub stations: Vec<Station>,
    pub start: Point,
    pub cat_home: Point,
    pub supplies: Vec<Supply>,
    pub hazards: Vec<Point>,
    pub terrain: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct World {
    view: WorldView,
    tiles: Vec<Tile>,
}

struct Placement {
    kind: &'static str,
    grid: Point,
    variant: usize,
}

fn layout(seed: u32) -> (Vec<Placement>, usize) {
    let mut rng = Seeded::new(seed);
    let mut kinds = vec!["workshop", "fuel", "office", "control", "break"];
    for extra in ["stores", "records"] {
        if f64::from(rng.next_u32()) / 4294967296.0 > 0.33 {
            kinds.push(extra);
        }
    }
    rng.shuffle(&mut kinds);
    let mut placed = vec![Placement {
        kind: "generator",
        grid: Point::default(),
        variant: 0,
    }];
    for kind in kinds {
        // Duplicate frontier entries deliberately weight touching occupied rooms.
        let frontier: Vec<_> = placed
            .iter()
            .flat_map(|r| [(1, 0), (-1, 0), (0, 1), (0, -1)].map(|(x, y)| r.grid.offset(x, y)))
            .filter(|p| p.x.abs() <= 2 && p.y.abs() <= 2 && !placed.iter().any(|r| r.grid == *p))
            .collect();
        let grid = frontier[rng.index(frontier.len())];
        placed.push(Placement {
            kind,
            grid,
            variant: rng.index(3),
        });
    }
    let homes: Vec<_> = placed
        .iter()
        .enumerate()
        .filter(|(_, p)| matches!(p.kind, "workshop" | "break" | "stores" | "records"))
        .map(|(i, _)| i)
        .collect();
    let home = homes[rng.index(homes.len())];
    (placed, home)
}

impl World {
    pub fn new(seed: u32) -> Self {
        let (placed, home) = layout(seed);
        let min_x = placed.iter().map(|r| r.grid.x).min().unwrap();
        let min_y = placed.iter().map(|r| r.grid.y).min().unwrap();
        let width = (placed.iter().map(|r| r.grid.x).max().unwrap() - min_x + 1) * 22 + 2;
        let height = (placed.iter().map(|r| r.grid.y).max().unwrap() - min_y + 1) * 14 + 2;
        let mut w = Self {
            view: WorldView {
                seed,
                width,
                height,
                rooms: vec![],
                stations: vec![],
                start: Point::default(),
                cat_home: Point::default(),
                supplies: vec![],
                hazards: vec![],
                terrain: vec![],
            },
            tiles: vec![
                Tile {
                    terrain: Terrain::Void,
                    glyph: ' ',
                    room: None
                };
                (width * height) as usize
            ],
        };
        let mut rng = Seeded::new(seed ^ 0x739a);
        for (index, placement) in placed.iter().enumerate() {
            let center = Point {
                x: 12 + (placement.grid.x - min_x) * 22,
                y: 8 + (placement.grid.y - min_y) * 14,
            };
            let hw = 7 + rng.index(3) as i32;
            let hh = 3 + rng.index(3) as i32;
            let r = Room {
                id: placement.kind,
                kind: placement.kind,
                title: content::title(placement.kind),
                center,
                left: center.x - hw,
                right: center.x + hw,
                top: center.y - hh,
                bottom: center.y + hh,
            };
            for y in r.top..=r.bottom {
                for x in r.left..=r.right {
                    let horizontal = y == r.top || y == r.bottom;
                    let wall = horizontal || x == r.left || x == r.right;
                    w.set(
                        Point { x, y },
                        if wall { Terrain::Wall } else { Terrain::Floor },
                        if horizontal {
                            '-'
                        } else if wall {
                            '|'
                        } else {
                            '.'
                        },
                        Some(index),
                    );
                }
            }
            let slots = if index == 0 {
                [(0, -1), (3, -1), (-5, -1)]
            } else {
                [(-4, -1), (4, -1), (-4, 1)]
            };
            for (i, &(id, label, glyph)) in content::stations(placement.kind).iter().enumerate() {
                let (dx, dy) = slots[i];
                let position = center.offset(dx * if placement.variant == 1 { -1 } else { 1 }, dy);
                w.set(position, Terrain::Fixture, glyph, Some(index));
                w.view.stations.push(Station {
                    position,
                    id,
                    label,
                    glyph,
                    room: placement.kind,
                });
            }
            w.set(
                Point {
                    x: r.left + 2,
                    y: r.top + 1,
                },
                Terrain::Fixture,
                ']',
                Some(index),
            );
            if index == 0 {
                w.view.start = center.offset(0, 1);
            }
            if index == home {
                w.view.cat_home = center.offset(2, 1);
            }
            w.view.rooms.push(r);
        }
        for (i, a) in placed.iter().enumerate() {
            for (j, b) in placed.iter().enumerate() {
                if a.kind > b.kind || (a.grid.x - b.grid.x).abs() + (a.grid.y - b.grid.y).abs() != 1
                {
                    continue;
                }
                let ra = w.view.rooms[i].clone();
                let rb = w.view.rooms[j].clone();
                if a.grid.x != b.grid.x {
                    let (left, right) = if ra.center.x < rb.center.x {
                        (ra, rb)
                    } else {
                        (rb, ra)
                    };
                    for x in left.right..=right.left {
                        let door = x == left.right || x == right.left;
                        w.set(
                            Point {
                                x,
                                y: left.center.y,
                            },
                            if door {
                                Terrain::Door
                            } else {
                                Terrain::Corridor
                            },
                            if door { '+' } else { '#' },
                            None,
                        );
                    }
                } else {
                    let (top, bottom) = if ra.center.y < rb.center.y {
                        (ra, rb)
                    } else {
                        (rb, ra)
                    };
                    for y in top.bottom..=bottom.top {
                        let door = y == top.bottom || y == bottom.top;
                        w.set(
                            Point { x: top.center.x, y },
                            if door {
                                Terrain::Door
                            } else {
                                Terrain::Corridor
                            },
                            if door { '+' } else { '#' },
                            None,
                        );
                    }
                }
            }
        }
        let mut floors: Vec<_> = w
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.terrain == Terrain::Floor)
            .map(|(i, _)| Point {
                x: i as i32 % width,
                y: i as i32 / width,
            })
            .filter(|p| *p != w.view.start && *p != w.view.cat_home)
            .collect();
        rng.shuffle(&mut floors);
        for i in 0..4 {
            w.view.supplies.push(Supply {
                position: floors.pop().unwrap(),
                id: format!("supply-{i}"),
                kind: if i % 2 == 0 {
                    SupplyKind::Coffee
                } else {
                    SupplyKind::Biscuit
                },
            });
        }
        for _ in 0..5 {
            w.view.hazards.push(floors.pop().unwrap());
        }
        w.view.terrain = w
            .tiles
            .chunks(width as usize)
            .map(|row| row.iter().map(|t| t.glyph).collect())
            .collect();
        w
    }

    pub fn view(&self) -> &WorldView {
        &self.view
    }

    fn index(&self, p: Point) -> Option<usize> {
        (p.x >= 0 && p.y >= 0 && p.x < self.view.width && p.y < self.view.height)
            .then_some((p.y * self.view.width + p.x) as usize)
    }

    fn set(&mut self, p: Point, terrain: Terrain, glyph: char, room: Option<usize>) {
        let i = self
            .index(p)
            .expect("generated geometry must stay in bounds");
        self.tiles[i] = Tile {
            terrain,
            glyph,
            room,
        };
    }

    pub fn terrain(&self, p: Point) -> Terrain {
        self.index(p)
            .map_or(Terrain::Void, |i| self.tiles[i].terrain)
    }
    pub fn room(&self, p: Point) -> Option<&Room> {
        self.index(p)
            .and_then(|i| self.tiles[i].room)
            .map(|i| &self.view.rooms[i])
    }
    pub fn traversable(&self, p: Point) -> bool {
        matches!(
            self.terrain(p),
            Terrain::Floor | Terrain::Corridor | Terrain::Door
        )
    }

    pub fn line_visible(&self, from: Point, to: Point, open: &HashSet<Point>) -> bool {
        let mut p = from;
        let dx = (to.x - p.x).abs();
        let dy = -(to.y - p.y).abs();
        let sx = if p.x < to.x { 1 } else { -1 };
        let sy = if p.y < to.y { 1 } else { -1 };
        let mut error = dx + dy;
        while p != to {
            let twice = 2 * error;
            if twice >= dy {
                error += dy;
                p.x += sx;
            }
            if twice <= dx {
                error += dx;
                p.y += sy;
            }
            if p == to {
                return true;
            }
            match self.terrain(p) {
                Terrain::Void | Terrain::Wall => return false,
                Terrain::Door if !open.contains(&p) => return false,
                _ => (),
            }
        }
        true
    }
}
