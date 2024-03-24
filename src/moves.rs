use super::*;

use crate::movement::movement_mesh::SmallMesh;

impl GameState {
    fn check_if_occ(&self, a: Axial, check_fog: bool) -> bool {
        let game = self;
        let is_world_cell = game.world.get_game_cells().is_coord_set(a);

        let jjj = if check_fog {
            !game.env.fog.is_coord_set(a)
        } else {
            true
        };

        is_world_cell
            && !game.env.land.is_coord_set(a)
            && jjj
            && game.factions.dogs.find_slow(&a).is_none()
            && game.factions.cats.find_slow(&a).is_none()
    }

    pub fn generate_possible_moves_extra(
        &self,
        foo: &move_build::MovePhase,
        typ: Type,
        _team: ActiveTeam,
    ) -> SmallMesh {
        let game = self;
        let unit = foo.moveto;
        let original_pos = foo.original;
        let mut mesh = SmallMesh::new();

        for a in unit
            .to_cube()
            .ring(1)
            .chain(std::iter::once(original_pos.to_cube()))
        {
            let a = a.to_axial();

            if a != unit && game.check_if_occ(a, true) {
                mesh.add(a.sub(&unit));

                // for a in a.to_cube().ring(1) {
                //     let a = a.to_axial();

                //     if check_if_occ(a, true) {
                //         mesh.add(a.sub(&unit));
                //     }
                // }
            }
        }
        mesh
    }
    pub fn generate_possible_moves_movement(
        &self,
        &unit: &Axial,
        typ: Type,
        _team: ActiveTeam,
    ) -> SmallMesh {
        let game = self;
        let mut mesh = SmallMesh::new();
        for a in unit.to_cube().ring(1) {
            let a = a.to_axial();
            let dir = unit.dir_to(&a);

            if a != unit && game.check_if_occ(a, true) {
                mesh.add(a.sub(&unit));

                if typ.is_warrior() {
                    for b in a.to_cube().ring(1) {
                        let b = b.to_axial();

                        if b != unit && game.check_if_occ(b, true) {
                            mesh.add(b.sub(&unit));
                        }
                    }
                }
            } else {
                if let Type::Warrior { powerup } = typ {
                    if game.env.land.is_coord_set(a) {
                        let check = a.advance(dir);
                        if a != unit && game.check_if_occ(check, true) {
                            mesh.add(a.sub(&unit));
                        }
                    }
                }
            }
        }
        mesh
    }
}

#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct ActualMove {
    pub original: Axial,
    pub moveto: Axial,
    pub attackto: Axial,
}

impl GameState {
    pub fn for_all_moves_fast(&mut self, team: ActiveTeam) -> Vec<moves::ActualMove> {
        let state = self;
        let mut movs = Vec::new();
        for i in 0..state.factions.relative(team).this_team.units.len() {
            let pos = state.factions.relative_mut(team).this_team.units[i].position;
            let ttt = state.factions.relative_mut(team).this_team.units[i].typ;

            let mesh = state.generate_possible_moves_movement(&pos, ttt, team);
            for mm in mesh.iter_mesh(pos) {
                //Temporarily move the player in the game world.
                //We do this so that the mesh generated for extra is accurate.
                let mut mmm = move_build::MovePhase {
                    original: pos,
                    moveto: mm,
                };
                let effect = mmm.apply(team, state);

                let second_mesh = state.generate_possible_moves_extra(&mmm, ttt, team);

                for sm in second_mesh.iter_mesh(mm) {
                    assert!(!state.env.land.is_coord_set(sm));

                    let kkk = mmm.into_attack(sm);

                    let k = kkk.apply(team, state);

                    let mmo = moves::ActualMove {
                        original: pos,
                        moveto: mm,
                        attackto: sm,
                    };

                    movs.push(mmo);

                    mmm = kkk.undo(&k, state);
                }

                //revert it back just the movement component.
                mmm.undo(team, &effect, state);
            }
        }
        movs
    }
}
