use contest_algorithms::range_query::seg_tree::SegmentTree;

struct NumArray {
    tree: SegmentTree
}
impl NumArray {
    fn new(nums:Vec<i32>)->Self {
        if nums.len()>0{
            let mut tree= SegmentTree::new_segment_tree(nums);
            tree.build();
            return Self{
                tree
            };
        }
        panic!("No data")
    }

    fn sum_range(&self,left:i32,right:i32)->i32{
        return self.tree.query(left as usize,right as usize).unwrap();
    }
    fn update(&mut self,index:i32,val:i32){
        self.tree.set(index as usize,val);
    }
    
}

#[test]
fn test() {
    let obj = NumArray::new(vec![-2, 0, 3, -5, 2, -1]);
    assert_eq!(obj.sum_range(0, 2), 1);
    assert_eq!(obj.sum_range(2, 5), -1);
    assert_eq!(obj.sum_range(0, 5), -3);
}