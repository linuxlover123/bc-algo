use std::{
    fmt::Display,
    ptr,
};

type SizType=u64;

struct List<T: Clone + Display> {
    len: SizType ,
    head: *mut Node<T>,
}

#[derive(Clone)]
struct Node<T: Clone + Display> {
    data: T,
    back: *mut Node<T>,
}

impl<T: Clone + Display> List<T> {
    pub fn new() -> List<T> {
        List {
            len: 0,
            head: ptr::null_mut(),
        }
    }

    // 追加节点
    pub fn add(&mut self, data: T) {
        let new = Box::into_raw(Box::new(
            Node{
                data,
                back: self.head, 
            }));

        self.head = new;
        self.len += 1;
    }

    // 弹出最新的节点
    pub fn pop(&mut self) -> Option<T> {
        if 0 == self.len {
            return None;
        } else if 1 == self.len {
            let keep = self.head;

            self.len -= 1;
            self.head = ptr::null_mut();

            unsafe { return Some(Box::<Node<T>>::from_raw(keep).data); }
        } else {
            let keep = self.head;

            self.len -= 1;
            unsafe {
                self.head = (*keep).back;
                return Some(Box::<Node<T>>::from_raw(keep).data);
            };
        }
    }

    pub fn len(&self) -> SizType {
        self.len
    }

    pub fn stringify(&self) -> String {
        let mut res = String::new();

        let mut p = self.head;
        for _ in 0..self.len {
            unsafe {
                let Node{data, back} = *Box::<Node<T>>::from_raw(p);
                res.push_str(&format!("{}==>", data));
                p = back;
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
            list.add(x);
        }
        assert_eq!(list.len, 100);

        assert_eq!(Some(99), list.pop());
        assert_eq!(Some(98), list.pop());
        assert_eq!(Some(97), list.pop());
        assert_eq!(list.len, 97);

        for x in 0..97 {
            assert_eq!(Some(96 - x), list.pop());
        }
        assert_eq!(list.len, 0);
        assert_eq!(None, list.pop());

        println!("{}", list.stringify());
    }
}
