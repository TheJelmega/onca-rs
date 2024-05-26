use super::*;

#[test]
fn dynarr_new() {
    let arr = DynArr::<i32>::new();
    assert_eq!(arr.capacity(), 0);
    assert_eq!(arr.len(), 0);


    let arr = DynArr::<i32>::with_capacity(21);
    assert!(arr.capacity() >= 21);
    assert_eq!(arr.len(), 0);
}

#[test]
fn dynarr_reserve() {
    let mut arr = DynArr::<i32>::new();
    arr.reserve(21);
    assert!(arr.capacity() >= 21);

    let mut arr = DynArr::<i32>::new();
    arr.reserve_exact(21);
    assert!(arr.capacity() >= 21);

    let mut arr = DynArr::<i32>::new();
    assert!(matches!(arr.try_reserve(21), Ok(())));
    assert!(arr.capacity() >= 21);

    let mut arr = DynArr::<i32>::new();
    assert!(matches!(arr.try_reserve_exact(21), Ok(())));
    assert!(arr.capacity() >= 21);
}

#[test]
fn dynarr_push_and_access() {
    let mut arr = DynArr::<i32>::new();

    arr.push(42);
    assert!(arr.capacity() >= 1);
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0], 42);

    arr.push(84);
    assert!(arr.capacity() >= 2);
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[1], 84);

    let mut arr = dynarr![1, 2, 3];
    assert_eq!(arr.push_within_capacity(4), Err(4));
    arr.reserve(1);
    assert_eq!(arr.push_within_capacity(4), Ok(()));
    assert_eq!(arr[3], 4);
}

#[test]
fn dynarr_reserve_and_push() {
    let mut arr = DynArr::<i32>::new();
    arr.reserve(10);
    let old_cap = arr.capacity();

    arr.push(42);
    assert_eq!(arr.capacity(), old_cap);
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0], 42);

    arr.push(84);
    assert_eq!(arr.capacity(), old_cap);
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[1], 84);
}

#[test]
fn dynarr_resize() {
    let mut arr = dynarr![1, 2, 3];
    arr.resize(6, 5);
    assert_eq!(arr, [1, 2, 3, 5, 5, 5]);

    let mut arr = dynarr![1, 2, 3];
    arr.resize(2, 5);
    assert_eq!(arr, [1, 2]);

    
    let mut arr = dynarr![1, 2, 3];
    let mut i = 4;
    arr.resize_with(6, || { let res = i; i *= 2; res });
    assert_eq!(arr, [1, 2, 3, 4, 8, 16]);
}

#[test]
fn dynarr_from_array() {
    let arr = dynarr!["hello", "world", "!"];
    assert_eq!(arr, ["hello", "world", "!"]);
}

#[test]
fn dynarr_shrink() {
    let mut arr = dynarr![1, 2, 3, 4, 5];
    arr.reserve(20);
    let old_cap = arr.capacity();

    arr.shrink_to_fit();
    assert!(arr.capacity() < old_cap);


    let mut arr = dynarr![1, 2, 3, 4, 5];
    arr.reserve(20);
    let old_cap = arr.capacity();

    arr.shrink_to(6);
    assert!(arr.capacity() < old_cap);
}

#[test]
fn dynarr_truncate() {
    let mut arr = dynarr![1, 2, 3, 4, 5, 6];
    arr.truncate(3);
    assert_eq!(arr, [1, 2, 3]);
}

#[test]
fn dynarr_swap_remove() {
    let mut arr = dynarr![1, 2, 3, 4, 5, 6];
    arr.swap_remove(3);
    assert_eq!(arr, [1, 2, 3, 6, 5]);

    arr.swap_remove(1);
    assert_eq!(arr, [1, 5, 3, 6]);
}

#[test]
fn dynarr_insert() {
    let mut arr = dynarr![1, 2, 3, 4, 5, 6];
    arr.insert(2, 42);
    assert_eq!(arr, [1, 2, 42, 3, 4, 5, 6]);
    arr.insert(7, 84);
    assert_eq!(arr, [1, 2, 42, 3, 4, 5, 6, 84]);
}

#[test]
fn dynarr_remove() {
    let mut arr = dynarr![1, 2, 3, 4, 5, 6];
    arr.remove(2);
    assert_eq!(arr, [1, 2, 4, 5, 6]);
}

#[test]
fn dynarr_retain() {
    let mut arr = dynarr![1, 2, 3, 4, 5, 6];
    arr.retain(|x| x % 2 == 0);
    assert_eq!(arr, [2, 4, 6]);


    let mut arr = dynarr![1, 2, 3, 4, 5, 6];
    arr.retain_mut(|x| {
        let res = *x % 2 == 0;
        *x += 3;
        res
    });
    assert_eq!(arr, [5, 7, 9]);
}

#[test]
fn dynarr_dedup() {
    let mut arr = dynarr![10, 20, 21, 30, 20];
    arr.dedup_by_key(|val| *val / 10);
    assert_eq!(arr, [10, 20, 30, 20]);

    let mut arr = dynarr![10, 20, 21, 30, 20];
    arr.dedup_by(|b, a| *a / 10 == *b / 10);
    assert_eq!(arr, [10, 20, 30, 20]);

    let mut arr = dynarr![1, 2, 2, 3, 2];
    arr.dedup();
    assert_eq!(arr, [1, 2, 3, 2]);
}

#[test]
fn dynarr_pop() {
    let mut arr = dynarr![1, 2, 3];
    assert_eq!(arr.pop(), Some(3));
    assert_eq!(arr.pop(), Some(2));
    assert_eq!(arr.pop(), Some(1));
    assert_eq!(arr.pop(), None);

    let mut arr = dynarr![1, 2, 3];
    assert_eq!(arr.pop_if(|val| *val % 2 == 1), Some(3));
    assert_eq!(arr.pop_if(|val| *val % 2 == 1), None);
}

#[test]
fn dynarr_append() {
    let mut arr = dynarr![1, 2, 3];
    let mut arr2 = dynarr![4, 5, 6];

    arr.append(&mut arr2);
    assert_eq!(arr, [1, 2, 3, 4, 5, 6]);
    assert!(arr2.is_empty());
}

#[test]
fn dynarr_drain() {
    let mut arr = dynarr![1, 2, 3, 4];
    {
        let mut drain = arr.drain(1..3);
        
        assert_eq!(drain.next(), Some(2));
        assert_eq!(drain.next(), Some(3));
        assert_eq!(drain.next(), None);
    }
    assert_eq!(arr, [1, 4]);
}

#[test]
fn dynarr_clear() {
    let mut arr = dynarr![1, 2, 3, 4];
    arr.clear();
    assert!(arr.is_empty());
}

#[test]
fn dynarr_splitoff() {
    let mut arr = dynarr![1, 2, 3, 4, 5, 6];
    let other = arr.splitt_off(3);

    assert_eq!(arr, [1, 2, 3]);
    assert_eq!(other, [4, 5, 6]);
}

#[test]
fn dynarr_spare() {
    let mut arr = dynarr![1, 2, 3, 4, 5, 6];
    arr.truncate(3);
    assert_eq!(arr.spare_capacity_mut().len(), 3);
}


#[test]
fn dynarr_extend() {
    let mut arr = dynarr![1, 2, 3];
    arr.extend_from_slice(&[4, 5, 6]);
    assert_eq!(arr, [1, 2, 3, 4, 5, 6]);

    let mut arr = dynarr![1, 2, 3];
    arr.extend_from_array([4, 5, 6]);
    assert_eq!(arr, [1, 2, 3, 4, 5, 6]);

    let mut arr = dynarr![1, 2, 3, 4, 5];
    arr.extend_from_within(1..4);
    assert_eq!(arr, [1, 2, 3, 4, 5, 2, 3, 4]);

    let mut arr = dynarr![1, 2, 3];
    arr.extend_with(3, 5);
    assert_eq!(arr, [1, 2, 3, 5, 5, 5]);
    
    let mut arr = dynarr![1, 2, 3];
    arr.extend([4, 5, 6]);
    assert_eq!(arr, [1, 2, 3, 4, 5, 6]);

}

#[test]
fn dynarr_into_flattened() {
    let arr = dynarr![[1, 2], [3, 4], [5, 6]];
    let arr = arr.into_flattened();
    assert_eq!(arr, [1, 2, 3, 4, 5, 6]);
}

#[test]
fn dynarr_into_iter() {
    let arr = dynarr![0, 1, 2, 3, 4];
    for (idx, elem) in arr.into_iter().enumerate() {
        assert_eq!(idx as u32, elem);
    }
}

#[test]
fn dynarr_splice() {
    let mut arr = dynarr![1, 2, 3, 4];
    let new = [7, 8, 9];
    let u: DynArr<_> = arr.splice(1..3, new).collect();
    assert_eq!(arr, [1, 7, 8, 9, 4]);
    assert_eq!(u, [2, 3]);
}

#[test]
fn dynarr_extact_if() {
    let mut numbers = dynarr![1, 2, 3, 4, 5, 6, 8, 9, 11, 13, 14, 15];
    
    let evens = numbers.extract_if(|x| *x % 2 == 0).collect::<DynArr<_>>();
    let odds = numbers;
    
    assert_eq!(evens, [2, 4, 6, 8, 14]);
    assert_eq!(odds, [1, 3, 5, 9, 11, 13, 15]);
}