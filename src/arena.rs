#[derive(Debug, Clone)]
pub struct Arena<T: Clone> {
    a: Vec<T>,
    b: Vec<T>,
    current: Heap,
}

impl<T: Clone> Arena<T> {
    pub fn new() -> Arena<T> {
        Arena {
            a: Vec::new(),
            b: Vec::new(),
            current: Heap::A,
        }
    }

    pub fn get(&self, index: usize) -> &T {
        match self.current {
            Heap::A => &self.a[index],
            Heap::B => &self.b[index],
        }
    }

    pub fn len(&self) -> usize {
        match self.current {
            Heap::A => self.a.len(),
            Heap::B => self.b.len(),
        }
    }

    pub fn push(&mut self, item: T) {
        match self.current {
            Heap::A => self.a.push(item),
            Heap::B => self.b.push(item),
        }
    }

    #[allow(dead_code)]
    fn clean(&mut self) {
        let clean = |a: &mut Vec<T>, b: &mut Vec<T>| -> Vec<T> {
            for item in a {
                b.push(item.clone());
            }

            Vec::new()
        };

        match self.current {
            Heap::A => self.a = clean(&mut self.a, &mut self.b),
            Heap::B => self.b = clean(&mut self.b, &mut self.a),
        }

        self.current = self.current.next();
    }
}

#[derive(Debug, Clone, Copy)]
enum Heap {
    A,
    B,
}

impl Heap {
    fn next(&self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}
