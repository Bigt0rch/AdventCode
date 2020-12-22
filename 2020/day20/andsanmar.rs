use std::fs;
use std::collections::HashMap;

type Tile = Vec<Vec<bool>>;

fn print_tile(tile: &Tile) { for x in tile {println!("{}", x.iter().map(|c| if *c {'#'} else {'.'}).collect::<String>())} }

fn rotate_left(tile: &Tile, dimension : usize) -> Tile {
    (0..dimension).map(|i| {
        tile.iter().map(|line| line[dimension-i-1]).collect()
    }).collect()
}

fn flip_upside_down(tile: &Tile, dimension: usize) -> Tile {
    (0..dimension).map(|i| {
        tile[dimension-i-1].clone()
    }).collect()
}

fn matches(tiles : &[(usize,[[Tile;4];2])]) -> HashMap<usize,(usize,bool)> { // ID_a, ID_b, rot_sum%4, flipped_XOR
    let mut matched : HashMap<usize,(usize,bool)> = HashMap::new();
    match tiles.split_first() {
        None => matched,
        Some(((id_to_match, tiles_to_match), rest)) => {
            for (flipped_to_match, to_match) in tiles_to_match.iter().enumerate() {
                for (rot_to_match, tile_to_match) in to_match.iter().enumerate() {
                    let border_to_match = &tile_to_match[0];
                    for (id_matched, tiles_potential_matches) in rest {
                        for (flipped_matched, potential_matches) in tiles_potential_matches.iter().enumerate() {
                            for (rot_matched, tile_matched) in potential_matches.iter().enumerate() {
                                if &tile_matched[0] == border_to_match {
                                    matched.insert(id_to_match*id_matched,((rot_to_match+rot_matched) % 4, (flipped_to_match==1) ^ (flipped_matched==1)));
                                }
                            }
                        }
                    }
                }
            }
            matched.extend(matches(rest));
            matched
        }
    }
}

fn tile_adjacents(ids : &Vec<usize>, matches_id : Vec<&usize>) -> Vec<(usize,usize)> {
    ids.iter().map(|id| (*id,matches_id.iter().filter(|id_m| **id_m % id == 0).count())).collect()
}

fn tile_by_right<'a>(left : &Tile, possibilities : &'a[[Tile;4];2]) -> &'a Tile {
    let to_match : Vec<&bool> = left.iter().map(|r| r.last().unwrap()).collect();
    possibilities.iter().flatten().find(|t| t.iter().map(|r| r.first().unwrap()).collect::<Vec<&bool>>() == to_match).unwrap()
}

fn tile_by_down<'a>(up : &Tile, possibilities : &'a [[Tile;4];2]) -> &'a Tile {
    let to_match : &Vec<bool> = up.last().unwrap();
    possibilities.iter().flatten().find(|t| t.first().unwrap() == to_match).unwrap()
}


fn reconstruct_image(ids : &Vec<usize>, edges_matches : HashMap<usize,(usize,bool)>, tiles: &[(usize,[[Tile;4];2])]) -> Tile {
    let dimension = (tiles.len() as f64).sqrt() as usize;
    let adjacencies : Vec<(usize,Vec<usize>)> = ids.iter().map(|id| (*id,edges_matches.keys().filter(|id_m| **id_m % id == 0).map(|id_m| id_m / id).collect())).collect();
    // 1st step, place the ids to satisfy the adjacency constraints
    // `image_ids` could be defined with slices, but their dimension is not dynamic
    let mut image_ids : Vec<Vec<Option<usize>>> = vec![vec![None;dimension];dimension];
    let first_corner : &(usize,Vec<usize>) = adjacencies.iter().find(|(_id,adjacents)| adjacents.len() == 2).unwrap();
    image_ids[0][0] = Some(first_corner.0);
    for x in 0..dimension {
        for y in 0..dimension {
            if image_ids[x][y] == None {
                let mut adjacent : Vec<Option<usize>> = Vec::new();
                for (x1,y1) in &[(x,y-1), (x,y+1), (x+1,y), (x-1,y)] {
                    match image_ids.get(*x1) {
                        None => {},
                        Some(row) => match row.get(*y1) {
                            None => {},
                            Some(e) => adjacent.push(*e)
                        }
                    }
                }
                let next_tile = adjacencies.iter().find(|(id, adjs)| adjacent.iter().all(|maybe_id| {
                    match maybe_id {
                        None => true,
                        Some(id_n) => adjs.contains(id_n)
                    }
                } && adjs.len() == adjacent.len() && image_ids.iter().all(|row| !row.contains(&Some(*id))))).unwrap().0;
                image_ids[x][y] = Some(next_tile);
            }
        }
    }
    // 2nd step, place the tiles correctly flipped & rotated
    let mut image_tiles : Vec<Vec<Option<&Tile>>> = vec![vec![None;dimension];dimension];
    image_tiles[0][0] = Some(&tiles.iter().find(|(t_id,_)| t_id == &image_ids[0][0].unwrap()).unwrap().1[0][0]); // FIXME selection of the first tile (it may not be oriented)
    for x in 0..dimension {
        for y in 0..dimension {
            if image_tiles[x][y] == None {
                let id = image_ids[x][y].unwrap();
                let possibilities = &tiles.iter().find(|(t_id,_)| t_id == &id).unwrap().1;
                image_tiles[x][y] = Some(
                    if y > 0 { tile_by_right(image_tiles[x][y-1].unwrap(), possibilities) }
                    else { tile_by_down(image_tiles[x-1][y].unwrap(), possibilities) })
            }
        }
    };    
    // 3rd step, remove the borders of the tiles to form the image
    // fn remove_border(t : &Tile) -> Tile {t}
    let mut image_div : Vec<Tile> = image_tiles.iter().map(|r| {
        let mut without_borders : Vec<Tile> = r.iter().map(|t| t.unwrap().split_first().unwrap().1.split_last().unwrap().1.iter().map(|r| r.split_first().unwrap().1.split_last().unwrap().1.to_vec()).collect::<Tile>()).collect();
        let (merged, rest) = without_borders.split_first_mut().unwrap();
        for el in rest {
            for x in 0..merged.len() {
                merged[x].append(&mut el[x]);
            }
        }
        merged.to_vec()
    }).collect();

    let (image, rest) = image_div.split_first_mut().unwrap();
    for x in 0..rest.len() {
        image.append(&mut rest[x]);
    }    
    image.to_vec()
}

fn find_pattern(image : &mut Tile) {
    let pattern = [(0,18),(1,0),(1,5),(1,6),(1,11),(1,12),(1,17),(1,18),(1,19),(2,1),(2,4),(2,7),(2,10),(2,13),(2,16)];
    let mut found = false;
    for x in 0..image.len()-2 {
        for y in 0..image[x].len()-19 {
            if pattern.iter().all(|(x1,y1)| image[x+x1][y+y1]) {
                found = true;
                for (x1,y1) in pattern.iter() { image[x+x1][y+y1] = false }
            }
        }
    }
    if found {
        println!("{}", image.iter().flatten().filter(|x| **x).count());
    }
}

fn obtain_transformations(tile : Tile) -> [[Tile;4];2] {
    let dimension = tile.first().unwrap().len();
    let rot_1 = rotate_left(&tile, dimension);
    let rot_2 = rotate_left(&rot_1, dimension);
    let rot_3 = rotate_left(&rot_2, dimension);
    let flip_0 = flip_upside_down(&tile, dimension);
    let flip_1 = flip_upside_down(&rot_1, dimension);
    let flip_2 = flip_upside_down(&rot_2, dimension);
    let flip_3 = flip_upside_down(&rot_3, dimension);
    [[tile, rot_1, rot_2, rot_3], [flip_0, flip_1, flip_2, flip_3]]
}

fn main () {
    let r = fs::read_to_string("input").unwrap();
    // 4 rotations included
    let tiles : Vec<(usize,[[Tile;4];2])> = r.split("\n\n").map(|tile| {
        let mut it = tile.lines();
        let id_tile = it.next().unwrap().split_whitespace().skip(1).next().unwrap()[..4].parse::<usize>().unwrap();
        let tile : Tile = it.map(|l| l.chars().map(|c| c == '#').collect()).collect();
        (id_tile, obtain_transformations(tile))
    }).collect();
    // key is a `usize` because ids are prime numbers, this way search is supposed to be easier, a tuple of both numbers is OK too
    // TODO hashmap values are never used, drop them
    let m : HashMap<usize,(usize,bool)> = matches(tiles.as_slice());
    let ids = tiles.iter().map(|(id,_)| *id).collect();
    let adjs = tile_adjacents(&ids,m.keys().collect());
    let corners = adjs.iter().filter(|(_,n)| n == &2).map(|(id,_)| id);
    println!("{:?}", corners.product::<usize>());
    let image = reconstruct_image(&ids, m, tiles.as_slice());
    for t in obtain_transformations(image).iter().flatten() {find_pattern(&mut t.to_vec());}
}
