//! ## (Merkle)Skip List
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
//!```

pub mod error;
pub mod traits;

use error::XErr;
use traits::AsBytes;

use std::rc::{Rc, Weak};

type HashSig = Box<[u8]>;
type HashFunc = Box<dyn Fn(&[&[u8]]) -> HashSig>;

///- @glob_entry: 根节点
///- @unit_siz: 成员数量超过此值将进行单元分裂
///- @merklesig: 根哈希值
///- @item_cnt: 最底一层的节点数，即全局元素数量
///- @merklesig_len: 哈希结果的字节长度
///- @hash: 所使用的哈希函数指针
pub struct SkipList<V: AsBytes> {
    glob_entry: Option<Rc<Node<V>>>,
    unit_siz: usize,

    item_cnt: usize,
    merklesig: HashSig,

    merklesig_len: usize,
    hash: HashFunc,
}

///- @key: 用于索引的键，由HashFunc(V)得到
///- @value: 被索引的目标
///- @merklesig: 各节点的merkle路径哈希值
///- @lower: 下侧节点(一对一，只存储下层的第一个节点)
///- @upper: 上侧节点(多对一)
///- @left: 左侧节点(一对一)
///- @right: 右侧节点(一对一)
pub struct Node<V: AsBytes> {
    key: Rc<HashSig>,
    value: Rc<V>,
    merklesig: HashSig,

    lower: Option<Rc<Node<V>>>,
    upper: Weak<Node<V>>,
    left: Option<Rc<Node<V>>>,
    right: Weak<Node<V>>,
}

//- @self_unit: 自身所在单元的节点集合
//- @left_unit: 左邻单元的节点集合
//- @right_unit: 右邻单元的节点集合
struct Adjacency<V: AsBytes> {
    self_unit: Vec<Rc<Node<V>>>,
    left_unit: Vec<Rc<Node<V>>>,
    right_unit: Vec<Rc<Node<V>>>,
}

impl<V: AsBytes> SkipList<V> {
    pub fn destroy(self) {}

    ///#### 以默认配置初始化
    #[inline(always)]
    pub fn default() -> SkipList<V> {
        SkipList::init(8, Box::new(sha256))
    }

    ///#### 初始化
    ///- @unit_siz[in]: 元素数量超过此值将进行单元分裂
    #[inline(always)]
    pub fn init(unit_siz: usize, hash: HashFunc) -> SkipList<V> {
        assert!(unit_siz >= 2);
        SkipList {
            glob_entry: None,
            unit_siz,
            item_cnt: 0,
            merklesig: Box::new([]),
            merklesig_len: hash(&[&0i32.to_be_bytes()[..]]).len(),
            hash,
        }
    }

    //#### 检查输入的merklesig长度是否合法
    #[inline(always)]
    fn check_merklesig_len(&self, h: &[u8]) -> bool {
        h.len() == self.merklesig_len
    }

    ///- #: 全局根哈希
    #[inline(always)]
    pub fn glob_entry_merklesig(&self) -> Option<HashSig> {
        self.glob_entry.as_ref().map(|_| self.merklesig.clone())
    }

    ///#### 获取跳表的树高度
    #[inline(always)]
    pub fn height(&self) -> usize {
        let mut height = 0usize;
        if let Some(glob_entry) = self.glob_entry.as_ref() {
            height += 1;
            let mut glob_entry = Rc::clone(glob_entry);
            while let Some(l) = glob_entry.lower.as_ref() {
                height += 1;
                glob_entry = Rc::clone(l);
            }
        }

        height
    }

    ///#### 获取跳表的单元容量
    #[inline(always)]
    pub fn unit_siz(&self) -> usize {
        self.unit_siz
    }

    ///#### 获取跳表中所有元素的数量
    #[inline(always)]
    pub fn item_cnt(&self) -> usize {
        self.item_cnt
    }

    //#### 查询数据
    //- #: 成功返回目标节点指针，
    //失败返回错误原因(其中不存在的情况，返回可插入位置的Option<左兄弟指针>)
    fn get_inner(&self, key: &[u8]) -> Result<Rc<Node<V>>, XErr<V>> {
        if !self.check_merklesig_len(key) {
            return Err(XErr::HashLen);
        }
        if self.glob_entry.is_none() {
            return Err(XErr::NotExists(None));
        }

        let mut res = None;
        Self::get_inner_r(key, Rc::clone(&self.glob_entry.as_ref().unwrap()), &mut res);

        if let Some(n) = res {
            if key == &n.key[..] {
                return Ok(n);
            } else {
                return Err(XErr::NotExists(Some(n)));
            }
        } else {
            return Err(XErr::NotExists(None));
        }
    }

    //#### should be a tail-recursion
    //- @res[out]: 最终结果写出之地
    #[allow(clippy::never_loop)]
    fn get_inner_r(key: &[u8], node: Rc<Node<V>>, res: &mut Option<Rc<Node<V>>>) {
        let mut cur = node;
        let mut curkey = &cur.key[..];

        if key == curkey {
            *res = Some(cur);
            return;
        } else if key > curkey {
            'm: loop {
                while let Some(right) = Weak::upgrade(&cur.right) {
                    cur = right;
                    curkey = &cur.key[..];
                    if key == curkey {
                        *res = Some(cur);
                        return;
                    } else if key < curkey {
                        if let Some(lower) = cur.lower.as_ref() {
                            cur = Rc::clone(lower);
                            break 'm;
                        } else {
                            //查找失败返回其左兄弟
                            *res = cur.left.clone();
                            return;
                        }
                    }
                }

                //比当前层的所有元素都大
                if let Some(lower) = cur.lower.as_ref() {
                    cur = Rc::clone(lower);
                    break;
                } else {
                    return;
                }
            }
        } else {
            'n: loop {
                while let Some(left) = cur.left.as_ref() {
                    cur = Rc::clone(left);
                    curkey = &cur.key[..];
                    if key == curkey {
                        *res = Some(cur);
                        return;
                    } else if key > curkey {
                        if let Some(lower) = cur.lower.as_ref() {
                            cur = Rc::clone(lower);
                            break 'n;
                        } else {
                            //查找失败返回其左兄弟
                            *res = Some(cur);
                            return;
                        }
                    }
                }

                //比当前层的所有元素都小，由于所有层级的第一个节点都是相同的，
                //故此种情况即是查询失败，不需要进一步判断
                return;
            }
        }

        Self::get_inner_r(key, cur, res);
    }

    ///#### 查询数据
    pub fn get(&self, key: &[u8]) -> Option<V> {
        self.get_inner(key).map(|n| (*n.value).clone()).ok()
    }

    ///#### 删除数据，并按需调整整体的数据结构，若被删节点：
    ///1. 若左右兄弟皆为空，则说明删除的是节点总数为一的跳表的唯一节点，直接将根节点置空即可
    ///2. 若左兄弟为空，右兄弟不为空，说明删除的是首节点，只需调整右兄弟指针
    ///3. 若左兄弟不为空，右兄弟为空，说明删除的是末尾节点，只需调整左兄弟指针
    ///4. 若左右兄弟皆不为空，需同时调节左右兄弟指针
    ///5. 2、3、4三种情况，均需检查其是否是其父节点的长子，
    ///若是，则递归向上删除其所有父辈节点，并新建替代垂直线
    pub fn remove(&mut self, key: &[u8]) -> Result<Rc<Node<V>>, XErr<V>> {
        Ok(self.remove_inner(self.get_inner(key)?))
    }

    //#### same as remove()
    fn remove_inner(&mut self, node: Rc<Node<V>>) -> Rc<Node<V>> {
        let left = if let Some(l) = node.left.as_ref() {
            Some(Rc::clone(l))
        } else {
            None
        };
        let right = if let Some(r) = Weak::upgrade(&node.right) {
            Some(r)
        } else {
            None
        };

        let mut raw;
        if left.is_none() && right.is_none() {
            self.glob_entry = None;
        } else if left.is_none() && right.is_some() {
            let r = right.unwrap();
            raw = Rc::into_raw(r) as *mut Node<V>;
            unsafe {
                (*raw).left = None;
                Rc::from_raw(raw);
            }
        } else if left.is_some() && right.is_none() {
            let l = left.unwrap();
            raw = Rc::into_raw(l) as *mut Node<V>;
            unsafe {
                (*raw).right = Weak::new();
                Rc::from_raw(raw);
            }
        } else {
            let mut l = left.unwrap();
            let r = right.unwrap();
            raw = Rc::into_raw(l) as *mut Node<V>;
            unsafe {
                (*raw).right = Rc::downgrade(&r);
                l = Rc::from_raw(raw);
            }
            raw = Rc::into_raw(r) as *mut Node<V>;
            unsafe {
                (*raw).left = Some(l);
                Rc::from_raw(raw);
            }
        }

        //被删除的节点，此时仍然保持其原先的上下左右连接关系
        //直接基于被删节点进行重塑即可
        self.restruct_remove(Rc::clone(&node));

        //重塑merkle proof hashsig
        self.merkle_refresh(Rc::clone(&node));

        self.item_cnt -= 1;
        node
    }

    ///#### 插入数据，并按需调整整体的数据结构
    ///- 目标已存在，且键值均相同，视为成功，否则返回哈希碰撞错误
    ///- 目标不存在，若存在左兄弟，则在其右侧插入新节点，否则插入为全局第一个元素
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
                    if let Some(right) = Weak::upgrade(&n.right) {
                        new = Rc::new(Node {
                            key: Rc::new(sig.clone()),
                            value: Rc::new(value),
                            merklesig: sig.clone(),
                            lower: None,
                            upper: Weak::upgrade(&n.upper)
                                .map(|u| Rc::downgrade(&u))
                                .unwrap_or_default(),
                            left: Some(Rc::clone(&n)),
                            right: Rc::downgrade(&right),
                        });

                        raw = Rc::into_raw(Rc::clone(&n)) as *mut Node<V>;
                        unsafe {
                            (*raw).right = Rc::downgrade(&new);
                            Rc::from_raw(raw);
                        }

                        raw = Rc::into_raw(Rc::clone(&right)) as *mut Node<V>;
                        unsafe {
                            (*raw).left = Some(right);
                            Rc::from_raw(raw);
                        }
                    } else {
                        new = Rc::new(Node {
                            key: Rc::new(sig.clone()),
                            value: Rc::new(value),
                            merklesig: sig.clone(),
                            lower: None,
                            upper: Weak::upgrade(&n.upper)
                                .map(|u| Rc::downgrade(&u))
                                .unwrap_or_default(),
                            left: Some(Rc::clone(&n)),
                            right: Weak::new(),
                        });

                        raw = Rc::into_raw(Rc::clone(&n)) as *mut Node<V>;
                        unsafe {
                            (*raw).right = Rc::downgrade(&new);
                            Rc::from_raw(raw);
                        }
                    }

                    //重塑跳表结构
                    self.restruct_put(n);
                } else if self.glob_entry.is_some() {
                    let mut lowest = self.get_lowest_first_node().unwrap();
                    new = Rc::new(Node {
                        key: Rc::new(sig.clone()),
                        value: Rc::new(value),
                        merklesig: sig.clone(),
                        lower: None,
                        upper: Weak::new(),
                        left: None,
                        right: Rc::downgrade(&lowest),
                    });

                    raw = Rc::into_raw(lowest) as *mut Node<V>;
                    unsafe {
                        (*raw).left = Some(Rc::clone(&new));
                        lowest = Rc::from_raw(raw);
                    }

                    let mut uppest = lowest;
                    let mut uppest_new;
                    let mut rightest;
                    let mut glob_entry = Rc::clone(&new);
                    while let Some(u) = Weak::upgrade(&uppest.upper) {
                        uppest_new = Rc::new(Node {
                            key: Rc::clone(&glob_entry.key),
                            value: Rc::clone(&glob_entry.value),
                            merklesig: Box::new([]), //将在restruct_put中刷新
                            lower: Some(Rc::clone(&glob_entry)),
                            upper: Weak::new(),
                            left: None,
                            right: Weak::upgrade(&u.right)
                                .map(|r| Rc::downgrade(&r))
                                .unwrap_or_default(),
                        });

                        raw = Rc::into_raw(glob_entry) as *mut Node<V>;
                        unsafe {
                            (*raw).upper = Rc::downgrade(&uppest_new);
                            glob_entry = Rc::from_raw(raw);
                        }

                        rightest = Weak::upgrade(&glob_entry.right)
                            .map(|r| Rc::downgrade(&r))
                            .unwrap_or_default();
                        while let Some(mut r) = Weak::upgrade(&rightest) {
                            if self.is_first_node(Rc::clone(&r)) {
                                break;
                            }

                            raw = Rc::into_raw(r) as *mut Node<V>;
                            unsafe {
                                (*raw).upper = Rc::downgrade(&uppest_new);
                                r = Rc::from_raw(raw);
                            }
                            rightest = Rc::downgrade(&r);
                        }

                        uppest = u;
                        glob_entry = uppest_new;
                    }

                    self.glob_entry = Some(glob_entry);

                    self.restruct_put(Rc::clone(&new));
                } else {
                    new = Rc::new(Node {
                        key: Rc::new(sig.clone()),
                        value: Rc::new(value),
                        merklesig: sig.clone(),
                        lower: None,
                        upper: Weak::new(),
                        left: None,
                        right: Weak::new(),
                    });
                    self.glob_entry = Some(Rc::clone(&new));
                }

                //重塑merkle proof hashsig
                self.merkle_refresh(new);

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
        if self.glob_entry.is_none() {
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

        let res = path
            .last()
            .map(|p| {
                (self.hash)(
                    p.merklesigs
                        .iter()
                        .map(|h| &h[..])
                        .collect::<Vec<&[u8]>>()
                        .as_slice(),
                )
            })
            .unwrap_or_else(|| Box::new([]));

        Ok(self.merklesig == res)
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

    //#### should be a tail-recursion
    //- @cur[in]: 当前节点
    //- @path[out]: 从叶到根的順序写出结果
    fn get_proof_path_r(&self, cur: Rc<Node<V>>, path: &mut Vec<ProofPath>) {
        let unit = Self::self_unit(Rc::clone(&cur));
        let sigs = unit
            .iter()
            .map(|i| i.merklesig.clone())
            .collect::<Vec<HashSig>>();
        path.push(ProofPath {
            selfidx: sigs.binary_search(&cur.merklesig).unwrap(),
            merklesigs: sigs,
        });

        if let Some(u) = Weak::upgrade(&cur.upper) {
            self.get_proof_path_r(u, path);
        } else {
            return;
        }
    }

    //#### 新增节点后，
    //- 递归向上调整跳表结构，递归至最顶层时，检查是否有需要分裂的满员单元
    //- 刷新merkle proof hashsig
    //- should be a tail-recursion
    fn restruct_put(&mut self, node: Rc<Node<V>>) {
        let unit = Self::self_unit(Rc::clone(&node));

        //满员则执行单元分裂
        if self.unit_siz == unit.len() {
            //跳表初始化时，已保证self.unit_siz >= 2
            let a = Rc::clone(&unit[0]); //等同于self.glob_entry.unwrap()
            let b = Rc::clone(&unit[self.unit_siz / 2]);

            if let Some(u) = Weak::upgrade(&node.upper) {
                let new = Rc::new(Node {
                    key: Rc::clone(&b.key),
                    value: Rc::clone(&b.value),
                    merklesig: Box::new([]), //will be refreshed by another function
                    lower: Some(Rc::clone(&b)),
                    upper: Weak::upgrade(&u.upper)
                        .map(|u| Rc::downgrade(&u))
                        .unwrap_or_default(),
                    left: Some(Rc::clone(&u)),
                    right: Weak::upgrade(&u.right)
                        .map(|r| Rc::downgrade(&r))
                        .unwrap_or_default(),
                });

                let raw = Rc::into_raw(u) as *mut Node<V>;
                unsafe {
                    (*raw).right = Rc::downgrade(&new);
                    Rc::from_raw(raw);
                }

                self.restruct_put(new);
            } else {
                let glob_entry = Rc::new(Node {
                    key: Rc::clone(&a.key),
                    value: Rc::clone(&a.value),
                    merklesig: Box::new([]), //will be refreshed by another function
                    lower: Some(Rc::clone(&a)),
                    upper: Weak::new(),
                    left: None,
                    right: Weak::new(),
                });

                let mut raw;
                raw = Rc::into_raw(a) as *mut Node<V>;
                unsafe {
                    (*raw).upper = Rc::downgrade(&glob_entry);
                    Rc::from_raw(raw);
                }
                raw = Rc::into_raw(b) as *mut Node<V>;
                unsafe {
                    (*raw).upper = Rc::downgrade(&glob_entry);
                    Rc::from_raw(raw);
                }

                self.glob_entry = Some(glob_entry);
            }
        }
    }

    //#### 删除节点后，
    //- 递归向上调整跳表结构，递归至最顶层时，检查是否可以降低树高度
    //- 刷新merkle proof hashsig
    //- should be a tail-recursion
    fn restruct_remove(&mut self, node: Rc<Node<V>>) {
        if let Some(u) = Weak::upgrade(&node.upper) {
            if self.is_first_node(Rc::clone(&node)) {
                self.remove_inner(Rc::clone(&u));
                self.restruct_remove(u);
            }

            let adj = Self::adjacent_statistics(node);
            let standard = self.unit_siz / 2;

            //本单元及其所有非空邻接单元，必然存在父节点
            if standard > adj.self_unit.len() {
                let upper;
                if !adj.left_unit.is_empty() && standard > adj.left_unit.len() {
                    upper = Weak::upgrade(&adj.self_unit[0].right).unwrap();
                } else if !adj.right_unit.is_empty() && standard > adj.right_unit.len() {
                    upper = Weak::upgrade(&adj.right_unit[0].right).unwrap();
                } else {
                    return;
                }

                self.remove_inner(Rc::clone(&upper));
                self.restruct_remove(upper);
            }
        } else {
            //顶层除根结点外，还存在其它结点，则不需要降低树高度
            if Weak::upgrade(&self.glob_entry.as_ref().unwrap().right).is_some() {
                return;
            }

            let mut glob_entry = Rc::clone(&self.glob_entry.as_ref().unwrap());
            while let Some(l) = glob_entry.lower.as_ref() {
                //沿根节点垂直向下的所有节点，其左单元一定为空，无须判断
                if 1 != Self::self_unit(Rc::clone(l)).len()
                    || !Self::right_unit(Rc::clone(l)).is_empty()
                {
                    glob_entry = Rc::clone(l);
                    break;
                }
            }

            //处理新的顶层结元
            let mut n = Rc::clone(&glob_entry);
            let mut raw = Rc::into_raw(n) as *mut Node<V>;
            unsafe {
                (*raw).upper = Weak::new();
                n = Rc::from_raw(raw);
            }
            while let Some(r) = Weak::upgrade(&n.right) {
                raw = Rc::into_raw(r) as *mut Node<V>;
                unsafe {
                    (*raw).upper = Weak::new();
                    n = Rc::from_raw(raw);
                }
            }

            self.glob_entry = Some(glob_entry);
            return;
        }
    }

    ///#### 基于最底一层对其上各层进行彻底重塑，
    ///- 对于长期运行的应用，可在业务闲时调用此函数，优化整体性能
    pub fn restruct_all(&mut self) {
        if let Some(lowest) = self.get_lowest_first_node() {
            let mut optim_siz = self.unit_siz / 2;
            if optim_siz < 2 {
                optim_siz = 2;
            }

            let mut sigbuf = Vec::with_capacity(optim_siz);
            self.restruct_all_inner(lowest, optim_siz, &mut sigbuf);
        }
    }

    //#### should be a tail-recursion
    //- @head[in]: 每一层的首节点
    //- @optim_siz[in]: 优化的目标单元大小
    //- @sigbug[in]: 缓存区复用
    fn restruct_all_inner(
        &mut self,
        head: Rc<Node<V>>,
        optim_siz: usize,
        sigbuf: &mut Vec<HashSig>,
    ) {
        let mut raw;
        let mut new;
        let mut node;
        let mut i = 0;
        let mut upper_len = 0;

        //处理上层的首节点
        node = head;
        sigbuf.push(node.merklesig.clone());
        i += 1;
        let upper_head = Rc::new(Node {
            key: Rc::clone(&node.key),
            value: Rc::clone(&node.value),
            merklesig: Box::new([]),
            lower: Some(Rc::clone(&node)),
            upper: Weak::new(),
            left: None,
            right: Weak::new(),
        });
        let mut upper_tail = Rc::clone(&upper_head);
        upper_len += 1;

        raw = Rc::into_raw(node) as *mut Node<V>;
        unsafe {
            (*raw).upper = Rc::downgrade(&upper_tail);
            node = Rc::from_raw(raw);
        }

        //从左到右处理后续节点(若有)
        while let Some(r) = Weak::upgrade(&node.right) {
            node = r;
            sigbuf.push(node.merklesig.clone());
            i += 1;

            //添加上层节点
            if 1 == i {
                new = Rc::new(Node {
                    key: Rc::clone(&node.key),
                    value: Rc::clone(&node.value),
                    merklesig: Box::new([]),
                    lower: Some(Rc::clone(&node)),
                    upper: Weak::new(),
                    left: Some(Rc::clone(&upper_tail)),
                    right: Weak::new(),
                });
                upper_tail = Rc::clone(&new);
                upper_len += 1;
            }

            //更改本层节点的upper指针
            raw = Rc::into_raw(node) as *mut Node<V>;
            unsafe {
                (*raw).upper = Rc::downgrade(&upper_tail);
                node = Rc::from_raw(raw);
            }

            //为上层节点计算merkle hashsig
            if optim_siz == i {
                i = 0;
                raw = Rc::into_raw(upper_tail) as *mut Node<V>;
                unsafe {
                    (*raw).merklesig =
                        (self.hash)(&sigbuf.iter().map(|i| &i[..]).collect::<Vec<&[u8]>>()[..]);
                    upper_tail = Rc::from_raw(raw);
                }
                sigbuf.clear();
            }
        }

        //处理最后一个单元的merkle hashsig
        if 0 != i {
            raw = Rc::into_raw(upper_tail) as *mut Node<V>;
            unsafe {
                (*raw).merklesig =
                    (self.hash)(&sigbuf.iter().map(|i| &i[..]).collect::<Vec<&[u8]>>()[..]);
                Rc::from_raw(raw);
            }
            sigbuf.clear();
        }

        if upper_len < self.unit_siz {
            self.glob_entry = Some(upper_head);
            return;
        } else {
            self.restruct_all_inner(upper_head, optim_siz, sigbuf);
        }
    }

    //获取最底一层的首节点
    fn get_lowest_first_node(&self) -> Option<Rc<Node<V>>> {
        let mut glob_entry = Rc::clone(self.glob_entry.as_ref()?);
        while let Some(l) = glob_entry.lower.as_ref() {
            glob_entry = Rc::clone(l);
        }
        Some(glob_entry)
    }

    //#### 由下而上递归刷新merkle proof hashsig
    //- 根哈希需要特殊处理
    //- should be a tail-recursion
    fn merkle_refresh(&self, node: Rc<Node<V>>) {
        let unit = Self::self_unit(Rc::clone(&node));
        let sigs = unit
            .iter()
            .map(|i| &i.merklesig[..])
            .collect::<Vec<&[u8]>>();
        if let Some(mut u) = Weak::upgrade(&node.upper) {
            let raw = Rc::into_raw(u) as *mut Node<V>;
            unsafe {
                (*raw).merklesig = (self.hash)(&sigs.as_slice());
                u = Rc::from_raw(raw);
            }

            self.merkle_refresh(u);
        } else {
            let raw = Rc::into_raw(Rc::clone(self.glob_entry.as_ref().unwrap())) as *mut Node<V>;
            unsafe {
                (*raw).merklesig = (self.hash)(&sigs.as_slice());
                Rc::from_raw(raw);
            }

            return;
        }
    }

    //#### 根据给定的节点，统计其自身所在单元及左右相邻单元的节点指针集合
    fn adjacent_statistics(node: Rc<Node<V>>) -> Adjacency<V> {
        Adjacency {
            self_unit: Self::self_unit(Rc::clone(&node)),
            left_unit: Self::left_unit(Rc::clone(&node)),
            right_unit: Self::right_unit(node),
        }
    }

    //#### self helper for adjacent_statistics
    fn self_unit(node: Rc<Node<V>>) -> Vec<Rc<Node<V>>> {
        let mut res = vec![];
        if let Some(u) = Weak::upgrade(&node.upper) {
            let mut cur = Rc::clone(u.lower.as_ref().unwrap());
            res.push(Rc::clone(&cur));
            while let Some(r) = Weak::upgrade(&cur.right) {
                if Rc::ptr_eq(&r, Weak::upgrade(&r.upper).unwrap().lower.as_ref().unwrap()) {
                    break;
                }
                res.push(Rc::clone(&r));
                cur = r;
            }
        } else {
            let mut cur = node;
            while let Some(l) = cur.left.as_ref() {
                cur = Rc::clone(l);
            }
            while let Some(r) = Weak::upgrade(&cur.right) {
                res.push(Rc::clone(&r));
                cur = r;
            }
        }
        res
    }

    //#### left helper for adjacent_statistics
    fn left_unit(node: Rc<Node<V>>) -> Vec<Rc<Node<V>>> {
        let mut res = vec![];
        if let Some(u) = Weak::upgrade(&node.upper) {
            if let Some(l) = u.left.as_ref() {
                let mut cur = Rc::clone(l.lower.as_ref().unwrap());
                res.push(Rc::clone(&cur));
                while let Some(r) = Weak::upgrade(&cur.right) {
                    if Rc::ptr_eq(&r, Weak::upgrade(&r.upper).unwrap().lower.as_ref().unwrap()) {
                        break;
                    }
                    res.push(Rc::clone(&r));
                    cur = r;
                }
            }
        }

        //无父节点时，说明处于顶层，左右单元均为空
        res
    }

    //#### right helper for adjacent_statistics
    fn right_unit(node: Rc<Node<V>>) -> Vec<Rc<Node<V>>> {
        let mut res = vec![];
        if let Some(u) = Weak::upgrade(&node.upper) {
            if let Some(r) = Weak::upgrade(&u.right) {
                let mut cur = Rc::clone(r.lower.as_ref().unwrap());
                res.push(Rc::clone(&cur));
                while let Some(r) = Weak::upgrade(&cur.right) {
                    if Rc::ptr_eq(&r, Weak::upgrade(&r.upper).unwrap().lower.as_ref().unwrap()) {
                        break;
                    }
                    res.push(Rc::clone(&r));
                    cur = r;
                }
            }
        }

        //无父节点时，说明处于顶层，左右单元均为空
        res
    }

    #[inline(always)]
    fn is_first_node(&self, node: Rc<Node<V>>) -> bool {
        let glob_entry = if let Some(r) = self.glob_entry.as_ref() {
            r
        } else {
            return false;
        };

        if let Some(u) = Weak::upgrade(&node.upper) {
            Rc::ptr_eq(u.lower.as_ref().unwrap(), &node)
        } else {
            Rc::ptr_eq(&node, glob_entry)
        }
    }
}

//- @selfidx: 路径上的每个节点在所有兄弟节点中的索引
//- @merklesigs: 当前节点及其所有兄弟节点的哈希值的有序集合
struct ProofPath {
    selfidx: usize,
    merklesigs: Vec<HashSig>,
}

#[inline(always)]
fn sha256(item: &[&[u8]]) -> Box<[u8]> {
    use ring::digest::{Context, SHA256};

    let mut context = Context::new(&SHA256);
    for x in item {
        context.update(x);
    }
    context
        .finish()
        .as_ref()
        .iter()
        .cloned()
        .collect::<Box<[u8]>>()
}

#[cfg(test)]
mod test {
    macro_rules! source_type_test {
        ($name: ident, $type: ty) => {
            mod $name {
                use super::super::*;
                use rand::random;

                pub fn rand() -> Vec<impl AsBytes> {
                    let mut sample = vec![];
                    (0..500).for_each(|_| sample.push(random::<$type>()));
                    sample.sort();
                    sample.dedup();
                    sample
                }

                pub fn rand_box() -> Vec<impl AsBytes> {
                    let mut sample = vec![];
                    (0..500).for_each(|_| {
                        sample.push(
                            (0..10)
                                .into_iter()
                                .map(|_| random::<$type>())
                                .collect::<Box<[$type]>>(),
                        )
                    });
                    sample.sort();
                    sample.dedup();
                    sample
                }

                pub fn rand_vec() -> Vec<impl AsBytes> {
                    let mut sample = vec![];
                    (0..500).for_each(|_| {
                        sample.push(
                            (0..10)
                                .into_iter()
                                .map(|_| random::<$type>())
                                .collect::<Vec<$type>>(),
                        )
                    });
                    sample.sort();
                    sample.dedup();
                    sample
                }

                pub fn $name<T: AsBytes>(sample: Vec<T>) {
                    let mut hashsigs = vec![];
                    let mut sl = SkipList::default();

                    for v in sample.iter().cloned() {
                        hashsigs.push(sl.put(v).unwrap());
                    }

                    let mut node = sl.get_lowest_first_node().unwrap();
                    let mut hashsigs1 = vec![Rc::clone(&node.key)];
                    while let Some(r) = Weak::upgrade(&node.right) {
                        hashsigs1.push(Rc::clone(&r.key));
                        node = r;
                    }

                    assert_eq!(sample.len(), sl.item_cnt());
                    assert_eq!(hashsigs.len(), sl.item_cnt());
                    assert_eq!(hashsigs1.len(), sl.item_cnt());

                    dbg!(&hashsigs);
                    dbg!(&hashsigs1);

                    assert!(sl.glob_entry_merklesig().is_some());
                    for (v, h) in sample.iter().zip(hashsigs.iter()) {
                        assert_eq!(v, &sl.get(h).unwrap());
                        assert!(sl.proof(h).unwrap());
                    }
                }
            }

            #[test]
            fn $name() {
                let sample0 = $name::rand();
                let sample1 = $name::rand_box();
                let sample2 = $name::rand_vec();

                $name::$name(sample0);
                $name::$name(sample1);
                $name::$name(sample2);
            }
        };
    }

    source_type_test!(msl_char, char);
    source_type_test!(msl_u8, u8);
    source_type_test!(msl_u16, u16);
    source_type_test!(msl_u32, u32);
    source_type_test!(msl_u64, u64);
    source_type_test!(msl_u128, u128);
    source_type_test!(msl_usize, usize);
    source_type_test!(msl_i8, i8);
    source_type_test!(msl_i16, i16);
    source_type_test!(msl_i32, i32);
    source_type_test!(msl_i64, i64);
    source_type_test!(msl_i128, i128);
    source_type_test!(msl_isize, isize);
}
