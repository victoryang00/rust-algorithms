pub struct SegmentTree {
    data: Vec<i32>,
    tree: Vec<Option<i32>>,
}

// https://www.zhihu.com/people/Classicalcastle
impl SegmentTree {
    pub fn new_segment_tree(arr: Vec<i32>) -> SegmentTree {
        let data_len = arr.len();
        Self {
            data: arr,
            tree: vec![None; 4 * data_len],
        }
    }

    fn left_child(index: usize) -> usize {
        return 2 * index + 1;
    }

    fn right_child(index: usize) -> usize {
        return 2 * index + 2;
    }

    pub fn get(&self, index: usize) -> Option<i32> {
        if index >= self.data.len() {
            return None;
        }
        return Some(self.data[index]);
    }

    pub fn build(&mut self) {
        self.build_segment_tree(0, 0, self.data.len() - 1);
    }

    fn build_segment_tree(&mut self, tree_index: usize, left: usize, right: usize) {
        if left == right {
            self.tree[tree_index] = Some(self.data[left]);
            return;
        }
        let left_tree_index = Self::left_child(tree_index);
        let right_tree_index = Self::right_child(tree_index);
        let mid = (right - left) / 2 + left;
        self.build_segment_tree(left_tree_index, left, mid);
        self.build_segment_tree(right_tree_index, mid + 1, right);
        if let Some(l) = self.tree[left_tree_index] {
            if let Some(r) = self.tree[right_tree_index] {
                self.tree[tree_index] = Some(l + r)
            }
        }
    }
    pub fn query(&self, l: usize, r: usize) -> Result<i32, &'static str> {
        if l > self.data.len() || r > self.data.len() || l > r {
            return Err("Error");
        }
        Ok(self.recursion_query(0, 0, self.data.len() - 1, l, r))
    }
    fn recursion_query(
        &self,
        tree_index: usize,
        l: usize,
        r: usize,
        query_left: usize,
        query_right: usize,
    ) -> i32 {
        if l == query_left && r == query_right {
            if let Some(d) = self.tree[tree_index] {
                return d;
            }
            return 0;
        }
        let mid = l + (r - l) / 2;
        let l_t_ind = Self::left_child(tree_index);
        let r_t_ind = Self::right_child(tree_index);

        if query_left >= mid + 1 {
            return self.recursion_query(r_t_ind, mid + 1, r, query_left, query_right);
        } else if query_right <= mid {
            return self.recursion_query(l_t_ind, l, mid, query_left, query_right);
        }
        let l_res = self.recursion_query(l_t_ind, l, mid, query_left, mid);
        let r_res = self.recursion_query(r_t_ind, mid + 1, r, mid + 1, query_right);
        l_res + r_res
    }
    pub fn set(&mut self, index: usize, e: i32) -> Result<(), &'static str> {
        if index >= self.data.len() {
            return Err("Error");
        }
        self.data[index] = e;
        self.recursion_set(0, 0, self.data.len() - 1, index, e);
        Ok(())
    }

    fn recursion_set(&mut self, index_tree: usize, l: usize, r: usize, index: usize, e: i32) {
        if l == r {
            self.tree[index_tree] = Some(e);
            return;
        }
        let mid = l + (r - 1) / 2;
        let left_child = Self::left_child(index_tree);
        let right_child = Self::right_child(index_tree);
        if index >= mid + 1 {
            self.recursion_set(right_child, mid + 1, r, index, e);
        } else {
            self.recursion_set(left_child, l, mid, index, e);
        }
        if let Some(l_d) = self.tree[left_child] {
            if let Some(r_d) = self.tree[right_child] {
                self.tree[index_tree] = Some(l_d + r_d);
            }
        }
    }
}
