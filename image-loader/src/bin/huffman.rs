use std::{
    collections::HashMap,
    ops::{Add, Shl, ShlAssign},
};

pub fn main() {
    //                      0 0 0 111 111 110 110 100 1011 1010 0 0 0
    //                      0 0 0 111 11|1 110 110 1|00 1011 10|10 0 0 0
    let test_string = "aaabbssdkeaaa".to_string();

    huffman_encode(test_string);
}

fn huffman_encode(text: String) {
    let mut chars_queue = text
        .chars()
        .fold(HashMap::new(), |mut acc, c| {
            acc.entry(c).and_modify(|x| *x += 1).or_insert(1);
            acc
        })
        .into_iter()
        .map(|e| HuffmanNode::new(e.1, Some(e.0)))
        .collect::<Vec<HuffmanNode<char>>>();

    chars_queue.sort_by(|a, b| b.count.cmp(&a.count));

    let current = HuffmanNode::<char>::new(0, None);

    let test = create_huffman_tree(current, chars_queue);
    let mut chars_map = HashMap::new();

    traverse(Some(Box::new(test)), 0, &mut chars_map);

    println!("{:?}", chars_map);

    let mut byte_array = vec![];

    let mut temp: u8 = 0;
    let mut bits_filled = 0;

    let mut s = "".to_string();
    let mut length = 0;

    for c in text.chars() {
        let code = *chars_map.get(&c).unwrap() as u8;
        let size = (8 - code.leading_zeros()).max(1);

        if bits_filled + size <= 8 {
            let shift = code.leading_zeros() - bits_filled;
            temp |= code.checked_shl(shift).unwrap_or(0);
            bits_filled += size;
            length += size;
        } else {
            let shift = (bits_filled + size) % 8;
            temp |= code >> shift;
            byte_array.push(temp);
            temp = 0;
            temp |= code.checked_shl(8 - shift).unwrap_or(0);
            bits_filled = shift;
            length += shift;
        }

        s.push_str(format!("{:b}({c}) ", code).as_str());
    }

    byte_array.push(temp);

    println!("{}, {}", s, length);
    println!(
        "{:?}",
        byte_array
            .iter()
            .map(|x| format!("{:08b}", x).to_string())
            .collect::<Vec<String>>()
    );
}

fn traverse<T: std::fmt::Debug>(
    current: Option<Box<HuffmanNode<T>>>,
    code: usize,
    chars_map: &mut HashMap<T, usize>,
) where
    T: Eq,
    T: std::hash::Hash,
{
    if let Some(node) = current {
        if let Some(v) = node.value {
            // println!("{:?}: {:b}", v, code);
            chars_map.insert(v, code);
        }

        traverse(node.left, code << 1, chars_map);
        traverse(node.right, (code << 1) + 1, chars_map);
    }
}

fn create_huffman_tree<T: std::fmt::Debug>(
    mut current: HuffmanNode<T>,
    mut chars_queue: Vec<HuffmanNode<T>>,
) -> HuffmanNode<T> {
    if chars_queue.len() == 1 {
        return chars_queue.pop().unwrap();
    }

    current.left = chars_queue.pop().map(|x| Box::new(x));
    current.right = chars_queue.pop().map(|x| Box::new(x));

    current.count = current
        .left
        .as_ref()
        .map(|x| x.count)
        .unwrap_or(0)
        .add(current.right.as_ref().map(|x| x.count).unwrap_or(0));

    chars_queue.push(current);
    chars_queue.sort_by(|a, b| b.count.cmp(&a.count));

    return create_huffman_tree(HuffmanNode::new(0, None), chars_queue);
}

#[derive(Debug)]
struct HuffmanNode<T> {
    count: usize,
    value: Option<T>,
    left: Option<Box<HuffmanNode<T>>>,
    right: Option<Box<HuffmanNode<T>>>,
}

impl<T> HuffmanNode<T> {
    fn new(count: usize, value: Option<T>) -> Self {
        Self {
            count,
            value,
            left: None,
            right: None,
        }
    }
}
