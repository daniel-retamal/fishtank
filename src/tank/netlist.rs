use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visit {
    Unseen,
    Open,
    Done,
}

pub struct Settling {
    pub feed_forward: Vec<(usize, usize)>,
    pub order: Vec<usize>,
}

pub struct Netlist {
    listeners: Vec<Vec<usize>>,
}

impl Netlist {
    pub fn of<P: AsRef<[String]>, C: AsRef<[String]>>(produces: &[P], consumes: &[C]) -> Self {
        let mut consumers: HashMap<&str, Vec<usize>> = HashMap::new();
        for (index, lines) in consumes.iter().enumerate() {
            for line in lines.as_ref() {
                consumers.entry(line.as_str()).or_default().push(index);
            }
        }
        let listeners = produces
            .iter()
            .map(|lines| {
                lines
                    .as_ref()
                    .iter()
                    .filter_map(|line| consumers.get(line.as_str()))
                    .flatten()
                    .copied()
                    .collect::<BTreeSet<usize>>()
                    .into_iter()
                    .collect()
            })
            .collect();
        Self { listeners }
    }

    pub fn len(&self) -> usize {
        self.listeners.len()
    }

    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty()
    }

    pub fn settle(&self) -> Settling {
        let mut state = vec![Visit::Unseen; self.len()];
        let mut feed_forward = Vec::new();
        let mut finished = Vec::with_capacity(self.len());
        let mut stack: Vec<(usize, usize)> = Vec::new();
        for root in 0..self.len() {
            if state[root] != Visit::Unseen {
                continue;
            }
            state[root] = Visit::Open;
            stack.push((root, 0));
            while let Some(top) = stack.last_mut() {
                let (node, next) = *top;
                let Some(&listener) = self.listeners[node].get(next) else {
                    state[node] = Visit::Done;
                    finished.push(node);
                    stack.pop();
                    continue;
                };
                top.1 += 1;
                if state[listener] == Visit::Open {
                    continue;
                }
                feed_forward.push((node, listener));
                if state[listener] == Visit::Unseen {
                    state[listener] = Visit::Open;
                    stack.push((listener, 0));
                }
            }
        }
        finished.reverse();
        Settling {
            feed_forward,
            order: finished,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn position(order: &[usize], node: usize) -> usize {
        order
            .iter()
            .position(|&seen| seen == node)
            .expect("every node is ordered")
    }

    #[test]
    fn a_driver_is_ordered_before_everything_it_feeds_whatever_the_board_order() {
        let produces = [lines(&["c"]), lines(&["b"]), lines(&["a"])];
        let consumes = [lines(&["b"]), lines(&["a"]), lines(&["x"])];

        let settling = Netlist::of(&produces, &consumes).settle();

        assert_eq!(settling.order, vec![2, 1, 0]);
        assert_eq!(settling.feed_forward, vec![(1, 0), (2, 1)]);
    }

    #[test]
    fn the_edge_that_closes_a_loop_is_left_out_of_the_feed_forward_graph() {
        let produces = [lines(&["q"]), lines(&["qn"])];
        let consumes = [lines(&["r", "qn"]), lines(&["s", "q"])];

        let settling = Netlist::of(&produces, &consumes).settle();

        assert_eq!(settling.feed_forward, vec![(0, 1)]);
        assert_eq!(settling.order, vec![0, 1]);
    }

    #[test]
    fn a_fish_that_hears_itself_feeds_nothing_forward() {
        let produces = [lines(&["ring"])];
        let consumes = [lines(&["ring"])];

        let settling = Netlist::of(&produces, &consumes).settle();

        assert!(settling.feed_forward.is_empty());
        assert_eq!(settling.order, vec![0]);
    }

    #[test]
    fn every_feed_forward_edge_runs_down_the_order() {
        let produces = [
            lines(&["a"]),
            lines(&["b"]),
            lines(&["c"]),
            lines(&["a"]),
            lines(&["d"]),
        ];
        let consumes = [
            lines(&["c"]),
            lines(&["a"]),
            lines(&["b", "d"]),
            lines(&["x"]),
            lines(&["a"]),
        ];

        let settling = Netlist::of(&produces, &consumes).settle();

        for (driver, listener) in settling.feed_forward {
            assert!(position(&settling.order, driver) < position(&settling.order, listener));
        }
    }

    #[test]
    fn a_chain_ten_thousand_fish_deep_settles_without_recursing() {
        let depth = 10_000;
        let produces: Vec<Vec<String>> = (0..depth).map(|i| vec![format!("n{}", i + 1)]).collect();
        let consumes: Vec<Vec<String>> = (0..depth).map(|i| vec![format!("n{i}")]).collect();

        let settling = Netlist::of(&produces, &consumes).settle();

        assert_eq!(settling.feed_forward.len(), depth - 1);
        assert_eq!(settling.order, (0..depth).collect::<Vec<_>>());
    }
}
