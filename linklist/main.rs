enum List {
    Cons(u64, Box<List>),
    NULL,
}

impl List {
    fn new() -> List {
        List::NULL
    }

    fn prepend(self, elem: u64) -> List {
        List::Cons(elem, Box::new(self))
    }

    fn len(&self) -> u64 {
        let mut t = self;
        let mut res = 0u64;
        while let List::Cons(_, ref next) = *t {
            res += 1;
            t = next;
        }

        res
    }

    fn stringify(&self) -> String {
        let mut t = self;
        let mut res = String::new();
        while let List::Cons(r, ref next) = *t {
            res.push_str(&format!("{}==>", r));
            t = next;
        }

        res.push_str(&format!("NULL"));

        res
    }
}

fn main() {
    let mut list = List::new();

    for x in 0..10000 {
        list = list.prepend(x);
    }

    println!("{}", list.stringify());
    println!("linked list has length: {}", list.len());
}
