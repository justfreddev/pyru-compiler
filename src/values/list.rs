use std::{ cell::RefCell, rc::Rc };

use crate::value::Value;

pub fn call_list_method(list: Rc<RefCell<Vec<Value>>>, method: &str, args: Vec<Value>) -> Value {
    let mut borrowed = list.borrow_mut();

    match method {
        "index" => {
            assert!(args.len() == 1, "index expects 1 argument");
            match borrowed.iter().position(|v| *v == args[0]) {
                Some(i) => Value::Num(i as f64),
                None => panic!("Item not found"),
            }
        }

        "insertAt" => {
            assert!(args.len() == 2, "insert_at expects 2 arguments");
            let idx = match args[0] {
                Value::Num(n) => n as usize,
                _ => panic!("insert_at index must be a number"),
            };
            borrowed.insert(idx, args[1].clone());
            Value::List(list.clone())
        }

        "len" => {
            assert!(args.is_empty(), "len expects no arguments");
            Value::Num(borrowed.len() as f64)
        }

        "pop" => {
            if args.is_empty() {
                borrowed.pop().unwrap_or(Value::Null)
            } else if args.len() == 1 {
                let idx = match args[0] {
                    Value::Num(n) => n as usize,
                    _ => panic!("pop index must be a number"),
                };
                if idx >= borrowed.len() {
                    Value::Null
                } else {
                    borrowed.remove(idx)
                }
            } else {
                panic!("pop expects 0 or 1 arguments");
            }
        }

        "push" => {
            assert!(args.len() == 1, "push expects 1 argument");
            borrowed.push(args[0].clone());
            Value::List(list.clone())
        }

        "remove" => {
            assert!(args.len() == 1, "remove expects 1 argument");
            let target = &args[0];
            let pos = borrowed.iter().position(|v| v == target);
            match pos {
                Some(idx) => {
                    borrowed.remove(idx);
                    Value::List(list.clone())
                }
                None => panic!("Item not found to remove"),
            }
        }

        "sort" => {
            assert!(args.is_empty(), "sort expects no arguments");
            let sorted = tim_sort(borrowed.clone());
            *borrowed = sorted;
            Value::List(list.clone())
        }

        _ => panic!("Unknown list method: {method}"),
    }
}

const THRESHOLD: f32 = 32.0;

fn tim_sort(mut list: Vec<Value>) -> Vec<Value> {
    let n = list.len();

    let mut run_length = calc_min_run(n as f32);

    for start in (0..n).step_by(run_length) {
        let end = std::cmp::min(start + run_length - 1, n - 1);
        list = insertion_sort(list, start, end);
    }

    if n <= 32 {
        return list;
    }

    while run_length < n {
        for left in (0..n).step_by(2 * run_length) {
            let mid = std::cmp::min(n - 1, left + run_length - 1);
            let right = std::cmp::min(n - 1, left + 2 * run_length - 1);

            if mid < right {
                list = merge_sort(list, left, mid, right);
            }
        }
        run_length *= 2;
    }
    return list;
}

fn calc_min_run(len: f32) -> usize {
    let mut run_len = len;
    let mut remainder: f32 = 0.0;
    while run_len > THRESHOLD {
        if run_len % 2.0 == 1.0 {
            remainder = 1.0;
        }
        run_len = run_len.floor() / 2.0;
    }

    return (run_len + remainder) as usize;
}

fn insertion_sort(mut list: Vec<Value>, left: usize, right: usize) -> Vec<Value> {
    for i in left + 1..=right {
        let mut j = i;
        while j > left && list[j] < list[j - 1] {
            list.swap(j, j - 1);
            j -= 1;
        }
    }
    return list;
}

fn merge_sort(mut list: Vec<Value>, l: usize, m: usize, r: usize) -> Vec<Value> {
    let left_len = m - l + 1;
    let right_len = r - m;

    let left = list[l..=m].to_vec();
    let right = list[m + 1..=r].to_vec();

    let mut i = 0;
    let mut j = 0;
    let mut k = l;

    while i < left_len && j < right_len {
        match (&left[i], &right[j]) {
            (Value::Num(n1), Value::Num(n2)) => {
                if n1 <= n2 {
                    list[k] = left[i].clone();
                    i += 1;
                } else {
                    list[k] = right[j].clone();
                    j += 1;
                }
            }
            (Value::Str(s1), Value::Str(s2)) => {
                if s1 <= s2 {
                    list[k] = left[i].clone();
                    i += 1;
                } else {
                    list[k] = right[j].clone();
                    j += 1;
                }
            }
            _ => panic!("Cannot compare list elements in merge sort"),
        }
    }

    while i < left_len {
        list[k] = left[i].clone();
        i += 1;
        k += 1;
    }

    while j < right_len {
        list[k] = right[j].clone();
        j += 1;
        k += 1;
    }

    return list;
}
