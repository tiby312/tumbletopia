use super::*;

//TODO use this?
pub trait MoveCost {
    fn foop(&self, a: GridCoord) -> MoveUnit;

    fn chain<B:MoveCost>(self,other:B)->Chain<Self,B> where Self:Sized{
        Chain{a:self,b:other}
    }
}

pub struct Chain<A,B>{
    a:A,
    b:B
}
impl<A:MoveCost,B:MoveCost> MoveCost for Chain<A,B>{
    fn foop(&self,g:GridCoord)->MoveUnit{
        let a=self.a.foop(g);
        let b=self.b.foop(g);
        MoveUnit(a.0+b.0)
    }
}




pub struct TerrainCollection{
    pub cost:MoveUnit,
    pub pos:Vec<GridCoord>
}
impl TerrainCollection {
    fn find_mut(&mut self, a: &GridCoord) -> Option<&mut GridCoord> {
        self.pos.iter_mut().find(|b| *b == a)
    }
    fn foo(&self)->TerrainCollectionFoo{
        TerrainCollectionFoo { cost:self.cost, a: &self.pos }
    }
}



pub struct TerrainCollectionFoo<'a> {
    cost:MoveUnit,
    a: &'a [GridCoord],
}
impl<'a> MoveCost for TerrainCollectionFoo<'a> {
    fn foop(&self, g: GridCoord) -> MoveUnit{
        if self.a.contains(&g){
            self.cost
        }else{
            MoveUnit(0)
        }
    }
}




//There is a base 2 cost movement everywhere.

//roads subtract 1

//mountains add cost
