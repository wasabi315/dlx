use std::collections::HashSet;
use std::hash::Hash;

mod node;
mod problem;
mod dlx;
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
    use super::solve;

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
}
