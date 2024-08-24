use std::collections::btree_map::Keys;

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

    // pub fn generate_possible_moves_extra(
    //     &self,
    //     world: &board::MyWorld,
    //     foo: &move_build::MovePhase,
    //     effect: &move_build::MoveEffect,
    //     _team: ActiveTeam,
    // ) -> SmallMesh {
    //     let game = self;
    //     let unit = foo.moveto;
    //     let original_pos = foo.original;
    //     let mut mesh = SmallMesh::new();

    //     mesh.add(Axial::zero());

    //     // if effect.destroyed_unit.is_some() {
    //     //     mesh.add(Axial::zero())
    //     // } else {
    //     //     let tt = self.factions.has_a_set_type(unit).unwrap();

    //     //     // match tt {
    //     //     //     UnitType::Mouse => {
    //     //     //         for a in unit.to_cube().neighbours2() {
    //     //     //             let a = a.to_axial();

    //     //     //             if a != unit
    //     //     //                 && world.get_game_cells().is_set(a)
    //     //     //                 && !game.factions.has_a_set(a)
    //     //     //                 && !game.env.terrain.is_set(a)
    //     //     //                 && !game.env.fog.is_set(a)
    //     //     //             {
    //     //     //                 mesh.add(a.sub(&unit));

    //     //     //                 // for a in a.to_cube().ring(1) {
    //     //     //                 //     let a = a.to_axial();

    //     //     //                 //     if check_if_occ(a, true) {
    //     //     //                 //         mesh.add(a.sub(&unit));
    //     //     //                 //     }
    //     //     //                 // }
    //     //     //             }
    //     //     //         }
    //     //     //     }
    //     //     //     UnitType::Rabbit => mesh.add(foo.original.sub(&unit)),
    //     //     // }
    //     //     mesh.add(foo.original.sub(&unit))
    //     // }
    //     mesh
    // }

    pub fn generate_possible_moves_movement(
        &self,
        world: &board::MyWorld,
        unit: Option<Axial>,
        team: ActiveTeam,
    ) -> SmallMesh {
        let game = self;
        let mut mesh = SmallMesh::new();


        // fn check_a_unit(unit:Axial,val:usize,team:ActiveTeam,world:&board::MyWorld,game:&GameState,mut func:impl FnMut(Axial))->(usize,impl Iterator<Item=Axial>){
        //     //let sm=SmallMesh::new();

        //     let mut enemy_end_points:Vec<Axial>=Vec::new();
        //     let mut num_friendlies_in_sight = 0;
        //     for h in hex::OFFSETS.into_iter() {
        //         for k in unit.to_cube().ray_from_vector(hex::Cube::from_arr(h)) {
        //             let k = k.to_axial();
        //             if !world.get_game_cells().is_set(k) {
        //                 break;
        //             }

        //             if let Some((a, b)) = game.factions.cells.get_cell(k) {
        //                 if b == team {
        //                     num_friendlies_in_sight += 1;
        //                 }else{
        //                     enemy_end_points.push(k);
        //                 }

        //                 break;
        //             }

        //             func(k)
        //         }
        //     }

        //     // if num_friendlies_in_sight > val {
        //     //     mesh.add(unit);
        //     // }
        //     (num_friendlies_in_sight,enemy_end_points.into_iter())

        // }

        fn iterate_out(){

        }

        let for_ray=|unit:Axial,dir:[i8;3]|{
            unit.to_cube().ray_from_vector(hex::Cube::from_arr(dir)).take_while(|k|{
                let k = k.to_axial();
                world.get_game_cells().is_set(k)
            }).map(|x|x.to_axial())
        };


        let func=|unit:Axial,mesh:&mut SmallMesh,val:usize|{
            
            let mut num_friendlies_in_sight = 0;
            for h in hex::OFFSETS.into_iter() {
                for k in for_ray(unit,h){
                
                    if let Some((_, b)) = game.factions.cells.get_cell(k) {
                        if b == team {
                            num_friendlies_in_sight += 1;
                        }

                        break;
                    }

                    mesh.add(k);
                }
            }

            if num_friendlies_in_sight > val {
                mesh.add(unit);
            }

        };


        //if let Some(unit)=unit{
            // let (val, tt) = self.factions.cells.get_cell(unit).unwrap();

            // let (num_friendlies_in_sight,end)=check_a_unit(unit,val,team,world,self,|a|mesh.add(a));

            // if num_friendlies_in_sight>val{
            //     mesh.add(unit);
            // }

            // for e in end{

            // }
        
        //}else
        {

            for ho in world.get_game_cells().iter_mesh() {
                if let Some((val, tt)) = self.factions.cells.get_cell(ho) {
                    assert!(val > 0);
                    if tt == team {
                        func(ho,&mut mesh,val);
                    } else {
                        let unit = ho;
                        let mut num_friendlies_in_sight = 0;
    
                        for h in hex::OFFSETS.into_iter() {
                            for k in unit.to_cube().ray_from_vector(hex::Cube::from_arr(h)) {
                                let k = k.to_axial();
                                if !world.get_game_cells().is_set(k) {
                                    break;
                                }
    
                                if let Some((_, b)) = game.factions.cells.get_cell(k) {
                                    if b == team {
                                        num_friendlies_in_sight += 1;
                                    }
    
                                    break;
                                }
    
                            }
                        }
    
                        if num_friendlies_in_sight > val {
                            mesh.add(ho);
                        }
                    }
                }
            }
        }


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
    //pub original: Axial,
    pub moveto: Axial,
    //pub attackto: Axial,
}

impl GameState {
    pub fn for_all_moves_fast(
        &mut self,
        team: ActiveTeam,
        world: &board::MyWorld,
        mut func: impl FnMut(&move_build::MoveEffect, moves::ActualMove, &GameState),
    ) {
        //let state = self;

        for mm in self.generate_possible_moves_movement(world, None, team).iter_mesh(Axial::zero()){
            let mut mmm = move_build::MovePhase {
                moveto: mm,
            };

            let mut effect = mmm.apply(team, self, world);

            let mmo = moves::ActualMove {
                moveto: mm,
            };
            
            func(&effect,mmo,self);

            mmm.undo(team,&effect,self);
        }



        //let mut movs = Vec::new();
        //for i in 0..state.factions.relative(team).this_team.units.len() {
        // for pos in state.factions.relative(team).this_team.clone().iter_mesh() {
        //     let mesh = state.generate_possible_moves_movement(world, &pos, team);
        //     for mm in mesh.iter_mesh(pos) {
        //         //Temporarily move the player in the game world.
        //         //We do this so that the mesh generated for extra is accurate.
        //         let mut mmm = move_build::MovePhase {
        //             original: pos,
        //             moveto: mm,
        //         };

        //         let mut effect = mmm.apply(team, state, world);

        //         let second_mesh = state.generate_possible_moves_extra(world, &mmm, &effect, team);

        //         for sm in second_mesh.iter_mesh(mm) {
        //             assert!(!state.env.terrain.is_set(sm));

        //             let kkk = mmm.into_attack(sm);

        //             let k = kkk.apply(team, state, world, &effect);

        //             let mmo = moves::ActualMove {
        //                 original: pos,
        //                 moveto: mm,
        //                 attackto: sm,
        //             };

        //             let jjj = effect.combine(k);

        //             func(jjj.clone(), mmo, state);

        //             mmm = kkk.undo(&jjj.extra_effect, state);
        //             effect = jjj.move_effect;
        //         }

        //         //revert it back just the movement component.
        //         mmm.undo(team, &effect, state);
        //     }
        // }

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
