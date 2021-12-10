use std::collections::VecDeque;

#[derive(Clone)]
pub struct HoleVec<T> {
    vec: VecDeque<T>,
    // Amount of values before the hole
    hole_position: usize,
}

impl<T> Default for HoleVec<T> {
    fn default() -> Self {
        Self {
            vec: VecDeque::default(),
            hole_position: 0,
        }
    }
}

impl<T> HoleVec<T> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: VecDeque::with_capacity(capacity),
            hole_position: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn len_before_hole(&self) -> usize {
        self.hole_position
    }

    pub fn len_after_hole(&self) -> usize {
        self.len() - self.hole_position
    }

    pub fn push_before_hole(&mut self, value: T) {
        self.vec.push_back(value);
        self.hole_position += 1;
    }

    pub fn push_after_hole(&mut self, value: T) {
        self.vec.push_front(value);
    }

    pub fn pop_before_hole(&mut self) -> Option<T> {
        (self.len_before_hole() > 0).then(|| {
            self.hole_position -= 1;
            self.vec.pop_back().expect("BUG")
        })
    }

    pub fn pop_after_hole(&mut self) -> Option<T> {
        (self.len_after_hole() > 0).then(|| self.vec.pop_front().expect("BUG"))
    }

    pub fn move_hole_right(&mut self, amount: usize) {
        assert!(amount <= self.len_after_hole());
        self.hole_position += amount;
        self.vec.rotate_left(amount);
    }

    pub fn move_hole_left(&mut self, amount: usize) {
        assert!(amount <= self.len_before_hole());
        self.hole_position -= amount;
        self.vec.rotate_right(amount);
    }

    pub fn set_hole_position(&mut self, position: usize) {
        assert!(position <= self.len());
        if position > self.hole_position {
            self.move_hole_right(position - self.hole_position);
        } else {
            self.move_hole_left(self.hole_position - position);
        }
    }

    pub fn as_slices(&self) -> (&[T], &[T], &[T]) {
        let (after_hole, before_hole) = self.vec.as_slices();
        if self.len_after_hole() <= after_hole.len() {
            let (end, start) = after_hole.split_at(self.len_after_hole());
            (start, before_hole, end)
        } else {
            let (end, start) = before_hole.split_at(before_hole.len() - self.len_before_hole());
            (start, after_hole, end)
        }
    }

    pub fn as_slices_before_hole(&self) -> (&[T], &[T]) {
        let (after_hole, before_hole) = self.vec.as_slices();
        if self.len_after_hole() <= after_hole.len() {
            let (_end, start) = after_hole.split_at(self.len_after_hole());
            (start, before_hole)
        } else {
            let (_end, start) = before_hole.split_at(before_hole.len() - self.len_before_hole());
            (start, &[])
        }
    }

    pub fn as_slices_after_hole(&self) -> (&[T], &[T]) {
        let (after_hole, before_hole) = self.vec.as_slices();
        if self.len_after_hole() <= after_hole.len() {
            let (end, _start) = after_hole.split_at(self.len_after_hole());
            (end, &[])
        } else {
            let (end, _start) = before_hole.split_at(before_hole.len() - self.len_before_hole());
            (after_hole, end)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};

    #[derive(Default, Debug)]
    struct Model<T> {
        before_hole: Vec<T>,
        after_hole: Vec<T>, // reversed order
    }

    impl<T> Model<T> {
        pub fn new() -> Self {
            Self {
                before_hole: Vec::new(),
                after_hole: Vec::new(),
            }
        }

        pub fn len(&self) -> usize {
            self.before_hole.len() + self.after_hole.len()
        }

        pub fn is_empty(&self) -> bool {
            self.len() != 0
        }

        pub fn len_before_hole(&self) -> usize {
            self.before_hole.len()
        }

        pub fn len_after_hole(&self) -> usize {
            self.after_hole.len()
        }

        pub fn push_before_hole(&mut self, value: T) {
            self.before_hole.push(value);
        }

        pub fn push_after_hole(&mut self, value: T) {
            self.after_hole.push(value);
        }

        pub fn pop_before_hole(&mut self) -> Option<T> {
            self.before_hole.pop()
        }

        pub fn pop_after_hole(&mut self) -> Option<T> {
            self.after_hole.pop()
        }

        pub fn move_hole_right(&mut self, amount: usize) {
            println!("moving right by {}", amount);
            for _ in 0..amount {
                self.before_hole.push(self.after_hole.pop().unwrap())
            }
        }

        pub fn move_hole_left(&mut self, amount: usize) {
            println!("moving left by {}", amount);
            for _ in 0..amount {
                self.after_hole.push(self.before_hole.pop().unwrap())
            }
        }

        pub fn set_hole_position(&mut self, position: usize) {
            assert!(position <= self.len());
            if position > self.before_hole.len() {
                self.move_hole_right(position - self.before_hole.len());
            } else {
                self.move_hole_left(self.before_hole.len() - position);
            }
        }

        pub fn iter(&self) -> impl Iterator<Item = &T> {
            self.before_hole().chain(self.after_hole())
        }

        pub fn before_hole(&self) -> impl Iterator<Item = &T> {
            self.before_hole.iter()
        }

        pub fn after_hole(&self) -> impl Iterator<Item = &T> {
            self.after_hole.iter().rev()
        }
    }

    #[derive(Copy, Clone, Debug)]
    enum Operation<T> {
        PushBefore(T),
        PushAfter(T),
        PopBefore,
        PopAfter,
        MoveLeft(usize),
        MoveRight(usize),
        SetPosition(usize),
    }

    impl<T: Copy + std::fmt::Debug + Eq> Operation<T>
    where
        rand::distributions::Standard: rand::distributions::Distribution<T>,
    {
        fn rand(rng: &mut impl Rng, model: &Model<T>) -> Self {
            match (rng.gen::<u64>() % 10, model.len() < 20) {
                (0, _) => Operation::PushBefore(rng.gen()),
                (1, _) => Operation::PushAfter(rng.gen()),
                (2, _) => Operation::PopBefore,
                (3, _) => Operation::PopAfter,
                (4, _) => Operation::MoveLeft(rng.gen::<usize>() % (1 + model.len_before_hole())),
                (5, _) => Operation::MoveRight(rng.gen::<usize>() % (1 + model.len_after_hole())),
                (6, _) => Operation::SetPosition(rng.gen::<usize>() % (1 + model.len())),
                (n, false) => {
                    if n % 2 == 0 {
                        Operation::PopBefore
                    } else {
                        Operation::PopAfter
                    }
                }
                (n, _) => {
                    if n % 2 == 0 {
                        Operation::PushBefore(rng.gen())
                    } else {
                        Operation::PushAfter(rng.gen())
                    }
                }
            }
        }

        fn apply(self, hole: &mut HoleVec<T>, model: &mut Model<T>) {
            match self {
                Operation::PushBefore(value) => {
                    hole.push_before_hole(value);
                    model.push_before_hole(value);
                }
                Operation::PushAfter(value) => {
                    hole.push_after_hole(value);
                    model.push_after_hole(value);
                }
                Operation::PopBefore => {
                    assert_eq!(hole.pop_before_hole(), model.pop_before_hole());
                }
                Operation::PopAfter => {
                    assert_eq!(hole.pop_after_hole(), model.pop_after_hole());
                }
                Operation::MoveLeft(amount) => {
                    hole.move_hole_left(amount);
                    model.move_hole_left(amount);
                }
                Operation::MoveRight(amount) => {
                    hole.move_hole_right(amount);
                    model.move_hole_right(amount);
                }
                Operation::SetPosition(pos) => {
                    hole.set_hole_position(pos);
                    model.set_hole_position(pos);
                }
            }

            assert_eq!(hole.len(), model.len());
            assert_eq!(hole.len_before_hole(), model.len_before_hole());
            assert_eq!(hole.len_after_hole(), model.len_after_hole());

            let (a0, a1) = hole.as_slices_before_hole();
            assert!(a0.iter().chain(a1.iter()).eq(model.before_hole()));

            let (a0, a1) = hole.as_slices_after_hole();
            assert!(a0.iter().chain(a1.iter()).eq(model.after_hole()));

            let (a0, a1, a2) = hole.as_slices();
            assert!(a0.iter().chain(a1.iter()).chain(a2.iter()).eq(model.iter()));
        }
    }

    fn run_test<T: std::fmt::Debug + Eq + Copy>()
    where
        rand::distributions::Standard: rand::distributions::Distribution<T>,
    {
        let mut rng = thread_rng();
        for _ in 0..1000 {
            let mut model = Model::<T>::new();
            let mut hole_vec = HoleVec::<T>::new();

            for _ in 0..1000 {
                println!("{:?}", model);
                println!("{:?}", hole_vec.as_slices());
                let operation = Operation::rand(&mut rng, &model);
                println!("{:?}", operation);
                operation.apply(&mut hole_vec, &mut model);
            }
        }
    }

    #[test]
    fn it_works() {
        run_test::<u8>();
        run_test::<u32>();
    }
}
