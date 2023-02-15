use super::*;


pub async fn animate(a:animation::Animation<Warrior>){
    
}


pub struct Doop<'a, 'b> {
    pub team: usize,
    pub game: std::sync::Arc<futures::lock::Mutex<&'b mut Game>>,
    pub rx: &'a mut futures::channel::mpsc::Receiver<[f32; 2]>,
}

impl<'a, 'b> Doop<'a, 'b> {
    pub async fn get_possible_moves(
        &mut self,
    ) -> (futures::lock::OwnedMutexGuard<&'b mut Game>, CellSelection) {
        //Wait for user to click a unit and present move options to user
        loop {
            let mouse_world: [f32; 2] = self.rx.next().await.unwrap();
            let mut glock = self.game.clone().lock_owned().await;
            let cc = {
                let mut gg = &mut *glock;
                let [this_team, that_team] = team_view([&mut gg.cats, &mut gg.dogs], self.team);

                let cell: GridCoord =
                    GridCoord(gg.grid_matrix.to_grid((mouse_world).into()).into());

                let Some(unit)=this_team.find(&cell) else {
                                continue;
                            };

                if !unit.is_selectable() {
                    continue;
                }

                get_cat_move_attack_matrix(
                    unit,
                    this_team.filter().chain(that_team.filter()),
                    terrain::Grass,
                    &gg.grid_matrix,
                )
            };

            //At this point we have found the friendly unit the user clicked on.
            break (glock, cc);
        }
    }

    pub async fn pick_possible_move(&mut self) -> bool {
        //Wait for user to click on a move option and handle that.
        loop {
            let mouse_world: [f32; 2] = self.rx.next().await.unwrap();
            let mut gg1 = self.game.lock().await;
            let gg = &mut *gg1;
            let [this_team, that_team] = team_view([&mut gg.cats, &mut gg.dogs], self.team);

            let cell: GridCoord = GridCoord(gg.grid_matrix.to_grid((mouse_world).into()).into());

            let s = gg.selected_cells.as_mut().unwrap();

            match s {
                CellSelection::MoveSelection(ss, attack) => {
                    let target_cat_pos = &cell;

                    if movement::contains_coord(ss.iter_coords(), &cell) {
                        let mut c = this_team.remove(ss.start());
                        let (dd, aa) = ss.get_path_data(cell).unwrap();
                        c.position = cell;
                        c.move_deficit = *aa;
                        //c.moved = true;
                        gg.animation = Some(animation::Animation::new(
                            ss.start(),
                            dd,
                            &gg.grid_matrix,
                            c,
                        ));
                        gg.selected_cells = None;
                        return true;
                    } else {
                        gg.selected_cells = None;
                        return false;
                    }
                }
                _ => {
                    todo!()
                }
            }
        }
    }
}
