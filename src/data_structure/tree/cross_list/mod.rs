//! ## Cross List
//!
//! #### 算法说明
//! - 使用`提拉逻辑`实现的跳表(交叉链表).
//!
//! #### 应用场景
//! - 增删改查.
//!
//! #### 实现属性
//! - <font color=Green>√</font> 多线程安全
//! - <font color=Red>×</font> 无 unsafe 代码

mod error;
use error::XErr;
use std::rc::{Rc, Weak};

pub trait Key: Clone + Eq + PartialEq + Ord + PartialOrd {}

pub struct CrossList<K: Key, V: Clone> {
    unit_siz: usize,             //必须是偶数
    right_part_start_idx: usize, //必须保证:right_part_start_idx == unit_siz / 2

    kv_cnt: usize,                //全局key-value总数量
    root: Option<Rc<Node<K, V>>>, //根节点,全局入口节点
}

//节点删除时,必须从下而上逐一操作,否则将造成内存泄漏
pub struct Node<K: Key, V: Clone> {
    key: K,
    value: Rc<V>,

    left: Option<Rc<Node<K, V>>>,
    right: Option<Rc<Node<K, V>>>,
    upper: Option<Rc<Node<K, V>>>,
    children: Vec<Rc<Node<K, V>>>,
}

macro_rules! get {
    ($slice: expr, $idx: expr) => {
        unsafe { $slice.get_unchecked($idx) }
    };
    (@$slice: expr, $idx: expr) => {
        $slice.get_unchecked($idx)
    };
}

macro_rules! set {
    ($slice: expr, $idx: expr) => {
        unsafe { $slice.get_unchecked_mut($idx) }
    };
    (@$slice: expr, $idx: expr) => {
        $slice.get_unchecked_mut($idx)
    };
}

macro_rules! chg {
    ($($obj: expr, $field: tt, $value: expr), +) => {
        let mut raw;
        $(
            raw = Rc::into_raw($obj) as *mut Node<K, V>;
            unsafe {
                (*raw).$field = $value;
                $obj = Rc::from_raw(raw);
            }
        )+
    };
    ($(@$obj: expr, $field: tt, $value: expr), +) => {
        let mut raw;
        $(
            raw = Rc::into_raw($obj) as *mut Node<K, V>;
            unsafe {
                (*raw).$field = $value;
                Rc::from_raw(raw);
            }
        )+
    };
}

//`读`接口
impl<K: Key, V: Clone> CrossList<K, V> {
    #[inline(always)]
    pub fn unit_siz(&self) -> usize {
        self.unit_siz
    }

    #[inline(always)]
    pub fn kv_cnt(&self) -> usize {
        self.kv_cnt
    }

    //查询成功，返回目标节点指针；
    //查询失败，返回其左兄弟或右兄弟(若全局为空表，返回None)
    fn query(&self, key: &K) -> Result<Rc<Node<K, V>>, XErr<K, V>> {
        let mut node = if let Some(r) = self.root.as_ref() {
            let root = Rc::clone(r);
            if key == &root.key {
                return Ok(root);
            }
            root
        } else {
            return Err(XErr::NotExists(None));
        };

        loop {
            if *key > node.key {
                if node.right.is_none() {
                    if node.children.is_empty() {
                        return Err(XErr::NotExists(Some(node)));
                    } else {
                        //进入下一层继续查找
                        node = Rc::clone(&node.children[self.right_part_start_idx]);
                    }
                } else {
                    while let Some(right) = node.right.as_ref() {
                        if *key > right.key {
                            node = Rc::clone(right);
                        } else if key == &right.key {
                            return Ok(Rc::clone(right));
                        } else if right.children.is_empty() {
                            return Err(XErr::NotExists(Some(Rc::clone(right))));
                        } else {
                            //进入下一层继续查找
                            node = Rc::clone(&right.children[self.right_part_start_idx]);
                            break;
                        }
                    }
                }
            } else {
                if node.left.is_none() {
                    if node.children.is_empty() {
                        return Err(XErr::NotExists(Some(node)));
                    } else {
                        //进入下一层继续查找
                        node = Rc::clone(&node.children[self.right_part_start_idx]);
                    }
                } else {
                    while let Some(left) = node.left.as_ref() {
                        if *key < left.key {
                            node = Rc::clone(left);
                        } else if key == &left.key {
                            return Ok(Rc::clone(left));
                        } else if left.children.is_empty() {
                            return Err(XErr::NotExists(Some(Rc::clone(left))));
                        } else {
                            //进入下一层继续查找
                            node = Rc::clone(&left.children[self.right_part_start_idx]);
                            break;
                        }
                    }
                }
            }
        }
    }
}

//`写`接口
impl<K: Key, V: Clone> CrossList<K, V> {
    #[inline(always)]
    pub fn default() -> CrossList<K, V> {
        CrossList {
            unit_siz: 2,
            right_part_start_idx: 1,
            kv_cnt: 0,
            root: None,
        }
    }

    #[inline(always)]
    pub fn new(siz: usize) -> CrossList<K, V> {
        let mut siz = siz;

        //单元大小修正为>=2的偶数
        if 0 == siz {
            siz = 2;
        } else if 0 != siz % 2 {
            siz += 1;
        }

        CrossList {
            unit_siz: siz,
            right_part_start_idx: siz / 2,
            kv_cnt: 0,
            root: None,
        }
    }

    #[inline(always)]
    pub fn new_strict(siz: usize) -> Result<CrossList<K, V>, XErr<K, V>> {
        if 0 == siz {
            return Err(XErr::UnitSizIsZero);
        } else if 0 != siz % 2 {
            return Err(XErr::UnitSizNotEven);
        }

        Ok(CrossList {
            unit_siz: siz,
            right_part_start_idx: siz / 2,
            kv_cnt: 0,
            root: None,
        })
    }

    ///## 更新指定的key对应的value
    ///#### Error:
    ///- all same as query
    pub fn update(&mut self, key: &K, value: V) -> Result<(), XErr<K, V>> {
        let node = self.query(&key)?;
        let raw = Rc::into_raw(node) as *mut Node<K, V>;
        unsafe {
            (*raw).value = Rc::new(value);
            Rc::from_raw(raw);
        }
        Ok(())
    }

    ///## 移除指定的key/value
    ///#### Error:
    ///-
    pub fn remove(&mut self, key: &K) -> Result<Rc<Node<K, V>>, XErr<K, V>> {
        let mut raw;
        let mut node = self.query(&key)?;

        while let Some(upper) = node.upper.as_ref() {
            //TODO?
            //if node本身是散落节点 {
            //    直接删除之,将其左右邻直接串连在一起即可
            //} else if 左侧或右侧存在散落的节点 {
            //    1.删除node节点
            //    2.将左侧或右侧最近一个散落节点移入父节点的children中
            //    3.将node的父节点的新的children[self.right_part_start_idx]的key/value复制给其父节点
            //    (若未变动,则无需复制,直接结束即可),若其父节点也是父父节点的直连节点,
            //    则以同样的逻辑循环向上处理,直至条件不再满足或到达顶层
            //} else if 左右都不存在散落的节点 {
            //    1.删除node节点
            //    2.将node的所有原兄弟节点的父节点指针置为None
            //    3.将node的所有原兄弟节点指针复制出来,互相之间及与左右邻之间重新串连
            //    3.删除node的直辖父节点(递归调用本函数处理)
            //}
        }

        //检测是否需要处理项层节点
        if node.upper.is_none() {
            //根节点
            if Rc::ptr_eq(&node, self.root.as_ref().unwrap()) {
                self.root = node.right.clone();
            }
            //首层节点,但非根节点
            else {
                raw = Rc::into_raw(Rc::clone(node.left.as_ref().unwrap())) as *mut Node<K, V>;
                unsafe {
                    (*raw).right = node.right.clone();
                    Rc::from_raw(raw);
                }
                if let Some(right) = node.right.as_ref() {
                    raw = Rc::into_raw(Rc::clone(right)) as *mut Node<K, V>;
                    unsafe {
                        (*raw).left = node.left.clone();
                        Rc::from_raw(raw);
                    }
                }
            }
        }

        Ok(node)
    }

    ///> 插入新的key=>value
    pub fn insert(&mut self, key: K, value: V) -> Result<Rc<Node<K, V>>, XErr<K, V>> {
        let mut node = Rc::new(Node::new(key, value));

        //根节点为空的情况,会在ops_after_insert函数中处理
        if self.root.is_some() {
            let mut brother = match self.query(&node.key) {
                Ok(n) => return Err(XErr::AlreadyExists(n)),
                //非空跳表一定会返回目标节点的一个相邻节点
                Err(XErr::NotExists(n)) => n.unwrap(),
                Err(e) => return Err(e),
            };

            //query函数返回的Err中携带的节点,
            //必然位于最底一层,故新节点的children无需处理
            if brother.key > node.key {
                chg!(
                    node,
                    right,
                    Some(Rc::clone(&brother)),
                    node,
                    left,
                    brother.left.clone(),
                    brother,
                    left,
                    Some(Rc::clone(&node))
                );
                if let Some(left) = brother.left.as_ref() {
                    chg!(@Rc::clone(left), right, Some(Rc::clone(&node)));
                }
            } else {
                chg!(
                    node,
                    left,
                    Some(Rc::clone(&brother)),
                    node,
                    right,
                    brother.right.clone(),
                    brother,
                    right,
                    Some(Rc::clone(&node))
                );
                if let Some(right) = brother.right.as_ref() {
                    chg!(@Rc::clone(right), left, Some(Rc::clone(&node)));
                }
            }
        }

        self.ops_after_insert(Rc::clone(&node));
        return Ok(node);
    }

    //> **此时newnode的upper指针尚未更新**
    //- @node[in]: 新建的节点
    fn ops_after_insert(&mut self, node: Rc<Node<K, V>>) {
        if node.left.is_none() {
            if let Some(right) = node.right.as_ref() {
                if right.upper.is_none() {
                    //只有根节点的左邻和上邻可以同时为空
                    self.root = Some(Rc::clone(&node));
                }
            } else {
                //向空跳表中插入第一个节点,无需更多操作
                self.root = Some(node);
                return;
            }
        }

        let lu = node
            .left
            .as_ref()
            .and_then(|l| l.upper.as_ref())
            .map(|u| Rc::clone(u));
        let ru = node
            .right
            .as_ref()
            .and_then(|r| r.upper.as_ref())
            .map(|u| Rc::clone(u));

        let n = if lu.is_some()
            && ru.is_some()
            && Rc::ptr_eq(lu.as_ref().unwrap(), ru.as_ref().unwrap())
        {
            /* 左右邻的父节点都存在,且具有共同的父节点,
             * 则需要首先调整父节点的children,之后返回一个合适的孤儿节点
             */
            let upper = lu.unwrap();
            let idx = upper
                .children
                .binary_search_by(|n| n.key.cmp(&node.key))
                .unwrap_err();
            let raw = Rc::into_raw(upper) as *mut Node<K, V>;
            let n;

            //确保任一children不为空的节点,其key在children的keys中永远是`中值`,
            //children只有两种状态: 要么满员,要么为空
            if idx >= self.right_part_start_idx {
                unsafe {
                    n = (*raw).children.pop().unwrap(); //被弹出的节点不会被系统drop
                    (*raw).children.insert(idx, node);
                    Rc::from_raw(raw);
                }
            } else {
                unsafe {
                    n = Rc::clone(get!(@(*raw).children, 0));
                    for i in 1..self.right_part_start_idx {
                        *set!(@(*raw).children, i - 1) = Rc::clone(get!(@(*raw).children, i)); //被覆盖的首节点不会被系统drop
                    }
                    *set!(@(*raw).children, idx) = node;
                    Rc::from_raw(raw);
                }
            }
            n
        } else {
            node
        };

        self.push_up(n);
    }

    //> 在插入新节点后执行,
    //> 基于新产生的某个孤儿节点,搜索其左右方向的连续"无父"节点集合,
    //> 若集合数量已超过单元容量(== unit_siz + 1),则将中值节点提升到上一层
    fn push_up(&mut self, node: Rc<Node<K, V>>) {
        if let Some(mut head) = self.push_up_check(node) {
            let mut unit = Vec::with_capacity(self.unit_siz);
            for i in 0..self.right_part_start_idx {
                unit.push(head);
                head = get!(unit, i).right.clone().unwrap();
            }

            let mut mid = head;

            let mut head = mid.right.clone();
            for i in self.right_part_start_idx..self.unit_siz {
                unit.push(head.unwrap());
                head = get!(unit, i).right.clone();
            }

            chg!(
                @Rc::clone(get!(unit, self.right_part_start_idx)),
                left,
                Some(Rc::clone(get!(@unit, self.right_part_start_idx - 1))),
                @Rc::clone(get!(unit, self.right_part_start_idx - 1)),
                right,
                Some(Rc::clone(get!(@unit, self.right_part_start_idx)))
            );

            let midl = if let Some(left) = get!(unit, 0).left.as_ref() {
                if let Some(n) = left.upper.as_ref() {
                    chg!(@Rc::clone(n), right, Some(Rc::clone(&mid)));
                }
                left.upper.clone()
            } else {
                None
            };

            let midr = if let Some(right) = get!(unit, unit.len() - 1).right.as_ref() {
                if let Some(n) = right.upper.as_ref() {
                    chg!(@Rc::clone(n), left, Some(Rc::clone(&mid)));
                }
                right.upper.clone()
            } else {
                None
            };

            chg!(mid, left, midl, mid, right, midr, mid, children, unit);

            self.ops_after_insert(mid);
        }
    }

    //> 检测是否有节点需要被拉升到上一层
    fn push_up_check(&self, node: Rc<Node<K, V>>) -> Option<Rc<Node<K, V>>> {
        if node.upper.is_some() {
            //node本身不是孤儿节点,无需上拉
            return None;
        }

        let mut head = node;
        let mut tail = Rc::clone(&head);

        let mut left_cnter = 0;
        while let Some(left) = head.left.as_ref() {
            if left.upper.is_some() {
                break;
            }
            head = Rc::clone(left);
            left_cnter += 1;
        }

        let mut right_cnter = 0;
        while let Some(right) = tail.right.as_ref() {
            if right.upper.is_some() {
                break;
            }
            tail = Rc::clone(right);
            right_cnter += 1;
        }

        if self.unit_siz == (1 + 1/*self*/ + left_cnter + right_cnter) {
            return Some(head);
        }

        None
    }

    //> 在删除节点后,并且被删节点所在单元节点数已小于单元容量时执行,
    //> 基于被删节点,将其父节点接低到其下一层(即:先从其原属层删除,然后再插入到其下一层),
    //> 此时暂不转换Vec<_>为纯链表形式,若后续再从其中删一个节点,再行转换
    fn pull_down(&self) {}
}

//`读`接口
impl<K: Key, V: Clone> Node<K, V> {
    #[inline(always)]
    pub fn get_kv(&self) -> (&K, &V) {
        (&self.key, &*self.value)
    }

    #[inline(always)]
    pub fn at_top_layer(&self) -> bool {
        self.upper.is_none()
    }

    #[inline(always)]
    pub fn at_bottom_layer(&self) -> bool {
        self.children.is_empty()
    }

    #[inline(always)]
    pub fn is_leftmost_node(&self) -> bool {
        self.left.is_none()
    }

    #[inline(always)]
    pub fn is_rightmost_node(&self) -> bool {
        self.right.is_none()
    }
}

//`写`接口
impl<K: Key, V: Clone> Node<K, V> {
    #[inline(always)]
    fn new(k: K, v: V) -> Node<K, V> {
        Node {
            key: k,
            value: Rc::new(v),
            left: None,
            right: None,
            upper: None,
            children: Vec::with_capacity(0),
        }
    }
}
