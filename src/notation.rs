
// pub fn from_foo(input: &str) -> Result<Vec<ActualMove>, std::fmt::Error> {
//     input
//         .split(",")
//         .filter(|a| *a != "")
//         .map(|a| {
//             dbg!(&a);
//             let mut s = a.chars();

//             match s.next().ok_or(std::fmt::Error)? {
//                 'N' => {
//                     let s = s.as_str();
//                     let mut k = s.split(":").map(|a| a.parse::<i16>());

//                     let mut foo = || {
//                         k.next()
//                             .ok_or(std::fmt::Error)?
//                             .map_err(|_| std::fmt::Error)
//                     };

//                     let unit = GridCoord([foo()?, foo()?]);
//                     let moveto = GridCoord([foo()?, foo()?]);
//                     todo!();
//                     //Ok(ActualMove::NormalMove(PartialMoveSigl { unit, moveto }))
//                 }
//                 'E' => {
//                     let s = s.as_str();
//                     let mut k = s.split(":").map(|a| a.parse::<i16>());
//                     let mut foo = || {
//                         k.next()
//                             .ok_or(std::fmt::Error)?
//                             .map_err(|_| std::fmt::Error)
//                     };
//                     let unit = GridCoord([foo()?, foo()?]);
//                     let moveto = GridCoord([foo()?, foo()?]);

//                     let unit2 = GridCoord([foo()?, foo()?]);
//                     let moveto2 = GridCoord([foo()?, foo()?]);
//                     Ok(ActualMove::ExtraMove(
//                         PartialMoveSigl { unit, moveto },
//                         PartialMoveSigl {
//                             unit: unit2,
//                             moveto: moveto2,
//                         },
//                     ))
//                 }
//                 // 'I' => {
//                 //     let s = s.as_str();
//                 //     let mut k = s.split(":").map(|a| a.parse::<i16>());
//                 //     let mut foo = || {
//                 //         k.next()
//                 //             .ok_or(std::fmt::Error)?
//                 //             .map_err(|_| std::fmt::Error)
//                 //     };

//                 //     let unit = GridCoord([foo()?, foo()?]);
//                 //     let moveto = GridCoord([foo()?, foo()?]);
//                 //     Ok(ActualMove::Invade(InvadeSigl { unit, moveto }))
//                 // }
//                 //'S' => Ok(ActualMove::SkipTurn),
//                 'F' => {
//                     let c = s.next().ok_or(std::fmt::Error)?;
//                     Ok(ActualMove::GameEnd(match c {
//                         'W' => GameEnding::Win(ActiveTeam::Cats),
//                         'B' => GameEnding::Win(ActiveTeam::Dogs),
//                         'D' => GameEnding::Draw,
//                         _ => return Err(std::fmt::Error),
//                     }))
//                 }
//                 _ => Err(std::fmt::Error),
//             }
//         })
//         .collect()
// }

// pub fn to_foo(a: &[ActualMove], mut f: impl std::fmt::Write) -> std::fmt::Result {
//     for a in a.iter() {
//         match a {
//             // ActualMove::Invade(i) => {
//             //     let a = i.unit.0;
//             //     let b = i.moveto.0;
//             //     write!(f, "I{}:{}:{}:{},", a[0], a[1], b[0], b[1])?;
//             // }
//             // ActualMove::NormalMove(i) => {
//             //     let a = i.unit.0;
//             //     let b = i.moveto.0;
//             //     write!(f, "N{}:{}:{}:{},", a[0], a[1], b[0], b[1])?;
//             // }
//             ActualMove::ExtraMove(i, j) => {
//                 let a = i.unit.0;
//                 let b = i.moveto.0;
//                 let c = j.unit.0;
//                 let d = j.moveto.0;
//                 write!(
//                     f,
//                     "E{}:{}:{}:{}:{}:{}:{}:{},",
//                     a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]
//                 )?;
//             }
//             ActualMove::SkipTurn => {
//                 write!(f, "S,")?;
//             }
//             ActualMove::GameEnd(g) => {
//                 let w = match g {
//                     GameEnding::Win(ActiveTeam::Cats) => "W",
//                     GameEnding::Win(ActiveTeam::Dogs) => "B",
//                     GameEnding::Draw => "D",
//                 };

//                 write!(f, "F{}", w)?;
//             }
//         }
//     }
//     Ok(())
// }