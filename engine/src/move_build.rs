use super::*;

impl crate::moves::ActualMove {
    // pub fn as_extra(&self) -> move_build::ExtraPhase {
    //     move_build::ExtraPhase {
    //         original: self.original,
    //         moveto: self.moveto,
    //         target: self.attackto,
    //     }
    // }
    // pub fn as_move(&self) -> move_build::MovePhase {
    //     move_build::MovePhase {
    //         //original: self.original,
    //         moveto: self.moveto,
    //     }
    // }
}

// pub struct ExtraPhase {
//     pub original: Axial,
//     pub moveto: Axial,
//     pub target: Axial,
// }
// impl ExtraPhase {
//     pub fn undo(self, meta: &ExtraEffect, state: &mut GameState) -> MovePhase {
//         let moveto = self.moveto;
//         let unit = self.original;
//         let attackto = self.target;

//         for a in meta.fog.0.iter_mesh(moveto) {
//             assert!(!state.env.fog.is_set(a));
//             state.env.fog.set_coord(a, true);
//         }

//         if moveto != attackto {
//             if let Some(m) = &meta.bomb {
//                 assert_eq!(unit, attackto);
//                 assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 2);
//                 for a in m.0.iter_mesh(unit) {
//                     assert!(state.env.terrain.land.is_set(a));
//                     state.env.terrain.land.set_coord(a, false);
//                 }
//             } else if state.env.terrain.forest.is_set(attackto) {
//                 state.env.terrain.forest.set_coord(attackto, false);
//             } else if state.env.terrain.land.is_set(attackto) {
//                 state.env.terrain.land.set_coord(attackto, false);
//             } else {
//                 unreachable!();
//             }
//         }

//         MovePhase {
//             //original: self.original,
//             moveto: self.moveto,
//         }
//     }
//     //returns a mesh where set bits indicate cells
//     //that were fog before this function was called,
//     //and were then unfogged.
//     fn compute_bomb(&self, game: &GameState, world: &board::MyWorld) -> Option<BombInfo> {
//         // if self.target != self.original || self.original.to_cube().dist(&self.moveto.to_cube()) != 2
//         // {
//         //     return None;
//         // }

//         // let mut mesh = SmallMesh::new();

//         // for a in self.original.to_cube().range(2).map(|a| a.to_axial()) {
//         //     if !world.get_game_cells().is_set(a) {
//         //         continue;
//         //     }

//         //     if game.factions.has_a_set(a) {
//         //         continue;
//         //     }

//         //     if game.env.terrain.is_set(a) {
//         //         continue;
//         //     }

//         //     if game.env.fog.is_set(a) {
//         //         continue;
//         //     }

//         //     mesh.add(a.sub(&self.original));
//         // }

//         // Some(BombInfo(mesh))
//         return None;
//     }

//     pub fn apply(
//         &self,
//         _team: ActiveTeam,
//         game: &mut GameState,
//         world: &board::MyWorld,
//         mov_eff: &MoveEffect,
//     ) -> ExtraEffect {
//         let original = self.original;
//         let moveto = self.moveto;
//         let target_cell = self.target;

//         if self.moveto == self.target {
//             let fog = compute_fog(moveto, &mut game.env);

//             fog.apply(moveto, &mut game.env);

//             return ExtraEffect { fog, bomb: None };
//         }

//         let bb = if let Some(bb) = self.compute_bomb(game, world) {
//             bb.apply(original, game);
//             Some(bb)
//         } else {
//             if !game.env.terrain.land.is_set(target_cell) {
//                 game.env.terrain.land.set_coord(target_cell, true)
//             } else {
//                 // if !env.forest.is_coord_set(target_cell) {
//                 //     env.forest.set_coord(target_cell, true);
//                 // }
//                 unreachable!("WAT");
//             }
//             None
//         };

//         // let bb = if target_cell == original && original.to_cube().dist(&moveto.to_cube()) == 2 {
//         //     //if false{
//         //     let bb = compute_bomb(original, game);
//         //     bb.apply(original, game);
//         //     Some(bb)
//         // } else {
//         //     if !game.env.land.is_coord_set(target_cell) {
//         //         game.env.land.set_coord(target_cell, true)
//         //     } else {
//         //         // if !env.forest.is_coord_set(target_cell) {
//         //         //     env.forest.set_coord(target_cell, true);
//         //         // }
//         //         unreachable!("WAT");
//         //     }
//         //     None
//         // };

//         let mut fog = compute_fog(moveto, &mut game.env);

//         if let PushInfo::PushedUnit = mov_eff.pushpull {
//             let dir = original.dir_to(&moveto);
//             let check = moveto.advance(dir);
//             let fog2 = compute_fog(check, &mut game.env);

//             for f in fog2.0.iter_mesh(check) {
//                 fog.0.add(f.sub(&moveto));
//             }
//         }

//         fog.apply(moveto, &mut game.env);

//         ExtraEffect { fog, bomb: bb }
//     }

//     pub async fn animate(
//         &self,
//         team: ActiveTeam,
//         state: &GameState,
//         world: &board::MyWorld,
//         data: &mut ace::WorkerManager,
//     ) -> &Self {
//         // let target = self.target;

//         // let mut gg = state.clone();

//         // if let Some(bb) = self.compute_bomb(state, world) {
//         //     let k = self.original.to_cube();
//         //     for a in std::iter::once(k).chain(k.ring(1)).chain(k.ring(2)) {
//         //         if bb.0.is_set(a.sub(self.original.to_cube()).to_axial()) {
//         //             data.wait_animation(
//         //                 animation::AnimationCommand::Terrain {
//         //                     pos: a.to_axial(),
//         //                     terrain_type: animation::TerrainType::Grass,
//         //                     dir: animation::AnimationDirection::Up,
//         //                 },
//         //                 team,
//         //                 &mut gg,
//         //             )
//         //             .await;
//         //             gg.env.terrain.land.set_coord(a.to_axial(), true);
//         //         }
//         //     }
//         // } else {
//         //     data.wait_animation(
//         //         animation::AnimationCommand::Terrain {
//         //             pos: target,
//         //             terrain_type: animation::TerrainType::Grass,
//         //             dir: animation::AnimationDirection::Up,
//         //         },
//         //         team,
//         //         &mut gg,
//         //     )
//         //     .await;

//         //     gg.env.terrain.land.set_coord(target, true);
//         // }

//         // let fog = compute_fog(self.moveto, &state.env);

//         // //let mut game = state.clone();
//         // for a in fog.0.iter_mesh(self.moveto) {
//         //     gg.env.fog.set_coord(a, false);
//         //     // Change mesh
//         //     data.wait_animation(
//         //         animation::AnimationCommand::Terrain {
//         //             pos: a,
//         //             terrain_type: animation::TerrainType::Fog,
//         //             dir: animation::AnimationDirection::Down,
//         //         },
//         //         team,
//         //         &mut gg,
//         //     )
//         //     .await;
//         // }

//         self
//     }
// }

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub struct MoveEffect {
    pushpull: PushInfo,
    powerup: PowerupAction,
    pub destroyed_unit: Option<(u8, ActiveTeam)>,
}
impl MoveEffect {
    // pub fn combine(self, extra_effect: ExtraEffect) -> CombinedEffect {
    //     CombinedEffect {
    //         move_effect: self,
    //         extra_effect,
    //     }
    // }
}

// #[derive(Clone, Debug)]
// pub struct MovePhase {
//     //pub original: Axial,
//     pub moveto: Axial,
// }

#[test]
fn test_move() {
    let k = mesh::small_mesh::inverse(ActualMove::from_str("A1").unwrap().moveto);
    assert_eq!(k, Axial { q: 0, r: -7 });

    let k = mesh::small_mesh::inverse(ActualMove::from_str("H15").unwrap().moveto);
    assert_eq!(k, Axial { q: 7, r: 0 });
}
impl ActualMove {
    pub fn from_str(foo: &str) -> Option<ActualMove> {
        let mut char_iter = foo.chars();

        let Some(letter) = char_iter.next() else {
            return None;
        };

        let r = match letter {
            'A' => -7,
            'B' => -6,
            'C' => -5,
            'D' => -4,
            'E' => -3,
            'F' => -2,
            'G' => -1,
            'H' => 0,
            'I' => 1,
            'J' => 2,
            'K' => 3,
            'L' => 4,
            'M' => 5,
            'N' => 6,
            'O' => 7,
            _ => return None,
        };

        let Some(first_digit) = char_iter.next() else {
            return None;
        };

        let s = if let Some(second_digit) = char_iter.next() {
            let mut s = String::new();
            s.push(first_digit);
            s.push(second_digit);
            let Ok(foo) = u8::from_str_radix(&s, 10) else {
                return None;
            };
            foo
        } else {
            let Ok(foo) = u8::from_str_radix(&first_digit.to_string(), 10) else {
                return None;
            };
            foo
        };

        dbg!(s);

        let s = -(s as i8 - 1 - 7);

        dbg!(s);

        //q+r+s=0
        let q = -r - s;

        Some(ActualMove {
            moveto: mesh::small_mesh::conv(Axial { q, r }),
        })
    }
    pub fn as_text(&self, mut w: impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        let k = mesh::small_mesh::inverse(self.moveto).to_cube();

        let s = k.s;
        let r = k.r;

        let first = match r {
            -7 => 'A',
            -6 => 'B',
            -5 => 'C',
            -4 => 'D',
            -3 => 'E',
            -2 => 'F',
            -1 => 'G',
            0 => 'H',
            1 => 'I',
            2 => 'J',
            3 => 'K',
            4 => 'L',
            5 => 'M',
            6 => 'N',
            7 => 'O',
            _ => '?',
        };

        write!(w, "{}{}", first, (-s + 7) + 1)
    }

    // pub fn into_attack(self, target: Axial) -> ExtraPhase {
    //     ExtraPhase {
    //         original: self.original,
    //         moveto: self.moveto,
    //         target,
    //     }
    // }

    pub fn undo(&self, _team_index: ActiveTeam, effect: &MoveEffect, state: &mut GameState) {
        let moveto = self.moveto;

        if moveto == moves::PASS_MOVE_INDEX {
            return;
        }

        //let unit = self.original;

        if let Some((fooo, typ)) = effect.destroyed_unit {
            state.factions.add_cell_inner(moveto, fooo, typ);
        } else {
            state.factions.remove_inner(moveto)
        };

        // match effect.pushpull {
        //     PushInfo::UpgradedLand => {
        //         assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 1);

        //         let dir = unit.dir_to(&moveto);
        //         let t3 = moveto.advance(dir);

        //         if state.env.terrain.land.is_set(t3) {
        //             panic!("This is impossible!");
        //         } else if state.env.terrain.forest.is_set(t3) {
        //             state.env.terrain.forest.set_coord(t3, false);
        //             state.env.terrain.land.set_coord(t3, true);
        //             state.env.terrain.land.set_coord(moveto, true);
        //         } else if state.env.terrain.mountain.is_set(t3) {
        //             state.env.terrain.mountain.set_coord(t3, false);
        //             state.env.terrain.forest.set_coord(t3, true);
        //             state.env.terrain.forest.set_coord(moveto, true);
        //         }
        //     }
        //     PushInfo::PushedUnit => {
        //         todo!()
        //         // assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 1);
        //         // let dir = unit.dir_to(&moveto);
        //         // let t3 = moveto.advance(dir);

        //         // let tt = state.factions.relative_mut(team_index);
        //         // if tt.this_team.units.is_set(t3) {
        //         //     tt.this_team.units.set_coord(t3, false);
        //         //     tt.this_team.units.set_coord(moveto, true);
        //         // } else if tt.that_team.units.is_set(t3) {
        //         //     tt.that_team.units.set_coord(t3, false);
        //         //     tt.that_team.units.set_coord(moveto, true);
        //         // } else {
        //         //     unreachable!("PushedUnit enum error");
        //         // }
        //     }
        //     PushInfo::PushedLand => {
        //         assert_eq!(unit.to_cube().dist(&moveto.to_cube()), 1);

        //         let dir = unit.dir_to(&moveto);
        //         let t3 = moveto.advance(dir);

        //         if state.env.terrain.land.is_set(t3) {
        //             state.env.terrain.land.set_coord(t3, false);
        //             state.env.terrain.land.set_coord(moveto, true);
        //         } else if state.env.terrain.forest.is_set(t3) {
        //             state.env.terrain.forest.set_coord(t3, false);
        //             state.env.terrain.forest.set_coord(moveto, true);
        //         } else if state.env.terrain.mountain.is_set(t3) {
        //             state.env.terrain.mountain.set_coord(t3, false);
        //             state.env.terrain.mountain.set_coord(moveto, true);
        //         }

        //         // assert!(state.env.terrain.land.is_set(t3));
        //         // state.env.terrain.land.set_coord(t3, false);
        //         // assert!(!state.env.terrain.land.is_set(moveto));
        //         // state.env.terrain.land.set_coord(moveto, true);
        //     }

        //     PushInfo::None => {}
        // }
    }

    pub fn apply(
        &self,
        team: ActiveTeam,
        game: &mut GameState,
        world: &board::MyWorld,
    ) -> MoveEffect {
        //this is a pass
        if self.moveto == moves::PASS_MOVE_INDEX {
            return MoveEffect {
                pushpull: PushInfo::None,
                powerup: PowerupAction::None,
                destroyed_unit: None,
            };
        }

        //let env = &mut game.env;
        let target_cell = self.moveto;
        let e = PushInfo::None;

        let mut stack_size = 0;

        for (_, rest) in game.factions.iter_end_points(world, target_cell) {
            if let Some((_, tt)) = rest {
                if tt == team {
                    stack_size += 1;
                }
            }
        }

        // for (i, h) in hex::OFFSETS.into_iter().enumerate() {
        //     for k in target_cell
        //         .to_cube()
        //         .ray_from_vector(hex::Cube::from_arr(h))
        //     {
        //         let k = k.to_axial();
        //         if !world.get_game_cells().is_set(k) {
        //             break;
        //         }

        //         if let Some((vv, team2)) = game.factions.cells.get_cell(k) {
        //             if team2 == team {
        //                 stack_size += 1;
        //             }
        //             break;
        //         }
        //     }
        // }

        //console_dbg!("Adding stacksize=", stack_size);

        let destroyed_unit = if let Some((a, v)) = game.factions.get_cell_inner(target_cell) {
            Some((a, v))
        } else {
            None
        };
        game.factions.remove_inner(target_cell);
        game.factions.add_cell_inner(target_cell, stack_size, team);

        MoveEffect {
            pushpull: e,
            powerup: PowerupAction::None,
            destroyed_unit,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
pub enum PowerupAction {
    GotPowerup,
    DiscardedPowerup,
    None,
}

#[derive(Serialize, Deserialize, PartialOrd, Ord, Clone, Copy, Eq, PartialEq, Debug)]
pub enum PushInfo {
    UpgradedLand,
    PushedLand,
    PushedUnit,
    None,
}

// #[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
// pub struct CombinedEffect {
//     pub move_effect: MoveEffect,
//     pub extra_effect: ExtraEffect,
// }

// #[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
// struct BombInfo(pub SmallMesh);
// impl BombInfo {
//     fn apply(&self, original: Axial, game: &mut GameState) {
//         for a in self.0.iter_mesh(Axial::zero()) {
//             game.env.terrain.land.set_coord(original.add(a), true);
//         }
//     }
// }

// #[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
// pub struct ExtraEffect {
//     fog: FogInfo,
//     bomb: Option<BombInfo>,
// }

// #[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone)]
// pub struct FogInfo(pub SmallMesh);

// impl FogInfo {
//     pub fn apply(&self, og: Axial, env: &mut Environment) {
//         for a in self.0.iter_mesh(Axial::zero()) {
//             env.fog.set_coord(og.add(a), false);
//         }
//     }
// }

// //returns a mesh where set bits indicate cells
// //that were fog before this function was called,
// //and were then unfogged.
// pub fn compute_fog(og: Axial, env: &Environment) -> FogInfo {
//     let mut mesh = SmallMesh::new();
//     for a in og.to_cube().range(1) {
//         if env.fog.is_set(a.to_axial()) {
//             mesh.add(a.to_axial().sub(&og));
//         }
//     }
//     FogInfo(mesh)
// }

// fn calculate_paths(
//     position: Axial,
//     target: Axial,
//     state: &GameState,
//     world: &board::MyWorld,
// ) -> SmallMesh {
//     let typ = state.factions.has_a_set_type(position).unwrap();

//     let env = &state.env;
//     let mut paths = SmallMesh::new();

//     paths.add(target.sub(&position));

//     for a in position.to_cube().range(2) {
//         let a = a.to_axial();
//         //TODO this is duplicated logic in selection function???

//         if !env.fog.is_set(a)
//             && !env.terrain.is_set(a)
//             && a != position
//             && !state.factions.has_a_set(a)
//             && world.get_game_cells().is_set(a)
//         {
//             //if a != target {
//             paths.add(a.sub(&position));
//             //}
//         }
//     }
//     // match typ {
//     //     UnitType::Mouse => {
//     //         paths.add(target.sub(&position));

//     //         for a in position.to_cube().range(2) {
//     //             let a = a.to_axial();
//     //             //TODO this is duplicated logic in selection function???

//     //             if !env.fog.is_set(a)
//     //                 && !env.terrain.is_set(a)
//     //                 && a != position
//     //                 && !state.factions.has_a_set(a)
//     //                 && world.get_game_cells().is_set(a)
//     //             {
//     //                 //if a != target {
//     //                 paths.add(a.sub(&position));
//     //                 //}
//     //             }
//     //         }
//     //     }
//     //     UnitType::Rabbit => {
//     //         paths.add(target.sub(&position));

//     //         for a in position.to_cube().range(2) {
//     //             let a = a.to_axial();
//     //             //TODO this is duplicated logic in selection function???

//     //             if env.terrain.is_set(a) {
//     //                 paths.add(a.sub(&position));
//     //             }
//     //         }

//     //         let pos: Vec<_> = paths.iter_mesh(position).collect();

//     //         console_dbg!("walls size={}", pos);
//     //     }
//     // }

//     paths
// }
