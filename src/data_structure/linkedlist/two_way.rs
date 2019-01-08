use std::{
    fmt::Display,
    ptr,
};

type SizType=u64;

struct List<T: Clone + Display> {
    len: SizType ,
    head: *mut Node<T>,
    tail: *mut Node<T>,
}

#[derive(Clone)]
struct Node<T: Clone + Display> {
    data: T,
    prev: *mut Node<T>,
    back: *mut Node<T>,
}

impl<T: Clone + Display> List<T> {
    pub fn new() -> List<T> {
        List {
            len: 0,
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    // 前向追加节点
    pub fn prevadd(&mut self, data: T) {
        let new = Box::into_raw(Box::new(
            Node{
                data,
                prev: ptr::null_mut(),
                back: self.head, 
            }));

        if 0 == self.len {
            self.tail = new;
            self.head = new;
        } else {
            unsafe { (*self.head).prev = new; }
            self.head = new;
        }

        self.len += 1;
    }

    // 后向追加节点
    pub fn backadd(&mut self, data: T) {
        let new = Box::into_raw(Box::new(
            Node{
                data,
                prev: self.tail, 
                back: ptr::null_mut(),
            }));

        if 0 == self.len {
            self.tail = new;
            self.head = new;
        } else {
            unsafe { (*self.tail).back = new };
            self.tail = new;
        }

        self.len += 1;
    }

    // 弹出最前面的节点
    pub fn prevpop(&mut self) -> Option<T> {
        if 0 == self.len {
            return None;
        } else if 1 == self.len {
            let keep = self.head;

            self.len -= 1;
            self.head = ptr::null_mut();
            self.tail = ptr::null_mut();

            unsafe { return Some(Box::<Node<T>>::from_raw(keep).data); }
        } else {
            let keep = self.head;

            self.len -= 1;
            unsafe {
                self.head = (*keep).back;
                (*self.head).prev = ptr::null_mut();
                return Some(Box::<Node<T>>::from_raw(keep).data);
            };
        }
    }

    // 弹出最后面的节点
    pub fn backpop(&mut self) -> Option<T> {
        if 0 == self.len {
            return None;
        } else if 1 == self.len {
            let keep = self.tail;

            self.len -= 1;
            self.head = ptr::null_mut();
            self.tail = ptr::null_mut();

            unsafe { return Some(Box::<Node<T>>::from_raw(keep).data); }
        } else {
            let keep = self.tail;

            self.len -= 1;
            unsafe {
                self.tail = (*keep).prev;
                (*self.tail).back = ptr::null_mut();
                return Some(Box::<Node<T>>::from_raw(keep).data);
            };
        }
    }

    pub fn len(&self) -> SizType {
        self.len
    }

    pub fn stringify(&self) -> String {
        let mut res = String::new();

        let mut p = self.tail;
        for _ in 0..self.len {
            unsafe {
                let Node{data, prev, back: _} = *Box::<Node<T>>::from_raw(p);
                res.push_str(&format!("{}==>", data));
                p = prev;
            }
        }

        res.push_str(&format!("Nil"));
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut list = List::new();
        for x in 0..=99 {
            list.prevadd(x);
        }
        assert_eq!(list.len, 100);

        for x in 1..=100 {
            list.backadd(-x);
        }
        assert_eq!(list.len, 200);

        assert_eq!(99, list.prevpop().unwrap());
        assert_eq!(98, list.prevpop().unwrap());
        assert_eq!(97, list.prevpop().unwrap());
        assert_eq!(-100, list.backpop().unwrap());
        assert_eq!(-99, list.backpop().unwrap());
        assert_eq!(-98, list.backpop().unwrap());
        assert_eq!(list.len, 194);

        println!("{}", list.stringify());
    }
}
