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

        if effect.destroyed_unit.is_some() {
            mesh.add(Axial::zero())
        } else {
            for a in unit.to_cube().ring(1) {
                let a = a.to_axial();

                if a != unit
                    && world.get_game_cells().is_set(a)
                    && !game.factions.has_a_set(a)
                    && !game.env.terrain.is_set(a)
                    && !game.env.fog.is_set(a)
                {
                    mesh.add(a.sub(&unit));

                    // for a in a.to_cube().ring(1) {
                    //     let a = a.to_axial();

                    //     if check_if_occ(a, true) {
                    //         mesh.add(a.sub(&unit));
                    //     }
                    // }
                }
            }
        }
        mesh
    }
    pub fn generate_possible_moves_movement(
        &self,
        world: &board::MyWorld,
        &unit: &Axial,
        team: ActiveTeam,
    ) -> SmallMesh {
        //TODO use
        let typ = self.factions.relative(team).this_team.get_type(unit);

        let game = self;
        let mut mesh = SmallMesh::new();

        let check_empty = |a: Axial| {
            world.get_game_cells().is_set(a)
                && !game.factions.has_a_set(a)
                && !game.env.fog.is_set(a)
        };

        let terrain = &game.env.terrain;

        let kll = |mesh: &mut SmallMesh, a: Axial, dir: hex::HDir| {
            if game.factions.relative(team).that_team.is_set(a) {
                let check = a.advance(dir);

                mesh.add(a.sub(&unit));
            }
        };

        fn for_every_cell(unit: Axial, mut func: impl FnMut(Axial, &[HDir]) -> bool) {
            for a in unit.to_cube().ring(1) {
                let a = a.to_axial();
                let dir = unit.dir_to(&a);

                if func(a, &[dir]) {
                    continue;
                }

                for b in a.to_cube().ring(1) {
                    let b = b.to_axial();
                    let dir2 = a.dir_to(&b);

                    if b.to_cube().dist(&unit.to_cube()) < a.to_cube().dist(&unit.to_cube()) {
                        continue;
                    }

                    if func(b, &[dir, dir2]) {
                        continue;
                    }

                    for c in b.to_cube().ring(1) {
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

        for_every_cell(unit, |a, pp| {
            if pp.len() == 1 {
                let dir = pp[0];
                if terrain.land.is_set(a) {
                    let check = a.advance(dir);
                    if world.get_game_cells().is_set(check)
                        && !game.factions.has_a_set(check)
                        && !game.env.fog.is_set(check)
                        && (!terrain.is_set(check)/*|| terrain.land.is_set(check)*/)
                    {
                        mesh.add(a.sub(&unit));
                    }
                    return true;
                }
            }
            if a != unit
                && world.get_game_cells().is_set(a)
                && !game.factions.relative(team).this_team.is_set(a)
                && !game.env.fog.is_set(a)
                && !terrain.is_set(a)
            {
                mesh.add(a.sub(&unit));
                if game.factions.relative(team).that_team.is_set(a) {
                    true
                } else {
                    false
                }
            } else {
                true
            }
        });

        // for a in unit.to_cube().ring(1) {
        //     let a = a.to_axial();
        //     let dir = unit.dir_to(&a);

        //     if a != unit && check_empty(a) && !terrain.is_set(a) {
        //         mesh.add(a.sub(&unit));

        //         for b in a.to_cube().ring(1) {
        //             let b = b.to_axial();
        //             let dir = a.dir_to(&b);

        //             if b != unit && check_empty(b) && !terrain.is_set(b) {
        //                 mesh.add(b.sub(&unit));

        //                 for c in b.to_cube().ring(1) {
        //                     let c = c.to_axial();
        //                     let dir = b.dir_to(&c);

        //                     if c != unit && check_empty(c) && !terrain.is_set(c) {
        //                         mesh.add(c.sub(&unit));
        //                     }
        //                     if c.to_cube().dist(&unit.to_cube()) > b.to_cube().dist(&unit.to_cube())
        //                     {
        //                         kll(&mut mesh, c, dir);
        //                     }
        //                 }
        //             }

        //             if b.to_cube().dist(&unit.to_cube()) > a.to_cube().dist(&unit.to_cube()) {
        //                 kll(&mut mesh, b, dir);
        //             }
        //         }
        //     } else {
        //         if terrain.land.is_set(a) {
        //             let check = a.advance(dir);
        //             if check_empty(check)
        //                 && (!terrain.is_set(check)/*|| terrain.land.is_set(check)*/)
        //             {
        //                 mesh.add(a.sub(&unit));
        //             }
        //         }

        //         kll(&mut mesh, a, dir);

        //         if game.factions.relative(team).this_team.is_set(a) {
        //             // let check = a.advance(dir);

        //             // if world.get_game_cells().is_set(check)
        //             //     && !game.env.fog.is_set(check)
        //             //     && !terrain.is_set(check)
        //             //     && !game.factions.has_a_set(check)
        //             // {
        //             //     mesh.add(a.sub(&unit));
        //             // }
        //         }
        //     }
        // }
        mesh
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
        for pos in state.factions.relative(team).this_team.clone().iter_mesh() {
            let mesh = state.generate_possible_moves_movement(world, &pos, team);
            for mm in mesh.iter_mesh(pos) {
                //Temporarily move the player in the game world.
                //We do this so that the mesh generated for extra is accurate.
                let mut mmm = move_build::MovePhase {
                    original: pos,
                    moveto: mm,
                };

                let mut effect = mmm.apply(team, state, world);

                let second_mesh = state.generate_possible_moves_extra(world, &mmm, &effect, team);

                for sm in second_mesh.iter_mesh(mm) {
                    assert!(!state.env.terrain.is_set(sm));

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
