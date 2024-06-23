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
        let typ = self.factions.relative(team).this_team.get_type(unit);
        let game = self;
        let terrain = &game.env.terrain;

        let i = match typ {
            UnitType::Rook => {
                for (i, h) in hex::OFFSETS.into_iter().enumerate() {
                    for a in unit.to_cube().ray_from_vector(hex::Cube::from_arr(h)) {
                        assert!(unit != a.to_axial());
                        let a = a.ax;
                        if !world.get_game_cells().is_set(a)
                            || game.env.fog.is_set(a)
                            || terrain.is_set(a)
                            || game.factions.relative(team).this_team.is_set(a)
                        {
                            break;
                        }

                        if for_show || game.factions.relative(team).that_team.is_set(a) {
                            //mesh.add(a.sub(&unit));
                            mesh.add(a);
                        }

                        if game.factions.relative(team).that_team.is_set(a) {
                            break;
                        }
                    }
                }
                return;
            }
            UnitType::King => {
                for k in unit.to_cube().ring(1).map(|x| x.to_axial()) {
                // for k in hex::OFFSETS.into_iter().chain(hex::DIAG_OFFSETS.into_iter()) {
                //     let k=hex::Cube::from_arr(k).to_axial();
                //     let k=unit.add(k);
                    if world.get_game_cells().is_set(k)
                        && !game.env.fog.is_set(k)
                        && !terrain.is_set(k)
                        && !game.factions.relative(team).this_team.is_set(k)
                    {
                        mesh.add(k)
                    }
                }

                return;
            }
            UnitType::Pawn => {
                let dd = if let ActiveTeam::White = team { 3 } else { 0 };
                let k = unit.add(hex::Cube::from_arr(hex::OFFSETS[dd]).ax);

                if world.get_game_cells().is_set(k)
                    && !game.env.fog.is_set(k)
                    && !terrain.is_set(k)
                    && !game.factions.has_a_set(k)
                {
                    mesh.add(k);
                }

                for o in [hex::OFFSETS[dd + 1], hex::OFFSETS[(dd + 5) % 6]] {
                    let k = unit.add(hex::Cube::from_arr(o).ax);
                    if game.factions.relative(team).that_team.is_set(k) {
                        mesh.add(k);
                    }
                }

                return;
            }
            UnitType::Knight => {
                // for (i, h) in hex::OFFSETS.into_iter().enumerate() {
                //     let point = unit
                //         .to_cube()
                //         .ray_from_vector(hex::Cube::from_arr(h))
                //         .nth(1)
                //         .unwrap();

                //     let diags = [hex::OFFSETS[(i + 1) % 6], hex::OFFSETS[(i + 5) % 6]];

                //     for a in diags {
                //         let a = point.add(hex::Cube::from_arr(a)).ax;
                //         if world.get_game_cells().is_set(a)
                //             && !game.env.fog.is_set(a)
                //             && !terrain.is_set(a)
                //             && !game.factions.relative(team).this_team.is_set(a)
                //         {
                //             mesh.add(a);
                //         }
                //     }
                // }
                //let diags = [hex::OFFSETS[(i + 1) % 6], hex::OFFSETS[(i + 5) % 6]];
                let dd = if let ActiveTeam::White = team { 0} else { 3 };
                let k = unit.add(hex::Cube::from_arr(hex::OFFSETS[dd]).ax);

                if world.get_game_cells().is_set(k)
                && !game.env.fog.is_set(k)
                && !terrain.is_set(k)
                && !game.factions.has_a_set(k)
                {
                    mesh.add(k);
                }
                
                for a in hex::DIAG_OFFSETS {
                    let a = unit.add(hex::Cube::from_arr(a).ax);
                    if world.get_game_cells().is_set(a)
                        && !game.env.fog.is_set(a)
                        && !terrain.is_set(a)
                        && !game.factions.relative(team).this_team.is_set(a)
                    {
                        mesh.add(a);
                    }
                }
                return;
            }
            UnitType::Book1 => 0,
            UnitType::Book2 => 1,
            UnitType::Book3 => 2,
        };

        let j = i + 1;
        let k = [
            hex::OFFSETS[i],
            hex::OFFSETS[(i + 2) % 6],
            hex::OFFSETS[(i + 4) % 6],
            
            // hex::DIAG_OFFSETS[j],
            // hex::DIAG_OFFSETS[(j + 3) % 6],
        ]
        .map(hex::Cube::from_arr);

        for h in k {
            for a in unit.to_cube().ray_from_vector(h).take(15) {
                //for (a, _) in unit.to_cube().ray(h).skip(1).take(2) {
                assert!(unit != a.to_axial());
                let a = a.ax;
                if !world.get_game_cells().is_set(a)
                    || game.env.fog.is_set(a)
                    || terrain.is_set(a)
                    || game.factions.relative(team).this_team.is_set(a)
                {
                    break;
                }

                if for_show || game.factions.relative(team).that_team.is_set(a) {
                    //mesh.add(a.sub(&unit));
                    mesh.add(a);
                }

                if game.factions.relative(team).that_team.is_set(a) {
                    break;
                }
            }
        }
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

        let terrain = &game.env.terrain;

        let enemy_cover = {
            //TODO use a workspace instead
            let mut total = BitField::new();
            for a in self.factions.relative(team).that_team.iter_mesh() {
                let mut mesh = SmallMesh::new();
                self.attack_mesh_add(&mut mesh, world, &a, team.not(), true);
                for m in mesh.iter_mesh(a) {
                    total.set_coord(m, true);
                }
            }
            total
        };

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
        for pos in state.factions.relative(team).this_team.clone().iter_mesh() {
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
