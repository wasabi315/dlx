use std::collections::HashSet;
use std::hash::Hash;

mod dlx;
mod node;
mod problem;
mod solver;

use node::NodeArena;
use problem::Problem;
use solver::Solver;

pub fn solve<L, T, S>(subsets: impl IntoIterator<Item = (L, HashSet<T, S>)>) -> Option<Vec<L>>
where
    T: Hash + Eq,
{
    let arena = NodeArena::new();
    let mut problem = Problem::new(&arena);

    for (label, subset) in subsets {
        problem.add_subset(label, subset);
    }

    Solver::new(problem).solve()
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! ecp {
        ($($label:expr => {$($elem:expr),*},)*) => {
            [$(($label, ::std::collections::HashSet::from([$($elem),*]))),*]
        };
    }

    #[test]
    fn test1() {
        let ecp = ecp! {
            'A' => {0, 3, 6},
            'B' => {0, 3},
            'C' => {3, 4, 6},
            'D' => {2, 4, 5},
            'E' => {1, 2, 5, 6},
            'F' => {1, 6},
        };
        assert_eq!(solve(ecp), Some(vec!['B', 'D', 'F']));
    }

    #[test]
    fn test2() {
        let ecp = ecp! {
           0 => {0, 2}, // *
           1 => {0, 3, 4},
           0 => {1, 3}, // *
           1 => {1, 4},
           0 => {2, 3},
           1 => {4}, // *
        };
        assert_eq!(solve(ecp), Some(vec![0, 0, 1]));
    }

    #[test]
    fn test3() {
        let ecp = ecp! {
            () => {0, 2},
            () => {0, 3, 4},
            () => {1},
            () => {1, 4},
            () => {2, 3},
            () => {4},
        };
        assert_eq!(solve(ecp), None);
    }

    #[test]
    fn test4() {
        let arena = NodeArena::new();
        let ecp = Problem::<(), ()>::new(&arena);
        let mut solutions = Solver::new(ecp).solutions();
        assert_eq!(solutions.next(), Some(vec![]));
        assert_eq!(solutions.next(), None);
    }

    #[test]
    fn test5() {
        let arena = NodeArena::new();
        let mut ecp = Problem::<i32, ()>::new(&arena);
        ecp.add_subset(1, HashSet::from([]));
        ecp.add_subset(2, HashSet::from([]));
        ecp.add_subset(3, HashSet::from([]));
        let mut solutions = Solver::new(ecp).solutions();
        assert_eq!(solutions.next(), Some(vec![]));
        assert_eq!(solutions.next(), None);
    }

    #[test]
    fn test6() {
        let arena = NodeArena::new();
        let mut ecp = Problem::<i32, ()>::new(&arena);
        ecp.add_subset(1, HashSet::from([()]));
        ecp.add_subset(2, HashSet::from([()]));
        ecp.add_subset(3, HashSet::from([()]));
        let mut solutions = Solver::new(ecp).solutions();
        assert_eq!(solutions.next(), Some(vec![1]));
        assert_eq!(solutions.next(), Some(vec![2]));
        assert_eq!(solutions.next(), Some(vec![3]));
        assert_eq!(solutions.next(), None);
    }

    #[test]
    fn test7() {
        let arena = NodeArena::new();
        let mut ecp = Problem::<i32, bool>::new(&arena);
        ecp.add_subset(1, HashSet::from([true]));
        ecp.add_subset(2, HashSet::from([true]));
        ecp.add_subset(3, HashSet::from([false]));
        ecp.add_subset(4, HashSet::from([false]));
        let mut solutions = Solver::new(ecp).solutions();
        assert_eq!(solutions.next(), Some(vec![1, 3]));
        assert_eq!(solutions.next(), Some(vec![1, 4]));
        assert_eq!(solutions.next(), Some(vec![2, 3]));
        assert_eq!(solutions.next(), Some(vec![2, 4]));
        assert_eq!(solutions.next(), None);
    }
}
