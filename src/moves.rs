use super::*;

use crate::{hex::HDir, mesh::small_mesh::SmallMesh};

impl GameState {
    // pub fn is_trap(&self, team: ActiveTeam, world: &board::MyWorld, check: Axial,typ:UnitType) -> bool {
    //     let k=match typ{
    //         UnitType::Mouse=>self.env.terrain.is_set(check),
    //         UnitType::Rabbit=>!self.env.terrain.is_set(check) && world.get_game_cells().is_set(check)
    //     };
    //     //if you push an enemy unit into a wall, they die
    //     //if you push an enemy off the map, they die
    //     //if you push an enemy into one of your teamates they die
    //     k
    //         || self.factions.has_a_set(check)
    //         || !world.get_game_cells().is_set(check)
    // }

    // fn check_if_occ(&self, world: &board::MyWorld, a: Axial, check_fog: bool) -> bool {
    //     let game = self;
    //     let is_world_cell = world.get_game_cells().is_coord_set(a);

    //     let jjj = if check_fog {
    //         !game.env.fog.is_coord_set(a)
    //     } else {
    //         true
    //     };

    //     is_world_cell
    //         && !game.env.terrain.is_coord_set(a)
    //         && jjj
    //         && game.factions.dogs.find_slow(&a).is_none()
    //         && game.factions.cats.find_slow(&a).is_none()
    // }

    pub fn generate_possible_moves_extra(
        &self,
        world: &board::MyWorld,
        foo: &move_build::MovePhase,
        effect: &move_build::MoveEffect,
        _team: ActiveTeam,
    ) -> SmallMesh {
        let game = self;
        let unit = foo.moveto;
        let original_pos = foo.original;
        let mut mesh = SmallMesh::new();

        mesh.add(unit);

        // if effect.destroyed_unit.is_some() {
        //     mesh.add(Axial::zero())
        // } else {
        //     let tt = self.factions.has_a_set_type(unit).unwrap();

        //     // match tt {
        //     //     UnitType::Mouse => {
        //     //         for a in unit.to_cube().neighbours2() {
        //     //             let a = a.to_axial();

        //     //             if a != unit
        //     //                 && world.get_game_cells().is_set(a)
        //     //                 && !game.factions.has_a_set(a)
        //     //                 && !game.env.terrain.is_set(a)
        //     //                 && !game.env.fog.is_set(a)
        //     //             {
        //     //                 mesh.add(a.sub(&unit));

        //     //                 // for a in a.to_cube().ring(1) {
        //     //                 //     let a = a.to_axial();

        //     //                 //     if check_if_occ(a, true) {
        //     //                 //         mesh.add(a.sub(&unit));
        //     //                 //     }
        //     //                 // }
        //     //             }
        //     //         }
        //     //     }
        //     //     UnitType::Rabbit => mesh.add(foo.original.sub(&unit)),
        //     // }
        //     mesh.add(foo.original.sub(&unit))
        // }
        mesh
    }

    pub fn attack_mesh_add(
        &self,
        mesh: &mut SmallMesh,
        world: &board::MyWorld,
        &unit: &Axial,
        team: ActiveTeam,
        for_show: bool,
    ) {
        let typ = self.factions.units.get_type(unit);
        let game = self;
        //let terrain = &game.env.terrain;

        let this_team = game.factions.get_all_team(team);
        let that_team = game.factions.get_all_team(team.not());

        let other_side_occupied = |x: Axial| {
            game.factions.units.is_set(x)
                && game.factions.parity.is_set(x) != game.factions.parity.is_set(unit)
        };

        let is_enemy_same_parity = |x: Axial| {
            that_team.is_set(x)
                && game.factions.parity.is_set(x) == game.factions.parity.is_set(unit)
        };

        let is_friendly_same_parity = |x: Axial| {
            this_team.is_set(x)
                && game.factions.parity.is_set(x) == game.factions.parity.is_set(unit)
        };
        let is_friendly_different_parity = |x: Axial| {
            this_team.is_set(x)
                && game.factions.parity.is_set(x) != game.factions.parity.is_set(unit)
        };

        let ray = |mut c: Axial, v: Axial| {
            std::iter::repeat_with(move || {
                c = c.add(v);
                c
            })
        };

        let ray2 = |uni: Axial, mesh: &mut SmallMesh, num: usize, attacking: bool| {
            for a in ray(unit, uni).take(num) {
                if !world.get_game_cells().is_set(a) {
                    break;
                }

                if is_friendly_same_parity(a) {
                    break;
                }

                if !attacking && is_enemy_same_parity(a) {
                    break;
                }

                if !other_side_occupied(a) {
                    mesh.add(a);
                }

                if is_enemy_same_parity(a) {
                    break;
                }
            }
        };

        let i = match typ {
            UnitType::Rook => {
                for [q, r] in [[1, 0], [-1, 0], [0, 1], [0, -1]] {
                    let uni = Axial { q, r };
                    ray2(uni, mesh, 8, true);
                }
            }
            UnitType::King => {
                for q in [-1, 0, 1] {
                    for r in [-1, 0, 1] {
                        if q == 0 && r == 0 {
                            continue;
                        };
                        let k = Axial { q, r };

                        ray2(k, mesh, 1, true);
                    }
                }
            }
            UnitType::Pawn => {
                let (forward, diag) = if let ActiveTeam::Black = team {
                    ([1, 0], [[1, -1], [1, 1]])
                } else {
                    ([-1, 0], [[-1, -1], [-1, 1]])
                };

                ray2(Axial::from_arr(forward), mesh, 1, false);

                for diag in diag {
                    let j = Axial::from_arr(diag);

                    if is_enemy_same_parity(unit.add(j)) {
                        ray2(j, mesh, 1, true);
                    }
                }

                // let k = unit.add(hex::Cube::from_arr(hex::OFFSETS[dd]).ax);

                // if world.get_game_cells().is_set(k)
                //     && !game.env.fog.is_set(k)
                //     && !terrain.is_set(k)
                //     && !game.factions.has_a_set(k)
                // {
                //     mesh.add(k);
                // }

                // for o in [hex::OFFSETS[dd + 1], hex::OFFSETS[(dd + 5) % 6]] {
                //     let k = unit.add(hex::Cube::from_arr(o).ax);
                //     if game.factions.relative(team).that_team.is_set(k) {
                //         mesh.add(k);
                //     }
                // }
            }
            UnitType::Knight => {
                for [q, r] in [
                    [2, 1],
                    [2, -1],
                    [-2, 1],
                    [-2, -1],
                    [1, 2],
                    [-1, 2],
                    [1, -2],
                    [-1, -2],
                ] {
                    let uni = Axial { q, r };
                    ray2(uni, mesh, 1, true);
                }
            }
            UnitType::Bishop => {
                for [q, r] in [[1, 1], [-1, -1], [-1, 1], [1, -1]] {
                    let uni = Axial { q, r };
                    ray2(uni, mesh, 8, true);
                }
            }
            UnitType::Queen => {
                for [q, r] in [
                    [1, 1],
                    [-1, -1],
                    [-1, 1],
                    [1, -1],
                    [1, 0],
                    [-1, 0],
                    [0, 1],
                    [0, -1],
                ] {
                    let uni = Axial { q, r };
                    ray2(uni, mesh, 8, true);
                }
            }
        };
    }
    pub fn generate_possible_moves_movement(
        &self,
        world: &board::MyWorld,
        &unit: &Axial,
        team: ActiveTeam,
    ) -> SmallMesh {
        //TODO use
        let typ = self.factions.units.get_type(unit);

        let game = self;
        let mut mesh = SmallMesh::new();

        //let terrain = &game.env.terrain;

        // let enemy_cover = {
        //     //TODO use a workspace instead
        //     let mut total = BitField::new();
        //     for a in self.factions.relative(team).that_team.iter_mesh() {
        //         let mut mesh = SmallMesh::new();
        //         self.attack_mesh_add(&mut mesh, world, &a, team.not(), true);
        //         for m in mesh.iter_mesh(a) {
        //             total.set_coord(m, true);
        //         }
        //     }
        //     total
        // };

        //console_dbg!("enemy cover size= {}",enemy_cover.count_ones(..));

        // for_every_cell(unit, |a, pp| {
        //     let max_range=match typ{
        //         UnitType::Pawn=>1,
        //         UnitType::Rook => 2,
        //         UnitType::Bishop => 3,
        //         UnitType::Knight => 1,
        //     };

        //     if pp.len()>max_range{
        //         return false;
        //     }

        //     // if pp.len() == 1 {
        //     //     let dir = pp[0];
        //     //     if terrain.land.is_set(a) {
        //     //         let check = a.advance(dir);
        //     //         if world.get_game_cells().is_set(check)
        //     //             && !game.factions.has_a_set(check)
        //     //             && !game.env.fog.is_set(check)
        //     //             && (!terrain.is_set(check)/*|| terrain.land.is_set(check)*/)
        //     //         {
        //     //             mesh.add(a.sub(&unit));
        //     //         }
        //     //         return true;
        //     //     }
        //     // }
        //     if a != unit
        //         && world.get_game_cells().is_set(a)
        //         && !game.factions.has_a_set(a)
        //         && !game.env.fog.is_set(a)
        //         && !terrain.is_set(a)

        //     {
        //         mesh.add(a.sub(&unit));

        //         if enemy_cover.is_set(a){
        //             true
        //         }else{
        //             false
        //         }

        //         //false
        //     } else {
        //         true
        //     }
        // });

        self.attack_mesh_add(&mut mesh, world, &unit, team, true);
        mesh
    }
}

fn for_every_cell(unit: Axial, mut func: impl FnMut(Axial, &[HDir]) -> bool) {
    for a in unit.to_cube().neighbours2() {
        let a = a.to_axial();
        let dir = unit.dir_to(&a);

        if func(a, &[dir]) {
            continue;
        }

        for b in a.to_cube().neighbours2() {
            let b = b.to_axial();
            let dir2 = a.dir_to(&b);

            if b.to_cube().dist(&unit.to_cube()) < a.to_cube().dist(&unit.to_cube()) {
                continue;
            }

            if func(b, &[dir, dir2]) {
                continue;
            }

            for c in b.to_cube().neighbours2() {
                let c = c.to_axial();
                let dir3 = b.dir_to(&c);

                if c.to_cube().dist(&unit.to_cube()) < b.to_cube().dist(&unit.to_cube()) {
                    continue;
                }

                if func(c, &[dir, dir2, dir3]) {
                    continue;
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct ActualMove {
    pub original: Axial,
    pub moveto: Axial,
    pub attackto: Axial,
}

impl GameState {
    pub fn for_all_moves_fast(
        &mut self,
        team: ActiveTeam,
        world: &board::MyWorld,
        mut func: impl FnMut(move_build::CombinedEffect, moves::ActualMove, &GameState),
    ) {
        let state = self;
        //let mut movs = Vec::new();
        //for i in 0..state.factions.relative(team).this_team.units.len() {
        for pos in state.factions.get_all_team(team).iter_mesh() {
            let mesh = state.generate_possible_moves_movement(world, &pos, team);
            for mm in mesh.iter_mesh(Axial::zero()) {
                //Temporarily move the player in the game world.
                //We do this so that the mesh generated for extra is accurate.
                let mut mmm = move_build::MovePhase {
                    original: pos,
                    moveto: mm,
                };

                let mut effect = mmm.apply(team, state, world);

                let second_mesh = state.generate_possible_moves_extra(world, &mmm, &effect, team);

                for sm in second_mesh.iter_mesh(Axial::zero()) {
                    //assert!(!state.env.terrain.is_set(sm));

                    let kkk = mmm.into_attack(sm);

                    let k = kkk.apply(team, state, world, &effect);

                    let mmo = moves::ActualMove {
                        original: pos,
                        moveto: mm,
                        attackto: sm,
                    };

                    let jjj = effect.combine(k);

                    func(jjj.clone(), mmo, state);

                    mmm = kkk.undo(&jjj.extra_effect, state);
                    effect = jjj.move_effect;
                }

                //revert it back just the movement component.
                mmm.undo(team, &effect, state);
            }
        }

        // {
        //     for a in movs.iter() {
        //         assert!(state
        //             .factions
        //             .relative(team)
        //             .this_team
        //             .units
        //             .is_set(a.original));
        //     }
        // }
        // movs
    }
}
