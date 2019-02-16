//! ## (Merkle)Cross List
//!
//! #### 算法说明
//! - 读写效率与AVL、红黑树等相当；
//! - 易于理解和实现；
//! - 结构固定，可提供merkle proof。
//!
//! #### 应用场景
//! - 存在性证明，数据检索。
//!
//! #### 实现属性
//! - <font color=Red>×</font> 多线程安全
//! - <font color=Red>×</font> 无 unsafe 代码
//!
//! #### Example
//!```
//!    use bc_algo::mcl::*;
//!    use rand::random;
//!
//!    fn main() {
//!        let mut sample = vec![];
//!        (0..1117).for_each(|_| sample.push(random::<u128>()));
//!        sample.sort();
//!        sample.dedup();
//!
//!        let mut mcl = MCL::default();
//!        let mut hashsigs = vec![];
//!
//!        for v in sample.iter().cloned() {
//!            hashsigs.push(mcl.put(v).unwrap());
//!        }
//!
//!        assert_eq!(sample.len(), mcl.item_cnt());
//!        assert_eq!(hashsigs.len(), mcl.item_cnt());
//!        assert_eq!(mcl.item_cnt_realtime(), mcl.item_cnt());
//!
//!        assert!(mcl.merklesig().is_some());
//!        for (v, h) in sample.iter().zip(hashsigs.iter()) {
//!            assert_eq!(v, &mcl.get(h).unwrap());
//!            assert!(mcl.proof(h).unwrap());
//!        }
//!    }
//!```

pub mod error;
#[cfg(test)]
mod test;
pub mod traits;

use error::XErr;
use traits::AsBytes;

use std::rc::{Rc, Weak};

type HashSig = Rc<Box<[u8]>>;
type HashFunc = Box<dyn Fn(&[&[u8]]) -> HashSig>;

pub struct MCL<V: AsBytes> {
    /*以下三项为静态属性*/
    unit_maxsiz: usize,   //成员数量超过此值将进行单元分裂
    merklesig_len: usize, //哈希结果的字节长度
    hash: HashFunc,       //所使用的哈希函数指针

    /*以下两项动态属性*/
    merklesig: Option<HashSig>, //链表全局根哈希值(注：不是根结节的merkle hashsig)
    item_cnt: usize,            //最底一层的节点数，即全局元素数量

    root: Option<Rc<Node<V>>>, //根结节，顶层的首节点(注：此结节的merkle hashsig不是全局根哈希！)
}

pub struct Node<V: AsBytes> {
    key: HashSig,               //用于索引的键，由HashFunc(V)得到
    value: Rc<V>,               //被索引的目标
    merklesig: Option<HashSig>, //各节点的merkle路径哈希值

    /*
     * 十字链表：
     * - 下、右方向使用Rc连接
     * - 上、左方向使用Weak连接
     * 以保证资源构造、析构的正确性
     */
    upper: Weak<Node<V>>, //上侧节点(多对一)，每个子节点均存储其父节点指针
    lower: Option<Rc<Node<V>>>, //下侧节点(一对一)，父节点只存储长子节点(下层的第一个节点)
    left: Weak<Node<V>>,        //左侧节点(一对一)
    right: Option<Rc<Node<V>>>, //右侧节点(一对一)
}

macro_rules! rc_update {
    ($n: expr, $raw: expr, $v: expr) => {
        $raw = Rc::into_raw($n) as *mut Node<V>;
        unsafe {
            (*$raw).lower = $v;
            $n = Rc::from_raw($raw);
        }
    };
    (^$n: expr, $raw: expr, $v: expr) => {
        $raw = Rc::into_raw($n) as *mut Node<V>;
        unsafe {
            (*$raw).upper = $v;
            $n = Rc::from_raw($raw);
        }
    };
    (->$n: expr, $raw: expr, $v: expr) => {
        $raw = Rc::into_raw($n) as *mut Node<V>;
        unsafe {
            (*$raw).right = $v;
            $n = Rc::from_raw($raw);
        }
    };
    (<-$n: expr, $raw: expr, $v: expr) => {
        $raw = Rc::into_raw($n) as *mut Node<V>;
        unsafe {
            (*$raw).left = $v;
            $n = Rc::from_raw($raw);
        }
    };
    (@$n: expr, $raw: expr, $v: expr) => {
        $raw = Rc::into_raw($n) as *mut Node<V>;
        unsafe {
            (*$raw).lower = $v;
            Rc::from_raw($raw);
        }
    };
    (@^$n: expr, $raw: expr, $v: expr) => {
        $raw = Rc::into_raw($n) as *mut Node<V>;
        unsafe {
            (*$raw).upper = $v;
            Rc::from_raw($raw);
        }
    };
    (@->$n: expr, $raw: expr, $v: expr) => {
        $raw = Rc::into_raw($n) as *mut Node<V>;
        unsafe {
            (*$raw).right = $v;
            Rc::from_raw($raw);
        }
    };
    (@<-$n: expr, $raw: expr, $v: expr) => {
        $raw = Rc::into_raw($n) as *mut Node<V>;
        unsafe {
            (*$raw).left = $v;
            Rc::from_raw($raw);
        }
    };
}

impl<V: AsBytes> MCL<V> {
    ///#### 初始化
    ///- @unit_maxsiz[in]: 必须是不小于2的整数，元素数量超过此值将进行单元分裂
    ///- @hash[in]: 用于计算哈希值的函数指针
    #[inline(always)]
    pub fn init(unit_maxsiz: usize, hash: HashFunc) -> MCL<V> {
        assert!(unit_maxsiz >= 2);
        MCL {
            root: None,
            unit_maxsiz,
            merklesig_len: hash(&[&0i32.to_be_bytes()[..]]).len(),
            hash,
            merklesig: None,
            item_cnt: 0,
        }
    }

    ///#### 以默认配置初始化
    #[inline(always)]
    pub fn default() -> MCL<V> {
        MCL::init(8, Box::new(sha256))
    }

    ///#### 销毁
    pub fn destroy(self) {}

    ///#### 获取单元容量
    #[inline(always)]
    pub fn unit_maxsiz(&self) -> usize {
        self.unit_maxsiz
    }

    ///- #: 全局根哈希
    #[inline(always)]
    pub fn merklesig(&self) -> Option<HashSig> {
        self.merklesig.clone()
    }

    ///#### 获取链表中所有元素的数量
    #[inline(always)]
    pub fn item_cnt(&self) -> usize {
        self.item_cnt
    }

    ///#### 获取链表中所有元素的数量
    #[inline(always)]
    pub fn item_cnt_realtime(&self) -> usize {
        if let Some(mut n) = self.get_lowest_first_node() {
            let mut i = 1;
            while let Some(r) = n.right.as_ref() {
                i += 1;
                n = Rc::clone(r);
            }
            i
        } else {
            0
        }
    }

    ///#### 获取链表的树高度
    #[inline(always)]
    pub fn height(&self) -> usize {
        let mut height = 0usize;
        if let Some(root) = self.root.as_ref() {
            height += 1;
            let mut root = Rc::clone(root);
            while let Some(l) = root.lower.as_ref() {
                height += 1;
                root = Rc::clone(l);
            }
        }

        height
    }

    ///#### 查询数据
    #[inline(always)]
    pub fn get(&self, key: &[u8]) -> Option<V> {
        self.get_inner(key).map(|n| (*n.value).clone()).ok()
    }

    ///- @key[in]: 将要被移除的目标节点的键
    #[inline(always)]
    pub fn remove(&mut self, key: &[u8]) -> Result<Rc<Node<V>>, XErr<V>> {
        Ok(self.remove_inner(self.get_inner(key)?))
    }

    ///#### 插入数据，并按需调整整体的数据结构
    ///- #: 成功返回新节点的merklesig
    ///    - 目标已存在，且键值均相同，视为成功，否则返回哈希碰撞错误
    ///    - 目标不存在，若存在左兄弟，则在其右侧插入新节点，否则插入为全局第一个元素
    ///- @value[in]: 待存的目标值
    pub fn put(&mut self, value: V) -> Result<HashSig, XErr<V>> {
        let sig = (self.hash)(&[&value.as_bytes()[..]]);
        match self.get_inner(&sig[..]) {
            Ok(n) => {
                if *n.value == value {
                    Ok(sig)
                } else {
                    Err(XErr::HashCollision(n))
                }
            }
            Err(XErr::NotExists(n)) => {
                let mut new;
                let mut raw;
                if let Some(n) = n {
                    let mut node = Node {
                        key: Rc::clone(&sig),
                        value: Rc::new(value),
                        merklesig: Some(Rc::clone(&sig)),
                        upper: Weak::clone(&n.upper),
                        lower: None,
                        left: Rc::downgrade(&n),
                        right: None,
                    };
                    if let Some(right) = n.right.as_ref() {
                        node.right = Some(Rc::clone(right));
                        new = Rc::new(node);
                        rc_update!(@->Rc::clone(&n), raw, Some(Rc::clone(&new)));
                        rc_update!(@<-Rc::clone(right), raw, Rc::downgrade(&new));
                    } else {
                        new = Rc::new(node);
                        rc_update!(@->n, raw, Some(Rc::clone(&new)));
                    }

                    self.restruct_put(new); //重塑链表结构
                } else if self.root.is_some() {
                    let mut lowest = self.get_lowest_first_node().unwrap();
                    new = Rc::new(Node {
                        key: Rc::clone(&sig),
                        value: Rc::new(value),
                        merklesig: Some(Rc::clone(&sig)),
                        upper: Weak::new(),
                        lower: None,
                        left: Weak::new(),
                        right: Some(Rc::clone(&lowest)),
                    });

                    rc_update!(<-lowest, raw, Rc::downgrade(&new));

                    let mut uppest = lowest;
                    let mut uppest_new;
                    let mut root = Rc::clone(&new);
                    while let Some(u) = Weak::upgrade(&uppest.upper) {
                        uppest_new = Rc::new(Node {
                            key: Rc::clone(&root.key),
                            value: Rc::clone(&root.value),
                            merklesig: Some(self.merklesig_upper(&self.my_unit(Rc::clone(&root)))),
                            upper: Weak::new(),
                            lower: Some(Rc::clone(&root)),
                            left: Weak::new(),
                            right: u.right.clone(),
                        });

                        rc_update!(^root, raw, Rc::downgrade(&uppest_new));

                        //首先，断开旧root垂直线的父连接线
                        //然后，基于长兄节点，右向刷新兄弟节点的父节点
                        rc_update!(@^uppest, raw, Weak::new());
                        self.parent_refresh(Rc::clone(&root));

                        root = uppest_new;
                        uppest = u;
                    }

                    self.root = Some(root);
                    self.restruct_put(new); //重塑链表结构
                } else {
                    new = Rc::new(Node {
                        key: Rc::clone(&sig),
                        value: Rc::new(value),
                        merklesig: Some(Rc::clone(&sig)),
                        upper: Weak::new(),
                        lower: None,
                        left: Weak::new(),
                        right: None,
                    });

                    self.merklesig = Some((self.hash)(&[&sig[..]]));
                    self.root = Some(new);
                }

                self.item_cnt += 1;
                Ok(sig)
            }
            Err(e) => Err(e),
        }
    }

    ///#### 获取merkle proof
    ///- #: 若根哈希值与计算出的根哈希相等，返回true
    ///- @key[in]: 查找对象
    pub fn proof(&self, key: &[u8]) -> Result<bool, XErr<V>> {
        if self.merklesig.is_none() {
            return Ok(false);
        }

        let path = self.get_proof_path(key)?;
        for (i, _) in path.iter().enumerate().rev().skip(1).rev() {
            if (self.hash)(
                &path[i]
                    .merklesigs
                    .iter()
                    .map(|h| &h[..])
                    .collect::<Vec<&[u8]>>()
                    .as_slice(),
            ) != path[i + 1].merklesigs[path[i + 1].selfidx]
            {
                return Ok(false);
            }
        }

        let res = path.last().map(|p| {
            (self.hash)(
                p.merklesigs
                    .iter()
                    .map(|h| &h[..])
                    .collect::<Vec<&[u8]>>()
                    .as_slice(),
            )
        });

        Ok(self.merklesig == res)
    }

    ///#### 基于最底一层对其上各层进行彻底重塑，
    ///- 对于长期运行的应用，可在业务闲时调用此函数，优化整体性能
    #[inline(always)]
    pub fn restruct_all(&mut self) {
        if let Some(lowest) = self.get_lowest_first_node() {
            let mut unit_siz = self.unit_maxsiz / 2;
            if unit_siz < 2 {
                unit_siz = 2;
            }

            let mut sigbuf = Vec::with_capacity(unit_siz);
            self.restruct_all_inner(lowest, unit_siz, &mut sigbuf);
        }
    }
}

//- @my_unit: 自身所在单元的节点集合
//- @right_unit: 右邻单元的节点集合
struct Adjacency<V: AsBytes> {
    left_unit: Vec<Rc<Node<V>>>,
    my_unit: Vec<Rc<Node<V>>>,
    right_unit: Vec<Rc<Node<V>>>,
}

impl<V: AsBytes> MCL<V> {
    //#### 检查输入的merklesig字节长度是否合法
    #[inline(always)]
    fn check_merklesig_len(&self, hashsig: &[u8]) -> bool {
        hashsig.len() == self.merklesig_len
    }

    #[inline(always)]
    fn is_first_node(&self, node: Rc<Node<V>>) -> bool {
        self.root
            .as_ref()
            .map(|root| {
                Weak::upgrade(&node.upper)
                    .map(|u| Rc::ptr_eq(u.lower.as_ref().unwrap(), &node))
                    .unwrap_or_else(|| Rc::ptr_eq(&node, root))
            })
            .unwrap_or(false)
    }

    //#### 获取最底一层的首节点
    #[inline(always)]
    fn get_lowest_first_node(&self) -> Option<Rc<Node<V>>> {
        let mut lowest = Rc::clone(self.root.as_ref()?);
        while let Some(l) = lowest.lower.as_ref() {
            lowest = Rc::clone(l);
        }
        Some(lowest)
    }

    //#### 查询数据
    //- #: 成功返回目标节点指针，
    //失败返回错误原因(其中不存在的情况，返回可插入位置的Option<`左`兄弟指针>)
    #[inline(always)]
    fn get_inner(&self, key: &[u8]) -> Result<Rc<Node<V>>, XErr<V>> {
        if !self.check_merklesig_len(key) {
            return Err(XErr::HashLen);
        }
        if self.root.is_none() {
            return Err(XErr::NotExists(None));
        }

        let root = Rc::clone(self.root.as_ref().unwrap());
        let rootkey = &root.key[..];

        //处理root节点的特殊情况
        if key == rootkey {
            Ok(root)
        } else if key < rootkey {
            Err(XErr::NotExists(None))
        } else {
            let mut res = None;
            Self::get_inner_r(key, root, &mut res);
            //无论无何，都不可能为None：
            //    - 查询成功，返回目标节点
            //    - 查询失败，返回其可插入点的左兄弟
            let node = res.unwrap();
            if key == &node.key[..] {
                Ok(node)
            } else {
                Err(XErr::NotExists(Some(node)))
            }
        }
    }

    //- @key[in]: 待查找的目标key
    //- @node[in]: 已经过上一层判断过节点，key > node.key一定成立
    //- @res[out]: 最终结果写出之地
    fn get_inner_r(key: &[u8], node: Rc<Node<V>>, res: &mut Option<Rc<Node<V>>>) {
        let mut cur = node;
        let mut right;
        let mut curkey;

        //单调右向搜索
        while let Some(r) = cur.right.as_ref() {
            right = Rc::clone(r);
            curkey = &right.key[..];
            if key == curkey {
                *res = Some(right);
                return;
            } else if key < curkey {
                break; //之后将进入下一轮递归
            } else {
                cur = right;
            }
        }

        //放在循环外部，以兼容全局只有root一个节点时，
        //同时新key比root.key大的情况
        if cur.lower.is_none() {
            //查找失败返回其左兄弟
            *res = Some(cur);
            return;
        } //之后将进入下一轮递归

        //should be a tail-recursion
        Self::get_inner_r(key, Rc::clone(cur.lower.as_ref().unwrap()), res);
    }

    //#### 删除数据，并按需调整整体的数据结构，若被删节点：
    //1. 若左右兄弟皆为空，则说明删除的是节点总数为一的链表的唯一节点，直接将根节点置空即可
    //2. 若左兄弟为空，右兄弟不为空，说明删除的是首节点，只需调整右兄弟指针
    //3. 若左兄弟不为空，右兄弟为空，说明删除的是末尾节点，只需调整左兄弟指针
    //4. 若左右兄弟皆不为空，需同时调节左右兄弟指针
    //5. 2、3、4三种情况，均需检查其是否是其父节点的长子，
    //若是，则递归向上删除其所有父辈节点，并新建替代垂直线
    //- @node[in]: 将要被移除的目标节点
    fn remove_inner(&mut self, node: Rc<Node<V>>) -> Rc<Node<V>> {
        let left = if let Some(r) = Weak::upgrade(&node.left) {
            Some(r)
        } else {
            None
        };
        let right = if let Some(l) = node.right.as_ref() {
            Some(Rc::clone(l))
        } else {
            None
        };

        let mut raw;
        if right.is_none() && left.is_none() {
            self.root = None;
        } else if left.is_some() && right.is_none() {
            let l = left.unwrap();
            rc_update!(@->l, raw, None);
        } else if left.is_none() && right.is_some() {
            let r = right.unwrap();
            rc_update!(@<-r, raw, Weak::new());
        } else {
            let mut l = left.unwrap();
            rc_update!(->l, raw, right.clone());
            rc_update!(@<-right.unwrap(), raw, Rc::downgrade(&l));
        }

        //被删除的节点，此时仍然保持其原先的上下左右连接关系
        //直接基于被删节点进行重塑即可
        self.restruct_remove(Rc::clone(&node));

        self.item_cnt -= 1;
        node
    }

    //#### 根据子节点集合，生成父节点的merklesig
    //- @lowers[in]: children of the goal
    fn merklesig_upper(&self, lowers: &[Rc<Node<V>>]) -> HashSig {
        (self.hash)(
            &lowers
                .iter()
                .map(|n| &n.merklesig.as_ref().unwrap()[..])
                .collect::<Vec<&[u8]>>(),
        )
    }

    //#### 由下而上递归刷新merkle proof hashsig
    //- 根节点需要特殊处理
    fn merkle_refresh(&mut self, node: Rc<Node<V>>) {
        let upper;
        let unit = self.my_unit(Rc::clone(&node));
        if let Some(u) = Weak::upgrade(&node.upper) {
            let raw = Rc::into_raw(u) as *mut Node<V>;
            unsafe {
                (*raw).merklesig = Some(self.merklesig_upper(&unit));
                upper = Rc::from_raw(raw);
            } //之后将进入下一轮递归
        } else {
            self.merklesig = Some(self.merklesig_upper(&unit));
            return;
        }

        //should be a tail-recursion
        self.merkle_refresh(upper);
    }

    //#### 根据给定的节点，统计其自身所在单元及左右相邻单元的节点指针集合
    fn adjacent_statistics(&self, node: Rc<Node<V>>) -> Adjacency<V> {
        Adjacency {
            left_unit: self.left_unit(Rc::clone(&node)),
            my_unit: self.my_unit(Rc::clone(&node)),
            right_unit: self.right_unit(node),
        }
    }

    //#### self helper for adjacent_statistics
    fn my_unit(&self, node: Rc<Node<V>>) -> Vec<Rc<Node<V>>> {
        let mut res = vec![];
        if let Some(u) = Weak::upgrade(&node.upper) {
            let mut cur = Rc::clone(u.lower.as_ref().unwrap());
            res.push(Rc::clone(&cur));
            while let Some(r) = cur.right.as_ref() {
                //相同父节点下的所有子节点集合
                if !Rc::ptr_eq(&u, &Weak::upgrade(&r.upper).unwrap()) {
                    break;
                }
                res.push(Rc::clone(r));
                cur = Rc::clone(r);
            }
        } else {
            //此时根节点不可能为空
            let mut cur = Rc::clone(self.root.as_ref().unwrap());
            res.push(Rc::clone(&cur));

            while let Some(r) = cur.right.as_ref() {
                res.push(Rc::clone(r));
                cur = Rc::clone(r);
            }
        }
        res
    }

    //#### left helper for adjacent_statistics
    fn left_unit(&self, node: Rc<Node<V>>) -> Vec<Rc<Node<V>>> {
        let mut res = vec![];
        if let Some(u) = Weak::upgrade(&node.upper) {
            if let Some(lu) = Weak::upgrade(&u.left) {
                let mut cur = Rc::clone(lu.lower.as_ref().unwrap());
                res.push(Rc::clone(&cur));
                while let Some(r) = cur.right.as_ref() {
                    //相同父节点下的所有子节点集合
                    if !Rc::ptr_eq(&lu, &Weak::upgrade(&r.upper).unwrap()) {
                        break;
                    }
                    res.push(Rc::clone(r));
                    cur = Rc::clone(r);
                }
            }
        }

        /* 无父节点时，说明处于顶层，而顶层永远只有一个单元，故其左右单元均为空 */
        res
    }

    //#### right helper for adjacent_statistics
    fn right_unit(&self, node: Rc<Node<V>>) -> Vec<Rc<Node<V>>> {
        let mut res = vec![];
        if let Some(u) = Weak::upgrade(&node.upper) {
            if let Some(ru) = u.right.as_ref() {
                let mut cur = Rc::clone(ru.lower.as_ref().unwrap());
                res.push(Rc::clone(&cur));
                while let Some(r) = cur.right.as_ref() {
                    //相同父节点下的所有子节点集合
                    if !Rc::ptr_eq(&ru, &Weak::upgrade(&r.upper).unwrap()) {
                        break;
                    }
                    res.push(Rc::clone(r));
                    cur = Rc::clone(r);
                }
            }
        }

        /* 无父节点时，说明处于顶层，而顶层永远只有一个单元，故其左右单元均为空*/
        res
    }

    //#### 基于长兄节点，右向刷新各兄弟节点的父节点
    fn parent_refresh(&self, eldest_brother: Rc<Node<V>>) {
        let mut node = Rc::clone(&eldest_brother);
        let mut raw;
        while let Some(r) = node.right.as_ref() {
            if self.is_first_node(Rc::clone(r)) {
                break;
            }
            node = Rc::clone(r);
            rc_update!(^node, raw, Weak::clone(&eldest_brother.upper));
        }
    }

    //#### 新增节点后，
    //- 递归向上调整链表结构，递归至最顶层时，检查是否有需要分裂的满员单元
    //- 刷新merkle proof hashsig
    fn restruct_put(&mut self, node: Rc<Node<V>>) {
        let unit = self.my_unit(Rc::clone(&node));
        let new;

        //满员或超员则执行单元分裂
        if self.unit_maxsiz <= unit.len() {
            let mut raw;
            //链表初始化时，已保证self.unit_maxsiz >= 2
            let mid = self.unit_maxsiz / 2;
            let mut a = Rc::clone(&unit[0]);
            let mut b = Rc::clone(&unit[mid]);
            if let Some(u) = Weak::upgrade(&node.upper) {
                new = Rc::new(Node {
                    key: Rc::clone(&b.key),
                    value: Rc::clone(&b.value),
                    merklesig: Some(self.merklesig_upper(&unit[mid..])),
                    upper: Weak::clone(&u.upper),
                    lower: Some(Rc::clone(&b)),
                    left: Rc::downgrade(&u),
                    right: u.right.clone(),
                });

                raw = Rc::into_raw(u) as *mut Node<V>;
                unsafe {
                    (*raw).right = Some(Rc::clone(&new));
                    (*raw).merklesig = Some(self.merklesig_upper(&unit[..mid]));
                    Rc::from_raw(raw);
                }

                rc_update!(^b, raw, Rc::downgrade(&new));

                self.parent_refresh(b); //之后将进入下一轮递归
            } else {
                let mut root = Rc::new(Node {
                    key: Rc::clone(&a.key),
                    value: Rc::clone(&a.value),
                    merklesig: Some(self.merklesig_upper(&unit[..mid])),
                    upper: Weak::new(),
                    lower: Some(Rc::clone(&a)),
                    left: Weak::new(),
                    right: None,
                });

                new = Rc::new(Node {
                    key: Rc::clone(&b.key),
                    value: Rc::clone(&b.value),
                    merklesig: Some(self.merklesig_upper(&unit[mid..])),
                    upper: Weak::new(),
                    lower: Some(Rc::clone(&b)),
                    left: Rc::downgrade(&root),
                    right: None,
                });

                rc_update!(->root, raw, Some(Rc::clone(&new)));

                rc_update!(^a, raw, Rc::downgrade(&root));
                self.parent_refresh(a);

                rc_update!(^b, raw, Rc::downgrade(&new));
                self.parent_refresh(b);

                //刷新全局根哈希
                self.merklesig = Some(self.merklesig_upper(&[Rc::clone(&root), new]));

                self.root = Some(root);
                return;
            }
        } else {
            //由下到上批量刷受影响的节点的merklesig
            self.merkle_refresh(node);
            return;
        }

        //should be a tail-recursion
        self.restruct_put(new);
    }

    //#### 删除节点后，
    //- 递归向上调整链表结构，递归至最顶层时，检查是否可以降低树高度
    //- 刷新merkle proof hashsig
    fn restruct_remove(&mut self, node: Rc<Node<V>>) {
        let mut raw;
        if let Some(u) = Weak::upgrade(&node.upper) {
            //若被删除的节点的父节点也是其所在单元的首节点，
            //则首先递归向上处理之
            if self.is_first_node(Rc::clone(&node)) {
                //若被删除的节点的右兄弟存在，
                //且与其属于同一个单元(即：非其所在单元的首节点)，
                //则需为其创建新的父节点
                if let Some(r) = node.right.as_ref() {
                    if !self.is_first_node(Rc::clone(r)) {
                        let ru = Rc::new(Node {
                            key: Rc::clone(&r.key),
                            value: Rc::clone(&r.value),
                            merklesig: None,
                            upper: Weak::clone(&u.upper),
                            lower: Some(Rc::clone(r)),
                            left: Weak::clone(&u.left),
                            right: u.right.clone(),
                        });

                        rc_update!(@^Rc::clone(r), raw, Rc::downgrade(&ru));

                        if let Some(l) = Weak::upgrade(&u.left) {
                            rc_update!(@->l, raw, Some(Rc::clone(&ru)));
                        }

                        if let Some(r) = u.right.as_ref() {
                            rc_update!(@<-Rc::clone(r), raw, Rc::downgrade(&ru));
                        }

                        raw = Rc::into_raw(ru) as *mut Node<V>;
                        unsafe {
                            (*raw).merklesig =
                                Some(self.merklesig_upper(&self.my_unit(Rc::clone(r))));
                            Rc::from_raw(raw);
                        }
                    }
                }

                self.remove_inner(Rc::clone(&u));
                self.restruct_remove(u);
            } else {
                //除顶层外，其余各层的任何单元，其节点数都>=2
                //因为每当降至1时，就会触发单元合并
                //故直接刷新其父节点merklesig即可
                raw = Rc::into_raw(u) as *mut Node<V>;
                unsafe {
                    (*raw).merklesig = Some(self.merklesig_upper(&self.my_unit(Rc::clone(&node))));
                    Rc::from_raw(raw);
                }
            }

            //**如下进行单元合并**
            //若所在单元只剩一个节点，则与其左右单元中节点数最少的一方进行合并
            //若合并后新的单元已超限，则对其进行中值分裂
            let adj = self.adjacent_statistics(Rc::clone(&node));
            if 1 == adj.my_unit.len() {
                let mut x;
                let n = if adj.right_unit.len() < adj.left_unit.len() {
                    if adj.right_unit.is_empty() {
                        x = Rc::clone(&adj.my_unit[0]);
                        rc_update!(^x, raw, Weak::new());
                        //基于左单元最后一个节点即可，不必从左单元首节点开始
                        self.parent_refresh(Rc::clone(&adj.left_unit[adj.left_unit.len() - 1]));
                    } else {
                        x = Rc::clone(&adj.right_unit[0]);
                        rc_update!(^x, raw, Weak::new());
                        self.parent_refresh(Rc::clone(&node));
                    }
                    x
                } else if adj.right_unit.len() > adj.left_unit.len() {
                    if adj.left_unit.is_empty() {
                        x = Rc::clone(&adj.right_unit[0]);
                        rc_update!(^x, raw, Weak::new());
                        self.parent_refresh(Rc::clone(&node));
                    } else {
                        x = Rc::clone(&adj.my_unit[0]);
                        rc_update!(^x, raw, Weak::new());
                        //基于左单元最后一个节点即可，不必从左单元首节点开始
                        self.parent_refresh(Rc::clone(&adj.left_unit[adj.left_unit.len() - 1]));
                    }
                    x
                } else {
                    //非顶层单元，左右邻同时为空是不可能的
                    unreachable!();
                };

                //本单元及其所有非空邻接单元，必然存在父节点
                let upper = Weak::upgrade(&n.upper).unwrap();

                //删除临界节点的父节点并递归重塑之
                self.remove_inner(Rc::clone(&upper));
                self.restruct_remove(upper);
            } else {
                return;
            }
        } else {
            //已到达顶层
            let mut root = Rc::clone(&self.root.as_ref().unwrap());

            if Rc::ptr_eq(&node, &root) {
                self.root = root.right.clone();
            }

            if let Some(r) = self.root.as_ref() {
                if r.right.is_some() {
                    //顶层除根结点外，还存在其它结点，则不需要降低树高度
                    return;
                }
                root = Rc::clone(r);
            } else {
                //若此时根结点为空，说明整个跳表已被清空
                self.merklesig = None;
                return;
            }

            while let Some(lower) = root.lower.as_ref() {
                //沿根节点垂直向下的所有节点，其左单元一定为空，无须判断
                if 1 != self.my_unit(Rc::clone(lower)).len()
                    || !self.right_unit(Rc::clone(lower)).is_empty()
                {
                    root = Rc::clone(lower);
                    break;
                }
            }

            rc_update!(^root, raw, Weak::new());
            self.parent_refresh(Rc::clone(&root));

            self.merklesig = Some(self.merklesig_upper(&self.my_unit(Rc::clone(&root))));
            self.root = Some(root);
            return;
        }
    }

    //- @head[in]: 每一层的首节点
    //- @unit_siz[in]: 优化的目标单元大小
    //- @sigbug[in]: 缓存区复用
    fn restruct_all_inner(
        &mut self,
        head: Rc<Node<V>>,
        unit_siz: usize,
        sigbuf: &mut Vec<HashSig>,
    ) {
        let mut raw;
        let mut new;
        let mut node;
        let mut i = 0;
        let mut upper_len = 0;

        //处理上层的首节点
        node = head;
        sigbuf.push(node.merklesig.clone().unwrap());
        i += 1;
        let upper_head = Rc::new(Node {
            key: Rc::clone(&node.key),
            value: Rc::clone(&node.value),
            merklesig: None,
            upper: Weak::new(),
            lower: Some(Rc::clone(&node)),
            left: Weak::new(),
            right: None,
        });
        let mut upper_tail = Rc::clone(&upper_head);
        upper_len += 1;

        rc_update!(^node, raw, Rc::downgrade(&upper_tail));

        //从左到右处理后续节点(若有)
        while let Some(r) = node.right.as_ref() {
            node = Rc::clone(r);
            sigbuf.push(node.merklesig.clone().unwrap());
            i += 1;

            //添加上层节点
            if 1 == i {
                new = Rc::new(Node {
                    key: Rc::clone(&node.key),
                    value: Rc::clone(&node.value),
                    merklesig: None,
                    upper: Weak::new(),
                    lower: Some(Rc::clone(&node)),
                    left: Rc::downgrade(&upper_tail),
                    right: None,
                });
                upper_tail = Rc::clone(&new);
                upper_len += 1;
            }

            //更改本层节点的upper指针
            rc_update!(^node, raw, Rc::downgrade(&upper_tail));

            //为上层节点计算merkle hashsig
            if unit_siz == i {
                i = 0;
                raw = Rc::into_raw(upper_tail) as *mut Node<V>;
                unsafe {
                    (*raw).merklesig = Some((self.hash)(
                        &sigbuf.iter().map(|i| &i[..]).collect::<Vec<&[u8]>>()[..],
                    ));
                    upper_tail = Rc::from_raw(raw);
                }
                sigbuf.clear();
            }
        }

        //处理最后一个单元的merkle hashsig
        if 0 != i {
            raw = Rc::into_raw(upper_tail) as *mut Node<V>;
            unsafe {
                (*raw).merklesig = Some((self.hash)(
                    &sigbuf.iter().map(|i| &i[..]).collect::<Vec<&[u8]>>()[..],
                ));
                Rc::from_raw(raw);
            }
            sigbuf.clear();
        }

        if upper_len < self.unit_maxsiz {
            self.root = Some(upper_head);
            return;
        } //之后将进入下一轮递归

        //should be a tail-recursion
        self.restruct_all_inner(upper_head, unit_siz, sigbuf);
    }

    //#### 获取指定的key存在的merkle路径证明
    //- #: 按从叶到根的順序排列的哈希集合，使用proof函数验证
    //- @key[in]: 查找对象
    fn get_proof_path(&self, key: &[u8]) -> Result<Vec<ProofPath>, XErr<V>> {
        let n = self.get_inner(key)?;
        let mut path = vec![];
        self.get_proof_path_r(n, &mut path);
        Ok(path)
    }

    //- @cur[in]: 当前节点
    //- @path[out]: 从叶到根的順序写出结果
    fn get_proof_path_r(&self, cur: Rc<Node<V>>, path: &mut Vec<ProofPath>) {
        let unit = self.my_unit(Rc::clone(&cur));
        let sigs = unit
            .iter()
            .map(|i| i.merklesig.clone().unwrap())
            .collect::<Vec<HashSig>>();

        path.push(ProofPath {
            //merklesig并没有排序，不能采用对sigs使用二分查找的方式
            selfidx: unit.iter().position(|x| x.key == cur.key).unwrap(),
            merklesigs: sigs,
        });

        let upper = Weak::upgrade(&cur.upper);
        if upper.is_none() {
            return;
        } //之后将进入下一轮递归

        //should be a tail-recursion
        self.get_proof_path_r(upper.unwrap(), path);
    }
}

//- @selfidx: 路径上的每个节点在所有兄弟节点中的索引
//- @merklesigs: 当前节点及其所有兄弟节点的哈希值的有序集合
struct ProofPath {
    selfidx: usize,
    merklesigs: Vec<HashSig>,
}

#[inline(always)]
fn sha256(item: &[&[u8]]) -> HashSig {
    use ring::digest::{Context, SHA256};

    let mut context = Context::new(&SHA256);
    for x in item {
        context.update(x);
    }
    Rc::new(
        context
            .finish()
            .as_ref()
            .iter()
            .cloned()
            .collect::<Box<[u8]>>(),
    )
}
