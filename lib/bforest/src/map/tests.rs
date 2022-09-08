use std::{fmt, mem};

use super::*;

impl<K, V> Map<K, V>
where
    K: Copy + fmt::Display,
    V: Copy,
{
    // /// Verify consistency.
    // fn verify<C: Comparator<K>>(self, forest: &MapForest<K, V>, comp: &C)
    // where
    //     NodeData<MapTypes<K, V>>: fmt::Display,
    // {
    //     if let Some(root) = self.root.expand() {
    //         forest.nodes.verify_tree(root, comp);
    //     }
    // }

    /// Get a text version of the path to `key`.
    fn tpath<C: Comparator<K>>(self, key: K, forest: &MapForest<K, V>, comp: &C) -> String {
        match self.root.expand() {
            None => "map(empty)".to_string(),
            Some(root) => {
                let mut path = Path::default();
                path.find(key, root, &forest.nodes, comp);
                path.to_string()
            }
        }
    }
}

impl<'a, K, V, C> MapCursor<'a, K, V, C>
where
    K: Copy + fmt::Display,
    V: Copy + fmt::Display,
    C: Comparator<K>,
{
    fn verify(&self) {
        self.path.verify(self.pool);
        // self.root.map(|root| self.pool.verify_tree(root, self.comp));
    }

    /// Get a text version of the path to the current position.
    fn tpath(&self) -> String {
        self.path.to_string()
    }
}

#[test]
fn node_size() {
    // check that nodes are cache line sized when keys and values are 32 bits.
    type F = MapTypes<u32, u32>;
    assert_eq!(mem::size_of::<NodeData<F>>(), 64);
}

#[test]
fn empty() {
    let mut f = MapForest::<u32, f32>::new();
    f.clear();

    let mut m = Map::<u32, f32>::new();
    assert!(m.is_empty());
    m.clear(&mut f);

    assert_eq!(m.get(7, &f, &()), None);
    assert_eq!(m.iter(&f).next(), None);
    assert_eq!(m.get_or_less(7, &f, &()), None);
    m.retain(&mut f, |_, _| unreachable!());

    let mut c = m.cursor(&mut f, &());
    assert!(c.is_empty());
    assert_eq!(c.key(), None);
    assert_eq!(c.value(), None);
    assert_eq!(c.next(), None);
    assert_eq!(c.prev(), None);
    c.verify();
    assert_eq!(c.tpath(), "<empty path>");
    assert_eq!(c.goto_first(), None);
    assert_eq!(c.tpath(), "<empty path>");
}

#[test]
fn inserting() {
    let f = &mut MapForest::<u32, f32>::new();
    let mut m = Map::<u32, f32>::new();

    // The first seven values stay in a single leaf node.
    assert_eq!(m.insert(50, 5.0, f, &()), None);
    assert_eq!(m.insert(50, 5.5, f, &()), Some(5.0));
    assert_eq!(m.insert(20, 2.0, f, &()), None);
    assert_eq!(m.insert(80, 8.0, f, &()), None);
    assert_eq!(m.insert(40, 4.0, f, &()), None);
    assert_eq!(m.insert(60, 6.0, f, &()), None);
    assert_eq!(m.insert(90, 9.0, f, &()), None);
    assert_eq!(m.insert(200, 20.0, f, &()), None);

    // m.verify(f, &());

    assert_eq!(
        m.iter(f).collect::<Vec<_>>(),
        [(20, 2.0), (40, 4.0), (50, 5.5), (60, 6.0), (80, 8.0), (90, 9.0), (200, 20.0),]
    );

    assert_eq!(m.get(0, f, &()), None);
    assert_eq!(m.get(20, f, &()), Some(2.0));
    assert_eq!(m.get(30, f, &()), None);
    assert_eq!(m.get(40, f, &()), Some(4.0));
    assert_eq!(m.get(50, f, &()), Some(5.5));
    assert_eq!(m.get(60, f, &()), Some(6.0));
    assert_eq!(m.get(70, f, &()), None);
    assert_eq!(m.get(80, f, &()), Some(8.0));
    assert_eq!(m.get(100, f, &()), None);

    assert_eq!(m.get_or_less(0, f, &()), None);
    assert_eq!(m.get_or_less(20, f, &()), Some((20, 2.0)));
    assert_eq!(m.get_or_less(30, f, &()), Some((20, 2.0)));
    assert_eq!(m.get_or_less(40, f, &()), Some((40, 4.0)));
    assert_eq!(m.get_or_less(200, f, &()), Some((200, 20.0)));
    assert_eq!(m.get_or_less(201, f, &()), Some((200, 20.0)));

    {
        let mut c = m.cursor(f, &());
        assert_eq!(c.prev(), Some((200, 20.0)));
        assert_eq!(c.prev(), Some((90, 9.0)));
        assert_eq!(c.prev(), Some((80, 8.0)));
        assert_eq!(c.prev(), Some((60, 6.0)));
        assert_eq!(c.prev(), Some((50, 5.5)));
        assert_eq!(c.prev(), Some((40, 4.0)));
        assert_eq!(c.prev(), Some((20, 2.0)));
        assert_eq!(c.prev(), None);
    }

    // Test some removals where the node stays healthy.
    assert_eq!(m.tpath(50, f, &()), "node0[2]");
    assert_eq!(m.tpath(80, f, &()), "node0[4]");
    assert_eq!(m.tpath(200, f, &()), "node0[6]");

    assert_eq!(m.remove(80, f, &()), Some(8.0));
    assert_eq!(m.tpath(50, f, &()), "node0[2]");
    assert_eq!(m.tpath(80, f, &()), "node0[4]");
    assert_eq!(m.tpath(200, f, &()), "node0[5]");
    assert_eq!(m.remove(80, f, &()), None);
    // m.verify(f, &());

    assert_eq!(m.remove(20, f, &()), Some(2.0));
    assert_eq!(m.tpath(50, f, &()), "node0[1]");
    assert_eq!(m.tpath(80, f, &()), "node0[3]");
    assert_eq!(m.tpath(200, f, &()), "node0[4]");
    assert_eq!(m.remove(20, f, &()), None);
    // m.verify(f, &());

    // [ 40 50 60 90 200 ]

    {
        let mut c = m.cursor(f, &());
        assert_eq!(c.goto_first(), Some(4.0));
        assert_eq!(c.key(), Some(40));
        assert_eq!(c.value(), Some(4.0));
        assert_eq!(c.next(), Some((50, 5.5)));
        assert_eq!(c.next(), Some((60, 6.0)));
        assert_eq!(c.next(), Some((90, 9.0)));
        assert_eq!(c.next(), Some((200, 20.0)));
        c.verify();
        assert_eq!(c.next(), None);
        c.verify();
    }

    // Removals from the root leaf node beyond underflow.
    assert_eq!(m.remove(200, f, &()), Some(20.0));
    assert_eq!(m.remove(40, f, &()), Some(4.0));
    assert_eq!(m.remove(60, f, &()), Some(6.0));
    // m.verify(f, &());
    assert_eq!(m.remove(50, f, &()), Some(5.5));
    // m.verify(f, &());
    assert_eq!(m.remove(90, f, &()), Some(9.0));
    // m.verify(f, &());
    assert!(m.is_empty());
}

#[test]
fn split_level0_leaf() {
    // Various ways of splitting a full leaf node at level 0.
    let f = &mut MapForest::<u32, f32>::new();

    fn full_leaf(f: &mut MapForest<u32, f32>) -> Map<u32, f32> {
        let mut m = Map::new();
        for n in 1..8 {
            m.insert(n * 10, n as f32 * 1.1, f, &());
        }
        m
    }

    // Insert at front of leaf.
    let mut m = full_leaf(f);
    m.insert(5, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(5, f, &()), Some(4.2));

    // Retain even entries, with altered values.
    m.retain(f, |k, v| {
        *v = (k / 10) as f32;
        (k % 20) == 0
    });
    assert_eq!(m.iter(f).collect::<Vec<_>>(), [(20, 2.0), (40, 4.0), (60, 6.0)]);

    // Insert at back of leaf.
    let mut m = full_leaf(f);
    m.insert(80, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(80, f, &()), Some(4.2));

    // Insert before middle (40).
    let mut m = full_leaf(f);
    m.insert(35, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(35, f, &()), Some(4.2));

    // Insert after middle (40).
    let mut m = full_leaf(f);
    m.insert(45, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(45, f, &()), Some(4.2));

    m.clear(f);
    assert!(m.is_empty());
}

#[test]
fn split_level1_leaf() {
    // Various ways of splitting a full leaf node at level 1.
    let f = &mut MapForest::<u32, f32>::new();

    // Return a map whose root node is a full inner node, and the leaf nodes are all full
    // containing:
    //
    // 110, 120, ..., 170
    // 210, 220, ..., 270
    // ...
    // 810, 820, ..., 870
    fn full(f: &mut MapForest<u32, f32>) -> Map<u32, f32> {
        let mut m = Map::new();

        // Start by inserting elements in order.
        // This should leave 8 leaf nodes with 4 elements in each.
        for row in 1..9 {
            for col in 1..5 {
                m.insert(row * 100 + col * 10, row as f32 + col as f32 * 0.1, f, &());
            }
        }

        // Then top up the leaf nodes without splitting them.
        for row in 1..9 {
            for col in 5..8 {
                m.insert(row * 100 + col * 10, row as f32 + col as f32 * 0.1, f, &());
            }
        }

        m
    }

    let mut m = full(f);
    // Verify geometry. Get get node2 as the root and leaves node0, 1, 3, ...
    // m.verify(f, &());
    assert_eq!(m.tpath(110, f, &()), "node2[0]--node0[0]");
    assert_eq!(m.tpath(140, f, &()), "node2[0]--node0[3]");
    assert_eq!(m.tpath(210, f, &()), "node2[1]--node1[0]");
    assert_eq!(m.tpath(270, f, &()), "node2[1]--node1[6]");
    assert_eq!(m.tpath(310, f, &()), "node2[2]--node3[0]");
    assert_eq!(m.tpath(810, f, &()), "node2[7]--node8[0]");
    assert_eq!(m.tpath(870, f, &()), "node2[7]--node8[6]");

    {
        let mut c = m.cursor(f, &());
        assert_eq!(c.goto_first(), Some(1.1));
        assert_eq!(c.key(), Some(110));
    }

    // Front of first leaf.
    m.insert(0, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(0, f, &()), Some(4.2));

    // First leaf split 4-4 after appending to LHS.
    f.clear();
    m = full(f);
    m.insert(135, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(135, f, &()), Some(4.2));

    // First leaf split 4-4 after prepending to RHS.
    f.clear();
    m = full(f);
    m.insert(145, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(145, f, &()), Some(4.2));

    // First leaf split 4-4 after appending to RHS.
    f.clear();
    m = full(f);
    m.insert(175, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(175, f, &()), Some(4.2));

    // Left-middle leaf split, ins LHS.
    f.clear();
    m = full(f);
    m.insert(435, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(435, f, &()), Some(4.2));

    // Left-middle leaf split, ins RHS.
    f.clear();
    m = full(f);
    m.insert(445, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(445, f, &()), Some(4.2));

    // Right-middle leaf split, ins LHS.
    f.clear();
    m = full(f);
    m.insert(535, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(535, f, &()), Some(4.2));

    // Right-middle leaf split, ins RHS.
    f.clear();
    m = full(f);
    m.insert(545, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(545, f, &()), Some(4.2));

    // Last leaf split, ins LHS.
    f.clear();
    m = full(f);
    m.insert(835, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(835, f, &()), Some(4.2));

    // Last leaf split, ins RHS.
    f.clear();
    m = full(f);
    m.insert(845, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(845, f, &()), Some(4.2));

    // Front of last leaf.
    f.clear();
    m = full(f);
    m.insert(805, 4.2, f, &());
    // m.verify(f, &());
    assert_eq!(m.get(805, f, &()), Some(4.2));

    m.clear(f);
    // m.verify(f, &());
}

// Make a tree with two barely healthy leaf nodes:
// [ 10 20 30 40 ] [ 50 60 70 80 ]
fn two_leaf(f: &mut MapForest<u32, f32>) -> Map<u32, f32> {
    f.clear();
    let mut m = Map::new();
    for n in 1..9 {
        m.insert(n * 10, n as f32, f, &());
    }
    m
}

#[test]
fn remove_level1() {
    let f = &mut MapForest::<u32, f32>::new();
    let mut m = two_leaf(f);

    // Verify geometry.
    // m.verify(f, &());
    assert_eq!(m.tpath(10, f, &()), "node2[0]--node0[0]");
    assert_eq!(m.tpath(40, f, &()), "node2[0]--node0[3]");
    assert_eq!(m.tpath(49, f, &()), "node2[0]--node0[4]");
    assert_eq!(m.tpath(50, f, &()), "node2[1]--node1[0]");
    assert_eq!(m.tpath(80, f, &()), "node2[1]--node1[3]");

    // Remove the front entry from a node that stays healthy.
    assert_eq!(m.insert(55, 5.5, f, &()), None);
    assert_eq!(m.remove(50, f, &()), Some(5.0));
    // m.verify(f, &());
    assert_eq!(m.tpath(49, f, &()), "node2[0]--node0[4]");
    assert_eq!(m.tpath(50, f, &()), "node2[0]--node0[4]");
    assert_eq!(m.tpath(55, f, &()), "node2[1]--node1[0]");

    // Remove the front entry from the first leaf node: No critical key to update.
    assert_eq!(m.insert(15, 1.5, f, &()), None);
    assert_eq!(m.remove(10, f, &()), Some(1.0));
    // m.verify(f, &());

    // [ 15 20 30 40 ] [ 55 60 70 80 ]

    // Remove the front entry from a right-most node that underflows.
    // No rebalancing for the right-most node. Still need critical key update.
    assert_eq!(m.remove(55, f, &()), Some(5.5));
    // m.verify(f, &());
    assert_eq!(m.tpath(55, f, &()), "node2[0]--node0[4]");
    assert_eq!(m.tpath(60, f, &()), "node2[1]--node1[0]");

    // [ 15 20 30 40 ] [ 60 70 80 ]

    // Replenish the right leaf.
    assert_eq!(m.insert(90, 9.0, f, &()), None);
    assert_eq!(m.insert(100, 10.0, f, &()), None);
    // m.verify(f, &());
    assert_eq!(m.tpath(55, f, &()), "node2[0]--node0[4]");
    assert_eq!(m.tpath(60, f, &()), "node2[1]--node1[0]");

    // [ 15 20 30 40 ] [ 60 70 80 90 100 ]

    // Removing one entry from the left leaf should trigger a rebalancing from the right
    // sibling.
    assert_eq!(m.remove(20, f, &()), Some(2.0));
    // m.verify(f, &());

    // [ 15 30 40 60 ] [ 70 80 90 100 ]
    // Check that the critical key was updated correctly.
    assert_eq!(m.tpath(50, f, &()), "node2[0]--node0[3]");
    assert_eq!(m.tpath(60, f, &()), "node2[0]--node0[3]");
    assert_eq!(m.tpath(70, f, &()), "node2[1]--node1[0]");

    // Remove front entry from the left-most leaf node, underflowing.
    // This should cause two leaf nodes to be merged and the root node to go away.
    assert_eq!(m.remove(15, f, &()), Some(1.5));
    // m.verify(f, &());
}

#[test]
fn remove_level1_rightmost() {
    let f = &mut MapForest::<u32, f32>::new();
    let mut m = two_leaf(f);

    // [ 10 20 30 40 ] [ 50 60 70 80 ]

    // Remove entries from the right leaf. This doesn't trigger a rebalancing.
    assert_eq!(m.remove(60, f, &()), Some(6.0));
    assert_eq!(m.remove(80, f, &()), Some(8.0));
    assert_eq!(m.remove(50, f, &()), Some(5.0));
    // m.verify(f, &());

    // [ 10 20 30 40 ] [ 70 ]
    assert_eq!(m.tpath(50, f, &()), "node2[0]--node0[4]");
    assert_eq!(m.tpath(70, f, &()), "node2[1]--node1[0]");

    // Removing the last entry from the right leaf should cause a collapse.
    assert_eq!(m.remove(70, f, &()), Some(7.0));
    // m.verify(f, &());
}

// Make a 3-level tree with barely healthy nodes.
// 1 root, 8 inner nodes, 7*4+5=33 leaf nodes, 4 entries each.
fn level3_sparse(f: &mut MapForest<u32, f32>) -> Map<u32, f32> {
    f.clear();
    let mut m = Map::new();
    for n in 1..133 {
        m.insert(n * 10, n as f32, f, &());
    }
    m
}

#[test]
fn level3_removes() {
    let f = &mut MapForest::<u32, f32>::new();
    let mut m = level3_sparse(f);
    // m.verify(f, &());

    // Check geometry.
    // Root: node11
    // [ node2 170 node10 330 node16 490 node21 650 node26 810 node31 970 node36 1130 node41 ]
    // L1: node11
    assert_eq!(m.tpath(0, f, &()), "node11[0]--node2[0]--node0[0]");
    assert_eq!(m.tpath(10000, f, &()), "node11[7]--node41[4]--node40[4]");

    // 650 is a critical key in the middle of the root.
    assert_eq!(m.tpath(640, f, &()), "node11[3]--node21[3]--node19[3]");
    assert_eq!(m.tpath(650, f, &()), "node11[4]--node26[0]--node20[0]");

    // Deleting 640 triggers a rebalance from node19 to node 20, cascading to n21 -> n26.
    assert_eq!(m.remove(640, f, &()), Some(64.0));
    // m.verify(f, &());
    assert_eq!(m.tpath(650, f, &()), "node11[3]--node26[3]--node20[3]");

    // 1130 is in the first leaf of the last L1 node. Deleting it triggers a rebalance node35
    // -> node37, but no rebalance above where there is no right sibling.
    assert_eq!(m.tpath(1130, f, &()), "node11[6]--node41[0]--node35[0]");
    assert_eq!(m.tpath(1140, f, &()), "node11[6]--node41[0]--node35[1]");
    assert_eq!(m.remove(1130, f, &()), Some(113.0));
    // m.verify(f, &());
    assert_eq!(m.tpath(1140, f, &()), "node11[6]--node41[0]--node37[0]");
}

#[test]
fn insert_many() {
    let f = &mut MapForest::<u32, f32>::new();
    let mut m = Map::<u32, f32>::new();

    let mm = 4096;
    let mut x = 0;

    for n in 0..mm {
        assert_eq!(m.insert(x, n as f32, f, &()), None);
        // m.verify(f, &());

        x = (x + n + 1) % mm;
    }

    x = 0;
    for n in 0..mm {
        assert_eq!(m.get(x, f, &()), Some(n as f32));
        x = (x + n + 1) % mm;
    }

    x = 0;
    for n in 0..mm {
        assert_eq!(m.remove(x, f, &()), Some(n as f32));
        // m.verify(f, &());

        x = (x + n + 1) % mm;
    }

    assert!(m.is_empty());
}

#[test]
fn insert_sorted() {
    let f = &mut MapForest::<u32, f32>::new();
    let mut m = Map::<u32, f32>::new();

    let mm = 4096;

    for n in 0..mm {
        assert_eq!(m.insert(2 * n, (2 * n) as f32, f, &()), None);
    }

    m.insert_sorted_iter((0..mm).map(|n| (2 * n + 1, (2 * n + 1) as f32)), f, &(), |a, b| {
        assert_eq!(a, None);
        b
    });

    assert!(m.iter(f).eq((0..2 * mm).map(|n| (n, n as f32))));

    let mut m = Map::<u32, f32>::new();

    let mm = 4096;

    let iter = (0..mm).map(|n| (2 * n + 1, (2 * n + 1) as f32));

    m.insert_sorted_iter(iter.clone(), f, &(), |a, b| {
        assert_eq!(a, None);
        b
    });

    assert!(m.iter(f).eq(iter.clone()));

    m.insert_sorted_iter(std::iter::empty(), f, &(), |_, _: Option<f32>| unreachable!());

    assert!(m.iter(f).eq(iter));

    let iter = (0..mm).map(|n| (2 * n + 1, (4 * n + 1) as f32));

    m.insert_sorted_iter(iter, f, &(), |old, new| new + old.unwrap());

    let iter = (0..mm).map(|n| (2 * n + 1, (6 * n + 2) as f32));
    assert!(m.iter(f).eq(iter));
}
